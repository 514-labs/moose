import ts from "typescript";

interface QueryField {
  name: string;
  data_type: ScalarType | ArrayType;
  required: boolean;
}

type ScalarType = "String" | "Float" | "Int" | "Boolean" | "DateTime";

interface ArrayType {
  element_type: ScalarType;
}

interface TypeTag {
  name: string;
  value?: string;
}

const extractTypiaTags = (t: ts.Type, checker: ts.TypeChecker): TypeTag[] => {
  // If it's an intersection type, examine each part
  if (t.flags & ts.TypeFlags.Intersection) {
    const intersectionType = t as ts.IntersectionType;
    const tags: TypeTag[] = [];

    for (const type of intersectionType.types) {
      // Look for types from the tags namespace
      const symbol = type.getSymbol();

      // Check if this is from the tags namespace by looking at the parent identifier
      const parentIdentifier: ts.Node | undefined =
        symbol?.declarations?.[0]?.parent;
      const isTagsNamespace =
        parentIdentifier !== undefined &&
        ts.isIdentifier(parentIdentifier) &&
        parentIdentifier.text === "tags";

      if (isTagsNamespace && symbol !== undefined) {
        const tagName = symbol.getName();

        // For tags with generic parameters (like Pattern<"regex">)
        if (tagName === "Pattern" && (type as ts.TypeReference).typeArguments) {
          const typeRef = type as ts.TypeReference;
          if (typeRef.typeArguments && typeRef.typeArguments.length > 0) {
            // Extract the string literal type value
            const literalType = typeRef.typeArguments[0] as ts.LiteralType;
            if (literalType.value) {
              tags.push({ name: "Pattern", value: String(literalType.value) });
            }
          }
        } else {
          // For simple tags without parameters
          tags.push({ name: tagName });
        }
      }
    }
    return tags;
  }
  return [];
};

const toScalarType = (t: ts.Type, checker: ts.TypeChecker): ScalarType => {
  // Get the base type without tags
  const baseType = t.isIntersection()
    ? t.types.find((intersectionType: ts.Type) => {
        const symbol = intersectionType.getSymbol();
        const parentIdentifier = symbol?.declarations?.[0]?.parent;
        // Return true for non-tag types
        return !(
          parentIdentifier !== undefined &&
          ts.isIdentifier(parentIdentifier) &&
          parentIdentifier.text === "tags"
        );
      }) ?? t
    : t;

  // Extract any typia tags
  const tags = extractTypiaTags(t, checker);

  if (baseType.flags & ts.TypeFlags.String) return "String";
  if (baseType.flags & ts.TypeFlags.Number) return "Float";
  if (baseType.flags & ts.TypeFlags.Boolean) return "Boolean";

  // Check if it's a Date type
  const dateType = checker
    .getTypeOfSymbol(
      checker.resolveName("Date", undefined, ts.SymbolFlags.Type, false)!,
    )
    .getConstructSignatures()[0]
    .getReturnType();

  if (checker.isTypeAssignableTo(baseType, dateType)) return "DateTime";

  throw new Error(`Unsupported type: ${checker.typeToString(t)}`);
};

export const dumpParamType = (
  paramType: ts.TypeNode,
  checker: ts.TypeChecker,
): QueryField[] => {
  const t = checker.getTypeFromTypeNode(paramType);
  const fields: QueryField[] = [];

  // Get interface declaration if available
  const declarations = t.symbol?.getDeclarations();
  const declaration = declarations === undefined ? undefined : declarations[0];

  if (declaration !== undefined && ts.isInterfaceDeclaration(declaration)) {
    // Process each property in the interface
    const properties = checker.getPropertiesOfType(t);

    for (const prop of properties) {
      const propDeclaration = prop.declarations![0];
      const propType: ts.Type = checker.getTypeOfSymbolAtLocation(
        prop,
        propDeclaration,
      );
      const name = prop.getName();

      // Get the non-nullable type
      const nonNullableType = checker.getNonNullableType(propType);
      console.log("nonNullableType", nonNullableType);
      console.log("propType", propType);
      const required = nonNullableType == propType;

      // Handle array types
      if (checker.isArrayType(nonNullableType)) {
        const elementType = checker.getTypeArguments(
          nonNullableType as ts.TypeReference,
        )[0];
        fields.push({
          name,
          data_type: { element_type: toScalarType(elementType, checker) },
          required,
        });
      } else {
        // Handle scalar types
        fields.push({
          name,
          data_type: toScalarType(nonNullableType, checker),
          required,
        });
      }
    }
  }

  return fields;
};
