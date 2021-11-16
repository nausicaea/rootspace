pub fn validate_f32(nums: &[f32]) -> bool {
    nums.iter().all(|n| n.is_normal() && n.abs() > f32::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_fails_for_infinity() {
        assert!(!validate_f32(&[f32::INFINITY, f32::NEG_INFINITY]))
    }

    #[test]
    fn validation_fails_for_nan() {
        assert!(!validate_f32(&[f32::NAN]))
    }

    #[test]
    fn validation_fails_for_epsilon() {
        assert!(!validate_f32(&[f32::EPSILON]))
    }

    #[test]
    fn validation_fails_for_zero() {
        assert!(!validate_f32(&[0.0f32]))
    }

    #[test]
    fn validation_succeeds_for_normals() {
        assert!(validate_f32(&[1.0f32, 2.0f32, std::f32::consts::PI]))
    }
}
