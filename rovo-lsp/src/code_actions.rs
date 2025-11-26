use crate::parser::{parse_annotations, AnnotationKind};
use syn::{parse_str, Meta};
use tower_lsp::lsp_types::*;

/// Get available code actions for the given range
///
/// Provides quick fixes and refactoring actions like adding #[rovo] attributes,
/// inserting annotations, or adding JsonSchema derives.
///
/// # Arguments
/// * `content` - The document content
/// * `range` - The selected range in the document
/// * `uri` - Document URI for constructing edit locations
///
/// # Returns
/// A vector of available code actions
pub fn get_code_actions(content: &str, range: Range, uri: Url) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    let start_line = range.start.line as usize;
    let lines: Vec<&str> = content.lines().collect();

    if start_line >= lines.len() {
        return actions;
    }

    // Check if we're inside or near (including above) a function with #[rovo]
    let (is_near_rovo, rovo_line, _fn_line) = find_rovo_function_context(content, start_line);

    // Check if we're in a struct and offer JsonSchema derive
    if let Some(struct_info) = find_struct_context(content, start_line) {
        if !struct_info.has_jsonschema {
            actions.push(create_add_jsonschema_action(
                content,
                struct_info.derive_line,
                struct_info.struct_line,
                struct_info.has_derive,
                uri.clone(),
            ));
        }
        // Don't return - might also be in a function
    }

    if !is_near_rovo {
        // Not in a rovo function - offer to initialize Rovo only if we're in a function
        if let Some((_fn_line, attr_insert_line)) = find_function_for_rovo_init(content, start_line)
        {
            actions.push(create_init_rovo_action(attr_insert_line, uri.clone()));
        }
        return actions;
    }

    // We're in a rovo function - find where to insert annotations (above #[rovo])
    let insert_line = rovo_line.unwrap_or(start_line);

    // Parse existing annotations and filter to only those for this #[rovo] block
    let all_annotations = parse_annotations(content);

    // Find the doc comment range for this specific #[rovo] block
    let doc_start_line = {
        let mut start = insert_line;
        while start > 0 {
            let prev_line = start - 1;
            if prev_line < lines.len() && lines[prev_line].trim_start().starts_with("///") {
                start = prev_line;
            } else {
                break;
            }
        }
        start
    };

    // Filter annotations to only those in the current doc block
    let filtered_annotations: Vec<_> = all_annotations
        .iter()
        .filter(|ann| ann.line >= doc_start_line && ann.line < insert_line)
        .collect();

    // Action 1: Add response entry
    actions.push(create_smart_section_action(
        content,
        "Add response",
        "Responses",
        "200: Json<T> - Description",
        doc_start_line,
        insert_line,
        uri.clone(),
    ));

    // Action 2: Add example entry
    actions.push(create_smart_section_action(
        content,
        "Add example",
        "Examples",
        "200: T::default()",
        doc_start_line,
        insert_line,
        uri.clone(),
    ));

    // Check if Metadata section exists
    let (has_metadata, _, _) = find_section(content, "Metadata", doc_start_line, insert_line);

    // Action 3: Add @tag
    actions.push(create_smart_metadata_action(
        content,
        "Add @tag",
        "@tag TAG_NAME",
        has_metadata,
        doc_start_line,
        insert_line,
        uri.clone(),
    ));

    // Action 4: Add @security
    actions.push(create_smart_metadata_action(
        content,
        "Add @security",
        "@security SCHEME",
        has_metadata,
        doc_start_line,
        insert_line,
        uri.clone(),
    ));

    // Action 5: Add @id annotation (only if missing in this block)
    let has_id = filtered_annotations
        .iter()
        .any(|ann| ann.kind == AnnotationKind::Id);

    if !has_id {
        actions.push(create_smart_metadata_action(
            content,
            "Add @id",
            "@id OPERATION_ID",
            has_metadata,
            doc_start_line,
            insert_line,
            uri.clone(),
        ));
    }

    // Action 6: Add @hidden annotation (only if missing in this block)
    let has_hidden = filtered_annotations
        .iter()
        .any(|ann| ann.kind == AnnotationKind::Hidden);

    if !has_hidden {
        actions.push(create_smart_metadata_action(
            content,
            "Add @hidden",
            "@hidden",
            has_metadata,
            doc_start_line,
            insert_line,
            uri.clone(),
        ));
    }

    // Action 7: Add full REST response set (only if this block has no responses yet)
    let has_responses = filtered_annotations.iter().any(|ann| {
        matches!(
            ann.kind,
            AnnotationKind::Response | AnnotationKind::ResponsesSection
        )
    });

    if !has_responses {
        actions.push(create_smart_rest_responses_action(
            content,
            doc_start_line,
            insert_line,
            uri.clone(),
        ));
    }

    actions
}

