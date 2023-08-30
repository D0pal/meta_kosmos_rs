#![allow(dead_code)]

use revm::interpreter::opcode;

#[derive(Debug, Clone)]
pub struct OpCode {
    name: String,
    code: u8,
}

impl OpCode {
    // creat a new opcode instance from numeric opcode
    //
    // Arguments:
    // * `code`: numberic opcode
    //
    // Returns:
    // `OpCode`: new opcode instance
    fn new_from_code(code: u8) -> Self {
        let name = match opcode::OPCODE_JUMPMAP[code as usize] {
            Some(name) => name.to_string(),
            None => "UNKNOWN".to_string(),
        };

        OpCode { code, name }
    }
}

