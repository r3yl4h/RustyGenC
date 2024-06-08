use std::collections::BTreeSet;
use crate::ARCH;
use crate::converter::IDENT;
use crate::stack;

pub fn handle_push(stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: BTreeSet<stack::StackMem>, c_code: &mut Vec<String>, op: &str, skip: &mut bool) -> bool {
    if let Some(rsp_ptr) = stack_ptr.iter_mut().find(|ptr|ptr.reg == "rsp") {
        rsp_ptr.stack_addr_ptr -= 8;
        if !op.ends_with("bp") {
            if let Some(var) = stack::finder::get_elm_in_stack_with_addr(stack_mem, rsp_ptr.stack_addr_ptr) {
                c_code.push(format!("{}{} {} = {op};", unsafe {IDENT.clone()}, unsafe {ARCH.get_expr_with_type()}, var.name))
            }
        }
    }
    *skip = true;
    true
}
