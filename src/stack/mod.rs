use std::cmp::Ordering;
use std::collections::BTreeSet;
use crate::*;
pub mod finder;
pub mod variable;



#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StackMem {
    pub addr: u64,
    pub name: String,
    pub value: String,
    pub size: usize,
    pub line_def: usize
}


impl StackMem {
    pub(crate) fn new() -> StackMem {
        return StackMem {
            addr: HIGH_OF_STACK,
            name: "".to_string(),
            value: "".to_string(),
            size: 0,
            line_def: 1234567890,
        }
    }
}


impl Ord for StackMem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line_def.cmp(&other.line_def)
    }
}


impl PartialOrd for StackMem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub const HIGH_OF_STACK: u64 = 10000000000000;


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StackPtr {
    pub stack_addr_ptr: u64,
    pub reg: String
}


impl StackPtr {
    pub fn new() -> StackPtr {
        return StackPtr {
            stack_addr_ptr: HIGH_OF_STACK,
            reg: "rsp".to_string()
        }
    }
}




pub fn push_in_stack(op1: &str, stack_mem: &mut BTreeSet<StackMem>, stack_ptr: &mut Vec<StackPtr>, line_def: usize) {
    if let Some((op_type, _)) =  types::get_type_operand(op1) {
        match op_type {
            converter::OperandType::Register(reg) => {
                let type_reg = types::Type::get_type_of_reg(&reg);
                if type_reg != types::Type::UN {
                    stack_mem.insert(StackMem {
                        addr: stack_mem.last().unwrap().addr - type_reg.get_size_with_type() as u64,
                        name: format!("st_{}", stack_mem.len()+1),
                        value: reg,
                        size: type_reg.get_size_with_type(),
                        line_def
                    });
                }
            }
            converter::OperandType::Immediate(op_im) => {
                stack_mem.insert(StackMem {
                    addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                    name: format!("st_{}", stack_mem.len()+1),
                    value: op_im,
                    size: unsafe {ARCH.get_size_with_type()},
                    line_def
                });
            }
            converter::OperandType::Memory(mem) => {
                match mem {
                    converter::MemoryType::Register(reg, _) => {
                        stack_mem.insert(StackMem {
                            addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                            name: format!("st_{}", stack_mem.len()+1),
                            value: reg.to_string(),
                            size: unsafe {ARCH.get_size_with_type()},
                            line_def
                        });
                    }
                    converter::MemoryType::Immediate(im ) => {
                        stack_mem.insert(StackMem {
                            addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                            name: format!("st_{}", stack_mem.len()+1),
                            value: im.to_string(),
                            size: unsafe {ARCH.get_size_with_type()},
                            line_def
                        });
                    }
                    converter::MemoryType::Operator(opt) => {
                        stack_mem.insert(StackMem {
                            addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                            name: format!("st_{}", stack_mem.len()+1),
                            value: opt,
                            size: unsafe {ARCH.get_size_with_type()},
                            line_def
                        });
                    }
                    converter::MemoryType::Label(name) => {
                        stack_mem.insert(StackMem {
                            addr: stack_mem.last().unwrap().addr - unsafe {ARCH.get_size_with_type() as u64},
                            name: format!("st_{}", stack_mem.len()+1),
                            value: name,
                            size: unsafe {ARCH.get_size_with_type()},
                            line_def
                        });
                    }
                }
            }
            _ => {}
        }
        if let Some(rsp_ptr) = stack_ptr.iter_mut().find(|st_ptr|st_ptr.reg.ends_with("sp")) {
            rsp_ptr.stack_addr_ptr -= unsafe {ARCH.get_size_with_type()} as u64;
        }
    }
}


























