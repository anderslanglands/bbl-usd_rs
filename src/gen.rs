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

    let mut allow_list = vec![
        r"^pxr$",
    ];
    let mut binding_includes = vec![];

    let mut binding_fns = Vec::new();

    // binding_fns.push(bind_vtvalue(&mut allow_list, &mut binding_includes));
    binding_fns.push(bind_sdf_path(&mut allow_list, &mut binding_includes));
    // binding_fns.push(bind_usd_property(&mut allow_list, &mut binding_includes));
    // binding_fns.push(bind_usd_prim(&mut allow_list, &mut binding_includes));

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
        // compile_definitions: &["-Wno-deprecated"],
        ..Default::default()
    };

    // parse the given cpp snippet, which just includes the header of the library we want to bind, giving us an AST
    let include_str = binding_includes.join("\n");
    let mut ast = parse(&include_str, &options)?;

    // Now that we have the AST, we can manipulate it, for example to give an external name to the versioned internal
    // namespace, "Test_1_0". We could also ignore and rename methods, try and override bind kinds of classes etc.
    let ns = ast.find_namespace(namespace_internal)?;
    ast.rename_namespace(ns, namespace_external);

    println!("{ast:?}");

    // now call all the binding functions to manipulate the ast
    for fun in binding_fns {
        fun(&mut ast)?;
    }

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

fn bind_usd_property(allow_list: &mut Vec<&str>, includes: &mut Vec<&str>) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/usd/property.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::UsdProperty$",
        r"^pxr::UsdProperty::UsdProperty\(.*\)$",
        r"^pxr::SdfHandle<T>::operator->.*$",
        r"^pxr::SdfPropertySpec.*Handle.*$",
        r"^pxr::UsdProperty::GetPropertyStack\(.*\)$",        // SdfPropertySpecHandleVector
        // r"^pxr::UsdProperty::GetBaseName\(.*\)$",
        // r"^pxr::UsdProperty::GetNamespace\(.*\)$",
        // r"^pxr::UsdProperty::SplitName\(.*\)$",                
        // r"^pxr::UsdProperty::GetDisplayGroup\(.*\)$",
        // r"^pxr::UsdProperty::SetDisplayGroup\(.*\)$",
        // r"^pxr::UsdProperty::ClearDisplayGroup\(.*\)$",
        // r"^pxr::UsdProperty::HasAuthoredDisplayGroup\(.*\)$",
        // r"^pxr::UsdProperty::GetNestedDisplayGroups\(.*\)$",   
        // r"^pxr::UsdProperty::SetNestedDisplayGroups\(.*\)$",
        // r"^pxr::UsdProperty::GetDisplayName\(.*\)$",
        // r"^pxr::UsdProperty::SetDisplayName\(.*\)$",
        // r"^pxr::UsdProperty::ClearDisplayName\(.*\)$",
        // r"^pxr::UsdProperty::HasAuthoredDisplayName\(.*\)$",
        // r"^pxr::UsdProperty::IsCustom\(.*\)$",
        // r"^pxr::UsdProperty::SetCustom\(.*\)$",
        // r"^pxr::UsdProperty::IsDefined\(.*\)$",
        // r"^pxr::UsdProperty::IsAuthored\(.*\)$",
        // r"^pxr::UsdProperty::IsAuthoredAt\(.*\)$",           // UsdEditTarget
        // r"^pxr::UsdProperty::FlattenTo\(.*\)$",
    ]);

    Box::new(|ast: &mut AST| {
        Ok(())
    })
}


fn bind_sdf_path(allow_list: &mut Vec<&str>, includes: &mut Vec<&str>) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/sdf/path.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::SdfAncestorsRange.*$",
        r"^pxr::SdfPath$",
        r"^pxr::SdfPath::.*$",
    ]);

    Box::new(|ast: &mut AST| {
        Ok(())
    })
}

