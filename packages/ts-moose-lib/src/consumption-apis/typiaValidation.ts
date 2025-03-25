import ts, { factory } from "typescript";
import {
  avoidTypiaNameClash,
  isMooseFile,
  replaceProgram,
  typiaJsonSchemas,
} from "../compilerPluginHelper";

const iife = (statements: ts.Statement[]): ts.CallExpression =>
  factory.createCallExpression(
    factory.createParenthesizedExpression(
      factory.createArrowFunction(
        undefined,
        undefined,
        [],
        undefined,
        factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),
        factory.createBlock(statements, true),
      ),
    ),
    undefined,
    [],
  );

export const isCreateConsumptionApi = (
  node: ts.Node,
  checker: ts.TypeChecker,
): node is ts.CallExpression => {
  if (!ts.isCallExpression(node)) {
    return false;
  }

  const declaration: ts.Declaration | undefined =
    checker.getResolvedSignature(node)?.declaration;
  if (!declaration || !isMooseFile(declaration.getSourceFile())) {
    return false;
  }

  const { name } = checker.getTypeAtLocation(declaration).symbol;

  return name == "createConsumptionApi";
};

const getParamType = (
  funcType: ts.Type,
  checker: ts.TypeChecker,
): ts.TypeNode | undefined => {
  const sig = funcType.getCallSignatures()[0];
  const firstParam = sig && sig.getParameters()[0];
  const t = firstParam && checker.getTypeOfSymbol(firstParam);
  return t && checker.typeToTypeNode(t, undefined, undefined);
};

const typeToOutputSchema = (t: ts.Type, checker: ts.TypeChecker): ts.Type => {
  if (
    t.isIntersection() &&
    t.types.some((inner) => inner.getSymbol()?.name === "ResultSet")
  ) {
    const queryResultSymbol = t.getProperty("__query_result_t");
    if (queryResultSymbol) {
      return checker.getNonNullableType(
        checker.getTypeOfSymbol(queryResultSymbol),
      );
    } else {
      return checker.getAnyType();
    }
  } else if (
    t.getProperty("status") !== undefined &&
    t.getProperty("body") !== undefined
  ) {
    return checker.getTypeOfSymbol(t.getProperty("body")!);
  } else {
    return t;
  }
};

