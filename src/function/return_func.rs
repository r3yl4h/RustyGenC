use crate::*;
use crate::label::Label;

pub fn func_have_return(code: Vec<String>) -> (bool, types::Type) {
    let mut last_ax = String::new();
    for line in code.iter().filter(|l|!l.is_empty()) {
        let linev: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        let mnemonic = *linev.get(0).unwrap_or(&"");
        let line = line.replace(" ", "");
        let (op1, op2) = types::get_operands(&line, mnemonic.len());
        if line.contains(":") && unsafe { label::LABEL.clone().contain_funcion(&line.replace(":", ""))} {
            break
        }
        if op1.ends_with("ax"){
            last_ax = op1.to_string();
        }

        if mnemonic == "call" {
            last_ax.clear();
        }

        if op2.contains("ax") && !op1.contains("ax"){
            last_ax.clear();
        }

        if mnemonic.contains("ret") {
            if last_ax != "" {
                return (true, types::Type::get_type_of_reg(&last_ax))
            }else {
                break
            }
        }
    }
    (false, types::Type::UN)
}


pub fn track_rax_after_call(asm_code: Vec<String>) -> (bool, types::Type){
    let black_insn = ["call", "cbw", "cwde", "cdqe", "ret"];
    let white_insn = ["cmp", "xchg", "test"];
    for line in asm_code.iter().filter(|&l|!l.is_empty()) {
        let mnemonic = *line.split_whitespace().collect::<Vec<&str>>().get(0).unwrap_or(&"");
        let line = line.replace(" ", "");
        let (op1, op2) = types::get_operands(&line, mnemonic.len());
        if black_insn.contains(&mnemonic){
            break
        }
        let vr = function::verify_ax(op1);
        if vr.0 {
            return if white_insn.contains(&mnemonic) {
                vr
            } else {
                (false, types::Type::UN)
            }
        }else {
            let real_vr = function::verify_ax(op2);
            if real_vr.0{
                return real_vr;
            }
        }
    }
    (false, types::Type::UN)
}