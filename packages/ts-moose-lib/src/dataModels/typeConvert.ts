import ts, {
  isIdentifier,
  isTypeReferenceNode,
  SymbolFlags,
  TupleType,
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

const toArrayType = ([elementNullable, _, elementType]: [
  boolean,
  AggregationFunction | undefined,
  DataType,
]): ArrayType => {
  return {
    elementNullable,
    elementType,
  };
};

const isNumberType = (t: ts.Type, checker: TypeChecker): boolean => {
  return checker.isTypeAssignableTo(t, checker.getNumberType());
};

const handleAggregated = (
  t: ts.Type,
  checker: TypeChecker,
  fieldName: string,
  typeName: string,
): AggregationFunction | undefined => {
  const functionSymbol = t.getProperty("_aggregationFunction");
  const argsTypesSymbol = t.getProperty("_argTypes");

  if (functionSymbol === undefined || argsTypesSymbol === undefined) {
    return undefined;
  }
  const functionStringLiteral = checker.getNonNullableType(
    checker.getTypeOfSymbol(functionSymbol),
  );
  const types = checker.getNonNullableType(
    checker.getTypeOfSymbol(argsTypesSymbol),
  );

  if (functionStringLiteral.isStringLiteral() && checker.isTupleType(types)) {
    const argumentTypes = ((types as TupleType).typeArguments || []).map(
      (t) => tsTypeToDataType(t, checker, fieldName, typeName, false)[2],
    );
    return { functionName: functionStringLiteral.value, argumentTypes };
  } else {
    console.log("Unexpected type inside Aggregated", functionStringLiteral);
    return undefined;
  }
};

const handleNumberType = (
  t: ts.Type,
  checker: TypeChecker,
  fieldName: string,
): string => {
  const tagSymbol = t.getProperty("typia.tag");
  if (tagSymbol === undefined) {
    return "Float";
  } else {
    const typiaProps = checker.getNonNullableType(
      checker.getTypeOfSymbol(tagSymbol),
    );
    const props: ts.Type[] = typiaProps.isIntersection()
      ? typiaProps.types
      : [typiaProps];

    for (const prop of props) {
      const valueSymbol = prop.getProperty("value");
      if (valueSymbol === undefined) {
        console.log(`Props.value is undefined for ${fieldName}`);
      } else {
        const valueTypeLiteral = checker.getTypeOfSymbol(valueSymbol);
        if (
          checker.isTypeAssignableTo(
            valueTypeLiteral,
            checker.getStringLiteralType("int64"),
          )
        ) {
          return "Int";
        } else {
          const typeString = valueTypeLiteral.isStringLiteral()
            ? valueTypeLiteral.value
            : "unknown";

          console.log(
            `Other number types are not supported: ${typeString} in field ${fieldName}`,
          );
        }
      }
    }

    return "Float";
  }
};

export interface AggregationFunction {
  functionName: string;
  argumentTypes: DataType[];
}

const tsTypeToDataType = (
  t: ts.Type,
  checker: TypeChecker,
  fieldName: string,
  typeName: string,
  isJwt: boolean,
): [boolean, AggregationFunction | undefined, DataType] => {
  const nonNull = t.getNonNullableType();
  const nullable = nonNull != t;

  const aggregationFunction = handleAggregated(t, checker, fieldName, typeName);

  // this looks nicer if we turn on experimentalTernaries in prettier
  const dataType: DataType = isEnum(nonNull)
    ? enumConvert(nonNull)
    : checker.isTypeAssignableTo(nonNull, checker.getStringType())
      ? "String"
      : isNumberType(nonNull, checker)
        ? handleNumberType(nonNull, checker, fieldName)
        : checker.isTypeAssignableTo(nonNull, checker.getBooleanType())
          ? "Boolean"
          : checker.isTypeAssignableTo(nonNull, dateType(checker))
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
                    name: getNestedName(nonNull, fieldName),
                    columns: toColumns(nonNull, checker),
                    jwt: isJwt,
                  }
                : nonNull == checker.getNeverType()
                  ? throwNullType(fieldName, typeName)
                  : throwUnknownType(t, fieldName, typeName);

  return [nullable, aggregationFunction, dataType];
};

const getNestedName = (t: ts.Type, fieldName: string) => {
  const name = t.symbol.name;
  // replace default name
  return name === "__type" ? fieldName : name;
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
    const [nullable, aggregationFunction, dataType] = tsTypeToDataType(
      type,
      checker,
      prop.name,
      t.symbol.name,
      isJwt,
    );

    const annotations: [string, any][] = [];
    if (aggregationFunction !== undefined) {
      annotations.push(["aggregationFunction", aggregationFunction]);
    }

    return {
      name: prop.name,
      data_type: dataType,
      primary_key: isKey,
      required: !nullable,
      unique: false,
      default: null,
      annotations,
    };
  });
};
