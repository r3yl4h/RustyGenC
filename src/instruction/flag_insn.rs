pub fn handle_single_instruction(mnemonic: &str, sequence: &mut Vec<String>) -> bool {
    sequence.push(format!("asm(\"{mnemonic}\")"));
    true
}