use ast::Decl;
use resolver::ModuleLinker;
use std::fs;
use std::path::PathBuf;

fn write_temp_files(test_name: &str, files: &[(&str, &str)]) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("viyal_resolver_tests");
    dir.push(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let mut main_path = None;
    for (name, content) in files {
        let mut path = dir.clone();
        path.push(name);
        fs::write(&path, content).unwrap();
        if *name == "main.vyl" {
            main_path = Some(path);
        }
    }
    main_path.unwrap()
}

#[test]
fn test_single_file_no_imports() {
    let source = "class Person { void greet() {} }\n";
    let main_path = write_temp_files("test_single_file_no_imports", &[("main.vyl", source)]);

    let mut linker = ModuleLinker::new();
    let program = match linker.link(&main_path) {
        Ok(p) => p,
        Err(e) => panic!("Link failed: {}", e.message),
    };

    assert_eq!(program.declarations.len(), 1);
}

#[test]
fn test_valid_imports_and_mangling() {
    let main_path = write_temp_files(
        "test_valid_imports_and_mangling",
        &[
            (
                "main.vyl",
                "\nimport { MathUtils as Utils } from \"math.vyl\";\nvoid test() { var utils = 1; }\n",
            ),
            ("math.vyl", "\nexport class MathUtils { void calc() {} }\n"),
        ],
    );

    let mut linker = ModuleLinker::new();
    let program = match linker.link(&main_path) {
        Ok(p) => p,
        Err(e) => panic!("Link failed: {}", e.message),
    };

    assert_eq!(program.declarations.len(), 2);

    let mut class_found = false;
    let mut fn_found = false;
    for decl in &program.declarations {
        match &decl.node {
            Decl::Class { name, .. } => {
                assert_eq!(name, "math::MathUtils");
                class_found = true;
            }
            Decl::Function(method, _) => {
                assert_eq!(method.name, "test");
                fn_found = true;
            }
            _ => {}
        }
    }
    assert!(class_found);
    assert!(fn_found);
}

#[test]
fn test_circular_imports() {
    let main_path = write_temp_files(
        "test_circular_imports",
        &[
            ("main.vyl", "import { A } from \"a.vyl\";\n"),
            ("a.vyl", "import { B } from \"b.vyl\";\nexport class A {}\n"),
            ("b.vyl", "import { A } from \"a.vyl\";\nexport class B {}\n"),
        ],
    );

    let mut linker = ModuleLinker::new();
    let result = linker.link(&main_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Circular import detected"));
}

#[test]
fn test_missing_file() {
    let main_path = write_temp_files(
        "test_missing_file",
        &[("main.vyl", "import { A } from \"does_not_exist.vyl\";\n")],
    );

    let mut linker = ModuleLinker::new();
    let result = linker.link(&main_path);

    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Failed to read file"));
}
