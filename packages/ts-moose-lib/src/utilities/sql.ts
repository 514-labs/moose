/**
 * Quote a ClickHouse identifier with backticks if not already quoted.
 * Backticks allow special characters (e.g., hyphens) in identifiers.
 */
export const quoteIdentifier = (name: string): string => {
  const trimmed = name.trim();
  return trimmed.startsWith("`") && trimmed.endsWith("`") ?
      trimmed
    : `\`${trimmed}\``;
};
