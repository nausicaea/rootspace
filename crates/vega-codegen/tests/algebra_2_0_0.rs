use num_traits::Zero;
use proptest::{
    collection, num::i32::ANY as I32_FULL_RANGE, prop_assert_eq, prop_compose, proptest, strategy::Strategy,
};
use std::ops::Range;
use vega_codegen::algebra;
use vega_internal::Multivector as _;

const I32_HALF_RANGE: Range<i32> = (i32::MIN / 2)..(i32::MAX / 2);
const I32_SQRT_RANGE: Range<i32> = -46340..46340;
const I32_QBRT_RANGE: Range<i32> = -215..215;

algebra!(2, 0, 0);

prop_compose! {
    fn e1(s: impl Strategy<Value = i32>)(v in s) -> E1<i32> {
        E1(v)
    }
}

prop_compose! {
    fn e2(s: impl Strategy<Value = i32>)(v in s) -> E2<i32> {
        E2(v)
    }
}

prop_compose! {
    fn e12(s: impl Strategy<Value = i32>)(v in s) -> E12<i32> {
        E12(v)
    }
}

prop_compose! {
    fn multivector(s: impl Strategy<Value = i32>)(v in collection::vec(s, 4)) -> Multivector<i32> {
         Multivector {
            s: v[0],
            e1: E1(v[1]),
            e2: E2(v[2]),
            e12: E12(v[3]),
        }
    }
}

#[test]
fn multivector_grade_is_zero_for_scalar_multivectors() {
    let a = Multivector { s: 1, ..Zero::zero() };

    assert_eq!(a.grade(), Some(0));
}

#[test]
fn multivector_grade_is_none_for_mixed_grade_multivectors() {
    let a = Multivector {
        s: 1,
        e1: E1(2),
        ..Zero::zero()
    };

    assert_eq!(a.grade(), None);
}

#[test]
fn multivector_gproj_returns_a_scalar_multivector_for_grade_zero() {
    let a = Multivector {
        s: 1,
        e1: E1(2),
        ..Zero::zero()
    };

    assert_eq!(a.gproj(0), Multivector { s: 1, ..Zero::zero() });
}

#[test]
fn multivector_gproj_returns_a_pure_vector_for_grade_one() {
    let a = Multivector {
        s: 1,
        e1: E1(2),
        e2: E2(3),
        e12: E12(4),
    };

    assert_eq!(
        a.gproj(1),
        Multivector {
            e1: E1(2),
            e2: E2(3),
            ..Zero::zero()
        }
    );
}

#[test]
fn multivector_gproj_returns_the_zero_multivector_for_grades_too_high() {
    let a = Multivector {
        s: 1,
        e1: E1(2),
        ..::num_traits::Zero::zero()
    };

    assert_eq!(a.gproj(10), Multivector::zero());
}

