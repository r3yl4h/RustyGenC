use std::collections::BTreeSet;
use crate::ARCH;
use crate::converter::IDENT;
use crate::stack;

pub fn handle_pop(stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: &mut BTreeSet<stack::StackMem>, op: &str, c_code_final: &mut Vec<String>, skip: &mut bool) -> bool {
    if let Some(st_ptr) = stack_ptr.iter_mut().find(|st_ptr| st_ptr.reg == "rsp") {
        if !op.ends_with("bp") {
            if let Some(closed_stack) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), st_ptr.stack_addr_ptr) {
                c_code_final.push(format!("{}{op} = {};", unsafe {IDENT.clone()}, closed_stack.name))
            }
        }
        st_ptr.stack_addr_ptr += unsafe { ARCH.get_size_with_type() } as u64;
        *skip = true;
    }
    true
}