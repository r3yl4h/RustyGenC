use crate::converter::{ALL_REG, IDENT, LINE_STR, OPERATORS};
use std::collections::BTreeSet;
use crate::converter::{LINE_COUNT, OperandType};
use crate::{eval, print_msg, stack, types};
use crate::logs::*;


fn remove_reg_and_expr(s: String) -> (String, String) {
    let mut result = s;
    let mut removed_parts = String::new();
    for reg_group in ALL_REG.iter() {
        for &reg in reg_group.iter() {
            while let Some(pos) = result.find(reg) {
                let mut start_pos = pos;
                if start_pos > 0 && OPERATORS.contains(&(result.as_bytes()[start_pos - 1] as char)) {
                    start_pos -= 1;
                }
                let mut end_pos = pos + reg.len();
                if end_pos < result.len() && result.as_bytes()[end_pos] == b'*' {
                    end_pos += 1;
                    while end_pos < result.len() && result.as_bytes()[end_pos].is_ascii_digit() {
                        end_pos += 1;
                    }
                }
                removed_parts.push_str(&result[start_pos..end_pos]);
                removed_parts.push(' ');
                result.replace_range(start_pos..end_pos, "");
            }
        }
    }
    (result, removed_parts.trim().to_string())
}



pub fn handle_arthm_op(stack_ptr: &[stack::StackPtr], stack_mem: &BTreeSet<stack::StackMem>, opt: String, sequence: &mut Vec<String>, operand_type: (OperandType, types::Type), wait2type: &mut bool) {
    if let Some(st_ptr) = stack_ptr.iter().find(|st|opt.contains(&st.reg)) {
        match eval::calcule_st_addr(&opt.replace(&st_ptr.reg, &st_ptr.stack_addr_ptr.to_string())) {
            Ok(value) =>  {
                if let Some(st_var) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), value) {
                    let expr = types::get_expr_with_len(st_var.size);
                    if st_var.line_def == unsafe {LINE_COUNT - 1} {
                        if expr.contains("[") {
                            sequence.push(format!("uint8_t {}[{}]", st_var.name, st_var.size));
                        }else {
                            sequence.push(format!("{expr} {}", st_var.name))
                        }
                        if operand_type.1 != types::Type::UN && operand_type.1.get_size_with_type() != st_var.size {
                            sequence.push(format!(";\n{}*({}*)&{}", unsafe {IDENT.clone()}, operand_type.1.get_expr_with_type(), st_var.name))
                        }
                    }else {
                        if operand_type.1 != types::Type::UN && operand_type.1.get_size_with_type() != st_var.size {
                            sequence.push(format!("*({}*)&{}", operand_type.1.get_expr_with_type(), st_var.name))
                        }else {
                            if !types::valid_type_with_size(st_var.size) {
                                sequence.push(format!("*(wait*)&{}", st_var.name));
                                *wait2type = true;
                            }else {
                                sequence.push(st_var.name)
                            }
                        }
                    }
                }else {
                    if let Some(st_n) = stack::finder::find_nearest_one_less(stack_mem.clone(), value) {
                        let typ_o = if operand_type.1 != types::Type::UN {
                            operand_type.1.get_expr_with_type()
                        }else {
                            String::from("wait")
                        };
                        if unsafe { LINE_STR.split_whitespace().collect::<Vec<&str>>().first().unwrap() } == &"lea" {
                            sequence.push(format!("&((uint8_t*)&{})[{}]", st_n.name, value - st_n.addr))
                        }else {
                            sequence.push(format!("*({}*)((uint8_t*)&{})[{}]", typ_o, st_n.name, value - st_n.addr))
                        };
                    }
                }
            }
            Err(_) =>  {
                let (opt, removed) = remove_reg_and_expr(opt.replace(&st_ptr.reg, &st_ptr.stack_addr_ptr.to_string()));
                match eval::calcule_st_addr(&opt) {
                    Ok(addr_b) => {
                        if let Some(st_var) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr_b) {
                            let mut is = 1;
                            let mut opt2 = opt;
                            if let Some(indx_star) = removed.find("*") {
                                is = removed[indx_star..indx_star+1].parse().unwrap_or(1);
                                opt2 = removed[..indx_star].to_string();
                            }
                            let types_s = if operand_type.1 != types::Type::UN {
                                operand_type.1.get_expr_with_type()
                            }else {
                                String::from("wait")
                            };
                            sequence.push(format!("*({types_s}*)(({}*)&{})[{opt2}]", types::get_expr_with_len(is), st_var.name))
                        }
                    }
                    Err(e) => {
                        print_msg!(LogLevel::Error(Error::Arithmetic(Box::new(e))), "")
                    }
                }
            }
        }
    }else {
        if operand_type.1 != types::Type::UN {
            sequence.push(format!("*({}*)((uint8_t*){opt})", operand_type.1.get_expr_with_type()))
        }else {
            sequence.push(format!("*(wait*)((uint8_t*){opt})"));
            *wait2type = true;
        }
    }
}





