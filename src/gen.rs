use anyhow::Result;
use bbl::*;
use std::path::{Path, PathBuf};

pub fn main() -> Result<()> {        
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .init();

    // Point CMake to our library. In a real project we would probably expect this to be done by setting CMAKE_PREFIX_PATH
    // directly in the environment, or perhaps with a config file
    let cmake_prefix_path = PathBuf::from(env!("CMAKE_PREFIX_PATH"));

    let namespace_internal = "pxrInternal_v0_22__pxrReserved__";
    let namespace_external = "pxr";

    let allow_list = [
        r"^pxr$",
        r"^pxr::VtValue$",
        r"^pxr::VtValue::VtValue\(.*\)$",
        r"^pxr::VtValue::~VtValue\(\)$",
        r"^pxr::VtValue::GetArraySize\(\)$",
        r"^pxr::VtValue::GetTypeName\(\)$",
    ];

    let allow_list: Vec<String> = allow_list
        .iter()
        .map(|s| s.replace(namespace_external, namespace_internal))
        .collect();

    let options = BindOptions {
        // We use CMake to configure the compilation and linking of our shim library, so need to point CMAKE_PREFIX_PATH
        // to find the target cpp library as well as provide the library name for find_package() and the actual targets
        // to link against
        cmake_prefix_path: Some(cmake_prefix_path),
        find_packages: &["pxr REQUIRED"],
        link_libraries: &["vt"],
        // We can limit our extraction to a single namespace in the target library. This is usually a good idea to
        // avoid doing extra work (bbl-extract will extract everything it finds, even if it's never used, and the less
        // c++ it has to exract, the less likely it is to choke on constructs we haven't implemented yet)
        limit_to_namespace: Some(namespace_internal),
        allow_list: AllowList::new(allow_list),
        ..Default::default()
    };

    // parse the given cpp snippet, which just includes the header of the library we want to bind, giving us an AST
    let mut ast = parse("#include <pxr/base/vt/value.h>\n", &options)?;

    // Now that we have the AST, we can manipulate it, for example to give an external name to the versioned internal
    // namespace, "Test_1_0". We could also ignore and rename methods, try and override bind kinds of classes etc.
    let ns = ast.find_namespace(namespace_internal)?;
    ast.rename_namespace(ns, namespace_external);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let ffi_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("ffi.rs")
        .to_string_lossy()
        .to_string();

    // Now bind the AST, which will write, compile and link a shim library, and create the rust ffi binding
    // we also copy the generated ffi.rs into the source tree. This isn't hygienic but using the "correct" method of
    // include!'ing it into the source stops rust-analyzer from working on it, which is worse.
    bind("usd", &out_dir, Some(&ffi_path), &ast, &options)?;

    Ok(())
}