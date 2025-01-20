import { transform as typiaTransform } from "typia/lib/transform";
import ts, { TypeChecker, factory, CallExpression } from "typescript";
import type { PluginConfig, TransformerExtras } from "ts-patch";
import path from "path";

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
  if (!ts.isCallExpression(node)) return false;

  const declaration: ts.Declaration | undefined =
    checker.getResolvedSignature(node)?.declaration;
  if (!declaration) return false;

  const location: string = path.resolve(declaration.getSourceFile().fileName);
  console.log("location", location);
  if (location.indexOf("runner.") == -1) return false;

  return true;
};

const transformCreateConsumptionApi = (node: ts.Node, checker: TypeChecker) => {
  if (!isCreateConsumptionApi(node, checker)) return node;

  const handlerFunc = node.arguments[0];

  iife([
    factory.createImportDeclaration(
      undefined,
      factory.createImportClause(
        false,
        factory.createIdentifier("typia"),
        undefined,
      ),
      factory.createStringLiteral("typia"),
      undefined,
    ),

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
                factory.createIdentifier("typia"),
                factory.createIdentifier("createAssertGuard"),
              ),
              [node.typeArguments!![0]],
              [],
            ),
          ),
        ],
        ts.NodeFlags.Const | ts.NodeFlags.Constant | ts.NodeFlags.Constant,
      ),
    ),

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
        ts.NodeFlags.Const | ts.NodeFlags.Constant | ts.NodeFlags.Constant,
      ),
    ),
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
            factory.createExpressionStatement(
              factory.createCallExpression(
                factory.createIdentifier("assertGuard"),
                undefined,
                [factory.createIdentifier("params")],
              ),
            ),
            factory.createReturnStatement(
              factory.createCallExpression(
                factory.createIdentifier("handlerFunc"),
                undefined,
                [
                  factory.createIdentifier("params"),
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

export default (
    program: ts.Program,
    _pluginConfig: PluginConfig,
    extras: TransformerExtras,
  ) =>
  (context: ts.TransformationContext) =>
  (file: ts.SourceFile) => {
    const transformed = ts.visitEachChild(
      file,
      (node) => transformCreateConsumptionApi(node, program.getTypeChecker()),
      undefined,
    );
    // cast to any because the types in ts versions clash
    typiaTransform(program, undefined, extras as any)(context)(transformed);
  };
