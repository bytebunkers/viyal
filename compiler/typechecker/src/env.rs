use ast::Type;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MethodSignature {
    pub params: Vec<Type>,
    pub return_type: Option<Type>,
}

#[derive(Clone)]
pub struct ClassSignature {
    pub methods: HashMap<String, MethodSignature>,
}

pub struct TypeEnv {
    scopes: Vec<HashMap<String, (Type, bool)>>, // bool is `is_mut`
    pub classes: HashMap<String, ClassSignature>,
    pub type_aliases: HashMap<String, Type>,
    pub functions: HashMap<String, MethodSignature>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            classes: HashMap::new(),
            type_aliases: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, ty: Type, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (ty, is_mut));
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        self.lookup_full(name).map(|(ty, _)| ty)
    }

    pub fn lookup_full(&self, name: &str) -> Option<(Type, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some((ty, is_mut)) = scope.get(name) {
                return Some((ty.clone(), *is_mut));
            }
        }
        None
    }

    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(name) => {
                if let Some(resolved) = self.type_aliases.get(name) {
                    self.resolve_type(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.resolve_type(inner))),
            Type::Map(k, v) => Type::Map(Box::new(self.resolve_type(k)), Box::new(self.resolve_type(v))),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.resolve_type(inner))),
            Type::Result(inner) => Type::Result(Box::new(self.resolve_type(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.resolve_type(inner))),
        }
    }
}
