use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[token("PUSH_I64")]
    PushI64,
    #[token("PUSH_F64")]
    PushF64,
    #[token("PUSH_BYTES")]
    PushBytes,
    #[token("POP")]
    Pop,
    #[token("DUP")]
    Dup,
    #[token("SWAP")]
    Swap,
    #[token("PUSH_NIL")]
    PushNil,
    #[token("ADD")]
    Add,
    #[token("SUB")]
    Sub,
    #[token("MUL")]
    Mul,
    #[token("DIV")]
    Div,
    #[token("MOD")]
    Mod,
    #[token("NEG")]
    Neg,
    #[token("EQ")]
    Eq,
    #[token("NE")]
    Ne,
    #[token("LT")]
    Lt,
    #[token("LE")]
    Le,
    #[token("GT")]
    Gt,
    #[token("GE")]
    Ge,
    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("XOR")]
    Xor,
    #[token("NOT")]
    Not,
    #[token("BOOL_AND")]
    BoolAnd,
    #[token("BOOL_OR")]
    BoolOr,
    #[token("BOOL_XOR")]
    BoolXor,
    #[token("BOOL_NOT")]
    BoolNot,
    #[token("JUMP")]
    Jump,
    #[token("JUMP_IF")]
    JumpIf,
    #[token("JUMP_IF_NOT")]
    JumpIfNot,
    #[token("CALL")]
    Call,
    #[token("RETURN")]
    Return,
    #[token("HALT")]
    Halt,
    #[token("YIELD")]
    Yield,
    #[token("EMIT")]
    Emit,
    #[token("LOAD")]
    Load,
    #[token("STORE")]
    Store,
    #[token("ALLOC")]
    Alloc,
    #[token("FREE")]
    Free,
    #[token("COPY")]
    Copy,
    #[token("SET")]
    Set,
    #[token("GET")]
    Get,
    #[token("VEC_PULL")]
    VecPull,
    #[token("VEC_PUSH")]
    VecPush,
    #[token("PROB_B")]
    ProbB,
    #[token("SNAP_S")]
    SnapS,
    #[token("SNAP_R")]
    SnapR,
    #[token("TOOL_X")]
    ToolX,

    #[token(";")]
    Comment,
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"0x[0-9a-fA-F]+", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..], 16).ok()
    })]
    HexInteger(Option<i64>),

    #[regex(r"0b[01]+", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..], 2).ok()
    })]
    BinaryInteger(Option<i64>),

    #[regex(r"-?\d+", |lex| {
        let s = lex.slice();
        s.parse::<i64>().ok()
    })]
    DecimalInteger(Option<i64>),

    #[regex(r"-?\d+\.\d*", |lex| {
        let s = lex.slice();
        s.parse::<f64>().ok()
    })]
    Float(Option<f64>),

    #[regex(r"-?\d+\.\d*[eE][+-]?\d+", |lex| {
        let s = lex.slice();
        s.parse::<f64>().ok()
    })]
    ScientificFloat(Option<f64>),

    #[token("\"")]
    QuoteBegin,
    #[token("\"")]
    QuoteEnd,

    #[error]
    Error,
}