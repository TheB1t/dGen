use crate::parser::*;
use crate::parser::Type;

#[derive(Debug, Clone, PartialEq)]
enum Symbol {
    Var {
        typename: Type,
        name: String,
    }
}

#[derive(Debug, Clone)]
struct Scope {
    symbols: Vec<Symbol>,
    parent: Option<Box<Scope>>,
}

#[derive(Debug)]
pub struct SemanticAnalyzer {
    scope: Box<Scope>,
    errors: Vec<String>,
}

impl Symbol {
    pub fn get_type(&self) -> Type {
        match &self {
            Symbol::Var { typename, name } => typename.clone()
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: Vec::new(),
            parent: None,
        }
    }

    pub fn define_var(&mut self, typename: Type, name: String) {
        let symbol = Symbol::Var {typename, name};
        self.symbols.push(symbol);
    }

    pub fn find_symbol(&self, name: &str) -> Option<Symbol> {
        self.symbols.iter().find(|symbol| match symbol {
            Symbol::Var { name: var_name, .. } => var_name == name,
        }).cloned()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.find_symbol(name).is_some() || self.parent.as_ref().map_or_else(|| false, |parent| parent.exists(name))
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scope: Box::new(Scope::new()),
            errors: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        let parent = Some(self.scope.clone());
        self.scope = Box::new(Scope {
            symbols: Vec::new(),
            parent,
        })
    }

    fn exit_scope(&mut self) {
        self.scope = self.scope.parent.take().unwrap();
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn analyze_expr(&mut self, expr: Expr) -> Type {
        match expr.clone() {
            Expr::Identifier(name) => {
                let sym = self.scope.find_symbol(&name);

                match sym {
                    Option::Some(v) => v.get_type(),
                    Option::None => {
                        self.add_error(format!("Variable '{}' does not exist", name));
                        Type::Any
                    }
                }
            },

            Expr::BinaryOp { op, left, right } => {
                let a_left = self.analyze_expr(*left);
                let a_right = self.analyze_expr(*right);

                if a_left != a_right {
                    self.add_error(format!("Binary expression type mismatch: {:#?} != {:#?} in {:#?}", a_left, a_right, expr));
                    Type::Any
                } else {
                    a_left
                }
            },

            Expr::UnaryOp { op, expr, .. } => {
                self.analyze_expr(*expr)
            },

            Expr::Bool(_) => Type::Boolean,
            Expr::Number(_) => Type::Number,

            _ => Type::Any,
        }
    }

    pub fn analyze(&mut self, root: Stmt) -> Stmt {
        match root.clone() {
            Stmt::Block(stmts) => {
                self.enter_scope();
                let res = Stmt::Block(stmts.into_iter().map(|stmt| self.analyze(stmt)).collect());
                self.exit_scope();
                res
            },

            Stmt::Expr(expr) => {
                self.analyze_expr(expr.clone());

                Stmt::Expr(expr)
            }

            Stmt::Assign { name, value } => {
                let sym = self.scope.find_symbol(&name);

                match sym {
                    Option::Some(v) => {
                        let a_value = self.analyze_expr(value.clone());
                        let expr_type = v.get_type();

                        if expr_type != a_value {
                            self.add_error(format!("Assign type mismatch: {:#?} != {:#?} in {:#?}", expr_type, a_value, root));
                        }
                    },
                    Option::None => {
                        self.add_error(format!("Variable '{}' does not exist", name));
                    }
                };

                Stmt::Assign { name, value }
            }

            Stmt::VarDecl { typename, name, value } => {
                if self.scope.exists(&name) {
                    self.add_error(format!("Variable '{}' already exists", name));
                } else {
                    self.scope.define_var(typename.clone(), name.clone());

                    let sym = self.scope.find_symbol(&name);
                    match sym {
                        Option::Some(v) => {
                            let a_value = self.analyze_expr(value.clone());
                            let expr_type = v.get_type();

                            if expr_type != a_value {
                                self.add_error(format!("Assign type mismatch: {:#?} != {:#?} in {:#?}", expr_type, a_value, root));
                            }
                        },
                        Option::None => {
                            self.add_error(format!("Variable '{}' does not exist", name));
                        }
                    };
                }
                Stmt::VarDecl { typename, name, value }
            }
        }
    }
}