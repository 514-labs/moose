export function topBrowsersQuery() {
  const test = fetch("http://localhost:4000/consumption/top_paths")
    .then((response) => response.json())
    .then((data) => data)
    .catch((error) => console.error("Error:", error));

  return test;
}
