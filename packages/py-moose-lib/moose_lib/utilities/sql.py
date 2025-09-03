def quote_identifier(name: str) -> str:
    """Quote a ClickHouse identifier with backticks if not already quoted.

    Backticks allow special characters (e.g., hyphens) in identifiers.
    """
    trimmed = name.strip()
    if trimmed.startswith("`") and trimmed.endswith("`"):
        return trimmed
    return f"`{trimmed}`"


