use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemTrait, LitStr, TraitItem};
use proc_macro2::Span;

struct ControllerPath {
    path: LitStr,
}

impl Parse for ControllerPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(ControllerPath {
                path: LitStr::new("/", Span::call_site()),
            })
        } else {
            let path: LitStr = input.parse()?;
            if !path.value().starts_with('/') {
                return Err(syn::Error::new(
                    path.span(),
                    "Controller path must start with '/'",
                ));
            }
            Ok(ControllerPath { path })
        }
    }
}

struct GetMappingPath {
    path: Option<LitStr>,
}

impl Parse for GetMappingPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(GetMappingPath { path: None })
        } else {
            let path: LitStr = input.parse()?;
            Ok(GetMappingPath { path: Some(path) })
        }
    }
}

fn clean_path(path: &str) -> String {
    path.trim_end_matches('/').to_string()
}

#[proc_macro_attribute]
pub fn Controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ctrl_path = match syn::parse::<ControllerPath>(attr) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };

    let base_path = clean_path(&ctrl_path.path.value());
    let input = parse_macro_input!(item as ItemTrait);
    let trait_name = &input.ident;

    let route_implementations = input
        .items
        .iter()
        .filter_map(|item| {
            if let TraitItem::Fn(method) = item {
                let method_name = &method.sig.ident;
                let get_mapping = method.attrs.iter().find(|attr| {
                    attr.path().is_ident("GetMapping")
                });

                match get_mapping {
                    Some(mapping) => {
                        let path = match mapping.parse_args::<GetMappingPath>() {
                            Ok(GetMappingPath { path: Some(p) }) => {
                                let p_str = p.value();
                                if p_str.starts_with('/') {
                                    clean_path(&p_str)
                                } else {
                                    format!("/{}", clean_path(&p_str))
                                }
                            },
                            Ok(GetMappingPath { path: None }) => "".to_string(),
                            Err(_) => "".to_string(),
                        };

                        let full_path = if path.is_empty() {
                            base_path.clone()
                        } else {
                            format!("{}{}", base_path, path)
                        };

                        Some(quote! {
                            {
                                router.add_route(#full_path.to_string(), || {
                                    <() as #trait_name>::#method_name();
                                });
                            }
                        })
                    }
                    None => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #input

        pub struct Router {
            routes: std::collections::HashMap<String, Box<dyn Fn() + Send + Sync>>,
        }

        impl Router {
            pub fn new() -> Self {
                Self {
                    routes: std::collections::HashMap::new(),
                }
            }

            pub fn add_route<F>(&mut self, path: String, handler: F)
            where
                F: Fn() + Send + Sync + 'static,
            {
                if let Some(_) = self.routes.insert(path.clone(), Box::new(handler)) {
                    eprintln!("Warning: Route {} was overwritten", path);
                }
            }

            pub fn handle_request(&self, path: &str) -> Result<(), String> {
                let clean_path = path.trim_end_matches('/');
                match self.routes.get(clean_path) {
                    Some(handler) => {
                        handler();
                        Ok(())
                    }
                    None => {
                        Err(format!("404 Not Found: {}", path))
                    }
                }
            }

            pub fn list_routes(&self) -> Vec<String> {
                self.routes.keys().cloned().collect()
            }
        }

        pub fn setup_router() -> Router {
            let mut router = Router::new();
            #(#route_implementations)*
            router
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn GetMapping(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}