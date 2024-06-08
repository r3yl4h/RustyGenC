pub fn handle_cwde(sequence: &mut Vec<String>) -> bool {
    sequence.push("*(int32_t*)&rax = (int16_t)rax".to_string());
    true
}