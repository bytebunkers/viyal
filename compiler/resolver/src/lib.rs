use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use ast::{Program, Decl, Expr, Stmt, Type, Spanned, MatchPattern};
use parser::Parser;

pub struct LinkError {
    pub message: String,
    pub path: PathBuf,
}

pub struct ModuleLinker {
    visited: HashSet<PathBuf>,
    /// Files currently being resolved — used to detect circular imports.
    resolving: HashSet<PathBuf>,
    merged_decls: Vec<Spanned<Decl>>,
}

impl ModuleLinker {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            resolving: HashSet::new(),
            merged_decls: Vec::new(),
        }
    }

    pub fn link(&mut self, main_path: &Path) -> Result<Program, LinkError> {
        let abs_path = main_path.canonicalize().unwrap_or_else(|_| main_path.to_path_buf());
        
        self.process_file(&abs_path, "main")?;
        
        Ok(Program {
            declarations: self.merged_decls.clone(),
        })
    }
    
    fn process_file(&mut self, file_path: &Path, module_name: &str) -> Result<(), LinkError> {
        // Cycle guard: if this file is already on the current resolution stack,
        // a circular import exists. Emit a clear error rather than infinite recursing.
        if self.resolving.contains(file_path) {
            return Err(LinkError {
                message: format!(
                    "Circular import detected: '{}' is already being resolved. \
                     Check your import chain for cycles.",
                    file_path.display()
                ),
                path: file_path.to_path_buf(),
            });
        }
        if self.visited.contains(file_path) {
            return Ok(()); // Already processed in a previous branch
        }
        self.resolving.insert(file_path.to_path_buf());
        self.visited.insert(file_path.to_path_buf());
        
        let source = fs::read_to_string(file_path).map_err(|e| LinkError {
            message: format!("Failed to read file: {}", e),
            path: file_path.to_path_buf(),
        })?;
        
        let mut parser = Parser::new(&source);
        let mut program = parser.parse_program().map_err(|errs| LinkError {
            message: format!("Parse error: {}", errs[0].message),
            path: file_path.to_path_buf(),
        })?;
        
        // Find exports in this file so we know what can be renamed internally
        let mut local_exports = HashSet::new();
        for decl in &program.declarations {
            match &decl.node {
                Decl::Function(method, is_exported) => {
                    if *is_exported || module_name == "main" { local_exports.insert(method.name.clone()); }
                }
                Decl::Class { name, is_exported, .. } => {
                    if *is_exported || module_name == "main" { local_exports.insert(name.clone()); }
                }
                Decl::TypeAlias { name, is_exported, .. } => {
                    if *is_exported || module_name == "main" { local_exports.insert(name.clone()); }
                }
                _ => {}
            }
        }
        
        // Process imports
        // Map of local identifier -> mangled identifier
        let mut rename_map = HashMap::new();
        
        for decl in &program.declarations {
            if let Decl::Import { path, items } = &decl.node {
                // Resolve path relative to current file
                let import_path = file_path.parent().unwrap_or(Path::new("")).join(path);
                let import_path = if import_path.extension().is_none() {
                    import_path.with_extension("vyl")
                } else {
                    import_path
                };
                
                let import_abs = import_path.canonicalize().unwrap_or_else(|_| import_path.to_path_buf());
                
                // The module name is the file stem (e.g. "math")
                let import_mod_name = import_abs.file_stem().unwrap().to_string_lossy().to_string();
                
                // Process the imported file recursively
                self.process_file(&import_abs, &import_mod_name)?;
                
                // Register aliases in rename_map
                for (item_name, alias) in items {
                    let local_name = alias.clone().unwrap_or_else(|| item_name.clone());
                    let mangled_name = format!("{}::{}", import_mod_name, item_name);
                    rename_map.insert(local_name, mangled_name);
                }
            }
        }
        
        // Mangle local definitions
        if module_name != "main" {
            for decl in &mut program.declarations {
                match &mut decl.node {
                    Decl::Function(method, _) => {
                        let mangled = format!("{}::{}", module_name, method.name);
                        rename_map.insert(method.name.clone(), mangled.clone());
                        method.name = mangled;
                    }
                    Decl::Class { name, .. } => {
                        let mangled = format!("{}::{}", module_name, name);
                        rename_map.insert(name.clone(), mangled.clone());
                        *name = mangled;
                    }
                    Decl::TypeAlias { name, .. } => {
                        let mangled = format!("{}::{}", module_name, name);
                        rename_map.insert(name.clone(), mangled.clone());
                        *name = mangled;
                    }
                    _ => {}
                }
            }
        }
        
        // Rename all usages
        let mut renamer = Renamer { map: rename_map };
        for decl in &mut program.declarations {
            renamer.visit_decl(decl);
        }
        
        // Push all non-import decls to merged
        for decl in program.declarations {
            if !matches!(decl.node, Decl::Import { .. }) {
                self.merged_decls.push(decl);
            }
        }
        
        // Pop the file from the resolving stack — we have finished processing it.
        self.resolving.remove(file_path);
        Ok(())
    }
}

