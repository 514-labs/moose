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

export const transformNewMooseResource = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  const typeName = checker.getSymbolAtLocation(node.expression)!.name;

  const typeNode = node.typeArguments![0];
  const internalArguments =
    typeName === "DeadLetterQueue" ?
      [typiaTypeGuard(node)]
    : [
        typiaJsonSchemas(typeNode),
        parseAsAny(
          JSON.stringify(
            toColumns(checker.getTypeAtLocation(typeNode), checker),
          ),
        ),
      ];
  if (typeName === "IngestPipeline") {
    internalArguments.push(typiaTypeGuard(node));
  }

  const argLength = typesToArgsLength.get(typeName)!;
  const needsExtraArg = node.arguments!.length === argLength - 1; // provide empty config if undefined

  const updatedArgs = [
    ...node.arguments!,
    ...(needsExtraArg ?
      [factory.createObjectLiteralExpression([], false)]
    : []),
    ...internalArguments,
  ];

  return ts.factory.updateNewExpression(
    node,
    node.expression,
    node.typeArguments,
    updatedArgs,
  );
};
