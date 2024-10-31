use crate::dgen_ast::*;
use crate::boxable::Boxable;

#[derive(Debug, Clone, PartialEq)]
enum Symbol {
    Var {
        typename: Type,
        name: String,
    },
    Func {
        return_type: Type,
        name: String,
        params: Vec<Type>
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
            Symbol::Var { typename, .. }        => typename.clone(),
            Symbol::Func { return_type, .. }    => return_type.clone()
        }
    }

    pub fn get_name(&self) -> String {
        match &self {
            Symbol::Var { typename:_, name }          => name.clone(),
            Symbol::Func { return_type:_, name, .. }  => name.clone()
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: Vec::new(),
            parent: None
        }
    }

    pub fn with_parent(parent: Box<Scope>) -> Self {
        Scope {
            symbols: Vec::new(),
            parent: Some(parent)
        }
    }

    pub fn define_var(&mut self, typename: Type, name: String) {
        self.symbols.push(Symbol::Var { typename, name });
    }

    pub fn define_func(&mut self, return_type: Type, name: String, params: Vec<Type>) {
        self.symbols.push(Symbol::Func { return_type, name, params });
    }

    pub fn find_symbol(&self, target: &str) -> Option<Symbol> {
        self.symbols.iter().find(|symbol| symbol.get_name() == target).cloned()
            .or_else(|| self.parent.as_deref()?.find_symbol(target))
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scope: Scope::new().into_box(),
            errors: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.scope = Scope::with_parent(self.scope.clone()).into_box();
    }

    fn exit_scope(&mut self) {
        self.scope = self.scope.parent.take().expect("No parent scope to exit to");
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn analyze_expr(&mut self, expr: Expr) -> Type {
        match expr.clone() {
            Expr::Identifier(name)              => self.lookup_type(&name, expr),
            Expr::BinaryOp  { left, right, .. } => self.check_binary_expr(*left, *right, expr),
            Expr::UnaryOp   { expr, .. }        => self.analyze_expr(*expr),
            Expr::FuncCall { name, args }       => self.check_func_call(name, *args),
            Expr::Bool(_)                       => Type::Boolean,
            Expr::Number(_)                     => Type::Number,
            _                                   => Type::Any,
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
            Stmt::Program(stmts) => Stmt::Program(stmts.into_iter().map(|stmt| self.analyze(stmt)).collect()),
            Stmt::Expr(expr) => {
                self.analyze_expr(expr);
                root
            },
            Stmt::Assign    { name, value }                     => self.check_assignment(name, value, root),
            Stmt::VarDecl   { typename, name, value }           => self.check_var_decl(typename, name, value, root),
            Stmt::FuncDecl  { return_type, name, params }       => self.check_func_decl(return_type, name, params, root),
            Stmt::FuncDef   { return_type, name, params, body } => self.check_func_def(return_type, name, params, body, root),
            Stmt::Return(expr) => {
                self.analyze_expr(expr);
                root
            },
            Stmt::Break => root,
            Stmt::Continue => root,
            Stmt::If { condition, if_block, else_block } => {
                self.analyze_expr(condition);
                self.analyze(*if_block);
                if let Some(else_block) = else_block {
                    self.analyze(*else_block);
                }
                root
            },
            Stmt::For { init, condition, step, block } => {
                self.analyze(*init);
                self.analyze_expr(condition);
                self.analyze(*step);
                self.analyze(*block);
                root
            },
            Stmt::While { condition, block } =>  {
                self.analyze_expr(condition);
                self.analyze(*block);
                root
            },
            _ => {
                println!("Unhandled statement type ({:?}) in semantic_analyzer", root.clone());
                root
            }
        }
    }

    fn lookup_type(&mut self, name: &str, expr: Expr) -> Type {
        self.scope.find_symbol(name)
            .map_or_else(|| {
                self.add_error(format!("Variable '{}' does not exist", name));
                Type::Any
            }, |symbol| symbol.get_type())
    }

    fn check_binary_expr(&mut self, left: Expr, right: Expr, expr: Expr) -> Type {
        let left_type = self.analyze_expr(left);
        let right_type = self.analyze_expr(right);

        if left_type != right_type {
            self.add_error(format!("Binary expression type mismatch: {:?} != {:?} in {:?}", left_type, right_type, expr));
            Type::Any
        } else {
            left_type
        }
    }

    fn check_func_call(&mut self, name: String, expr_list: Stmt) -> Type {
        match self.scope.find_symbol(&name) {
            Some(Symbol::Func { return_type, params, .. }) => {
                if let Stmt::ExprList(exprs) = expr_list {
                    if params.len() != exprs.len() {
                        self.add_error(format!("Function '{}' expects {} arguments, found {}", name, params.len(), exprs.len()));
                    } else {
                        for (param_type, expr) in params.iter().zip(exprs) {
                            let expr_type = self.analyze_expr(expr);
                            if expr_type != *param_type {
                                self.add_error(format!("Argument type mismatch in '{}': expected {:?}, found {:?}", name, param_type, expr_type));
                            }
                        }
                    }
                    return_type
                } else {
                    Type::Any
                }
            }
            _ => {
                self.add_error(format!("Function '{}' is not defined", name));
                Type::Any
            }
        }
    }

    fn check_assignment(&mut self, name: String, value: Expr, root: Stmt) -> Stmt {
        if let Some(symbol) = self.scope.find_symbol(&name) {
            let value_type = self.analyze_expr(value);
            if symbol.get_type() != value_type {
                self.add_error(format!("Assignment type mismatch: {:?} != {:?} in {:?}", symbol.get_type(), value_type, root));
            }
        } else {
            self.add_error(format!("Variable '{}' does not exist", name));
        }
        root
    }

    fn check_var_decl(&mut self, typename: Type, name: String, value: Option<Expr>, root: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(format!("Redeclaration of '{}'", name));
        } else {
            self.scope.define_var(typename.clone(), name.clone());
            if let Some(init_value) = value {
                let init_type = self.analyze_expr(init_value);
                if init_type != typename {
                    self.add_error(format!("Declaration type mismatch: {:?} != {:?} in {:?}", typename, init_type, root));
                }
            }
        }
        root
    }

    fn check_func_decl(&mut self, return_type: Type, name: String, params: Box<Stmt>, root: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(format!("Redeclaration of '{}'", name));
        } else if let Stmt::TypeList(param_types) = *params {
            self.scope.define_func(return_type, name, param_types);
        }
        root
    }

    fn check_func_def(&mut self, return_type: Type, name: String, params: Box<Stmt>, body: Box<Stmt>, root: Stmt) -> Stmt {
        if self.scope.find_symbol(&name).is_some() {
            self.add_error(format!("Redeclaration of '{}'", name));
        } else if let Stmt::ParamList(param_decls) = *params {
            let param_types = param_decls.clone().into_iter().map(|param| match param {
                Stmt::VarDecl { typename, .. }  => typename,
                _                               => Type::Any,
            }).collect();
            self.scope.define_func(return_type, name, param_types);
            self.enter_scope();
            for param in param_decls {
                self.analyze(param);
            };
            self.analyze(*body);
            self.exit_scope();
        }
        root
    }
}