#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]
// Allow some overly strict pedantic lints
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]

//! Procedural macros for the Rovo `OpenAPI` documentation framework.
//!
//! This crate provides the `#[rovo]` attribute macro that processes doc comments
//! with special annotations to generate `OpenAPI` documentation automatically.

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};

mod parser;
mod utils;

use parser::{parse_rovo_function, PathParamDoc, PathParamInfo};

/// Known primitive types that map to `OpenAPI` types
const PRIMITIVE_TYPES: &[&str] = &[
    "String", "u64", "u32", "u16", "u8", "i64", "i32", "i16", "i8", "bool", "Uuid",
];

/// Check if a type is a known primitive
fn is_primitive_type(type_name: &str) -> bool {
    PRIMITIVE_TYPES.contains(&type_name.trim())
}

/// Check if a tuple contains only primitives
fn is_primitive_tuple(type_str: &str) -> bool {
    let inner = type_str
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')');
    inner.split(',').map(str::trim).all(is_primitive_type)
}

/// Extract individual types from a tuple type string like "(Uuid, u32)"
fn extract_tuple_types(type_str: &str) -> Vec<String> {
    let inner = type_str
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')');
    inner
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Generate path parameter setters for primitive types
fn generate_path_param_setters(
    path_info: Option<&PathParamInfo>,
    path_docs: &[PathParamDoc],
) -> Vec<proc_macro2::TokenStream> {
    let Some(info) = path_info else {
        return vec![];
    };

    // If it's a struct pattern, let aide handle it via JsonSchema
    if info.is_struct_pattern {
        return vec![];
    }

    // Check if the type is primitive (single or tuple)
    let is_primitive = if info.inner_type.starts_with('(') {
        is_primitive_tuple(&info.inner_type)
    } else {
        is_primitive_type(&info.inner_type)
    };

    if !is_primitive {
        return vec![];
    }

    // Extract types for each binding
    let types: Vec<String> = if info.inner_type.starts_with('(') {
        extract_tuple_types(&info.inner_type)
    } else {
        vec![info.inner_type.clone()]
    };

    // Generate a parameter setter for each binding
    info.bindings
        .iter()
        .zip(types.iter())
        .map(|(name, type_str)| {
            // Find the description from docs
            let description = path_docs
                .iter()
                .find(|doc| doc.name == *name)
                .map(|doc| doc.description.clone());

            let desc_setter = description.map_or_else(
                || quote! { description: None, },
                |desc| quote! { description: Some(#desc.to_string()), },
            );

            // Parse the type string to a TokenStream for use in generic context
            let type_tokens: proc_macro2::TokenStream = type_str.parse().unwrap_or_else(|_| {
                quote! { String }
            });

            quote! {
                .with(|mut op| {
                    op.inner_mut().parameters.push(
                        ::rovo::aide::openapi::ReferenceOr::Item(
                            ::rovo::aide::openapi::Parameter::Path {
                                parameter_data: ::rovo::aide::openapi::ParameterData {
                                    name: #name.to_string(),
                                    #desc_setter
                                    required: true,
                                    deprecated: None,
                                    format: ::rovo::aide::openapi::ParameterSchemaOrContent::Schema(
                                        ::rovo::aide::openapi::SchemaObject {
                                            json_schema: <#type_tokens as ::rovo::schemars::JsonSchema>::json_schema(
                                                &mut ::rovo::schemars::SchemaGenerator::default()
                                            ),
                                            example: None,
                                            external_docs: None,
                                        }
                                    ),
                                    example: None,
                                    examples: ::std::default::Default::default(),
                                    explode: None,
                                    extensions: ::std::default::Default::default(),
                                },
                                style: ::rovo::aide::openapi::PathStyle::Simple,
                            }
                        )
                    );
                    op
                })
            }
        })
        .collect()
}

/// Macro that generates `OpenAPI` documentation from doc comments.
///
/// This macro automatically generates `OpenAPI` documentation for your handlers
/// using doc comments with special annotations.
///
/// # Documentation Format
///
/// Use Rust-style doc comment sections and metadata annotations:
///
/// ## Sections
/// - `# Path Parameters` - Document path parameters for primitive types
/// - `# Responses` - Document response status codes
/// - `# Examples` - Provide example responses
/// - `# Metadata` - Add tags, security, and other metadata
///
/// ## Path Parameters
///
/// For primitive path parameters (`String`, `u64`, `Uuid`, `bool`, etc.), you can
/// document them directly without creating wrapper structs:
///
/// ```rust,ignore
/// /// # Path Parameters
/// ///
/// /// user_id: The user's unique identifier
/// /// index: Zero-based item index
/// ```
///
/// The parameter names are inferred from the variable bindings in your function
/// signature (e.g., `Path(user_id)` creates a parameter named `user_id`).
///
/// For complex types, continue using structs with `#[derive(JsonSchema)]`.
///
/// ## Metadata Annotations
/// - `@tag <tag_name>` - Add a tag for grouping operations (can be used multiple times)
/// - `@security <scheme_name>` - Add security requirements (can be used multiple times)
/// - `@id <operation_id>` - Set a custom operation ID (defaults to function name)
/// - `@hidden` - Hide this operation from documentation
/// - `@rovo-ignore` - Stop processing annotations after this point
///
/// Additionally, the Rust `#[deprecated]` attribute is automatically detected
/// and will mark the operation as deprecated in the `OpenAPI` spec.
///
/// # Examples
///
/// ## Primitive Path Parameter
///
/// ```rust,ignore
/// /// Get user by ID.
/// ///
/// /// # Path Parameters
/// ///
/// /// id: The user's numeric identifier
/// ///
/// /// # Responses
/// ///
/// /// 200: Json<User> - User found
/// /// 404: () - User not found
/// #[rovo]
/// async fn get_user(Path(id): Path<u64>) -> impl IntoApiResponse {
///     // ...
/// }
/// ```
///
/// ## Tuple Path Parameters
///
/// ```rust,ignore
/// /// Get item in collection.
/// ///
/// /// # Path Parameters
/// ///
/// /// collection_id: The collection UUID
/// /// index: Item index within collection
/// ///
/// /// # Responses
/// ///
/// /// 200: Json<Item> - Item found
/// #[rovo]
/// async fn get_item(
///     Path((collection_id, index)): Path<(Uuid, u32)>
/// ) -> impl IntoApiResponse {
///     // ...
/// }
/// ```
///
/// ## Struct-based Path (for complex types)
///
/// ```rust,ignore
/// /// Get a single Todo item.
/// ///
/// /// Retrieve a Todo item by its ID from the database.
/// ///
/// /// # Responses
/// ///
/// /// 200: Json<TodoItem> - Successfully retrieved the todo item
/// /// 404: () - Todo item was not found
/// ///
/// /// # Examples
/// ///
/// /// 200: TodoItem::default()
/// ///
/// /// # Metadata
/// ///
/// /// @tag todos
/// #[rovo]
/// async fn get_todo(
///     State(app): State<AppState>,
///     Path(todo): Path<SelectTodo>  // SelectTodo implements JsonSchema
/// ) -> impl IntoApiResponse {
///     // ...
/// }
/// ```
///
/// ## Deprecated Endpoint
///
/// ```rust,ignore
/// /// This is a deprecated endpoint.
/// ///
/// /// # Metadata
/// ///
/// /// @tag admin
/// /// @security bearer_auth
/// #[deprecated]
/// #[rovo]
/// async fn old_handler() -> impl IntoApiResponse {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn rovo(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item;

    match parse_rovo_function(input.into()) {
        Ok((func_item, doc_info)) => {
            let func_name = &func_item.name;

            let title = doc_info.title.as_deref().unwrap_or("");
            let description = doc_info.description.as_deref().unwrap_or("");

            // Generate response setters if we have doc comments
            let response_code_setters = if doc_info.responses.is_empty() {
                // No responses specified - generate a minimal docs function
                vec![]
            } else {
                doc_info
                    .responses
                    .iter()
                    .map(|resp| {
                        let code = resp.status_code;
                        let response_type = &resp.response_type;
                        let desc = &resp.description;

                        // Check if there's an explicit example for this status code
                        doc_info
                            .examples
                            .iter()
                            .find(|e| e.status_code == code)
                            .map_or_else(
                                || {
                                    // No explicit example, just add the description
                                    quote! {
                                        .response_with::<#code, #response_type, _>(|res| {
                                            res.description(#desc)
                                        })
                                    }
                                },
                                |example| {
                                    let example_code = &example.example_code;
                                    quote! {
                                        .response_with::<#code, #response_type, _>(|res| {
                                            res.description(#desc)
                                                .example(#example_code)
                                        })
                                    }
                                },
                            )
                    })
                    .collect()
            };

            // Generate tag setters
            let tag_setters: Vec<_> = doc_info
                .tags
                .iter()
                .map(|tag| {
                    quote! { .tag(#tag) }
                })
                .collect();

            // Generate security requirement setters
            let security_setters: Vec<_> = doc_info
                .security_requirements
                .iter()
                .map(|scheme| {
                    quote! { .security_requirement(#scheme) }
                })
                .collect();

            // Generate operation ID setter
            let operation_id_setter = doc_info.operation_id.as_ref().map_or_else(
                || {
                    // Default to function name if no custom ID provided
                    let default_id = func_name.to_string();
                    quote! { .id(#default_id) }
                },
                |id| quote! { .id(#id) },
            );

            // Generate deprecated setter
            let deprecated_setter = if doc_info.deprecated {
                quote! { .with(|mut op| { op.inner_mut().deprecated = true; op }) }
            } else {
                quote! {}
            };

            // Generate hidden setter
            let hidden_setter = if doc_info.hidden {
                quote! { .hidden(true) }
            } else {
                quote! {}
            };

            // Generate path parameter setters for primitive types
            let path_param_setters =
                generate_path_param_setters(func_item.path_params.as_ref(), &doc_info.path_params);

            // Generate an internal implementation name
            let impl_name = quote::format_ident!("__{}_impl", func_name);

            // Get the renamed function tokens
            let impl_func = func_item.with_renamed(&impl_name);

            // Create a const with an uppercase version of the handler name
            let const_name = quote::format_ident!("{}", func_name.to_string().to_uppercase());

            // Determine the state type for the trait implementation
            let state_type = func_item
                .state_type
                .as_ref()
                .map_or_else(|| quote! { () }, |st| quote! { #st });

            let output = quote! {
                // Internal implementation with renamed function
                #[allow(non_snake_case, private_interfaces)]
                #impl_func

                // Create a zero-sized type that can be passed to routing functions
                #[allow(non_camel_case_types)]
                #[derive(Clone, Copy)]
                pub struct #func_name;

                impl #func_name {
                    #[doc(hidden)]
                    pub fn __docs(op: ::rovo::aide::transform::TransformOperation) -> ::rovo::aide::transform::TransformOperation {
                        op
                            #operation_id_setter
                            .summary(#title)
                            .description(#description)
                            #(#tag_setters)*
                            #deprecated_setter
                            #hidden_setter
                            #(#security_setters)*
                            #(#path_param_setters)*
                            #(#response_code_setters)*
                    }
                }

                // Implement the IntoApiMethodRouter trait
                impl ::rovo::IntoApiMethodRouter<#state_type> for #func_name {
                    fn into_get_route(self) -> ::rovo::aide::axum::routing::ApiMethodRouter<#state_type> {
                        ::rovo::aide::axum::routing::get_with(#impl_name, Self::__docs)
                    }

                    fn into_post_route(self) -> ::rovo::aide::axum::routing::ApiMethodRouter<#state_type> {
                        ::rovo::aide::axum::routing::post_with(#impl_name, Self::__docs)
                    }

                    fn into_patch_route(self) -> ::rovo::aide::axum::routing::ApiMethodRouter<#state_type> {
                        ::rovo::aide::axum::routing::patch_with(#impl_name, Self::__docs)
                    }

                    fn into_delete_route(self) -> ::rovo::aide::axum::routing::ApiMethodRouter<#state_type> {
                        ::rovo::aide::axum::routing::delete_with(#impl_name, Self::__docs)
                    }

                    fn into_put_route(self) -> ::rovo::aide::axum::routing::ApiMethodRouter<#state_type> {
                        ::rovo::aide::axum::routing::put_with(#impl_name, Self::__docs)
                    }
                }

                // Also create a CONST for explicit use
                #[allow(non_upper_case_globals)]
                pub const #const_name: #func_name = #func_name;
            };

            output.into()
        }
        Err(err) => {
            let err_msg = err.to_string();
            // Use the span from the error if available, otherwise use call_site
            let error_tokens = err.span().map_or_else(
                || {
                    quote! {
                        compile_error!(#err_msg);
                    }
                },
                |span| {
                    quote_spanned! {span=>
                        compile_error!(#err_msg);
                    }
                },
            );
            error_tokens.into()
        }
    }
}
