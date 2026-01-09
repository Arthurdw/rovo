use crate::completion;
use crate::diagnostics;
use crate::utils::{byte_index_to_utf16_col, utf16_pos_to_byte_index};
use tower_lsp::lsp_types::*;

/// Handle completion request for a text document
///
/// # Arguments
/// * `content` - The document content
/// * `position` - Cursor position where completion was requested
///
/// # Returns
/// Completion suggestions if available
pub fn text_document_completion(content: &str, position: Position) -> Option<CompletionResponse> {
    // Only provide completions if we're near a #[rovo] attribute
    if !crate::parser::is_near_rovo_attribute(content, position.line as usize) {
        return None;
    }

    let pos = completion::Position {
        line: position.line as usize,
        character: position.character as usize,
    };

    let items = completion::get_completions(content, pos);

    if items.is_empty() {
        return None;
    }

    let lsp_items: Vec<CompletionItem> = items
        .into_iter()
        .map(|item| {
            let kind = match item.kind {
                completion::CompletionItemKind::Keyword => CompletionItemKind::KEYWORD,
                completion::CompletionItemKind::Snippet => CompletionItemKind::SNIPPET,
            };

            CompletionItem {
                label: item.label,
                kind: Some(kind),
                detail: item.detail,
                documentation: item.documentation.map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc,
                    })
                }),
                insert_text: item.insert_text.clone(),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            }
        })
        .collect();

    Some(CompletionResponse::Array(lsp_items))
}

/// Handle hover request for a text document
///
/// Provides information when hovering over status codes, security schemes, or types.
///
/// # Arguments
/// * `content` - The document content
/// * `position` - Cursor position where hover was requested
///
/// # Returns
/// Hover information if available
pub fn text_document_hover(content: &str, position: Position) -> Option<Hover> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    // Only provide hover if we're near a #[rovo] attribute
    if !crate::parser::is_near_rovo_attribute(content, line_idx) {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;

    // Check if cursor is on a status code
    if let Some(status_info) = get_status_code_at_position(line, char_idx) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: status_info,
            }),
            range: None,
        });
    }

    // Check if cursor is on a security scheme
    if let Some(scheme_info) = get_security_scheme_at_position(line, char_idx) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: scheme_info,
            }),
            range: None,
        });
    }

    // First, check if cursor is on a type in an annotation
    if let Some((response_type, _, _)) = crate::type_resolver::get_type_at_position(line, char_idx)
    {
        if let Some(type_name) = crate::type_resolver::extract_type_from_response(&response_type) {
            if let Some(def_line) = crate::type_resolver::find_type_definition(content, &type_name)
            {
                let hover_text = format!(
                    "**{}**\n\nDefined at line {}\n\n```rust\n{}\n```",
                    type_name,
                    def_line + 1,
                    lines.get(def_line).unwrap_or(&"")
                );

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                });
            }
        }
    }

    // Check if cursor is on an annotation keyword
    if let Some(annotation_type) = get_annotation_at_position(line, char_idx) {
        let documentation = crate::docs::get_annotation_documentation(&annotation_type);

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: documentation.to_string(),
            }),
            range: None,
        });
    }

    None
}

fn get_annotation_at_position(line: &str, char_idx: usize) -> Option<String> {
    // Check if we're in a doc comment
    if !line.trim_start().starts_with("///") {
        return None;
    }

    let content = line.trim_start().trim_start_matches("///").trim();

    // Check for section headers first
    if content.starts_with("# ") {
        let section_name = content.trim_start_matches("# ").trim();
        let section_start = line.find('#').unwrap_or(0);
        let section_end = section_start + content.len();

        if char_idx >= section_start && char_idx <= section_end {
            match section_name {
                "Path Parameters" => return Some("section:path-parameters".to_string()),
                "Responses" => return Some("section:responses".to_string()),
                "Examples" => return Some("section:examples".to_string()),
                "Metadata" => return Some("section:metadata".to_string()),
                _ => {}
            }
        }
    }

    // Find the annotation keyword at the cursor position (for metadata section)
    let annotations = ["@tag", "@security", "@id", "@hidden"];

    for annotation in annotations {
        if let Some(pos) = line.find(annotation) {
            let end = pos + annotation.len();
            if char_idx >= pos && char_idx <= end {
                return Some(annotation.to_string());
            }
        }
    }

    None
}

/// Handle document change and return diagnostics
///
/// # Arguments
/// * `content` - The updated document content
/// * `_uri` - Document URI (currently unused)
///
/// # Returns
/// A vector of diagnostics for any validation errors
pub fn text_document_did_change(content: &str, _uri: Url) -> Vec<Diagnostic> {
    let diagnostics_list = diagnostics::validate_annotations(content);
    let lines: Vec<&str> = content.lines().collect();

    diagnostics_list
        .into_iter()
        .map(|diag| {
            let severity = match diag.severity {
                diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            };

            // Get the line content to convert byte indices to UTF-16 positions
            let line = lines.get(diag.line).map(|l| *l).unwrap_or("");
            let line_utf16_len = byte_index_to_utf16_col(line, line.len());

            // Convert byte indices to UTF-16 code unit offsets
            let char_start = diag
                .char_start
                .map(|idx| byte_index_to_utf16_col(line, idx))
                .unwrap_or(0);
            let char_end = diag
                .char_end
                .map(|idx| byte_index_to_utf16_col(line, idx))
                .unwrap_or(line_utf16_len);

            // Handle multi-line diagnostics
            let (end_line, end_char) = if let Some(end_line_num) = diag.end_line {
                let end_line_content = lines.get(end_line_num).map(|l| *l).unwrap_or("");
                let end_line_utf16_len =
                    byte_index_to_utf16_col(end_line_content, end_line_content.len());
                let end_char_pos = diag
                    .end_char
                    .map(|idx| byte_index_to_utf16_col(end_line_content, idx))
                    .unwrap_or(end_line_utf16_len);
                (end_line_num as u32, end_char_pos as u32)
            } else {
                (diag.line as u32, char_end as u32)
            };

            Diagnostic {
                range: Range {
                    start: Position {
                        line: diag.line as u32,
                        character: char_start as u32,
                    },
                    end: Position {
                        line: end_line,
                        character: end_char,
                    },
                },
                severity: Some(severity),
                source: Some("rovo-lsp".to_string()),
                message: diag.message,
                code: None,
                code_description: None,
                related_information: None,
                tags: None,
                data: None,
            }
        })
        .collect()
}

/// Find all references to a tag in the document
///
/// # Arguments
/// * `content` - The document content
/// * `position` - Cursor position on a tag annotation
/// * `uri` - Document URI for constructing locations
///
/// # Returns
/// A vector of locations where the tag is referenced
/// Find references to a path parameter (doc, binding, and body usages)
pub fn find_path_param_references(
    content: &str,
    position: Position,
    uri: Url,
) -> Option<Vec<Location>> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;

    // Get the param name from either doc or binding
    let param_name = get_path_param_at_position(content, line_idx, char_idx)
        .map(|(name, _)| name)
        .or_else(|| get_path_binding_at_position(content, line_idx, char_idx).map(|(name, _)| name))
        .or_else(|| get_path_param_usage_at_position(content, line_idx, char_idx))?;

    // Find the rovo block boundaries
    let (doc_start, fn_end) = find_rovo_block_boundaries(content, line_idx)?;

    let mut locations = Vec::new();

    // Find in # Path Parameters section
    let mut in_path_params = false;
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        let trimmed = line.trim_start().trim_start_matches("///").trim();

        if trimmed.starts_with("# ") {
            in_path_params = trimmed == "# Path Parameters";
            continue;
        }

        if in_path_params && !trimmed.is_empty() {
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim();
                if name == param_name {
                    let doc_start_pos = line.find("///")? + 3;
                    let content_after = &line[doc_start_pos..];
                    let leading_ws = content_after.len() - content_after.trim_start().len();
                    let name_start = doc_start_pos + leading_ws;
                    let name_end = name_start + name.len();

                    let start_utf16 = byte_index_to_utf16_col(line, name_start);
                    let end_utf16 = byte_index_to_utf16_col(line, name_end);

                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: start_utf16 as u32,
                            },
                            end: Position {
                                line: idx as u32,
                                character: end_utf16 as u32,
                            },
                        },
                    });
                }
            }
        }
    }

    // Find in Path binding
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        if let Some(path_pos) = line.find("Path(") {
            let after_path = &line[path_pos + 5..];

            let (bindings_str, bindings_start) = if after_path.starts_with('(') {
                let close = after_path.find(')')?;
                (&after_path[1..close], path_pos + 6)
            } else {
                let close = after_path.find(')')?;
                (&after_path[..close], path_pos + 5)
            };

            let mut current_pos = bindings_start;
            for binding in bindings_str.split(',') {
                let binding = binding.trim();
                if binding == param_name {
                    if let Some(rel_pos) = line[current_pos..].find(binding) {
                        let abs_start = current_pos + rel_pos;
                        let abs_end = abs_start + binding.len();

                        let start_utf16 = byte_index_to_utf16_col(line, abs_start);
                        let end_utf16 = byte_index_to_utf16_col(line, abs_end);

                        locations.push(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: idx as u32,
                                    character: start_utf16 as u32,
                                },
                                end: Position {
                                    line: idx as u32,
                                    character: end_utf16 as u32,
                                },
                            },
                        });
                    }
                }
                if let Some(rel_pos) = line[current_pos..].find(binding) {
                    current_pos = current_pos + rel_pos + binding.len();
                }
            }
        }
    }

    // Find usages in function body
    let word_pattern = format!(r"\b{}\b", regex::escape(&param_name));
    let re = regex::Regex::new(&word_pattern).ok()?;

    let mut in_fn_body = false;
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        if !in_fn_body {
            if line.contains('{') {
                in_fn_body = true;
            }
            continue;
        }

        for mat in re.find_iter(line) {
            let start_utf16 = byte_index_to_utf16_col(line, mat.start());
            let end_utf16 = byte_index_to_utf16_col(line, mat.end());

            locations.push(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: idx as u32,
                        character: start_utf16 as u32,
                    },
                    end: Position {
                        line: idx as u32,
                        character: end_utf16 as u32,
                    },
                },
            });
        }
    }

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

/// Go to the definition of a path parameter (the doc comment)
pub fn goto_path_param_definition(content: &str, position: Position, uri: Url) -> Option<Location> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;

    // Get the param name from doc, binding, or body usage
    let doc_result = get_path_param_at_position(content, line_idx, char_idx);
    let binding_result = get_path_binding_at_position(content, line_idx, char_idx);

    // If we're on the doc comment, go to the binding instead
    if let Some((param_name, _)) = doc_result {
        return goto_path_param_binding(content, line_idx, &param_name, uri);
    }

    let param_name = binding_result
        .map(|(name, _)| name)
        .or_else(|| get_path_param_usage_at_position(content, line_idx, char_idx))?;

    // Find the rovo block boundaries
    let (doc_start, fn_end) = find_rovo_block_boundaries(content, line_idx)?;

    // Find the definition in # Path Parameters section
    let mut in_path_params = false;
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        let trimmed = line.trim_start().trim_start_matches("///").trim();

        if trimmed.starts_with("# ") {
            in_path_params = trimmed == "# Path Parameters";
            continue;
        }

        if in_path_params && !trimmed.is_empty() {
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim();
                if name == param_name {
                    let doc_start_pos = line.find("///")? + 3;
                    let content_after = &line[doc_start_pos..];
                    let leading_ws = content_after.len() - content_after.trim_start().len();
                    let name_start = doc_start_pos + leading_ws;
                    let name_end = name_start + name.len();

                    let start_utf16 = byte_index_to_utf16_col(line, name_start);
                    let end_utf16 = byte_index_to_utf16_col(line, name_end);

                    return Some(Location {
                        uri,
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: start_utf16 as u32,
                            },
                            end: Position {
                                line: idx as u32,
                                character: end_utf16 as u32,
                            },
                        },
                    });
                }
            }
        }
    }

    None
}

/// Go to the binding location of a path parameter (from doc to signature)
fn goto_path_param_binding(
    content: &str,
    line_idx: usize,
    param_name: &str,
    uri: Url,
) -> Option<Location> {
    let lines: Vec<&str> = content.lines().collect();
    let (doc_start, fn_end) = find_rovo_block_boundaries(content, line_idx)?;

    // Find the Path( binding in the function signature
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        if let Some(path_pos) = line.find("Path(") {
            let after_path = &line[path_pos + 5..];

            let (bindings_str, bindings_start) = if after_path.starts_with('(') {
                let close = after_path.find(')')?;
                (&after_path[1..close], path_pos + 6)
            } else {
                let close = after_path.find(')')?;
                (&after_path[..close], path_pos + 5)
            };

            let mut current_pos = bindings_start;
            for binding in bindings_str.split(',') {
                let binding = binding.trim();
                if binding == param_name {
                    if let Some(rel_pos) = line[current_pos..].find(binding) {
                        let abs_start = current_pos + rel_pos;
                        let abs_end = abs_start + binding.len();

                        let start_utf16 = byte_index_to_utf16_col(line, abs_start);
                        let end_utf16 = byte_index_to_utf16_col(line, abs_end);

                        return Some(Location {
                            uri,
                            range: Range {
                                start: Position {
                                    line: idx as u32,
                                    character: start_utf16 as u32,
                                },
                                end: Position {
                                    line: idx as u32,
                                    character: end_utf16 as u32,
                                },
                            },
                        });
                    }
                }
                if let Some(rel_pos) = line[current_pos..].find(binding) {
                    current_pos = current_pos + rel_pos + binding.len();
                }
            }
        }
    }

    None
}

/// Get path param usage at position (variable in function body)
fn get_path_param_usage_at_position(
    content: &str,
    line_idx: usize,
    byte_idx: usize,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(line_idx)?;

    // Must NOT be in a doc comment
    if line.trim_start().starts_with("///") {
        return None;
    }

    // Check if we're in a function body (after {)
    let (doc_start, _) = find_rovo_block_boundaries(content, line_idx)?;

    let mut in_fn_body = false;
    for check_line in lines.iter().skip(doc_start).take(line_idx - doc_start + 1) {
        if check_line.contains('{') {
            in_fn_body = true;
            break;
        }
    }

    if !in_fn_body {
        return None;
    }

    // Convert byte index to character index for non-ASCII support
    let char_idx = line[..byte_idx.min(line.len())].chars().count();

    // Extract identifier at position
    let mut start = char_idx;
    let mut end = char_idx;
    let chars: Vec<char> = line.chars().collect();

    // Find start of identifier
    while start > 0 {
        let prev = start - 1;
        if prev < chars.len() && (chars[prev].is_alphanumeric() || chars[prev] == '_') {
            start = prev;
        } else {
            break;
        }
    }

    // Find end of identifier
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start == end {
        return None;
    }

    let ident: String = chars[start..end].iter().collect();
    if ident.is_empty() || ident.chars().next()?.is_numeric() {
        return None;
    }

    Some(ident)
}

pub fn find_tag_references(content: &str, position: Position, uri: Url) -> Option<Vec<Location>> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];

    // Extract tag name from current position
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;
    let tag_name = extract_tag_at_position(line, char_idx)?;

    // Find all references to this tag in the document
    let mut locations = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        // Look for @tag annotations
        if let Some(pos) = line.find("@tag") {
            // Extract the tag name from this line
            let raw_after_tag = &line[pos + 4..];
            let trimmed_after_tag = raw_after_tag.trim_start();
            let tag_in_line = trimmed_after_tag.split_whitespace().next().unwrap_or("");

            if tag_in_line == tag_name {
                // Found a reference!
                let whitespace = raw_after_tag.len() - trimmed_after_tag.len();
                let start_byte = pos;
                let end_byte = pos + 4 + whitespace + tag_name.len();

                // Convert byte offsets to UTF-16 columns for LSP positions
                let start_char = byte_index_to_utf16_col(line, start_byte);
                let end_char = byte_index_to_utf16_col(line, end_byte);

                locations.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: idx as u32,
                            character: start_char as u32,
                        },
                        end: Position {
                            line: idx as u32,
                            character: end_char as u32,
                        },
                    },
                });
            }
        }
    }

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

