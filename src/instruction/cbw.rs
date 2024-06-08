pub fn handle_cbw(sequence: &mut Vec<String>) -> bool {
    sequence.push("*(int16_t*)&rax = (int8_t)rax".to_string());
    true
}