pub fn init_st(stack_ptr: &mut Vec<StackPtr>, stack_mem: &mut BTreeSet<StackMem>, add_abt_st: &mut usize, mnemonic: &str, op1: &str, op2: &str) -> bool {


    fn handle_register_case(op1: &str, op2: &str, stack_ptr: &mut Vec<StackPtr>) -> bool {
        if let Some((op_t, _)) = types::get_type_operand(op2) {
            if let converter::OperandType::Register(reg) = op_t {
                if reg.ends_with("sp") {
                    if let Some(sp_ptr) = stack_ptr.iter().find(|s| s.reg.ends_with("sp")) {
                        stack_ptr.push(StackPtr {
                            reg: op1.to_string(),
                            stack_addr_ptr: sp_ptr.stack_addr_ptr,
                        });
                        return true;
                    } else {
                        print_msg!(LogLevel::CriticalErr(CriticalErr::StackPtrIsNotRsp), "line {}, instruction: '{}'", unsafe {converter::LINE_COUNT}, unsafe {converter::LINE_STR.clone()});
                    }
                }
            }
        }
        false
    }




    fn handle_memory_case(op1: &str, op2: &str, stack_ptr: &mut Vec<StackPtr>) -> bool {
        if let Some((op_t, _)) = types::get_type_operand(op2) {
            if let converter::OperandType::Memory(mem) = op_t {
                if let converter::MemoryType::Operator(opt) = mem {
                    if let Some(st_ptr) = stack_ptr.iter().find(|s| opt.contains(&s.reg)) {
                        match eval::calcule_st_addr(&opt.replace(&st_ptr.reg, &st_ptr.stack_addr_ptr.to_string())) {
                            Ok(value) => {
                                stack_ptr.push(StackPtr {
                                    reg: op1.to_string(),
                                    stack_addr_ptr: value,
                                });
                                return true;
                            }
                            Err(_) => {}
                        }
                    }
                }else if let converter::MemoryType::Register(reg, _) = mem {
                    if let Some(st_c) = stack_ptr.iter().find(|s|s.reg == reg) {
                        stack_ptr.push(StackPtr {
                            reg: op1.to_string(),
                            stack_addr_ptr: st_c.stack_addr_ptr
                        });
                        return true
                    }
                }
            }
        }
        false
    }





    match mnemonic {
        "enter" => {
            instruction::handle_enter(op1, stack_ptr, stack_mem, add_abt_st);
            true
        }
        "leave" => {
            instruction::handle_leave(stack_ptr, stack_mem, *add_abt_st as u64);
            true
        }
        "mov" => {
            if op1.ends_with("bp") || op1.ends_with("sp") {
                handle_register_case(op1, op2, stack_ptr)
            } else {
                false
            }
        }
        "lea" => {
            if op1.ends_with("bp") || op1.ends_with("sp") {
                handle_memory_case(op1, op2, stack_ptr)
            } else {
                false
            }
        }
        _ => false,
    }
}






fn handle_memory_register_case(
    reg: &str,
    op2: &str,
    stack_ptr: &mut Vec<StackPtr>,
    stack_mem: &mut BTreeSet<StackMem>,
    i: usize,
    op_type: (converter::OperandType, types::Type),
) {
    if let Some(st_ptr) = stack_ptr.iter_mut().find(|st_ptr| st_ptr.reg == reg) {
        if !stack_mem.iter().any(|st| st.addr == st_ptr.stack_addr_ptr) {
            stack_mem.insert(StackMem {
                value: op2.to_string(),
                addr: st_ptr.stack_addr_ptr,
                name: format!("st_{}", stack_mem.len() + 1),
                line_def: i,
                size:
                if op_type.1 != types::Type::UN {
                    op_type.1.get_size_with_type()
                } else {
                    let type_reg = types::Type::get_type_of_reg(op2);
                    if type_reg != types::Type::UN {
                        type_reg.get_size_with_type()
                    } else {
                        0
                    }
                },
            });
        }
    }
}









fn handle_memory_operator_case(
    opt: &str,
    op2: &str,
    stack_ptr: &mut Vec<StackPtr>,
    stack_mem: &mut BTreeSet<StackMem>,
    i: usize,
    op_type: (converter::OperandType, types::Type),
) {
    if let Some(st_reg) = stack_ptr.iter().find(|st| opt.contains(&st.reg)) {
        match eval::calcule_st_addr(&opt.replace(&st_reg.reg, &st_reg.stack_addr_ptr.to_string())) {
            Ok(value) => {
                if finder::get_elm_in_stack_with_addr(stack_mem.clone(), value).is_none() {
                    variable::little_push(stack_mem, value, op2, i, op_type.1)
                }
            }
            Err(_) => {},
        }
    }
}



