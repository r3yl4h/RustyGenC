use crate::converter::LINE_STR;

pub fn handle_conditional(wait_2insn: &mut bool, sequence: &mut Vec<String>, last_sequence: &mut Vec<String>) -> bool {
    *last_sequence = unsafe { LINE_STR.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>() };
    sequence.push("|".to_string());
    *wait_2insn = true;
    true
}