fn extract_tag_at_position(line: &str, char_idx: usize) -> Option<String> {
    // Check if we're in a doc comment with @tag
    if !line.trim_start().starts_with("///") {
        return None;
    }

    // Find @tag in the line
    let tag_pos = line.find("@tag")?;

    // Get the part after @tag (untrimmed)
    let raw_after_tag = &line[tag_pos + 4..];

    // Trim to get the tag name part
    let trimmed_after_tag = raw_after_tag.trim_start();

    // Extract the tag name (first word)
    let tag_name = trimmed_after_tag.split_whitespace().next()?;

    // Calculate whitespace before tag name
    let whitespace = raw_after_tag.len() - trimmed_after_tag.len();

    // Check if cursor is on the @tag keyword or the tag name
    let tag_start = tag_pos + 4 + whitespace;
    let tag_end = tag_start + tag_name.len();

    if char_idx >= tag_pos && char_idx <= tag_end {
        Some(tag_name.to_string())
    } else {
        None
    }
}

/// Prepare rename - check if rename is possible at position and return the range
///
/// # Arguments
/// * `content` - The document content
/// * `position` - Cursor position to check
///
/// # Returns
/// The range and placeholder text for the rename, or None if not renameable
pub fn prepare_rename(content: &str, position: Position) -> Option<(Range, String)> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;

    // Check if we're on a path parameter in # Path Parameters section
    if let Some((name, range)) = get_path_param_at_position(content, line_idx, char_idx) {
        return Some((range, name));
    }

    // Check if we're on a Path binding in function signature
    // Only claim rename if there's a corresponding doc comment to update
    if let Some((name, range)) = get_path_binding_at_position(content, line_idx, char_idx) {
        // Check if this param is documented in # Path Parameters
        if has_path_param_doc(content, line_idx, &name) {
            return Some((range, name));
        }
        // No doc to update - let rust-analyzer handle the rename
        return None;
    }

    // Check for @tag rename (existing functionality)
    let tag_name = extract_tag_at_position(line, char_idx)?;

    // Find @tag in the line to get the range
    let tag_pos = line.find("@tag")?;
    let raw_after_tag = &line[tag_pos + 4..];
    let trimmed_after_tag = raw_after_tag.trim_start();
    let whitespace = raw_after_tag.len() - trimmed_after_tag.len();

    let tag_name_start = tag_pos + 4 + whitespace;
    let tag_name_end = tag_name_start + tag_name.len();

    // Convert byte indices to UTF-16 positions
    let start_utf16 = byte_index_to_utf16_col(line, tag_name_start);
    let end_utf16 = byte_index_to_utf16_col(line, tag_name_end);

    Some((
        Range {
            start: Position {
                line: line_idx as u32,
                character: start_utf16 as u32,
            },
            end: Position {
                line: line_idx as u32,
                character: end_utf16 as u32,
            },
        },
        tag_name,
    ))
}

/// Get path parameter name and range if cursor is on a path param in # Path Parameters section
fn get_path_param_at_position(
    content: &str,
    line_idx: usize,
    char_idx: usize,
) -> Option<(String, Range)> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(line_idx)?;

    // Must be a doc comment
    if !line.trim_start().starts_with("///") {
        return None;
    }

    // Check if we're in a # Path Parameters section
    let mut in_path_params_section = false;
    for i in (0..line_idx).rev() {
        let prev_line = lines.get(i)?;
        let trimmed = prev_line.trim_start().trim_start_matches("///").trim();

        if trimmed.starts_with("# ") {
            in_path_params_section = trimmed == "# Path Parameters";
            break;
        }

        // Stop if we hit non-doc-comment
        if !prev_line.trim_start().starts_with("///") {
            break;
        }
    }

    if !in_path_params_section {
        return None;
    }

    // Parse the path parameter line: "name: description"
    let doc_content = line.trim_start().trim_start_matches("///").trim();
    let colon_pos = doc_content.find(':')?;
    let param_name = doc_content[..colon_pos].trim();

    if param_name.is_empty() {
        return None;
    }

    // Find the position of the param name in the line
    let doc_start = line.find("///")? + 3;
    let content_after_slashes = &line[doc_start..];
    let leading_whitespace = content_after_slashes.len() - content_after_slashes.trim_start().len();
    let name_start = doc_start + leading_whitespace;
    let name_end = name_start + param_name.len();

    // Check if cursor is on the param name
    if char_idx < name_start || char_idx > name_end {
        return None;
    }

    let start_utf16 = byte_index_to_utf16_col(line, name_start);
    let end_utf16 = byte_index_to_utf16_col(line, name_end);

    Some((
        param_name.to_string(),
        Range {
            start: Position {
                line: line_idx as u32,
                character: start_utf16 as u32,
            },
            end: Position {
                line: line_idx as u32,
                character: end_utf16 as u32,
            },
        },
    ))
}

