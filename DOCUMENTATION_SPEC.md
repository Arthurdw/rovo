# Rovo Documentation Format Specification

**Version:** 2.0
**Status:** Draft
**Branch:** `rustdoc-compatibility`

## Overview

This document specifies the Rust-style documentation format for Rovo API handlers. The format follows Rust documentation conventions using markdown sections while maintaining concise annotation syntax for metadata.

## Design Goals

1. **Rust-native**: Use standard Rust doc comment conventions (`///` and markdown)
2. **Linter-friendly**: Compatible with standard markdown linters
3. **Readable**: Natural to read in source code and rendered documentation
4. **Concise**: Avoid unnecessary verbosity for simple fields
5. **Clear semantics**: Distinguish between behavioral documentation and API metadata

## Format Structure

### Basic Template

```rust
/// Brief description of the endpoint (one line).
///
/// Longer description with more details about what this endpoint does,
/// its purpose, and any important context.
///
/// # Responses
///
/// <status_code>: <type> - <description>
/// <status_code>: <type> - <description>
///
/// # Examples
///
/// <status_code>: <rust_expression>
/// <status_code>: <rust_expression>
///
/// # Metadata
///
/// @id <operation_id>
/// @tag <tag_name>
/// @security <scheme_name>
/// @hidden
#[rovo]
async fn handler_name(/* ... */) -> impl IntoApiResponse {
    // implementation
}
```

## Section Specifications

### 1. Brief Description (Required)

The first line of the doc comment is a brief, one-line summary.

```rust
/// Get a todo item by its ID.
```

**Rules:**
- Must be the first line
- Should be concise (< 80 characters recommended)
- Should start with a verb (Get, Create, Update, Delete, List, etc.)
- Must end with a period

### 2. Detailed Description (Optional)

Additional paragraphs providing context, usage notes, or important details.

```rust
/// Get a todo item by its ID.
///
/// This endpoint retrieves a single todo item from the database.
/// If the item doesn't exist, a 404 error is returned.
```

**Rules:**
- Separated from brief description by a blank line (`///`)
- Can contain multiple paragraphs
- Standard markdown formatting supported
- Appears before any section headers

### 3. Responses Section (Optional but recommended)

Documents HTTP responses with status codes, types, and descriptions.

#### Syntax

```
/// # Responses
///
/// <status_code>: <type> - <description>
```

#### Format Details

- **status_code**: HTTP status code (100-599)
- **type**: Rust type expression (e.g., `Json<T>`, `()`, `(StatusCode, Json<T>)`)
- **description**: Human-readable description of when this response occurs

#### Examples

**Single-line responses:**

```rust
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item
/// 404: () - Todo item was not found
/// 500: Json<ErrorResponse> - Internal server error occurred
```

**Multi-line responses:**

Description can span multiple lines. Continuation lines are trimmed and joined with spaces:

```rust
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item from the
///      database with all associated metadata
/// 404: () - Todo item was not found in the database or has been
///      deleted by another user
```

Parses as:
- `200: Json<TodoItem> - Successfully retrieved the todo item from the database with all associated metadata`
- `404: () - Todo item was not found in the database or has been deleted by another user`

**Multiple response types for same status:**

```rust
/// # Responses
///
/// 200: Json<TodoItem> - Todo item retrieved successfully
/// 200: (StatusCode, Json<TodoItem>) - Todo item retrieved with custom headers
/// 404: () - Todo item not found
```

**Rules:**
- Status codes must be valid HTTP status codes (100-599)
- Type must be valid Rust syntax
- Description is required and should explain when this response occurs
- Multiple responses with the same status code are allowed
- Order doesn't matter but convention is to list success cases first

### 4. Examples Section (Optional)

Provides concrete Rust code examples of response values.

#### Syntax

```
/// # Examples
///
/// <status_code>: <rust_expression>
```

#### Format Details

- **status_code**: HTTP status code matching a response
- **rust_expression**: Valid Rust expression that produces the response type

#### Examples

**Single-line examples:**

```rust
/// # Examples
///
/// 200: TodoItem { id: 1, title: "Buy milk".into(), completed: false }
/// 404: ()
```

**Multi-line examples:**

Parser collects lines until the expression is complete (brackets/braces close). Whitespace is normalized:

```rust
/// # Examples
///
/// 200: TodoItem {
///          id: 1,
///          title: "Buy milk".into(),
///          completed: false
///      }
/// 404: ()
```

**Primitive type examples:**

