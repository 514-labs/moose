import ts, { factory } from "typescript";
import { replaceProgram } from "../compilerPluginHelper";
import { transformCreateConsumptionApi } from "./typiaValidation";
import { importTypia } from "../dmv2/compilerPlugin";

const transform =
  (typeChecker: ts.TypeChecker) =>
  (_context: ts.TransformationContext) =>
  (sourceFile: ts.SourceFile): ts.SourceFile => {
    const recurse = (node: ts.Node): ts.Node =>
      ts.visitEachChild(
        transformCreateConsumptionApi(node, typeChecker),
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