/// Get path binding name and range if cursor is on Path(name) in function signature
fn get_path_binding_at_position(
    content: &str,
    line_idx: usize,
    char_idx: usize,
) -> Option<(String, Range)> {
    let lines: Vec<&str> = content.lines().collect();
    let line = lines.get(line_idx)?;

    // Look for Path( pattern
    let path_pos = line.find("Path(")?;
    let after_path = &line[path_pos + 5..];

    // Handle tuple: Path((a, b))
    let (bindings_str, bindings_start) = if after_path.starts_with('(') {
        // Tuple pattern
        let close_paren = after_path.find(')')?;
        (&after_path[1..close_paren], path_pos + 6)
    } else {
        // Single binding: Path(name)
        let close_paren = after_path.find(')')?;
        (&after_path[..close_paren], path_pos + 5)
    };

    // Parse bindings
    let mut current_pos = bindings_start;
    for binding in bindings_str.split(',') {
        let binding = binding.trim();
        if binding.is_empty() {
            continue;
        }

        // Find the actual position of this binding in the line
        if let Some(rel_pos) = line[current_pos..].find(binding) {
            let abs_start = current_pos + rel_pos;
            let abs_end = abs_start + binding.len();

            // Check if cursor is on this binding
            if char_idx >= abs_start && char_idx <= abs_end {
                let start_utf16 = byte_index_to_utf16_col(line, abs_start);
                let end_utf16 = byte_index_to_utf16_col(line, abs_end);

                return Some((
                    binding.to_string(),
                    Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start_utf16 as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end_utf16 as u32,
                        },
                    },
                ));
            }

            current_pos = abs_end;
        }
    }

    None
}

/// Rename a tag and update all its references in the document
///
/// # Arguments
/// * `content` - The document content
/// * `position` - Cursor position on a tag annotation
/// * `new_name` - The new name for the tag
/// * `uri` - Document URI for constructing edit locations
///
/// # Returns
/// A WorkspaceEdit containing all the rename changes, or None if not on a tag
pub fn rename_tag(
    content: &str,
    position: Position,
    new_name: &str,
    uri: Url,
) -> Option<WorkspaceEdit> {
    let line_idx = position.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;

    // Check if we're renaming a path parameter
    if let Some(edit) = rename_path_parameter(content, line_idx, char_idx, new_name, uri.clone()) {
        return Some(edit);
    }

    // Otherwise, try tag rename
    let old_tag_name = extract_tag_at_position(line, char_idx)?;

    // Find all references and create text edits
    let mut text_edits = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        // Look for @tag annotations
        if let Some(pos) = line.find("@tag") {
            // Extract the tag name from this line
            let raw_after_tag = &line[pos + 4..];
            let trimmed_after_tag = raw_after_tag.trim_start();
            let tag_in_line = trimmed_after_tag.split_whitespace().next().unwrap_or("");

            if tag_in_line == old_tag_name {
                // Calculate positions for the tag name (not the @tag keyword)
                let whitespace = raw_after_tag.len() - trimmed_after_tag.len();
                let tag_name_start = pos + 4 + whitespace;
                let tag_name_end = tag_name_start + old_tag_name.len();

                // Convert byte indices to UTF-16 positions
                let start_utf16 = byte_index_to_utf16_col(line, tag_name_start);
                let end_utf16 = byte_index_to_utf16_col(line, tag_name_end);

                text_edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: idx as u32,
                            character: start_utf16 as u32,
                        },
                        end: Position {
                            line: idx as u32,
                            character: end_utf16 as u32,
                        },
                    },
                    new_text: new_name.to_string(),
                });
            }
        }
    }

    if text_edits.is_empty() {
        return None;
    }

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri, text_edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}

