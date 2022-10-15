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

    let mut allow_list = vec![r"^pxr$"];
    let mut override_list = vec![];
    let mut binding_includes = vec![];

    let mut binding_fns = Vec::new();

    binding_fns.push(bind_vtvalue(&mut allow_list, &mut binding_includes));
    binding_fns.push(bind_sdf_path(&mut allow_list, &mut binding_includes));
    binding_fns.push(bind_usd_property(&mut allow_list, &mut binding_includes));
    binding_fns.push(bind_usd_prim(
        &mut allow_list,
        &mut binding_includes,
        &mut override_list,
    ));

    let allow_list: Vec<String> = allow_list
        .iter()
        .map(|s| s.replace(namespace_external, namespace_internal))
        .collect();

    let override_list: Vec<(String, Box<ClassExtractionFn>)> = override_list
        .into_iter()
        .map(|(s, c)| (s.replace(namespace_external, namespace_internal), c))
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
        overrides: OverrideList::new(override_list),
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

fn bind_usd_property(
    allow_list: &mut Vec<&str>,
    includes: &mut Vec<&str>,
) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/usd/property.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::UsdProperty$",
        r"^pxr::UsdProperty::.*$",
        // r"^pxr::UsdProperty::UsdProperty\(.*\)$",
        // r"^pxr::UsdProperty::GetPropertyStack\(.*\)$",
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
        r"^pxr::SdfHandle<T>::operator->.*$",
        r"^pxr::SdfPropertySpec.*Handle.*$",
    ]);

    Box::new(|ast: &mut AST| Ok(()))
}

fn bind_sdf_path(
    allow_list: &mut Vec<&str>,
    includes: &mut Vec<&str>,
) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/sdf/path.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::SdfAncestorsRange.*$",
        r"^pxr::SdfPath$",
        r"^pxr::SdfPath::.*$",
    ]);

    Box::new(|ast: &mut AST| Ok(()))
}

fn bind_usd_prim(
    allow_list: &mut Vec<&str>,
    includes: &mut Vec<&str>,
    overrides: &mut Vec<(&str, Box<ClassExtractionFn>)>,
) -> Box<dyn Fn(&mut AST) -> Result<()>> {
    includes.push("#include <pxr/usd/usd/property.h>");
    includes.push("#include <pxr/usd/usd/prim.h>");
    includes.push("#include <pxr/usd/usd/attribute.h>");
    includes.push("#include <pxr/usd/usd/relationship.h>");
    includes.push("#include <pxr/usd/usd/variantSets.h>");
    includes.push("#include <pxr/usd/usd/inherits.h>");
    includes.push("#include <pxr/usd/usd/specializes.h>");
    includes.push("#include <pxr/usd/usd/references.h>");
    includes.push("#include <pxr/usd/usd/payloads.h>");

    allow_list.extend_from_slice(&[
        r"^pxr::UsdPrimSiblingRange.*$",
        r"^pxr::UsdPrimSubtreeRange.*$",
        r"^pxr::UsdPrimTypeInfo$",
        r"^pxr::UsdPrimTypeInfo::.*$",
        r"^pxr::UsdPrim$",
        r"^pxr::UsdPrim::.*$",
    ]);

    overrides.push((
        r"^pxr::TfWeakPtr<.*>$",
        Box::new(
            |cursor, ast, tu, already_visited, allow_list, override_list| {
                create_tfweakptr(cursor, ast, already_visited, tu, allow_list, override_list)
            },
        ),
    ));

    Box::new(|ast: &mut AST| Ok(()))
}

fn bind_vtvalue(
    allow_list: &mut Vec<&str>,
    includes: &mut Vec<&str>,
) -> Box<dyn Fn(&mut AST) -> Result<()>> {
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
pub fn create_tfweakptr(
    c: Cursor,
    ast: &mut AST,
    already_visited: &mut Vec<USR>,
    tu: &TranslationUnit,
    allow_list: &AllowList,
    class_overrides: &OverrideList,
) -> Result<USR, ExtractError> {
    if already_visited.contains(&c.usr()) {
        return Ok(c.usr());
    } else {
        already_visited.push(c.usr());
    }

    println!("Got TfWeakPtr {c:?}");

    let c_tmpl = c.specialized_template().unwrap();
    let usr_tmpl = create_tfweakptr_tmpl(c_tmpl, ast, tu, already_visited)?;

    let name = c.display_name();

    let namespaces = get_namespaces_for_decl(c, tu, ast, already_visited)?;
    let ty = c.template_argument_type(0)?;
    let template_arguments = vec![TemplateArgument::Type(extract_type(
        ty,
        &[],
        already_visited,
        ast,
        tu,
        allow_list,
        class_overrides,
    )?)];

    let cts =
        ClassTemplateSpecialization::new(usr_tmpl, c.usr(), &name, template_arguments, namespaces);

    let id = ast.insert_class_template_specialization(cts);
    let cd = ast.get_class_mut(usr_tmpl).unwrap();
    cd.add_specialization(id);

    Ok(c.usr())
}

fn create_tfweakptr_tmpl(
    c_tmpl: Cursor,
    ast: &mut AST,
    tu: &TranslationUnit,
    already_visited: &mut Vec<USR>,
) -> Result<USR, ExtractError> {
    if already_visited.contains(&c_tmpl.usr()) {
        return Ok(c_tmpl.usr());
    } else {
        already_visited.push(c_tmpl.usr());
    }

    // get the namespaces for std::vector<> as we might not have found them already
    let namespaces = get_namespaces_for_decl(c_tmpl, tu, ast, already_visited)?;

    let u_std = ast
        .find_namespace("std")
        .map(|id| ast.namespaces()[id].usr())
        .unwrap();

    let method_namespaces = vec![u_std, c_tmpl.usr()];

    let methods = vec![
        Method::new(
            USR::new("BBL:tfweakptr_ctor_default"),
            "TfWeakPtr".to_string(),
            MethodKind::Constructor,
            QualType::void(),
            Vec::new(),
            Some("ctor".to_string()),
            method_namespaces.clone(),
            Vec::new(),
            ExceptionSpecificationKind::None,
            Const(false),
            Static(false),
            Virtual(false),
            PureVirtual(false),
            Deleted(false),
        ),
        Method::new(
            USR::new("BBL:tfweakptr_operator->_const"),
            "operator->".to_string(),
            MethodKind::Method,
            QualType::pointer("const T *", QualType::template_parameter("T", "T", true)),
            vec![],
            Some("get".to_string()),
            method_namespaces.clone(),
            Vec::new(),
            ExceptionSpecificationKind::None,
            Const(true),
            Static(false),
            Virtual(false),
            PureVirtual(false),
            Deleted(false),
        ),
        Method::new(
            USR::new("BBL:tfweakptr_operator->_mut"),
            "operator->".to_string(),
            MethodKind::Method,
            QualType::pointer("T *", QualType::template_parameter("T", "T", false)),
            vec![],
            Some("get_mut".to_string()),
            method_namespaces,
            Vec::new(),
            ExceptionSpecificationKind::None,
            Const(false),
            Static(false),
            Virtual(false),
            PureVirtual(false),
            Deleted(false),
        ),
    ];

    let cd = ClassDecl::new(
        c_tmpl.usr(),
        "TfWeakPtr".to_string(),
        Vec::new(),
        methods,
        vec![u_std],
        vec![TemplateParameterDecl::typ("T", 0)],
        false,
        NeedsImplicit {
            dtor: true,
            ..Default::default()
        },
    );

    ast.insert_class(cd);

    Ok(c_tmpl.usr())
}
