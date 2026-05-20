use std::collections::{HashMap, HashSet};

use super::checker::SimpleType;

/// Signature for a callable value (function, builtin or method).
#[derive(Clone)]
pub(super) struct CallableSignature {
    pub(super) params: Vec<Option<SimpleType>>,
    pub(super) return_type: Option<SimpleType>,
}

/// Context holds scoped variables, type/function/macro registries and builtins.
#[derive(Clone)]
pub struct Context {
    var_scopes: Vec<HashSet<String>>,
    var_types: Vec<HashMap<String, SimpleType>>,
    functions: HashMap<String, HashSet<usize>>,
    function_signatures: HashMap<String, CallableSignature>,
    macros: HashMap<String, HashSet<usize>>,
    types: HashMap<String, TypeInfo>,
    protocols: HashSet<String>,
    builtin_functions: HashMap<String, HashSet<usize>>,
    builtin_function_signatures: HashMap<String, HashMap<usize, CallableSignature>>,
    builtin_types: HashSet<String>,
    builtin_consts: HashSet<String>,
    pub(super) current_type: Option<CurrentTypeInfo>,
    pub(super) in_method: bool,
}

/// Type metadata recorded in the context.
#[derive(Clone)]
struct TypeInfo {
    param_count: usize,
    attrs: HashSet<String>,
    methods: HashMap<String, HashMap<usize, CallableSignature>>,
}

/// Information about the currently checked type (attributes and methods).
#[derive(Clone)]
pub(super) struct CurrentTypeInfo {
    pub(super) parent: Option<String>,
    pub(super) attrs: HashSet<String>,
    pub(super) methods: HashMap<String, HashMap<usize, CallableSignature>>,
}

impl Context {
    /// Create a new context populated with builtin functions/types/constants.
    pub(super) fn new() -> Self {
        Self {
            var_scopes: vec![HashSet::new()],
            var_types: vec![HashMap::new()],
            functions: HashMap::new(),
            function_signatures: HashMap::new(),
            macros: HashMap::new(),
            types: HashMap::new(),
            protocols: HashSet::new(),
            builtin_functions: builtin_functions(),
            builtin_function_signatures: builtin_function_signatures(),
            builtin_types: builtin_types(),
            builtin_consts: builtin_consts(),
            current_type: None,
            in_method: false,
        }
    }

    /// Push a new variable scope.
    pub(super) fn push_scope(&mut self) {
        self.var_scopes.push(HashSet::new());
        self.var_types.push(HashMap::new());
    }

    /// Pop the current variable scope.
    pub(super) fn pop_scope(&mut self) {
        self.var_scopes.pop();
        self.var_types.pop();
    }

    /// Define a variable in the current scope. Returns false on redefinition.
    pub(super) fn define_var(&mut self, name: &str) -> bool {
        if let Some(scope) = self.var_scopes.last_mut() {
            return scope.insert(name.to_string());
        }
        false
    }

