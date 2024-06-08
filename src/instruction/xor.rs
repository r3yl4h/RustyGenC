use crate::converter::{IDENT, LINE_STR};
use crate::print_msg;
use crate::logs::*;

pub fn handle_xor(op1: &str, op2: &str, sequence: &mut Vec<String>, c_code_final: &mut Vec<String>, skip2: &mut bool) -> bool {
    if op1.is_empty() {
        print_msg!(LogLevel::Error(Error::NotOp), "instruction: \"{}\"", unsafe {LINE_STR.clone()});
        return true;
    }
    if op2 == op1 {
        sequence.push(" = 0;".to_string());
        c_code_final.push(unsafe {IDENT.clone()} + &sequence.join(""));
        *skip2 = true;
        return true;
    } else {
        sequence.push(" ^= ".to_string());
        true
    }
}
