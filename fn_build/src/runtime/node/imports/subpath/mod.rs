pub(crate) use read::read_subpath_imports;

mod read;

#[cfg(test)]
mod read_test;

/// Subpath import parsing errors surfaced through building NodeConfig.
#[derive(Debug, thiserror::Error)]
pub enum SubpathImportError {
    #[error("subpath import condition `{0}` is invalid")]
    BadImportCondition(String),
}

/// Priority ordered collection of subpath import mappings. This collection is selective of
/// conditions as configured in package.json and only contains the specifiers relevant for
/// import resolution.
pub type NodeSubpathImports = Vec<NodeSubpathImportMapping>;

/// Represents explicitly specified subpath imports like `#data` and wildcard specifiers such as
/// `#lib/*.js`. Wildcard specifiers are built with str::split_once on the asterisk.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum NodeSubpathImportMapping {
    Explicit {
        from: String,
        to: String,
    },
    Wildcard {
        from: NodeSubpathImportWildcard,
        to: NodeSubpathImportMapWildcardTo,
    },
}

/// Represents a subpath import specifier that uses a wildcard asterisk for string substitution.
/// The specifier is stored pre-split when building NodeSubpathImports.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct NodeSubpathImportWildcard {
    pub before: String,
    pub after: Option<String>,
}

/// Represents the destination of a NodeSubpathImportMapping::Wildcard mapping.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum NodeSubpathImportMapWildcardTo {
    Explicit(String),
    Wildcard(String, NodeSubpathImportAsterisks),
}

/// Flags whether to use replace or replacen 1 for wildcard substitution.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum NodeSubpathImportAsterisks {
    Single,
    Multiple,
}

impl NodeSubpathImportMapping {
    /// Builds a wildcard mapping if the mapping from specifier includes an asterisk. A mapping
    /// from specifier without an asterisk can map to a specifier with an asterisk that will be
    /// treated as an explicit mapping for path replacement instead of wildcard mapping.
    pub fn new(from: String, to: String) -> Self {
        match from.split_once('*').map(|(b, a)| {
            (
                b.to_string(),
                if a.is_empty() {
                    None
                } else {
                    Some(a.to_string())
                },
            )
        }) {
            None => NodeSubpathImportMapping::Explicit { from, to },
            Some((before, after)) => Self::Wildcard {
                from: NodeSubpathImportWildcard { before, after },
                to: NodeSubpathImportMapWildcardTo::new(to),
            },
        }
    }
}

impl NodeSubpathImportMapWildcardTo {
    fn new(to: String) -> Self {
        let mut asterisks = 0;
        for c in to.chars() {
            if c == '*' {
                asterisks += 1;
                if asterisks > 1 {
                    continue;
                }
            }
        }
        match asterisks {
            0 => NodeSubpathImportMapWildcardTo::Explicit(to),
            1 => NodeSubpathImportMapWildcardTo::Wildcard(to, NodeSubpathImportAsterisks::Single),
            _ => NodeSubpathImportMapWildcardTo::Wildcard(to, NodeSubpathImportAsterisks::Multiple),
        }
    }
}
