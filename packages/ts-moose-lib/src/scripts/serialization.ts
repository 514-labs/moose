// Add serialization helpers
export const mooseJsonEncode = (data: any): string => {
  return JSON.stringify(data, (_, value) => {
    if (value instanceof Map) {
      return {
        __type: "Map",
        value: Array.from(value.entries()),
      };
    }
    return value;
  });
};

export const mooseJsonDecode = (text: string): any => {
  return JSON.parse(text, (_, value) => {
    if (value && typeof value === "object" && value.__type === "Map") {
      return new Map(value.value);
    }
    return value;
  });
};
