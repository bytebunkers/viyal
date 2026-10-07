mod fuzz;

#[cfg(test)]
mod tests {
    use bytecode::compiler::BytecodeCompiler;
    use interpreter::Interpreter;
    use parser::Parser;
    use stdlib::register_stdlib;
    use typechecker::TypeChecker;
    use vm::vm::VM;

    // A test runner that runs both the interpreter and VM on a given program,
    // and returns (InterpreterOutput, VMOutput) or errors
    fn run_conformance(
        source: &str,
        expect_fail: bool,
    ) -> (Result<String, String>, Result<String, String>) {
        // Parse and Typecheck
        let mut parser = Parser::new(source);
        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(errors) => {
                if expect_fail {
                    return (Err("parse_error".into()), Err("parse_error".into()));
                }
                panic!(
                    "Parse error: {}",
                    errors
                        .first()
                        .map_or("Unknown".to_string(), |e| e.message.clone())
                );
            }
        };

        let mut tc = TypeChecker::new();
        if let Err(e) = tc.check_program(&program) {
            if expect_fail {
                return (Err("type_error".into()), Err("type_error".into()));
            }
            panic!("Type error: {}", e.message);
        }

        // 1. Interpreter execution
        let mut interpreter_output = String::new();
        let interp_res = {
            let mut interpreter = Interpreter::new(&mut interpreter_output);
            let res = interpreter.interpret(&program);
            if res.is_ok() {
                Ok(interpreter_output)
            } else {
                Err(format!("{:?}", res))
            }
        };

        // 2. VM execution
        let mut vm_output = String::new();
        let vm_res = {
            let compiler = BytecodeCompiler::new();
            let chunk = compiler.compile(&program).expect("Compile error");
            let mut vm = VM::new(chunk);
            register_stdlib(&mut vm);

            let res = vm.run(&mut vm_output);
            if matches!(res, vm::vm::InterpretResult::Ok) {
                Ok(vm_output)
            } else {
                Err(format!("{:?}", res))
            }
        };

        (interp_res, vm_res)
    }

    macro_rules! conformance_test {
        ($name:ident, $code:expr) => {
            #[test]
            fn $name() {
                let (interp, vm_out) = run_conformance($code, false);
                // If both succeeded, their outputs must match exactly.
                if let (Ok(i), Ok(v)) = (&interp, &vm_out) {
                    assert_eq!(i, v, "Interpreter and VM outputs differ");
                }

                // We expect VM to support the test since it's the golden backend.
                assert!(
                    vm_out.is_ok(),
                    "VM failed on a valid program: VM={:?}",
                    vm_out
                );
            }
        };
        ($name:ident, $code:expr, fail) => {
            #[test]
            fn $name() {
                let (interp, vm_out) = run_conformance($code, true);
                assert!(vm_out.is_err(), "VM succeeded when it was expected to fail");
            }
        };
    }

    conformance_test!(
        test_arithmetic,
        r#"
        class Main {
            void run() {
                var a = 10;
                var b = 20;
                print(a + b);
                print(b - a);
            }
        }
    "#
    );

    conformance_test!(
        test_string,
        r#"
        class Main {
            void run() {
                var s = "hello";
                print(s);
            }
        }
    "#
    );

    conformance_test!(
        test_functions,
        r#"
        class Main {
            int add(int a, int b) {
                return a + b;
            }
            void run() {
                var m = new Main();
                print(m.add(5, 7));
            }
        }
    "#
    );

    conformance_test!(
        test_control_flow,
        r#"
        class Main {
            void run() {
                mut a = 0;
                while (a < 5) {
                    print(a);
                    a = a + 1;
                }
                if (a == 5) {
                    print("done");
                } else {
                    print("not done");
                }
            }
        }
    "#
    );
    conformance_test!(
        test_arrays,
        r#"
        class Main {
            void run() {
                var arr = [10, 20, 30];
                print(arr[0]);
                print(arr[2]);
                mut a = [1, 2];
                a[1] = 5;
                print(a[1]);
            }
        }
    "#
    );

    conformance_test!(
        test_maps,
        r#"
        class Main {
            void run() {
                var m = {"a": 1, "b": 2};
                print(m["a"]);
                mut m2 = {"k": 10};
                m2["k"] = 20;
                print(m2["k"]);
            }
        }
    "#
    );

    conformance_test!(
        test_classes_and_fields,
        r#"
        class Main {
            int val;
            
            void run() {
                this.val = 42;
                print(this.val);
            }
        }
    "#
    );

    conformance_test!(
        test_error_handling,
        r#"
        class Main {
            int! get_err() {
                return error("test error");
            }
            int! get_ok() {
                return 100;
            }
            
            void run() {
                var a = this.get_err();
                var fallback = a or {
                    print("caught error");
                    return;
                };
                print("should not reach here");
            }
        }
    "#
    );
}