/// Rename a path parameter in both the doc comment and function signature
fn rename_path_parameter(
    content: &str,
    line_idx: usize,
    char_idx: usize,
    new_name: &str,
    uri: Url,
) -> Option<WorkspaceEdit> {
    let lines: Vec<&str> = content.lines().collect();

    // Try to get the old name from either doc or binding
    let old_name = get_path_param_at_position(content, line_idx, char_idx)
        .map(|(name, _)| name)
        .or_else(|| {
            get_path_binding_at_position(content, line_idx, char_idx).map(|(name, _)| name)
        })?;

    // Find the rovo block boundaries
    let (doc_start, fn_end) = find_rovo_block_boundaries(content, line_idx)?;

    let mut text_edits = Vec::new();

    // Find and rename in # Path Parameters section
    let mut in_path_params = false;
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        let trimmed = line.trim_start().trim_start_matches("///").trim();

        if trimmed.starts_with("# ") {
            in_path_params = trimmed == "# Path Parameters";
            continue;
        }

        if in_path_params && !trimmed.is_empty() {
            // Check if this line has a path param with the old name
            if let Some(colon_pos) = trimmed.find(':') {
                let param_name = trimmed[..colon_pos].trim();
                if param_name == old_name {
                    // Find position in original line
                    let doc_start_pos = line.find("///")? + 3;
                    let content_after = &line[doc_start_pos..];
                    let leading_ws = content_after.len() - content_after.trim_start().len();
                    let name_start = doc_start_pos + leading_ws;
                    let name_end = name_start + old_name.len();

                    let start_utf16 = byte_index_to_utf16_col(line, name_start);
                    let end_utf16 = byte_index_to_utf16_col(line, name_end);

                    text_edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: start_utf16 as u32,
                            },
                            end: Position {
                                line: idx as u32,
                                character: end_utf16 as u32,
                            },
                        },
                        new_text: new_name.to_string(),
                    });
                }
            }
        }
    }

    // Find and rename Path binding in function signature
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        if let Some(path_pos) = line.find("Path(") {
            let after_path = &line[path_pos + 5..];

            // Handle tuple: Path((a, b))
            let (bindings_str, bindings_start) = if after_path.starts_with('(') {
                let close = after_path.find(')')?;
                (&after_path[1..close], path_pos + 6)
            } else {
                let close = after_path.find(')')?;
                (&after_path[..close], path_pos + 5)
            };

            // Find and rename matching binding
            let mut current_pos = bindings_start;
            for binding in bindings_str.split(',') {
                let binding = binding.trim();
                if binding == old_name {
                    if let Some(rel_pos) = line[current_pos..].find(binding) {
                        let abs_start = current_pos + rel_pos;
                        let abs_end = abs_start + binding.len();

                        let start_utf16 = byte_index_to_utf16_col(line, abs_start);
                        let end_utf16 = byte_index_to_utf16_col(line, abs_end);

                        text_edits.push(TextEdit {
                            range: Range {
                                start: Position {
                                    line: idx as u32,
                                    character: start_utf16 as u32,
                                },
                                end: Position {
                                    line: idx as u32,
                                    character: end_utf16 as u32,
                                },
                            },
                            new_text: new_name.to_string(),
                        });
                    }
                }
                if let Some(rel_pos) = line[current_pos..].find(binding) {
                    current_pos = current_pos + rel_pos + binding.len();
                }
            }
        }
    }

    // Find and rename variable usages in function body
    // Look for the variable name as a whole word
    let word_pattern = format!(r"\b{}\b", regex::escape(&old_name));
    let re = regex::Regex::new(&word_pattern).ok()?;

    // Track positions we've already edited to avoid duplicates
    let edited_positions: std::collections::HashSet<(u32, u32)> = text_edits
        .iter()
        .map(|e| (e.range.start.line, e.range.start.character))
        .collect();

    // Find function body start (after the opening brace)
    let mut in_fn_body = false;

    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(doc_start)
        .take(fn_end - doc_start + 1)
    {
        // Track when we enter the function body (start AFTER the line with opening brace)
        if !in_fn_body {
            if line.contains('{') {
                in_fn_body = true;
            }
            continue; // Skip the signature line
        }

        // Find all matches in this line
        for mat in re.find_iter(line) {
            let start_utf16 = byte_index_to_utf16_col(line, mat.start());
            let end_utf16 = byte_index_to_utf16_col(line, mat.end());

            // Skip if we've already added an edit at this position
            let pos = (idx as u32, start_utf16 as u32);
            if edited_positions.contains(&pos) {
                continue;
            }

            text_edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: idx as u32,
                        character: start_utf16 as u32,
                    },
                    end: Position {
                        line: idx as u32,
                        character: end_utf16 as u32,
                    },
                },
                new_text: new_name.to_string(),
            });
        }
    }

    if text_edits.is_empty() {
        return None;
    }

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri, text_edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}

/// Find the boundaries of a #[rovo] block (doc start to function end)
fn find_rovo_block_boundaries(content: &str, line_idx: usize) -> Option<(usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();

    // Find doc comment start by going backwards
    // We need to handle the case where we start from the function body or signature
    let mut doc_start = line_idx;
    let mut found_doc_or_attr = false;

    for i in (0..=line_idx).rev() {
        let line = lines.get(i)?;
        let trimmed = line.trim();

        if trimmed.starts_with("///") || trimmed.starts_with("#[") {
            doc_start = i;
            found_doc_or_attr = true;
        } else if trimmed.is_empty() {
            // Empty line - continue looking
            if found_doc_or_attr {
                // We've found doc/attr lines, stop at empty line
                break;
            }
        } else {
            // Non-empty, non-doc line
            if found_doc_or_attr {
                // Found the start of the block
                break;
            }
            // Haven't found any doc/attr yet, keep looking
            // (we might be starting from inside the function body)
        }
    }

    // Find function end by going forward and tracking braces
    let mut found_fn = false;
    let mut brace_depth: i32 = 0;

    for (i, line) in lines.iter().enumerate().skip(doc_start) {
        if line.contains("fn ") && !line.trim().starts_with("//") {
            found_fn = true;
        }

        if found_fn {
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            return Some((doc_start, i));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Some((doc_start, lines.len().saturating_sub(1)))
}

/// Check if a path parameter has a corresponding doc entry in # Path Parameters
fn has_path_param_doc(content: &str, line_idx: usize, param_name: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    // Find the rovo block boundaries
    let Some((doc_start, fn_end)) = find_rovo_block_boundaries(content, line_idx) else {
        return false;
    };

    // Look for # Path Parameters section and check if param is documented
    let mut in_path_params = false;
    for line in lines.iter().skip(doc_start).take(fn_end - doc_start + 1) {
        let trimmed = line.trim_start().trim_start_matches("///").trim();

        if trimmed.starts_with("# ") {
            in_path_params = trimmed == "# Path Parameters";
            continue;
        }

        if in_path_params && !trimmed.is_empty() {
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim();
                if name == param_name {
                    return true;
                }
            }
        }
    }

    false
}

fn get_status_code_at_position(line: &str, char_idx: usize) -> Option<String> {
    // Check if we're in a doc comment
    if !line.trim_start().starts_with("///") {
        return None;
    }

    let content = line.trim_start().trim_start_matches("///").trim();

    // Check if line contains status code patterns
    // Format: "200: Type - Description" or "200: example_code"
    let has_status_context = content
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);

    if !has_status_context {
        return None;
    }

    // Find all 3-digit numbers that look like status codes (100-599)
    let mut current_pos = 0;
    for word in line.split_whitespace() {
        if let Some(word_start) = line[current_pos..].find(word) {
            let abs_start = current_pos + word_start;
            let abs_end = abs_start + word.len();

            // Check if cursor is within this word
            if char_idx >= abs_start && char_idx <= abs_end {
                // Check if it's a status code potentially followed by a colon
                let trimmed_word = word.trim_end_matches(':');
                if let Ok(code) = trimmed_word.parse::<u16>() {
                    if (100..=599).contains(&code) {
                        return Some(get_status_code_info(code));
                    }
                }
            }

            current_pos = abs_end;
        }
    }

    None
}

