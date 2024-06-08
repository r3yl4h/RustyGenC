use regex::Regex;
use crate::*;
use anyhow::{anyhow, Result};

pub static mut LABEL: Vec<LabelType> = Vec::new();

pub const INSN_INIT: [&str;8] = ["db", "dw", "dd", "dq", "resb", "resw", "resd", "resq"];
pub const ALLOC_INSN: [&str;4] = ["resb", "resw", "resd", "resq"];


#[derive(Clone, Default, Debug, PartialEq)]
pub struct Var {
    pub name: String,
    pub value: String,
    pub expr: VarDet,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct VarDet {
    pub types_expr: String,
    pub size: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub enum LabelType {
    Function(String),
    LJump(LJump),
    Variable(Var),
}


pub trait Label {
    fn contain_funcion(self, element: &str) -> bool;
    fn get_variable_with_name(self, name: &str) -> Option<Var>;
    fn contains_with_name(self, name: &str) -> bool;
}



impl Label for Vec<LabelType> {
    fn contain_funcion(self, element: &str) -> bool {
        for label in self {
            match label {
                LabelType::Function(name) => {
                    if name == element { return true }
                }
                _=> {}
            }
        }
        false
    }

    fn get_variable_with_name(self, name: &str) -> Option<Var> {
        for label in self {
            match label {
                LabelType::Variable(var) => {
                    if var.name == name {
                        return Some(var)
                    }
                }
                _=> {}
            }
        }
        None
    }

    fn contains_with_name(self, name: &str) -> bool {
        for label in self {
            match label {
                LabelType::Function(func) => {
                    if func == name {
                        return true
                    }
                }
                LabelType::LJump(sub) => {
                    if sub.name == name {
                        return true
                    }
                }
                LabelType::Variable(var) => {
                    if var.name == name {
                        return true
                    }
                }
            }
        }
        false
    }
}


#[derive(Default, PartialEq, Clone, Debug)]
pub struct LJump {
    pub name: String,
    pub end_indx: usize,
}



fn is_in_bracket(input: String) -> String {
    let re = Regex::new(r"(.*?)\[.*?]").unwrap();
    if let Some(captures) = re.captures(&input) {
        if let Some(matched) = captures.get(1) {
            return matched.as_str().to_string()
        }
    }
    "".to_string()
}


pub fn is_insn_init(name: &str, c_code_final: &mut Vec<String>, lines: Vec<&str>) -> bool {
    if INSN_INIT.iter().any(|insn| lines.iter().any(|w|w == insn)) {
        if let Some(var) = unsafe {LABEL.clone().get_variable_with_name(name)}{
            if var.expr.types_expr.contains("[") {
                if var.value != "" {
                    c_code_final.push(format!("{} {}[{}] = {};", is_in_bracket(var.expr.types_expr), var.name, var.expr.size, var.value));
                }else {
                    c_code_final.push(format!("{} {}[{}];", is_in_bracket(var.expr.types_expr), var.name, var.expr.size))
                }
            }else {
                if var.value != "" {
                    c_code_final.push(format!("{} {} = {};", var.expr.types_expr, var.name, var.value));
                }else {
                    c_code_final.push(format!("{} {};", var.expr.types_expr, var.name))
                }
            }
            return true
        }
    }
    return false
}







pub fn get_in_guil(expr: String) -> usize{
    let mut count = 0;
    let mut br = false;
    for char in expr.chars() {
        if char == '\'' {
            br = true;
            continue
        }
        if br && char == '\''{
            break
        }else if br == true {
            count += 1
        }
    }
    return count
}

fn remove_spaces_after_commas(input: &str) -> String {
    let mut result = String::new();
    let mut skip_space = false;

    for c in input.chars() {
        if c == ',' {
            result.push(c);
            skip_space = true;
            continue
        } else if c == ' ' && skip_space {
            skip_space = false;
            continue
        } else {
            result.push(c);
            skip_space = false;
            continue
        }
    }

    result
}

fn format_value(value: &str) -> String {
    let value = remove_spaces_after_commas(value.trim_start());
    let mut result = String::new();
    if value.contains(",") || value.contains("'"){
        result.push('{');
        let mut wait_next_char = false;
        let mut is_com = false;
        for chars in value.chars(){
            if chars == '\'' && !is_com{
                is_com = true;
                continue
            }
            if chars == '\'' && is_com{
                is_com = false;
                continue
            }
            if chars == '\\' {
                result.push_str(&format!("'{chars}"));
                wait_next_char = true;
                continue
            }
            if wait_next_char {
                wait_next_char = false;
                result.push_str(&format!("{chars}', "));
                continue
            }
            if chars == ',' && !is_com {
                continue
            }
            if is_com {
                result.push_str(&format!("'{chars}', "))
            }else {
                result.push_str(&format!("{chars}, "));
            }
        }
        result.truncate(result.len()-2);
        result.push('}');
    }else {
        result = value.to_string();
    }
    result
}


fn get_resb_digit(number: &str, insn: &str) -> Result<usize> {
    if number.ends_with("h") {
        Ok(usize::from_str_radix(&number[..number.len() - 1], 16)?)
    } else if number.starts_with("0x") {
        Ok(usize::from_str_radix(&number[2..], 16)?)
    } else if number.chars().all(|c| c.is_digit(10)) {
        Ok(number.parse()?)
    } else {
        Err(anyhow!(
            "The operand of the instruction '{}' is not a valid operand",
            insn
        ))
    }
}



fn get_var_with_insn(line: &str, insn: &str) -> Var {
    let indx_f = line.find(&format!(" {insn}")).unwrap();
    let value = line.replace(&line[0..indx_f+1+insn.len()], "");
    let comma_indices: Vec<usize> = value.match_indices(',').map(|(i, _)| i).collect();
    let mut variable_ins = Var {
        name: line[0..indx_f].replace(" ", ""),
        value: format_value(line[indx_f+1 + insn.len()..].trim_start().trim_end()),
        expr: VarDet::default(),
    };

    let (types_expr, multiplier) = match insn {
        "db" | "resb" => ("uint8_t", 1),
        "dw" | "resw" => ("uint16_t", 2),
        "dd" | "resd" => ("uint32_t", 4),
        "dq" | "resq"  => ("uint64_t", 8),
        _ => return Var::default(),
    };

    if ALLOC_INSN.contains(&insn) {
        variable_ins.value = "".to_string();
        match get_resb_digit(line.split_whitespace().collect::<Vec<&str>>().last().unwrap(), insn) {
            Ok(value) => {
                return if value == 1 {
                    variable_ins.expr.types_expr = types_expr.to_string();
                    variable_ins.expr.size = multiplier;
                    variable_ins
                } else {
                    variable_ins.expr.types_expr = format!("{types_expr}[{value}]");
                    variable_ins.expr.size = value * multiplier;
                    variable_ins
                }
            },
            Err(e) => {
                print_msg!(LogLevel::Error(Error::Arithmetic(Box::new(e))), "")
            }
        }
    }


    let mut total_len = 0;

    if comma_indices.len() > 0 {
        let mut count = 0;
        for i in 0..comma_indices.len() {
            let curr_indx = comma_indices[i];
            let next_indx;
            if i + 1 < comma_indices.len() {
                next_indx = comma_indices[i + 1];
            }else {
                next_indx = value.len();
            }
            let before_c = &value[count..curr_indx];
            let after_c = &value[curr_indx..next_indx];
            if before_c.contains("'") {
                total_len += get_in_guil(before_c.to_string());
            }else {
                total_len += 1;
            }
            if after_c.contains("'") {
                total_len += get_in_guil(after_c.to_string());
            }else {
                total_len += 1;
            }
            if next_indx == value.len() {
                break
            }
            count = curr_indx;
        }
    }else if value.contains("'") {
        total_len = get_in_guil(value);
    }else if ALLOC_INSN.contains(&insn){
        if value.starts_with("0x") {
            total_len = value[2..].parse().unwrap_or_default();
        }else if value.ends_with("h") {
            total_len = value[0..value.len()-1].parse().unwrap_or_default();
        }else {
            total_len = value.parse().unwrap_or_default();
        }
    }
    let types_exprs;
    if total_len > 1 {
        types_exprs = format!("{types_expr}[{total_len}]");
    }else {
        types_exprs = types_expr.to_string();
    }
    variable_ins.expr = VarDet {
        types_expr: types_exprs,
        size: total_len * multiplier,
    };
    variable_ins
}




pub fn get_label(asm_code: &[String]) -> Vec<LabelType>{
    let mut temp_label = Vec::new();
    let mut is_in_label = (false, String::new());
    for line in asm_code {
        let liner: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        let line = line.replace(" ", "");
        if line == "" { continue }
        let mnemonic = *liner.get(0).unwrap();
        if mnemonic == "call" {
            let name = liner.last().unwrap().to_string();
            if !name.contains("[") && types::Type::get_type_of_reg(&name) == types::Type::UN {
                temp_label.push(LabelType::Function(name))
            }
        }
        else if mnemonic.starts_with("j"){
            let name = liner.last().unwrap().to_string();
            if !name.contains("[") {
                temp_label.push(LabelType::LJump(LJump {
                    end_indx: get_endp_jmp(asm_code, name.clone()),
                    name,
                }));
            }
        }
        else if let Some(&insn) = liner.iter().find(|&word| INSN_INIT.contains(word)) {
            temp_label.push(LabelType::Variable(get_var_with_insn(&liner.join(" "), insn)))
        }
        else if line.contains("pushrbp") || line.contains("movrbp,") && line.contains("sp") {
            temp_label.push(LabelType::Function(is_in_label.1.clone()));
            is_in_label.0 = false;
        }

        else if line.contains(":") {
            if is_in_label.0 && !temp_label.clone().contain_funcion(&is_in_label.1){
                temp_label.push(LabelType::LJump(LJump {
                    name: is_in_label.1.clone(),
                    end_indx: get_endp_jmp(asm_code, is_in_label.1.clone())
                }));
            }
            temp_label.dedup();
            is_in_label.0 = true;
            is_in_label.1 = line.replace(":", "");
            continue
        }
    }
    if is_in_label.0 {
        temp_label.push(LabelType::LJump(LJump {
            name: is_in_label.1.clone(),
            end_indx: get_endp_jmp(asm_code, is_in_label.1)
        }))
    }
    temp_label
}





pub fn get_endp_jmp(asm_code: &[String], name_func: String) -> usize {
    let mut is_in_sub = false;
    for (i, line) in asm_code.iter().enumerate(){
        let line = line.replace(" ", "");
        if line.contains(&format!("{name_func}:")) {
            is_in_sub = true;
            continue
        }
        if is_in_sub && line.contains(":"){
            return i-1
        }
    }
    if is_in_sub {
        return asm_code.len()
    }
    0
}



pub fn get_struct_ljump_with_name(name: String, label_vec: Vec<LabelType>) -> LJump {
    for label in label_vec {
        match label {
            LabelType::LJump(l_jump) => {
                if l_jump.name == name {
                    return l_jump;
                }
            }
            _ => continue
        }
    }
    LJump::default()
}