import ts, { isInterfaceDeclaration } from "typescript";
import type { PluginConfig, TransformerExtras } from "ts-patch";
import { toColumns } from "./typeConvert";
import * as fs from "node:fs";
import { Column, UnknownType, UnsupportedEnum } from "./dataModelTypes";

export default function (
  program: ts.Program,
  _pluginConfig: PluginConfig,
  _extras: TransformerExtras,
) {
  const checker = program.getTypeChecker();

  const cwd = program.getCurrentDirectory();
  const dataModelDir = `${cwd}/app/datamodels/`;
  const outputDir = `${cwd}/.moose/serialized_datamodels/`;
  fs.mkdirSync(outputDir, { recursive: true });

  return (_ctx: ts.TransformationContext) => {
    return (sourceFile: ts.SourceFile) => {
      if (!sourceFile.fileName.startsWith(dataModelDir)) return sourceFile;
      const fileNameWithoutPrefix = sourceFile.fileName.slice(
        dataModelDir.length,
      );

      checker
        .getExportsOfModule(checker.getSymbolAtLocation(sourceFile)!)
        .forEach((exported) => {
          const declaration = exported.declarations![0];
          if (isInterfaceDeclaration(declaration)) {
            const name = declaration.name.text;
            const t = checker.getTypeAtLocation(declaration);

            try {
              const columns: Column[] = toColumns(t, checker);
              fs.writeFileSync(
                `${outputDir}/${name}.json`,
                JSON.stringify({ fileName: fileNameWithoutPrefix, columns }),
                "utf8",
              );
            } catch (e) {
              if (e instanceof UnknownType) {
                const unknownType = e.t.getSymbol()?.name;
                fs.writeFileSync(
                  `${outputDir}/${name}.json`,
                  `Unknown type: ${unknownType}`,
                  "utf8",
                );
              } else if (e instanceof UnsupportedEnum) {
                fs.writeFileSync(
                  `${outputDir}/${name}.json`,
                  `Unsupported enum members in ${e.name}`,
                  "utf8",
                );
              } else {
                throw e;
              }
            }
          }
        });

      return sourceFile;
    };
  };
}
