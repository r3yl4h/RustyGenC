use std::collections::BTreeSet;
use crate::converter::*;
use crate::*;
use crate::label::Label;

pub fn target_arg(_stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: BTreeSet<stack::StackMem>, name_func: &str, asm_code: &[String]) -> String{
    let mut name_expr = format!("{}(", name_func);
    let mut reg_suspect = Vec::new();
    function::convention::detect_reg_with_convention(&mut reg_suspect, asm_code);
    function::convention::remove_reg_is_not_call_conv(&mut reg_suspect);
    for elm in reg_suspect {
        match elm {
            function::RegTrack::Reg(reg) => {
                name_expr.push_str(&(reg + ", "));
            }
            function::RegTrack::Addr(addr) => {
                name_expr.push_str(&(stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr).unwrap().name + ", "))
            }
            _ => {}
        }
    }
    if name_expr.ends_with(", ") {
        name_expr.truncate(name_expr.len()-2)
    }
    let tr = function::return_func::track_rax_after_call(asm_code[unsafe {LINE_COUNT}..].to_vec());
    if tr.0 {
        if tr.1 != types::Type::X64{
            name_expr = format!("*({}*)&rax = {name_expr}", tr.1.get_expr_with_type());
        }else {
            name_expr = format!("rax = {name_expr}");
        }
    }
    name_expr.push(')');
    name_expr
}




pub fn get_arg_expr_of_func(stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: BTreeSet<stack::StackMem>, name_func: &str, asm_code: &[String]) -> String {
    let mut name_expr = format!("{}(", name_func);
    let mut reg_tracker = Vec::new();
    let mut reg_suspect = Vec::new();
    for line in &asm_code[unsafe{LINE_COUNT}..] {
        let tokens: Vec<&str> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        let line = line.replace(" ", "");
        let mnemonic = *tokens.get(0).unwrap_or(&"");
        if line.contains("ret")|| unsafe { label::LABEL.clone().contain_funcion(&mnemonic.replace(":", ""))}{
            break;
        }
        let (operand_dest, operand_source) = types::get_operands(&line, mnemonic.len());
        function::process_operand_dest(operand_dest, &mut reg_tracker, &mut reg_suspect, stack_ptr, &stack_mem);
        if mnemonic == "lea" {
            function::process_operand_dest(operand_source, &mut reg_tracker, &mut reg_suspect, stack_ptr, &stack_mem);
        }else {
            if let Some(op_t) = types::get_type_operand(operand_source) {
                match op_t.0 {
                    OperandType::Memory(mem) => match mem {
                        MemoryType::Register(reg, _) => {
                            if !function::is_tracked(reg, &*reg_tracker, stack_ptr) {
                                function::track_reg(reg, &mut reg_suspect);
                            }
                        }
                        MemoryType::Operator(opt_expr) => {
                            if let Some(reg) = function::find_reg_in_expr(&opt_expr) {
                                if !function::is_tracked(&types::_to64b(reg.to_string()), &reg_tracker, stack_ptr) {
                                    function::handle_operator_expr(&opt_expr, reg, &mut reg_suspect, stack_ptr, &stack_mem);
                                }
                            }
                        }
                        _ => {}
                    }
                    OperandType::Register(reg) => {
                        if !function::is_tracked(&types::_to64b(reg.to_string()), &*reg_tracker, stack_ptr) {
                            function::track_reg(&reg, &mut reg_suspect);
                        }
                    }
                    _=> {}
                }
            }
        }
    }
    function::convention::remove_reg_is_not_call_conv(&mut reg_suspect);
    for elm in reg_suspect {
        match elm {
            function::RegTrack::Reg(reg) => {
                name_expr.push_str(&format!("{} {}, ", types::Type::get_type_of_reg(&reg).get_expr_with_type(), reg));
            }
            function::RegTrack::Addr(addr) => {
                name_expr.push_str(&(stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr).unwrap().name + ", "))
            }
            _ => {}
        }
    }
    if name_expr.ends_with(", ") {
        name_expr.truncate(name_expr.len()-2)
    }
    name_expr.push(')');
    name_expr
}