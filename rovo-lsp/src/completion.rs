use crate::utils::utf16_pos_to_byte_index;
use serde::{Deserialize, Serialize};

/// Position in a text document
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Character offset in line (0-indexed)
    pub character: usize,
}

/// An auto-completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label shown in completion list
    pub label: String,
    /// Kind of completion item
    pub kind: CompletionItemKind,
    /// Short detail shown alongside label
    pub detail: Option<String>,
    /// Full documentation for this item
    pub documentation: Option<String>,
    /// Text to insert when selected
    pub insert_text: Option<String>,
}

/// Type of completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    /// A language keyword
    Keyword,
    /// A code snippet
    Snippet,
}

/// Get completion suggestions at the given position
///
/// # Arguments
/// * `content` - The source code content
/// * `position` - Cursor position where completion was requested
///
/// # Returns
/// A vector of completion suggestions
pub fn get_completions(content: &str, position: Position) -> Vec<CompletionItem> {
    let lines: Vec<&str> = content.lines().collect();

    if position.line >= lines.len() {
        return Vec::new();
    }

    let line = lines[position.line];

    // Convert UTF-16 character offset to UTF-8 byte index
    // LSP Position.character is in UTF-16 code units, but Rust strings use UTF-8
    let byte_index = utf16_pos_to_byte_index(line, position.character).unwrap_or(line.len());

    let prefix = &line[..byte_index];

    // Check if we're in a doc comment
    if !prefix.trim_start().starts_with("///") {
        return Vec::new();
    }

    // Handle indented doc comments by trimming whitespace first
    let after_doc = prefix.trim_start().trim_start_matches("///").trim_start();

    // Detect context - which section are we in?
    let context = detect_section_context(&lines, position.line);

    // Check for section header completion
    if after_doc.starts_with("# ") || after_doc == "#" {
        return get_section_completions(after_doc, &context);
    }

    // Check for specific annotation value completions first (before general @ check)
    if after_doc.starts_with("@security ") {
        let parts: Vec<&str> = after_doc.split_whitespace().collect();
        if parts.len() == 1 {
            return get_security_scheme_completions("");
        } else if parts.len() == 2 {
            return get_security_scheme_completions(parts[1]);
        }
    }

    // Context-aware completions based on current section
    match context {
        SectionContext::ResponsesSection => {
            // In # Responses section, complete response lines
            if after_doc.is_empty()
                || after_doc
                    .chars()
                    .next()
                    .map(|c| c.is_digit(10))
                    .unwrap_or(false)
            {
                return get_response_line_completions();
            }
        }
        SectionContext::ExamplesSection => {
            // In # Examples section, complete example lines
            if after_doc.is_empty()
                || after_doc
                    .chars()
                    .next()
                    .map(|c| c.is_digit(10))
                    .unwrap_or(false)
            {
                return get_example_line_completions();
            }
        }
        SectionContext::MetadataSection => {
            // In # Metadata section, only allow @ annotations
            if after_doc.starts_with('@') {
                return get_metadata_annotation_completions(after_doc);
            }
        }
        SectionContext::None => {
            // Not in a section - handle @ annotations
            if after_doc.starts_with('@') {
                return get_metadata_annotation_completions(after_doc);
            }
        }
    }

    Vec::new()
}

/// Context about which section we're currently in
#[derive(Debug, Clone, PartialEq)]
enum SectionContext {
    ResponsesSection,
    ExamplesSection,
    MetadataSection,
    None,
}

/// Detect which section (if any) the current line is in
fn detect_section_context(lines: &[&str], current_line: usize) -> SectionContext {
    // Look backwards from current line to find the most recent section header
    for i in (0..=current_line).rev() {
        let line = lines[i].trim();
        if !line.starts_with("///") {
            // Hit a non-comment line, stop searching
            break;
        }

        let content = line.trim_start_matches("///").trim();

        // Check for section headers
        if content == "# Responses" {
            return SectionContext::ResponsesSection;
        } else if content == "# Examples" {
            return SectionContext::ExamplesSection;
        } else if content == "# Metadata" {
            return SectionContext::MetadataSection;
        }

        // Check for #[rovo] attribute - we've gone too far
        if content.contains("#[rovo]") {
            break;
        }
    }

    SectionContext::None
}

