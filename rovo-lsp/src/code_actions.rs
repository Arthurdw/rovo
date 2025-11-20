use crate::parser::{parse_annotations, AnnotationKind};
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

    // Action 1: Add @response (generic - user fills in details)
    actions.push(create_insert_annotation_action(
        "Add @response",
        "/// @response STATUS TYPE Description",
        insert_line,
        uri.clone(),
    ));

    // Action 2: Add @tag
    actions.push(create_insert_annotation_action(
        "Add @tag",
        "/// @tag TAG_NAME",
        insert_line,
        uri.clone(),
    ));

    // Action 3: Add @security
    actions.push(create_insert_annotation_action(
        "Add @security",
        "/// @security SCHEME",
        insert_line,
        uri.clone(),
    ));

    // Action 4: Add @example
    actions.push(create_insert_annotation_action(
        "Add @example",
        "/// @example STATUS {\"key\": \"value\"}",
        insert_line,
        uri.clone(),
    ));

    // Action 5: Add @id annotation (only if missing in this block)
    let has_id = filtered_annotations.iter().any(|ann| ann.kind == AnnotationKind::Id);

    if !has_id {
        actions.push(create_insert_annotation_action(
            "Add @id",
            "/// @id OPERATION_ID",
            insert_line,
            uri.clone(),
        ));
    }

    // Action 6: Add @hidden annotation (only if missing in this block)
    let has_hidden = filtered_annotations
        .iter()
        .any(|ann| ann.kind == AnnotationKind::Hidden);

    if !has_hidden {
        actions.push(create_insert_annotation_action(
            "Add @hidden",
            "/// @hidden",
            insert_line,
            uri.clone(),
        ));
    }

    // Action 7: Add full REST response set (only if this block has no annotations yet)
    if filtered_annotations.is_empty() {
        actions.push(create_insert_multiple_annotations_action(
            "Add common REST responses",
            vec![
                "/// @response 200 Json<T> Success",
                "/// @response 400 Json<Error> Bad request",
                "/// @response 404 Json<Error> Not found",
                "/// @response 500 Json<Error> Internal server error",
            ],
            insert_line,
            uri.clone(),
        ));
    }

    actions
}

fn create_insert_annotation_action(
    title: &str,
    text: &str,
    line: usize,
    uri: Url,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            new_text: format!("{}\n", text),
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn create_insert_multiple_annotations_action(
    title: &str,
    lines: Vec<&str>,
    line: usize,
    uri: Url,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();
    let text = lines.join("\n") + "\n";

    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            new_text: text,
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    })
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

    // This is a simplified version - in production you'd parse the line more carefully
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range,
            new_text: format!("/// @response {} ", new_status),
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
        // Add JsonSchema to existing #[derive(...)]
        let line_num = derive_line.unwrap();
        let existing_line = lines.get(line_num).unwrap_or(&"");

        // Parse the existing derive and add JsonSchema
        let new_line = if let Some(start) = existing_line.find("derive(") {
            let before = &existing_line[..start + 7]; // "derive("
            let after_start = start + 7;

            if let Some(end) = existing_line[after_start..].find(")]") {
                let derives = &existing_line[after_start..after_start + end];
                let new_line = format!("{}JsonSchema, {})]", before, derives);
                new_line
            } else {
                existing_line.to_string()
            }
        } else {
            existing_line.to_string()
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
