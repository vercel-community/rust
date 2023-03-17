use glob::glob;
use lazy_static::lazy_static;
use quote::format_ident;
use regex::Regex;
use std::cmp::Ordering;

mod utils {
    pub fn get_segments(p: &str) -> Vec<&str> {
        let stripped = p.strip_prefix('/').unwrap_or(p);
        stripped.split('/').collect::<Vec<&str>>()
    }
}

use utils::get_segments;

lazy_static! {
        // Dynamic Route - /api/[id]
        static ref DYNAMIC_ROUTE_REGEX: Regex = Regex::new(r"\[[^/\.]+\]").unwrap();
        // Catch-all Route - /api/[...slug]
        static ref DYNAMIC_CATCH_ALL_REGEX: Regex = Regex::new(r"\[\.{3}\S+\]").unwrap();
        // Optional catch-all Route - /api/[[...slug]]
        static ref DYNAMIC_OPTIONAL_CATCH_ALL_REGEX: Regex = Regex::new(r"\[{2}\.{3}\S+\]{2}").unwrap();
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum RouteKind {
    Static,
    Dynamic,
    CatchAll,
    OptionalCatchAll,
}

#[derive(Debug)]
pub struct Route {
    pub kind: RouteKind,
    pub module_file: String,
    pub module_name: syn::Ident,
    pub path: String,
    pub segments: Option<Vec<String>>,
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.kind {
            // Sort by length in descending order
            RouteKind::Static => match other.kind {
                RouteKind::Static => other.path.len().cmp(&self.path.len()),
                _ => Ordering::Less,
            },
            // Sort by segment length in descending order
            RouteKind::Dynamic => match other.kind {
                RouteKind::Static => Ordering::Greater,
                RouteKind::Dynamic => match self.segments {
                    Some(ref s) => match other.segments {
                        Some(ref o) => o.len().cmp(&s.len()),
                        None => Ordering::Greater,
                    },
                    None => Ordering::Equal,
                },
                RouteKind::CatchAll | RouteKind::OptionalCatchAll => Ordering::Less,
            },
            // Sort by segment length in descending order
            RouteKind::CatchAll | RouteKind::OptionalCatchAll => match other.kind {
                RouteKind::Static => Ordering::Greater,
                RouteKind::Dynamic => Ordering::Greater,
                RouteKind::CatchAll | RouteKind::OptionalCatchAll => match self.segments {
                    Some(ref s) => match other.segments {
                        Some(ref o) => o.len().cmp(&s.len()),
                        None => Ordering::Greater,
                    },
                    None => Ordering::Equal,
                },
            },
        }
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Route {}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl From<&str> for Route {
    fn from(file_path: &str) -> Self {
        let file_path = file_path.to_string();
        let route = file_path.strip_suffix(".rs").unwrap_or(&file_path);

        let module_name = file_path.strip_prefix('/').unwrap_or(&file_path);
        let module_name = module_name.replace('/', "_");

        let module_name = module_name.replace('[', "_");
        let module_name = module_name.replace(']', "_");
        let module_name = module_name.replace("...", "___");

        let module_name = module_name.replace('-', "_");
        let module_name = module_name.strip_suffix(".rs").unwrap_or(&module_name);

        // TODO validation that [...slug] and [[...slug]] can only be in the last segment
        let get_route_kind = |r: &str| -> RouteKind {
            if DYNAMIC_ROUTE_REGEX.is_match(r) {
                match DYNAMIC_OPTIONAL_CATCH_ALL_REGEX.is_match(r) {
                    true => return RouteKind::OptionalCatchAll,
                    // false => return RouteKind::Dynamic,
                    false => match DYNAMIC_CATCH_ALL_REGEX.is_match(r) {
                        true => return RouteKind::CatchAll,
                        false => return RouteKind::Dynamic,
                    },
                }
            }

            if DYNAMIC_OPTIONAL_CATCH_ALL_REGEX.is_match(r) {
                return RouteKind::OptionalCatchAll;
            }

            if DYNAMIC_CATCH_ALL_REGEX.is_match(r) {
                return RouteKind::CatchAll;
            }
            RouteKind::Static
        };

        let route_kind = get_route_kind(route);

        let segments = match route_kind {
            RouteKind::Static => None,
            RouteKind::Dynamic => Some(get_segments(route)),
            RouteKind::CatchAll => Some(get_segments(route)),
            RouteKind::OptionalCatchAll => Some(get_segments(route)),
        };

        let segments = segments.map(|s| s.iter().map(|s| s.to_string()).collect::<Vec<_>>());

        Route {
            kind: route_kind,
            // module_file: format!("../{}", file_path),
            module_file: file_path.to_owned(),
            module_name: format_ident!("{}", module_name.to_owned()),
            path: route.to_owned(),
            segments,
        }
    }
}

pub struct Router {
    pub routes: Vec<Route>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new("api/**/*.rs")
    }
}

impl From<Vec<&str>> for Router {
    fn from(raw_paths: Vec<&str>) -> Self {
        let mut routes: Vec<Route> = raw_paths.into_iter().map(Route::from).collect();
        routes.sort();
        Router { routes }
    }
}

impl Router {
    pub fn new(file_pattern: &str) -> Self {
        let mut routes = glob(file_pattern)
            .expect("Failed to read glob pattern")
            .filter_map(|e| e.ok())
            .map(|raw_path| {
                let path = raw_path.to_str().unwrap();
                Route::from(path)
            })
            .collect::<Vec<_>>();

        routes.sort();
        Router { routes }
    }

    pub fn call(&self, req_path: &str) -> Option<&Route> {
        // Check if there is an optional catch all route
        if let Some(optional_catch_all) = self.routes.iter().find(|r| {
            let dynamic_optional_catch_all_exp = Regex::new(r"\[{2}\.{3}\S+\]{2}").unwrap();
            let optional_catchall_route =
                dynamic_optional_catch_all_exp.replace_all(r.path.as_str(), "");
            let optional_catchall_route = optional_catchall_route.trim_end_matches('/');

            r.kind == RouteKind::OptionalCatchAll && req_path == optional_catchall_route
        }) {
            return Some(optional_catch_all);
        };

        let result = self.routes.iter().find(|route| {
            match route.kind {
                RouteKind::Static => route.path == req_path,
                RouteKind::Dynamic => {
                    let path_segements = get_segments(req_path);
                    // Check if all segements are identical (ignoring wildcards)
                    match route.segments {
                        None => false,
                        Some(ref route_segments) => {
                            if route_segments.len() != path_segements.len() {
                                return false;
                            }

                            route_segments.iter().enumerate().all(|(i, rs)| {
                                (rs.contains('[') && rs.contains(']')) || rs == path_segements[i]
                            })
                        }
                    }
                }
                RouteKind::OptionalCatchAll => {
                    // todo extract logic
                    let optional_catchall_prefix =
                        DYNAMIC_OPTIONAL_CATCH_ALL_REGEX.replace_all(route.path.as_str(), "");
                    req_path.starts_with(optional_catchall_prefix.as_ref())
                }
                RouteKind::CatchAll => {
                    // todo extract logic
                    let catchall_prefix =
                        DYNAMIC_CATCH_ALL_REGEX.replace_all(route.path.as_str(), "");
                    req_path.starts_with(catchall_prefix.as_ref())
                }
            }
        });

        result
    }
}

#[cfg(test)]
mod tests {
    use super::Router;

    #[test]
    fn dynamic_routing() {
        let router = Router::from(vec![
            "api/posts.rs",
            "api/[id].rs",
            "api/posts/[id].rs",
            "api/[...id].rs",
            "api/nested/posts.rs",
            "api/nested/[id].rs",
            "api/nested/posts/[id].rs",
            "api/nested/[...id].rs",
            "api/optional/posts.rs",
            "api/optional/[id].rs",
            "api/optional/posts/[id].rs",
            "api/optional.rs",
            "api/optional/[[...id]].rs",
            "api/deep/nested/[id]/comments/[cid].rs",
            "api/other/[ab]/[cd]/ef.rs",
            "api/foo/[d]/bar/baz/[f].rs",
        ]);

        // Root
        insta::assert_debug_snapshot!(router.call("api/posts"));
        insta::assert_debug_snapshot!(router.call("api/[id]"));
        insta::assert_debug_snapshot!(router.call("api/posts/[id]"));
        insta::assert_debug_snapshot!(router.call("api"));
        insta::assert_debug_snapshot!(router.call("api/root/catch/all/route"));
        // Catch-all - Nested
        insta::assert_debug_snapshot!(router.call("api/nested/posts"));
        insta::assert_debug_snapshot!(router.call("api/nested/[id]"));
        insta::assert_debug_snapshot!(router.call("api/nested/posts/[id]"));
        insta::assert_debug_snapshot!(router.call("api/nested"));
        insta::assert_debug_snapshot!(router.call("api/nested/catch/all/route"));
        // Optional Catch-all - Nested
        insta::assert_debug_snapshot!(router.call("api/optional/posts"));
        insta::assert_debug_snapshot!(router.call("api/optional/[id]"));
        insta::assert_debug_snapshot!(router.call("api/optional/posts/[id]"));
        insta::assert_debug_snapshot!(router.call("api/optional"));
        insta::assert_debug_snapshot!(router.call("api/optional/catch/all/route"));
        // Dynamic Deep Nested
        insta::assert_debug_snapshot!(router.call("api/deep/nested/[id]/comments/[cid]"));
        insta::assert_debug_snapshot!(router.call("api/should/be/caught/by/root/catch/all"));
        insta::assert_debug_snapshot!(router.call("api/other/[ab]/[cd]/ef"));
        insta::assert_debug_snapshot!(router.call("api/foo/[d]/bar/baz/[f]"));
    }
}

#[cfg(test)]
mod route_tests {
    use super::{Route, RouteKind};

    #[test]
    fn it_creates_static_route() {
        let path = "api/handler";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::Static));
        assert_eq!(route.path, path);
        assert!(route.segments.is_none());
    }

    #[test]
    fn it_creates_dynamic_route() {
        let path = "api/[dyn]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::Dynamic));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(route.segments.unwrap(), vec!["api", "[dyn]"]);
    }

    #[test]
    fn it_creates_complex_dynamic_route() {
        let path = "api/[dyn]/handler/[dyn2]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::Dynamic));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(
            route.segments.unwrap(),
            vec!["api", "[dyn]", "handler", "[dyn2]"]
        );
    }

    #[test]
    fn it_creates_catch_all_route() {
        let path = "api/[...all]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::CatchAll));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(route.segments.unwrap(), vec!["api", "[...all]"]);
    }

    #[test]
    fn it_creates_complex_catch_all_route() {
        let path = "api/[dyn]/handler/[...all]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::CatchAll));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(
            route.segments.unwrap(),
            vec!["api", "[dyn]", "handler", "[...all]"]
        );
    }

    #[test]
    fn it_creates_optional_catch_all_route() {
        let path = "api/[[...all]]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::OptionalCatchAll));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(route.segments.unwrap(), vec!["api", "[[...all]]"]);
    }

    #[test]
    fn it_creates_complex_optional_catch_all_route() {
        let path = "api/[dyn]/handler/[[...all]]";
        let route = Route::from(path);
        assert!(matches!(route.kind, RouteKind::OptionalCatchAll));
        assert_eq!(route.path, path);
        assert!(route.segments.is_some());
        assert_eq!(
            route.segments.unwrap(),
            vec!["api", "[dyn]", "handler", "[[...all]]"]
        );
    }
}
