use crate::parser::AnnotationKind;

/// Severity level for diagnostic messages
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    /// An error that should be fixed
    Error,
    /// A warning that should be addressed
    Warning,
}

/// A diagnostic message indicating an issue with annotations
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Line number where the diagnostic applies (0-indexed)
    pub line: usize,
    /// Human-readable diagnostic message
    pub message: String,
    /// Severity level of this diagnostic
    pub severity: DiagnosticSeverity,
}

/// Validate Rovo annotations in the given content
///
/// Checks for issues like invalid HTTP status codes and returns a list of diagnostics.
///
/// # Arguments
/// * `content` - The source code content to validate
///
/// # Returns
/// A vector of diagnostics for any validation errors found
pub fn validate_annotations(content: &str) -> Vec<Diagnostic> {
    let annotations = crate::parser::parse_annotations(content);
    let mut diagnostics = Vec::new();

    for ann in annotations {
        match ann.kind {
            AnnotationKind::Response => {
                if let Some(status) = ann.status {
                    if status < 100 || status > 599 {
                        diagnostics.push(Diagnostic {
                            line: ann.line,
                            message: format!(
                                "Invalid HTTP status code: {}. Must be between 100 and 599.",
                                status
                            ),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}