/// Get code actions to fix diagnostics
///
/// Provides quick fixes for issues like invalid status codes.
///
/// # Arguments
/// * `content` - The document content
/// * `diagnostic` - The diagnostic to fix
/// * `uri` - Document URI for constructing edit locations
///
/// # Returns
/// A vector of quick fix actions
pub fn get_diagnostic_code_actions(
    content: &str,
    diagnostic: &Diagnostic,
    uri: Url,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Check if this is an invalid status code error
    if diagnostic.message.contains("Invalid HTTP status") {
        // Extract the invalid status code from the diagnostic
        let line = diagnostic.range.start.line as usize;
        let lines: Vec<&str> = content.lines().collect();

        if line < lines.len() {
            // Suggest common valid status codes
            for status in [200, 201, 400, 404, 500] {
                actions.push(create_fix_status_code_action(
                    format!("Change to {}", status).as_str(),
                    status,
                    diagnostic.range,
                    uri.clone(),
                ));
            }
        }
    }

    actions
}

fn create_fix_status_code_action(
    title: &str,
    new_status: u16,
    range: Range,
    uri: Url,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();

    // Range already targets just the status code digits; only swap the numbers
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range,
            new_text: new_status.to_string(),
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        is_preferred: Some(new_status == 200),
        ..Default::default()
    })
}

/// Find if we're inside, above (in comments), or near a function with #[rovo]
/// Returns: (is_near_rovo, rovo_line_number, function_line_number)
fn find_rovo_function_context(
    content: &str,
    current_line: usize,
) -> (bool, Option<usize>, Option<usize>) {
    let lines: Vec<&str> = content.lines().collect();

    // Case 0: Check if we're on the #[rovo] line or function signature with #[rovo] above
    if let Some(line) = lines.get(current_line) {
        let trimmed = line.trim();

        // On #[rovo] line itself
        if trimmed.starts_with("#[") && line.contains("rovo") {
            // Look forward for function
            for i in (current_line + 1)..std::cmp::min(current_line + 5, lines.len()) {
                let fwd_line = lines.get(i).unwrap_or(&"");
                if fwd_line.contains("fn ") && !fwd_line.trim().starts_with("//") {
                    return (true, Some(current_line), Some(i));
                }
            }
            return (true, Some(current_line), None);
        }

        // On function signature - check if #[rovo] is above
        if trimmed.contains("fn ") && !trimmed.starts_with("//") {
            for i in (current_line.saturating_sub(5)..current_line).rev() {
                let back_line = lines.get(i).unwrap_or(&"");
                if back_line.trim().starts_with("#[") && back_line.contains("rovo") {
                    return (true, Some(i), Some(current_line));
                }
            }
        }
    }

    // Case 1: Check if we're in a doc comment above a #[rovo] function
    if lines
        .get(current_line)
        .map(|l| l.trim().starts_with("///"))
        .unwrap_or(false)
    {
        // Check if there's a continuous comment block from here to #[rovo]
        let mut found_rovo = None;
        let mut found_fn = None;

        // Look forward, but only through continuous comments/attributes
        for i in current_line..std::cmp::min(current_line + 20, lines.len()) {
            let line = lines.get(i).unwrap_or(&"");
            let trimmed = line.trim();

            // Found #[rovo]
            if trimmed.starts_with("#[") && line.contains("rovo") {
                found_rovo = Some(i);
            }

            // Found function - check if we already found #[rovo]
            if trimmed.contains("fn ") && !trimmed.starts_with("//") {
                if found_rovo.is_some() {
                    found_fn = Some(i);
                }
                break; // Stop at function
            }

            // Stop if we hit a non-comment, non-attribute, non-empty line
            if !trimmed.starts_with("///")
                && !trimmed.starts_with("#[")
                && !trimmed.is_empty()
                && !trimmed.contains("fn ")
            {
                break;
            }

            // Stop if we hit a blank line followed by non-comment (end of comment block)
            if trimmed.is_empty() && i > current_line {
                if let Some(next_line) = lines.get(i + 1) {
                    let next_trimmed = next_line.trim();
                    if !next_trimmed.starts_with("///") && !next_trimmed.starts_with("#[") {
                        break;
                    }
                }
            }
        }

        if let (Some(rovo), Some(func)) = (found_rovo, found_fn) {
            return (true, Some(rovo), Some(func));
        }
    }

    // Case 2: Check if we're inside a function with #[rovo] above it
    // First, check if we're actually inside a function (not after it ended)

    // Count braces backwards from current line
    let mut brace_count = 0;
    let mut found_fn = None;

    for i in (0..=current_line).rev() {
        let line = lines.get(i).unwrap_or(&"");

        // Count closing braces
        brace_count += line.matches('}').count() as i32;
        // Subtract opening braces
        brace_count -= line.matches('{').count() as i32;

        // If we found more closing than opening, we're outside any function
        if brace_count > 0 {
            return (false, None, None);
        }

        // Found function signature
        if line.contains("fn ") && !line.trim().starts_with("//") {
            found_fn = Some(i);
            break;
        }
    }

    let fn_line = match found_fn {
        Some(line) => line,
        None => return (false, None, None),
    };

    // Verify there's an opening brace between function and current line (or a bit after)
    // We check a few lines ahead to handle function signatures that span multiple lines
    let mut has_opening_brace = false;
    for i in fn_line..=std::cmp::min(current_line + 3, lines.len().saturating_sub(1)) {
        if lines.get(i).unwrap_or(&"").contains("{") {
            has_opening_brace = true;
            break;
        }
    }

    if !has_opening_brace {
        return (false, None, None);
    }

    // Search upwards from function for #[rovo]
    for i in (fn_line.saturating_sub(10)..fn_line).rev() {
        let line = lines.get(i).unwrap_or(&"");

        // Found #[rovo] attribute
        if line.trim().starts_with("#[") && line.contains("rovo") {
            return (true, Some(i), Some(fn_line));
        }

        // Stop if we hit another function
        if line.contains("fn ") {
            break;
        }
    }

    (false, None, None)
}

