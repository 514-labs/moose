import ts, {
  isIdentifier,
  isTypeReferenceNode,
  SymbolFlags,
  TypeChecker,
  TypeFlags,
} from "typescript";
import { enumConvert, isEnum } from "./enumConvert";
import {
  ArrayType,
  Column,
  DataType,
  NullType,
  UnknownType,
  UnsupportedFeature,
} from "./dataModelTypes";

const dateType = (checker: TypeChecker) =>
  checker
    .getTypeOfSymbol(
      checker.resolveName("Date", undefined, SymbolFlags.Type, false)!,
    )
    .getConstructSignatures()[0]
    .getReturnType();

// making throws expressions so that they can be used in ternaries

const throwUnknownType = (
  t: ts.Type,
  fieldName: string,
  typeName: string,
): never => {
  throw new UnknownType(t, fieldName, typeName);
};

const throwNullType = (fieldName: string, typeName: string): never => {
  throw new NullType(fieldName, typeName);
};

const toArrayType = ([elementNullable, elementType]: [
  boolean,
  DataType,
]): ArrayType => {
  return {
    elementNullable,
    elementType,
  };
};

const tsTypeToDataType = (
  t: ts.Type,
  checker: TypeChecker,
  fieldName: string,
  typeName: string,
  isJwt: boolean,
): [boolean, DataType] => {
  const nonNull = t.getNonNullableType();
  const nullable = nonNull != t;

  // this looks nicer if we turn on experimentalTernaries in prettier
  const dataType: DataType = isEnum(nonNull)
    ? enumConvert(nonNull)
    : nonNull == checker.getStringType()
      ? "String"
      : nonNull == checker.getNumberType()
        ? "Float"
        : nonNull == checker.getBooleanType()
          ? "Boolean"
          : nonNull == dateType(checker)
            ? "DateTime"
            : checker.isArrayType(nonNull)
              ? toArrayType(
                  tsTypeToDataType(
                    nonNull.getNumberIndexType()!,
                    checker,
                    fieldName,
                    typeName,
                    isJwt,
                  ),
                )
              : nonNull.isClassOrInterface() ||
                  (nonNull.flags & TypeFlags.Object) !== 0
                ? {
                    name: t.symbol.name,
                    columns: toColumns(nonNull, checker),
                    jwt: isJwt,
                  }
                : nonNull == checker.getNeverType()
                  ? throwNullType(fieldName, typeName)
                  : throwUnknownType(t, fieldName, typeName);

  return [nullable, dataType];
};

const hasWrapping = (
  typeNode: ts.TypeNode | undefined,
  wrapperName: string,
) => {
  if (typeNode !== undefined && isTypeReferenceNode(typeNode)) {
    const typeName = typeNode.typeName;
    const name = isIdentifier(typeName) ? typeName.text : typeName.right.text;
    return name === wrapperName && typeNode.typeArguments?.length === 1;
  } else {
    return false;
  }
};

const hasKeyWrapping = (typeNode: ts.TypeNode | undefined) => {
  return hasWrapping(typeNode, "Key");
};

const hasJwtWrapping = (typeNode: ts.TypeNode | undefined) => {
  return hasWrapping(typeNode, "JWT");
};

export const toColumns = (t: ts.Type, checker: TypeChecker): Column[] => {
  if (checker.getIndexInfosOfType(t).length !== 0) {
    console.log(checker.getIndexInfosOfType(t));
    throw new UnsupportedFeature("index type");
  }

  return checker.getPropertiesOfType(t).map((prop) => {
    const node = prop.getDeclarations()![0] as ts.PropertyDeclaration;
    const type = checker.getTypeOfSymbolAtLocation(prop, node);

    const isKey = hasKeyWrapping(node.type);
    const isJwt = hasJwtWrapping(node.type);
    const [nullable, dataType] = tsTypeToDataType(
      type,
      checker,
      prop.name,
      t.symbol.name,
      isJwt,
    );

    return {
      name: prop.name,
      data_type: dataType,
      primary_key: isKey,
      required: !nullable,
      unique: false,
      default: null,
    };
  });
};
