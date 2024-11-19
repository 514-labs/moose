use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Version {
    version: String,
    parsed: Vec<i32>,
}
impl Hash for Version {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.version, state)
    }
}
impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.version.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_string(String::deserialize(deserializer)?))
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.version, f)
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.version, f)
    }
}
impl Version {
    pub fn from_string(version: String) -> Version {
        let parsed = parse_version(&version);
        Version { version, parsed }
    }
    pub fn as_str(&self) -> &str {
        &self.version
    }

    pub fn as_suffix(&self) -> String {
        self.version.replace('.', "_")
    }

    pub fn parsed(&self) -> &[i32] {
        &self.parsed
    }
}

impl Eq for Version {}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.parsed == other.parsed
    }
}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.parsed(), &other.parsed())
    }
}

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
