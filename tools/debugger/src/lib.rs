use vm::vm::{VM, InterpretResult};
use std::io::{self, Write};

use bytecode::compiler::CompiledProgram;

pub struct Debugger {
    vm: VM,
}

impl Debugger {
    pub fn new(program: CompiledProgram) -> Self {
        Self {
            vm: VM::new(program),
        }
    }
    
    pub fn vm_mut(&mut self) -> &mut VM {
        &mut self.vm
    }

    pub fn start_repl(&mut self) {
        println!("Viyal Debugger MVP started. Type 'h' for help.");
        
        loop {
            print!("(viyal-dbg) ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd = parts[0];
            
            match cmd {
                "h" | "help" => {
                    println!("Commands:");
                    println!("  b <ip>   - Set breakpoint at instruction pointer <ip>");
                    println!("  r, c     - Run or Continue execution");
                    println!("  s        - Step next instruction");
                    println!("  p        - Print VM stack");
                    println!("  q        - Quit");
                }
                "b" => {
                    if parts.len() < 2 {
                        println!("Error: Missing IP for breakpoint");
                        continue;
                    }
                    if let Ok(ip) = parts[1].parse::<usize>() {
                        self.vm.add_breakpoint(ip);
                        println!("Breakpoint set at IP: {}", ip);
                    } else {
                        println!("Error: Invalid IP");
                    }
                }
                "r" | "c" => {
                    let mut output = String::new();
                    match self.vm.run(&mut output) {
                        InterpretResult::Ok => {
                            print!("{}", output);
                            println!("Program execution finished.");
                            break;
                        }
                        InterpretResult::Breakpoint => {
                            println!("Hit breakpoint at IP: {}", self.vm.ip());
                        }
                        InterpretResult::CompileError => {
                            eprintln!("Runtime Compile Error");
                            break;
                        }
                        InterpretResult::RuntimeError(msg) => {
                            println!("Runtime error: {}", msg);
                            break;
                        }
                    }
                }
                "s" => {
                    // To step, we set a temporary breakpoint at the very next instruction
                    // But for MVP, we just add breakpoint at IP+1, run, then remove it
                    let next_ip = self.vm.ip();
                    self.vm.add_breakpoint(next_ip);
                    let mut output = String::new();
                    match self.vm.run(&mut output) {
                        InterpretResult::Breakpoint => {
                            println!("Stepped to IP: {}", self.vm.ip());
                        }
                        InterpretResult::Ok => {
                            println!("Program execution finished.");
                            break;
                        }
                        _ => {
                            println!("Execution halted due to error.");
                            break;
                        }
                    }
                }
                "p" => {
                    let stack = self.vm.stack();
                    println!("Stack ({} elements):", stack.len());
                    for (i, val) in stack.iter().enumerate() {
                        println!("  [{}] {:?}", i, val);
                    }
                }
                "q" => {
                    println!("Exiting debugger.");
                    break;
                }
                _ => {
                    println!("Unknown command: {}", cmd);
                }
            }
        }
    }
}
