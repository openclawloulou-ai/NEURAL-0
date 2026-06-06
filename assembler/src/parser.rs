use crate::lexer::Token;
use logos::Lexer;

/// AST nodes for the assembly language
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Core
    Nop,
    PushI64(i64),
    PushF64(f64),
    PushBytes(Vec<u8>),
    Pop,
    Dup,
    Swap,
    PushNil,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logic
    And,
    Or,
    Xor,
    Not,
    BoolAnd,
    BoolOr,
    BoolXor,
    BoolNot,

    // Control Flow
    Jump(String),           // Label or address
    JumpIf(String),         // Label or address
    JumpIfNot(String),      // Label or address
    Call(u32, u16),         // module_id, function_index
    Return,
    Halt,
    Yield,
    Emit(u16),              // event_id

    // Memory
    Load(u32),              // address
    Store(u32),             // address
    Alloc(u32),             // size
    Free,
    Copy { dst: u32, src: u32, len: u32 },
    Set { addr: u32, value: u8 },
    Get(u32),               // address

    // Agent-Semantic
    VecPull(String),        // hash or label
    VecPush(String),        // hash or label
    ProbB { threshold: f32, offset: String }, // threshold and jump target
    SnapS,
    SnapR(u32),             // snapshot ID
    ToolX { cap_token: u32, tool_id: u16 },

    // Directives
    Module(u32),
    Ports { inputs: u8, outputs: u8 },
    Input(Vec<super::value::Value>),
    Output(Vec<super::value::Value>),
    Data(Vec<DataItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataItem {
    Bytes { label: Option<String>, data: Vec<u8> },
    Quad { label: Option<String>, value: i64 },
    Float { label: Option<String>, value: f64 },
    Align(u8),
}

pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let lexer = Token::lexer(source);
        let tokens: Vec<Token> = lexer.filter(|t| !matches!(t, Token::Whitespace | Token::Comment)).collect();
        Self { lexer, tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, String> {
        let mut instructions = Vec::new();

        while self.pos < self.tokens.len() {
            let instr = self.parse_instruction()?;
            instructions.push(instr);
        }

        Ok(instructions)
    }

    fn parse_instruction(&mut self) -> Result<Instruction, String> {
        // Skip any leading newlines or semicolons (should have been filtered)
        match self.current() {
            Some(Token::Identifier(name)) => {
                // Could be a label or an instruction
                let name = name.clone();
                self.next(); // consume identifier

                // Check if followed by colon (label)
                if self.current() == Some(&Token::Colon) {
                    // This is a label - we'll handle labels in a second pass
                    // For now, just consume the colon and continue
                    self.next(); // consume colon
                    // Labels are handled during code generation
                    // For parsing, we treat them as zero-width instructions
                    continue;
                } else {
                    // This is an instruction mnemonic
                    self.parse_mnemonic(&name)
                }
            }
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_mnemonic(&mut self, mnemonic: &str) -> Result<Instruction, String> {
        match mnemonic.to_uppercase().as_str() {
            // Core
            "NOP" => Ok(Instruction::Nop),
            "PUSH_I64" => Ok(Instruction::PushI64(self.parse_immediate_i64()?)),
            "PUSH_F64" => Ok(Instruction::PushF64(self.parse_immediate_f64()?)),
            "PUSH_BYTES" => Ok(Instruction::PushBytes(self.parse_immediate_bytes()?)),
            "POP" => Ok(Instruction::Pop),
            "DUP" => Ok(Instruction::Dup),
            "SWAP" => Ok(Instruction::Swap),
            "PUSH_NIL" => Ok(Instruction::PushNil),

            // Arithmetic
            "ADD" => Ok(Instruction::Add),
            "SUB" => Ok(Instruction::Sub),
            "MUL" => Ok(Instruction::Mul),
            "DIV" => Ok(Instruction::Div),
            "MOD" => Ok(Instruction::Mod),
            "NEG" => Ok(Instruction::Neg),

            // Comparison
            "EQ" => Ok(Instruction::Eq),
            "NE" => Ok(Instruction::Ne),
            "LT" => Ok(Instruction::Lt),
            "LE" => Ok(Instruction::Le),
            "GT" => Ok(Instruction::Gt),
            "GE" => Ok(Instruction::Ge),

            // Logic
            "AND" => Ok(Instruction::And),
            "OR" => Ok(Instruction::Or),
            "XOR" => Ok(Instruction::Xor),
            "NOT" => Ok(Instruction::Not),
            "BOOL_AND" => Ok(Instruction::BoolAnd),
            "BOOL_OR" => Ok(Instruction::BoolOr),
            "BOOL_XOR" => Ok(Instruction::BoolXor),
            "BOOL_NOT" => Ok(Instruction::BoolNot),

            // Control Flow
            "JUMP" => Ok(Instruction::Jump(self.parse_label_or_address()?)),
            "JUMP_IF" => Ok(Instruction::JumpIf(self.parse_label_or_address()?)),
            "JUMP_IF_NOT" => Ok(Instruction::JumpIfNot(self.parse_label_or_address()?)),
            "CALL" => {
                let module = self.parse_immediate_u32()?;
                let function = self.parse_immediate_u16()?;
                Ok(Instruction::Call(module, function))
            }
            "RETURN" => Ok(Instruction::Return),
            "HALT" => Ok(Instruction::Halt),
            "YIELD" => Ok(Instruction::Yield),
            "EMIT" => Ok(Instruction::Emit(self.parse_immediate_u16()?)),

            // Memory
            "LOAD" => Ok(Instruction::Load(self.parse_immediate_u32()?)),
            "STORE" => Ok(Instruction::Store(self.parse_immediate_u32()?)),
            "ALLOC" => Ok(Instruction::Alloc(self.parse_immediate_u32()?)),
            "FREE" => Ok(Instruction::Free),
            "COPY" => {
                let dst = self.parse_immediate_u32()?;
                let src = self.parse_immediate_u32()?;
                let len = self.parse_immediate_u32()?;
                Ok(Instruction::Copy { dst, src, len })
            }
            "SET" => {
                let addr = self.parse_immediate_u32()?;
                let value = self.parse_immediate_u8()?;
                Ok(Instruction::Set { addr, value })
            }
            "GET" => Ok(Instruction::Get(self.parse_immediate_u32()?)),

            // Agent-Semantic
            "VEC_PULL" => Ok(Instruction::VecPull(self.parse_identifier_or_string()?)),
            "VEC_PUSH" => Ok(Instruction::VecPush(self.parse_identifier_or_string()?)),
            "PROB_B" => {
                let threshold = self.parse_immediate_f32()?;
                let offset = self.parse_label_or_address()?;
                Ok(Instruction::ProbB { threshold, offset })
            }
            "SNAP_S" => Ok(Instruction::SnapS),
            "SNAP_R" => Ok(Instruction::SnapR(self.parse_immediate_u32()?)),
            "TOOL_X" => {
                let cap_token = self.parse_immediate_u32()?;
                let tool_id = self.parse_immediate_u16()?;
                Ok(Instruction::ToolX { cap_token, tool_id })
            }

            // Directives
            ".MODULE" => Ok(Instruction::Module(self.parse_immediate_u32()?)),
            ".PORTS" => {
                let inputs = self.parse_immediate_u8()?;
                let outputs = self.parse_immediate_u8()?;
                Ok(Instruction::Ports { inputs, outputs })
            }
            ".INPUT" => Ok(Instruction::Input(self.parse_port_types()?)),
            ".OUTPUT" => Ok(Instruction::Output(self.parse_port_types()?)),
            ".DATA" => Ok(Instruction::Data(self.parse_data_section()?)),
            _ => Err(format!("Unknown mnemonic: {}", mnemonic)),
        }
    }

    // Helper parsing methods
    fn parse_immediate_i64(&mut self) -> Result<i64, String> {
        match self.next() {
            Some(Token::DecimalInteger(Some(v))) => Ok(v),
            Some(Token::HexInteger(Some(v))) => Ok(v),
            Some(Token::BinaryInteger(Some(v))) => Ok(v),
            Some(token) => Err(format!("Expected integer, got {:?}", token)),
            None => Err("Unexpected end of input while parsing integer"),
        }
    }

    fn parse_immediate_u32(&mut self) -> Result<u32, String> {
        self.parse_immediate_i64().map(|v| v as u32)
    }

    fn parse_immediate_u16(&mut self) -> Result<u16, String> {
        self.parse_immediate_i64().map(|v| v as u16)
    }

    fn parse_immediate_u8(&mut self) -> Result<u8, String> {
        self.parse_immediate_i64().map(|v| v as u8)
    }

    fn parse_immediate_f64(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Float(Some(v))) => Ok(v),
            Some(Token::ScientificFloat(Some(v))) => Ok(v),
            Some(token) => Err(format!("Expected float, got {:?}", token)),
            None => Err("Unexpected end of input while parsing float"),
        }
    }

    fn parse_immediate_f32(&mut self) -> Result<f32, String> {
        self.parse_immediate_f64().map(|v| v as f32)
    }

    fn parse_immediate_bytes(&mut self) -> Result<Vec<u8>, String> {
        match self.next() {
            Some(Token::QuoteBegin) => self.parse_string_literal(),
            Some(Token::Identifier(name)) if name.starts_with("0x") => {
                self.parse_hex_bytes(&name[2..])
            }
            Some(token) => Err(format!("Expected byte array, got {:?}", token)),
            None => Err("Unexpected end of input while parsing byte array"),
        }
    }

    fn parse_string_literal(&mut self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut escaped = false;

        loop {
            match self.next() {
                Some(Token::QuoteEnd) => break,
                Some(Token::Identifier(ch)) if escaped => {
                    match ch.as_str() {
                        "n" => result.push(b'\n'),
                        "t" => result.push(b'\t'),
                        "r" => result.push(b'\r'),
                        "\\" => result.push(b'\\'),
                        "\"" => result.push(b'"'),
                        _ => return Err(format!("Unknown escape sequence: \\{}", ch)),
                    }
                    escaped = false;
                }
                Some(Token::Identifier(ch)) if ch == "\\" => {
                    escaped = true;
                }
                Some(Token::Identifier(ch)) => {
                    if escaped {
                        return Err(format!("Lone backslash in string"));
                    }
                    // For simplicity, we'll treat single characters as their ASCII value
                    // In a full implementation, we'd handle multi-character strings properly
                    if ch.len() == 1 {
                        result.push(ch.as_bytes()[0]);
                    } else {
                        return Err(format!("Multi-character literals not supported in strings"));
                    }
                    escaped = false;
                }
                Some(token) => return Err(format!("Unexpected token in string: {:?}", token)),
                None => return Err("Unterminated string literal"),
            }
        }

        Ok(result)
    }

    fn parse_hex_bytes(&mut self, hex: &str) -> Result<Vec<u8>, String> {
        if hex.len() % 2 != 0 {
            return Err(format!("Hex string must have even length: {}", hex));
        }

        let mut bytes = Vec::new();
        for chunk in hex.as_bytes().chunks(2) {
            let byte = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16)
                .map_err(|_| format!("Invalid hex byte: {}", std::str::from_utf8(chunk).unwrap()))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    fn parse_label_or_address(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Identifier(name)) => Ok(name),
            Some(token) => Err(format!("Expected label or address, got {:?}", token)),
            None => Err("Unexpected end of input while parsing label/address"),
        }
    }

    fn parse_identifier_or_string(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Identifier(name)) => Ok(name),
            Some(Token::QuoteBegin) => {
                let result = self.parse_string_literal()?;
                // Convert bytes to string (lossy)
                Ok(String::from_utf8_lossy(&result).to_string())
            }
            Some(token) => Err(format!("Expected identifier or string, got {:?}", token)),
            None => Err("Unexpected end of input while parsing identifier/string"),
        }
    }

    fn parse_port_types(&mut self) -> Result<Vec<super::value::Value>, String> {
        let mut types = Vec::new();
        loop {
            match self.current() {
                Some(Token::Identifier(name)) => {
                    let ty = match name.to_uppercase().as_str() {
                        "I64" => Ok(super::value::Value::I64(0)), // Placeholder value
                        "F64" => Ok(super::value::Value::F64(0.0)),
                        "BYTES" => Ok(super::value::Value::Bytes(vec![])),
                        "NIL" => Ok(super::value::Value::Nil),
                        _ => Err(format!("Unknown type: {}", name)),
                    }?;
                    types.push(ty);
                    self.next(); // consume the type identifier

                    // Check for comma
                    if self.current() == Some(&Token::Identifier(ref comma)) if comma == "," {
                        self.next(); // consume comma
                        continue;
                    } else {
                        break;
                    }
                }
                Some(token) => return Err(format!("Expected type, got {:?}", token)),
                None => break,
            }
        }
        Ok(types)
    }

    fn parse_data_section(&mut self) -> Result<Vec<DataItem>, String> {
        let mut items = Vec::new();
        loop {
            match self.current() {
                Some(Token::Identifier(label)) if label.ends_with(":") => {
                    // Label definition
                    let label_name = label[..label.len()-1].to_string();
                    self.next(); // consume label with colon

                    match self.current() {
                        Some(Token::Identifier(directive)) => {
                            let item = match directive.to_uppercase().as_str() {
                                ".BYTES" => {
                                    self.next(); // consume .BYTES
                                    DataItem::Bytes {
                                        label: Some(label_name),
                                        data: self.parse_immediate_bytes()?,
                                    }
                                }
                                ".QUAD" => {
                                    self.next(); // consume .QUAD
                                    DataItem::Quad {
                                        label: Some(label_name),
                                        value: self.parse_immediate_i64()?,
                                    }
                                }
                                ".FLOAT" => {
                                    self.next(); // consume .FLOAT
                                    DataItem::Float {
                                        label: Some(label_name),
                                        value: self.parse_immediate_f64()?,
                                    }
                                }
                                ".ALIGN" => {
                                    self.next(); // consume .ALIGN
                                    DataItem::Align(self.parse_immediate_u8()? as u8)
                                }
                                _ => return Err(format!("Unknown data directive: {}", directive)),
                            };
                            items.push(item);
                        }
                        _ => return Err(format!("Expected data directive after label {:?}", label)),
                    }
                }
                Some(Token::Identifier(directive)) => {
                    // Directive without label
                    let item = match directive.to_uppercase().as_str() {
                        ".BYTES" => {
                            self.next(); // consume .BYTES
                            DataItem::Bytes {
                                label: None,
                                data: self.parse_immediate_bytes()?,
                            }
                        }
                        ".QUAD" => {
                            self.next(); // consume .QUAD
                            DataItem::Quad {
                                label: None,
                                value: self.parse_immediate_i64()?,
                            }
                        }
                        ".FLOAT" => {
                            self.next(); // consume .FLOAT
                            DataItem::Float {
                                label: None,
                                value: self.parse_immediate_f64()?,
                            }
                        }
                        ".ALIGN" => {
                            self.next(); // consume .ALIGN
                            DataItem::Align(self.parse_immediate_u8()? as u8)
                        }
                        _ => return Err(format!("Unknown data directive: {}", directive)),
                    };
                    items.push(item);
                    self.next(); // consume the directive
                }
                Some(token) => return Err(format!("Unexpected token in data section: {:?}", token)),
                None => break,
            }
        }
        Ok(items)
    }
}