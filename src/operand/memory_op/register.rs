use std::collections::BTreeSet;
use crate::*;

pub fn get_op1_mem_reg(sequence: &mut Vec<String>, stack_ptr: &[stack::StackPtr], reg: &str, types_reg: types::Type, stack_mem: &mut BTreeSet<stack::StackMem>, operand_type: (converter::OperandType, types::Type)) {
    if let Some(st) = stack_ptr.iter().find(|st| st.reg == reg) {
        if let Some(sti) = stack::finder::get_elm_in_stack_with_addr(stack_mem.clone(), st.stack_addr_ptr) {
            sequence.push(sti.name.clone());
        } else if st.stack_addr_ptr >= stack::HIGH_OF_STACK{
            print_msg!(LogLevel::Error(Error::BadStackPtrHigh), "{} point to high of stack", st.reg);
            return
        }else {
            stack_mem.insert(stack::StackMem {
                addr: st.stack_addr_ptr,
                value: String::from("flemme"),
                size: if operand_type.1 != types::Type::UN {
                    operand_type.1.get_size_with_type()
                }else {
                    (stack_mem.last().unwrap().addr - st.stack_addr_ptr) as usize
                },
                name: "a".to_owned() + &(stack_mem.len() + 1).to_string(),
                line_def: 0
            });
            if operand_type.1 != types::Type::UN {
                sequence.push(format!("{} a{}", types::get_expr_with_len(operand_type.1.get_size_with_type()), stack_mem.len()));
            }else {
                sequence.push(format!("{} a{}", types::get_expr_with_len(stack_mem.last().unwrap().size), stack_mem.len()));
            }
        }
    } else {
        let expr_type = types::get_expr_with_len(types_reg.get_size_with_type());
        let reg_string = match types_reg {
            types::Type::X64 => reg.to_string(),
            _ => {
                let mut breg = reg.to_string();
                breg.replace_range(0..1, "r");
                format!("*({}*)&{}", expr_type, breg)
            }
        };

        if types_reg == types::Type::X64 && operand_type.1 != types::Type::UN {
            sequence.push(format!("({}*){}", types::get_expr_with_len(operand_type.1.get_size_with_type()), reg_string));
        } else {
            sequence.push(reg_string);
        }

        sequence.insert(0, "*".to_string());
    }
}








