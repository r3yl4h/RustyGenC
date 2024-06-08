use crate::converter::LINE_STR;
use crate::print_msg;
use crate::logs::*;

pub fn handle_mov_movzx_lea(op2: &str, sequence: &mut Vec<String>) -> bool {
    if op2.is_empty() {
        print_msg!(LogLevel::Error(Error::NotOp), "instruction: {}", unsafe {LINE_STR.clone()});
        return false;
    }
    sequence.push(" = ".to_string());
    true
}
