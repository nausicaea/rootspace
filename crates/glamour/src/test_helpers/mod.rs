use crate::mat::Mat4;

mod approx;
mod cmp;
mod convert;
pub mod proptest;

pub fn diff<R, F>(lhs: &Mat4<R>, rhs: &Mat4<R>, eq: F) -> impl std::fmt::Display
where
    R: std::fmt::Display,
    F: Fn(&R, &R) -> bool,
{
    let mut diff_buff = String::new();
    for r in 0..4 {
        for c in 0..4 {
            let lhsv = &lhs[(r, c)];
            let rhsv = &rhs[(r, c)];
            if !eq(lhsv, rhsv) {
                diff_buff.push_str(&format!("(r={}, c={}): {} != {}\n", r, c, lhsv, rhsv));
            }
        }
    }
    diff_buff
}
