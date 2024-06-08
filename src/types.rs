use regex::Regex;
use once_cell::sync::Lazy;
use crate::converter::*;
use crate::label;
use crate::label::Label;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    X64,
    X32,
    X16,
    X8,
    UN,
    CHAR(usize)
}

impl Default for Type {
    fn default() -> Self {
        Type::UN
    }
}

impl Type {
    pub fn get_size_with_type(self) -> usize {
        match self {
            Type::X64 => 8,
            Type::X32 => 4,
            Type::X16 => 2,
            Type::X8 => 1,
            Type::CHAR(size) => size,
            Type::UN => 0,
        }
    }

    pub fn get_expr_with_type(self) -> String {
        match self {
            Type::X64 => String::from("uint64_t"),
            Type::X32 => String::from("uint32_t"),
            Type::X16 => String::from("uint16_t"),
            Type::X8 => String::from("uint8_t"),
            Type::CHAR(len) => format!("uint8_t[{len}]"),
            Type::UN => String::from(""),
        }
    }

    pub fn get_type_of_reg(elm: &str) -> Type {
        if ALL_REG64.contains(&elm) {
            Type::X64
        } else if ALL_REG32.contains(&elm) {
            Type::X32
        } else if ALL_REG16.contains(&elm) {
            Type::X16
        } else if ALL_REG8_LOW.contains(&elm) {
            Type::X8
        } else {
            Type::UN
        }
    }

    pub fn get_type_of_mem(mem: &str) -> Type {
        if mem.contains("qword") {
            Type::X64
        } else if mem.contains("dword") {
            Type::X32
        } else if mem.contains("word") {
            Type::X16
        } else if mem.contains("byte") {
            Type::X8
        } else {
            Type::UN
        }
    }

    pub fn get_arch_with_type(self) -> usize{
        return if self.get_size_with_type() == 1 {
            8
        }else if self.get_size_with_type() == 2 {
            16
        }else if self.get_size_with_type() == 4 {
            32
        }else if self.get_size_with_type() == 8 {
            64
        }else {
            0
        }
    }
}





pub fn get_expr_with_len(len: usize) -> String {
    match len {
        8 => String::from("uint64_t"),
        4 => String::from("uint32_t"),
        2 => String::from("uint16_t"),
        1 => String::from("uint8_t"),
        0 => String::from(""),
        _ => format!("uint8_t[{len}]"),
    }
}



pub fn get_type_operand(operand: &str) -> Option<(OperandType, Type)> {
    let regx = Lazy::new(|| Regex::new(r"\[(.*?)]").unwrap());
    if operand.contains('[') {
        if let Some(captures) = regx.captures(operand) {
            let op_content = captures.get(1).unwrap().as_str().trim();
            let type_op = Type::get_type_of_reg(op_content);
            let op = match type_op {
                Type::UN => {
                    unsafe {
                        if OPERATORS.iter().any(|&op| op_content.contains(op)) {
                            MemoryType::Operator(op_content.to_string())
                        } else if is_immediate_op(op_content) {
                            MemoryType::Immediate(op_content)
                        } else if label::LABEL.clone().contains_with_name(op_content) {
                            MemoryType::Label(op_content.to_string())
                        }else {
                            return None
                        }
                    }
                }
                _ => MemoryType::Register(op_content, type_op),
            };
            return Some((OperandType::Memory(op), Type::get_type_of_mem(operand)));
        }
    } else {
        let types = Type::get_type_of_reg(operand);
        if types != Type::UN {
            return Some((OperandType::Register(operand.to_string()), types));
        } else if is_immediate_op(operand) {
            let mut op_im = operand.to_string();
            if operand.ends_with("h") {
                op_im.pop();
                op_im = format!("0x{op_im}");
            }
            return Some((OperandType::Immediate(op_im), Type::UN));
        }else if unsafe { label::LABEL.clone().contains_with_name(operand)} {
            return Some((OperandType::Label(String::from(operand)), Type::UN));
        }
    }
    None
}




fn is_immediate_op(operand: &str) -> bool {
    let operand = operand.trim();
    operand.contains('\'') || operand.starts_with("0x") || operand.ends_with('h') || operand.parse::<i64>().is_ok()
}




pub fn usigned_to_signed(sequences: &mut Vec<String>) {
    for str in sequences {
        if str.contains("uint") {
            *str = str.replace("uint", "int")
        }
    }
}


fn contains_numbers(s: &str) -> bool {
    for c in s.chars() {
        if c.is_digit(10) {
            return true;
        }
    }
    false
}


fn target_number_in_reg(s: String) -> u8 {
    let mut count = 0;
    for c in s.chars() {
        if c.is_digit(10) {
            count += 1;
        }
    }
    count
}



pub fn _to64b(reg: String) -> String {
    let mut reg = reg;
    if reg.len() == 3 {
        if contains_numbers(&reg) {
            let nir = target_number_in_reg(reg.clone());
            if nir == 1 {
                reg.replace_range(2..3, "");
            }else if nir == 0 {
                if let Some(first_c) = reg.chars().collect::<Vec<char>>().first() {
                    if first_c != &'r' {
                        reg.replace_range(0..1, "r");
                    }
                }
            }
        }else {
            reg.replace_range(0..1, "r");
        }
    }
    else if reg.len() == 2 {
        if contains_numbers(&reg) {
            reg.pop();
        }else {
            reg.insert(0, 'r')
        }
    }
    return reg
}



pub fn get_operands(line: &str, mnemonic_len: usize) -> (&str, &str) {
    let indx_v = line.find(",").unwrap_or(0);
    if indx_v != 0 {
        (&line[mnemonic_len..indx_v], &line[indx_v + 1..])
    } else {
        (&line[mnemonic_len..], "")
    }
}


pub fn valid_type_with_size(size: usize) -> bool{
    match size {
        8 | 4 | 2 | 1 => true,
        _ => false
    }
}




pub fn get_type_with_size(size: usize) -> Type {
    match size {
        8 => Type::X64,
        4 => Type::X32,
        2 => Type::X16,
        1 => Type::X8,
        0 => Type::UN,
        _ => Type::CHAR(size),
    }
}