fn bind_usd_prim(allow_list: &mut Vec<&str>, includes: &mut Vec<&str>) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/usd/property.h>");
    includes.push("#include <pxr/usd/usd/prim.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::UsdPrim$",
        r"^pxr::UsdPrimSiblingRange.*$",
        r"^pxr::UsdPrimSubtreeRange.*$",
        r"^pxr::UsdPrimTypeInfo$",
        r"^pxr::UsdPrimTypeInfo::.*$",
        r"^pxr::UsdPrim::UsdPrim\(\)*$",
        r"^pxr::UsdPrim::GetPrimTypeInfo\(\)*$",
        r"^pxr::UsdPrim::GetPrimDefinition\(\)*$",
        r"^pxr::UsdPrim::GetSpecifier\(\)*$",
        r"^pxr::UsdPrim::GetPrimStack\(\)*$",
        r"^pxr::UsdPrim::SetSpecifier\(.*\)*$",
        r"^pxr::UsdPrim::GetTypeName\(\)*$",
        r"^pxr::UsdPrim::SetTypeName\(.*\)*$",
        r"^pxr::UsdPrim::ClearActive\(\)*$",
        r"^pxr::UsdPrim::HasAuthoredActive\(\)*$",
        r"^pxr::UsdPrim::IsLoaded\(\)*$",
        r"^pxr::UsdPrim::IsModel\(\)*$",
        r"^pxr::UsdPrim::IsGroup\(\)*$",
        r"^pxr::UsdPrim::IsAbstract\(\)*$",
        r"^pxr::UsdPrim::IsDefined\(\)*$",
        r"^pxr::UsdPrim::HasDefiningSpecifier\(\)*$",
        r"^pxr::UsdPrim::GetAppliedSchemas\(\)*$",                   
        r"^pxr::UsdPrim::GetPropertyNames\(.*\)*$",                  
        r"^pxr::UsdPrim::GetAuthoredPropertyNames\(.*\)*$",          
        r"^pxr::UsdPrim::GetProperties\(.*\)*$",                     
        r"^pxr::UsdPrim::GetAuthoredProperties\(.*\)*$",             
        r"^pxr::UsdPrim::GetPropertiesInNamespace\(.*\)*$",          
        r"^pxr::UsdPrim::GetAuthoredPropertiesInNamespace\(.*\)*$",  
        r"^pxr::UsdPrim::GetPropertyOrder\(.*\)*$",                  
        r"^pxr::UsdPrim::SetPropertyOrder\(.*\)*$",                  
        r"^pxr::UsdPrim::ClearPropertyOrder\(.*\)*$",
        r"^pxr::UsdPrim::RemoveProperty\(.*\)*$",
        r"^pxr::UsdPrim::GetProperty\(.*\)*$",
        r"^pxr::UsdPrim::HasProperty\(.*\)*$",
        r"^pxr::UsdPrim::IsA\(const .*\)*$",
        r"^pxr::UsdPrim::HasAPI\(const TfType &, const TfToken &\)*$",
        r"^pxr::UsdPrim::CanApplyAPI\(const TfType &,.*\)*$",
        r"^pxr::UsdPrim::ApplyAPI\(const TfType &,.*\)*$",
        r"^pxr::UsdPrim::RemoveAPI\(const TfType &,.*\)*$",
        r"^pxr::UsdPrim::AddAppliedSchema\(.*\)*$",
        r"^pxr::UsdPrim::RemoveAppliedSchema\(.*\)*$",
        r"^pxr::UsdPrim::GetChild\(.*\)*$",
        /* All the rest of the Prim Children sectino relying on boost::iterator_adapter */
        r"^pxr::UsdPrim::GetChildren\(.*\)*$",
        r"^pxr::UsdPrim::GetAllChildren\(.*\)*$",
        r"^pxr::UsdPrim::GetFilteredChildren\(.*\)*$",
        r"^pxr::UsdPrim::GetChildrenNames\(.*\)*$",
        r"^pxr::UsdPrim::GetAllChildrenNames\(.*\)*$",
        r"^pxr::UsdPrim::GetFilteredChildrenNames\(.*\)*$",
        r"^pxr::UsdPrim::GetDescendents\(.*\)*$",
        r"^pxr::UsdPrim::GetAllDescendents\(.*\)*$",
        r"^pxr::UsdPrim::GetFilteredDescendents\(.*\)*$",
        r"^pxr::UsdPrim::GetChildrenReorder\(.*\)*$",
        r"^pxr::UsdPrim::SetChildrenReorder\(.*\)*$",
        r"^pxr::UsdPrim::ClearChildrenReorder\(.*\)*$",
        r"^pxr::UsdPrim::GetParent\(.*\)*$",
        r"^pxr::UsdPrim::GetNextSibling\(.*\)*$",
        r"^pxr::UsdPrim::GetFilteredNextSibling\(.*\)*$",
        r"^pxr::UsdPrim::IsPseudoRoot\(.*\)*$",
        // r"^pxr::UsdPrim::GetPrimAtPath\(.*\)*$",                   SdfPath 
        // r"^pxr::UsdPrim::GetObjectAtPath\(.*\)*$",
        // r"^pxr::UsdPrim::GetPropertyAtPath\(.*\)*$",
        // r"^pxr::UsdPrim::GetAttributeAtPath\(.*\)*$",
        // r"^pxr::UsdPrim::GetRelationshipAtPath\(.*\)*$",
        // r"^pxr::UsdPrim::GetVariantSets\(.*\)*$",                // UsdVariantSet
        // r"^pxr::UsdPrim::GetVariantSet\(.*\)*$",
        // r"^pxr::UsdPrim::HasVariantSets\(.*\)*$",                  
    ]);

    Box::new(|ast: &mut AST| {
        Ok(())
    })
}

