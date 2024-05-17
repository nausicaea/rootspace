use afl::fuzz;
use rootspace::fuzzing::parse_ply;

/// Find more fuzzing tips: https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/fuzzing_in_depth.md#fuzzing-with-afl
fn main() {
    fuzz!(|data: &[u8]| {
        let _ = parse_ply(data);
    });
}
