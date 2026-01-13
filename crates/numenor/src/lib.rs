#![no_std]

/// Defines the additive identity element. This trait is functionally equivalent to
/// [`num_traits::Zero`](https://docs.rs/num-traits/latest/num_traits/identities/trait.Zero.html) but without the type bound [`core::ops::Add`]`<Self, Output = Self>` to allow use in
/// special edge cases.
pub trait Zero {
    /// Create a new instance of `Self` that represents the additive identity.
    fn zero() -> Self;
}

/// Defines the compile-time constant additive identity element. As with [`Zero`], this trait is
/// functionally equivalent to
/// [`num_traits::ConstZero`](https://docs.rs/num-traits/latest/num_traits/identities/trait.ConstZero.html) but without the type bounds to allow use in
/// special edge cases.
pub trait ConstZero {
    /// Provide an instance of `Self` that represents the additive identity.
    const ZERO: Self;
}

/// Defines the multiplicative identity element. This trait is functionally equivalent to
/// [`num_traits::One`](https://docs.rs/num-traits/latest/num_traits/identities/trait.One.html) but without the type bound [`core::ops::Mul`]`<Self, Output = Self>` to allow use in
/// special edge cases.
pub trait One {
    /// Create a new instance of `Self` that represents the multiplicative identity.
    fn one() -> Self;
}

/// Defines the compile-time constant multiplicative identity element. As with [`One`], this trait is
/// functionally equivalent to
/// [`num_traits::ConstOne`](https://docs.rs/num-traits/latest/num_traits/identities/trait.ConstOne.html) but without the type bounds to allow use in
/// special edge cases.
pub trait ConstOne {
    /// Provide an instance of `Self` that represents the multiplicative identity.
    const ONE: Self;
}

/// Defines the compile-time constant additive inverse of the multiplicative identity.
pub trait ConstMinusOne: ConstZero + ConstOne {
    /// Provide an instance of `Self` that represents a negative unit value.
    const MINUS_ONE: Self;
}

/// Represents numbers which have upper and lower bounds. Functionally equivalent to
/// [`num_traits::Bounded`](https://docs.rs/num-traits/latest/num_traits/bounds/trait.Bounded.html).
pub trait ConstBounded {
    /// The lowest finite representable value.
    const MIN: Self;
    /// The highest finite representable value.
    const MAX: Self;
}

/// Performs a coercion of `f32` to the destination type. Functionally equivalent to the `as` operator. It completely glosses over things like:
///
/// 1. Precision loss
/// 2. Saturation
/// 3. Meaningless conversions (i.e. `f32::NAN` or `f32::INFINITY` to `i32`)
/// 4. Platform-specific behavior
///
/// # Bad Examples
///
/// ```rust
/// use numenor::FromF32Unchecked;
///
/// // Decimal values are truncated
/// assert_eq!(i32::from_f32_unchecked(1.25_f32), 1);
///
/// // Large values saturate
/// assert_eq!(i8::from_f32_unchecked(1000_f32), i8::MAX);
/// assert_eq!(i8::from_f32_unchecked(-1000_f32), i8::MIN);
///
/// // Nonsensical conversions behave like the as operator
/// assert_eq!(i32::from_f32_unchecked(f32::INFINITY), f32::INFINITY as i32);
/// assert_eq!(i32::from_f32_unchecked(-f32::INFINITY), -f32::INFINITY as i32);
/// assert_eq!(i32::from_f32_unchecked(f32::NAN), f32::NAN as i32);
/// assert_eq!(i32::from_f32_unchecked(f32::EPSILON), f32::EPSILON as i32);
/// ```
pub trait FromF32Unchecked {
    /// Coerce an `f32` value to the output type. Refer to [`FromF32Unchecked`] for more details.
    fn from_f32_unchecked(value: f32) -> Self;
}

/// Performs a coercion of the source type to `f32`. Functionally equivalent to the `as` operator. About as dangerous as [`FromF32Unchecked`].
///
/// # Weaknesses
///
/// 1. Precision loss
/// 2. Platform-specific behavior
pub trait IntoF32Unchecked {
    /// Coerce a generic value to `f32`. Refer to [`IntoF32Unchecked`] for more details.
    fn into_f32_unchecked(self) -> f32;
}

macro_rules! impl_const_zero {
    ($($type:ty => $zero:literal);+ $(;)*) => {
        $(
            impl ConstZero for $type {
                const ZERO: Self = $zero;
            }
        )*
    };
}

macro_rules! impl_const_one {
    ($($type:ty => $one:literal);+ $(;)*) => {
        $(
            impl ConstOne for $type {
                const ONE: Self = $one;
            }
        )*
    };
}

macro_rules! impl_const_minus_one {
    ($($type:ty => $minus_one:literal);+ $(;)*) => {
        $(
            impl ConstMinusOne for $type {
                const MINUS_ONE: Self = $minus_one;
            }
        )*
    };
}

macro_rules! impl_const_bounded {
    ($($t:ty),+) => {
        $(
            impl ConstBounded for $t {
                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;
            }
        )+
    };
}

macro_rules! impl_from_f32_unchecked {
    ($($t:ty),+) => {
        $(
            impl FromF32Unchecked for $t {
                fn from_f32_unchecked(value: f32) -> Self {
                    value as $t
                }
            }
        )+
    };
}

macro_rules! impl_into_f32_unchecked {
    ($($t:ty),+) => {
        $(
            impl IntoF32Unchecked for $t {
                fn into_f32_unchecked(self) -> f32 {
                    self as f32
                }
            }
        )+
    };
}

impl<T: ConstZero> Zero for T {
    fn zero() -> Self {
        T::ZERO
    }
}

impl<T: ConstOne> One for T {
    fn one() -> Self {
        T::ONE
    }
}

impl_const_zero! {
    u8 => 0;
    i8 => 0;
    u16 => 0;
    i16 => 0;
    u32 => 0;
    i32 => 0;
    u64 => 0;
    i64 => 0;
    u128 => 0;
    i128 => 0;
    usize => 0;
    isize => 0;
    f32 => 0.0;
    f64 => 0.0;
}

impl_const_one! {
    u8 => 1;
    i8 => 1;
    u16 => 1;
    i16 => 1;
    u32 => 1;
    i32 => 1;
    u64 => 1;
    i64 => 1;
    u128 => 1;
    i128 => 1;
    usize => 1;
    isize => 1;
    f32 => 1.0;
    f64 => 1.0;
}

impl_const_minus_one! {
    i8 => -1;
    i16 => -1;
    i32 => -1;
    i64 => -1;
    i128 => -1;
    isize => -1;
    f32 => -1.0;
    f64 => -1.0;
}

impl_const_bounded! { u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize }

impl_from_f32_unchecked! { u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize }
impl_into_f32_unchecked! { u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize }
