use std::env;
use std::fs;
use std::process;
use std::path::Path;
use std::sync::mpsc::channel;
use notify::{Watcher, RecursiveMode, EventKind};

use parser::Parser;
use formatter::format;
use analyzer::analyze;
use bytecode::compiler::BytecodeCompiler;
use typechecker::TypeChecker;
use vm::vm::VM;
use stdlib::register_stdlib;

fn print_usage() {
    println!("Viyal Language CLI");
    println!("Usage:");
    println!("  viyal new <name>         - Create a new Viyal project");
    println!("  viyal run [file.vy]      - Run a Viyal program or project");
    println!("  viyal debug <file.vy>    - Debug a Viyal program");
    println!("  viyal build <file.vy>    - Build a Viyal program to an executable (requires C compiler)");
    println!("  viyal format <file.vy>   - Format a Viyal program");
    println!("  viyal analyze <file.vy>  - Run static analysis on a Viyal program");
    println!("  viyal lsp                - Start the Language Server");
}
fn span_to_line_col(source: &str, span_start: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in source.chars().enumerate() {
        if i == span_start {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn execute_file(file_path: &str) -> bool {
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: Could not read file {}", file_path);
            return false;
        }
    };
    
    let mut parser = Parser::new(&source);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            let (line, col) = span_to_line_col(&source, e.span.start);
            eprintln!("Parse error at {}:{}:{} - {}", file_path, line, col, e.message);
            return false;
        }
    };
    
    let mut tc = TypeChecker::new();
    if let Err(e) = tc.check_program(&program) {
        let (line, col) = span_to_line_col(&source, e.span.start);
        eprintln!("Type error at {}:{}:{} - {}", file_path, line, col, e.message);
        return false;
    }

    let compiler = BytecodeCompiler::new();
    let chunk = match compiler.compile(&program) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Compile error: {}", e);
            return false;
        }
    };

    let mut vm = VM::new(chunk);
    register_stdlib(&mut vm);
    
    let mut output = String::new();
    match vm.run(&mut output) {
        vm::vm::InterpretResult::Ok => {
            print!("{}", output);
            true
        }
        vm::vm::InterpretResult::CompileError => {
            eprintln!("Runtime Compile Error");
            false
        }
        vm::vm::InterpretResult::RuntimeError(msg) => {
            print!("{}", output); // Flush output before crash
            eprintln!("Runtime Error: {}", msg);
            false
        }
        vm::vm::InterpretResult::Breakpoint => {
            true
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("Error: Missing project name for 'new'");
                process::exit(1);
            }
            let project_name = &args[2];
            match pub_tool::init_project(project_name) {
                Ok(_) => {
                    println!("Created new Viyal project `{}`", project_name);
                }
                Err(e) => {
                    eprintln!("Error creating project: {}", e);
                    process::exit(1);
                }
            }
        }
        "run" => {
            let mut watch_mode = false;
            let mut file_args = vec![];
            for arg in args.iter().skip(2) {
                if arg == "--watch" {
                    watch_mode = true;
                } else {
                    file_args.push(arg.clone());
                }
            }

            let file_path = if file_args.len() > 0 {
                file_args[0].clone()
            } else {
                // Try to find viyal.toml and run src/main.vy
                match pub_tool::find_project_root() {
                    Ok(root) => {
                        let manifest_path = root.join("viyal.toml");
                        let manifest = pub_tool::read_manifest(&manifest_path).unwrap_or_else(|e| {
                            eprintln!("Error reading viyal.toml: {}", e);
                            process::exit(1);
                        });
                        println!("Running project `{}` v{}", manifest.package.name, manifest.package.version);
                        let main_file = root.join("src").join("main.vy");
                        if !main_file.exists() {
                            eprintln!("Error: src/main.vy not found in project.");
                            process::exit(1);
                        }
                        main_file.to_string_lossy().to_string()
                    }
                    Err(e) => {
                        eprintln!("Error: Missing file path for 'run' and not in a Viyal project.");
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                }
            };

            execute_file(&file_path);

            if watch_mode {
                let watch_path = match pub_tool::find_project_root() {
                    Ok(root) => root,
                    Err(_) => {
                        // Just watch the parent directory of the script if not in a project
                        Path::new(&file_path).parent().unwrap_or(Path::new(".")).to_path_buf()
                    }
                };

                println!("Watching for changes in {}...", watch_path.display());

                let (tx, rx) = channel();
                let mut watcher = notify::recommended_watcher(tx).expect("Failed to create file watcher");
                
                watcher.watch(&watch_path, RecursiveMode::Recursive).expect("Failed to watch path");

                for res in rx {
                    match res {
                        Ok(event) => {
                            if matches!(event.kind, EventKind::Modify(_)) {
                                // Clear terminal for a fresh output
                                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                                println!("Change detected, reloading...");
                                execute_file(&file_path);
                                println!("Waiting for changes...");
                            }
                        }
                        Err(e) => println!("Watch error: {:?}", e),
                    }
                }
            }
        }
        "debug" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path for 'debug'");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {}", file_path);
                process::exit(1);
            });
            
            let mut parser = Parser::new(&source);
            let program = match parser.parse_program() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Parse error: {}", e.message);
                    process::exit(1);
                }
            };

            let compiler = BytecodeCompiler::new();
            let chunk = match compiler.compile(&program) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Compile error: {}", e);
                    process::exit(1);
                }
            };

            let mut dbg = debugger::Debugger::new(chunk);
            register_stdlib(dbg.vm_mut());
            dbg.start_repl();
        }
        "format" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path for 'format'");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {}", file_path);
                process::exit(1);
            });
            
            match format(&source) {
                Ok(formatted) => {
                    // For MVP, just print to stdout
                    println!("{}", formatted);
                }
                Err(e) => {
                    eprintln!("Format error: {}", e);
                    process::exit(1);
                }
            }
        }
        "analyze" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path for 'analyze'");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {}", file_path);
                process::exit(1);
            });
            
            match analyze(&source) {
                Ok(warnings) => {
                    if warnings.is_empty() {
                        println!("Analysis complete: No warnings.");
                    } else {
                        for warning in warnings {
                            println!("Warning: {}", warning.message);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Analyze error: {}", e);
                    process::exit(1);
                }
            }
        }
        "build" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path for 'build'");
                process::exit(1);
            }
            let file_path = &args[2];
            let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
                eprintln!("Error: Could not read file {}", file_path);
                process::exit(1);
            });
            
            let mut parser = Parser::new(&source);
            let program = match parser.parse_program() {
                Ok(p) => p,
                Err(e) => {
                    let (line, col) = span_to_line_col(&source, e.span.start);
                    eprintln!("Parse error at {}:{}:{} - {}", file_path, line, col, e.message);
                    process::exit(1);
                }
            };
            
            let mut tc = TypeChecker::new();
            if let Err(e) = tc.check_program(&program) {
                let (line, col) = span_to_line_col(&source, e.span.start);
                eprintln!("Type error at {}:{}:{} - {}", file_path, line, col, e.message);
                process::exit(1);
            }

            match codegen::generate_c(&program) {
                Ok(c_code) => {
                    let c_file = ".viyal_out.c";
                    fs::write(c_file, &c_code).expect("Failed to write temporary C file");
                    
                    println!("Compiling {} to native binary...", file_path);
                    
                    // Simple MVP compilation using gcc
                    // In a real scenario, this would detect gcc/clang/msvc
                    let output = process::Command::new("gcc")
                        .arg(c_file)
                        .arg("-o")
                        .arg("output.exe")
                        .output();
                        
                    match output {
                        Ok(res) if res.status.success() => {
                            println!("Build successful: output.exe");
                        }
                        Ok(res) => {
                            eprintln!("C Compiler Error:");
                            eprintln!("{}", String::from_utf8_lossy(&res.stderr));
                            process::exit(1);
                        }
                        Err(e) => {
                            eprintln!("Error: C compiler (gcc) not found or failed to execute: {}", e);
                            eprintln!("Viyal's optional AOT backend requires a C compiler. Please install GCC or use `viyal run` instead.");
                            process::exit(1);
                        }
                    }
                    
                    let _ = fs::remove_file(c_file);
                }
                Err(e) => {
                    eprintln!("Codegen error: {}", e);
                    process::exit(1);
                }
            }
        }
        "lsp" => {
            if let Err(e) = lsp::start_server() {
                eprintln!("LSP server error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}
