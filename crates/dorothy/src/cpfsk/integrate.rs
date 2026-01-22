use std::iter::Scan;

pub type Integrate<I> = Scan<I, f64, fn(&mut f64, f64) -> Option<f64>>;

pub fn integrate<I>(data: I) -> Integrate<I>
where
    I: Iterator<Item = f64>,
{
    data.scan(0.0, |state, sample| {
        *state += sample;
        Some(*state)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrate_0_2() {
        assert_eq!(
            integrate([0.0, 3.0, 2.0].into_iter()).collect::<Vec<_>>(),
            &[0.0, 3.0, 5.0],
        )
    }
}