/// Find function for rovo initialization (returns function line and where to insert attribute)
fn find_function_for_rovo_init(content: &str, current_line: usize) -> Option<(usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();

    let current_line_content = lines.get(current_line)?;
    let trimmed = current_line_content.trim();

    // Check if current line is a function signature
    let is_fn_signature = trimmed.contains("fn ") && !trimmed.starts_with("//");

    // Only trigger if:
    // 1. Current line is a function signature, OR
    // 2. Current line is indented (inside function body)
    // But NOT if it's a comment, attribute, or empty
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#[") {
        return None;
    }

    let is_indented =
        current_line_content.starts_with(" ") || current_line_content.starts_with("\t");

    if !is_fn_signature && !is_indented {
        return None;
    }

    // Find function signature (either current line or above)
    let fn_line = if is_fn_signature {
        current_line
    } else {
        let mut found_fn = None;
        for i in (0..current_line).rev() {
            let line = lines.get(i)?;
            if line.contains("fn ") && !line.trim().starts_with("//") {
                found_fn = Some(i);
                break;
            }

            // Stop if we hit another closing brace (we're outside any function)
            if line.trim() == "}" {
                return None;
            }
        }
        found_fn?
    };

    // Verify there's an opening brace at or after the function signature
    // Check a few lines ahead to handle multi-line signatures
    let mut found_opening_brace = false;
    for i in fn_line..=std::cmp::min(current_line + 3, lines.len().saturating_sub(1)) {
        if lines.get(i).unwrap_or(&"").contains("{") {
            found_opening_brace = true;
            break;
        }
    }

    if !found_opening_brace {
        return None;
    }

    // Check if this function already has #[rovo]
    for i in (fn_line.saturating_sub(10)..fn_line).rev() {
        let line = lines.get(i).unwrap_or(&"");
        if line.trim().starts_with("#[") && line.contains("rovo") {
            return None; // Already has rovo
        }
        if line.contains("fn ") {
            break;
        }
    }

    // Insert #[rovo] right above the function definition (after doc comments)
    let insert_line = fn_line;

    Some((fn_line, insert_line))
}

