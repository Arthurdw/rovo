mod annotations;
mod error;
mod tokens;
pub mod types;

pub use error::ParseError;
pub use types::{DocInfo, FuncItem, PathParamDoc, PathParamInfo};

use proc_macro2::{Span, TokenStream, TokenTree};
use types::DocLine;

use crate::utils::find_closest_annotation;

/// Special depth value indicating code block mode for multi-line examples
const CODE_BLOCK_MODE: usize = usize::MAX - 1;

/// Parse a function annotated with #[rovo]
pub fn parse_rovo_function(input: TokenStream) -> Result<(FuncItem, DocInfo), ParseError> {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();

    // Extract doc comments, attributes, and function name
    let mut doc_lines = Vec::new();
    let mut func_name = None;
    let mut is_deprecated = false;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                // Check if this is an attribute
                if i + 1 < tokens.len() {
                    if let TokenTree::Group(group) = &tokens[i + 1] {
                        let attr_content = group.stream().to_string();
                        if attr_content.starts_with("doc") {
                            // Extract the doc comment text and preserve the span
                            let doc_text = tokens::extract_doc_text(&attr_content);
                            let span = group.span();
                            doc_lines.push(DocLine {
                                text: doc_text,
                                span,
                            });
                        } else if attr_content.starts_with("deprecated") {
                            // Mark as deprecated
                            is_deprecated = true;
                        }
                    }
                }
                i += 1;
            }
            TokenTree::Ident(ident) if *ident == "fn" => {
                // Next token should be the function name
                if i + 1 < tokens.len() {
                    if let TokenTree::Ident(name) = &tokens[i + 1] {
                        func_name = Some(name.clone());
                    }
                }
                break;
            }
            _ => i += 1,
        }
    }

    let func_name = func_name.ok_or_else(|| ParseError::new("Could not find function name"))?;

    // Extract state type from function parameters
    let state_type = tokens::extract_state_type(&input);

    // Extract path parameter info from function signature
    let path_params = tokens::extract_path_info(&input);

    // Parse doc comments
    let mut doc_info = parse_doc_comments(&doc_lines)?;

    // Set deprecated flag from Rust attribute
    doc_info.deprecated = is_deprecated;

    // Validate that documented path parameters match function signature bindings
    if !doc_info.path_params.is_empty() {
        if let Some(ref sig_params) = path_params {
            // Only validate for primitive types (not struct patterns)
            if !sig_params.is_struct_pattern {
                for doc_param in &doc_info.path_params {
                    if !sig_params.bindings.contains(&doc_param.name) {
                        let bindings_list = sig_params.bindings.join(", ");
                        return Err(ParseError::with_span(
                            format!(
                                "Documented path parameter '{}' does not match any parameter in function signature\n\
                                 help: found parameters: {}\n\
                                 note: parameter names in # Path Parameters must match the binding names in Path(...)",
                                doc_param.name,
                                bindings_list
                            ),
                            doc_param.span,
                        ));
                    }
                }
            }
        } else {
            // Documented path params but no Path<T> in signature
            let first_param = &doc_info.path_params[0];
            return Err(ParseError::with_span(
                format!(
                    "Documented path parameter '{}' but function has no Path<T> extractor\n\
                     help: add a Path<T> parameter to your function signature",
                    first_param.name
                ),
                first_param.span,
            ));
        }
    }

    let func_item = FuncItem {
        name: func_name,
        tokens: input,
        state_type,
        path_params,
    };

    Ok((func_item, doc_info))
}

