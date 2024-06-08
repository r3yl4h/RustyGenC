use std::cmp::PartialEq;
use std::collections::BTreeSet;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use crate::{operand, stack};
use crate::logs::*;
use crate::*;
use crate::types::Type;


pub static mut LINE_STR: String = String::new();
pub static mut LINE_COUNT: usize = 0;
pub static mut IDENT: String = String::new();



#[derive(Debug, PartialEq, Clone)]
pub enum OperandType<'a> {
    Memory(MemoryType<'a>),
    Register(String),
    Immediate(String),
    Label(String),
}



#[derive(Debug, PartialEq, Clone)]
pub enum MemoryType<'a> {
    Register(&'a str, Type),
    Immediate(&'a str),
    Operator(String),
    Label(String)
}


#[derive(Debug, Default, Clone)]
pub struct Insn {
    pub mnemonic: String,
    pub op1: String,
    pub op2: String,
}


#[derive(Debug, Default)]
pub struct Context {
    pub name: String,
    pub types: Type,
    pub is_in_func: bool
}

pub static CONTEXT: OnceCell<Mutex<Context>> = OnceCell::new();


pub const ALL_REG64: [&str;16] = ["rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rbp", "rsp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"];
pub const ALL_REG32: [&str;16] = ["eax", "ebx", "ecx", "edx", "edi", "esi" ,"ebp", "esp", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d"];
pub const ALL_REG16: [&str;16] = ["ax", "bx", "cx", "dx", "di" , "si" , "bp", "sp", "r8w", "r9w", "r10w","r11w", "r12w", "r13w", "r14w", "r15w"];
pub const ALL_REG8_LOW: [&str;10] = ["al" , "bl" , "cl" , "dl" , "dil" , "sil" , "bpl", "spl" ,"r8b", "r9b"];
pub const OPERATORS: [char; 5] = ['+', '*', '-', '/', '%'];
pub const ALL_REG: [&[&str]; 4] = [&ALL_REG64, &ALL_REG32, &ALL_REG16, &ALL_REG8_LOW];




fn extract_instruction(input: &str) -> String {
    if let Some(pos) = input.find(';') {
        input[..pos].replace(" ", "")
    } else {
        input.replace(" ", "")
    }
}

pub fn converter(asm_code: &[String]) -> Vec<String>{
    CONTEXT.get_or_init(|| Mutex::new(Context::default()));
    let mut c_code_final = vec![String::from("#include <stdio.h>"), String::from("#include <stdint.h>")];
    let (mut stack_mem, mut stack_ptr) = (BTreeSet::from([stack::StackMem::new()]), vec![stack::StackPtr::new()]);
    unsafe { label::LABEL = label::get_label(asm_code)}
    let label = unsafe { label::LABEL.clone()};
    let mut conditional_wait = false;
    let mut vec_idxd = vec![-1];
    let mut last_insn = Insn::default();
    let mut add_abt_st = 0;
    let mut last_sequence = Vec::new();


    for (i, line) in asm_code.iter().enumerate() {
        if line.replace(" ", "").is_empty() {
            continue
        }
        unsafe { (LINE_COUNT, LINE_STR) = (i + 1, line.to_string()) }
        let mut sequence = Vec::new();
        let lines: Vec<_> = line.split(';').flat_map(|s| s.split_whitespace()).collect();
        let mut mnemonic = *lines.get(0).unwrap_or(&"");
        let line = extract_instruction(line);
        let (op1, op2) = types::get_operands(&line, mnemonic.len());
        let (mut wait_the_2insn, mut wait_the_2types) = (false, false);

        if mnemonic.to_lowercase() == "section" || mnemonic.to_lowercase() == "global" { continue }
        else if label::is_insn_init(mnemonic, &mut c_code_final, lines.clone()) { continue }
        else if function::is_function(mnemonic, &mut c_code_final, &mut stack_ptr, &mut stack_mem, asm_code) { continue }
        else if mnemonic.contains(":") && label::get_struct_ljump_with_name(mnemonic.replace(":", ""), label.clone()).name != "" {
            c_code_final.push(format!(" \n{}{mnemonic}", unsafe {IDENT.clone()}));
            unsafe {IDENT = "    ".to_string()}
            continue
        }
        else if mnemonic.contains(":") {
            print_msg!(LogLevel::Error(Error::UnknowLabel), "{mnemonic}");
            continue
        }
        else if conditional_wait && mnemonic.starts_with("j") && mnemonic != "jmp" || mnemonic.contains("cmov"){
            let name = *lines.last().unwrap();
            let mut sequence = Vec::new();
            if last_insn.mnemonic == "cmp" {
                flow_control::conditional::get_expr_with_conditional_cmp(mnemonic, last_insn.clone(), &mut sequence, last_sequence.clone())
            }else if last_insn.mnemonic == "test" {
                flow_control::conditional::get_expr_with_conditional_test(mnemonic, &mut sequence, last_insn.clone(), last_sequence.clone())
            }
            c_code_final.push(unsafe {IDENT.clone()} + "if (" + &sequence.join("") + ") {");
            c_code_final.push(format!("{}goto {name};", unsafe {IDENT.clone()}));
            c_code_final.push(format!("{}}}", unsafe {IDENT.clone()}));
            if mnemonic.contains("cmov") {
                mnemonic = "mov";
                vec_idxd.push(i as i32)
            }else {
                continue
            }
        }
        else if stack::init_st(&mut stack_ptr, &mut stack_mem, &mut add_abt_st, mnemonic, op1, op2) {
            continue
        }

        let mut type_op1 = Type::UN;
        let mut ops = (OperandType::Immediate(String::from("")), Type::UN);

        if let Some(operand_type) = types::get_type_operand(&op1) {
            ops = (operand_type.0.clone(), operand_type.1.clone());
            match operand_type.0 {
                OperandType::Memory(operand) => {
                    if operand_type.1 != Type::UN { type_op1 = operand_type.1; }
                    match operand {
                        MemoryType::Register(reg, types_reg) => {
                            operand::memory_op::register::get_op1_mem_reg(&mut sequence, &stack_ptr, reg, types_reg, &mut stack_mem, ops.clone())
                        }
                        MemoryType::Immediate(im) => {
                            operand::memory_op::immediat::im_for_dest_mem(im, operand_type.1, op2, &mut wait_the_2types, &mut sequence)
                        }
                        MemoryType::Operator(opt) => {
                            operand::memory_op::arthm::handle_arthm_op(&stack_ptr, &stack_mem, opt, &mut sequence, ops.clone(), &mut wait_the_2types)
                        }
                        MemoryType::Label(name) => sequence.push(name)
                    }
                }
                OperandType::Register(ref reg) => {
                    type_op1 = Type::get_type_of_reg(&reg);
                    let mut tem = false;
                    let ops = (operand_type.0.clone(), operand_type.1.clone());
                    operand::register::get_op1_reg(&mut sequence, &mut stack_ptr, reg.to_string(), op2, mnemonic, ops, &mut tem);
                    if tem { continue }
                }
                OperandType::Immediate(_) => {
                    print_msg!(LogLevel::Error(Error::DestinationImmediate), "destination: '{op1}'");
                    continue;
                }
                OperandType::Label(_) => {
                    if mnemonic != "call" && !mnemonic.starts_with("j") && mnemonic != "jmp"{
                        print_msg!(LogLevel::Error(Error::DestinationLabel), "{op1} ({mnemonic})");
                        continue
                    }
                }
            }
        }

        let mut skip2insn = false;
        if instruction::process_mnemonic(mnemonic, op1, op2, &mut sequence, &mut c_code_final, &mut stack_mem, &mut stack_ptr, &label, &mut wait_the_2insn, asm_code, &mut skip2insn, lines, ops, &mut last_sequence) == false {
            print_msg!(LogLevel::Error(Error::InvalidInstruction), "{}", mnemonic)
        }
        if skip2insn { continue }

        let mut types_op2 = Type::UN;
        let mut sequences = Vec::new();
        if let Some(operand_type) = types::get_type_operand(op2) {
            match operand_type.0 {
                OperandType::Register(reg) => {
                    if !operand::register::get_op2_reg(&mut stack_ptr, &reg, op1, &mut types_op2, &mut sequences, operand_type.1) { continue }
                }
                OperandType::Memory(operand) => {
                    sequences.push("*".to_string());
                    match operand {
                        MemoryType::Register(reg, _) => {
                            if mnemonic == "lea" {
                                sequence.push(reg.to_string())
                            }else if operand_type.1 != Type::UN {
                                types_op2 = operand_type.1;
                                sequences.push(format!("({}*){reg}", operand_type.1.get_expr_with_type()))
                            }else {
                                sequences.push(format!("({}*){reg}", type_op1.get_expr_with_type()))
                            }
                        }
                        MemoryType::Immediate(value) => {
                            if mnemonic == "lea" {
                                sequences.push(value.to_string())
                            }else if operand_type.1 != Type::UN {
                                types_op2 = operand_type.1;
                                sequences.push(format!("({}*){value}", operand_type.1.get_expr_with_type()))
                            }else {
                                sequences.push(format!("({}*){value}", type_op1.get_expr_with_type()))
                            }
                        }
                        MemoryType::Operator(opt) => {
                            if !operand::memory_op::arthm::handle_op_arthm2(&stack_ptr, &stack_mem, &mut sequences, &mut sequence, operand_type.1, type_op1, mnemonic, opt) {
                                continue
                            }
                        }
                        MemoryType::Label(name) => {
                            if operand_type.1 != Type::UN {
                                sequences.push(format!("({}){name}", operand_type.1.get_expr_with_type()))
                            }else {
                                if mnemonic == "lea" {
                                    sequences.push("&".to_owned() + &name)
                                }else {
                                    sequences.push(name)
                                }
                            }
                        }
                    }
                }
                OperandType::Immediate(operand) => {
                    sequences.push(operand)
                }
                OperandType::Label(name) => {
                    sequences.push(format!("&{name}"));
                }
            }
            if wait_the_2types {
                sequence.iter_mut().for_each(|word| replace_wait(word, types_op2));
            }
            if mnemonic == "lea" {
                sequences.iter_mut().for_each(|word| if word.contains("*") { *word = word.replace("*", ""); });
            }
            sequence.extend(sequences)
        }
        if wait_the_2insn {
            if instruction::process_mnemonic2(mnemonic, &sequence, &mut last_insn, &mut conditional_wait) {
                continue
            }
        }
        if !sequence.is_empty(){
            sequence.push(";".to_string());
            c_code_final.push(unsafe {IDENT.clone()} + &sequence.join(""));
            if *vec_idxd.last().unwrap() == i as i32 { unsafe { IDENT.truncate(IDENT.len()-4); c_code_final.push(format!("{}}}\n", IDENT.clone())); } }
        }
    }
    c_code_final.push("}".to_string());
    print_msg!(LogLevel::Debug(DebugMsg::Terminate), "");
    c_code_final
}




fn replace_wait(word: &mut String, types_op2: Type) {
    if word.contains("wait") {
        let expr = types_op2.get_expr_with_type();
        if !expr.is_empty() {
            *word = word.replace("wait", &expr);
        }
    }
}



pub fn cmp_bid(stack_ptr: &mut Vec<stack::StackPtr>, reg: &str, operand_dest: &str) -> bool {
    if let Some(st) = stack_ptr.iter().find(|st| st.reg == reg) {
        stack_ptr.push(stack::StackPtr {
            reg: operand_dest.to_string(),
            stack_addr_ptr: st.stack_addr_ptr,
        });
        true
    } else {
        false
    }
}




pub fn push_reg_with_len(sequence: &mut Vec<String>, df: Type, reg: &str) {
    if df != Type::X64 {
        sequence.push(format!("({}){}", df.get_expr_with_type(), types::_to64b(String::from(reg))));
    }else {
        sequence.push(reg.to_string())
    }
}