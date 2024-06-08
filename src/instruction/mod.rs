mod mov_movzx_lea;
mod add;
mod sub;
mod neg;
mod xor;
mod and;
mod push;
mod pop;
mod shr_sar;
mod shl_sal;
mod inc;
mod dec;
mod flag_insn;
mod cbw;
mod cwde;
mod cdqe;
mod movsx_movsxd;
mod imul;
mod conditional_insn;
mod call;
mod ret;
mod jmp;
mod idiv;

use std::collections::BTreeSet;
use crate::*;

pub fn process_mnemonic(mnemonic: &str, op1: &str, op2: &str, sequence: &mut Vec<String>, c_code_final: &mut Vec<String>,
                        stack_mem: &mut BTreeSet<stack::StackMem>, stack_ptr: &mut Vec<stack::StackPtr>, label: &Vec<label::LabelType>, wait2insn: &mut bool, asm_code: &[String], skip2: &mut bool, liner: Vec<&str>
                        , ops: (converter::OperandType, types::Type), last_sequence: &mut Vec<String>) -> bool {
    match mnemonic {
        "mov" | "movzx" | "lea" => mov_movzx_lea::handle_mov_movzx_lea(op2, sequence),
        "add" => add::handle_add(sequence),
        "sub" => sub::handle_sub(sequence),
        "neg" => neg::handle_neg(sequence),
        "xor" => xor::handle_xor(op1, op2, sequence, c_code_final, skip2),
        "and" => and::handle_and(sequence),
        "push" => push::handle_push(stack_ptr, stack_mem.clone(), c_code_final, op1, skip2),
        "pop" => pop::handle_pop(stack_ptr, stack_mem, op1, c_code_final, skip2),
        "shr" | "sar" => shr_sar::handle_shr_sar(sequence),
        "shl" | "sal" => shl_sal::handle_shl_sal(sequence),
        "inc" => inc::handle_inc(sequence),
        "dec" => dec::handle_dec(sequence),
        "clc" | "cld" | "cli" | "cmc" | "daa" | "das" => flag_insn::handle_single_instruction(mnemonic, sequence),
        "cbw" => cbw::handle_cbw(sequence),
        "cwde" => cwde::handle_cwde(sequence),
        "cdqe" => cdqe::handle_cdqe(sequence),
        "movsx" | "movsxd" => movsx_movsxd::handle_movsx_movsxd(sequence, ops),
        "imul" => imul::handle_imul(sequence, ops),
        "ud2" => handle_ud2(c_code_final),
        "cmp" | "test" => conditional_insn::handle_conditional(wait2insn, sequence, last_sequence),
        "call" => call::handle_call(op1, stack_ptr, stack_mem, c_code_final, sequence, label, asm_code, skip2),
        "ret" | "retn" => ret::handle_ret(sequence),
        "jmp" => jmp::handle_jmp(c_code_final, liner, skip2),
        "idiv" => idiv::handle_idiv(sequence, op2, ops, skip2),
        _ => false,
    }
}


pub fn process_mnemonic2(mnemonic: &str, sequence: &[String], last_insn: &mut converter::Insn, conditional_wait: &mut bool) -> bool{
    match mnemonic {
        "cmp" | "test" => {
            let full_c_line = sequence.join("");
            let t_indx = full_c_line.find("|").unwrap();
            last_insn.mnemonic = mnemonic.to_string();
            (last_insn.op1, last_insn.op2) = (full_c_line[0..t_indx].to_string(), full_c_line[t_indx+1..].to_string());
            *conditional_wait = true;
            true
        }
        _ => false
    }
}


pub fn handle_enter(op1: &str, stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: &mut BTreeSet<stack::StackMem>, add_abt_st: &mut usize) -> bool{
    stack::push_in_stack("rbp", stack_mem, stack_ptr, unsafe {converter::LINE_COUNT -1});
    let stac_ptr = stack_ptr.clone();
    if let Some(sp_ptr) = stac_ptr.iter().find(|ptr|ptr.reg.ends_with("sp")) {
        stack_ptr.push(stack::StackPtr {
            reg: "rbp".to_string(),
            stack_addr_ptr: sp_ptr.stack_addr_ptr
        })
    }else {
        print_msg!(LogLevel::CriticalErr(CriticalErr::StackPtrIsNotRsp), "instruction: {}", unsafe {converter::LINE_STR.clone()})
    }
    match op1.parse::<usize>() {
        Ok(value) => {
            *add_abt_st = value;
            stack_ptr.iter_mut().find(|ptr|ptr.reg.ends_with("sp")).unwrap().stack_addr_ptr -= value as u64;
        }Err(e) =>  {
            print_msg!(LogLevel::Error(Error::Arithmetic(Box::new(e))), "instruction: {}", unsafe {converter::LINE_STR.clone()});
            return false
        }
    }
    return true
}



pub fn handle_leave(stack_ptr: &mut Vec<stack::StackPtr>, stack_mem: &mut BTreeSet<stack::StackMem>, add_abt_st: u64) {
    if let Some(sp_ptr) = stack_ptr.iter_mut().find(|ptr|ptr.reg == "rsp") {
        sp_ptr.stack_addr_ptr += add_abt_st;
        stack_mem.retain(|st_var|!st_var.value.contains("bp"))
    }else {
        print_msg!(LogLevel::CriticalErr(CriticalErr::StackPtrIsNotRsp), "instruction: {}", unsafe {converter::LINE_STR.clone()})
    }
}



fn handle_ud2(c_code_final: &mut Vec<String>) -> bool {
    c_code_final.push(unsafe {converter::IDENT.clone()} + &"__builtin_trap();".to_string());
    false
}