use std::collections::BTreeSet;
use crate::converter::{IDENT, LINE_COUNT};
use crate::function;
use crate::label::LabelType;
use crate::stack::{StackMem, StackPtr};

pub fn handle_call(op1: &str, stack_ptr: &mut Vec<StackPtr>, stack_mem: &mut BTreeSet<StackMem>, c_code_final: &mut Vec<String>, sequence: &mut Vec<String>, label: &Vec<LabelType>, asm_code: &[String], skip2: &mut bool) -> bool {
    if label.contains(&LabelType::Function(op1.to_string())) {
        c_code_final.push(unsafe {IDENT.clone()} + function::arguments::target_arg(stack_ptr, stack_mem.clone(), op1, asm_code).as_str() + ";");
        *skip2 = true;
        true
    } else {
        let mut types = "void".to_string();
        let rm = function::return_func::track_rax_after_call(asm_code[unsafe {LINE_COUNT}..].to_vec());
        if rm.0 {
            types = rm.1.get_expr_with_type();
        }
        let name_formatted = format!("(({}(*)()){})", types, sequence.join(""));
        c_code_final.push(format!("{}{};", unsafe {IDENT.clone()}, function::arguments::target_arg(stack_ptr, stack_mem.clone(), &name_formatted, asm_code)));
        *skip2 = true;
        true
    }
}