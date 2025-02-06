import ts, { factory } from "typescript";
import type { PluginConfig, ProgramTransformerExtras } from "ts-patch";
import path from "path";
import fs from "node:fs";
import process from "process";

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
  if (
    !location.includes("@514labs/moose-lib") &&
    // workaround for e2e test
    !location.includes("packages/ts-moose-lib/dist")
  )
    return false;

  const { name } = checker.getTypeAtLocation(declaration).symbol;

  return name == "createConsumptionApi";
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

const transformCreateConsumptionApi = (
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

  const queryResultType = awaitedType.isUnion()
    ? factory.createUnionTypeNode(
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
        factory.createCallExpression(
          factory.createPropertyAccessExpression(
            factory.createPropertyAccessExpression(
              factory.createIdentifier(avoidTypiaNameClash),
              factory.createIdentifier("json"),
            ),
            factory.createIdentifier("schemas"),
          ),
          [factory.createTupleTypeNode([paramType])],
          [],
        ),
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
  { ts: tsInstance }: ProgramTransformerExtras,
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
    const newFile = printFile(sourceFile);
    const path = fileName.split("/").pop() || fileName;
    fs.mkdirSync(`${process.cwd()}/.moose/api-compile-step/`, {
      recursive: true,
    });
    fs.writeFileSync(
      `${process.cwd()}/.moose/api-compile-step/${path}`,
      newFile,
    );
    const updatedSourceFile = tsInstance.createSourceFile(
      fileName,
      newFile,
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
