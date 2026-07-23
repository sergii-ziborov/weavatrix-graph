use crate::{GraphError, Result};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct NodeId(String);

impl NodeId {
    /// Creates a non-empty node identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is empty.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            Err(GraphError::EmptyNodeId)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NodeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Borrow<str> for NodeId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl FromStr for NodeId {
    type Err = GraphError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}