fn get_status_code_info(code: u16) -> String {
    // Try to get from markdown files first
    if let Some(info) = crate::docs::get_status_code_from_markdown(code) {
        return info.to_string();
    }

    // Fallback to generic messages
    match code {
        _ if (100..=199).contains(&code) => format!("**{} Informational**\n\nIndicates that the request was received and is being processed.", code),
        _ if (200..=299).contains(&code) => format!("**{} Success**\n\nIndicates that the request was successfully received, understood, and accepted.", code),
        _ if (300..=399).contains(&code) => format!("**{} Redirection**\n\nIndicates that further action needs to be taken to complete the request.", code),
        _ if (400..=499).contains(&code) => format!("**{} Client Error**\n\nIndicates that the client seems to have made an error.", code),
        _ if (500..=599).contains(&code) => format!("**{} Server Error**\n\nIndicates that the server failed to fulfill an apparently valid request.", code),
        _ => format!("**{}**\n\nUnknown status code.", code),
    }
}

fn get_security_scheme_at_position(line: &str, char_idx: usize) -> Option<String> {
    // Check if we're in a doc comment with @security
    if !line.trim_start().starts_with("///") {
        return None;
    }

    if !line.contains("@security") {
        return None;
    }

    let schemes = [
        ("bearer", "**Bearer Authentication**\n\n\"Bearer\" means **whoever holds (bears) this token gets access**.\n\nThe token is passed in the `Authorization` header:\n```\nAuthorization: Bearer <token>\n```\n\n**Token types** (bearer is the transport, not the format):\n- **Session IDs**: Random strings mapped to DB sessions\n- **JWTs**: Self-contained tokens with claims\n- **OAuth tokens**: From OAuth authorization flows\n- **Custom tokens**: Any format you choose\n\n**Key point**: Bearer = HOW you send it, not WHAT you send.\n\n⚠️ Always use HTTPS - bearer tokens are credentials!"),
        ("basic", "**Basic Authentication**\n\nSimple authentication scheme built into HTTP. Credentials are sent as:\n\n```\nAuthorization: Basic <base64(username:password)>\n```\n\n⚠️ **Security Note**: Should only be used over HTTPS as credentials are only base64 encoded, not encrypted."),
        ("apiKey", "**API Key Authentication**\n\nAuthentication using an API key that can be sent in:\n- Header: `X-API-Key: <key>`\n- Query parameter: `?api_key=<key>`\n- Cookie\n\nCommonly used for:\n- Public APIs\n- Service-to-service authentication\n- Third-party integrations"),
        ("oauth2", "**OAuth 2.0**\n\nIndustry-standard protocol for authorization. Enables applications to obtain limited access to user accounts.\n\n**Common flows:**\n- Authorization Code: For web/mobile apps\n- Client Credentials: For service-to-service\n- Implicit: For browser-based apps (deprecated)\n- Resource Owner Password: For trusted apps\n\nProvides access tokens with specific scopes and expiration."),
    ];

    // Find which scheme the cursor is on
    let mut current_pos = 0;
    for word in line.split_whitespace() {
        if let Some(word_start) = line[current_pos..].find(word) {
            let abs_start = current_pos + word_start;
            let abs_end = abs_start + word.len();

            // Check if cursor is within this word
            if char_idx >= abs_start && char_idx <= abs_end {
                // Check if it matches a known scheme
                for (scheme, info) in &schemes {
                    if word == *scheme {
                        return Some(info.to_string());
                    }
                }
            }

            current_pos = abs_end;
        }
    }

    None
}

