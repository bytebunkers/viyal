import os
import re

directories = [
    "compiler/lexer", "compiler/parser", "compiler/ast", "compiler/resolver",
    "compiler/typechecker", "compiler/hir", "compiler/mir", "compiler/optimizer",
    "compiler/bytecode", "compiler/codegen", "compiler/backend",
    "runtime/gc", "runtime/async", "runtime/concurrency", "runtime/io", "runtime/ffi",
    "vm", "stdlib", "tools/cli", "tools/formatter", "tools/analyzer", "tools/lsp",
    "tools/debugger", "compiler/interpreter", "tools/pub", "tests/conformance"
]

for d in directories:
    toml_path = os.path.join(d, "Cargo.toml")
    if os.path.exists(toml_path):
        with open(toml_path, "r", encoding="utf-8") as f:
            content = f.read()
            
        # Update version
        content = re.sub(r'version = "0\.1\.1"', r'version = "0.2.0"', content)
        # Update internal dependencies
        content = re.sub(r'version = "0\.1\.1"(.*?)path =', r'version = "0.2.0"\1path =', content)
        
        with open(toml_path, "w", encoding="utf-8") as f:
            f.write(content)

print("Versions updated to 0.2.0")
