import ts, {
  isIdentifier,
  isTypeReferenceNode,
  SymbolFlags,
  TypeChecker,
} from "typescript";
import { enumConvert, isEnum } from "./enumConvert";
import { Column, DataType, UnknownType } from "./dataModelTypes";

const dateType = (checker: TypeChecker) =>
  checker
    .getTypeOfSymbol(
      checker.resolveName("Date", undefined, SymbolFlags.Type, false)!,
    )
    .getConstructSignatures()[0]
    .getReturnType();

const throwUnknownType = (t: ts.Type): never => {
  throw new UnknownType(t);
};

const tsTypeToDataType = (
  t: ts.Type,
  checker: TypeChecker,
): [boolean, DataType] => {
  const nonNull = t.getNonNullableType();
  const nullable = nonNull == t;

  // this looks nicer if we turn on experimentalTernaries in prettier
  const dataType: DataType = isEnum(nonNull)
    ? enumConvert(nonNull)
    : nonNull == checker.getStringType()
      ? "String"
      : nonNull == checker.getNumberType()
        ? "Float"
        : nonNull == checker.getBooleanType()
          ? "Float"
          : nonNull == dateType(checker)
            ? "Date"
            : checker.isArrayType(nonNull)
              ? {
                  elementType: tsTypeToDataType(
                    nonNull.getNumberIndexType()!,
                    checker,
                  )[1],
                }
              : throwUnknownType(t);

  return [nullable, dataType];
};

const hasKeyWrapping = (typeNode: ts.TypeNode | undefined) => {
  if (typeNode !== undefined && isTypeReferenceNode(typeNode)) {
    const typeName = typeNode.typeName;
    return (
      (isIdentifier(typeName) ? typeName.text : typeName.right.text) == "Key" &&
      typeNode.typeArguments?.length === 1
    );
  } else {
    return false;
  }
};

export const toColumns = (t: ts.Type, checker: TypeChecker): Column[] => {
  return checker.getPropertiesOfType(t).map((prop) => {
    const node = prop.getDeclarations()![0] as ts.PropertyDeclaration;
    const type = checker.getTypeOfSymbolAtLocation(prop, node);

    const isKey = hasKeyWrapping(node.type);
    const [nullable, dataType] = tsTypeToDataType(type, checker);

    return {
      name: prop.name,
      dataType,
      isKey,
      required: !nullable,
      unique: false,
      default: null,
    };
  });
};
