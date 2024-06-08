import ts, { isEnumDeclaration, isInterfaceDeclaration } from "typescript";
import type { PluginConfig, TransformerExtras } from "ts-patch";
import { toColumns } from "./typeConvert";
import * as fs from "node:fs";
import {
  Column,
  DataEnum,
  DataModel,
  UnknownType,
  UnsupportedEnum,
} from "./dataModelTypes";
import { enumConvert } from "./enumConvert";

const convertSourceFile = (
  sourceFile: ts.SourceFile,
  checker: ts.TypeChecker,
) => {
  const output = {
    models: [] as DataModel[],
    enums: [] as DataEnum[],
  };

  checker
    .getExportsOfModule(checker.getSymbolAtLocation(sourceFile)!)
    .forEach((exported) => {
      const declaration = exported.declarations![0];
      if (isInterfaceDeclaration(declaration)) {
        const name = declaration.name.text;
        const t = checker.getTypeAtLocation(declaration);

        const columns: Column[] = toColumns(t, checker);
        output.models.push({
          name,
          columns,
        });
      }
      if (isEnumDeclaration(declaration)) {
        const t = checker.getTypeAtLocation(declaration);
        output.enums.push(enumConvert(t));
      }
    });
  return output;
};

export default function (
  program: ts.Program,
  _pluginConfig: PluginConfig,
  _extras: TransformerExtras,
) {
  const checker = program.getTypeChecker();

  const cwd = program.getCurrentDirectory();
  const dataModelDir = `${cwd}/app/datamodels/`;
  const oldVersionDir = `${cwd}/.moose/versions/`;

  const outputDir = `${cwd}/.moose/serialized_datamodels/`;
  fs.mkdirSync(outputDir, { recursive: true });

  return (_ctx: ts.TransformationContext) => {
    return (sourceFile: ts.SourceFile) => {
      if (
        sourceFile.fileName.startsWith(dataModelDir) ||
        sourceFile.fileName.startsWith(oldVersionDir)
      ) {
        let output: any;
        try {
          output = convertSourceFile(sourceFile, checker);
        } catch (e) {
          if (e instanceof UnknownType) {
            output = {
              error_type: "unknown_type",
              field: e.fieldName,
              parent: e.typeName,
              type: e.t.getSymbol()?.name,
            };
          } else if (e instanceof UnsupportedEnum) {
            output = {
              error_type: "unsupported_enum",
              type_name: e.enum_name,
            };
          } else {
            throw e;
          }
        }
        const nameWithoutExtension = sourceFile.fileName
          .slice(sourceFile.fileName.lastIndexOf("/"))
          .replace(/\.ts$/, "");

        fs.writeFileSync(
          `${outputDir}/${nameWithoutExtension}.json`,
          JSON.stringify(output),
          "utf8",
        );
      }

      return sourceFile;
    };
  };
}
