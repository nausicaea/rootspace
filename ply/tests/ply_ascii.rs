#[macro_use]
extern crate assertions;
extern crate ply;
extern crate combine;

use combine::parser::Parser;

#[test]
fn header() {
    assert_ok!(ply::header().parse(&include_bytes!("example_ascii.ply")[..]));
}
