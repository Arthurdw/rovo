use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;

#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub status_code: u16,
    pub response_type: TokenStream,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ExampleInfo {
    pub status_code: u16,
    pub example_code: TokenStream,
    pub span: Span,
}

/// Information about a path parameter from the `# Path Parameters` doc section
#[derive(Debug, Clone)]
pub struct PathParamDoc {
    /// Parameter name (e.g., "id", "username")
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Span for error reporting
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub responses: Vec<ResponseInfo>,
    pub examples: Vec<ExampleInfo>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub security_requirements: Vec<String>,
    pub operation_id: Option<String>,
    pub hidden: bool,
    /// Path parameter documentation from `# Path Parameters` section
    pub path_params: Vec<PathParamDoc>,
}

/// Information about path parameters extracted from function signature
#[derive(Debug, Clone)]
pub struct PathParamInfo {
    /// Binding names from the Path pattern (e.g., `["id"]` or `["collection_id", "index"]`)
    pub bindings: Vec<String>,
    /// The inner type as a string (e.g., "u64" or "(Uuid, u32)")
    pub inner_type: String,
    /// Whether this is a struct destructuring pattern (for backwards compat)
    pub is_struct_pattern: bool,
}

#[derive(Clone)]
pub struct FuncItem {
    pub name: Ident,
    pub tokens: TokenStream,
    pub state_type: Option<TokenStream>,
    /// Path parameter info extracted from function signature
    pub path_params: Option<PathParamInfo>,
}

impl FuncItem {
    /// Generate a renamed version of this function with pub visibility
    pub fn with_renamed(&self, new_name: &Ident) -> TokenStream {
        let tokens: Vec<TokenTree> = self.tokens.clone().into_iter().collect();
        let mut result = Vec::new();
        let mut i = 0;
        let mut found_fn = false;
        let mut added_pub = false;

        while i < tokens.len() {
            if let TokenTree::Ident(ident) = &tokens[i] {
                // Check if we should add pub before async or fn
                if !added_pub && (*ident == "async" || *ident == "fn") {
                    // Look back to see if pub already exists
                    let has_pub = if i > 0 {
                        if let TokenTree::Ident(prev) = &tokens[i - 1] {
                            *prev == "pub"
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !has_pub {
                        let pub_ident = Ident::new("pub", ident.span());
                        result.push(TokenTree::Ident(pub_ident));
                    }
                    added_pub = true;
                }

                // Handle function name replacement
                if *ident == "fn" && !found_fn {
                    // Found 'fn', add it
                    result.push(tokens[i].clone());
                    i += 1;
                    found_fn = true;
                    // Skip the old name and add the new name
                    if i < tokens.len() {
                        if let TokenTree::Ident(_) = &tokens[i] {
                            result.push(TokenTree::Ident(new_name.clone()));
                            i += 1;
                            continue;
                        }
                    }
                    continue;
                }
            }
            result.push(tokens[i].clone());
            i += 1;
        }

        result.into_iter().collect()
    }
}

impl ToTokens for FuncItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

pub struct DocLine {
    pub text: String,
    pub span: Span,
}
