use std::collections::HashMap;
use std::cell::RefCell;
use crate::tokay::{Op, Program, Parselet};
use crate::value::RefValue;


/** Compiler symbolic scope.

In Tokay code, this relates to any block.
Scoped blocks (parselets) introduce new variable scopes.
*/
struct Scope {
    variables: Option<HashMap<String, usize>>,
    constants: HashMap<String, usize>,
    parselets: usize
}


/** Tokay compiler instance, with related objects. */
pub struct Compiler {
    scopes: Vec<Scope>,                     // Current compilation scopes
    values: Vec<RefValue>,                  // Constant values collected during compile
    parselets: Vec<RefCell<Parselet>>       // Parselets
}

impl Compiler {
    pub fn new() -> Self {
        Self{
            scopes: vec![
                Scope{
                    variables: Some(HashMap::new()),
                    constants: HashMap::new(),
                    parselets: 0
                }
            ],
            values: Vec::new(),
            parselets: Vec::new()
        }
    }

    /** Converts the compiled information into a Program. */
    pub fn into_program(mut self) -> Program {
        // Close any open scopes
        while self.scopes.len() > 1 {
            self.pop_scope();
        }

        // Resolve last scope
        self.resolve(true);

        // Finalize
        Parselet::finalize(&self.parselets);

        // Drain parselets into the new program
        Program::new(
            self.parselets.drain(..).map(|p| p.into_inner()).collect(),
            self.values,
            //self.scopes[0].variables.len()  # fixme: these are the globals...
        )
    }

    /// Introduces a new scope, either for variables or constants only.
    pub fn push_scope(&mut self, variables: bool) {
        self.scopes.insert(0,
            Scope{
                variables: if variables { Some(HashMap::new()) } else { None },
                constants: HashMap::new(),
                parselets: self.parselets.len()
            }
        );
    }

    /** Pops current scope. Returns number of locals defined.

    The final (main) scope cannot be dropped, the function panics when
    this is tried. */
    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Can't pop main scope");
        }

        self.resolve(false);
        self.scopes.remove(0);
    }

    /// Returns the total number of locals in current scope.
    pub fn get_locals(&self) -> usize {
        if let Some(locals) = &self.scopes.first().unwrap().variables {
            locals.len()
        }
        else {
            0
        }
    }

    /**
    Retrieve address of a local variable under a given name;
    The define-parameter for automatic variable inseration
    in case it doesn't exist.
    */
    pub fn get_local(&mut self, name: &str, define: bool)
        -> Option<usize>
    {
        for scope in &mut self.scopes {
            // Check for scope with variables
            if let Some(variables) = &mut scope.variables {
                if let Some(addr) = variables.get(name) {
                    return Some(*addr)
                }
                else if define {
                    let addr = variables.len();
                    variables.insert(name.to_string(), addr);
                    return Some(addr)
                }
                else {
                    break
                }
            }
        }

        None
    }

    /**
    Retrieve address of a global variable.
    */
    pub fn get_global(&self, name: &str) -> Option<usize>
    {
        let variables = self.scopes.last().unwrap().variables.as_ref().unwrap();

        if let Some(addr) = variables.get(name) {
            Some(*addr)
        }
        else {
            None
        }
    }

    /** Set constant to name in current scope. */
    pub fn set_constant(&mut self, name: &str, value: RefValue) {
        assert!(Self::is_constant(name));

        let addr = self.define_value(value);

        self.scopes.first_mut().unwrap().constants.insert(
            name.to_string(), addr
        );
    }

    /** Get constant, either from current or preceding scope. */
    pub fn get_constant(&self, name: &str) -> Option<RefValue> {
        assert!(Self::is_constant(name));

        for scope in &self.scopes {
            if let Some(addr) = scope.constants.get(name) {
                return Some(self.values[*addr].clone());
            }
        }

        None
    }

    /** Defines a new static value.

    Statics are moved into the program later on. */
    pub fn define_value(&mut self, value: RefValue) -> usize
    {
        self.values.push(value);
        self.values.len() - 1
    }

    /** Defines a new parselet code element.

    Parselets are moved into the program later on. */
    pub fn define_parselet(&mut self, parselet: Parselet) -> usize
    {
        self.parselets.push(RefCell::new(parselet));
        self.parselets.len() - 1
    }

    /** Resolve all parseletes defined in the current scope. */
    pub fn resolve(&mut self, strict: bool) {
        let scope = self.scopes.first().unwrap();

        for i in scope.parselets..self.parselets.len() {
            self.parselets[i].borrow_mut().resolve(&self, strict);
        }
    }

    /** Check if a str defines a constant or not. */
    pub fn is_constant(name: &str) -> bool {
        let ch = name.chars().nth(0).unwrap();
        ch.is_uppercase() || ch == '_'
    }

    pub fn gen_store(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name, false) {
            Op::StoreFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::StoreGlobal(addr)
        }
        else {
            Op::StoreFast(self.get_local(name, true).unwrap())
        }
    }

    pub fn gen_load(&mut self, name: &str) -> Op {
        if let Some(addr) = self.get_local(name, false) {
            Op::LoadFast(addr)
        }
        else if let Some(addr) = self.get_global(name) {
            Op::LoadGlobal(addr)
        }
        else {
            Op::LoadFast(self.get_local(name, true).unwrap())
        }
    }
}

