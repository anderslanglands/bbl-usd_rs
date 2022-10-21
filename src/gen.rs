use anyhow::Result;
use bbl::*;
use std::path::{Path, PathBuf};

pub fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    let cmake_prefix_path = PathBuf::from(env!("CMAKE_PREFIX_PATH"));

    let namespace_internal = "pxrInternal_v0_22__pxrReserved__";
    let namespace_external = "pxr";

    let mut allow_list = vec![r"^pxr$"];
    let mut override_list = vec![];
    let mut binding_includes = vec![];

    // each of these function calls populates the allow list and includes list and returns a closure we'll call after
    // the AST is extracted to do any modifications needed
    let binding_fns = vec![
        bind_vtvalue(&mut allow_list, &mut binding_includes),
        bind_sdf_path(&mut allow_list, &mut binding_includes),
        bind_usd_property(&mut allow_list, &mut binding_includes),
        bind_usd_prim(&mut allow_list, &mut binding_includes, &mut override_list),
    ];

    // replace the external namespace name with the internal one in the regexes so we can write "pxr..." instead of the
    // ridiculous versioned one pixar use.
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
        // These get translated directly to find_packages() calls in CMake
        find_packages: &["pxr REQUIRED"],
        // These get stuck directly into the target_link_libraries() call for the shim
        link_libraries: &["vt"],
        // We can limit our extraction to a single namespace in the target library. This is usually a good idea to
        // avoid doing extra work (bbl-extract will extract everything it finds, even if it's never used, and the less
        // c++ it has to exract, the less likely it is to choke on constructs we haven't implemented yet)
        // This will be replaced at some point soon with the AllowList mechanism
        limit_to_namespace: Some(namespace_internal),
        // AllowList is just a list of qualified name regexes for things that we want to extract. See the binding functions
        // for examples. We might want to augment this with separate allow/block-lists per decl kind (e.g. classes,
        // methods etc)
        allow_list: AllowList::new(allow_list),
        // OverrideList is a list of pairs of regexes and callbacks that allow to supply a manually generated AST for a
        // type. See create_tfweakptr() below for an example. Ideally we wouldn't need this as everything could be
        // extracted, but we need a stopgap for when there's particularly painful constructs like TfWeakPtr's use of
        // CRTP.
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

    // debug-printing the AST dumps everything for inspection
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

    // Note we have separate regexes for UsdProperty and UsdProperty::*. This is to avoid binding any functions that
    // begin with UsdProperty, as there tends to be a lot of these that are templated which we want to ignore anyway.
    //
    // As a general rule, while babble is being developed, a good strategy is to try binding a type like UsdProperty is
    // done, below. If the shim compiles, then all good you can carry on to the next one. If it fails, then figure out
    // what construct is causing it to fail and make a test case reproducing it in either bbl-extact, bbl-translate,
    // bbl-write or some combination of those, then fix the issue.
    allow_list.extend_from_slice(&[
        r"^pxr::UsdProperty$",
        r"^pxr::UsdProperty::.*$",
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
    // we need to add all these includes here as prim.h contains fwd decls for all this stuff so we need the actual
    // includes for the shim.
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

    // We manually bind TfWeakPtr because it uses CRTP which we haven't figured out a way to extract cleanly yet
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

/// This function takes a class id, a list of signiatures and a list of types, and for each signature and type combination
/// generates a template specialization for that method.
/// We can use this to quickly specialize e.g. VtValue for all the different types we want
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

    // first create the template class which will contain the actual methods
    let c_tmpl = c.specialized_template().unwrap();
    let usr_tmpl = create_tfweakptr_tmpl(c_tmpl, ast, tu, already_visited)?;

    let name = c.display_name();

    let namespaces = get_namespaces_for_decl(c, tu, ast, already_visited)?;

    // Get the first template argument for the decl we're extracting
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

    // create a new specialization of our manually generate template class with the template argument type we've extracted
    let cts =
        ClassTemplateSpecialization::new(usr_tmpl, c.usr(), &name, template_arguments, namespaces);

    // and insert it into the AST and add the specialization to the template class. The latter step just silences a
    // warning about it being ignored currently, but later we'll need this info in order to generate nice names for the
    // specialized classes by associating them with typedefs
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

    // construct the full namespace path for the methods including the class template
    let namespaces = get_namespaces_for_decl(c_tmpl, tu, ast, already_visited)?;
    let mut method_namespaces = namespaces.clone();
    method_namespaces.push(c_tmpl.usr());

    // manually create the AST for the methods we care about
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

    // and finally create the ClassDecl itself and add it to the AST
    let cd = ClassDecl::new(
        c_tmpl.usr(),
        "TfWeakPtr".to_string(),
        Vec::new(),
        methods,
        namespaces,
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
