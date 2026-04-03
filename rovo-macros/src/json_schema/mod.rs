// Vendored from schemars_derive 0.9.0 (MIT License)
// https://github.com/GREsau/schemars
//
// Modified: default crate path changed from `schemars` to `::rovo::schemars`
// so that `#[derive(JsonSchema)]` works without a direct `schemars` dependency.

#[allow(unused_imports)]
use quote::{format_ident, quote, quote_spanned};
#[allow(unused_imports)]
use syn::{parse_quote, Token};

mod ast;
mod attr;
mod idents;
mod schema_exprs;

use ast::*;
use idents::GENERATOR;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

pub fn derive_json_schema(mut input: syn::DeriveInput, repr: bool) -> syn::Result<TokenStream> {
    attr::process_serde_attrs(&mut input)?;

    let mut cont = Container::from_ast(&input)?;
    add_trait_bounds(&mut cont);

    // --- ROVO MODIFICATION ---
    // Default to `::rovo::schemars` when no explicit `#[schemars(crate = "...")]` is set.
    // Original schemars_derive leaves this as None (meaning bare `schemars` must be in scope).
    let crate_alias = Some(cont.attrs.crate_name.as_ref().map_or_else(
        || {
            quote! {
                use ::rovo::schemars as schemars;
            }
        },
        |path| {
            quote_spanned! {path.span()=>
                use #path as schemars;
            }
        },
    ));
    // --- END ROVO MODIFICATION ---

    let type_name = &cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    if let Some(transparent_field) = cont.transparent_field() {
        let (ty, type_def) = schema_exprs::type_for_field_schema(transparent_field);
        return Ok(quote! {
            const _: () = {
                #crate_alias
                #type_def

                #[automatically_derived]
                impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                    fn inline_schema() -> bool {
                        <#ty as schemars::JsonSchema>::inline_schema()
                    }

                    fn schema_name() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                        <#ty as schemars::JsonSchema>::schema_name()
                    }

                    fn schema_id() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                        <#ty as schemars::JsonSchema>::schema_id()
                    }

                    fn json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                        <#ty as schemars::JsonSchema>::json_schema(#GENERATOR)
                    }

                    fn _schemars_private_non_optional_json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                        <#ty as schemars::JsonSchema>::_schemars_private_non_optional_json_schema(#GENERATOR)
                    }

                    fn _schemars_private_is_option() -> bool {
                        <#ty as schemars::JsonSchema>::_schemars_private_is_option()
                    }
                };
            };
        });
    }

    let mut schema_base_name = cont.serde_attrs.name().deserialize_name().to_string();

    if !cont.attrs.is_renamed {
        if let Some(path) = cont.serde_attrs.remote() {
            if let Some(segment) = path.segments.last() {
                schema_base_name = segment.ident.to_string();
            }
        }
    }

    let type_params: Vec<_> = cont.generics.type_params().map(|ty| &ty.ident).collect();
    let const_params: Vec<_> = cont.generics.const_params().map(|c| &c.ident).collect();
    let params: Vec<_> = type_params.iter().chain(const_params.iter()).collect();

    let (schema_name, schema_id) = if params.is_empty()
        || (cont.attrs.is_renamed && !schema_base_name.contains('{'))
    {
        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Borrowed(#schema_base_name)
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Borrowed(::core::concat!(
                    ::core::module_path!(),
                    "::",
                    #schema_base_name
                ))
            },
        )
    } else if cont.attrs.is_renamed {
        let mut schema_name_fmt = schema_base_name;
        for tp in &params {
            schema_name_fmt.push_str(&format!("{{{}:.0}}", tp));
        }
        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        #schema_name_fmt
                        #(,#type_params=#type_params::schema_name())*
                        #(,#const_params=schemars::_private::alloc::string::ToString::to_string(&#const_params))*)
                )
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        ::core::concat!(
                            ::core::module_path!(),
                            "::",
                            #schema_name_fmt
                        )
                        #(,#type_params=#type_params::schema_id())*
                        #(,#const_params=#const_params)*
                    )
                )
            },
        )
    } else {
        let mut schema_name_fmt = schema_base_name;
        schema_name_fmt.push_str("_for_{}");
        schema_name_fmt.push_str(&"_and_{}".repeat(params.len() - 1));
        (
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(#schema_name_fmt #(,#type_params::schema_name())* #(,#const_params)*)
                )
            },
            quote! {
                schemars::_private::alloc::borrow::Cow::Owned(
                    schemars::_private::alloc::format!(
                        ::core::concat!(
                            ::core::module_path!(),
                            "::",
                            #schema_name_fmt
                        )
                        #(,#type_params::schema_id())*
                        #(,#const_params)*
                    )
                )
            },
        )
    };

    let schema_expr = if repr {
        schema_exprs::expr_for_repr(&cont)?
    } else {
        schema_exprs::expr_for_container(&cont)
    };

    let inline = cont.attrs.inline;

    Ok(quote! {
        const _: () = {
            #crate_alias

            #[automatically_derived]
            #[allow(unused_braces)]
            impl #impl_generics schemars::JsonSchema for #type_name #ty_generics #where_clause {
                fn schema_name() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                    #schema_name
                }

                fn schema_id() -> schemars::_private::alloc::borrow::Cow<'static, str> {
                    #schema_id
                }

                fn json_schema(#GENERATOR: &mut schemars::SchemaGenerator) -> schemars::Schema {
                    #schema_expr
                }

                fn inline_schema() -> bool {
                    #inline
                }
            };
        };
    })
}

fn add_trait_bounds(cont: &mut Container) {
    if let Some(bounds) = cont.serde_attrs.ser_bound() {
        let where_clause = cont.generics.make_where_clause();
        where_clause.predicates.extend(bounds.iter().cloned());
    } else {
        for param in &mut cont.generics.params {
            if let syn::GenericParam::Type(ref mut type_param) = *param {
                type_param.bounds.push(parse_quote!(schemars::JsonSchema));
            }
        }
    }
}
