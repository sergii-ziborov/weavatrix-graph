use crate::{GraphError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

macro_rules! string_kind {
    (
        $(#[$meta:meta])*
        $name:ident, $category:literal, {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[non_exhaustive]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $($variant,)+
            Custom(String),
        }

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $value,)+
                    Self::Custom(value) => value,
                }
            }

            /// Creates an application-specific kind without changing this crate.
            ///
            /// # Errors
            ///
            /// Returns an error for empty values or surrounding whitespace.
            pub fn custom(value: impl Into<String>) -> Result<Self> {
                Self::parse_owned(value.into())
            }

            fn parse_owned(value: String) -> Result<Self> {
                if value.is_empty() || value.trim() != value {
                    return Err(GraphError::InvalidKind {
                        category: $category,
                        value,
                    });
                }
                Ok(match value.as_str() {
                    $($value => Self::$variant,)+
                    _ => Self::Custom(value),
                })
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = GraphError;

            fn from_str(value: &str) -> Result<Self> {
                Self::parse_owned(value.to_owned())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse_owned(value).map_err(D::Error::custom)
            }
        }
    };
}

string_kind!(
    /// Programming, data, or configuration language associated with a node.
    Language,
    "language",
    {
        Rust => "rust",
        Go => "go",
        C => "c",
        Cpp => "cpp",
        Bash => "bash",
        Sql => "sql",
        JavaScript => "javascript",
        TypeScript => "typescript",
        Python => "python",
        Java => "java",
        CSharp => "csharp",
        Yaml => "yaml",
    }
);

string_kind!(
    /// Semantic role of a graph node.
    NodeKind,
    "node",
    {
        Repository => "repository",
        File => "file",
        Module => "module",
        Package => "package",
        Function => "function",
        Method => "method",
        Struct => "struct",
        Enum => "enum",
        Trait => "trait",
        TypeAlias => "type_alias",
        Constant => "constant",
        Static => "static",
        Service => "service",
        Endpoint => "endpoint",
        Table => "table",
        Column => "column",
        Topic => "topic",
        ConsumerGroup => "consumer_group",
        Exchange => "exchange",
        Queue => "queue",
        Binding => "binding",
        Collection => "collection",
        Index => "index",
        KubernetesResource => "kubernetes_resource",
        Container => "container",
        ConfigKey => "config_key",
        Unknown => "unknown",
    }
);

string_kind!(
    /// Semantic relationship between two graph nodes.
    EdgeKind,
    "edge",
    {
        Contains => "contains",
        Imports => "imports",
        Calls => "calls",
        References => "references",
        DependsOn => "depends_on",
        Inherits => "inherits",
        Publishes => "publishes",
        Consumes => "consumes",
        Binds => "binds",
        Reads => "reads",
        Writes => "writes",
        Deploys => "deploys",
        Exposes => "exposes",
        Mounts => "mounts",
        Configures => "configures",
    }
);

string_kind!(
    /// Origin of the evidence supporting an edge.
    EvidenceKind,
    "evidence",
    {
        Parsed => "parsed",
        Resolved => "resolved",
        Manifest => "manifest",
        Literal => "literal",
        Toolchain => "toolchain",
        Runtime => "runtime",
    }
);