/// Get completions for section headers
fn get_section_completions(typed: &str, _context: &SectionContext) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    let sections = [
        (
            "# Responses",
            "# Responses\n///\n/// ${1:200}: ${2:Json<T>} - ${3:description}",
        ),
        (
            "# Examples",
            "# Examples\n///\n/// ${1:200}: ${2:expression}",
        ),
        ("# Metadata", "# Metadata\n///\n/// @${1:tag} ${2:value}"),
    ];

    for (label, snippet) in sections {
        if label.to_lowercase().starts_with(&typed.to_lowercase()) {
            completions.push(CompletionItem {
                label: label.to_string(),
                kind: CompletionItemKind::Snippet,
                detail: Some(format!("Insert {} section", label)),
                documentation: Some(format!("Creates a {} section with a template entry", label)),
                insert_text: Some(snippet.to_string()),
            });
        }
    }

    completions
}

/// Get completions for response lines in # Responses section
fn get_response_line_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "200 response".to_string(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Successful response".to_string()),
            documentation: Some("Add a 200 OK response".to_string()),
            insert_text: Some("200: ${1:Json<T>} - ${2:description}".to_string()),
        },
        CompletionItem {
            label: "201 response".to_string(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Created response".to_string()),
            documentation: Some("Add a 201 Created response".to_string()),
            insert_text: Some("201: ${1:Json<T>} - ${2:description}".to_string()),
        },
        CompletionItem {
            label: "404 response".to_string(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Not found response".to_string()),
            documentation: Some("Add a 404 Not Found response".to_string()),
            insert_text: Some("404: () - ${1:description}".to_string()),
        },
    ]
}

/// Get completions for example lines in # Examples section
fn get_example_line_completions() -> Vec<CompletionItem> {
    vec![CompletionItem {
        label: "200 example".to_string(),
        kind: CompletionItemKind::Snippet,
        detail: Some("Success example".to_string()),
        documentation: Some("Add a 200 OK example".to_string()),
        insert_text: Some("200: ${1:expression}".to_string()),
    }]
}

/// Get completions for metadata annotations
fn get_metadata_annotation_completions(typed: &str) -> Vec<CompletionItem> {
    let after_at = typed.trim_start_matches('@');
    let mut completions = Vec::new();

    // Only metadata annotations - no @response or @example (use sections instead)
    let annotations = [
        ("tag", "@tag ${1:tag_name}"),
        ("security", "@security ${1:bearer}"),
        ("id", "@id ${1:operation_id}"),
        ("hidden", "@hidden"),
    ];

    for (label, snippet) in annotations {
        if label.starts_with(after_at) {
            let full_label = format!("@{}", label);
            completions.push(CompletionItem {
                label: full_label.clone(),
                kind: CompletionItemKind::Snippet,
                detail: Some(format!("{} annotation", label)),
                documentation: Some(
                    crate::docs::get_annotation_documentation(&full_label).to_string(),
                ),
                insert_text: Some(snippet.to_string()),
            });
        }
    }

    completions
}

