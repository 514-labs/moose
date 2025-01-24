import ts, { factory } from "typescript";
import type { PluginConfig } from "ts-patch";
import path from "path";

const avoidTypiaNameClash = "____moose____typia";

const importTypia = factory.createImportDeclaration(
  undefined,
  factory.createImportClause(
    false,
    factory.createIdentifier(avoidTypiaNameClash),
    undefined,
  ),
  factory.createStringLiteral("typia"),
  undefined,
);

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

const isCreateConsumptionApi = (
  node: ts.Node,
  checker: ts.TypeChecker,
): node is ts.CallExpression => {
  if (!ts.isCallExpression(node)) {
    return false;
  }

  const declaration: ts.Declaration | undefined =
    checker.getResolvedSignature(node)?.declaration;
  if (!declaration) {
    return false;
  }

  const location: string = path.resolve(declaration.getSourceFile().fileName);
  if (location.indexOf("@514labs/moose-lib") == -1) return false;

  const { name } = checker.getTypeAtLocation(declaration).symbol;

  return name == "createConsumptionApi";
};

const transformCreateConsumptionApi = (
  node: ts.Node,
  checker: ts.TypeChecker,
): ts.Node => {
  if (!isCreateConsumptionApi(node, checker)) {
    return node;
  }

  const handlerFunc = node.arguments[0];

  return iife([
    // const assertGuard = typia.http.createAssertQuery<T>()
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
              [node.typeArguments!![0]],
              [],
            ),
          ),
        ],
        ts.NodeFlags.Const,
      ),
    ),
    // the user provided function
    // const handlerFunc = async(...) => ...
    factory.createVariableStatement(
      undefined,
      factory.createVariableDeclarationList(
        [
          factory.createVariableDeclaration(
            factory.createIdentifier("handlerFunc"),
            undefined,
            undefined,
            handlerFunc,
          ),
        ],
        ts.NodeFlags.Const,
      ),
    ),
    // return (params, utils) => {
    //   const processedParams = assertGuard(new URLSearchParams(params))
    //   return handlerFunc(params, utils)
    // }
    factory.createReturnStatement(
      factory.createArrowFunction(
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
      ),
    ),
  ]);
};
function getPatchedHost(
  maybeHost: ts.CompilerHost | undefined,
  tsInstance: typeof ts,
  compilerOptions: ts.CompilerOptions,
): ts.CompilerHost & { fileCache: Map<string, ts.SourceFile> } {
  const fileCache = new Map();
  const compilerHost =
    maybeHost ?? tsInstance.createCompilerHost(compilerOptions, true);
  const originalGetSourceFile = compilerHost.getSourceFile;

  return Object.assign(compilerHost, {
    getSourceFile(fileName: string, languageVersion: ts.ScriptTarget) {
      fileName = tsInstance.server.toNormalizedPath(fileName);
      if (fileCache.has(fileName)) return fileCache.get(fileName);

      const sourceFile = originalGetSourceFile.apply(
        void 0,
        Array.from(arguments) as any,
      );
      fileCache.set(fileName, sourceFile);

      return sourceFile;
    },
    fileCache,
  });
}

export default function transformProgram(
  program: ts.Program,
  host: ts.CompilerHost | undefined,
  config: PluginConfig,
  { ts: tsInstance }: ts.ProgramTransformerExtras,
): ts.Program {
  const compilerOptions = program.getCompilerOptions();

  const compilerHost = getPatchedHost(host, tsInstance, compilerOptions);
  const rootFileNames = program
    .getRootFileNames()
    .map(tsInstance.server.toNormalizedPath);

  const transformedSource = tsInstance.transform(
    program
      .getSourceFiles()
      .filter((sourceFile) =>
        rootFileNames.includes(sourceFile.fileName as any),
      ),
    [transform(program.getTypeChecker())],
    compilerOptions,
  ).transformed;

  const { printFile } = tsInstance.createPrinter();
  for (const sourceFile of transformedSource) {
    const { fileName, languageVersion } = sourceFile;
    const updatedSourceFile = tsInstance.createSourceFile(
      fileName,
      printFile(sourceFile),
      languageVersion,
    );
    (updatedSourceFile as any).version = (sourceFile as any).version;
    compilerHost.fileCache.set(fileName, updatedSourceFile);
  }

  return tsInstance.createProgram(rootFileNames, compilerOptions, compilerHost);
}

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
