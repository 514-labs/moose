import ts, { factory } from "typescript";
import { avoidTypiaNameClash, replaceProgram } from "../compilerPluginHelper";
import {
  isNewMooseResourceWithTypeParam,
  transformNewOlapTable,
} from "./dataModelMetadata";
import {
  isCreateConsumptionApi,
  transformCreateConsumptionApi,
} from "../consumption-apis/typiaValidation";

export const importTypia = factory.createImportDeclaration(
  undefined,
  factory.createImportClause(
    false,
    factory.createIdentifier(avoidTypiaNameClash),
    undefined,
  ),
  factory.createStringLiteral("typia"),
  undefined,
);

const insertMdOrValidation = (
  node: ts.Node,
  checker: ts.TypeChecker,
): ts.Node => {
  if (isNewMooseResourceWithTypeParam(node, checker)) {
    return transformNewOlapTable(node, checker);
  } else if (isCreateConsumptionApi(node, checker)) {
    return transformCreateConsumptionApi(node, checker);
  }

  return node;
};

const transform =
  (typeChecker: ts.TypeChecker) =>
  (_context: ts.TransformationContext) =>
  (sourceFile: ts.SourceFile): ts.SourceFile => {
    const recurse = (node: ts.Node): ts.Node =>
      ts.visitEachChild(
        insertMdOrValidation(node, typeChecker),
        recurse,
        undefined,
      );
    const transformed = ts.visitEachChild(sourceFile, recurse, undefined);

    // prepend the import statement to the file's statements
    const withTypiaImport = factory.createNodeArray([
      importTypia,
      ...transformed.statements,
    ]);

    return factory.updateSourceFile(transformed, withTypiaImport);
  };

export default replaceProgram(transform);
