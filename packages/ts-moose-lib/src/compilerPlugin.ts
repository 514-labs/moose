import ts, { factory } from "typescript";
import { avoidTypiaNameClash, replaceProgram } from "./compilerPluginHelper";
import {
  isNewMooseResourceWithTypeParam,
  transformNewMooseResource,
} from "./dmv2/dataModelMetadata";
import {
  isCreateConsumptionApi,
  isCreateConsumptionApiV2,
  transformCreateConsumptionApi,
  transformLegacyConsumptionApi,
} from "./consumption-apis/typiaValidation";
import * as fs from "fs";
import * as path from "path";

// Create log file path
const logFilePath = path.join(
  process.cwd(),
  ".moose",
  "compiler-plugin-debug.log",
);

// Ensure .moose directory exists
const ensureLogDirectory = () => {
  const dir = path.dirname(logFilePath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
};

// Initialize log file
ensureLogDirectory();
fs.writeFileSync(
  logFilePath,
  `[CompilerPlugin] Debug log started at ${new Date().toISOString()}\n`,
);

// Helper function to write logs
const writeLog = (message: string) => {
  fs.appendFileSync(logFilePath, `${message}\n`);
};

/**
 * Creates the typia import statement to avoid name clashes
 */
export const createTypiaImport = () =>
  factory.createImportDeclaration(
    undefined,
    factory.createImportClause(
      false,
      factory.createIdentifier(avoidTypiaNameClash),
      undefined,
    ),
    factory.createStringLiteral("typia"),
    undefined,
  );

/**
 * Applies the appropriate transformation based on node type
 */
const applyTransformation = (
  node: ts.Node,
  typeChecker: ts.TypeChecker,
): ts.Node => {
  if (isCreateConsumptionApi(node, typeChecker)) {
    writeLog("[CompilerPlugin] Found legacy consumption API, transforming...");
    return transformLegacyConsumptionApi(node, typeChecker);
  }

  if (isCreateConsumptionApiV2(node, typeChecker)) {
    writeLog("[CompilerPlugin] Found consumption API v2, transforming...");
    return transformCreateConsumptionApi(node, typeChecker);
  }

  if (isNewMooseResourceWithTypeParam(node, typeChecker)) {
    writeLog(
      "[CompilerPlugin] Found Moose resource with type param, transforming...",
    );
    return transformNewMooseResource(node, typeChecker);
  }

  return node;
};

/**
 * Checks if typia import already exists in the source file
 */
const hasExistingTypiaImport = (sourceFile: ts.SourceFile): boolean => {
  const hasImport = sourceFile.statements.some((stmt) => {
    if (
      !ts.isImportDeclaration(stmt) ||
      !ts.isStringLiteral(stmt.moduleSpecifier)
    ) {
      return false;
    }

    if (stmt.moduleSpecifier.text !== "typia") {
      return false;
    }

    // Check if it has our specific aliased import
    const importClause = stmt.importClause;
    if (
      importClause &&
      importClause.name &&
      importClause.name.text === avoidTypiaNameClash
    ) {
      return true;
    }

    return false;
  });
  writeLog(
    `[CompilerPlugin] Checking for existing typia import (${avoidTypiaNameClash}) in ${sourceFile.fileName}: ${hasImport}`,
  );
  return hasImport;
};

/**
 * Adds typia import to the source file if transformations were applied
 */
const addTypiaImport = (sourceFile: ts.SourceFile): ts.SourceFile => {
  if (hasExistingTypiaImport(sourceFile)) {
    writeLog(
      `[CompilerPlugin] Typia import already exists in ${sourceFile.fileName}, skipping...`,
    );
    return sourceFile;
  }

  writeLog(`[CompilerPlugin] Adding typia import to ${sourceFile.fileName}`);
  const statementsWithImport = factory.createNodeArray([
    createTypiaImport(),
    ...sourceFile.statements,
  ]);

  return factory.updateSourceFile(sourceFile, statementsWithImport);
};

/**
 * Checks if a source file contains references to the typia namespace
 */
const containsTypiaReferences = (sourceFile: ts.SourceFile): boolean => {
  let hasTypiaReferences = false;
  let identifierCount = 0;

  const visitNode = (node: ts.Node): void => {
    if (ts.isIdentifier(node)) {
      identifierCount++;
      if (node.text === avoidTypiaNameClash) {
        writeLog(
          `[CompilerPlugin] Found typia reference: ${node.text} at position ${node.pos}`,
        );
        hasTypiaReferences = true;
        return;
      }
    }
    ts.forEachChild(node, visitNode);
  };

  writeLog(
    `[CompilerPlugin] Checking for typia references in ${sourceFile.fileName}...`,
  );
  ts.forEachChild(sourceFile, visitNode);
  writeLog(
    `[CompilerPlugin] Scanned ${identifierCount} identifiers, found typia references: ${hasTypiaReferences}`,
  );
  return hasTypiaReferences;
};

/**
 * Main transformation function that processes TypeScript source files
 */
const transform =
  (typeChecker: ts.TypeChecker) =>
  (_context: ts.TransformationContext) =>
  (sourceFile: ts.SourceFile): ts.SourceFile => {
    writeLog(
      `\n[CompilerPlugin] ========== Processing file: ${sourceFile.fileName} ==========`,
    );

    let transformationCount = 0;

    const visitNode = (node: ts.Node): ts.Node => {
      const originalNode = node;
      // Apply transformation and recursively visit children
      const transformed = applyTransformation(node, typeChecker);

      if (transformed !== originalNode) {
        transformationCount++;
        writeLog(
          `[CompilerPlugin] Transformation #${transformationCount} applied at position ${node.pos}`,
        );
      }

      return ts.visitEachChild(transformed, visitNode, undefined);
    };

    const transformedSourceFile = ts.visitEachChild(
      sourceFile,
      visitNode,
      undefined,
    );

    writeLog(
      `[CompilerPlugin] Total transformations applied: ${transformationCount}`,
    );

    // Check if the transformed file contains typia references
    const needsTypiaImport = containsTypiaReferences(transformedSourceFile);

    writeLog(`[CompilerPlugin] Needs typia import: ${needsTypiaImport}`);

    if (needsTypiaImport) {
      const result = addTypiaImport(transformedSourceFile);
      writeLog(
        `[CompilerPlugin] ========== Completed processing ${sourceFile.fileName} (with import) ==========\n`,
      );
      return result;
    } else {
      writeLog(
        `[CompilerPlugin] ========== Completed processing ${sourceFile.fileName} (no import needed) ==========\n`,
      );
      return transformedSourceFile;
    }
  };

export default replaceProgram(transform);