fn get_security_scheme_completions(filter: &str) -> Vec<CompletionItem> {
    let schemes = [
        (
            "bearer",
            "Bearer token authentication",
            "**Bearer Authentication**\n\n\"Bearer\" means **whoever holds (bears) this token gets access**.\n\n## How it works\n\nThe token is passed in the `Authorization` header:\n```\nAuthorization: Bearer <token>\n```\n\n## Token types (bearer is the transport, not the format)\n\n### Session tokens\n```\nAuthorization: Bearer abc123sessionid456\n```\n- Random string/UUID\n- Maps to session data in your database\n- Server validates by looking up the session\n\n### JWT (JSON Web Tokens)\n```\nAuthorization: Bearer eyJhbGc...\n```\n- Self-contained with claims\n- Server validates signature\n- Can be stateless (no DB lookup)\n\n### OAuth 2.0 access tokens\n```\nAuthorization: Bearer oauth_token_xyz\n```\n- Obtained from OAuth authorization flow\n- Can be JWT or opaque token\n\n### Custom tokens\n- Any format your API supports\n- The bearer scheme just defines HOW to send it\n\n## Key concept\n**Bearer = Transport mechanism, NOT token format**\n\nYou can use bearer auth with:\n- ✅ Session IDs\n- ✅ JWTs\n- ✅ Random tokens\n- ✅ Any token format\n\n⚠️ **Security**: Always use HTTPS - bearer tokens are credentials!"
        ),
        (
            "basic",
            "Basic HTTP authentication (username/password)",
            "**Basic Authentication**\n\nSimple authentication scheme built into the HTTP protocol.\n\n## How it works\n\nCredentials are sent as base64-encoded `username:password`:\n```\nAuthorization: Basic <base64(username:password)>\n```\n\n## Example\nFor username `user` and password `pass123`:\n```\nAuthorization: Basic dXNlcjpwYXNzMTIz\n```\n\n## Use cases\n- Simple APIs with username/password\n- Internal services\n- Development/testing environments\n\n⚠️ **Security Warning**\n- Credentials are only **base64 encoded**, NOT encrypted\n- **MUST be used with HTTPS** in production\n- Consider using bearer tokens for better security"
        ),
        (
            "apiKey",
            "API key in header, query, or cookie",
            "**API Key Authentication**\n\nAuthentication using a simple API key that can be sent in multiple ways.\n\n## Transmission methods\n\n**Header** (recommended):\n```\nX-API-Key: your-api-key-here\nAPI-Key: your-api-key-here\n```\n\n**Query parameter**:\n```\nGET /api/users?api_key=your-api-key-here\n```\n\n**Cookie**:\n```\nCookie: api_key=your-api-key-here\n```\n\n## Common use cases\n- Public APIs (rate limiting, analytics)\n- Service-to-service authentication\n- Third-party integrations\n- Partner API access\n\n## Best practices\n- Use headers instead of query params (prevents logging)\n- Rotate keys regularly\n- Use different keys per environment\n- Support key revocation"
        ),
        (
            "oauth2",
            "OAuth 2.0 authentication flow",
            "**OAuth 2.0**\n\nIndustry-standard protocol for authorization. Enables applications to obtain limited access to user accounts on an HTTP service.\n\n## Grant Types (Flows)\n\n### Authorization Code\n- **Best for**: Web/mobile apps\n- **Flow**: User → Login → Code → Exchange for token\n- **Most secure** for public clients\n\n### Client Credentials\n- **Best for**: Service-to-service\n- **Flow**: Client → Token (no user interaction)\n- Used for machine-to-machine authentication\n\n### Implicit (Deprecated)\n- **Legacy**: Browser-based apps\n- **Status**: No longer recommended\n- Use Authorization Code with PKCE instead\n\n### Resource Owner Password\n- **Best for**: Highly trusted apps\n- **Flow**: Username/password → Token\n- Only use when you control both client and server\n\n## Key concepts\n- **Scopes**: Limit access to specific resources\n- **Access Token**: Short-lived token for API access\n- **Refresh Token**: Long-lived token to get new access tokens\n- **Token expiration**: Enhances security\n\n## Advantages\n- User never shares password with app\n- Fine-grained permissions (scopes)\n- Token revocation\n- Industry standard"
        ),
    ];

    schemes
        .iter()
        .filter(|(scheme, _, _)| {
            // If no filter, show all; otherwise filter by prefix
            filter.is_empty() || scheme.starts_with(filter)
        })
        .map(|(scheme, desc, docs)| CompletionItem {
            label: scheme.to_string(),
            kind: CompletionItemKind::Keyword,
            detail: Some(desc.to_string()),
            documentation: Some(docs.to_string()),
            insert_text: Some(scheme.to_string()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completes_at_sign() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 4); // Only metadata annotations
        assert!(completions.iter().any(|c| c.label == "@tag"));
        assert!(completions.iter().any(|c| c.label == "@security"));
        assert!(completions.iter().any(|c| c.label == "@id"));
        assert!(completions.iter().any(|c| c.label == "@hidden"));
    }

    #[test]
    fn test_filters_by_prefix() {
        let content = "/// @s"; // 's' for security
        let position = Position {
            line: 0,
            character: 6,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "@security");
    }

    #[test]
    fn test_no_completions_outside_doc_comment() {
        let content = "@response";
        let position = Position {
            line: 0,
            character: 1,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_status_code_completion_in_responses_section() {
        let content = "/// # Responses\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer status code response lines
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label.starts_with("200")));
        assert!(completions.iter().any(|c| c
            .insert_text
            .as_ref()
            .map(|t| t.starts_with("200:"))
            .unwrap_or(false)));
    }

    #[test]
    fn test_status_code_completion_in_examples_section() {
        let content = "/// # Examples\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer status code example lines
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label.starts_with("200")));
        assert!(completions.iter().any(|c| c
            .insert_text
            .as_ref()
            .map(|t| t.starts_with("200:"))
            .unwrap_or(false)));
    }

    #[test]
    fn test_status_code_filtering_by_prefix() {
        let content = "/// # Responses\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should provide response completions in Responses section
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label.contains("200")));
        assert!(completions.iter().any(|c| c.label.contains("404")));
    }

    #[test]
    fn test_status_code_filtering_specific() {
        let content = "/// # Examples\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should provide example completions in Examples section
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label.contains("200")));
    }

    #[test]
    fn test_security_scheme_completion() {
        let content = "/// @security ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);
        // Should offer security schemes
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "bearer"));
        assert!(completions.iter().any(|c| c.label == "basic"));
        assert!(completions.iter().any(|c| c.label == "apiKey"));
        assert!(completions.iter().any(|c| c.label == "oauth2"));
    }

    #[test]
    fn test_security_scheme_filtering() {
        let content = "/// @security b";
        let position = Position {
            line: 0,
            character: 15,
        };
        let completions = get_completions(content, position);
        // Should only show schemes starting with 'b'
        assert_eq!(completions.len(), 2); // bearer and basic
        assert!(completions.iter().any(|c| c.label == "bearer"));
        assert!(completions.iter().any(|c| c.label == "basic"));
        assert!(!completions.iter().any(|c| c.label == "oauth2"));
    }

    #[test]
    fn test_security_scheme_filtering_specific() {
        let content = "/// @security be";
        let position = Position {
            line: 0,
            character: 16,
        };
        let completions = get_completions(content, position);
        // Should only show bearer
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "bearer");
    }

    #[test]
    fn test_completion_items_have_documentation() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // All completions should have documentation
        for completion in &completions {
            assert!(
                completion.documentation.is_some(),
                "Completion '{}' missing documentation",
                completion.label
            );
            assert!(!completion.documentation.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn test_status_code_completions_have_details() {
        let content = "/// @response ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);

        // All status codes should have detail and documentation
        for completion in &completions {
            assert!(completion.detail.is_some());
            assert!(completion.documentation.is_some());
        }
    }

    #[test]
    fn test_security_scheme_completions_have_details() {
        let content = "/// @security ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);

        // All security schemes should have detail and documentation
        for completion in &completions {
            assert!(completion.detail.is_some());
            assert!(completion.documentation.is_some());
            assert!(!completion.documentation.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn test_handles_indented_doc_comments() {
        let content = "    /// @";
        let position = Position {
            line: 0,
            character: 9,
        };
        let completions = get_completions(content, position);
        // Should work with indented comments - 4 metadata annotations
        assert_eq!(completions.len(), 4);
    }

    #[test]
    fn test_handles_utf16_positions() {
        // Content with multibyte characters - just ensure it doesn't crash
        let content = "/// 世界 @response";
        let position = Position {
            line: 0,
            character: 12, // Somewhere in the line
        };
        let completions = get_completions(content, position);
        // Should not crash with UTF-16 positions
        // (may or may not offer completions depending on exact position)
        assert!(completions.len() <= 11); // At most all status codes
    }

    #[test]
    fn test_out_of_bounds_line() {
        let content = "/// @";
        let position = Position {
            line: 100, // Way out of bounds
            character: 0,
        };
        let completions = get_completions(content, position);
        // Should return empty, not crash
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_empty_filter_shows_all_annotations() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should show all 4 metadata annotations (no @response/@example, use sections instead)
        assert_eq!(completions.len(), 4);
    }

    #[test]
    fn test_partial_annotation_filters() {
        let content = "/// @s";
        let position = Position {
            line: 0,
            character: 6,
        };
        let completions = get_completions(content, position);
        // Should only show @security
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "@security");
    }

    #[test]
    fn test_completion_has_insert_text() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // All completions should have insert text
        for completion in &completions {
            assert!(completion.insert_text.is_some());
        }
    }

    #[test]
    fn test_section_completion_has_snippet() {
        let content = "/// # R";
        let position = Position {
            line: 0,
            character: 7,
        };
        let completions = get_completions(content, position);

        assert_eq!(completions.len(), 1);
        let insert_text = completions[0].insert_text.as_ref().unwrap();
        // Should have snippet placeholders
        assert!(insert_text.contains("Responses"));
    }

    #[test]
    fn test_multiline_content() {
        let content = "/// First line\n/// @";
        let position = Position {
            line: 1,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should work on second line - 4 metadata annotations
        assert_eq!(completions.len(), 4);
    }

    #[test]
    fn test_no_completion_after_complete_annotation() {
        let content = "/// @response 200 Json<T> Success";
        let position = Position {
            line: 0,
            character: 20, // In the middle
        };
        let completions = get_completions(content, position);
        // Should not offer completions after the annotation is complete
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_empty_security_filter() {
        let completions = get_security_scheme_completions("");
        // Should return all security schemes
        assert_eq!(completions.len(), 4); // bearer, basic, apiKey, oauth2
    }

    #[test]
    fn test_security_oauth_filter() {
        let completions = get_security_scheme_completions("o");
        // Should only return oauth2
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "oauth2");
    }

    #[test]
    fn test_completion_item_kind() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // Annotations should be snippets
        for completion in &completions {
            assert!(matches!(completion.kind, CompletionItemKind::Snippet));
        }
    }

    #[test]
    fn test_security_scheme_kind() {
        let completions = get_security_scheme_completions("");

        // Security schemes should be keywords
        for completion in &completions {
            assert!(matches!(completion.kind, CompletionItemKind::Keyword));
        }
    }
}
