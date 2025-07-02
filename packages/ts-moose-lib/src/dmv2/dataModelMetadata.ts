import ts, { factory } from "typescript";
import {
  avoidTypiaNameClash,
  isMooseFile,
  typiaJsonSchemas,
} from "../compilerPluginHelper";
import { toColumns } from "../dataModels/typeConvert";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { dlqSchema } from "./internal";

const typesToArgsLength = new Map([
  ["OlapTable", 2],
  ["Stream", 2],
  ["DeadLetterQueue", 2],
  ["IngestPipeline", 2],
  ["IngestApi", 2],
  ["ConsumptionApi", 2],
  ["MaterializedView", 1],
  ["RefreshableMaterializedView", 1],
  ["Task", 2],
]);

export const isNewMooseResourceWithTypeParam = (
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
  const typeName = sym?.name ?? "";
  if (!typesToArgsLength.has(typeName)) {
    return false;
  }

  if (typeName === "Task") {
    return (
      node.arguments?.length === 2 &&
      (!node.typeArguments ||
        node.typeArguments.length === 0 ||
        node.typeArguments.length === 2)
    );
  }

  return (
    // name only
    (node.arguments?.length === 1 ||
      // config param
      node.arguments?.length === 2) &&
    node.typeArguments?.length === 1
  );
};

export const parseAsAny = (s: string) =>
  factory.createAsExpression(
    factory.createCallExpression(
      factory.createPropertyAccessExpression(
        factory.createIdentifier("JSON"),
        factory.createIdentifier("parse"),
      ),
      undefined,
      [factory.createStringLiteral(s)],
    ),
    factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword),
  );

const typiaTypeGuard = (node: ts.NewExpression) => {
  const typeNode = node.typeArguments![0];
  return factory.createCallExpression(
    factory.createPropertyAccessExpression(
      factory.createIdentifier(avoidTypiaNameClash),
      factory.createIdentifier("createAssert"),
    ),
    [typeNode],
    [],
  );
};

function taskInternalArgs(node: ts.NewExpression, checker: ts.TypeChecker) {
  // If no type arguments provided, treat as null, null
  const typeNodes = node.typeArguments || [
    factory.createLiteralTypeNode(factory.createNull()),
    factory.createLiteralTypeNode(factory.createNull()),
  ];
  const inputTypeNode = typeNodes[0];

  const inputSchema =
    checker.typeToString(checker.getTypeAtLocation(inputTypeNode)) === "null" ?
      factory.createObjectLiteralExpression([
        factory.createPropertyAssignment(
          factory.createIdentifier("version"),
          factory.createStringLiteral("3.1"),
        ),
        factory.createPropertyAssignment(
          factory.createIdentifier("components"),
          factory.createObjectLiteralExpression([
            factory.createPropertyAssignment(
              factory.createIdentifier("schemas"),
              factory.createObjectLiteralExpression([]),
            ),
          ]),
        ),
        factory.createPropertyAssignment(
          factory.createIdentifier("schemas"),
          factory.createArrayLiteralExpression([
            factory.createObjectLiteralExpression([
              factory.createPropertyAssignment(
                factory.createIdentifier("type"),
                factory.createStringLiteral("null"),
              ),
            ]),
          ]),
        ),
      ])
    : typiaJsonSchemas(inputTypeNode);

  return [
    inputSchema,
    checker.typeToString(checker.getTypeAtLocation(inputTypeNode)) === "null" ?
      factory.createArrayLiteralExpression([])
    : parseAsAny(
        JSON.stringify(
          toColumns(checker.getTypeAtLocation(inputTypeNode), checker),
        ),
      ),
  ];
}

export const transformNewMooseResource = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  const typeName = checker.getSymbolAtLocation(node.expression)!.name;

  // For Task, handle case where typeArguments is undefined
  const typeNode =
    typeName === "Task" && !node.typeArguments ?
      factory.createLiteralTypeNode(factory.createNull())
    : node.typeArguments![0];

  const internalArguments =
    typeName === "DeadLetterQueue" ? [typiaTypeGuard(node)]
    : typeName === "Task" ? taskInternalArgs(node, checker)
    : [
        typiaJsonSchemas(typeNode),
        parseAsAny(
          JSON.stringify(
            toColumns(checker.getTypeAtLocation(typeNode), checker),
          ),
        ),
      ];
  const resourceName = checker.getSymbolAtLocation(node.expression)!.name;

  const argLength = typesToArgsLength.get(resourceName)!;
  const needsExtraArg = node.arguments!.length === argLength - 1; // provide empty config if undefined

  let updatedArgs = [
    ...node.arguments!,
    ...(needsExtraArg ?
      [factory.createObjectLiteralExpression([], false)]
    : []),
    ...internalArguments,
  ];

  // For OlapTable and IngestPipeline, also inject typia validation functions
  if (resourceName === "OlapTable" || resourceName === "IngestPipeline") {
    // Create a single TypiaValidators object with all three validation functions
    const validatorsObject = factory.createObjectLiteralExpression(
      [
        factory.createPropertyAssignment(
          factory.createIdentifier("validate"),
          createTypiaValidator(typeNode),
        ),
        factory.createPropertyAssignment(
          factory.createIdentifier("assert"),
          createTypiaAssert(typeNode),
        ),
        factory.createPropertyAssignment(
          factory.createIdentifier("is"),
          createTypiaIs(typeNode),
        ),
      ],
      true,
    );

    updatedArgs = [...updatedArgs, validatorsObject];
  }

  return ts.factory.updateNewExpression(
    node,
    node.expression,
    node.typeArguments,
    updatedArgs,
  );
};

