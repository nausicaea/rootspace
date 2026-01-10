use dorothy::{Spec, decode, encode};

#[test]
fn roundtrip() {
    let source = "Hello, World!".as_bytes();
    let spec = Spec::with_kcs();
    let encoded = encode(&spec, source).collect::<Vec<_>>();
    let decoded = decode(&spec, encoded).unwrap();
    assert_eq!(&decoded[0], source);
}
