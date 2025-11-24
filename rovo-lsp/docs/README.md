# Rovo LSP Documentation

This directory contains markdown files that are automatically included in the LSP at compile time.

## Structure

- **sections/**: Documentation for Rovo documentation sections (e.g., `# Responses`, `# Examples`, `# Metadata`)
- **annotations/**: Documentation for Rovo metadata annotations (e.g., `@tag`, `@security`, `@id`, `@hidden`)
- **status-codes/**: HTTP status code descriptions

## Adding New Documentation

### Sections

To add a new documentation section:

1. Create a new `.md` file in `sections/` with the section name (e.g., `custom.md`)
2. Format the file with a `#` heading as the title:
   ```markdown
   # Custom Section

   Description of the section.

   ## Format
   ...
   ```
3. The section will be automatically available in the LSP hover documentation
4. The first non-empty line after the heading becomes the summary

### Annotations

To add a new metadata annotation:

1. Create a new `.md` file in `annotations/` with the annotation name (e.g., `custom.md`)
2. Format the file with a `#` heading as the title:
   ```markdown
   # @custom

   Description of the annotation.

   ## Syntax
   ...
   ```
3. The annotation will be automatically available as `@custom` in the LSP
4. The first non-empty line after the heading becomes the summary
5. Metadata annotations should be used within the `# Metadata` section

### Status Codes

To add a new status code:

1. Create a new `.md` file in `status-codes/` with the status code number (e.g., `418.md`)
2. Format the file with the status code and title:
   ```markdown
   # 418 I'm a teapot

   The server refuses to brew coffee because it is a teapot.
   ```
3. The status code will be automatically included in hover information
4. The title is extracted from the `#` heading (everything after the status code)
5. The description includes all content after the heading

## Build Process

The `build.rs` script automatically:
- Scans these directories for `.md` files
- Generates Rust code to include them at compile time
- Extracts titles from `#` headings
- Creates match statements for fast lookup
- Tells Cargo to rebuild when files change

No manual code changes needed - just add or edit markdown files!