struct StructContext {
    struct_line: usize,
    derive_line: Option<usize>,
    has_derive: bool,
    has_jsonschema: bool,
}

/// Find if we're in a struct and check for JsonSchema derive
fn find_struct_context(content: &str, current_line: usize) -> Option<StructContext> {
    let lines: Vec<&str> = content.lines().collect();

    // Find struct definition at or above current line
    let mut struct_line = None;
    for i in (0..=current_line).rev() {
        let line = lines.get(i)?;
        if (line.contains("struct ") || line.contains("enum ")) && !line.trim().starts_with("//") {
            struct_line = Some(i);
            break;
        }
    }

    let struct_line = struct_line?;

    // Check if we're inside the struct (after opening brace)
    let mut inside_struct = false;
    for i in struct_line..std::cmp::min(struct_line + 2, lines.len()) {
        let line = lines.get(i)?;
        if line.contains("{") && current_line >= i {
            inside_struct = true;
            break;
        }
    }

    if !inside_struct {
        return None;
    }

    // Look for #[derive(...)] above the struct
    let mut derive_line = None;
    let mut has_jsonschema = false;

    for i in (struct_line.saturating_sub(10)..struct_line).rev() {
        let line = lines.get(i).unwrap_or(&"");

        if line.trim().starts_with("#[derive(") {
            derive_line = Some(i);
            has_jsonschema = line.contains("JsonSchema");
            break;
        }

        // Stop if we hit a non-attribute line
        if !line.trim().starts_with("#[")
            && !line.trim().is_empty()
            && !line.trim().starts_with("///")
        {
            break;
        }
    }

    Some(StructContext {
        struct_line,
        derive_line,
        has_derive: derive_line.is_some(),
        has_jsonschema,
    })
}

/// Create action to add JsonSchema to a struct
fn create_add_jsonschema_action(
    content: &str,
    derive_line: Option<usize>,
    struct_line: usize,
    has_derive: bool,
    uri: Url,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    if has_derive {
        // Add JsonSchema to existing #[derive(...)] using syn for robust parsing
        let line_num = derive_line.unwrap();
        let existing_line = lines.get(line_num).unwrap_or(&"");

        // Extract the content inside #[...]
        let new_line = if let Some(start) = existing_line.find("#[") {
            let after_start = start + 2;
            if let Some(end) = existing_line[after_start..].find(']') {
                let meta_str = &existing_line[after_start..after_start + end];

                // Try to parse as Meta
                match parse_str::<Meta>(meta_str) {
                    Ok(Meta::List(meta_list)) if meta_list.path.is_ident("derive") => {
                        // Successfully parsed as derive attribute
                        let tokens_str = meta_list.tokens.to_string();
                        let new_tokens = if tokens_str.trim().is_empty() {
                            "JsonSchema".to_string()
                        } else {
                            format!("JsonSchema, {}", tokens_str)
                        };

                        // Reconstruct with original indentation
                        let indentation = existing_line.len() - existing_line.trim_start().len();
                        let indent = " ".repeat(indentation);
                        format!("{}#[derive({})]", indent, new_tokens)
                    }
                    _ => {
                        // Parsing failed or not a derive - fall back to new attribute
                        let indentation = existing_line.len() - existing_line.trim_start().len();
                        let indent = " ".repeat(indentation);
                        format!("{}#[derive(JsonSchema)]", indent)
                    }
                }
            } else {
                // Malformed attribute - fall back
                let indentation = existing_line.len() - existing_line.trim_start().len();
                let indent = " ".repeat(indentation);
                format!("{}#[derive(JsonSchema)]", indent)
            }
        } else {
            // No #[ found - shouldn't happen but fall back
            "#[derive(JsonSchema)]".to_string()
        };

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: existing_line.len() as u32,
                    },
                },
                new_text: new_line,
            }],
        );

        CodeActionOrCommand::CodeAction(CodeAction {
            title: "Add JsonSchema to derive".to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            ..Default::default()
        })
    } else {
        // Create new #[derive(JsonSchema)]
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: struct_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: struct_line as u32,
                        character: 0,
                    },
                },
                new_text: "#[derive(JsonSchema)]\n".to_string(),
            }],
        );

        CodeActionOrCommand::CodeAction(CodeAction {
            title: "Add JsonSchema derive".to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

/// Create action to initialize Rovo on a function
fn create_init_rovo_action(insert_line: usize, uri: Url) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();

    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: insert_line as u32,
                    character: 0,
                },
                end: Position {
                    line: insert_line as u32,
                    character: 0,
                },
            },
            new_text: "#[rovo]\n".to_string(),
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Add #[rovo] macro".to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Find the effective doc_end, respecting @rovo-ignore boundary
/// Everything after @rovo-ignore should be ignored per the documentation spec
fn find_effective_doc_end(content: &str, doc_start: usize, doc_end: usize) -> usize {
    let lines: Vec<&str> = content.lines().collect();

    for i in doc_start..doc_end {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim_start_matches("///").trim();
            if trimmed.starts_with("@rovo-ignore") {
                return i;
            }
        }
    }

    doc_end
}

