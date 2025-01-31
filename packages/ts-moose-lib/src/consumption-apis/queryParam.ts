// import ts from "typescript";
//
// interface ValidationRule {
//   type: string;
//   value?: string | number | bigint;
// }
//
// interface QueryField {
//   name: string;
//   data_type: ScalarType | ArrayType;
//   required: boolean;
//   validation?: ValidationRule[];
// }
//
// type ScalarType = "String" | "Float" | "Int" | "Boolean" | "DateTime";
//
// interface ArrayType {
//   element_type: ScalarType;
// }
//
// interface TypeTag {
//   name: string;
//   value?: string | number | bigint;
// }
//
// const extractTypiaTags = (
//   t: ts.Type,
//   checker: ts.TypeChecker,
// ): ValidationRule[] => {
//   // If it's not an intersection type, no tags to extract
//   if (!(t.flags & ts.TypeFlags.Intersection)) return [];
//
//   const intersectionType = t as ts.IntersectionType;
//   const rules: ValidationRule[] = [];
//
//   for (const type of intersectionType.types) {
//     const symbol = type.getSymbol();
//
//     console.log("type", type);
//
//     const parentIdentifier = symbol?.declarations?.[0]?.parent;
//
//     const isTagsNamespace =
//       parentIdentifier !== undefined &&
//       ts.isIdentifier(parentIdentifier) &&
//       parentIdentifier.text === "TagBase";
//
//     console.log("parentIdentifier.kind", (parentIdentifier as any)?.kind);
//     console.log("parentIdentifier.text", (parentIdentifier as any)?.text);
//
//     if (!isTagsNamespace) continue;
//
//     const tagName = symbol.getName();
//     const typeRef = type as ts.TypeReference;
//
//     // Handle different tag types
//     switch (tagName) {
//       // Number validations
//       case "Type": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({ type: "type", value: String(literalType.value) });
//         }
//         break;
//       }
//       case "Minimum":
//       case "Maximum":
//       case "ExclusiveMinimum":
//       case "ExclusiveMaximum":
//       case "MultipleOf": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({ type: tagName.toLowerCase(), value: literalType.value });
//         }
//         break;
//       }
//       // String validations
//       case "MinLength":
//       case "MaxLength": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({
//             type: tagName.toLowerCase(),
//             value: Number(literalType.value),
//           });
//         }
//         break;
//       }
//       case "Pattern": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({ type: "pattern", value: String(literalType.value) });
//         }
//         break;
//       }
//       case "Format": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({ type: "format", value: String(literalType.value) });
//         }
//         break;
//       }
//       // Array validations
//       case "MinItems":
//       case "MaxItems": {
//         if (typeRef.typeArguments?.[0]) {
//           const literalType = typeRef.typeArguments[0] as ts.LiteralType;
//           rules.push({
//             type: tagName.toLowerCase(),
//             value: Number(literalType.value),
//           });
//         }
//         break;
//       }
//       case "UniqueItems": {
//         rules.push({ type: "uniqueItems" });
//         break;
//       }
//     }
//   }
//
//   console.log("rules", rules);
//
//   return rules;
// };
//
// const toScalarType = (t: ts.Type, checker: ts.TypeChecker): ScalarType => {
//   // Get the base type without tags
//   const baseType =
//     t.flags & ts.TypeFlags.Intersection
//       ? (t as ts.IntersectionType).types.find((intersectionType: ts.Type) => {
//           const symbol = intersectionType.getSymbol();
//           const parentIdentifier = symbol?.declarations?.[0]?.parent;
//           return !(
//             parentIdentifier !== undefined &&
//             ts.isIdentifier(parentIdentifier) &&
//             parentIdentifier.text === "tags"
//           );
//         }) ?? t
//       : t;
//
//   // Extract any typia tags
//   const validationRules = extractTypiaTags(t, checker);
//
//   // Check for type tag first
//   const typeRule = validationRules.find((r) => r.type === "Type");
//   if (typeRule?.value) {
//     switch (typeRule.value) {
//       case "int32":
//       case "uint32":
//       case "int64":
//       case "uint64":
//         return "Int";
//       case "float":
//       case "double":
//         return "Float";
//     }
//   }
//
//   if (baseType.flags & ts.TypeFlags.String) return "String";
//   if (baseType.flags & ts.TypeFlags.Number) return "Float";
//   if (baseType.flags & ts.TypeFlags.Boolean) return "Boolean";
//
//   // Check if it's a Date type
//   const dateType = checker
//     .getTypeOfSymbol(
//       checker.resolveName("Date", undefined, ts.SymbolFlags.Type, false)!,
//     )
//     .getConstructSignatures()[0]
//     .getReturnType();
//
//   if (checker.isTypeAssignableTo(baseType, dateType)) return "DateTime";
//
//   throw new Error(`Unsupported type: ${checker.typeToString(t)}`);
// };
//
// export const dumpParamType = (
//   paramType: ts.TypeNode,
//   checker: ts.TypeChecker,
// ): QueryField[] => {
//   const t = checker.getTypeFromTypeNode(paramType);
//   const fields: QueryField[] = [];
//
//   // Get interface declaration if available
//   const declarations = t.symbol?.getDeclarations();
//   const declaration = declarations === undefined ? undefined : declarations[0];
//
//   if (declaration !== undefined && ts.isInterfaceDeclaration(declaration)) {
//     // Process each property in the interface
//     const properties = checker.getPropertiesOfType(t);
//
//     for (const prop of properties) {
//       const propDeclaration = prop.declarations![0];
//       const propType = checker.getTypeOfSymbolAtLocation(prop, propDeclaration);
//       const name = prop.getName();
//
//       // Check if property type is optional
//       const required = !(
//         propType.flags & ts.TypeFlags.Union &&
//         propType.types.some((t) => t.flags & ts.TypeFlags.Undefined)
//       );
//
//       // Get the non-nullable type
//       const nonNullableType = checker.getNonNullableType(propType);
//
//       // Extract validation rules
//       const validation = extractTypiaTags(nonNullableType, checker);
//
//       // Handle array types
//       if (checker.isArrayType(nonNullableType)) {
//         const elementType = checker.getTypeArguments(
//           nonNullableType as ts.TypeReference,
//         )[0];
//         fields.push({
//           name,
//           data_type: { element_type: toScalarType(elementType, checker) },
//           required,
//           validation,
//         });
//       } else {
//         // Handle scalar types
//         fields.push({
//           name,
//           data_type: toScalarType(nonNullableType, checker),
//           required,
//           validation,
//         });
//       }
//     }
//   }
//
//   return fields;
// };
