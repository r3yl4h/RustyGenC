pub fn handle_cdqe(sequence: &mut Vec<String>) -> bool {
    sequence.push("*(int64_t*)&rax = (int32_t)rax".to_string());
    true
}