proptest! {
    #[test]
    fn multivector_grade_is_only_defined_for_pure_multivectors(a in multivector(I32_FULL_RANGE)) {
        let expected = if a.e12 != E12(0) && a.e1 == E1(0) && a.e2 == E2(0) && a.s == 0 {
            Some(2)
        } else if a.e12 == E12(0) && (a.e1 != E1(0) || a.e2 != E2(0)) && a.s == 0 {
            Some(1)
        } else if a.e12 == E12(0) && a.e1 == E1(0) && a.e2 == E2(0) && a.s != 0 {
            Some(0)
        } else {
            None
        };

        prop_assert_eq!(a.grade(), expected);
    }

    #[test]
    fn multivector_gproj_returns_the_grade_projection(a in multivector(I32_FULL_RANGE), g: usize) {
        let expected = match g {
            0 => Multivector { s: a.s, ..Zero::zero() },
            1 => Multivector { e1: a.e1, e2: a.e2, ..Zero::zero() },
            2 => Multivector { e12: a.e12, ..Zero::zero() },
            _ => Multivector::zero(),
        };

        prop_assert_eq!(a.gproj(g), expected);
    }

    #[test]
    fn multivector_implements_add_self_with_output_self(a in multivector(I32_HALF_RANGE), b in multivector(I32_HALF_RANGE)) {
        let expected = Multivector {
            s: a.s + b.s,
            e1: a.e1 + b.e1,
            e2: a.e2 + b.e2,
            e12: a.e12 + b.e12,
        };

        prop_assert_eq!(expected, a + b);
    }

    #[test]
    fn multivector_implements_commutative_add_scalars(a in multivector(I32_HALF_RANGE), b in I32_HALF_RANGE) {
        let expected = Multivector {
            s: a.s + b,
            e1: a.e1,
            e2: a.e2,
            e12: a.e12,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn multivector_implements_commutative_add_e1(a in multivector(I32_HALF_RANGE), b in e1(I32_HALF_RANGE)) {
        let expected = Multivector {
            s: a.s,
            e1: a.e1 + b,
            e2: a.e2,
            e12: a.e12,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn multivector_implements_commutative_add_e2(a in multivector(I32_HALF_RANGE), b in e2(I32_HALF_RANGE)) {
        let expected = Multivector {
            s: a.s,
            e1: a.e1,
            e2: a.e2 + b,
            e12: a.e12,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn multivector_implements_commutative_add_e12(a in multivector(I32_HALF_RANGE), b in e12(I32_HALF_RANGE)) {
        let expected = Multivector {
            s: a.s,
            e1: a.e1,
            e2: a.e2,
            e12: a.e12 + b,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e1_implements_commutative_add_scalar(a in e1(I32_FULL_RANGE), b in I32_FULL_RANGE) {
        let expected = Multivector {
            s: b,
            e1: a,
            e2: E2(0),
            e12: E12(0),
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e2_implements_commutative_add_scalar(a in e2(I32_FULL_RANGE), b in I32_FULL_RANGE) {
        let expected = Multivector {
            s: b,
            e1: E1(0),
            e2: a,
            e12: E12(0),
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e12_implements_commutative_add_scalar(a in e12(I32_FULL_RANGE), b in I32_FULL_RANGE) {
        let expected = Multivector {
            s: b,
            e1: E1(0),
            e2: E2(0),
            e12: a,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e1_implements_commutative_add_e2(a in e1(I32_FULL_RANGE), b in e2(I32_FULL_RANGE)) {
        let expected = Multivector {
            s: 0,
            e1: a,
            e2: b,
            e12: E12(0),
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e1_implements_commutative_add_e12(a in e1(I32_FULL_RANGE), b in e12(I32_FULL_RANGE)) {
        let expected = Multivector {
            s: 0,
            e1: a,
            e2: E2(0),
            e12: b,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e2_implements_commutative_add_e12(a in e2(I32_FULL_RANGE), b in e12(I32_FULL_RANGE)) {
        let expected = Multivector {
            s: 0,
            e1: E1(0),
            e2: a,
            e12: b,
        };

        prop_assert_eq!(expected, a + b);
        prop_assert_eq!(expected, b + a);
    }

    #[test]
    fn e1_implements_commutative_mul_scalar(a in e1(I32_SQRT_RANGE), b in I32_SQRT_RANGE) {
        let expected = E1(a.0 * b);

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e2_implements_commutative_mul_scalar(a in e2(I32_SQRT_RANGE), b in I32_SQRT_RANGE) {
        let expected = E2(a.0 * b);

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e12_implements_commutative_mul_scalar(a in e12(I32_SQRT_RANGE), b in I32_SQRT_RANGE) {
        let expected = E12(a.0 * b);

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e1_implementes_commutative_mul(a in e1(I32_SQRT_RANGE), b in e1(I32_SQRT_RANGE)) {
        let expected = a.0 * b.0;

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e2_implementes_commutative_mul(a in e2(I32_SQRT_RANGE), b in e2(I32_SQRT_RANGE)) {
        let expected = a.0 * b.0;

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e12_implementes_commutative_mul(a in e12(I32_SQRT_RANGE), b in e12(I32_SQRT_RANGE)) {
        let expected = -a.0 * b.0;

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn e1_implementes_anticommutative_mul_e2(a in e1(I32_SQRT_RANGE), b in e2(I32_SQRT_RANGE)) {
        let expected = E12(a.0 * b.0);

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(-expected, b * a);
    }

    #[test]
    fn e1_implementes_anticommutative_mul_e12(a in e1(I32_SQRT_RANGE), b in e12(I32_SQRT_RANGE)) {
        let expected = E2(a.0 * b.0);

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(-expected, b * a);
    }

    #[test]
    fn e2_implementes_anticommutative_mul_e12(a in e2(I32_SQRT_RANGE), b in e12(I32_SQRT_RANGE)) {
        let expected = E1(a.0 * b.0);

        prop_assert_eq!(-expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn multivector_implements_commutative_mul_scalar(a in multivector(I32_SQRT_RANGE), b in I32_SQRT_RANGE) {
        let expected = Multivector {
            s: a.s * b,
            e1: E1(a.e1.0 * b),
            e2: E2(a.e2.0 * b),
            e12: E12(a.e12.0 * b),
        };

        prop_assert_eq!(expected, a * b);
        prop_assert_eq!(expected, b * a);
    }

    #[test]
    fn multivector_implements_mul_e1(a in multivector(I32_SQRT_RANGE), b in e1(I32_SQRT_RANGE)) {
        let expected_mv_lhs = Multivector {
            s: a.e1.0 * b.0,
            e1: E1(a.s * b.0),
            e2: E2(-(a.e12.0 * b.0)),
            e12: E12(-(a.e2.0 * b.0)),
        };

        prop_assert_eq!(expected_mv_lhs, a * b);
    }

    #[test]
    fn e1_implements_mul_multivector(a in e1(I32_SQRT_RANGE), b in multivector(I32_SQRT_RANGE)) {
        let expected_mv_rhs = Multivector {
            s: a.0 * b.e1.0,
            e1: E1(a.0 * b.s),
            e2: E2(a.0 * b.e12.0),
            e12: E12(a.0 * b.e2.0),
        };

        prop_assert_eq!(expected_mv_rhs, a * b);
    }

    #[test]
    fn multivector_implements_mul_e2(a in multivector(I32_SQRT_RANGE), b in e2(I32_SQRT_RANGE)) {
        let expected_mv_lhs = Multivector {
            s: a.e2.0 * b.0,
            e1: E1(a.e12.0 * b.0),
            e2: E2(a.s * b.0),
            e12: E12(a.e1.0 * b.0),
        };

        prop_assert_eq!(expected_mv_lhs, a * b);
    }

    #[test]
    fn e2_implements_mul_multivector(a in e2(I32_SQRT_RANGE), b in multivector(I32_SQRT_RANGE)) {
        let expected_mv_rhs = Multivector {
            s: a.0 * b.e2.0,
            e1: E1(-(a.0 * b.e12.0)),
            e2: E2(a.0 * b.s),
            e12: E12(-(a.0 * b.e1.0)),
        };

        prop_assert_eq!(expected_mv_rhs, a * b);
    }

    #[test]
    fn multivector_implements_mul_e12(a in multivector(I32_SQRT_RANGE), b in e12(I32_SQRT_RANGE)) {
        let expected_mv_lhs = Multivector {
            s: -(a.e12.0 * b.0),
            e1: E1(-(a.e2.0 * b.0)),
            e2: E2(a.e1.0 * b.0),
            e12: E12(a.s * b.0),
        };

        prop_assert_eq!(expected_mv_lhs, a * b);
    }

    #[test]
    fn e12_implements_mul_multivector(a in e12(I32_SQRT_RANGE), b in multivector(I32_SQRT_RANGE)) {
        let expected_mv_rhs = Multivector {
            s: -(a.0 * b.e12.0),
            e1: E1(a.0 * b.e2.0),
            e2: E2(-(a.0 * b.e1.0)),
            e12: E12(a.0 * b.s),
        };

        prop_assert_eq!(expected_mv_rhs, a * b);
    }

    #[test]
    fn multivector_implements_mul_self_with_output_self(a in multivector(I32_QBRT_RANGE), b in multivector(I32_QBRT_RANGE)) {
        let expected = Multivector {
            s: b.e1.0*a.e1.0 - b.e12.0*a.e12.0 + b.e2.0*a.e2.0 + b.s*a.s,
            e1: E1(b.e1.0*a.s - b.e12.0*a.e2.0 + b.e2.0*a.e12.0 + b.s*a.e1.0),
            e2: E2(-b.e1.0*a.e12.0 + b.e12.0*a.e1.0 + b.e2.0*a.s + b.s*a.e2.0),
            e12: E12(-b.e1.0*a.e2.0 + b.e12.0*a.s + b.e2.0*a.e1.0 + b.s*a.e12.0),
        };

        assert_eq!(expected, a * b);
    }
}