export const transformCreateConsumptionApi = (
  node: ts.Node,
  checker: ts.TypeChecker,
): ts.Node => {
  if (!isCreateConsumptionApi(node, checker)) {
    return node;
  }

  const handlerFunc = node.arguments[0];
  const paramType =
    (node.typeArguments && node.typeArguments[0]) ||
    getParamType(checker.getTypeAtLocation(handlerFunc), checker);

  if (paramType === undefined) {
    throw new Error("Unknown type for params");
  }

  const handlerType = checker.getTypeAtLocation(handlerFunc);
  const returnType = handlerType.getCallSignatures()[0]?.getReturnType();
  const awaitedType = checker.getAwaitedType(returnType) || returnType;

  const queryResultType = awaitedType.isUnion()
    ? factory.createUnionTypeNode(
        awaitedType.types.flatMap((t) => {
          const typeNode = checker.typeToTypeNode(
            typeToOutputSchema(t, checker),
            undefined,
            undefined,
          );
          return typeNode === undefined ? [] : [typeNode];
        }),
      )
    : checker.typeToTypeNode(
        typeToOutputSchema(awaitedType, checker),
        undefined,
        undefined,
      );

  const wrappedFunc = factory.createArrowFunction(
    undefined,
    undefined,
    [
      factory.createParameterDeclaration(
        undefined,
        undefined,
        factory.createIdentifier("params"),
        undefined,
        undefined,
        undefined,
      ),
      factory.createParameterDeclaration(
        undefined,
        undefined,
        factory.createIdentifier("utils"),
        undefined,
        undefined,
        undefined,
      ),
    ],
    undefined,
    factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),
    factory.createBlock(
      [
        factory.createVariableStatement(
          undefined,
          factory.createVariableDeclarationList(
            [
              factory.createVariableDeclaration(
                factory.createIdentifier("processedParams"),
                undefined,
                undefined,
                factory.createCallExpression(
                  factory.createIdentifier("assertGuard"),
                  undefined,
                  [
                    factory.createNewExpression(
                      factory.createIdentifier("URLSearchParams"),
                      undefined,
                      [factory.createIdentifier("params")],
                    ),
                  ],
                ),
              ),
            ],
            ts.NodeFlags.Const,
          ),
        ),
        factory.createReturnStatement(
          factory.createCallExpression(
            factory.createIdentifier("handlerFunc"),
            undefined,
            [
              factory.createIdentifier("processedParams"),
              factory.createIdentifier("utils"),
            ],
          ),
        ),
      ],
      true,
    ),
  );

  return iife([
    // const assertGuard = ____moose____typia.http.createAssertQuery<T>()
    factory.createVariableStatement(
      undefined,
      factory.createVariableDeclarationList(
        [
          factory.createVariableDeclaration(
            factory.createIdentifier("assertGuard"),
            undefined,
            undefined,
            factory.createCallExpression(
              factory.createPropertyAccessExpression(
                factory.createPropertyAccessExpression(
                  factory.createIdentifier(avoidTypiaNameClash),
                  factory.createIdentifier("http"),
                ),
                factory.createIdentifier("createAssertQuery"),
              ),
              [paramType],
              [],
            ),
          ),
        ],
        ts.NodeFlags.Const,
      ),
    ),
    // the user provided function
    // the Parameters of createConsumptionApi is a trick to avoid extra imports
    // const handlerFunc: Parameters<typeof createConsumptionApi<DailyActiveUsersParams>>[0] = async(...) => ...
    factory.createVariableStatement(
      undefined,
      factory.createVariableDeclarationList(
        [
          factory.createVariableDeclaration(
            factory.createIdentifier("handlerFunc"),
            undefined,
            factory.createIndexedAccessTypeNode(
              factory.createTypeReferenceNode(
                factory.createIdentifier("Parameters"),
                [
                  factory.createTypeQueryNode(
                    factory.createIdentifier("createConsumptionApi"),
                    [paramType],
                  ),
                ],
              ),
              factory.createLiteralTypeNode(factory.createNumericLiteral("0")),
            ),
            handlerFunc,
          ),
        ],
        ts.NodeFlags.Const,
      ),
    ),
    // const wrappedFunc = (params, utils) => {
    //   const processedParams = assertGuard(new URLSearchParams(params))
    //   return handlerFunc(params, utils)
    // }
    factory.createVariableStatement(
      undefined,
      factory.createVariableDeclarationList(
        [
          factory.createVariableDeclaration(
            factory.createIdentifier("wrappedFunc"),
            undefined,
            undefined,
            wrappedFunc,
          ),
        ],
        ts.NodeFlags.Const,
      ),
    ),
    // wrappedFunc["moose_input_schema"] = ____moose____typia.json.schemas<[T]>()
    factory.createExpressionStatement(
      factory.createBinaryExpression(
        factory.createElementAccessExpression(
          factory.createIdentifier("wrappedFunc"),
          factory.createStringLiteral("moose_input_schema"),
        ),
        factory.createToken(ts.SyntaxKind.EqualsToken),
        typiaJsonSchemas(paramType),
      ),
    ),

    factory.createExpressionStatement(
      factory.createBinaryExpression(
        factory.createElementAccessExpression(
          factory.createIdentifier("wrappedFunc"),
          factory.createStringLiteral("moose_output_schema"),
        ),
        factory.createToken(ts.SyntaxKind.EqualsToken),
        factory.createCallExpression(
          factory.createPropertyAccessExpression(
            factory.createPropertyAccessExpression(
              factory.createIdentifier(avoidTypiaNameClash),
              factory.createIdentifier("json"),
            ),
            factory.createIdentifier("schemas"),
          ),
          [
            factory.createTupleTypeNode([
              factory.createTypeReferenceNode(
                factory.createIdentifier("ReturnType"),
                [
                  factory.createTypeQueryNode(
                    factory.createIdentifier("handlerFunc"),
                  ),
                ],
              ),
            ]),
          ],
          [],
        ),
      ),
    ),

    factory.createReturnStatement(factory.createIdentifier("wrappedFunc")),
  ]);
};
