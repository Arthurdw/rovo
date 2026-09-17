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
    /// Generate a renamed version of this function.
    ///
    /// An existing visibility qualifier (`pub`, `pub(crate)`, `pub(super)`,
    /// `pub(in path)`) is preserved; functions without one are made `pub`.
    pub fn with_renamed(&self, new_name: &Ident) -> TokenStream {
        let tokens: Vec<TokenTree> = self.tokens.clone().into_iter().collect();
        let mut result = Vec::with_capacity(tokens.len() + 1);
        let mut i = 0;

        // Copy outer attributes (`#` followed by a bracketed group) verbatim
        while i < tokens.len() {
            match &tokens[i] {
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    result.push(tokens[i].clone());
                    i += 1;
                    if let Some(TokenTree::Group(_)) = tokens.get(i) {
                        result.push(tokens[i].clone());
                        i += 1;
                    }
                }
                _ => break,
            }
        }

        // Add `pub` unless the function already declares a visibility
        match tokens.get(i) {
            Some(TokenTree::Ident(ident)) if *ident == "pub" => {}
            Some(tt) => result.push(TokenTree::Ident(Ident::new("pub", tt.span()))),
            None => {}
        }

        let mut found_fn = false;
        while i < tokens.len() {
            if !found_fn {
                if let TokenTree::Ident(ident) = &tokens[i] {
                    if *ident == "fn" {
                        found_fn = true;
                        result.push(tokens[i].clone());
                        i += 1;
                        // Replace the old name with the new name
                        if let Some(TokenTree::Ident(_)) = tokens.get(i) {
                            result.push(TokenTree::Ident(new_name.clone()));
                            i += 1;
                        }
                        continue;
                    }
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
