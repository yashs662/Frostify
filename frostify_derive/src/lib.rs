use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, ImplItem, ItemImpl, parse_macro_input};

#[proc_macro_derive(EcsResource)]
pub fn derive_ecs_resource(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl EcsResource for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    // Convert back to token stream and return
    TokenStream::from(expanded)
}

#[proc_macro_derive(EcsComponent)]
pub fn derive_ecs_component(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl EcsComponent for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    // Convert back to token stream and return
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn time_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the token stream into an impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Extract the struct name from the self_ty field
    let struct_name = &input.self_ty;

    // Find the run method and wrap it with timing code
    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            if method.sig.ident == "run" {
                let original_body = &method.block;

                // Replace the method body with a timed version using the struct name
                method.block = syn::parse2(quote! {
                    {
                        let start = std::time::Instant::now();
                        let result = #original_body;
                        let elapsed = start.elapsed();
                        log::trace!("{} executed in: {:?}", stringify!(#struct_name), elapsed);
                        result
                    }
                })
                .unwrap();
            }
        }
    }

    // Generate the resulting token stream
    quote! { #input }.into()
}

#[proc_macro_derive(EntityCategories, attributes(modal, player))]
pub fn derive_entity_categories(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        panic!("EntityCategories can only be derived for enums");
    };

    let mut modal_variants = Vec::new();
    let mut player_variants = Vec::new();

    // Parse variants and their attributes
    for variant in &data_enum.variants {
        for attr in &variant.attrs {
            if attr.path().is_ident("modal") {
                modal_variants.push(&variant.ident);
            } else if attr.path().is_ident("player") {
                player_variants.push(&variant.ident);
            }
        }
    }

    // Generate trait implementations
    let modal_impl = if !modal_variants.is_empty() {
        quote! {
            impl ModalEntity for #name {
                fn is_modal(&self) -> bool {
                    matches!(self, #(#name::#modal_variants)|*)
                }
            }
        }
    } else {
        quote! {
            impl ModalEntity for #name {
                fn is_modal(&self) -> bool {
                    false
                }
            }
        }
    };

    let player_impl = if !player_variants.is_empty() {
        quote! {
            impl PlayerEntity for #name {
                fn is_player_entity(&self) -> bool {
                    matches!(self, #(#name::#player_variants)|*)
                }
            }
        }
    } else {
        quote! {
            impl PlayerEntity for #name {
                fn is_player_entity(&self) -> bool {
                    false
                }
            }
        }
    };

    let expanded = quote! {
        #modal_impl
        #player_impl
    };

    TokenStream::from(expanded)
}

// proc macro to time any function
#[proc_macro_attribute]
pub fn time_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the token stream into a function
    let mut input = parse_macro_input!(item as syn::ItemFn);

    // Get the function name
    let func_name = &input.sig.ident;

    // Wrap the function body with timing code
    let original_body = &input.block;
    input.block = syn::parse2(quote! {
        {
            let start = std::time::Instant::now();
            let result = #original_body;
            let elapsed = start.elapsed();
            log::trace!("{} executed in: {:?}", stringify!(#func_name), elapsed);
            result
        }
    })
    .unwrap();

    // Generate the resulting token stream
    quote! { #input }.into()
}
