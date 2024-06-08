use crate::{converter, types};
use crate::types::Type;

pub fn handle_idiv(sequence: &mut Vec<String>, op2: &str, ops: (converter::OperandType, Type), skip2: &mut bool) -> bool{
    let op_actually = sequence.join("");
    if sequence.iter().any(|w|w.contains("uint")) {
        types::usigned_to_signed(sequence)
    }else if ops.1 != Type::UN {
        sequence.insert(0, format!("*({}*)&", ops.1.get_expr_with_type().replace("u", "")))
    }
    sequence.push(String::from("/"));
    if op2 == "" {
        sequence.push("rax".to_string());
        sequence.insert(0, "rax =".to_string());
        sequence.push(format!(" \n{}rdx = {op_actually} % rax;", unsafe {converter::IDENT.clone()}));
        *skip2 = true;
    }
    return true
}