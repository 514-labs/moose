import ts, { factory } from "typescript";
import {
  avoidTypiaNameClash,
  isMooseFile,
  typiaJsonSchemas,
} from "../compilerPluginHelper";
import { toColumns } from "../dataModels/typeConvert";

const typesToArgsLength = new Map([
  ["OlapTable", 2],
  ["Stream", 2],
  ["DeadLetterQueue", 2],
  ["IngestPipeline", 2],
  ["IngestApi", 2],
  ["ConsumptionApi", 2],
  ["MaterializedView", 1],
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
  if (!typesToArgsLength.has(sym?.name ?? "")) {
    return false;
  }

  return (
    // name only
    (node.arguments?.length === 1 ||
      // config param
      node.arguments?.length === 2) &&
    // TODO: Is this okay? Should I add a new check?
    (node.typeArguments?.length === 1 || node.typeArguments?.length === 2)
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

const deadLetterQueueInternalArgs = (node: ts.NewExpression) => {
  const typeNode = node.typeArguments![0];
  return [
    parseAsAny(
      // DeadLetterModel might not be in scope
      // including the plain data here to avoid headache
      JSON.stringify({
        version: "3.1",
        components: {
          schemas: {
            DeadLetterModel: {
              type: "object",
              properties: {
                originalRecord: {
                  $ref: "#/components/schemas/Recordstringany",
                },
                errorMessage: {
                  type: "string",
                },
                errorType: {
                  type: "string",
                },
                failedAt: {
                  type: "string",
                  format: "date-time",
                },
                source: {
                  oneOf: [
                    {
                      const: "api",
                    },
                    {
                      const: "transform",
                    },
                    {
                      const: "table",
                    },
                  ],
                },
              },
              required: [
                "originalRecord",
                "errorMessage",
                "errorType",
                "failedAt",
                "source",
              ],
            },
            Recordstringany: {
              type: "object",
              properties: {},
              required: [],
              description:
                "Construct a type with a set of properties K of type T",
              additionalProperties: {},
            },
          },
        },
        schemas: [
          {
            $ref: "#/components/schemas/DeadLetterModel",
          },
        ],
      }),
    ),
    parseAsAny(
      JSON.stringify([
        {
          name: "originalRecord",
          data_type: "Json",
          primary_key: false,
          required: true,
          unique: false,
          default: null,
          annotations: [],
        },
        {
          name: "errorMessage",
          data_type: "String",
          primary_key: false,
          required: true,
          unique: false,
          default: null,
          annotations: [],
        },
        {
          name: "errorType",
          data_type: "String",
          primary_key: false,
          required: true,
          unique: false,
          default: null,
          annotations: [],
        },
        {
          name: "failedAt",
          data_type: "DateTime",
          primary_key: false,
          required: true,
          unique: false,
          default: null,
          annotations: [],
        },
        {
          name: "source",
          data_type: "String",
          primary_key: false,
          required: true,
          unique: false,
          default: null,
          annotations: [],
        },
      ]),
    ),
    factory.createCallExpression(
      factory.createPropertyAccessExpression(
        factory.createIdentifier(avoidTypiaNameClash),
        factory.createIdentifier("createAssert"),
      ),
      [typeNode],
      [],
    ),
  ];
};

export const transformNewMooseResource = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  const typeName = checker.getSymbolAtLocation(node.expression)!.name;

  const typeNode = node.typeArguments![0];
  const internalArguments =
    typeName === "DeadLetterQueue" ?
      deadLetterQueueInternalArgs(node)
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
