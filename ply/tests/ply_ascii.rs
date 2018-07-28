#[macro_use]
extern crate assertions;
extern crate ply;

#[test]
fn header() {
    assert_ok!(ply::header(include_bytes!("example_ascii.ply")));
}
