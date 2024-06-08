use crate::converter::Insn;
use crate::{converter, types};

pub fn get_expr_with_conditional_cmp(insn: &str, last_insn: Insn, sequence: &mut Vec<String>, last_sequence: Vec<String>) {
    let last_join = last_sequence.join("");
    let (op1, op2) = types::get_operands(&last_join, last_sequence.first().unwrap().len());
    let (op1t, op2t) = (get_num_of(&op1), get_num_of(&op2));
    let (op1, op2) = (last_insn.op1, last_insn.op2);
    match insn {
        "je" |"cmove" |"jz" |"cmovz" => *sequence = vec![format!("{op1} == {op2}")],
        "jne"|"cmovne" | "jnz"|"cmovnz" => *sequence = vec![format!("{op1} != {op2}")],
        "js" |"cmovs" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jns"|"cmovns"=> *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "jg" |"cmovg" => *sequence = vec![format!("(int{op1t}_t){op1} >= (int{op2t}_t){op2}")],
        "jge" |"cmovge" => *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "jng" |"cmovng" => *sequence = vec![format!("(int{op1t}_t){op1} <= (int{op2t}_t){op2}")],
        "jnge" |"cmovnge" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jl" |"cmovl" => *sequence = vec![format!("(int{op1t}_t){op1} <= (int{op2t}_t){op2}")],
        "jle" |"cmovle" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jnl" |"cmovnl"  => *sequence = vec![format!("(int{op1t}_t){op1} >= (int{op2t}_t){op2}")],
        "jnle"|"cmovnle" => *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "ja" | "cmova" => *sequence = vec![format!("(uint{op1t}_t){op1} > (uint{op2t}_t){op2}")],
        "jae" |"cmovae" => *sequence = vec![format!("(uint{op1t}_t){op1} >= (uint{op2t}_t){op2}")],
        "jna" |"cmovna" => *sequence = vec![format!("(uint{op1t}_t){op1} <= (uint{op2t}_t){op2}")],
        "jnae"|"cmovnae" => *sequence = vec![format!("(uint{op1t}_t){op1} < (uint{op2t}_t){op2}")],
        "jb" |"cmovb" => *sequence = vec![format!("(uint{op1t}_t){op1} < (uint{op2t}_t){op2}")],
        "jbe" |"cmovbe" => *sequence = vec![format!("(uint{op1t}_t){op1} <= (uint{op2t}_t){op2}")],
        "jnb" |"cmovnb" => *sequence = vec![format!("(uint{op1t}_t){op1} > (uint{op2t}_t){op2}")],
        "jnbe" |"cmovnbe" => *sequence = vec![format!("(uint{op1t}_t){op1} >= (uint{op2t}_t){op2}")],
        _ => {}
    }
}



fn get_num_of(op: &str) -> usize{
    if let Some((op_type, types)) = types::get_type_operand(op) {
        if types != types::Type::UN {
            types.get_arch_with_type()
        } else {
            match op_type {
                converter::OperandType::Register(reg) => types::Type::get_type_of_reg(&reg).get_arch_with_type(),
                converter::OperandType::Immediate(_) => 32,
                _ => 0
            }
        }
    } else {
        0
    }
}




pub fn get_expr_with_conditional_test(insn: &str, sequence: &mut Vec<String>, last_insn: Insn, last_sequence: Vec<String>) {
    let last_join = last_sequence.join("");
    let (op1, op2) = types::get_operands(&last_join, last_sequence.first().unwrap().len());
    let (op1t, op2t) = (get_num_of(&op1), get_num_of(&op2));
    let (op1, op2) = (last_insn.op1, last_insn.op2);
    let op_et = format!("({op1} & {op2})");
    match insn {
        "je" |"cmove" |"jz" |"cmovz"  => *sequence = vec![format!("{op_et} == 0")],
        "jne"|"cmovne"| "jnz"|"cmovnz"  => *sequence = vec![format!("{op_et} != 0")],
        "js" |"cmovs" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jns"|"cmovns"=> *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "jg" |"cmovg" => *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "jge" |"cmovge" => *sequence = vec![format!("(int{op1t}_t){op1} >= (int{op2t}_t){op2}")],
        "jng" |"cmovng" => *sequence = vec![format!("(int{op1t}_t){op1} <= (int{op2t}_t){op2}")],
        "jnge" |"cmovnge" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jl" | "cmovl" => *sequence = vec![format!("(int{op1t}_t){op1} <= (int{op2t}_t){op2}")],
        "jle" |"cmovle" => *sequence = vec![format!("(int{op1t}_t){op1} < (int{op2t}_t){op2}")],
        "jnl" |"cmovnl" => *sequence = vec![format!("(int{op1t}_t){op1} >= (int{op2t}_t){op2}")],
        "jnle"|"cmovnle" => *sequence = vec![format!("(int{op1t}_t){op1} > (int{op2t}_t){op2}")],
        "ja" | "cmova" => *sequence = vec![format!("(uint{op1t}_t){op1} >= (uint{op2t}_t){op2}")],
        "jae" |"cmovae" => *sequence = vec![format!("(uint{op1t}_t){op1} > (uint{op2t}_t){op2}")],
        "jna" |"cmovna" => *sequence = vec![format!("(uint{op1t}_t){op1} <= (uint{op2t}_t){op2}")],
        "jnae"|"cmovnae" => *sequence = vec![format!("(uint{op1t}_t){op1} < (uint{op2t}_t){op2}")],
        "jb" |"cmovb" => *sequence = vec![format!("(uint{op1t}_t){op1} <= (uint{op2t}_t){op2}")],
        "jbe" |"cmovbe" => *sequence = vec![format!("(uint{op1t}_t){op1} < (uint{op2t}_t){op2}")],
        "jnb" |"cmovnb" => *sequence = vec![format!("(uint{op1t}_t){op1} >= (uint{op2t}_t){op2}")],
        "jnbe" |"cmovnbe" => *sequence = vec![format!("(uint{op1t}_t){op1} > (uint{op2t}_t){op2}")],
        _ => {}
    }
}


