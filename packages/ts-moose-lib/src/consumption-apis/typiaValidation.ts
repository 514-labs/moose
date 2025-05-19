import ts, { factory } from "typescript";
import {
  avoidTypiaNameClash,
  isMooseFile,
  typiaJsonSchemas,
} from "../compilerPluginHelper";
import { toColumns } from "../dataModels/typeConvert";
import { parseAsAny } from "../dmv2/dataModelMetadata";

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

export const isCreateConsumptionApiV2 = (
  node: ts.Node,
  checker: ts.TypeChecker,
): node is ts.NewExpression => {
  if (!ts.isNewExpression(node)) {
    return false;
  }

  const declaration: ts.Declaration | undefined =
    checker.getResolvedSignature(node)?.declaration;
  if (!declaration || !isMooseFile(declaration.getSourceFile())) {
    return false;
  }

  const sym = checker.getSymbolAtLocation(node.expression);
  return sym?.name === "ConsumptionApi";
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
  if (isCreateConsumptionApi(node, checker)) {
    return transformLegacyConsumptionApi(node, checker);
  } else if (isCreateConsumptionApiV2(node, checker)) {
    return transformNewConsumptionApi(node as ts.NewExpression, checker);
  }

  return node;
};

export const transformLegacyConsumptionApi = (
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

  const queryResultType =
    awaitedType.isUnion() ?
      factory.createUnionTypeNode(
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

    // wrappedFunc["moose_output_schema"] = ____moose____typia.json.schemas<[Output]>()
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
              queryResultType ||
                factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword),
            ]),
          ],
          [],
        ),
      ),
    ),

    factory.createReturnStatement(factory.createIdentifier("wrappedFunc")),
  ]);
};

// TODO: When the legacy consumption api is removed, follow the args
// pattern in transformNewMooseResource
const transformNewConsumptionApi = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  if (!isCreateConsumptionApiV2(node, checker)) {
    return node;
  }

  if (!node.arguments || node.arguments.length < 2 || !node.typeArguments) {
    return node;
  }

  // Get both type parameters from ConsumptionApi<T, R>
  const typeNode = node.typeArguments[0];
  const responseTypeNode =
    node.typeArguments[1] ||
    factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword);

  // Get the handler function (second argument)
  const handlerFunc = node.arguments[1];

  // Create a new handler function that includes validation
  const wrappedHandler = factory.createArrowFunction(
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
                  [typeNode],
                  [],
                ),
              ),
            ],
            ts.NodeFlags.Const,
          ),
        ),
        // const searchParams = new URLSearchParams(params as any)
        factory.createVariableStatement(
          undefined,
          factory.createVariableDeclarationList(
            [
              factory.createVariableDeclaration(
                factory.createIdentifier("searchParams"),
                undefined,
                undefined,
                factory.createNewExpression(
                  factory.createIdentifier("URLSearchParams"),
                  undefined,
                  [
                    factory.createAsExpression(
                      factory.createIdentifier("params"),
                      factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword),
                    ),
                  ],
                ),
              ),
            ],
            ts.NodeFlags.Const,
          ),
        ),
        // const processedParams = assertGuard(searchParams)
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
                  [factory.createIdentifier("searchParams")],
                ),
              ),
            ],
            ts.NodeFlags.Const,
          ),
        ),
        // return originalHandler(processedParams, utils)
        factory.createReturnStatement(
          factory.createCallExpression(
            factory.createParenthesizedExpression(handlerFunc),
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

  // Create the schema arguments
  const inputSchemaArg =
    node.arguments.length > 3 ? node.arguments[3] : typiaJsonSchemas(typeNode);
  const responseSchemaArg = typiaJsonSchemas(responseTypeNode);

  // Create the columns argument if it doesn't exist
  const inputColumnsArg = toColumns(
    checker.getTypeAtLocation(typeNode),
    checker,
  );

  // --- Inject file location into config.metadata ---
  // Config is the third argument (index 2) if present, otherwise create one
  const fileUrl = node.getSourceFile().fileName;
  let configArg = node.arguments.length > 2 ? node.arguments[2] : undefined;
  let newConfigArg: ts.Expression;
  if (!configArg || !ts.isObjectLiteralExpression(configArg)) {
    newConfigArg = factory.createObjectLiteralExpression(
      [
        factory.createPropertyAssignment(
          "metadata",
          factory.createObjectLiteralExpression(
            [
              factory.createPropertyAssignment(
                "fileUrl",
                factory.createStringLiteral(fileUrl),
              ),
            ],
            false,
          ),
        ),
      ],
      false,
    );
  } else {
    let hasMetadata = false;
    const newProperties = configArg.properties.map((prop) => {
      if (
        ts.isPropertyAssignment(prop) &&
        ts.isIdentifier(prop.name) &&
        prop.name.text === "metadata" &&
        ts.isObjectLiteralExpression(prop.initializer)
      ) {
        hasMetadata = true;
        let hasFileUrl = false;
        const newMetaProps = prop.initializer.properties.map((metaProp) => {
          if (
            ts.isPropertyAssignment(metaProp) &&
            ts.isIdentifier(metaProp.name) &&
            metaProp.name.text === "fileUrl"
          ) {
            hasFileUrl = true;
            return factory.createPropertyAssignment(
              "fileUrl",
              factory.createStringLiteral(fileUrl),
            );
          }
          return metaProp;
        });
        const metaPropsWithFileUrl =
          hasFileUrl ? newMetaProps : (
            [
              ...newMetaProps,
              factory.createPropertyAssignment(
                "fileUrl",
                factory.createStringLiteral(fileUrl),
              ),
            ]
          );
        return factory.createPropertyAssignment(
          "metadata",
          factory.createObjectLiteralExpression(metaPropsWithFileUrl, false),
        );
      }
      return prop;
    });
    const finalProperties =
      hasMetadata ? newProperties : (
        [
          ...newProperties,
          factory.createPropertyAssignment(
            "metadata",
            factory.createObjectLiteralExpression(
              [
                factory.createPropertyAssignment(
                  "fileUrl",
                  factory.createStringLiteral(fileUrl),
                ),
              ],
              false,
            ),
          ),
        ]
      );
    newConfigArg = factory.createObjectLiteralExpression(
      finalProperties,
      false,
    );
  }

  // Update the ConsumptionApi constructor call with all necessary arguments
  return factory.updateNewExpression(
    node,
    node.expression,
    node.typeArguments,
    [
      node.arguments[0], // name
      wrappedHandler, // wrapped handler
      newConfigArg, // config object (with fileUrl in metadata)
      inputSchemaArg, // input schema
      parseAsAny(JSON.stringify(inputColumnsArg)), // input columns
      responseSchemaArg, // response schema
    ],
  );
};