// AST Visitor to rename identifiers
struct Renamer {
    map: HashMap<String, String>,
}

impl Renamer {
    fn rename(&self, name: &mut String) {
        if let Some(mangled) = self.map.get(name) {
            *name = mangled.clone();
        }
    }
    
    fn rename_type(&self, ty: &mut Type) {
        match ty {
            Type::Named(name, _) => self.rename(name),
            Type::Array(inner) => self.rename_type(inner),
            Type::Map(k, v) => {
                self.rename_type(k);
                self.rename_type(v);
            }
            Type::Nullable(inner) => self.rename_type(inner),
            Type::Result(inner) => self.rename_type(inner),
            Type::Option(inner) => self.rename_type(inner),
        }
    }

    fn visit_decl(&mut self, decl: &mut Spanned<Decl>) {
        match &mut decl.node {
            Decl::Function(method, _) => {
                if let Some(rt) = &mut method.return_type {
                    self.rename_type(rt);
                }
                for param in &mut method.params {
                    self.rename_type(&mut param.param_type);
                }
                self.visit_stmt(&mut method.body);
            }
            Decl::Class { extends_class, implements_interfaces, fields, primary_constructor, methods, .. } => {
                if let Some(ext) = extends_class {
                    self.rename(ext);
                }
                for imp in implements_interfaces {
                    self.rename(imp);
                }
                for field in fields {
                    self.rename_type(&mut field.field_type);
                }
                for param in primary_constructor {
                    self.rename_type(&mut param.param_type);
                }
                for method in methods {
                    if let Some(rt) = &mut method.return_type {
                        self.rename_type(rt);
                    }
                    for param in &mut method.params {
                        self.rename_type(&mut param.param_type);
                    }
                    self.visit_stmt(&mut method.body);
                }
            }
            Decl::TypeAlias { target_type, .. } => {
                self.rename_type(target_type);
            }
            Decl::Import { .. } => {}
        }
    }
    
    fn visit_stmt(&self, stmt: &mut Spanned<Stmt>) {
        match &mut stmt.node {
            Stmt::Expr(expr) => self.visit_expr(expr),
            Stmt::VarDecl { type_annot, initializer, .. } => {
                if let Some(ty) = type_annot {
                    self.rename_type(ty);
                }
                if let Some(init) = initializer {
                    self.visit_expr(init);
                }
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.visit_stmt(s);
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                self.visit_stmt(then_branch);
                if let Some(e) = else_branch {
                    self.visit_stmt(e);
                }
            }
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                self.visit_stmt(body);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.visit_expr(iterable);
                self.visit_stmt(body);
            }
            Stmt::ForRange { start, end, body, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
                self.visit_stmt(body);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.visit_expr(e);
                }
            }
        }
    }
    
    fn visit_expr(&self, expr: &mut Spanned<Expr>) {
        match &mut expr.node {
            Expr::Identifier(name) => self.rename(name),
            Expr::Array(items) => {
                for item in items {
                    self.visit_expr(item);
                }
            }
            Expr::Map(pairs) => {
                for (k, v) in pairs {
                    self.visit_expr(k);
                    self.visit_expr(v);
                }
            }
            Expr::Binary(left, _, right) => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Call(callee, _, arguments) => {
                self.visit_expr(callee);
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }
            Expr::PropertyAccess(object, _) => {
                self.visit_expr(object);
            }
            Expr::SafePropertyAccess(object, _) => {
                self.visit_expr(object);
            }
            Expr::PropertyAssign(object, _, value) => {
                self.visit_expr(object);
                self.visit_expr(value);
            }
            Expr::NullCoalesce(left, right) => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Index(object, index) => {
                self.visit_expr(object);
                self.visit_expr(index);
            }
            Expr::IndexAssign(object, index, value) => {
                self.visit_expr(object);
                self.visit_expr(index);
                self.visit_expr(value);
            }
            Expr::New(class_name, _, arguments) => {
                self.rename(class_name);
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }
            Expr::Match(value, branches) => {
                self.visit_expr(value);
                for (pat, body) in branches {
                    if let MatchPattern::Identifier(name) = pat {
                        self.rename(name);
                    }
                    self.visit_expr(body);
                }
            }
            Expr::Try(e) => self.visit_expr(e),
            Expr::UnwrapOrElse(e, stmt) => {
                self.visit_expr(e);
                self.visit_stmt(stmt);
            }
            Expr::Literal(_) | Expr::This | Expr::Super => {}
        }
    }
}
