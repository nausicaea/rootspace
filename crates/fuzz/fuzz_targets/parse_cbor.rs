#![no_main]

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;
use ciborium::from_reader;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestData {
    a: Vec<usize>,
    b: bool,
    c: f32,
    d: HashMap<usize, String>,
}

// let f = std::fs::File::create("in/parse_cbor/case1.cbor").unwrap();
// let v = TestData {
//     a: vec![0, 1],
//     b: false,
//     c: 5.0,
//     d: [(0, String::from("Hello"))].into_iter().collect(),
// };
// ciborium::into_writer(&v, f).unwrap()
fuzz_target!(|data: &[u8]| {
    if let Ok(td) = from_reader(data) {
        let _: TestData = td;
    }
});
