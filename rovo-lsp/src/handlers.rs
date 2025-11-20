use crate::completion;
use crate::diagnostics;
use crate::utils::utf16_pos_to_byte_index;
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

    diagnostics_list
        .into_iter()
        .map(|diag| {
            let severity = match diag.severity {
                diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            };

            Diagnostic {
                range: Range {
                    start: Position {
                        line: diag.line as u32,
                        character: diag.char_start.unwrap_or(0) as u32,
                    },
                    end: Position {
                        line: diag.line as u32,
                        character: diag.char_end.unwrap_or(1000) as u32,
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
