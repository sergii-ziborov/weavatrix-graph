use crate::{EdgeKind, EvidenceKind, GraphError, Language, NodeKind, Result};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::borrow::Borrow;
use std::collections::BTreeMap;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Confidence {
    Exact,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourcePosition {
    /// One-based line number.
    pub line: u32,
    /// One-based byte column.
    pub column: u32,
}

impl SourcePosition {
    #[must_use]
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub start: SourcePosition,
    /// Exclusive end position.
    pub end: SourcePosition,
}

impl SourceSpan {
    #[must_use]
    pub fn new(file: impl Into<String>, start: SourcePosition, end: SourcePosition) -> Self {
        Self {
            file: file.into(),
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Provenance {
    pub extractor: String,
    pub evidence: EvidenceKind,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Provenance {
    /// Creates evidence provenance with a non-empty extractor identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the extractor is empty.
    pub fn new(
        extractor: impl Into<String>,
        evidence: EvidenceKind,
        confidence: Confidence,
    ) -> Result<Self> {
        let extractor = extractor.into();
        if extractor.is_empty() {
            return Err(GraphError::EmptyExtractor);
        }
        Ok(Self {
            extractor,
            evidence,
            confidence,
            span: None,
            detail: None,
        })
    }

    #[must_use]
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl Node {
    /// Creates a graph node.
    ///
    /// # Errors
    ///
    /// Returns an error when the node identifier is empty.
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: NodeKind) -> Result<Self> {
        Ok(Self {
            id: NodeId::new(id)?,
            label: label.into(),
            kind,
            language: None,
            span: None,
            attributes: BTreeMap::new(),
        })
    }

    #[must_use]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    #[must_use]
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub provenance: Provenance,
}

impl Edge {
    #[must_use]
    pub const fn new(
        source: NodeId,
        target: NodeId,
        kind: EdgeKind,
        provenance: Provenance,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            provenance,
        }
    }
}
