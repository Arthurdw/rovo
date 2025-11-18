use proc_macro::TokenStream;
use quote::quote;

mod parser;
use parser::parse_rovo_function;

/// Macro that generates OpenAPI documentation from doc comments.
///
/// # Example
///
/// ```ignore
/// /// Get a single Todo item.
/// ///
/// /// Retrieve a Todo item by its ID.
/// ///
/// /// @response 200 Json<TodoItem> A single Todo item.
/// /// @response 404 () Todo was not found.
/// #[rovo]
/// async fn get_todo(State(app): State<AppState>, Path(todo): Path<SelectTodo>) -> impl IntoApiResponse {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn rovo(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();

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
                doc_info.responses.iter().map(|resp| {
                    let code = resp.status_code;
                    let response_type = &resp.response_type;
                    let desc = &resp.description;

                    // Check if there's an explicit example for this status code
                    if let Some(example) = doc_info.examples.iter().find(|e| e.status_code == code) {
                        let example_code = &example.example_code;
                        quote! {
                            .response_with::<#code, #response_type, _>(|res| {
                                res.description(#desc)
                                    .example(#example_code)
                            })
                        }
                    } else {
                        // No explicit example, just add the description
                        quote! {
                            .response_with::<#code, #response_type, _>(|res| {
                                res.description(#desc)
                            })
                        }
                    }
                }).collect()
            };

            // Generate an internal implementation name
            let impl_name = quote::format_ident!("__{}_impl", func_name);

            // Get the renamed function tokens
            let impl_func = func_item.with_renamed(&impl_name);

            // Create a const with an uppercase version of the handler name
            let const_name = quote::format_ident!("{}", func_name.to_string().to_uppercase());

            // Determine the state type for the trait implementation
            let state_type = func_item.state_type.as_ref().map(|st| quote! { #st }).unwrap_or(quote! { () });

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
                    pub fn __docs(op: aide::transform::TransformOperation) -> aide::transform::TransformOperation {
                        op.summary(#title)
                            .description(#description)
                            #(#response_code_setters)*
                    }
                }

                // Implement the IntoApiMethodRouter trait
                impl ::rovo::IntoApiMethodRouter<#state_type> for #func_name {
                    fn into_get_route(self) -> aide::axum::routing::ApiMethodRouter<#state_type> {
                        aide::axum::routing::get_with(#impl_name, Self::__docs)
                    }

                    fn into_post_route(self) -> aide::axum::routing::ApiMethodRouter<#state_type> {
                        aide::axum::routing::post_with(#impl_name, Self::__docs)
                    }

                    fn into_patch_route(self) -> aide::axum::routing::ApiMethodRouter<#state_type> {
                        aide::axum::routing::patch_with(#impl_name, Self::__docs)
                    }

                    fn into_delete_route(self) -> aide::axum::routing::ApiMethodRouter<#state_type> {
                        aide::axum::routing::delete_with(#impl_name, Self::__docs)
                    }

                    fn into_put_route(self) -> aide::axum::routing::ApiMethodRouter<#state_type> {
                        aide::axum::routing::put_with(#impl_name, Self::__docs)
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
            quote! {
                compile_error!(#err_msg);
            }
            .into()
        }
    }
}
