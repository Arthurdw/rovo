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

    // Find the annotation keyword at the cursor position
    let annotations = [
        "@response",
        "@tag",
        "@security",
        "@example",
        "@id",
        "@hidden",
    ];

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

            Diagnostic {
                range: Range {
                    start: Position {
                        line: diag.line as u32,
                        character: char_start as u32,
                    },
                    end: Position {
                        line: diag.line as u32,
                        character: char_end as u32,
                    },
                },
                severity: Some(severity),
                source: Some("rovo-lsp".to_string()),
                message: diag.message,
                ..Default::default()
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
                let start_char = pos;
                let end_char = pos + 4 + whitespace + tag_name.len();

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

    // Extract tag name from current position
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;
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

    // Extract tag name from current position
    let char_idx = utf16_pos_to_byte_index(line, position.character as usize)?;
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

fn get_status_code_at_position(line: &str, char_idx: usize) -> Option<String> {
    // Check if we're in a doc comment with @response or @example
    if !line.trim_start().starts_with("///") {
        return None;
    }

    if !line.contains("@response") && !line.contains("@example") {
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
                // Check if it's a valid status code
                if let Ok(code) = word.parse::<u16>() {
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
    match code {
        200 => "**200 OK**\n\nRequest succeeded. The meaning depends on the HTTP method:\n- GET: Resource fetched\n- POST: Resource created/action performed\n- PUT: Resource updated\n- DELETE: Resource deleted".to_string(),
        201 => "**201 Created**\n\nRequest succeeded and a new resource was created. Typically returned after POST or PUT requests.".to_string(),
        204 => "**204 No Content**\n\nRequest succeeded but there's no content to return. Often used for DELETE operations.".to_string(),
        400 => "**400 Bad Request**\n\nServer cannot process the request due to client error (e.g., malformed syntax, invalid request message framing, or deceptive request routing).".to_string(),
        401 => "**401 Unauthorized**\n\nClient must authenticate itself to get the requested response. The client is not authenticated.".to_string(),
        403 => "**403 Forbidden**\n\nClient does not have access rights to the content. Unlike 401, the client's identity is known to the server but they don't have permission.".to_string(),
        404 => "**404 Not Found**\n\nServer cannot find the requested resource. This is one of the most famous status codes.".to_string(),
        409 => "**409 Conflict**\n\nRequest conflicts with the current state of the server. Often used for concurrent modification conflicts.".to_string(),
        422 => "**422 Unprocessable Entity**\n\nRequest was well-formed but contains semantic errors. Often used for validation failures.".to_string(),
        500 => "**500 Internal Server Error**\n\nServer encountered an unexpected condition that prevented it from fulfilling the request.".to_string(),
        503 => "**503 Service Unavailable**\n\nServer is not ready to handle the request. Common causes are server maintenance or overload.".to_string(),
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
/// 0: KEYWORD - for annotations (@response, @tag, etc.)
/// 1: NUMBER - for status codes (200, 404, etc.)
/// 2: TYPE - for security schemes (bearer, oauth2, etc.)
pub fn semantic_tokens_full(content: &str) -> Option<SemanticTokensResult> {
    eprintln!("[ROVO] semantic_tokens_full called, content length: {}", content.len());
    let mut tokens = Vec::new();
    let mut prev_line = 0;
    let mut prev_start = 0;

    for (line_idx, line) in content.lines().enumerate() {
        // Only process lines near #[rovo] attributes
        if !crate::parser::is_near_rovo_attribute(content, line_idx) {
            continue;
        }

        // Match annotations: @response, @tag, @security, @example, @id, @hidden, @rovo-ignore
        let annotation_regex = regex::Regex::new(r"@(response|tag|security|example|id|hidden|rovo-ignore)\b").unwrap();
        for cap in annotation_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_char = m.start();
                let length = m.as_str().len();

                // Calculate delta encoding
                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    (start_char as u32).saturating_sub(prev_start)
                } else {
                    start_char as u32
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: length as u32,
                    token_type: 0, // MACRO
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_char as u32;
            }
        }

        // Match tag/id values: text after @tag, @id, etc.
        let tag_value_regex = regex::Regex::new(r"@(?:tag|id)\s+(\w+)").unwrap();
        for cap in tag_value_regex.captures_iter(line) {
            if let Some(m) = cap.get(1) {
                let start_char = m.start();
                let length = m.as_str().len();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    (start_char as u32).saturating_sub(prev_start)
                } else {
                    start_char as u32
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: length as u32,
                    token_type: 3, // STRING
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_char as u32;
            }
        }

        // Match status codes: 200, 404, etc.
        let status_regex = regex::Regex::new(r"\b([1-5][0-9]{2})\b").unwrap();
        for cap in status_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_char = m.start();
                let length = m.as_str().len();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    (start_char as u32).saturating_sub(prev_start)
                } else {
                    start_char as u32
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: length as u32,
                    token_type: 1, // NUMBER
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_char as u32;
            }
        }

        // Match security schemes: bearer, basic, apiKey, oauth2
        let security_regex = regex::Regex::new(r"\b(bearer|basic|apiKey|oauth2)\b").unwrap();
        for cap in security_regex.captures_iter(line) {
            if let Some(m) = cap.get(0) {
                let start_char = m.start();
                let length = m.as_str().len();

                let delta_line = (line_idx as u32).saturating_sub(prev_line);
                let delta_start = if delta_line == 0 {
                    (start_char as u32).saturating_sub(prev_start)
                } else {
                    start_char as u32
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: length as u32,
                    token_type: 2, // TYPE
                    token_modifiers_bitset: 1, // DOCUMENTATION modifier (bit 0)
                });

                prev_line = line_idx as u32;
                prev_start = start_char as u32;
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
