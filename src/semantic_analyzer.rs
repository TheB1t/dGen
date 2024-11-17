use std::clone;

use crate::dgen_ast::*;
use crate::boxable::Boxable;

#[derive(Debug, Clone, PartialEq)]
enum Symbol {
    Var { typename: Type, name: String },
    Array { typename: Type, name: String, size: usize },
    Func { return_type: Type, name: String, params: Vec<Type> },
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
    pub fn typename(&self) -> &Type {
        match self {
            Symbol::Var { typename, .. } | Symbol::Array { typename, .. } | Symbol::Func { return_type: typename, .. } => typename,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Symbol::Var { name, .. } | Symbol::Array { name, .. } | Symbol::Func { name, .. } => name,
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Symbol::Array { size, .. } => Some(*size),
            _ => None,
        }
    }
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>) -> Self {
        Self { symbols: Vec::new(), parent }
    }

    pub fn define(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    pub fn find_symbol(&self, name: &str) -> Option<Symbol> {
        self.symbols.iter().find(|symbol| symbol.name() == name).cloned()
            .or_else(|| self.parent.as_ref()?.find_symbol(name))
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self { scope: Scope::new(None).wrap(), errors: Vec::new() }
    }

    fn enter_scope(&mut self) {
        self.scope = Scope::new(Some(self.scope.clone())).wrap();
    }

    fn exit_scope(&mut self) {
        self.scope = self.scope.parent.take().expect("No parent scope to exit to");
    }

    fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    fn analyze_expr(&mut self, expr: Expr) -> Type {
        match expr.clone() {
            Expr::Identifier(name)                  => self.lookup_type(&name),
            Expr::BinaryOp(_, left, right)          => self.check_binary_expr(*left, *right),
            Expr::UnaryOp(_, expr, _)               => self.analyze_expr(*expr),
            Expr::FuncCall(name, args)              => self.check_func_call(&name, args),
            Expr::Bool(_)                           => Type::Boolean,
            Expr::Number(_)                         => Type::Number,
            Expr::Array(elements)                   => self.check_array_init(elements),
            Expr::ArrayAccess(array, index)         => self.check_array_access(array, *index),
            _ => Type::Any,
        }
    }

    pub fn analyze(&mut self, stmt: Stmt) -> Stmt {
        match stmt.clone() {
            Stmt::Block(stmts)                      => {
                self.enter_scope();
                let analyzed = Stmt::Block(stmts.into_iter().map(|stmt| self.analyze(stmt)).collect());
                self.exit_scope();
                analyzed
            }
            Stmt::Program(stmts)                    => Stmt::Program(stmts.into_iter().map(|stmt| self.analyze(stmt)).collect()),
            Stmt::Expr(expr)                        => { self.analyze_expr(expr); stmt },
            Stmt::Assign(name, value)               => self.check_assignment(name, value, stmt),
            Stmt::VarDecl(t, name, value)           => self.check_var_decl(t, name, value, stmt),
            Stmt::FuncDecl(rtype, name, params)     => self.check_func_decl(rtype, name, params, stmt),
            Stmt::FuncDef(rtype, name, params, body) => self.check_func_def(rtype, name, params, body, stmt),
            Stmt::Return(expr)                      => { self.analyze_expr(expr); stmt },
            Stmt::If(cond, ifb, elseb)              => {
                self.analyze_expr(cond);
                self.analyze(*ifb);
                if let Some(elseb) = elseb { self.analyze(*elseb); }
                stmt
            }
            Stmt::For(init, cond, step, block)      => {
                self.analyze(*init);
                self.analyze_expr(cond);
                self.analyze(*step);
                self.analyze(*block);
                stmt
            }
            Stmt::While(cond, block)                => { self.analyze_expr(cond); self.analyze(*block); stmt }
            _ => stmt,
        }
    }

    fn lookup_type(&mut self, name: &str) -> Type {
        self.scope.find_symbol(name)
            .map_or_else(|| {
                self.add_error(&format!("Undefined variable: '{}'", name));
                Type::Any
            }, |symbol| symbol.typename().clone())
    }

    fn check_binary_expr(&mut self, left: Expr, right: Expr) -> Type {
        let left_type = self.analyze_expr(left.clone());
        let right_type = self.analyze_expr(right.clone());
        if left_type != right_type {
            self.add_error(&format!("Binary expression type mismatch: {:?} != {:?}", left_type, right_type));
            Type::Any
        } else {
            left_type
        }
    }

    fn check_array_init(&mut self, elements: Vec<Expr>) -> Type {
        let element_type = elements.first().map(|el| self.analyze_expr(el.clone()));
        if elements.iter().all(|el| element_type == Some(self.analyze_expr(el.clone()))) {
            Type::Array(element_type.unwrap_or(Type::Any).wrap())
        } else {
            self.add_error("Array elements have inconsistent types.");
            Type::Any
        }
    }

    fn check_array_access(&mut self, array_name: String, index: Expr) -> Type {
        match self.scope.find_symbol(&array_name) {
            Some(Symbol::Array { typename, size, .. }) => {
                if self.analyze_expr(index.clone()) != Type::Number {
                    self.add_error("Array index must be a number.");
                } else if let Expr::Number(i) = index {
                    if (i as usize) >= size {
                        self.add_error(&format!("Array index out of bounds: {} >= {}", i, size));
                    }
                }
                if let Type::Array(t) = typename {
                    *t
                } else {
                    self.add_error("Array type is not an array.");
                    Type::Any
                }
            }
            _ => {
                self.add_error(&format!("'{}' is not an array", array_name));
                Type::Any
            }
        }
    }

    fn check_func_call(&mut self, func_name: &str, args: Vec<Expr>) -> Type {
        if let Some(Symbol::Func { return_type, params, .. }) = self.scope.find_symbol(func_name) {
            if params.len() != args.len() {
                self.add_error(&format!("Function '{}' expects {} arguments, got {}", func_name, params.len(), args.len()));
            } else {
                for (param_type, arg) in params.iter().zip(args) {
                    let arg_type = self.analyze_expr(arg);
                    if arg_type != *param_type {
                        self.add_error(&format!("Argument type mismatch in '{}': expected {:?}, got {:?}", func_name, param_type, arg_type));
                    }
                }
            }
            return_type
        } else {
            self.add_error(&format!("Function '{}' is not defined", func_name));
            Type::Any
        }
    }

    fn check_assignment(&mut self, name: String, value: Expr, stmt: Stmt) -> Stmt {
        if let Some(symbol) = self.scope.find_symbol(&name) {
            if symbol.typename() != &self.analyze_expr(value) {
                self.add_error(&format!("Type mismatch in assignment to '{}'", name));
            }
        } else {
            self.add_error(&format!("Undefined variable: '{}'", name));
        }
        stmt
    }

    fn check_var_decl(&mut self, typename: Type, name: String, value: Option<Expr>, stmt: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(&format!("Redeclaration of '{}'", name));
        } else if let Some(expr) = value {
            let value_type = self.analyze_expr(expr.clone());
            if value_type != typename {
                self.add_error(&format!("Type mismatch in declaration of '{}'", name));
            }

            if let Expr::Array(v) = expr {
                let size = v.len();
                self.scope.define(Symbol::Array { typename, name, size });
            } else {
                self.scope.define(Symbol::Var { typename, name });
            }
        } else {
            self.scope.define(Symbol::Var { typename, name });
        }
        stmt
    }

    fn check_func_decl(&mut self, return_type: Type, name: String, params: Vec<Type>, stmt: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(&format!("Redeclaration of '{}'", name));
        } else {
            self.scope.define(Symbol::Func { return_type, name, params });
        }
        stmt
    }

    fn check_func_def(&mut self, return_type: Type, name: String, params: Vec<(Type, String)>, body: Box<Stmt>, stmt: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(&format!("Redeclaration of '{}'", name));
        } else {
            let param_types = params.iter().map(|(t, _)| t.clone()).collect();
            self.scope.define(Symbol::Func { return_type: return_type.clone(), name: name.clone(), params: param_types });
            self.enter_scope();
            for (param_type, param_name) in params {
                self.scope.define(Symbol::Var { typename: param_type, name: param_name });
            }
            self.analyze(*body);
            self.exit_scope();
        }
        stmt
    }
}
