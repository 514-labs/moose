import ts, { factory } from "typescript";
import { isMooseFile, typiaJsonSchemas } from "../compilerPluginHelper";
import { toColumns } from "../dataModels/typeConvert";

const types = new Set([
  "OlapTable",
  "Stream",
  "IngestPipeline",
  "IngestApi",
  "ConsumptionApi",
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
  if (!types.has(sym?.name ?? "")) {
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

const parseAsAny = (s: string) =>
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

export const transformNewMooseResource = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  const typeNode = node.typeArguments![0];
  const columns = toColumns(checker.getTypeAtLocation(typeNode), checker);

  const updatedArgs = [
    ...node.arguments!,
    ...(node.arguments!.length === 1 // provide empty config if undefined
      ? [factory.createObjectLiteralExpression([], false)]
      : []),
    typiaJsonSchemas(typeNode),
    parseAsAny(JSON.stringify(columns)),
  ];

  return ts.factory.updateNewExpression(
    node,
    node.expression,
    node.typeArguments,
    updatedArgs,
  );
};
