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
        SectionContext::PathParametersSection => {
            // In # Path Parameters section, complete parameter lines
            if after_doc.is_empty() || after_doc.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return get_path_parameter_line_completions(
                    content,
                    &lines,
                    position.line,
                    after_doc,
                );
            }
        }
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
    PathParametersSection,
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
        } else if content == "# Path Parameters" {
            return SectionContext::PathParametersSection;
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
            "# Path Parameters",
            "# Path Parameters\n///\n/// ${1:param_name}: ${2:description}",
        ),
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

/// Get completions for path parameter lines in # Path Parameters section
fn get_path_parameter_line_completions(
    content: &str,
    lines: &[&str],
    current_line: usize,
    filter: &str,
) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Find the Path(...) bindings from the function signature
    let bindings = extract_path_bindings_from_context(content, lines, current_line);

    // Find which params are already documented
    let documented = get_documented_path_params(lines, current_line);

    // Add completions for each undocumented binding
    for binding in bindings {
        if documented.contains(&binding) {
            continue; // Skip already documented params
        }

        if !filter.is_empty() && !binding.starts_with(filter) {
            continue; // Skip if doesn't match filter
        }

        completions.push(CompletionItem {
            label: binding.clone(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Path parameter from function signature".to_string()),
            documentation: Some(format!("Document the '{}' path parameter", binding)),
            insert_text: Some(format!("{}: ${{1:Description of {}}}", binding, binding)),
        });
    }

    // Add fallback completions if no bindings found or all documented
    if completions.is_empty() && filter.is_empty() {
        completions.push(CompletionItem {
            label: "parameter".to_string(),
            kind: CompletionItemKind::Snippet,
            detail: Some("Generic path parameter".to_string()),
            documentation: Some(
                "Add a path parameter with a custom name and description".to_string(),
            ),
            insert_text: Some("${1:param_name}: ${2:description}".to_string()),
        });
    }

    completions
}

/// Extract path binding names from the function signature near the current line
fn extract_path_bindings_from_context(
    _content: &str,
    lines: &[&str],
    current_line: usize,
) -> Vec<String> {
    let mut bindings = Vec::new();

    // Look forward from current line to find function signature with Path(...)
    for line in lines.iter().skip(current_line) {
        // Stop if we hit a non-doc, non-attr, non-fn line after seeing fn
        let trimmed = line.trim();

        if trimmed.starts_with("///") || trimmed.starts_with("#[") {
            continue;
        }

        // Look for Path( in the line
        if let Some(path_pos) = line.find("Path(") {
            let after_path = &line[path_pos + 5..];

            // Handle tuple: Path((a, b))
            let bindings_str = if after_path.starts_with('(') {
                let close = after_path.find(')').unwrap_or(after_path.len());
                &after_path[1..close]
            } else {
                // Single binding: Path(name)
                let close = after_path.find(')').unwrap_or(after_path.len());
                &after_path[..close]
            };

            // Parse the bindings
            for binding in bindings_str.split(',') {
                let binding = binding.trim();
                if !binding.is_empty() && binding.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    bindings.push(binding.to_string());
                }
            }

            break;
        }

        // Stop if we've gone past the function
        if trimmed.contains('{') {
            break;
        }
    }

    bindings
}

/// Get the list of already documented path parameters
fn get_documented_path_params(lines: &[&str], current_line: usize) -> Vec<String> {
    let mut documented = Vec::new();
    let mut in_path_params = false;

    // Look backwards and forwards from current line within the doc block
    for i in (0..=current_line).rev() {
        let trimmed = lines[i].trim();
        if !trimmed.starts_with("///") {
            break;
        }

        let content = trimmed.trim_start_matches("///").trim();
        if content == "# Path Parameters" {
            in_path_params = true;
            break;
        } else if content.starts_with("# ") {
            break;
        }
    }

    if !in_path_params {
        return documented;
    }

    // Now scan from section header to current line to find documented params
    for line in lines.iter().take(current_line + 1) {
        let trimmed = line.trim();
        if !trimmed.starts_with("///") {
            continue;
        }

        let content = trimmed.trim_start_matches("///").trim();
        if content.starts_with("# ") {
            continue;
        }

        // Parse "name: description" format
        if let Some(colon_pos) = content.find(':') {
            let name = content[..colon_pos].trim();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                documented.push(name.to_string());
            }
        }
    }

    documented
}

