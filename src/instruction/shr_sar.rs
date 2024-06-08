pub fn handle_shr_sar(sequence: &mut Vec<String>) -> bool {
    sequence.push(" >>= ".to_string());
    true
}