/**
 * Creates a typia validator function call for the given type
 * e.g., ____moose____typia.createValidate<T>()
 */
export const createTypiaValidator = (typeNode: ts.TypeNode) => {
  // Create the typia validator call
  const typiaValidator = factory.createCallExpression(
    factory.createPropertyAccessExpression(
      factory.createIdentifier(avoidTypiaNameClash),
      factory.createIdentifier("createValidate"),
    ),
    [typeNode],
    [],
  );

  // Wrap it to transform the result to match our expected interface
  // (data: unknown) => {
  //   const result = typiaValidator(data);
  //   return {
  //     success: result.success,
  //     data: result.success ? result.data : undefined,
  //     errors: result.success ? undefined : result.errors
  //   };
  // }
  return factory.createArrowFunction(
    undefined,
    undefined,
    [
      factory.createParameterDeclaration(
        undefined,
        undefined,
        factory.createIdentifier("data"),
        undefined,
        factory.createKeywordTypeNode(ts.SyntaxKind.UnknownKeyword),
        undefined,
      ),
    ],
    undefined,
    factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),
    factory.createBlock(
      [
        // const result = typiaValidator(data);
        factory.createVariableStatement(
          undefined,
          factory.createVariableDeclarationList(
            [
              factory.createVariableDeclaration(
                factory.createIdentifier("result"),
                undefined,
                undefined,
                factory.createCallExpression(typiaValidator, undefined, [
                  factory.createIdentifier("data"),
                ]),
              ),
            ],
            ts.NodeFlags.Const,
          ),
        ),
        // return { success: result.success, data: result.success ? result.data : undefined, errors: result.success ? undefined : result.errors };
        factory.createReturnStatement(
          factory.createObjectLiteralExpression(
            [
              factory.createPropertyAssignment(
                factory.createIdentifier("success"),
                factory.createPropertyAccessExpression(
                  factory.createIdentifier("result"),
                  factory.createIdentifier("success"),
                ),
              ),
              factory.createPropertyAssignment(
                factory.createIdentifier("data"),
                factory.createConditionalExpression(
                  factory.createPropertyAccessExpression(
                    factory.createIdentifier("result"),
                    factory.createIdentifier("success"),
                  ),
                  factory.createToken(ts.SyntaxKind.QuestionToken),
                  factory.createPropertyAccessExpression(
                    factory.createIdentifier("result"),
                    factory.createIdentifier("data"),
                  ),
                  factory.createToken(ts.SyntaxKind.ColonToken),
                  factory.createIdentifier("undefined"),
                ),
              ),
              factory.createPropertyAssignment(
                factory.createIdentifier("errors"),
                factory.createConditionalExpression(
                  factory.createPropertyAccessExpression(
                    factory.createIdentifier("result"),
                    factory.createIdentifier("success"),
                  ),
                  factory.createToken(ts.SyntaxKind.QuestionToken),
                  factory.createIdentifier("undefined"),
                  factory.createToken(ts.SyntaxKind.ColonToken),
                  factory.createPropertyAccessExpression(
                    factory.createIdentifier("result"),
                    factory.createIdentifier("errors"),
                  ),
                ),
              ),
            ],
            true,
          ),
        ),
      ],
      true,
    ),
  );
};

/**
 * Creates a typia assert function call for the given type
 * e.g., ____moose____typia.createAssert<T>()
 */
export const createTypiaAssert = (typeNode: ts.TypeNode) =>
  factory.createCallExpression(
    factory.createPropertyAccessExpression(
      factory.createIdentifier(avoidTypiaNameClash),
      factory.createIdentifier("createAssert"),
    ),
    [typeNode],
    [],
  );

/**
 * Creates a typia is function call for the given type
 * e.g., ____moose____typia.createIs<T>()
 */
export const createTypiaIs = (typeNode: ts.TypeNode) =>
  factory.createCallExpression(
    factory.createPropertyAccessExpression(
      factory.createIdentifier(avoidTypiaNameClash),
      factory.createIdentifier("createIs"),
    ),
    [typeNode],
    [],
  );
