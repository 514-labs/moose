//! Version handling module for semantic versioning-like functionality.
//!
//! This module provides utilities for parsing, comparing, and manipulating version strings
//! in a format similar to semantic versioning (e.g., "1.2.3"). It supports basic version
//! comparison operations and serialization/deserialization.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a version number that can be parsed from strings like "1.2.3".
///
/// The version is stored both as the original string and as a parsed vector of integers
/// for efficient comparison operations.
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
    /// Creates a new `Version` instance from a string.
    ///
    /// # Arguments
    /// * `version` - A string representing the version (e.g., "1.2.3")
    ///
    /// # Examples
    /// ```
    /// let version = Version::from_string("1.2.3".to_string());
    /// ```
    pub fn from_string(version: String) -> Version {
        let parsed = parse_version(&version);
        Version { version, parsed }
    }
    /// Returns the version as a string slice.
    pub fn as_str(&self) -> &str {
        &self.version
    }

    /// Returns the version string with dots replaced by underscores.
    ///
    /// This is useful when the version needs to be used in contexts where
    /// dots are not allowed, such as in identifiers.
    ///
    /// # Examples
    /// ```
    /// let version = Version::from_string("1.2.3".to_string());
    /// assert_eq!(version.as_suffix(), "1_2_3");
    /// ```
    pub fn as_suffix(&self) -> String {
        self.version.replace('.', "_")
    }

    /// Returns the parsed version components as a slice of integers.
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

/// Parses a version string into a vector of integers.
///
/// # Arguments
/// * `v` - A string slice containing the version (e.g., "1.2.3")
///
/// # Returns
/// A vector of integers representing the version components.
/// Non-numeric components are parsed as 0.
pub fn parse_version(v: &str) -> Vec<i32> {
    v.split('.')
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect::<Vec<i32>>()
}

/// Converts a slice of version components back into a dot-separated version string.
///
/// # Arguments
/// * `v` - A slice of integers representing version components
///
/// # Returns
/// A string with the components joined by dots.
pub fn version_to_string(v: &[i32]) -> String {
    v.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(".")
}

/// Sorts a collection of version strings in ascending order.
///
/// # Arguments
/// * `versions` - An iterator of version strings
///
/// # Returns
/// A vector of sorted version strings.
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

/// Finds the highest version that is less than the specified version.
///
/// # Arguments
/// * `versions` - An iterator of version strings
/// * `version` - The reference version to compare against
///
/// # Returns
/// The highest version string that is less than the reference version,
/// or None if no such version exists.
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
