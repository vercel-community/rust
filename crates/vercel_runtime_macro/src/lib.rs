use glob::glob;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use syn::parse_macro_input;
use syn::AttributeArgs;
use vercel_runtime_router::{Route, Router};

#[proc_macro_attribute]
pub fn bundled_api(args: TokenStream, stream: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let mut args_map: HashMap<String, String> = HashMap::new();

    args.iter().for_each(|arg| {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
            path,
            lit: syn::Lit::Str(lit_str),
            ..
        })) = arg
        {
            if let Some(key) = path.get_ident() {
                args_map.insert(key.to_string(), lit_str.value());
            }
        }
    });

    let prefix = args_map
        .get("path")
        .map(|p| {
            if p.ends_with('/') {
                p.to_owned()
            } else {
                format!("{}/", p)
            }
        })
        .and_then(|p| {
            if fs::metadata(format!("{}api/main.rs", p)).is_ok() {
                Some(p)
            } else {
                // there is a `path` specified, but it doesn't appear to be in a cargo workspace,
                // so default to acting as if it is not in a cargo workspace.
                // This protects us from both
                // - using `vercel` to deploy via build container, but only uploading part of a
                // workspace
                // - path being specified incorrectly
                None
            }
        })
        .unwrap_or("".to_string());

    let glob_pattern = format!("{}api/**/*.rs", prefix);
    let main_path = PathBuf::from(format!("{}api/main.rs", prefix));

    let input: syn::ItemFn = syn::parse(stream).unwrap();

    let raw_routes = glob(&glob_pattern)
        .expect("a valid glob pattern")
        .filter_map(|e| e.ok())
        .filter(|raw_path| raw_path != &main_path)
        .map(|raw_path| raw_path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    let raw_routes = raw_routes
        .iter()
        .map(|f| f.strip_prefix(&prefix).unwrap())
        .collect::<Vec<_>>();

    let router = Router::from(raw_routes);

    let router_path_tokens = router.routes.iter().map(|r| {
        let Route { module_file, .. } = r;

        quote! {
            #module_file,
        }
    });

    let mod_statements = router.routes.iter().map(|r| {
        let Route {
            module_name,
            module_file,
            ..
        } = r;

        let module_file = format!("../{}", module_file);
        quote! {
            #[path = #module_file]
            mod #module_name;
        }
    });

    let matches = router.routes.iter().map(|r| {
        let Route {
            module_name,
            module_file,
            ..
        } = r;
        quote! {
            #module_file => {
                return #module_name::handler(req).await;
            }
        }
    });

    let syn::ItemFn {
        attrs, vis, sig, ..
    } = input;

    quote! {
        use vercel_runtime::{Route, Router};

        #(#mod_statements)*

        #(#attrs)* #vis #sig {
            let raw_routes = vec![#(#router_path_tokens)*];
            let router = Router::from(raw_routes);

            let request_uri = req.uri().path().to_string();
            let request_uri = request_uri.strip_prefix('/').unwrap_or(&request_uri);

            match router.call(&request_uri) {
                Some(route) => {
                    match route.module_file.as_str() {
                        #(#matches)*
                        _ => unreachable!()
                    }
                }
                None => unreachable!(),
            }
        }
    }
    .into()
}