fn bind_vtvalue(allow_list: &mut Vec<&str>, includes: &mut Vec<&str>) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/base/vt/value.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::VtValue$",
        r"^pxr::VtValue::VtValue\(.*\)$",
        r"^pxr::VtValue::~VtValue\(\)$",
        r"^pxr::VtValue::GetArraySize\(\)$",
        r"^pxr::VtValue::GetTypeName\(\)$",
        r"^pxr::VtValue::IsHolding\(\)$",
        r"^pxr::VtValue::IsArrayValued\(\)$",
        r"^pxr::VtValue::Remove\(\)$",
        r"^pxr::VtValue::GetTypeName\(\)$",
        r"^pxr::VtValue::Get\(\)$",
        r"^pxr::VtValue::GetWithDefault\(.*\)$",
        r"^pxr::VtValue::GetTypeid\(.*\)$",
        r"^pxr::VtValue::GetElementTypeid\(.*\)$",
        r"^pxr::VtValue::IsEmpty\(.*\)$",
        r"^pxr::VtValue::Cast\(.*\)$",
        r"^pxr::VtValue::CastToTypeOf\(.*\)$",
        r"^pxr::VtValue::CanCastToTypeid\(.*\)$",
        r"^pxr::VtValue::CanHash\(.*\)$",
        r"^pxr::VtValue::GetHash\(.*\)$",
        r"^pxr::VtValue::operator==\(.*\)$",
        r"^::std::type_info::operator==.*$",
        r"^::std::type_info::__name$",             // libstdc++
        r"^::std::type_info::__undecorated_name$", // libcxx
        r"^::std::type_info::__decorated_name$",
        r"^::std::type_info::_UndecoratedName$", // msvc
        r"^::std::type_info::_DecoratedName$",
    ]);

    Box::new(|ast: &mut AST| {
        let id_vtvalue = ast.find_class("VtValue")?;

        specialize_methods(
            ast,
            id_vtvalue,
            &[
                "VtValue(const T &)",
                "IsHolding",
                "Remove",
                "Get(",
                "GetWithDefault(",
                "Cast(",
            ],
            &[(QualType::float(), "float")],
        )?;

        // we need to force type_info to ValueType
        let type_info_id = ast.find_class("type_info")?;
        ast.class_set_bind_kind(type_info_id, ClassBindKind::ValueType)?;

        Ok(())
    })
}

pub fn specialize_methods(
    ast: &mut AST,
    class_id: ClassId,
    signatures: &[&str],
    types: &[(QualType, &str)],
) -> Result<()> {
    for sig in signatures {
        let (method_ids, _) = ast.find_methods(class_id, sig)?;

        for method_id in method_ids {
            for ty in types {
                ast.specialize_method(
                    class_id,
                    method_id,
                    &format!("{}_{}", ast.method_name(class_id, method_id), ty.1),
                    vec![TemplateArgument::Type(ty.0.clone())],
                )?;
            }
        }
    }

    Ok(())
}