    /// Set an inferred simple type for a variable in the current scope.
    pub(super) fn set_var_type(&mut self, name: &str, ty: SimpleType) {
        if let Some(scope) = self.var_types.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    /// Check if a variable is defined in any active scope.
    pub(super) fn is_var_defined(&self, name: &str) -> bool {
        self.var_scopes.iter().rev().any(|s| s.contains(name))
    }

    /// Get the inferred simple type for a variable if available.
    pub(super) fn var_type(&self, name: &str) -> Option<SimpleType> {
        for scope in self.var_types.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Register a function name with its arity.
    pub(super) fn insert_function(&mut self, name: &str, arity: usize) -> bool {
        insert_arity(&mut self.functions, name, arity)
    }

    /// Register the signature of a function.
    pub(super) fn insert_function_signature(
        &mut self,
        name: &str,
        params: Vec<Option<SimpleType>>,
        return_type: Option<SimpleType>,
    ) {
        self.function_signatures.insert(
            name.to_string(),
            CallableSignature { params, return_type },
        );
    }

    /// Register a macro name with its arity.
    pub(super) fn insert_macro(&mut self, name: &str, arity: usize) -> bool {
        insert_arity(&mut self.macros, name, arity)
    }

    /// Register a user-defined type with the number of type parameters.
    pub(super) fn insert_type(&mut self, name: &str, param_count: usize) {
        self.types
            .entry(name.to_string())
            .or_insert(TypeInfo { param_count, attrs: HashSet::new(), methods: HashMap::new() });
    }

    /// Set recorded attributes and methods for a previously registered type.
    pub(super) fn set_type_members(
        &mut self,
        name: &str,
        attrs: HashSet<String>,
        methods: HashMap<String, HashMap<usize, CallableSignature>>,
    ) {
        if let Some(t) = self.types.get_mut(name) {
            t.attrs = attrs;
            t.methods = methods;
        }
    }

    /// Get the signature of a method for a given type name and arity.
    pub(super) fn type_method_signature(
        &self,
        type_name: &str,
        method: &str,
        arity: usize,
    ) -> Option<&CallableSignature> {
        self.types
            .get(type_name)
            .and_then(|t| t.methods.get(method))
            .and_then(|by_arity| by_arity.get(&arity))
    }

    /// Register a protocol name.
    pub(super) fn insert_protocol(&mut self, name: &str) {
        self.protocols.insert(name.to_string());
    }

    /// Check whether a function with a given arity exists.
    pub(super) fn has_function(&self, name: &str, arity: usize) -> bool {
        has_arity(&self.functions, name, arity)
    }

    /// Check whether a macro with a given arity exists.
    pub(super) fn has_macro(&self, name: &str, arity: usize) -> bool {
        has_arity(&self.macros, name, arity)
    }

    /// Check whether a builtin function with a given arity exists.
    pub(super) fn has_builtin_function(&self, name: &str, arity: usize) -> bool {
        has_arity(&self.builtin_functions, name, arity)
    }

    /// Get the signature of a user-defined function if available.
    pub(super) fn function_signature(&self, name: &str) -> Option<&CallableSignature> {
        self.function_signatures.get(name)
    }

    /// Get the signature of a builtin function for a given arity if available.
    pub(super) fn builtin_function_signature(
        &self,
        name: &str,
        arity: usize,
    ) -> Option<&CallableSignature> {
        self.builtin_function_signatures
            .get(name)
            .and_then(|sigs| sigs.get(&arity))
    }

    /// Check whether any function with the name exists (any arity).
    pub(super) fn has_function_name(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Check whether any macro with the name exists (any arity).
    pub(super) fn has_macro_name(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Is the given name a builtin function?
    pub(super) fn is_builtin_function(&self, name: &str) -> bool {
        self.builtin_functions.contains_key(name)
    }

    /// Is the given name a builtin type?
    pub(super) fn is_builtin_type(&self, name: &str) -> bool {
        self.builtin_types.contains(name)
    }

    /// Is the given name a builtin constant?
    pub(super) fn is_builtin_const(&self, name: &str) -> bool {
        self.builtin_consts.contains(name)
    }

    /// Is a protocol defined with this name?
    pub(super) fn is_protocol_defined(&self, name: &str) -> bool {
        self.protocols.contains(name)
    }

    /// Is a type or protocol defined with this name?
    pub(super) fn is_type_or_protocol_defined(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.protocols.contains(name)
    }

    /// Is this a known type (builtin, user type or protocol)?
    pub(super) fn is_known_type(&self, name: &str) -> bool {
        self.builtin_types.contains(name)
            || self.types.contains_key(name)
            || self.protocols.contains(name)
    }

    /// Is this a constructible type (builtin or user-defined)?
    pub(super) fn is_constructible_type(&self, name: &str) -> bool {
        self.builtin_types.contains(name) || self.types.contains_key(name)
    }

    /// Return number of type parameters for a type if known.
    pub(super) fn type_param_count(&self, name: &str) -> Option<usize> {
        if self.builtin_types.contains(name) {
            return Some(0);
        }
        self.types.get(name).map(|t| t.param_count)
    }
}

/// Builtin functions and allowed arities.
fn builtin_functions() -> HashMap<String, HashSet<usize>> {
    let mut map = HashMap::new();
    map.insert("sin".to_string(), arity_set(&[1]));
    map.insert("cos".to_string(), arity_set(&[1]));
    map.insert("sqrt".to_string(), arity_set(&[1]));
    map.insert("exp".to_string(), arity_set(&[1]));
    map.insert("log".to_string(), arity_set(&[1, 2]));
    map.insert("rand".to_string(), arity_set(&[0]));
    map.insert("print".to_string(), arity_set(&[1]));
    map.insert("range".to_string(), arity_set(&[2]));
    map
}

/// Builtin function signatures used for type checking.
fn builtin_function_signatures() -> HashMap<String, HashMap<usize, CallableSignature>> {
    let mut map = HashMap::new();

    map.insert(
        "sin".to_string(),
        signature_map(vec![(1, vec![Some(SimpleType::Number)], Some(SimpleType::Number))]),
    );
    map.insert(
        "cos".to_string(),
        signature_map(vec![(1, vec![Some(SimpleType::Number)], Some(SimpleType::Number))]),
    );
    map.insert(
        "sqrt".to_string(),
        signature_map(vec![(1, vec![Some(SimpleType::Number)], Some(SimpleType::Number))]),
    );
    map.insert(
        "exp".to_string(),
        signature_map(vec![(1, vec![Some(SimpleType::Number)], Some(SimpleType::Number))]),
    );
    map.insert(
        "log".to_string(),
        signature_map(vec![
            (1, vec![Some(SimpleType::Number)], Some(SimpleType::Number)),
            (
                2,
                vec![Some(SimpleType::Number), Some(SimpleType::Number)],
                Some(SimpleType::Number),
            ),
        ]),
    );
    map.insert(
        "rand".to_string(),
        signature_map(vec![(0, vec![], Some(SimpleType::Number))]),
    );
    map.insert(
        "print".to_string(),
        signature_map(vec![(1, vec![None], None)]),
    );
    map.insert(
        "range".to_string(),
        signature_map(vec![(
            2,
            vec![Some(SimpleType::Number), Some(SimpleType::Number)],
            Some(SimpleType::Vector(Box::new(SimpleType::Number))),
        )]),
    );

    map
}

/// Builtin type names.
fn builtin_types() -> HashSet<String> {
    ["Number", "String", "Boolean", "Object"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Builtin constant names.
fn builtin_consts() -> HashSet<String> {
    ["PI", "E", "()"].iter().map(|s| s.to_string()).collect()
}

/// Create a HashSet of arities from an array.
fn arity_set(values: &[usize]) -> HashSet<usize> {
    values.iter().copied().collect()
}

/// Build a builtin signature map keyed by arity.
fn signature_map(
    values: Vec<(usize, Vec<Option<SimpleType>>, Option<SimpleType>)>,
) -> HashMap<usize, CallableSignature> {
    values
        .into_iter()
        .map(|(arity, params, return_type)| {
            (arity, CallableSignature { params, return_type })
        })
        .collect()
}

/// Insert an arity into the map for a given name.
fn insert_arity(map: &mut HashMap<String, HashSet<usize>>, name: &str, arity: usize) -> bool {
    let entry = map.entry(name.to_string()).or_insert_with(HashSet::new);
    entry.insert(arity)
}

/// Check if a name has a given arity in the map.
fn has_arity(map: &HashMap<String, HashSet<usize>>, name: &str, arity: usize) -> bool {
    map.get(name).map_or(false, |set| set.contains(&arity))
}