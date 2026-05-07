use crate::lexer::lexer::SpannedToken;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Term(Term),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Factor(Factor),
    Binary {
        left: Box<Term>,
        op: BinaryOp,
        right: Box<Term>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Factor {
    Number(String),
    Group(Box<Expression>),
    Binary {
        left: Box<Factor>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Ident(String),
    Call {
        callee: String,
        args: Vec<Expression>,
    },
    BuiltinCall {
        func: BuiltinFn,
        args: Vec<Expression>,
    },
    Const(ConstValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: SpannedToken,
    pub ty: Option<SpannedToken>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Inline(Expression),
    Block(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: SpannedToken,
    pub params: Vec<FunctionParam>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFn {
    Sin,
    Cos,
    Log,
    Tan,
    Sqrt,
    Exp,
    Rand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstValue {
    Pi,
    E,
}