/// Generate semantic tokens for the document
///
/// Token types (indices in legend):
/// 0: KEYWORD - for annotations (@tag, @security, @id, @hidden, @rovo-ignore)
/// 1: NUMBER - for status codes (200, 404, etc.)
/// 2: TYPE - for security schemes (bearer, oauth2, etc.)
pub fn semantic_tokens_full(content: &str) -> Option<SemanticTokensResult> {
    eprintln!(
        "[ROVO] semantic_tokens_full called, content length: {}",
        content.len()
    );
    let mut tokens = Vec::new();
    let mut prev_line: u32 = 0;
    let mut prev_start: u32 = 0;

    // Compile regexes once outside the loop for efficiency
    let annotation_regex = regex::Regex::new(r"@(tag|security|id|hidden|rovo-ignore)\b").unwrap();
    let tag_value_regex = regex::Regex::new(r"@(?:tag|id)\s+(\w+)").unwrap();
    let status_regex = regex::Regex::new(r"\b([1-5][0-9]{2})\b").unwrap();
    let security_regex = regex::Regex::new(r"\b(bearer|basic|apiKey|oauth2)\b").unwrap();
    let section_regex =
        regex::Regex::new(r"^///\s*#\s+(Path Parameters|Responses|Examples|Metadata)\b").unwrap();
    // Match path param lines: "/// param_name: description"
    let path_param_regex = regex::Regex::new(r"^///\s+(\w+):\s").unwrap();

    let mut in_path_params_section = false;

    for (line_idx, line) in content.lines().enumerate() {
        // Only process lines near #[rovo] attributes
        if !crate::parser::is_near_rovo_attribute(content, line_idx) {
            continue;
        }

        // Match section headers: # Responses, # Examples, # Metadata, # Path Parameters
        for cap in section_regex.captures_iter(line) {
            if let Some(_m) = cap.get(0) {
                // Track if we're entering/leaving Path Parameters section
                let section_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                in_path_params_section = section_name == "Path Parameters";

                // Find the position of the '#' character
                if let Some(hash_pos) = line.find('#') {
                    let start_col = byte_index_to_utf16_col(line, hash_pos) as u32;
                    // Length is from '#' to end of section name
                    let length: u32 = (2 + section_name.len()) as u32; // "# " + section_name

                    // Calculate delta encoding (UTF-16 units)
                    let delta_line = (line_idx as u32).saturating_sub(prev_line);
                    let delta_start = if delta_line == 0 {
                        start_col.saturating_sub(prev_start)
                    } else {
                        start_col
                    };

                    tokens.push(SemanticToken {
                        delta_line,
                        delta_start,
                        length,
                        token_type: 4,             // KEYWORD type for section headers
                        token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                    });

                    prev_line = line_idx as u32;
                    prev_start = start_col;
                }
            }
        }

        // Check if we're leaving a section (hit another # header or non-doc line)
        let trimmed = line.trim();
        if !trimmed.starts_with("///") {
            in_path_params_section = false;
        } else if trimmed.starts_with("/// #") && !section_regex.is_match(line) {
            in_path_params_section = false;
        }

        // Match path parameter names in # Path Parameters section
        if in_path_params_section {
            for cap in path_param_regex.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    let start_byte = m.start();
                    let start_col = byte_index_to_utf16_col(line, start_byte) as u32;
                    let length: u32 = m.as_str().chars().map(|ch| ch.len_utf16() as u32).sum();

                    let delta_line = (line_idx as u32).saturating_sub(prev_line);
                    let delta_start = if delta_line == 0 {
                        start_col.saturating_sub(prev_start)
                    } else {
                        start_col
                    };

                    tokens.push(SemanticToken {
                        delta_line,
                        delta_start,
                        length,
                        token_type: 5, // PARAMETER type for path param names
                        token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                    });

                    prev_line = line_idx as u32;
                    prev_start = start_col;
                }
            }
        }

        // Match annotations: @tag, @security, @id, @hidden, @rovo-ignore
        for cap in annotation_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_byte = m.start();
                let start_col = byte_index_to_utf16_col(line, start_byte) as u32;
                let length: u32 = m.as_str().chars().map(|ch| ch.len_utf16() as u32).sum();

                // Calculate delta encoding (UTF-16 units)
                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    start_col.saturating_sub(prev_start)
                } else {
                    start_col
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: 0,             // MACRO
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_col;
            }
        }

        // Match tag/id values: text after @tag, @id, etc.
        for cap in tag_value_regex.captures_iter(line) {
            if let Some(m) = cap.get(1) {
                let start_byte = m.start();
                let start_col = byte_index_to_utf16_col(line, start_byte) as u32;
                let length: u32 = m.as_str().chars().map(|ch| ch.len_utf16() as u32).sum();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    start_col.saturating_sub(prev_start)
                } else {
                    start_col
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: 3,             // STRING
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_col;
            }
        }

        // Match status codes: 200, 404, etc.
        for cap in status_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_byte = m.start();
                let start_col = byte_index_to_utf16_col(line, start_byte) as u32;
                let length: u32 = m.as_str().chars().map(|ch| ch.len_utf16() as u32).sum();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    start_col.saturating_sub(prev_start)
                } else {
                    start_col
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: 1,             // NUMBER
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_col;
            }
        }

        // Match security schemes: bearer, basic, apiKey, oauth2
        for cap in security_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_byte = m.start();
                let start_col = byte_index_to_utf16_col(line, start_byte) as u32;
                let length: u32 = m.as_str().chars().map(|ch| ch.len_utf16() as u32).sum();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    start_col.saturating_sub(prev_start)
                } else {
                    start_col
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: 2,             // TYPE
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_col;
            }
        }
    }

    eprintln!("[ROVO] Found {} semantic tokens", tokens.len());

    if tokens.is_empty() {
        None
    } else {
        Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        }))
    }
}
