pub fn parse_version(v: &str) -> Vec<i32> {
    v.split('.')
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect::<Vec<i32>>()
}

pub fn version_to_string(v: &[i32]) -> String {
    v.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(".")
}

pub fn sort_versions(versions: impl Iterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut parsed_versions = versions
        .map(|v| parse_version(v.as_ref()))
        .collect::<Vec<Vec<i32>>>();

    parsed_versions.sort();

    parsed_versions
        .into_iter()
        .map(|v| version_to_string(&v))
        .collect::<Vec<String>>()
}

pub fn find_previous_version(
    versions: impl Iterator<Item = impl AsRef<str>>,
    version: &str,
) -> Option<String> {
    sort_versions(versions)
        .iter()
        .rev()
        .find(|v| parse_version(v) < parse_version(version))
        .cloned()
}
