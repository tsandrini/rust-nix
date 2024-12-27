// TODO
// 1. Handle nested recursive attrsets
// 2. Add functions
// 3. Builtins

use std::collections::HashMap;

#[macro_export]
macro_rules! nix {
    // String literals
    ("$str:literal") => {
        NixValue::Str($str.to_string())
    };

    // Boolean literals
    (true) => { NixValue::Bool(true) };
    (false) => { NixValue::Bool(false) };

    // Variable reference
    ($id:ident) => {
        NixValue::VarRef(stringify!($id).to_string())
    };

    // Number/fallback literals
    ($val:literal) => {{
        if let Ok(parsed_int) = stringify!($val).parse::<i64>() {
            NixValue::Int(parsed_int)
        } else {
           NixValue::Int(30)
        }
    }};

    // Recursive attribute sets
    (rec { $($key:ident = $value:tt;)* }) => {
        NixValue::AttrSet({
            let mut map = HashMap::new();
            $(
                map.insert(stringify!($key).to_string(), (nix!($value), true));
            )*
            map
        })
    };

    // Regular attribute sets
    ({ $($key:ident = $value:tt;)* }) => {
        NixValue::AttrSet({
            let mut map = HashMap::new();
            $(
                map.insert(stringify!($key).to_string(), (nix!($value), false));
            )*
            map
        })
    };

    // Lists
    ([$($value:tt) *]) => {
        NixValue::List(vec![$(nix!($value)),*])
    };
}

#[derive(Debug, Clone)]
enum NixValue {
    Int(i64),
    Str(String),
    Bool(bool),
    List(Vec<NixValue>),
    AttrSet(HashMap<String, (NixValue, bool)>), // (value, is_recursive)
    VarRef(String),
}

impl NixValue {
    fn evaluate(&self, scope: &NixScope) -> Self {
        match self {
            NixValue::Int(_) | NixValue::Str(_) | NixValue::Bool(_) => self.clone(),
            NixValue::List(items) => {
                NixValue::List(items.iter().map(|v| v.evaluate(scope)).collect())
            }
            NixValue::AttrSet(attrs) => {
                let mut local_scope = scope.clone();
                let mut result = HashMap::new();

                // First pass: evaluate non-references
                for (key, (value, is_recursive)) in attrs {
                    let evaluated = if *is_recursive {
                        value.evaluate(&local_scope)
                    } else {
                        value.evaluate(scope)
                    };
                    result.insert(key.clone(), (evaluated.clone(), *is_recursive));
                    if *is_recursive {
                        local_scope.insert(key, evaluated);
                    }
                }

                // Second pass: resolve references
                let final_attrs = result
                    .into_iter()
                    .map(|(k, (v, is_rec))| {
                        let final_value = if is_rec { v.evaluate(&local_scope) } else { v };
                        (k, (final_value, is_rec))
                    })
                    .collect();

                NixValue::AttrSet(final_attrs)
            }
            NixValue::VarRef(name) => scope
                .get(name)
                .map(|v| v.clone())
                .unwrap_or_else(|| NixValue::VarRef(name.clone())),
        }
    }
}
#[derive(Debug, Clone)]
struct NixScope {
    variables: HashMap<String, NixValue>,
}

impl NixScope {
    fn new() -> Self {
        NixScope {
            variables: HashMap::new(),
        }
    }

    fn insert(&mut self, key: &str, value: NixValue) {
        self.variables.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<&NixValue> {
        self.variables.get(key)
    }
}

fn main() {
    let mut scope = NixScope::new();

    let nix_expression = nix!({
        x = 10;
        eh = x;
        uh = [ 3 4 6 ];
        m = {
            l = 10;
        };
    });

    println!("{:?}", nix_expression.evaluate(&scope));
}