/* A minimalistic Tokay compiler as Rust macros. */

#[macro_export]
macro_rules! tokay_item {

    // Assign a value
    ( $compiler:expr, ( $name:ident = $value:literal ) ) => {
        {
            let name = stringify!($name).to_string();
            let value = Value::String($value.to_string()).into_ref();

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    value
                );

                None
            }
            else {
                let addr = $compiler.define_value(value);

                Some(
                    Sequence::new(
                        vec![
                            (Op::LoadStatic(addr), None),
                            ($compiler.gen_store(&name), None)
                        ]
                    )
                )
            }

            //println!("assign {} = {}", stringify!($name), stringify!($value));
        }
    };

    // Assign whitespace
    ( $compiler:expr, ( _ = { $( $item:tt ),* } ) ) => {
        {
            $compiler.push_scope(true);
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            let body = Repeat::new(body, 0, 0, true);

            let parselet = $compiler.define_parselet(
                Parselet::new_muted(body, $compiler.get_locals())
            );

            $compiler.pop_scope();

            $compiler.set_constant(
                "_",
                Value::Parselet(parselet).into_ref()
            );

            //println!("assign _ = {}", stringify!($item));
            None
        }
    };

    // Assign parselet
    ( $compiler:expr, ( $name:ident = { $( $item:tt ),* } ) ) => {
        {
            let name = stringify!($name).to_string();

            $compiler.push_scope(true);
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            let body = Block::new(
                items.into_iter()
                    .filter(|item| item.is_some())
                    .map(|item| item.unwrap())
                    .collect()
            );

            let parselet = $compiler.define_parselet(
                Parselet::new(body, $compiler.get_locals())
            );

            $compiler.pop_scope();

            let parselet = Value::Parselet(parselet).into_ref();

            if Compiler::is_constant(&name) {
                $compiler.set_constant(
                    &name,
                    parselet
                );

                None
            }
            else {
                let addr = $compiler.define_value(parselet);

                Some(
                    Sequence::new(
                        vec![
                            (Op::LoadStatic(addr), None),
                            ($compiler.gen_store(&name), None)
                        ]
                    )
                )
            }

            //println!("assign {} = {}", stringify!($name), stringify!($item));
        }
    };

    // Sequence
    ( $compiler:expr, [ $( $item:tt ),* ] ) => {
        {
            //println!("sequence");
            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            Some(
                Sequence::new(
                    items.into_iter()
                        .filter(|item| item.is_some())
                        .map(|item| (item.unwrap(), None))
                        .collect()
                )
            )
        }
    };

    // Block
    ( $compiler:expr, { $( $item:tt ),* } ) => {
        {
            /*
            $(
                println!("{:?}", stringify!($item));
            )*
            */

            let items = vec![
                $(
                    tokay_item!($compiler, $item)
                ),*
            ];

            Some(
                Block::new(
                    items.into_iter()
                        .filter(|item| item.is_some())
                        .map(|item| item.unwrap())
                        .collect()
                )
            )
        }
    };

    // Call
    ( $compiler:expr, $ident:ident ) => {
        {
            //println!("call = {}", stringify!($ident));
            let name = stringify!($ident);

            if Compiler::is_constant(name) {
                let mut item = Op::Name(name.to_string());
                item.resolve(&$compiler, false);
                Some(item)
            }
            else {
                Some(
                    Sequence::new(
                        vec![
                            ($compiler.gen_load(name), None),
                            (Op::TryCall, None)
                        ]
                    )
                )
            }
        }
    };

    // Whitespace
    ( $compiler:expr, _ ) => {
        {
            //println!("expr = {}", stringify!($expr));
            let mut item = Op::Name("_".to_string());
            item.resolve(&$compiler, false);
            Some(item)
        }
    };

    // Match / Touch
    ( $compiler:expr, $literal:literal ) => {
        {
            Some(Match::new($literal))
        }
    };

    // Fallback
    ( $compiler:expr, $expr:tt ) => {
        {
            //println!("expr = {}", stringify!($expr));
            Some($expr)
        }
    };
}


#[macro_export]
macro_rules! tokay {
    ( $( $items:tt ),* ) => {
        {
            let mut compiler = Compiler::new();
            let main = tokay_item!(compiler, $( $items ),*);

            if let Some(main) = main {
                compiler.define_parselet(
                    Parselet::new(
                        main,
                        compiler.get_locals()
                    )
                );
            }

            compiler.into_program()
        }
    }
}
