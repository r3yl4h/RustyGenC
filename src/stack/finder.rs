use std::collections::BTreeSet;
use crate::stack;


pub fn get_elm_in_stack_with_addr(stack_mem: BTreeSet<stack::StackMem>, addr: u64) -> Option<stack::StackMem>{
    for var in stack_mem {
        if var.addr == addr {
            return Some(var)
        }
    }
    return None
}




pub fn fin_nearest_one_greate(stack_mem: BTreeSet<stack::StackMem>, addr: u64) -> Option<stack::StackMem> {
    let mut last_st = stack::StackMem::default();
    for var in stack_mem {
        if last_st.addr < addr && var.addr > addr {
            return Some(var)
        }
        last_st = var;
    }
    return None
}




pub fn find_nearest_one_less(stack_mem: BTreeSet<stack::StackMem>, addr: u64) -> Option<stack::StackMem> {
    let st_vec: Vec<stack::StackMem> = stack_mem.iter().cloned().collect();
    for (i, var) in stack_mem.iter().enumerate() {
        let mut temp_st = stack::StackMem::default();
        if i == st_vec.len() - 1 {
            temp_st.addr = addr + 1;
        }
        let proch_st = if i == st_vec.len() - 1 {
            &temp_st
        } else {
            &st_vec[i + 1]
        };
        if var.addr < addr && proch_st.addr > addr {
            return Some(var.clone())
        }
    }
    return None
}