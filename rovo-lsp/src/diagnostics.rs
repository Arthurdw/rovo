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
    /// Line number where the diagnostic applies (0-indexed, matching LSP protocol)
    ///
    /// Note: This uses 0-based line indexing to align with the LSP specification.
    /// Line 0 is the first line of the file.
    pub line: usize,
    /// Human-readable diagnostic message
    pub message: String,
    /// Severity level of this diagnostic
    pub severity: DiagnosticSeverity,
    /// Character range for the diagnostic (optional, defaults to whole line)
    pub char_start: Option<usize>,
    /// End character position (optional, defaults to end of line)
    pub char_end: Option<usize>,
    /// End line for multi-line diagnostics (optional, defaults to same as line)
    pub end_line: Option<usize>,
    /// End character position on the end line (optional)
    pub end_char: Option<usize>,
}

/// Parse example code and extract helpful error information
fn parse_example_error(code: &str) -> String {
    // Try to parse with syn to get detailed error information
    match syn::parse_str::<syn::Expr>(code) {
        Ok(_) => String::new(), // Valid, no error
        Err(e) => {
            let error_msg = e.to_string();
            let lower = error_msg.to_lowercase();

            // Provide user-friendly hints for common issues
            // These use broad pattern matching to be resilient to minor message changes
            let hint = if lower.contains("end of input") || lower.contains("eof") {
                Some("Incomplete expression - check for missing closing delimiters")
            } else if lower.contains("expected") && lower.contains(",") {
                Some("Missing comma between fields or elements")
            } else if lower.contains("expected") && lower.contains("}") {
                Some("Possible missing field in struct initialization")
            } else if lower.contains("identifier") {
                Some("Check field names and syntax")
            } else {
                None
            };

            match hint {
                Some(h) => format!("{}\nDetails: {}", h, error_msg),
                None => format!("Syntax error: {}", error_msg),
            }
        }
    }
}

/// Validate Rovo annotations in the given content
///
/// Checks for issues like invalid HTTP status codes and example syntax errors.
///
/// # Arguments
/// * `content` - The source code content to validate
///
/// # Returns
/// A vector of diagnostics for any validation errors found
pub fn validate_annotations(content: &str) -> Vec<Diagnostic> {
    let annotations = crate::parser::parse_annotations(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut diagnostics = Vec::new();

    for ann in annotations {
        match ann.kind {
            AnnotationKind::Response => {
                if let Some(status) = ann.status {
                    if status < 100 || status > 599 {
                        // Find the position of the status code in the line
                        let (char_start, char_end) = if ann.line < lines.len() {
                            let line = lines[ann.line];
                            let status_str = status.to_string();
                            if let Some(pos) = line.find(&status_str) {
                                (Some(pos), Some(pos + status_str.len()))
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };

                        diagnostics.push(Diagnostic {
                            line: ann.line,
                            message: format!(
                                "Invalid HTTP status code: {}. Must be between 100 and 599.",
                                status
                            ),
                            severity: DiagnosticSeverity::Error,
                            char_start,
                            char_end,
                            end_line: None,
                            end_char: None,
                        });
                    }
                }
            }
            AnnotationKind::Example => {
                // Validate example syntax
                if let Some(example_code) = ann.example_value {
                    let error_msg = parse_example_error(&example_code);
                    if !error_msg.is_empty() {
                        // Find the start and end lines for multi-line examples
                        let start_line = ann.line;
                        let mut end_line = start_line;

                        // Scan forward to find the end of the example
                        for i in (start_line + 1)..lines.len() {
                            let line = lines[i].trim();
                            if line.starts_with("///") {
                                let content = line.trim_start_matches("///").trim();

                                // Check if this line starts a new example entry (STATUS: ...)
                                let starts_new_example = content
                                    .chars()
                                    .next()
                                    .map(|c| c.is_ascii_digit())
                                    .unwrap_or(false)
                                    && content.contains(':');

                                // Check if this line is part of the current example
                                if content.is_empty()
                                    || content.starts_with('#')
                                    || content.starts_with('@')
                                    || starts_new_example
                                {
                                    break;
                                }
                                // Update end_line for each content line we find
                                end_line = i;
                            } else {
                                break;
                            }
                        }

                        let line_content = lines.get(start_line).unwrap_or(&"");
                        // Find the expression start (after "STATUS:") for better highlighting
                        let char_start = line_content.find(':').map(|pos| {
                            // Skip past the colon and any whitespace
                            let after_colon = &line_content[pos + 1..];
                            let trimmed_start = after_colon.len() - after_colon.trim_start().len();
                            pos + 1 + trimmed_start
                        });

                        diagnostics.push(Diagnostic {
                            line: start_line,
                            message: format!("Invalid example expression.\n{}", error_msg),
                            severity: DiagnosticSeverity::Error,
                            char_start,
                            char_end: None,
                            end_line: if end_line > start_line {
                                Some(end_line)
                            } else {
                                None
                            },
                            end_char: if end_line > start_line {
                                lines.get(end_line).map(|l| l.len())
                            } else {
                                None
                            },
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}
