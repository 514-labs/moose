export function chainQueries(
  snippetOne: string,
  snippetTwo: string,
  cte = "CommonTable",
) {
  return `WITH ${cte} AS (
        ${snippetOne}
        ) ${snippetTwo}`;
}

export function createCTE(cteList: { [name: string]: string }) {
  const ctes = Object.entries(cteList)
    .map(([name, snippet]) => `${name} AS (${snippet})`)
    .join(",\n");
  return `WITH ${ctes}`;
}
