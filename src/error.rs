//! Structured parse errors for mermaid diagrams.
//!
//! The library historically returns [`anyhow::Error`] for every
//! parse or layout failure. That is ergonomic but erases error
//! kind. [`ParseError`] sits alongside the `anyhow` surface as a
//! typed enum so callers (CMSs, editors, LLM correction loops)
//! can classify failures and produce actionable diagnostics
//! without scraping error strings.
//!
//! See `docs/error_tracking.md` for the line/column conventions
//! detection paths follow.

use thiserror::Error;

/// Typed parse errors surfaced by [`parse_mermaid_strict`]
/// (defined in `lib.rs`) and downstream strict entry points.
///
/// Line numbers are 1-based and count the raw input lines
/// before any `%%`-style comment stripping. Column numbers are
/// 1-based UTF-8 character offsets within the reported line
/// (not byte offsets).
///
/// This enum is `#[non_exhaustive]` so new variants can be added
/// without breaking semver. Matchers should include a wildcard
/// arm or upgrade with each release.
///
/// [`parse_mermaid_strict`]: crate::parse_mermaid_strict
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// A sequence, flowchart, or class/state diagram referenced
    /// a participant or node id that was never declared, at a
    /// site where auto-creation is not applied.
    #[error("unknown participant '{name}' at line {line}")]
    UnknownParticipant {
        /// The undeclared name as it appeared in the source.
        name: String,
        /// 1-based line number of the reference.
        line: u32,
        /// Declared names that are similar to `name`; may be
        /// empty. Useful for "did you mean?" suggestions.
        candidates: Vec<String>,
    },

    /// A `subgraph`, `group`, `alt`, `opt`, `loop`, or other
    /// block-style construct was opened but never closed with
    /// its matching `end` before EOF.
    #[error("unclosed subgraph opened at line {opened_at}")]
    UnclosedSubgraph {
        /// 1-based line number of the opening `subgraph` (or
        /// equivalent) keyword.
        opened_at: u32,
    },

    /// A token appeared where a different token was expected
    /// (e.g. a line started with an arrow operator and no
    /// source node, or a quoted string was not closed).
    #[error("unexpected token '{found}' at {line}:{col}; expected {expected}")]
    UnexpectedToken {
        /// 1-based line number.
        line: u32,
        /// 1-based character-column number.
        col: u32,
        /// The token or fragment that was actually encountered.
        found: String,
        /// A short human-readable description of what would
        /// have been valid here (e.g. `"node identifier"`,
        /// `"matching subgraph"`).
        expected: String,
    },

    /// A directive such as `%%{init: ... }%%` was present but
    /// could not be parsed. Typical causes: invalid JSON inside
    /// the `init` block, unsupported directive name, or a
    /// malformed opening/closing fence.
    #[error("invalid directive '{directive}' at {line}:{col}: {reason}")]
    InvalidDirective {
        /// 1-based line number of the directive opening.
        line: u32,
        /// 1-based character-column number of the directive
        /// opening.
        col: u32,
        /// The directive name (e.g. `"init"`), or `"unknown"`
        /// if the name itself could not be extracted.
        directive: String,
        /// Short human-readable reason explaining what failed
        /// (e.g. `"JSON parse error: expected comma at 1:42"`).
        reason: String,
    },
}

impl ParseError {
    /// Return the 1-based source line number associated with this
    /// error, when the variant carries one.
    ///
    /// `UnclosedSubgraph` returns the line where the opening
    /// `subgraph` keyword appeared. Other variants return the line
    /// where the offending token was seen.
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        match self {
            Self::UnknownParticipant { line, .. }
            | Self::UnexpectedToken { line, .. }
            | Self::InvalidDirective { line, .. } => Some(*line),
            Self::UnclosedSubgraph { opened_at } => Some(*opened_at),
        }
    }

    /// Return the 1-based source column number associated with
    /// this error, when the variant carries one. `UnknownParticipant`
    /// and `UnclosedSubgraph` have no meaningful column and return
    /// `None`.
    #[must_use]
    pub fn col(&self) -> Option<u32> {
        match self {
            Self::UnexpectedToken { col, .. } | Self::InvalidDirective { col, .. } => Some(*col),
            Self::UnknownParticipant { .. } | Self::UnclosedSubgraph { .. } => None,
        }
    }

    /// Return a short human-readable suggestion for fixing the
    /// error, when one is available. Callers render it on a
    /// separate `Suggestion:` line in the lenient warning box.
    ///
    /// The text is deliberately short (one line, imperative mood)
    /// so LLM correction loops and authoring tools can include it
    /// verbatim.
    #[must_use]
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::UnknownParticipant { candidates, .. } if !candidates.is_empty() => {
                let mut quoted: Vec<String> = candidates.iter().map(|c| format!("'{c}'")).collect();
                quoted.sort();
                quoted.dedup();
                Some(format!("did you mean {}?", quoted.join(" or ")))
            }
            Self::UnclosedSubgraph { .. } => {
                Some("add a matching `end` before EOF to close the block".to_string())
            }
            Self::UnexpectedToken { expected, .. } => Some(format!("expected {expected}")),
            Self::InvalidDirective { directive, .. } if directive == "init" => {
                Some("check JSON syntax of the %%{init: ...}%% block".to_string())
            }
            _ => None,
        }
    }
}