```rust
/// # Examples
///
/// 200: "success"           // String
/// 200: 42                  // i32
/// 200: true                // bool
/// 200: 3.14                // f64
/// 404: ()                  // unit
```

**Complex nested examples:**

```rust
/// # Examples
///
/// 200: Json(User {
///     id: 1,
///     name: "Alice".into(),
///     email: "alice@example.com".into(),
///     created_at: Utc::now(),
/// })
/// 400: Json(ErrorResponse {
///     error: "Invalid user ID".into(),
///     code: "INVALID_ID".into(),
/// })
```

**Rules:**
- Status code should correspond to a documented response
- Expression must be valid Rust syntax
- Expression should compile and produce the documented type
- Multi-line expressions supported (parser tracks bracket/brace depth)
- Indentation doesn't matter - will be normalized during parsing

### 5. Metadata Section (Optional)

Contains API metadata annotations using the `@` syntax.

#### Syntax

```
/// # Metadata
///
/// @id <operation_id>
/// @tag <tag_name>
/// @security <scheme_name>
/// @hidden
```

#### Available Annotations

##### @id

Sets a custom operation ID for the endpoint.

```rust
/// # Metadata
///
/// @id get_todo_by_id
```

**Rules:**
- Operation ID must contain only alphanumeric characters and underscores
- No hyphens or special characters
- Must be unique across all endpoints
- Optional (defaults to function name with module path)

##### @tag

Categorizes the endpoint into a logical group.

```rust
/// # Metadata
///
/// @tag todos
/// @tag users
```

**Rules:**
- Tag name cannot be empty
- Multiple `@tag` annotations allowed (endpoint appears in multiple groups)
- Convention: use lowercase with underscores for multi-word tags

##### @security

Declares security schemes required for this endpoint.

```rust
/// # Metadata
///
/// @security bearer
/// @security api_key
```

**Rules:**
- Scheme name must match a security scheme defined in the API specification
- Multiple `@security` annotations allowed (all schemes required)
- For optional security, document multiple response cases

##### @hidden

Excludes the endpoint from generated documentation.

```rust
/// # Metadata
///
/// @hidden
```

**Rules:**
- Takes no parameters
- Endpoint will not appear in OpenAPI spec
- Useful for internal/debug endpoints

### 6. @rovo-ignore Directive (Location-independent)

Special directive that stops annotation processing at its location.

```rust
/// # Responses
///
/// 200: Json<String> - Success
///
/// @rovo-ignore
///
/// Everything after this is ignored.
/// @invalid_annotation won't cause compile errors
/// Random content that might confuse the parser
```

**Rules:**
- Can appear anywhere in the doc comment
- Everything after it is ignored by the parser
- Useful for:
  - Adding notes that shouldn't be processed
  - Temporarily disabling annotations
  - Working around parser limitations

## Complete Examples

### Simple Endpoint

```rust
/// Get a todo item by ID.
///
/// # Responses
///
/// 200: Json<TodoItem> - Successfully retrieved the todo item
/// 404: () - Todo item was not found
///
/// # Metadata
///
/// @tag todos
#[rovo]
async fn get_todo(
    State(app): State<AppState>,
    Path(TodoId { id }): Path<TodoId>,
) -> impl IntoApiResponse {
    match app.db.get_todo(id).await {
        Some(todo) => (StatusCode::OK, Json(todo)),
        None => (StatusCode::NOT_FOUND, Json(())),
    }
}
```

### Complex Endpoint with All Features

```rust
/// Create a new todo item.
///
/// Creates a new todo item in the database. The title must be non-empty
/// and the item starts in an incomplete state by default.
///
/// # Responses
///
/// 201: Json<TodoItem> - Todo item created successfully
/// 400: Json<ErrorResponse> - Invalid input data
/// 401: () - Authentication required
/// 500: Json<ErrorResponse> - Internal server error
///
/// # Examples
///
/// 201: Json(TodoItem {
///     id: 1,
///     title: "Buy groceries".into(),
///     completed: false,
/// })
/// 400: Json(ErrorResponse {
///     error: "Title cannot be empty".into(),
///     code: "VALIDATION_ERROR".into(),
/// })
///
/// # Metadata
///
/// @id create_todo_item
/// @tag todos
/// @security bearer
#[rovo]
async fn create_todo(
    State(app): State<AppState>,
    Json(input): Json<CreateTodoInput>,
) -> impl IntoApiResponse {
    // implementation
}
```

### Multiple Tags and Security