/// Determine blank line requirements around a section insertion
/// Returns (needs_prefix_blank, needs_suffix_blank)
fn check_blank_line_requirements(lines: &[&str], insert_line: usize) -> (bool, bool) {
    let needs_blank_line = if insert_line > 0 && insert_line <= lines.len() {
        lines[insert_line - 1].trim() != "///"
    } else {
        true
    };

    let needs_suffix_blank = if insert_line < lines.len() {
        let next = lines[insert_line].trim();
        !next.is_empty() && next != "///"
    } else {
        false
    };

    (needs_blank_line, needs_suffix_blank)
}

/// Find a documentation section (# Responses, # Examples, # Metadata) and return its boundaries
/// Returns: (section_exists, section_start_line, last_content_line)
/// Note: Respects @rovo-ignore - sections after it are not found
fn find_section(
    content: &str,
    section_name: &str,
    doc_start: usize,
    doc_end: usize,
) -> (bool, Option<usize>, Option<usize>) {
    let lines: Vec<&str> = content.lines().collect();
    let section_header = format!("# {}", section_name);

    // Respect @rovo-ignore boundary
    let effective_end = find_effective_doc_end(content, doc_start, doc_end);

    let mut section_line = None;

    // Find the section header (only before @rovo-ignore)
    for i in doc_start..effective_end {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim_start_matches("///").trim();
            if trimmed == section_header {
                section_line = Some(i);
                break;
            }
        }
    }

    let section_start = match section_line {
        Some(line) => line,
        None => return (false, None, None),
    };

    // Find the last line of content in this section (respecting @rovo-ignore)
    let mut last_content_line = section_start;
    for i in (section_start + 1)..effective_end {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim_start_matches("///").trim();

            // Stop at next section or end of doc comments
            if trimmed.starts_with("# ") || !line.trim_start().starts_with("///") {
                break;
            }

            // Update last content line if this line has content (not just ///)
            if !trimmed.is_empty() {
                last_content_line = i;
            }
        }
    }

    (true, Some(section_start), Some(last_content_line))
}

/// Find the correct insertion point for a new section based on desired order:
/// # Responses -> # Examples -> # Metadata
/// Note: Respects @rovo-ignore - insertions will happen before @rovo-ignore if present
fn find_section_insertion_point(
    content: &str,
    section_name: &str,
    doc_start: usize,
    doc_end: usize,
) -> usize {
    let lines: Vec<&str> = content.lines().collect();

    // Respect @rovo-ignore boundary
    let effective_end = find_effective_doc_end(content, doc_start, doc_end);

    // Define section order
    let section_order = ["Responses", "Examples", "Metadata"];
    let Some(target_index) = section_order.iter().position(|&s| s == section_name) else {
        return effective_end;
    };

    // Find all section positions (only before @rovo-ignore)
    let mut section_positions: Vec<(usize, usize)> = Vec::new(); // (order_index, line_number)

    for i in doc_start..effective_end {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim_start_matches("///").trim();
            if trimmed.starts_with("# ") {
                let found_section = trimmed.trim_start_matches("# ").trim();
                if let Some(order_idx) = section_order.iter().position(|&s| s == found_section) {
                    section_positions.push((order_idx, i));
                }
            }
        }
    }

    // If no sections exist, insert at effective_end
    if section_positions.is_empty() {
        return effective_end;
    }

    // Find the right position based on order
    for (order_idx, line_num) in &section_positions {
        if *order_idx > target_index {
            // Found a section that should come after ours, insert before it
            return *line_num;
        }
    }

    // If we get here, insert after the last section
    if let Some((_, _last_section_line)) = section_positions.last() {
        // Find the end of the last section
        let (_, _, last_content_line) = find_section(
            content,
            section_order[section_positions.last().unwrap().0],
            doc_start,
            effective_end,
        );
        if let Some(end_line) = last_content_line {
            return end_line + 1;
        }
    }

    effective_end
}