/// Bridge [`ParseError`] into [`anyhow::Error`] so the legacy
/// [`render`]/[`render_with_options`] façade can keep its
/// `anyhow::Result<_>` signature.
///
/// The derived [`std::error::Error`] from `thiserror` is enough
/// on its own (via `anyhow`'s blanket `From<E: Error>`), so this
/// impl exists solely to pin the semantic contract in one place
/// and to make intent explicit at call sites such as
/// `parse_mermaid(input).map_err(Into::into)`.
///
/// [`render`]: crate::render
/// [`render_with_options`]: crate::render_with_options
#[cfg(test)]
mod anyhow_bridge_is_derived {
    use super::ParseError;
    // Compile-time check: ParseError implements std::error::Error
    // via thiserror, which is all anyhow needs for auto-conversion.
    const _: fn() = || {
        fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
        assert_error::<ParseError>();
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_implements_display() {
        let e = ParseError::UnclosedSubgraph { opened_at: 7 };
        assert_eq!(format!("{e}"), "unclosed subgraph opened at line 7");
    }

    #[test]
    fn parse_error_unexpected_token_shape() {
        let e = ParseError::UnexpectedToken {
            line: 3,
            col: 5,
            found: "-->".into(),
            expected: "node identifier".into(),
        };
        assert_eq!(
            format!("{e}"),
            "unexpected token '-->' at 3:5; expected node identifier"
        );
    }

    #[test]
    fn parse_error_invalid_directive_shape() {
        let e = ParseError::InvalidDirective {
            line: 1,
            col: 1,
            directive: "init".into(),
            reason: "JSON parse error: expected '}'".into(),
        };
        assert_eq!(
            format!("{e}"),
            "invalid directive 'init' at 1:1: JSON parse error: expected '}'"
        );
    }

    #[test]
    fn parse_error_is_anyhow_convertible() {
        let e: anyhow::Error = ParseError::UnclosedSubgraph { opened_at: 2 }.into();
        assert!(e.to_string().contains("unclosed subgraph"));
    }

    #[test]
    fn line_and_col_accessors() {
        let e = ParseError::UnexpectedToken {
            line: 4,
            col: 7,
            found: "-->".into(),
            expected: "node id".into(),
        };
        assert_eq!(e.line(), Some(4));
        assert_eq!(e.col(), Some(7));

        let e = ParseError::UnclosedSubgraph { opened_at: 9 };
        assert_eq!(e.line(), Some(9));
        assert_eq!(e.col(), None);
    }

    #[test]
    fn suggestion_for_unknown_participant_with_candidates() {
        let e = ParseError::UnknownParticipant {
            name: "Aliec".into(),
            line: 3,
            candidates: vec!["Alice".into(), "Bob".into()],
        };
        let s = e.suggestion().expect("suggestion");
        assert!(s.contains("'Alice'"));
        assert!(s.contains("'Bob'"));
    }

    #[test]
    fn suggestion_for_unknown_participant_without_candidates_is_none() {
        let e = ParseError::UnknownParticipant {
            name: "X".into(),
            line: 1,
            candidates: vec![],
        };
        assert!(e.suggestion().is_none());
    }

    #[test]
    fn suggestion_for_unclosed_subgraph_is_some() {
        let e = ParseError::UnclosedSubgraph { opened_at: 2 };
        assert!(e.suggestion().is_some());
    }

    #[test]
    fn suggestion_for_unexpected_token_describes_expected() {
        let e = ParseError::UnexpectedToken {
            line: 1,
            col: 1,
            found: "-->".into(),
            expected: "node identifier".into(),
        };
        assert_eq!(e.suggestion().as_deref(), Some("expected node identifier"));
    }
}
