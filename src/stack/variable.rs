use std::collections::BTreeSet;
use crate::*;
use crate::label::Label;

pub fn get_var_in_stack_func(stack_mem: &mut BTreeSet<stack::StackMem>, stack_ptrs: Vec<stack::StackPtr>, func_code: &[String], c_code: &mut Vec<String>) {
    let mut stack_ptr = stack_ptrs;
    let mut abs_offset = 0;
    let func_code = &func_code[unsafe { converter::LINE_COUNT }..];
    let mut reg_tr = Vec::new();
    for (i, line) in func_code.iter().enumerate(){
        let liner: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        if liner.iter().any(|w|label::INSN_INIT.contains(w)) {
            continue
        }
        let mnemonic = *liner.get(0).unwrap_or(&"");
        if mnemonic.contains(":") && unsafe { label::LABEL.clone().contain_funcion(&mnemonic.replace(":", ""))} {
            break
        }else if mnemonic.contains(":") {
            continue
        }
        let line = line.replace(" ", "");
        let (op1, op2) = types::get_operands(&line, mnemonic.len());

        match mnemonic {
            "push" => {
                stack_mem.insert(stack::StackMem {
                    addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                    name: format!("st_{}", stack_mem.len()+1),
                    value: op1.to_string(),
                    size: unsafe {ARCH.get_size_with_type()},
                    line_def: i
                });
                if let Some(rsp_ptr) = stack_ptr.iter_mut().find(|st|st.reg.ends_with("sp")) {
                    rsp_ptr.stack_addr_ptr -= unsafe {ARCH.get_size_with_type() as u64};
                }
            },
            "pop" => {
                if let Some(rsp_ptr) = stack_ptr.iter_mut().find(|st|st.reg.ends_with("sp")) {
                    rsp_ptr.stack_addr_ptr += unsafe {ARCH.get_size_with_type() as u64};
                }
            }
            "cmp" | "test" => continue,
            _ => {
                if stack::init_st(&mut stack_ptr, stack_mem, &mut abs_offset, mnemonic, op1, op2) {
                    continue
                }
            }
        }


        if let Some(op_type) = types::get_type_operand(op1){
            let ops = op_type.clone();
            match op_type.0 {
                converter::OperandType::Register(reg) => {
                    if !reg_tr.contains(&types::_to64b(reg.clone())) && !stack_ptr.iter().any(|st_ptr|st_ptr.reg == reg){
                        reg_tr.push(types::_to64b(reg.clone()));
                    }
                    if reg != "" {
                        if let Some(st_ptr) = stack_ptr.iter_mut().find(|st_ptr|st_ptr.reg == reg) {
                            let mut op_value = 0;
                            if op2.starts_with("0x") {
                                op_value = u64::from_str_radix(&op2[2..], 16).unwrap_or(0);
                            }
                            else if op2.ends_with("h") {
                                op_value = u64::from_str_radix(&op2[..op2.len()-1], 16).unwrap_or(0);
                            }
                            else if let Ok(n) = op2.chars().filter(|c| c.is_digit(10)).collect::<String>().parse::<u64>() {
                                op_value = n;
                            }
                            match mnemonic {
                                "add" => st_ptr.stack_addr_ptr += op_value,
                                "sub" => st_ptr.stack_addr_ptr -= op_value,
                                _ => {}
                            }
                        }
                    }
                }
                converter::OperandType::Memory(mem) => {
                    match mem {
                        converter::MemoryType::Register(reg, _) => stack::handle_memory_register_case(reg, op2, &mut stack_ptr, stack_mem, i, ops),
                        converter::MemoryType::Operator(opt) => stack::handle_memory_operator_case(&opt, op2, &mut stack_ptr, stack_mem, i, ops),
                        _ => {},
                    }
                }
                _ => {}
            }
        }

        if let Some(op_type) = types::get_type_operand(op2){
            let ops = op_type.clone();
            match op_type.0 {
                converter::OperandType::Memory(mem) => {
                    match mem {
                        converter::MemoryType::Register(reg, _) => stack::handle_memory_register_case(reg, op2, &mut stack_ptr, stack_mem, i, ops),
                        converter::MemoryType::Operator(opt) => {
                            if let Some(st_reg) = stack_ptr.iter().find(|st|opt.contains(&st.reg)) {
                                match eval::calcule_st_addr(&opt.replace(&st_reg.reg, &*st_reg.stack_addr_ptr.to_string())
                                    .replace(")", "").replace("(", "")
                                ) {
                                    Ok(value) => {
                                        if let Some(_) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), value) {
                                        }else if !stack_mem.iter().any(|s|s.addr == value) {
                                            little_push(stack_mem, value, op2, i, op_type.1)
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                        _ => {},
                    }
                }
                _ => {}
            }
        }
    }
    fix_size_of_var(stack_mem, func_code);
    for reg in reg_tr { c_code.push(format!("{}{} {reg};", unsafe { converter::IDENT.clone() }, unsafe {ARCH.get_expr_with_type()})) }
}






fn fix_size_of_var(stack_mem: &mut BTreeSet<stack::StackMem>, func_code: &[String]) {
    let mut stack_mem_vec: Vec<_> = stack_mem.iter().cloned().collect();
    stack_mem_vec.sort_by(|a, b| b.addr.cmp(&a.addr));
    if let Some(mut last_st) = stack_mem_vec.first().cloned() {
        for st_mem in &mut stack_mem_vec {
            st_mem.size = (last_st.addr - st_mem.addr) as usize;
            if !types::valid_type_with_size(st_mem.size) {
                if let Some(line) = func_code.get(st_mem.line_def) {
                    let liner: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
                    let mnemonic = liner.get(0).unwrap();
                    let (_, op2) = types::get_operands(line, mnemonic.len());
                    if let Some((_, t)) = types::get_type_operand(op2) {
                        if t != types::Type::UN {
                            st_mem.size = t.get_size_with_type();
                        }
                    }
                }
            }
            st_mem.line_def += unsafe { converter::LINE_COUNT };
            last_st = st_mem.clone();
        }
    }
    *stack_mem = stack_mem_vec.into_iter().collect::<BTreeSet<_>>();
}







pub fn little_push(stack_mem: &mut BTreeSet<stack::StackMem>, addr: u64, op2: &str, i: usize, op_type: types::Type) {
    stack_mem.insert(stack::StackMem {
        addr,
        name: format!("st_{}", stack_mem.len()+1),
        line_def: i,
        value: op2.to_string(),
        size: {
            if op_type != types::Type::UN {
                op_type.get_size_with_type()
            }else {
                let size_reg = types::Type::get_type_of_reg(op2);
                if size_reg != types::Type::UN {
                    size_reg.get_size_with_type()
                }else {
                    if let Some(st_var2) = stack::finder::fin_nearest_one_greate(stack_mem.clone(), addr){
                        (st_var2.addr - addr) as usize
                    }else {
                        0
                    }
                }
            }
        }
    });
}