#![no_main]

use libfuzzer_sys::fuzz_target;
use plyers::parse_ply;

fuzz_target!(|data: &[u8]| {
    let _ = parse_ply(data);
});
