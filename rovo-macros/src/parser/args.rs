use proc_macro2::{Span, TokenStream, TokenTree};

use super::tokens::follows_joint_dash;
use super::ParseError;

/// Arguments passed to the attribute itself, as in `#[rovo(state = AppState)]`
#[derive(Debug, Default)]
pub struct RovoArgs {
    /// Explicit router state type, used instead of the one inferred from a `State<T>` parameter
    pub state_type: Option<TokenStream>,
}

/// Parse the arguments of `#[rovo(...)]`
pub fn parse_rovo_args(attr: TokenStream) -> Result<RovoArgs, ParseError> {
    let mut args = RovoArgs::default();
    for arg in split_args(attr) {
        parse_arg(arg, &mut args)?;
    }
    Ok(args)
}

/// Split the arguments on top-level commas, allowing a trailing comma.
/// Commas inside `<...>` belong to a type, as in `state = HashMap<K, V>`.
fn split_args(attr: TokenStream) -> Vec<Vec<TokenTree>> {
    let trees: Vec<TokenTree> = attr.into_iter().collect();
    let mut args = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for (i, tt) in trees.iter().enumerate() {
        if let TokenTree::Punct(p) = tt {
            match p.as_char() {
                '<' => depth += 1,
                '>' if !follows_joint_dash(&trees, i) => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    args.push(std::mem::take(&mut current));
                    continue;
                }
                _ => {}
            }
        }
        current.push(tt.clone());
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Parse a single `key = value` argument into `args`
fn parse_arg(arg: Vec<TokenTree>, args: &mut RovoArgs) -> Result<(), ParseError> {
    let mut tokens = arg.into_iter();

    let first = tokens.next();
    let Some(TokenTree::Ident(key)) = &first else {
        let span = first.as_ref().map_or_else(Span::call_site, TokenTree::span);
        return Err(ParseError::with_span("expected `state = Type`", span));
    };
    if key != "state" {
        return Err(ParseError::with_span(
            format!("unknown #[rovo] argument `{key}`\nnote: the only supported argument is `state = Type`"),
            key.span(),
        ));
    }
    if args.state_type.is_some() {
        return Err(ParseError::with_span(
            "duplicate `state` argument",
            key.span(),
        ));
    }

    match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
        other => {
            let span = other.as_ref().map_or_else(|| key.span(), TokenTree::span);
            return Err(ParseError::with_span("expected `=` after `state`", span));
        }
    }

    let state_type: TokenStream = tokens.collect();
    if state_type.is_empty() {
        return Err(ParseError::with_span(
            "expected a type after `state =`",
            key.span(),
        ));
    }
    if let Err(e) = syn::parse2::<syn::Type>(state_type.clone()) {
        let span = state_type
            .into_iter()
            .next()
            .map_or_else(Span::call_site, |t| t.span());
        return Err(ParseError::with_span(
            format!("invalid state type: {e}"),
            span,
        ));
    }

    args.state_type = Some(state_type);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn empty_args() {
        let args = parse_rovo_args(TokenStream::new()).unwrap();
        assert!(args.state_type.is_none());
    }

    #[test]
    fn parses_state_type() {
        let args = parse_rovo_args(quote! { state = AppState }).unwrap();
        assert_eq!(args.state_type.unwrap().to_string(), "AppState");
    }

    #[test]
    fn parses_generic_state_type() {
        let args = parse_rovo_args(quote! { state = std::sync::Arc<AppState> }).unwrap();
        assert_eq!(
            args.state_type.unwrap().to_string(),
            "std :: sync :: Arc < AppState >"
        );
    }

    #[test]
    fn rejects_unknown_argument() {
        let err = parse_rovo_args(quote! { stat = AppState }).unwrap_err();
        assert!(err.to_string().contains("unknown #[rovo] argument `stat`"));
    }

    #[test]
    fn rejects_non_identifier_argument() {
        let err = parse_rovo_args(quote! { "state" = AppState }).unwrap_err();
        assert!(err.to_string().contains("expected `state = Type`"));
    }

    #[test]
    fn rejects_state_without_value() {
        let err = parse_rovo_args(quote! { state }).unwrap_err();
        assert!(err.to_string().contains("expected `=`"));
    }

    #[test]
    fn rejects_missing_equals() {
        let err = parse_rovo_args(quote! { state AppState }).unwrap_err();
        assert!(err.to_string().contains("expected `=`"));
    }

    #[test]
    fn rejects_missing_type() {
        let err = parse_rovo_args(quote! { state = }).unwrap_err();
        assert!(err.to_string().contains("expected a type"));
    }

    #[test]
    fn rejects_invalid_type() {
        let err = parse_rovo_args(quote! { state = 123 }).unwrap_err();
        assert!(err.to_string().contains("invalid state type"));
    }

    #[test]
    fn allows_trailing_comma() {
        let args = parse_rovo_args(quote! { state = AppState, }).unwrap();
        assert_eq!(args.state_type.unwrap().to_string(), "AppState");
    }

    #[test]
    fn keeps_commas_inside_generic_state_type() {
        let args =
            parse_rovo_args(quote! { state = HashMap<String, Box<dyn Fn() -> u32>>, }).unwrap();
        assert_eq!(
            args.state_type.unwrap().to_string(),
            "HashMap < String , Box < dyn Fn () -> u32 >>"
        );
    }

    #[test]
    fn rejects_unknown_argument_after_state() {
        let err = parse_rovo_args(quote! { state = AppState, extra }).unwrap_err();
        assert!(err.to_string().contains("unknown #[rovo] argument `extra`"));
    }

    #[test]
    fn rejects_duplicate_state() {
        let err = parse_rovo_args(quote! { state = A, state = B }).unwrap_err();
        assert!(err.to_string().contains("duplicate `state` argument"));
    }

    #[test]
    fn rejects_empty_argument() {
        let err = parse_rovo_args(quote! { , state = A }).unwrap_err();
        assert!(err.to_string().contains("expected `state = Type`"));
    }
}
