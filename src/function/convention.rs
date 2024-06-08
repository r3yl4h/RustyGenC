use std::cmp::Ordering;
use crate::*;

#[derive(Eq, PartialEq, Hash)]
enum CallingConvention {
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall,
    Vectorcall,
    Msfastcall,
    Syscall,
}



const CONVENTIONS_ARG: [(CallingConvention, &[&str]);7] = [
    (CallingConvention::Cdecl, &[]),
    (CallingConvention::Stdcall, &[]),
    (CallingConvention::Fastcall, &["cx", "dx"]),
    (CallingConvention::Thiscall, &["cx"]),
    (CallingConvention::Vectorcall, &["cx", "dx", "r8", "r9", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5"]),
    (CallingConvention::Msfastcall, &["cx", "dx", "r8", "r9", "xmm0", "xmm1", "xmm2", "xmm3"]),
    (CallingConvention::Syscall, &["rax", "rdi", "rsi", "rdx", "r10", "r8", "r9"]),
];




pub fn remove_reg_is_not_call_conv(reg_suspect: &mut Vec<function::RegTrack>) {
    for arg in &mut *reg_suspect {
        match arg {
            function::RegTrack::Reg(reg) => {
                let mut trof = Vec::new();
                for (_, reg_convention) in CONVENTIONS_ARG {
                    trof.push(reg_convention.iter().any(|regs|reg.ends_with(regs)));
                }
                if !trof.contains(&true) {
                    *arg = function::RegTrack::Default
                }
            }
            _ => {}
        }
    }
    reg_suspect.retain(|r|r != &function::RegTrack::Default)
}


pub fn detect_reg_with_convention(reg_suspect: &mut Vec<function::RegTrack>, code: &[String]){
    for (i, line) in code.iter().enumerate(){
        if line.replace(" ", "").is_empty() {
            continue
        }
        if i == unsafe { converter::LINE_COUNT -1} {
            break
        }
        let liner: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        let line = line.replace(" ", "");
        let mnemonic = *liner.get(0).unwrap();
        if mnemonic == "call" || line.contains(":"){
            reg_suspect.clear();
            continue
        }
        if mnemonic == "push" {
            continue
        }
        let (op1, op2) = types::get_operands(&line, mnemonic.len());
        if let Some(op_type) = types::get_type_operand(op1) {
            match op_type.0 {
                converter::OperandType::Register(reg) => {
                    if !reg_suspect.contains(&function::RegTrack::Reg(types::_to64b(reg.clone()))) && reg.ends_with("x"){
                        reg_suspect.push(function::RegTrack::Reg(types::_to64b(reg)));
                    }
                }
                _=> {}
            }
        }
        if let Some(op_type) = types::get_type_operand(op2) {
            match op_type.0 {
                converter::OperandType::Register(regs) => {
                    if reg_suspect.contains(&function::RegTrack::Reg(types::_to64b(regs.clone()))){
                        reg_suspect.retain(|reg| reg != &function::RegTrack::Reg(types::_to64b(regs.clone())));
                    }
                }
                _=> {}
            }
        }
    }
    reg_suspect.retain(|r|match r { function::RegTrack::Reg(reg) => {!reg.ends_with("sp") || !reg.ends_with("bp")}, _=> true});
    let number_of_reg = reg_suspect.iter().filter(|&reg| match reg {
        function::RegTrack::Reg(_) => true,
        _ => false,
    }).count();

    reg_suspect.sort_by(|a, b| compare_regtrack(a, b, number_of_reg));
}



fn compare_regtrack(a: &function::RegTrack, b: &function::RegTrack, number_of_reg: usize) -> Ordering {
    match (a, b) {
        (function::RegTrack::Reg(reg_a), function::RegTrack::Reg(reg_b)) => {
            for (_, reg) in CONVENTIONS_ARG {
                let pos_a = reg.iter().position(|r| reg_a.contains(r));
                let pos_b = reg.iter().position(|r| reg_b.contains(r));
                match (pos_a, pos_b) {
                    (Some(pos_a), Some(pos_b)) => {
                        if number_of_reg >= pos_a && number_of_reg >= pos_b {
                            return pos_a.cmp(&pos_b);
                        }
                    },
                    (Some(_), None) => return Ordering::Greater,
                    (None, Some(_)) => return Ordering::Less,
                    _ => continue,
                }
            }
            Ordering::Equal
        },
        (function::RegTrack::Reg(_), function::RegTrack::Addr(_)) => Ordering::Less,
        (function::RegTrack::Addr(_), function::RegTrack::Reg(_)) => Ordering::Greater,
        (function::RegTrack::Addr(addr_a), function::RegTrack::Addr(addr_b)) => addr_a.cmp(addr_b),
        (_, _) => Ordering::Equal,
    }
}