pub fn handle_op_arthm2(stack_ptr: &[stack::StackPtr], stack_mem: &BTreeSet<stack::StackMem>, sequences: &mut Vec<String>,
                        sequence: &mut Vec<String>, operand_type: types::Type, type_op1: types::Type, mnemonic: &str, opt: String) -> bool {

    return if let Some(st_ptr) = stack_ptr.iter().find(|st_ptr| opt.contains(&st_ptr.reg)) {
        match eval::calcule_st_addr(&opt
            .replace(&st_ptr.reg, &st_ptr.stack_addr_ptr.to_string()).to_lowercase()
        ) {
            Ok(addr) => {
                if let Some(st_var) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr) {
                    sequences.push(format!("{}", st_var.name));
                    sequences.remove(0);
                    if mnemonic == "lea" {
                        if st_var.line_def == unsafe { LINE_COUNT - 1 } {
                            if !types::valid_type_with_size(st_var.size) {
                                sequence.insert(0, format!("uint8_t {}[{}];\n{}", st_var.name, st_var.size, unsafe { IDENT.clone() }))
                            } else {
                                sequence.insert(0, format!("{} {};\n{}", types::get_expr_with_len(st_var.size), st_var.name, unsafe { IDENT.clone() }))
                            }
                        }
                        sequences.insert(0, String::from('&'))
                    }
                    true
                } else {
                    if let Some(st_n) = stack::finder::find_nearest_one_less(stack_mem.clone(), addr) {
                        let typ_o = if operand_type != types::Type::UN {
                            operand_type.get_expr_with_type()
                        }else{
                            type_op1.get_expr_with_type()
                        };
                        sequence.push(format!("*({}*)(uint8_t*)&{}[{}]", typ_o, st_n.name, st_n.addr - addr));
                        true
                    }else {
                        false
                    }
                }
            }
            Err(e) => {
                let (opt, mut remove_str) = remove_reg_and_expr(opt.replace(&st_ptr.reg, &st_ptr.stack_addr_ptr.to_string()));
                match eval::calcule_st_addr(&opt) {
                    Ok(addr_b) => {
                        remove_str = remove_str[1..].to_string();
                        if let Some(st_var) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), addr_b) {
                            let mut mul_value = 1;
                            if let Some(indx_star) = remove_str.find("*") {
                                if remove_str.get(indx_star+1..indx_star+2).is_some() {
                                    mul_value = remove_str[indx_star+1..indx_star+2].parse().unwrap_or(1);
                                    remove_str = remove_str[..indx_star].to_string();
                                }else {
                                    print_msg!(LogLevel::Error(Error::Arithmetic(Box::new("there is no valid next value after the '*'"))), "");
                                }
                            }
                            if unsafe { *LINE_STR.split_whitespace().collect::<Vec<&str>>().first().unwrap() } == "lea" {
                                sequences.push(format!("&(({}*)&{})[{remove_str}]", types::get_expr_with_len(mul_value), st_var.name))
                            }else {
                                let types_s = if operand_type != types::Type::UN {
                                    operand_type
                                }else {
                                    type_op1
                                };
                                let types_is = types::get_type_with_size(mul_value);
                                if types_s == types_is {
                                    sequences.push(format!("(({}*)&{})[{remove_str}]", types_is.get_expr_with_type(), st_var.name))
                                }else {
                                    sequences.push(format!("({}*)&(({}*)&{})[{remove_str}]", types_s.get_expr_with_type(), types::get_expr_with_len(mul_value), st_var.name))
                                }
                            }
                            true
                        }else {
                            false
                        }
                    }
                    Err(_) => {
                        print_msg!(LogLevel::Error(Error::Arithmetic(Box::new(e))), "");
                        false
                    }
                }
            }
        }
    } else {
        let types_of = if operand_type != types::Type::UN {
            operand_type
        } else {
            type_op1
        };
        if types_of == types::Type::X8 {
            sequences.push(format!("(uint8_t*){opt}"));
        }else {
            sequences.push(format!("({}*)(uint8_t*){opt}", types_of.get_expr_with_type()));
        }
        true
    }
}