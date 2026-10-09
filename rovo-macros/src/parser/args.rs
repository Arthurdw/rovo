use proc_macro2::{Span, TokenStream, TokenTree};

use super::ParseError;

/// Arguments passed to the attribute itself, as in `#[rovo(state = AppState)]`
#[derive(Debug, Default)]
pub struct RovoArgs {
    /// Explicit router state type, used instead of the one inferred from a `State<T>` parameter
    pub state_type: Option<TokenStream>,
}

/// Parse the arguments of `#[rovo(...)]`
pub fn parse_rovo_args(attr: TokenStream) -> Result<RovoArgs, ParseError> {
    let mut tokens = attr.into_iter();
    let mut args = RovoArgs::default();

    let Some(first) = tokens.next() else {
        return Ok(args);
    };

    let TokenTree::Ident(key) = &first else {
        return Err(ParseError::with_span(
            "expected `state = Type`",
            first.span(),
        ));
    };
    if key != "state" {
        return Err(ParseError::with_span(
            format!("unknown #[rovo] argument `{key}`\nnote: the only supported argument is `state = Type`"),
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
    Ok(args)
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
        let err = parse_rovo_args(quote! { state = AppState, extra }).unwrap_err();
        assert!(err.to_string().contains("invalid state type"));
    }
}