/// Create a smart action that adds entries to Responses or Examples sections
/// If section exists, appends to the end. If not, creates the section.
fn create_smart_section_action(
    content: &str,
    title: &str,
    section_name: &str,
    entry: &str,
    doc_start: usize,
    doc_end: usize,
    uri: Url,
) -> CodeActionOrCommand {
    let (exists, _, last_line) = find_section(content, section_name, doc_start, doc_end);

    let mut changes = std::collections::HashMap::new();

    if exists {
        // Section exists - append after the last content line
        let insert_line = last_line.unwrap() + 1;
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text: format!("/// {}\n", entry),
            }],
        );
    } else {
        // Section doesn't exist - create it with the entry
        // Find the correct insertion point to maintain order
        let insert_line = find_section_insertion_point(content, section_name, doc_start, doc_end);

        let lines: Vec<&str> = content.lines().collect();
        let (needs_blank_line, needs_suffix_blank) =
            check_blank_line_requirements(&lines, insert_line);

        let new_text = match (needs_blank_line, needs_suffix_blank) {
            (true, true) => format!("///\n/// # {}\n///\n/// {}\n///\n", section_name, entry),
            (true, false) => format!("///\n/// # {}\n///\n/// {}\n", section_name, entry),
            (false, true) => format!("/// # {}\n///\n/// {}\n///\n", section_name, entry),
            (false, false) => format!("/// # {}\n///\n/// {}\n", section_name, entry),
        };

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text,
            }],
        );
    }

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        diagnostics: None,
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

/// Determine annotation type from annotation string
fn get_annotation_type(annotation: &str) -> &str {
    let trimmed = annotation.trim();
    if trimmed.starts_with("@id") {
        "id"
    } else if trimmed.starts_with("@tag") {
        "tag"
    } else if trimmed.starts_with("@security") {
        "security"
    } else if trimmed.starts_with("@hidden") {
        "hidden"
    } else {
        "unknown"
    }
}

/// Find the correct insertion point within a Metadata section based on annotation type
/// Order: @id -> @tag (grouped) -> @security (grouped) -> @hidden
fn find_metadata_insertion_point(
    content: &str,
    annotation: &str,
    metadata_start: usize,
    metadata_end: usize,
) -> usize {
    let lines: Vec<&str> = content.lines().collect();
    let annotation_type = get_annotation_type(annotation);

    // Define annotation order
    let annotation_order = ["id", "tag", "security", "hidden"];
    let Some(target_index) = annotation_order.iter().position(|&t| t == annotation_type) else {
        return metadata_end;
    };

    // Find all annotations in the Metadata section and their positions
    let mut annotation_positions: Vec<(usize, usize)> = Vec::new(); // (order_index, line_number)

    // Start scanning after the "# Metadata" header
    for i in (metadata_start + 1)..=metadata_end {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim_start_matches("///").trim();

            // Skip empty lines and section headers
            if trimmed.is_empty() || trimmed.starts_with("# ") {
                continue;
            }

            // Determine the type of this annotation
            let line_type = get_annotation_type(trimmed);
            if let Some(order_idx) = annotation_order.iter().position(|&t| t == line_type) {
                annotation_positions.push((order_idx, i));
            }
        }
    }

    // If no annotations exist in the section, insert right after the header
    if annotation_positions.is_empty() {
        // Skip the header line and any empty line after it
        for i in (metadata_start + 1)..=metadata_end {
            if let Some(line) = lines.get(i) {
                let trimmed = line.trim_start_matches("///").trim();
                if !trimmed.is_empty() {
                    return i;
                }
            }
        }
        return metadata_start + 1;
    }

    // Find the right position based on order
    let mut last_same_type: Option<usize> = None;

    for (order_idx, line_num) in &annotation_positions {
        if *order_idx == target_index {
            // Same type - track the last occurrence
            last_same_type = Some(*line_num);
        } else if *order_idx > target_index {
            // Found a type that comes after ours
            // If we have same-type annotations, insert after the last one
            // Otherwise, insert before this one
            return last_same_type.map(|l| l + 1).unwrap_or(*line_num);
        }
    }

    // If we get here, insert after the last annotation (all are before or same type)
    if let Some(last_line) = last_same_type.or_else(|| annotation_positions.last().map(|(_, l)| *l))
    {
        return last_line + 1;
    }

    metadata_end
}

