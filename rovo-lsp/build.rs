use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_docs.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // Generate annotation documentation
    writeln!(f, "// Auto-generated documentation - DO NOT EDIT").unwrap();
    writeln!(f, "").unwrap();

    // Scan annotations directory
    writeln!(
        f,
        "pub fn get_annotation_documentation(annotation: &str) -> &'static str {{"
    )
    .unwrap();
    writeln!(f, "    match annotation {{").unwrap();

    let annotations_dir = Path::new(&manifest_dir).join("docs/annotations");
    if annotations_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&annotations_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let annotation_name = format!("@{}", filename);

            // Convert path to string with forward slashes (works on all platforms)
            let path_str = path.to_str().unwrap().replace('\\', "/");

            writeln!(f, "        \"{}\" => {{", annotation_name).unwrap();
            writeln!(f, "            include_str!(\"{}\").trim()", path_str).unwrap();
            writeln!(f, "        }}").unwrap();

            // Tell Cargo to rerun if the file changes
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    writeln!(
        f,
        "        _ => \"Unknown annotation - check available annotations\","
    )
    .unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();

    // Generate annotation summary by extracting first line after # heading
    writeln!(
        f,
        "pub fn get_annotation_summary(annotation: &str) -> &'static str {{"
    )
    .unwrap();
    writeln!(f, "    match annotation {{").unwrap();

    if annotations_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&annotations_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let annotation_name = format!("@{}", filename);

            // Read the file and extract the summary (first non-empty line after # heading)
            let content = fs::read_to_string(&path).unwrap();
            let summary = extract_summary(&content);

            writeln!(f, "        \"{}\" => \"{}\",", annotation_name, summary).unwrap();
        }
    }

    writeln!(f, "        _ => \"Unknown annotation\",").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();

    // Generate status code documentation
    writeln!(
        f,
        "pub fn get_status_code_from_markdown(code: u16) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(f, "    match code {{").unwrap();

    let status_codes_dir = Path::new(&manifest_dir).join("docs/status-codes");
    if status_codes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&status_codes_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();

            if let Ok(code) = filename.parse::<u16>() {
                // Read the file and extract title and description
                let content = fs::read_to_string(&path).unwrap();
                let formatted = format_status_code_info(&content, code);

                writeln!(f, "        {} => Some(\"{}\"),", code, formatted).unwrap();

                // Tell Cargo to rerun if the file changes
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    writeln!(f, "        _ => None,").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();

    // Tell Cargo to rerun if the directories change
    println!("cargo:rerun-if-changed={}", annotations_dir.display());
    println!("cargo:rerun-if-changed={}", status_codes_dir.display());
}

fn extract_summary(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Skip the first line (# heading) and find the first non-empty line
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            return trimmed.to_string();
        }
    }

    "No description available".to_string()
}

fn format_status_code_info(content: &str, code: u16) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Extract title from first line (# XXX Title)
    let title = if let Some(first_line) = lines.first() {
        let first_line = first_line.trim();
        if first_line.starts_with('#') {
            // Remove the # and the status code, keep only the title
            let without_hash = first_line.trim_start_matches('#').trim();
            // Remove the code prefix (e.g., "200 " from "200 OK")
            let code_str = code.to_string();
            if let Some(title_part) = without_hash.strip_prefix(&code_str) {
                title_part.trim().to_string()
            } else {
                without_hash.to_string()
            }
        } else {
            format!("Status {}", code)
        }
    } else {
        format!("Status {}", code)
    };

    // Extract description (all lines after the first non-empty line after the heading)
    let description: String = lines
        .iter()
        .skip(1)
        .skip_while(|line| line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\\n");

    format!("**{} {}**\\n\\n{}", code, title, description)
}