/// Get completions for metadata annotations
fn get_metadata_annotation_completions(typed: &str) -> Vec<CompletionItem> {
    let after_at = typed.trim_start_matches('@');
    let mut completions = Vec::new();

    // Metadata annotations
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
        let content = "@tag";
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
        let content = "/// # Responses\n/// ";
        let position = Position {
            line: 1,
            character: 4,
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
        let content = "/// # Metadata\n/// 世界 @tag";
        let position = Position {
            line: 1,
            character: 12, // Somewhere in the line
        };
        let completions = get_completions(content, position);
        // Should not crash with UTF-16 positions
        // (may or may not offer completions depending on exact position)
        assert!(completions.len() <= 4); // At most all metadata annotations
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
        // Should show all 4 metadata annotations
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
        let content = "/// # Metadata\n/// @tag users";
        let position = Position {
            line: 1,
            character: 10, // In the middle of "users"
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

    #[test]
    fn test_path_parameters_section_completion() {
        let content = "/// # Path Parameters\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer fallback parameter completion when no Path() binding found
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "parameter"));
    }

    #[test]
    fn test_path_parameters_completion_from_signature() {
        let content =
            "/// # Path Parameters\n/// \n#[rovo]\nasync fn get_user(Path(user_id): Path<u64>) {}";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer completion for user_id from the function signature
        assert!(!completions.is_empty());
        assert!(
            completions.iter().any(|c| c.label == "user_id"),
            "Should find user_id from signature, got: {:?}",
            completions.iter().map(|c| &c.label).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_path_parameters_completion_skips_documented() {
        let content =
            "/// # Path Parameters\n/// user_id: Already documented\n/// \n#[rovo]\nasync fn get_user(Path(user_id): Path<u64>) {}";
        let position = Position {
            line: 2,
            character: 4,
        };
        let completions = get_completions(content, position);
        // user_id is already documented, so should not appear in completions
        assert!(
            !completions.iter().any(|c| c.label == "user_id"),
            "Should not suggest already documented param"
        );
    }

    #[test]
    fn test_path_parameters_completion_tuple() {
        let content =
            "/// # Path Parameters\n/// \n#[rovo]\nasync fn get_item(Path((collection_id, index)): Path<(String, u32)>) {}";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer completions for both tuple params
        assert!(
            completions.iter().any(|c| c.label == "collection_id"),
            "Should find collection_id"
        );
        assert!(
            completions.iter().any(|c| c.label == "index"),
            "Should find index"
        );
    }

    #[test]
    fn test_path_parameters_section_header_completion() {
        let content = "/// # P";
        let position = Position {
            line: 0,
            character: 7,
        };
        let completions = get_completions(content, position);
        // Should offer "# Path Parameters" as a section completion
        assert!(completions.iter().any(|c| c.label == "# Path Parameters"));
    }

    #[test]
    fn test_path_parameters_completions_have_snippets() {
        let content = "/// # Path Parameters\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);

        // All path parameter completions should have insert text with snippets
        for completion in &completions {
            assert!(completion.insert_text.is_some());
            let insert_text = completion.insert_text.as_ref().unwrap();
            // Should have the format "name: description"
            assert!(insert_text.contains(':'));
        }
    }

    #[test]
    fn test_detect_path_parameters_section_context() {
        let lines = vec![
            "/// Get user by ID.",
            "///",
            "/// # Path Parameters",
            "///",
            "/// ",
        ];
        let context = detect_section_context(&lines, 4);
        assert_eq!(context, SectionContext::PathParametersSection);
    }

    // Additional coverage tests

    #[test]
    fn test_detect_responses_section_context() {
        let lines = vec!["/// # Responses", "///", "/// "];
        let context = detect_section_context(&lines, 2);
        assert_eq!(context, SectionContext::ResponsesSection);
    }

    #[test]
    fn test_detect_examples_section_context() {
        let lines = vec!["/// # Examples", "///", "/// "];
        let context = detect_section_context(&lines, 2);
        assert_eq!(context, SectionContext::ExamplesSection);
    }

    #[test]
    fn test_detect_metadata_section_context() {
        let lines = vec!["/// # Metadata", "///", "/// "];
        let context = detect_section_context(&lines, 2);
        assert_eq!(context, SectionContext::MetadataSection);
    }

    #[test]
    fn test_detect_no_section_context() {
        let lines = vec!["/// Just a description", "///", "/// More text"];
        let context = detect_section_context(&lines, 2);
        assert_eq!(context, SectionContext::None);
    }

    #[test]
    fn test_responses_section_completions() {
        let content = "/// # Responses\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer response completions
        assert!(
            completions.iter().any(|c| c.label.contains("200")),
            "Should have 200 response completion"
        );
    }

    #[test]
    fn test_examples_section_completions() {
        let content = "/// # Examples\n/// ";
        let position = Position {
            line: 1,
            character: 4,
        };
        let completions = get_completions(content, position);
        // Should offer example completions
        assert!(
            completions.iter().any(|c| c.label.contains("example")),
            "Should have example completion"
        );
    }

    #[test]
    fn test_metadata_section_completions() {
        let content = "/// # Metadata\n/// @";
        let position = Position {
            line: 1,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should offer metadata annotations
        assert!(completions.iter().any(|c| c.label == "@tag"));
        assert!(completions.iter().any(|c| c.label == "@security"));
    }

    #[test]
    fn test_security_scheme_completions_no_filter() {
        let completions = get_security_scheme_completions("");
        // Should return all 4 schemes
        assert_eq!(completions.len(), 4);
        assert!(completions.iter().any(|c| c.label == "bearer"));
        assert!(completions.iter().any(|c| c.label == "basic"));
        assert!(completions.iter().any(|c| c.label == "apiKey"));
        assert!(completions.iter().any(|c| c.label == "oauth2"));
    }

    #[test]
    fn test_security_scheme_completions_no_match() {
        let completions = get_security_scheme_completions("xyz");
        // No schemes match "xyz"
        assert!(completions.is_empty());
    }

    #[test]
    fn test_section_completions_all_sections() {
        let completions = get_section_completions("# ", &SectionContext::None);
        assert_eq!(completions.len(), 4);
        assert!(completions.iter().any(|c| c.label == "# Responses"));
        assert!(completions.iter().any(|c| c.label == "# Examples"));
        assert!(completions.iter().any(|c| c.label == "# Metadata"));
        assert!(completions.iter().any(|c| c.label == "# Path Parameters"));
    }

    #[test]
    fn test_section_completions_filter() {
        let completions = get_section_completions("# R", &SectionContext::None);
        // Should only match "# Responses"
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "# Responses");
    }

    #[test]
    fn test_response_line_completions() {
        let completions = get_response_line_completions();
        assert!(!completions.is_empty());
        // Should have 200, 201, 404 responses
        assert!(completions.iter().any(|c| c.label.contains("200")));
        assert!(completions.iter().any(|c| c.label.contains("201")));
        assert!(completions.iter().any(|c| c.label.contains("404")));
    }

    #[test]
    fn test_example_line_completions() {
        let completions = get_example_line_completions();
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label.contains("200")));
    }

    #[test]
    fn test_path_params_with_filter() {
        let content =
            "/// # Path Parameters\n/// u\n#[rovo]\nasync fn get_user(Path(user_id): Path<u64>) {}";
        let position = Position {
            line: 1,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should filter to bindings starting with 'u'
        assert!(completions.iter().any(|c| c.label == "user_id"));
    }

    #[test]
    fn test_completions_out_of_bounds() {
        let content = "/// @tag";
        let position = Position {
            line: 5, // Out of bounds
            character: 0,
        };
        let completions = get_completions(content, position);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_detect_context_stops_at_non_comment() {
        let lines = vec!["fn foo() {}", "/// # Responses", "/// "];
        let context = detect_section_context(&lines, 2);
        assert_eq!(context, SectionContext::ResponsesSection);
    }

    #[test]
    fn test_extract_path_bindings_no_path() {
        let content =
            "/// # Path Parameters\n/// \n#[rovo]\nasync fn get_user(Query(q): Query<String>) {}";
        let lines: Vec<&str> = content.lines().collect();
        let bindings = extract_path_bindings_from_context(content, &lines, 1);
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_get_documented_params_not_in_section() {
        let lines = vec!["/// Just a comment", "/// id: some text"];
        let documented = get_documented_path_params(&lines, 1);
        assert!(documented.is_empty());
    }
}
