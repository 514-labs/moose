import ts, { factory } from "typescript";
import { isMooseFile, typiaJsonSchemas } from "../compilerPluginHelper";
import { toColumns } from "../dataModels/typeConvert";

const typesToArgsLength = new Map([
  ["OlapTable", 2],
  ["Stream", 2],
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

export const transformNewMooseResource = (
  node: ts.NewExpression,
  checker: ts.TypeChecker,
): ts.Node => {
  const typeNode = node.typeArguments![0];
  const columns = toColumns(checker.getTypeAtLocation(typeNode), checker);

  const argLength = typesToArgsLength.get(
    checker.getSymbolAtLocation(node.expression)!.name,
  )!;
  const needsExtraArg = node.arguments!.length === argLength - 1; // provide empty config if undefined

  // --- Inject file location into config.metadata ---
  // Find the config argument (usually the second argument)
  let configArgIdx = argLength === 2 ? 1 : -1;
  let configArg =
    configArgIdx !== -1 ? node.arguments![configArgIdx] : undefined;
  const fileUrl = node.getSourceFile().fileName;

  let newConfigArg: ts.Expression;
  if (!configArg || !ts.isObjectLiteralExpression(configArg)) {
    // No config or not an object, create a new one
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
    // Config exists, update or add metadata.fileUrl
    let hasMetadata = false;
    const newProperties = configArg.properties.map((prop) => {
      if (
        ts.isPropertyAssignment(prop) &&
        ts.isIdentifier(prop.name) &&
        prop.name.text === "metadata" &&
        ts.isObjectLiteralExpression(prop.initializer)
      ) {
        hasMetadata = true;
        // Update or add fileUrl inside metadata
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
    // If no metadata property, add it
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

  // Build the updated argument list
  const updatedArgs = [
    node.arguments![0],
    ...(argLength === 2 ? [newConfigArg] : []),
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