/// Parse doc comments and extract documentation info
#[allow(clippy::cognitive_complexity)]
fn parse_doc_comments(lines: &[DocLine]) -> Result<DocInfo, ParseError> {
    let mut doc_info = DocInfo::default();
    let mut description_lines = Vec::new();
    let mut in_description = false;
    let mut title_set = false;
    let mut current_section: Option<&str> = None;
    let mut pending_response: Option<(u16, String, String, Span)> = None; // (status, type, desc, span)
    let mut pending_example: Option<(u16, String, Span, usize)> = None; // (status, code, span, depth)

    for doc_line in lines {
        let trimmed = doc_line.text.trim();
        let span = doc_line.span;

        // Check for @rovo-ignore first (location-independent)
        if trimmed == "@rovo-ignore" {
            break;
        }

        // Check if we're starting a markdown section
        if trimmed.starts_with("# ") {
            // Finalize any pending multi-line content
            if let Some((status, type_str, desc, sp)) = pending_response.take() {
                let response_info =
                    annotations::parse_response_from_parts(&type_str, status, &desc, sp)?;
                doc_info.responses.push(response_info);
            }
            if let Some((status, code, sp, _)) = pending_example.take() {
                let example_info = annotations::parse_example_from_parts(status, &code, sp)?;
                doc_info.examples.push(example_info);
            }

            let section_name = trimmed.trim_start_matches("# ").trim();
            current_section = match section_name {
                "Responses" => Some("responses"),
                "Examples" => Some("examples"),
                "Metadata" => Some("metadata"),
                "Path Parameters" => Some("path_parameters"),
                _ => None, // Unknown section - ignore
            };
            continue;
        }

        match current_section {
            Some("responses") if !trimmed.is_empty() => {
                // Check if this line starts a new response or continues the previous one
                if let Some(colon_pos) = trimmed.find(':') {
                    let before_colon = &trimmed[..colon_pos];
                    if before_colon.chars().all(|c| c.is_ascii_digit()) {
                        // This is a new response line
                        // First, finalize any pending response
                        if let Some((status, type_str, desc, sp)) = pending_response.take() {
                            let response_info = annotations::parse_response_from_parts(
                                &type_str, status, &desc, sp,
                            )?;
                            doc_info.responses.push(response_info);
                        }

                        // Parse the new response line
                        let status_code = before_colon.parse::<u16>().map_err(|_| {
                            ParseError::with_span(
                                format!("Invalid status code '{before_colon}'"),
                                span,
                            )
                        })?;

                        let after_colon = trimmed[colon_pos + 1..].trim();
                        if let Some(dash_pos) = after_colon.find(" - ") {
                            let type_str = after_colon[..dash_pos].trim().to_string();
                            let description = after_colon[dash_pos + 3..].trim().to_string();
                            pending_response = Some((status_code, type_str, description, span));
                        } else {
                            return Err(ParseError::with_span(
                                "Invalid response format. Expected: <status>: <type> - <description>",
                                span,
                            ));
                        }
                    } else if let Some((_, _, ref mut desc, _)) = pending_response {
                        // Continuation line for description
                        desc.push(' ');
                        desc.push_str(trimmed);
                    }
                } else if let Some((_, _, ref mut desc, _)) = pending_response {
                    // Continuation line for description (no colon)
                    desc.push(' ');
                    desc.push_str(trimmed);
                }
            }
            Some("examples") if !trimmed.is_empty() => {
                // Check if we have a pending example that needs more lines
                if let Some((status, ref mut code, sp, ref mut depth)) = pending_example {
                    if *depth == CODE_BLOCK_MODE {
                        // In code block mode - looking for closing backticks
                        if trimmed == "```" && !code.is_empty() {
                            // Found closing backticks, finalize the example
                            let final_code = code.clone();
                            let example_info =
                                annotations::parse_example_from_parts(status, &final_code, sp)?;
                            doc_info.examples.push(example_info);
                            pending_example = None;
                        } else if code.is_empty()
                            && (trimmed == "```" || trimmed == "```rust" || trimmed == "```rs")
                        {
                            // Opening backticks on their own line - skip
                        } else {
                            // Regular content line in code block
                            if !code.is_empty() {
                                code.push('\n');
                            }
                            code.push_str(trimmed);
                        }
                    } else {
                        // Check if first line is switching to code block mode
                        if code.is_empty()
                            && (trimmed == "```" || trimmed == "```rust" || trimmed == "```rs")
                        {
                            // Switch to code block mode
                            *depth = CODE_BLOCK_MODE;
                        } else {
                            // Normal bracket/brace tracking mode
                            // Add this line to the pending example
                            if !code.is_empty() {
                                code.push('\n');
                            }
                            code.push_str(trimmed);

                            // Update bracket/brace depth
                            for ch in trimmed.chars() {
                                match ch {
                                    '{' | '[' | '(' => *depth += 1,
                                    '}' | ']' | ')' => *depth = depth.saturating_sub(1),
                                    _ => {}
                                }
                            }

                            // If depth is 0 and we have meaningful content, finalize it
                            if *depth == 0 && !code.trim().is_empty() {
                                let final_code = code.clone();
                                let example_info =
                                    annotations::parse_example_from_parts(status, &final_code, sp)?;
                                doc_info.examples.push(example_info);
                                pending_example = None;
                            }
                        }
                    }
                } else if let Some(colon_pos) = trimmed.find(':') {
                    let before_colon = &trimmed[..colon_pos];
                    if before_colon.chars().all(|c| c.is_ascii_digit()) {
                        // This is a new example line
                        let status_code = before_colon.parse::<u16>().map_err(|_| {
                            ParseError::with_span(
                                format!("Invalid status code '{before_colon}'"),
                                span,
                            )
                        })?;

                        let code = trimmed[colon_pos + 1..].trim().to_string();

                        // Check if code starts with triple backticks (code block on same line)
                        if code == "```" || code == "```rust" || code == "```rs" {
                            // Start code block mode
                            pending_example =
                                Some((status_code, String::new(), span, CODE_BLOCK_MODE));
                        } else if code.is_empty() {
                            // Store pending example with empty code, depth 0 (will accumulate on next lines)
                            pending_example = Some((status_code, String::new(), span, 0));
                        } else {
                            // Calculate bracket/brace depth
                            let mut depth: usize = 0;
                            for ch in code.chars() {
                                match ch {
                                    '{' | '[' | '(' => depth += 1,
                                    '}' | ']' | ')' => depth = depth.saturating_sub(1),
                                    _ => {}
                                }
                            }

                            if depth == 0 {
                                // Single-line example, process immediately
                                let example_info = annotations::parse_example_from_parts(
                                    status_code,
                                    &code,
                                    span,
                                )?;
                                doc_info.examples.push(example_info);
                            } else {
                                // Multi-line example, store for continuation
                                pending_example = Some((status_code, code, span, depth));
                            }
                        }
                    }
                }
            }
            Some("metadata") if !trimmed.is_empty() => {
                // Parse annotations in metadata section
                if trimmed.starts_with("@tag") {
                    let tag = annotations::parse_tag(trimmed, span)?;
                    doc_info.tags.push(tag);
                } else if trimmed.starts_with("@security") {
                    let scheme = annotations::parse_security(trimmed, span)?;
                    doc_info.security_requirements.push(scheme);
                } else if trimmed.starts_with("@id") {
                    let id = annotations::parse_id(trimmed, span)?;
                    doc_info.operation_id = Some(id);
                } else if trimmed == "@hidden" {
                    doc_info.hidden = true;
                } else if trimmed.starts_with('@') {
                    // Unknown annotation in metadata section
                    let annotation = trimmed.split_whitespace().next().unwrap_or(trimmed);
                    let annotation_name = annotation.strip_prefix('@').unwrap_or(annotation);

                    let error_msg = find_closest_annotation(annotation_name).map_or_else(
                        || {
                            format!(
                                "Unknown annotation '{annotation}'\n\
                             note: valid annotations are @tag, @security, @id, @hidden"
                            )
                        },
                        |suggestion| {
                            format!(
                                "Unknown annotation '{annotation}'\n\
                             help: did you mean '@{suggestion}'?\n\
                             note: valid annotations are @tag, @security, @id, @hidden"
                            )
                        },
                    );

                    return Err(ParseError::with_span(error_msg, span));
                }
            }
            Some("path_parameters") if !trimmed.is_empty() => {
                // Parse path parameter documentation
                // Format: "name: description"
                if let Some(colon_pos) = trimmed.find(':') {
                    let name = trimmed[..colon_pos].trim().to_string();
                    let description = trimmed[colon_pos + 1..].trim().to_string();
                    doc_info.path_params.push(PathParamDoc {
                        name,
                        description,
                        span,
                    });
                }
            }
            None if !trimmed.is_empty() => {
                // Not in a section - this is title or description
                if title_set {
                    in_description = true;
                    description_lines.push(trimmed.to_string());
                } else {
                    doc_info.title = Some(trimmed.to_string());
                    title_set = true;
                }
            }
            None if trimmed.is_empty() && in_description => {
                // Empty line in description
                description_lines.push(String::new());
            }
            _ => {
                // Empty line or unrecognized content in a section - skip
            }
        }
    }

    // Finalize any remaining pending content
    if let Some((status, type_str, desc, sp)) = pending_response {
        let response_info = annotations::parse_response_from_parts(&type_str, status, &desc, sp)?;
        doc_info.responses.push(response_info);
    }
    if let Some((status, code, sp, _)) = pending_example {
        let example_info = annotations::parse_example_from_parts(status, &code, sp)?;
        doc_info.examples.push(example_info);
    }

    if !description_lines.is_empty() {
        doc_info.description = Some(description_lines.join("\n").trim().to_string());
    }

    // Validate that all example status codes are defined in responses
    if !doc_info.examples.is_empty() && !doc_info.responses.is_empty() {
        let response_codes: std::collections::HashSet<u16> =
            doc_info.responses.iter().map(|r| r.status_code).collect();

        for example in &doc_info.examples {
            if !response_codes.contains(&example.status_code) {
                let available_codes: Vec<String> = doc_info
                    .responses
                    .iter()
                    .map(|r| r.status_code.to_string())
                    .collect();
                let available_list = available_codes.join(", ");

                return Err(ParseError::with_span(
                    format!(
                        "Example status code {} is not defined in responses. Available status codes: {}",
                        example.status_code,
                        available_list
                    ),
                    example.span,
                ));
            }
        }
    }

    Ok(doc_info)
}
