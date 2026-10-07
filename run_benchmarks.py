import subprocess
import time
import os

def run_cmd(cmd):
    start = time.time()
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=True)
        end = time.time()
        return float(end - start), res.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error running {' '.join(cmd)}: {e.stderr}")
        return 0.0, ""

print("==== Compiler Benchmark: fib(35) ====\n")

# 1. Python
t, out = run_cmd(["python", "benchmarks/fib.py"])
print(f"Python - Exec: {t:.4f}s")

# 2. Node.js
t, out = run_cmd(["node", "benchmarks/fib.js"])
if t > 0:
    print(f"Node.js - Exec: {t:.4f}s")

# 3. Viyal (VM Bytecode)
t, out = run_cmd(["target/release/viyal", "run", "benchmarks/fib.vyl"])
print(f"Viyal (VM Bytecode) - Total: {t:.4f}s")

# 4. Viyal (Cranelift JIT)
t, out = run_cmd(["target/release/viyal", "run", "--jit", "benchmarks/fib.vyl"])
print(f"Viyal (Cranelift JIT) - Total: {t:.4f}s")

# 4. Viyal C Codegen (TCC)
print("\nCompiling Viyal (C Codegen via TCC)...")
t_comp, _ = run_cmd(["target/release/viyal", "build", "benchmarks/fib.vyl"])
t_exec, out = run_cmd(["./output.exe"])
print(f"Viyal (C Codegen via TCC) - Compile: {t_comp:.4f}s | Exec: {t_exec:.4f}s")

# 5. Viyal C Codegen (Clang -O3)
print("\nCompiling Viyal (C Codegen via Clang -O3)...")
t_comp, _ = run_cmd(["target/release/viyal", "build", "--clang", "benchmarks/fib.vyl"])
if t_comp > 0 and os.path.exists("output.exe"):
    t_exec, out = run_cmd(["./output.exe"])
    print(f"Viyal (C Codegen via Clang -O3) - Compile: {t_comp:.4f}s | Exec: {t_exec:.4f}s")
else:
    print("Viyal (C Codegen via Clang -O3) failed (is clang installed?).")

# 6. Viyal AOT (Cranelift)
print("\nCompiling Viyal (Cranelift AOT)...")
t_comp, out = run_cmd(["target/release/viyal", "build", "--release", "benchmarks/fib.vyl"])
if t_comp > 0 and os.path.exists("output.exe"):
    t_exec, out = run_cmd(["./output.exe"])
    print(f"Viyal (Cranelift AOT) - Compile: {t_comp:.4f}s | Exec: {t_exec:.4f}s")
else:
    print("Viyal (Cranelift AOT) failed to link.")

if os.path.exists("output.exe"):
    os.remove("output.exe")

