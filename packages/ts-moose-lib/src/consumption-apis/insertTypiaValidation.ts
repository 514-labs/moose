import ts, { TypeChecker, factory, CallExpression } from "typescript";
import type { PluginConfig, TransformerExtras } from "ts-patch";
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
  checker: TypeChecker,
): node is CallExpression => {
  if (!ts.isCallExpression(node)) {
    console.log("not CallExpression");
    return false;
  }

  const declaration: ts.Declaration | undefined =
    checker.getResolvedSignature(node)?.declaration;
  if (!declaration) {
    console.log("no declaration");
    return false;
  }

  const location: string = path.resolve(declaration.getSourceFile().fileName);
  console.log("location", location);
  if (location.indexOf("@514labs/moose-lib") == -1) return false;

  const { name } = checker.getTypeAtLocation(declaration).symbol;

  console.log("name", name);

  return name == "createConsumptionApi";
};

const transformCreateConsumptionApi = (
  node: ts.Node,
  checker: TypeChecker,
): ts.Node => {
  if (!isCreateConsumptionApi(node, checker)) {
    console.log("not createConsumptionApi");
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
    //   assertGuard(params)
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
            // const processedParams = assertGuard(new URLSearchParams(params))
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

  console.log("compilerOptions", compilerOptions);

  const compilerHost = getPatchedHost(host, tsInstance, compilerOptions);
  const rootFileNames = program
    .getRootFileNames()
    .map(tsInstance.server.toNormalizedPath);

  /* Transform AST */
  const transformedSource = tsInstance.transform(
    /* sourceFiles */ program
      .getSourceFiles()
      .filter((sourceFile) =>
        rootFileNames.includes(sourceFile.fileName as any),
      ),
    /* transformers */ [transformAst(program).bind(tsInstance)],
    compilerOptions,
  ).transformed;

  /* Render modified files and create new SourceFiles for them to use in host's cache */
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

  /* Re-create Program instance */
  return tsInstance.createProgram(rootFileNames, compilerOptions, compilerHost);
}

const transformAst = (program: ts.Program) =>
  function (this: typeof ts, context: ts.TransformationContext) {
    const tsInstance = this;

    /* Transformer Function */
    return (sourceFile: ts.SourceFile) => {
      console.log("version sourceFile", (sourceFile as any).version);

      const recurse = (node: ts.Node, checker: TypeChecker): ts.Node =>
        ts.visitEachChild(
          transformCreateConsumptionApi(node, program.getTypeChecker()),
          (child) => recurse(child, checker),
          undefined,
        );
      const transformed = ts.visitEachChild(
        sourceFile,
        (node) => recurse(node, program.getTypeChecker()),
        undefined,
      );

      console.log("version transformed", (transformed as any).version);

      // Prepend the new statement to the file's statements
      const updatedStatements = ts.factory.createNodeArray([
        importTypia,
        ...transformed.statements,
      ]);

      // Return the updated source file
      const withImport = ts.factory.updateSourceFile(
        transformed,
        updatedStatements,
      );

      console.log("version withImport", (withImport as any).version);
      // (withImport as any).version = (sourceFile as any).version
      return withImport;
      //
      // return tsInstance.visitEachChild(sourceFile, visit, context);
      //
      // /* Visitor Function */
      // function visit(node: ts.Node): ts.Node {
      //   if (node.kind === ts.SyntaxKind.NumberKeyword)
      //     return context.factory.createKeywordTypeNode(
      //       tsInstance.SyntaxKind.StringKeyword,
      //     );
      //   else return tsInstance.visitEachChild(node, visit, context);
      // }
    };
  };
