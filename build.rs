use std::{env, fs, path::Path};

#[cfg(not(feature = "embed"))]
fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("web.rs");
    fs::write(out_path, "::axum::Router::new()").unwrap();
}

#[cfg(feature = "embed")]
fn main() {
    use quote::quote;
    use std::process::Command;
    use walkdir::WalkDir;

    println!("cargo:rerun-if-changed=./web/package.json");
    println!("cargo:rerun-if-changed=./web/tsconfig.json");
    println!("cargo:rerun-if-changed=./web/yarn.lock");
    println!("cargo:rerun-if-changed=./web/src");
    println!("cargo:rerun-if-changed=./web/public");

    let current_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let web_dir = Path::new(&current_dir).join("web");
    let build_dir = web_dir.join("build");

    let build = Command::new("yarn")
        .arg("build")
        .current_dir(&web_dir)
        .output()
        .unwrap();
    if !build.status.success() {
        let stderr = String::from_utf8(build.stderr).unwrap_or_default();
        panic!("{}", stderr);
    }

    let routes = WalkDir::new(&build_dir)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(e) if e.file_type().is_file() => Some(route(e, &build_dir)),
            _ => None,
        });

    let routes = quote! {
        ::axum::Router::new()
            #(#routes)*
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("web.rs");

    fs::write(out_path, routes.to_string()).unwrap();
}

#[cfg(feature = "embed")]
fn route(entry: walkdir::DirEntry, base: &Path) -> proc_macro2::TokenStream {
    use itertools::Itertools;
    use quote::quote;
    use std::ffi::OsStr;

    let path = entry.path();

    let mime = mime_guess::from_path(path).first().map(|mime| {
        let mime = mime.as_ref();
        quote! {
            ::axum::response::Headers([(
                ::axum::http::header::HeaderName::from_static("content-type"),
                ::axum::http::header::HeaderValue::from_static(#mime),
            )]),
        }
    });

    let contents_path = path.to_string_lossy();
    let contents = quote!(include_bytes!(#contents_path).as_ref());

    let mut path = path.strip_prefix(base).unwrap();
    if path.file_name().map_or(false, |f| f == "index.html") {
        path = path.parent().unwrap()
    }
    let path = path.iter().map(OsStr::to_string_lossy).join("/");

    quote! {
        .route(concat!("/", #path), ::axum::handler::get(|| async { (#mime #contents) }))
        .boxed()
    }
}
