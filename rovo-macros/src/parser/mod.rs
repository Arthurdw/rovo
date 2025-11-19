mod annotations;
mod error;
mod tokens;
mod types;

pub use error::ParseError;
pub use types::{DocInfo, FuncItem};

use proc_macro2::{TokenStream, TokenTree};
use types::DocLine;

use crate::utils::find_closest_annotation;

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

    // Parse doc comments
    let mut doc_info = parse_doc_comments(&doc_lines)?;

    // Set deprecated flag from Rust attribute
    doc_info.deprecated = is_deprecated;

    let func_item = FuncItem {
        name: func_name,
        tokens: input,
        state_type,
    };

    Ok((func_item, doc_info))
}

/// Parse doc comments and extract documentation info
fn parse_doc_comments(lines: &[DocLine]) -> Result<DocInfo, ParseError> {
    let mut doc_info = DocInfo::default();
    let mut description_lines = Vec::new();
    let mut in_description = false;
    let mut title_set = false;

    for doc_line in lines {
        let trimmed = doc_line.text.trim();
        let span = doc_line.span;

        if trimmed.starts_with("@response") {
            let response_info = annotations::parse_response(trimmed, span)?;
            doc_info.responses.push(response_info);
        } else if trimmed.starts_with("@example") {
            let example_info = annotations::parse_example(trimmed, span)?;
            doc_info.examples.push(example_info);
        } else if trimmed.starts_with("@tag") {
            let tag = annotations::parse_tag(trimmed, span)?;
            doc_info.tags.push(tag);
        } else if trimmed.starts_with("@security") {
            let scheme = annotations::parse_security(trimmed, span)?;
            doc_info.security_requirements.push(scheme);
        } else if trimmed.starts_with("@id") {
            let id = annotations::parse_id(trimmed, span)?;
            doc_info.operation_id = Some(id);
        } else if trimmed == "@hidden" {
            // Mark as hidden
            doc_info.hidden = true;
        } else if trimmed == "@rovo-ignore" {
            // Stop processing further doc comments
            break;
        } else if trimmed.starts_with('@') {
            // Unknown annotation
            let annotation = trimmed.split_whitespace().next().unwrap_or(trimmed);
            let annotation_name = annotation.strip_prefix('@').unwrap_or(annotation);

            let error_msg = find_closest_annotation(annotation_name).map_or_else(
                || {
                    format!(
                        "Unknown annotation '{annotation}'\n\
                     note: valid annotations are @response, @example, @tag, @security, @id, @hidden, @rovo-ignore"
                    )
                },
                |suggestion| {
                    format!(
                        "Unknown annotation '{annotation}'\n\
                     help: did you mean '@{suggestion}'?\n\
                     note: valid annotations are @response, @example, @tag, @security, @id, @hidden, @rovo-ignore"
                    )
                },
            );

            return Err(ParseError::with_span(error_msg, span));
        } else if !trimmed.is_empty() {
            if title_set {
                in_description = true;
                description_lines.push(trimmed.to_string());
            } else {
                doc_info.title = Some(trimmed.to_string());
                title_set = true;
            }
        } else if in_description {
            // Empty line in description - continue collecting
            description_lines.push(String::new());
        }
    }

    if !description_lines.is_empty() {
        doc_info.description = Some(description_lines.join("\n").trim().to_string());
    }

    Ok(doc_info)
}
