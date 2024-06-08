use std::collections::BTreeSet;
use crate::converter::*;
use crate::label::LabelType;
use crate::*;
pub mod convention;
pub mod arguments;
pub mod return_func;


#[derive(PartialEq, Debug, Clone)]
pub enum RegTrack {
    Addr(u64),
    Reg(String),
    Default
}



pub fn is_function(mnemonic: &str, c_code_final: &mut Vec<String>, stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: &mut BTreeSet<stack::StackMem>, asm_code: &[String]) -> bool {
    if mnemonic.contains(":") && unsafe { label::LABEL.contains(&LabelType::Function(mnemonic.replace(":", "")))} || mnemonic.replace(":", "") == "_start" {
        if let Some(ctx) = CONTEXT.get() {
            let mut context = ctx.lock().unwrap();
            if context.is_in_func {
                c_code_final.push("}".to_string());
            }
            *context = Context::default();
            context.name = mnemonic.replace(":", "");
            context.is_in_func = true;
            let mut types_func = "void".to_string();
            let return_func = return_func::func_have_return(asm_code[unsafe {LINE_COUNT}..].to_vec());
            if return_func.0 {
                context.types = return_func.1;
                types_func = return_func.1.get_expr_with_type();
            }
            let name = arguments::get_arg_expr_of_func(stack_ptr, stack_mem.clone(), &mnemonic.replace(":", ""), asm_code);
            c_code_final.push("\n".to_owned() + &types_func + " " + &name + "{");
            unsafe {IDENT = "    ".to_string()}
            stack::variable::get_var_in_stack_func(stack_mem, stack_ptr.clone(), asm_code, c_code_final);
            return true
        }
    }
    return false
}


fn process_operand_dest(operand: &str, reg_tracker: &mut Vec<RegTrack>, reg_suspect: &mut Vec<RegTrack>, stack_ptr: &[stack::StackPtr], stack_mem: &BTreeSet<stack::StackMem>) {
    if let Some((operand_type, _)) = types::get_type_operand(operand) {
        match operand_type {
            OperandType::Memory(mem) => match mem {
                MemoryType::Register(reg, _) => {
                    if !is_tracked(&types::_to64b(reg.to_string()), reg_tracker, stack_ptr) {
                        track_reg(&types::_to64b(reg.to_string()), reg_suspect);
                    }
                }
                MemoryType::Operator(opt_expr) => {
                    if let Some(reg) = find_reg_in_expr(&opt_expr) {
                        if !is_tracked(&types::_to64b(reg.to_string()), reg_tracker, stack_ptr) {
                            handle_operator_expr(&opt_expr, &types::_to64b(reg.to_string()), reg_tracker, stack_ptr, stack_mem);
                        }
                    }
                }
                _ => {}
            },
            OperandType::Register(reg) => {
                if !is_tracked(&types::_to64b(reg.clone()), reg_tracker, stack_ptr) {
                    track_reg(&types::_to64b(reg), reg_tracker);
                }
            }
            _ => {}
        }
    }
}


fn is_tracked(reg: &str, reg_tracker: &[RegTrack], stack_ptr: &[stack::StackPtr]) -> bool {
    reg_tracker.contains(&RegTrack::Reg(types::_to64b(reg.to_string()))) || stack_ptr.iter().any(|st| st.reg == types::_to64b(reg.to_string()))
}


fn find_reg_in_expr(expr: &str) -> Option<&str> {
    ALL_REG64.iter().find(|&&reg| expr.contains(reg)).map(|&reg| reg)
}



fn track_reg(reg: &str, reg_tracker: &mut Vec<RegTrack>) {
    reg_tracker.push(RegTrack::Reg(types::_to64b(reg.to_string())));
}


fn handle_operator_expr(opt_expr: &str, reg: &str, reg_tracker: &mut Vec<RegTrack>, stack_ptr: &[stack::StackPtr], stack_mem: &BTreeSet<stack::StackMem>) {
    if let Some(st) = stack_ptr.iter().find(|st| st.reg.contains(reg)) {
        let opt_expr = opt_expr.replace("(", "").replace(")", "").replace(reg, &st.stack_addr_ptr.to_string());
        if let Ok(addr) = eval::calcule_st_addr(&opt_expr) {
            if !reg_tracker.contains(&RegTrack::Addr(addr)) {
                if let Some(_) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr) {
                    reg_tracker.push(RegTrack::Addr(addr));
                }
            }
        }
    }
}


fn verify_ax(op: &str) -> (bool, types::Type){
    if let Some((op_types, types)) = types::get_type_operand(op) {
        match op_types {
            OperandType::Register(reg) => {
                if reg.ends_with("ax") {
                    return (true, types)
                }
            }
            OperandType::Memory(mem_type) => {
                match mem_type{
                    MemoryType::Register(reg, types) => {
                        if reg.ends_with("ax") {
                            return (true, types)
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (false, types::Type::UN)
}

