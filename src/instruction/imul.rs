use crate::converter::OperandType;
use crate::types::{Type, usigned_to_signed};

pub fn handle_imul(sequence: &mut Vec<String>, ops: (OperandType, Type)) -> bool {
    if sequence.iter().any(|w|w.contains("uint")){
        usigned_to_signed(sequence);
    }else {
        if ops.1 != Type::UN {
            sequence.insert(0, format!("*({}*)&", ops.1.get_expr_with_type().replace("u", "")))
        }else if Type::get_type_of_reg(&sequence.join("")) != Type::UN {
            sequence.insert(0, format!("*({}*)&", ops.1.get_expr_with_type().replace("u", "")));
        }
    }
    sequence.push("*=".to_string());
    true
}