/// Create a smart action for metadata annotations (@tag, @security, @id, @hidden)
/// If Metadata section exists, appends to it. If not, creates the section.
fn create_smart_metadata_action(
    content: &str,
    title: &str,
    annotation: &str,
    has_metadata: bool,
    doc_start: usize,
    doc_end: usize,
    uri: Url,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();

    if has_metadata {
        // Metadata section exists - find where to insert based on annotation type
        let (_, Some(metadata_start), Some(metadata_end)) =
            find_section(content, "Metadata", doc_start, doc_end)
        else {
            // Fallback - shouldn't happen if has_metadata is true
            return create_smart_section_action(
                content, title, "Metadata", annotation, doc_start, doc_end, uri,
            );
        };

        let insert_line =
            find_metadata_insertion_point(content, annotation, metadata_start, metadata_end);

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text: format!("/// {}\n", annotation),
            }],
        );
    } else {
        // Metadata section doesn't exist - create it
        // Find the correct insertion point to maintain order
        let insert_line = find_section_insertion_point(content, "Metadata", doc_start, doc_end);

        let lines: Vec<&str> = content.lines().collect();
        let (needs_blank_line, needs_suffix_blank) =
            check_blank_line_requirements(&lines, insert_line);

        let new_text = match (needs_blank_line, needs_suffix_blank) {
            (true, true) => format!("///\n/// # Metadata\n///\n/// {}\n///\n", annotation),
            (true, false) => format!("///\n/// # Metadata\n///\n/// {}\n", annotation),
            (false, true) => format!("/// # Metadata\n///\n/// {}\n///\n", annotation),
            (false, false) => format!("/// # Metadata\n///\n/// {}\n", annotation),
        };

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text,
            }],
        );
    }

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        diagnostics: None,
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}

/// Create action to add common REST responses
fn create_smart_rest_responses_action(
    content: &str,
    doc_start: usize,
    doc_end: usize,
    uri: Url,
) -> CodeActionOrCommand {
    let (exists, _, last_line) = find_section(content, "Responses", doc_start, doc_end);

    let mut changes = std::collections::HashMap::new();

    let responses = vec![
        "200: Json<T> - Success",
        "400: Json<Error> - Bad request",
        "404: Json<Error> - Not found",
        "500: Json<Error> - Internal server error",
    ];

    if exists {
        // Append to existing section
        let insert_line = last_line.unwrap() + 1;
        let text = responses
            .iter()
            .map(|r| format!("/// {}", r))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text: text,
            }],
        );
    } else {
        // Create new section
        // Find the correct insertion point to maintain order
        let insert_line = find_section_insertion_point(content, "Responses", doc_start, doc_end);

        let lines: Vec<&str> = content.lines().collect();
        let (needs_blank_line, needs_suffix_blank) =
            check_blank_line_requirements(&lines, insert_line);

        let responses_text = responses
            .iter()
            .map(|r| format!("/// {}", r))
            .collect::<Vec<_>>()
            .join("\n");

        let text = match (needs_blank_line, needs_suffix_blank) {
            (true, true) => format!("///\n/// # Responses\n///\n{}\n///\n", responses_text),
            (true, false) => format!("///\n/// # Responses\n///\n{}\n", responses_text),
            (false, true) => format!("/// # Responses\n///\n{}\n///\n", responses_text),
            (false, false) => format!("/// # Responses\n///\n{}\n", responses_text),
        };

        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: insert_line as u32,
                        character: 0,
                    },
                },
                new_text: text,
            }],
        );
    }

    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Add common REST responses".to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        diagnostics: None,
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    })
}