```rust
/// Get user profile with admin access.
///
/// # Responses
///
/// 200: Json<UserProfile> - Profile retrieved successfully
/// 403: () - Insufficient permissions
///
/// # Metadata
///
/// @tag users
/// @tag admin
/// @security bearer
/// @security admin_key
#[rovo]
async fn get_admin_profile(Path(id): Path<i32>) -> impl IntoApiResponse {
    // implementation
}
```

### Hidden Internal Endpoint

```rust
/// Internal health check endpoint.
///
/// # Responses
///
/// 200: Json<HealthStatus> - System is healthy
///
/// # Metadata
///
/// @hidden
#[rovo]
async fn internal_health() -> impl IntoApiResponse {
    // implementation
}
```

### Using @rovo-ignore

```rust
/// Experimental endpoint.
///
/// # Responses
///
/// 200: Json<Data> - Success
///
/// @rovo-ignore
///
/// TODO: Add more response types
/// TODO: Add security
/// These TODOs won't cause parser errors
#[rovo]
async fn experimental() -> impl IntoApiResponse {
    // implementation
}
```

## Parser Behavior

### Section Parsing

1. Sections are identified by markdown headings (`# SectionName`)
2. Section content continues until the next heading or end of comment
3. Section order doesn't matter
4. Sections are optional (though `# Responses` is recommended)
5. Unknown sections are ignored (forward compatibility)

### Annotation Parsing

1. Annotations start with `@` followed by the keyword
2. Annotations in `# Metadata` section are parsed
3. `@rovo-ignore` can appear anywhere and stops all parsing after it
4. Unknown annotations produce compile-time errors with suggestions
5. Multiple annotations of the same type are allowed where it makes sense

### Error Handling

The parser provides helpful error messages:

```
error: unknown annotation `@tg`
  --> src/main.rs:10:5
   |
10 | /// @tg users
   |     ^^^
   |
   = help: did you mean `@tag`?
```


## Implementation Notes

### Parser Requirements

- Must recognize markdown headings (`# SectionName`)
- Must parse section content until next heading
- Must handle `@` annotations within `# Metadata` section
- Must preserve `@rovo-ignore` behavior
- Must provide helpful error messages

### LSP Requirements

#### Core Features
- Autocomplete for section names (`# Responses`, `# Examples`, `# Metadata`)
- Autocomplete for annotation keywords (`@tag`, `@security`, `@id`, `@hidden`)
- Validation of status codes, types, and syntax
- Hover documentation for annotations and sections

#### Context-Aware Snippets

Snippets should be context-aware based on existing content:

1. **Adding `# Responses` section:**
   - If section doesn't exist: Insert entire section template with blank line before
   - If section exists: Add new response line in proper format

   ```rust
   // Section doesn't exist - insert:
   /// # Responses
   ///
   /// ${1:200}: ${2:Json<T>} - ${3:description}

   // Section exists - insert just the line:
   /// ${1:200}: ${2:Json<T>} - ${3:description}
   ```

2. **Adding `# Examples` section:**
   - If section doesn't exist: Insert entire section template
   - If section exists: Add new example line

3. **Adding `# Metadata` section:**
   - If section doesn't exist: Insert entire section template
   - If section exists: Add annotation on new line

4. **Adding annotations:**
   - Automatically place inside `# Metadata` section if it exists
   - Create `# Metadata` section if adding first annotation

#### Multi-line Parsing Support

LSP must handle:
- Multi-line response descriptions (join continuation lines)
- Multi-line example expressions (track bracket/brace depth)
- Proper syntax highlighting across multiple lines
- Indentation-aware formatting

### Testing Requirements

- Unit tests for each section type
- Integration tests with complete examples
- Error case testing (invalid syntax, unknown annotations)
- LSP feature tests

## Design Decisions

1. **Markdown support**: Yes, standard markdown is supported in descriptions
2. **Case sensitivity**: Yes, sections must match exactly: `# Responses`, `# Examples`, `# Metadata`
3. **Annotation placement**: All annotations (except `@rovo-ignore`) must be under `# Metadata`
4. **Directive naming**: Keep `@rovo-ignore` for clarity and avoiding conflicts with other tools

## Future Considerations

- Support for request body documentation (`# Request Body`)
- Support for path/query parameter documentation (`# Parameters`)
- Support for header documentation (`# Headers`)
- Integration with cargo doc for HTML documentation generation
- OpenAPI schema generation improvements

---

**Document History:**
- v2.0 (2025-11-24): Initial specification for Rust-style documentation format
