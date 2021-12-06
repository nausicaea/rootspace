use num_traits::Float;

pub fn validate_float<F>(nums: &[F]) -> bool
where
    F: Float,
{
    nums.iter().all(|n| n.is_normal() && n.abs() > F::epsilon())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_fails_for_infinity() {
        assert!(!validate_float(&[f32::INFINITY, f32::NEG_INFINITY]))
    }

    #[test]
    fn validation_fails_for_nan() {
        assert!(!validate_float(&[f32::NAN]))
    }

    #[test]
    fn validation_fails_for_epsilon() {
        assert!(!validate_float(&[f32::EPSILON]))
    }

    #[test]
    fn validation_fails_for_zero() {
        assert!(!validate_float(&[0.0f32]))
    }

    #[test]
    fn validation_succeeds_for_normals() {
        assert!(validate_float(&[1.0f32, 2.0f32, std::f32::consts::PI]))
    }
}
