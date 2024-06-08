use crate::*;


pub fn get_op1_reg(sequence: &mut Vec<String>, stack_ptr: &mut Vec<stack::StackPtr>, reg: String, op2: &str,
                   mnemonic: &str, operand_type: (converter::OperandType, types::Type), temp: &mut bool){
    if let Some(st) = stack_ptr.iter_mut().find(|st| st.reg == reg) {
        let op_value;
        if op2.starts_with("0x") {
            op_value = u64::from_str_radix(&op2[2..], 16).unwrap_or(0);
        }
        else if op2.ends_with("h") {
            op_value = u64::from_str_radix(&op2[..op2.len()-1], 16).unwrap_or(0);
        }
        else if let Ok(n) = op2.chars().filter(|c| c.is_digit(10)).collect::<String>().parse::<u64>() {
            op_value = n;
        }
        else {
           return
        }
        match mnemonic {
            "add" => st.stack_addr_ptr += op_value,
            "sub" => st.stack_addr_ptr -= op_value,
            _ => {}
        }
        if st.stack_addr_ptr >= stack::HIGH_OF_STACK {
            print_msg!(LogLevel::Warning(Warning::BadStackPtrHigh), "")
        }
        *temp = true;
        return
    }else {
        let expr_type = types::get_expr_with_len(operand_type.1.get_size_with_type());
        if operand_type.1 != types::Type::X64 {
            sequence.push(format!("*({expr_type}*)&{}", types::_to64b(reg)));
        }else {
            sequence.push(reg.to_string());
        }
    }
}



pub fn get_op2_reg(stack_ptr: &mut Vec<stack::StackPtr>, reg: &str, op1: &str, types_op2: &mut types::Type, sequences: &mut Vec<String>, type_op: types::Type) -> bool {
    let type_reg = types::Type::get_type_of_reg(reg);
    if type_reg != types::Type::UN {
        *types_op2 = type_reg;
    }
    if !converter::cmp_bid(stack_ptr, reg, op1) {
        converter::push_reg_with_len(sequences, type_op, reg);
        return true
    }else {
        false
    }
}