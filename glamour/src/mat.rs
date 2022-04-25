use std::{
    ops::{Add, Div, Index, IndexMut, Mul, Sub, Neg},
    iter::Sum,
};

use num_traits::{Num, One, Zero, Float, Signed, Inv};

use crate::{
    mul_elem::MulElem,
    inv_elem::InvElem,
    dot::Dot,
    abop,
};

/// Vector of 2 dimensions, interpreted as column
pub type Vec2<R> = Vec<R, 2>;

impl<R> Vec2<R> {
    pub fn new(x: R, y: R) -> Self {
        Mat([[x], [y]])
    }
}

impl<R> Vec2<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self[(0, 0)]
    }

    pub fn y(&self) -> R {
        self[(1, 0)]
    }
}

/// Vector of 3 dimensions, interpreted as column
pub type Vec3<R> = Vec<R, 3>;

impl<R> Vec3<R> {
    pub fn new(x: R, y: R, z: R) -> Self {
        Mat([[x], [y], [z]])
    }
}

impl<R> Vec3<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self[(0, 0)]
    }

    pub fn y(&self) -> R {
        self[(1, 0)]
    }

    pub fn z(&self) -> R {
        self[(2, 0)]
    }
}

/// Vector of 4 dimensions, interpreted as column
pub type Vec4<R> = Vec<R, 4>;

impl<R> Vec4<R> {
    pub fn new(x: R, y: R, z: R, w: R) -> Self {
        Mat([[x], [y], [z], [w]])
    }
}

impl<R> Vec4<R>
where
    R: Copy,
{
    pub fn x(&self) -> R {
        self[(0, 0)]
    }

    pub fn y(&self) -> R {
        self[(1, 0)]
    }

    pub fn z(&self) -> R {
        self[(2, 0)]
    }

    pub fn w(&self) -> R {
        self[(3, 0)]
    }
}

// Generalized vector, interpreted as column
pub type Vec<R, const I: usize> = Mat<R, I, 1>;

/// Matrix of 2x2 dimensions
pub type Mat2<R> = Mat<R, 2, 2>;

/// Matrix of 3x3 dimensions
pub type Mat3<R> = Mat<R, 3, 3>;

/// Matrix of 4x4 dimensions
pub type Mat4<R> = Mat<R, 4, 4>;

/// Generalized matrix type, with data stored in row-major format.
#[derive(Debug, PartialEq, Clone)]
pub struct Mat<R, const I: usize, const J: usize>([[R; J]; I]);

impl<R, const I: usize, const J: usize> Mat<R, I, J> {
    /// Given a one-dimensional array index, return the corresponding two-dimensional indices for
    /// this particular matrix' demensions
    fn as_2d_idx(idx: usize) -> (usize, usize) {
        (idx / J, idx % J)
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J> 
where
    R: Copy + Zero,
{
    /// Return a copy of the specified matrix column
    pub fn col(&self, j: usize) -> Mat<R, I, 1> {
        if j >= J {
            panic!("Index j is out of bounds (max: {}, actual: {})", J, j);
        }
        let mut mat = Mat::<R, I, 1>::zero();
        for i in 0..I {
            mat[(i, 0)] = self[(i, j)];
        }
        mat
    }

    /// Return a copy of the specified matrix row
    pub fn row(&self, i: usize) -> Mat<R, 1, J> {
        if i >= I {
            panic!("Index i is out of bounds (max: {}, actual: {})", I, i);
        }
        let mut mat = Mat::<R, 1, J>::zero();
        for j in 0..J {
            mat[(0, j)] = self[(i, j)];
        }
        mat
    }

    /// Return a sub-matrix of the given size with the given starting index
    pub fn subset<const O: usize, const P: usize>(&self, i: usize, j: usize) -> Mat<R, O, P> {
        debug_assert!(O <= I && P <= J);
        debug_assert!(i + O <= I && j + P <= J);

        let mut mat = Mat::<R, O, P>::zero();
        for o in 0..O {
            for p in 0..P {
                mat[(o, p)] = self[(i + o, j + p)];
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> MulElem for Mat<R, I, J>
where
    R: Copy + Num + Zero,
{
    type Output = Self;

    fn mul_elementwise(self, rhs: Self) -> Self::Output {
        (&self).mul_elementwise(&rhs)
    }
}

impl<'a, R, const I: usize, const J: usize> MulElem for &'a Mat<R, I, J>
where 
    R: Copy + Num + Zero,
{
    type Output = Mat<R, I, J>;

    fn mul_elementwise(self, rhs: Self) -> Self::Output {
        let mut mat = Mat::<R, I, J>::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(i, j)] = self[(i, j)] * rhs[(i, j)];
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J> 
where
    R: Float + Sum,
{
    pub fn norm(&self) -> R {
        self.0.iter().flatten().map(|e| e.powi(2)).sum::<R>().sqrt()
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Zero + Copy,
{
    pub fn t(&self) -> Mat<R, J, I> {
        let mut mat = Mat::<R, J, I>::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(j, i)] = self[(i, j)];
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Float,
{
    pub fn is_nan(&self) -> bool {
        self.0.iter().flatten().any(|e| e.is_nan())
    }
}


impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: Zero + Copy,
{
    pub fn zero() -> Self {
        Mat([[R::zero(); J]; I])
    }
}

impl<R, const I: usize, const J: usize> Mat<R, I, J>
where
    R: One + Copy,
{
    pub fn one() -> Self {
        Mat([[R::one(); J]; I])
    }
}

impl<R, const I: usize> Mat<R, I, I>
where
    R: Zero + One + Copy,
{
    pub fn identity() -> Self {
        let mut mat = Mat::<R, I, I>::zero();
        for i in 0..I {
            mat[(i, i)] = R::one();
        }

        mat
    }
}

impl<R, const I: usize> Mat<R, I, I> 
where
    R: Zero + Copy,
{
    pub fn diag(&self) -> Vec<R, I> {
        let mut mat = Vec::<R, I>::zero();
        for i in 0..I {
            mat[i] = self[(i, i)];
        }

        mat
    }
}

impl<R, const I: usize, const J: usize> std::fmt::Display for Mat<R, I, J>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..I {
            write!(f, "[")?;
            for j in 0..J {
                write!(f, "{}", self[(i, j)])?;
                if j < J - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if i < I - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl<R, const I: usize, const J: usize> From<[[R; J]; I]> for Mat<R, I, J> {
    fn from(v: [[R; J]; I]) -> Self {
        Mat(v)
    }
}

impl<R> From<R> for Mat<R, 1, 1> {
    fn from(v: R) -> Self {
        Mat([[v]])
    }
}

macro_rules! impl_from_1d_array {
    ($I:literal, $J:literal, [$([$($i:literal),+ $(,)*]),+ $(,)*] $(,)*) => {
        impl<R> From<[R; $I * $J]> for Mat<R, $I, $J>
            where
                R: Copy,
        {
            fn from(v: [R; $I * $J]) -> Self {
                Mat([$(
                    [$(v[$i]),+],
                )+])
            }
        }
    }
}

impl_from_1d_array!(1, 1, [[0]]);
impl_from_1d_array!(1, 2, [[0, 1]]);
impl_from_1d_array!(1, 3, [[0, 1, 2]]);
impl_from_1d_array!(1, 4, [[0, 1, 2, 3]]);
impl_from_1d_array!(2, 1, [[0], [1]]);
impl_from_1d_array!(2, 2, [[0, 1], [2, 3]]);
impl_from_1d_array!(2, 3, [[0, 1, 2], [3, 4, 5]]);
impl_from_1d_array!(2, 4, [[0, 1, 2, 3], [4, 5, 6, 7]]);
impl_from_1d_array!(3, 1, [[0], [1], [2]]);
impl_from_1d_array!(3, 2, [[0, 1], [2, 3], [4, 5]]);
impl_from_1d_array!(3, 3, [[0, 1, 2], [3, 4, 5], [6, 7, 8]]);
impl_from_1d_array!(3, 4, [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11]]);
impl_from_1d_array!(4, 1, [[0], [1], [2], [3]]);
impl_from_1d_array!(4, 2, [[0, 1], [2, 3], [4, 5], [6, 7]]);
impl_from_1d_array!(4, 3, [[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11]]);
impl_from_1d_array!(4, 4, [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]);

impl<R, const I: usize, const J: usize> Index<usize> for Mat<R, I, J> {
    type Output = R;

    fn index(&self, index: usize) -> &Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        Index::<(usize, usize)>::index(self, (i, j))
    }
}

impl<R, const I: usize, const J: usize> IndexMut<usize> for Mat<R, I, J> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let (i, j) = Self::as_2d_idx(index);
        IndexMut::<(usize, usize)>::index_mut(self, (i, j))
    }
}

impl<R, const I: usize, const J: usize> Index<(usize, usize)> for Mat<R, I, J> {
    type Output = R;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.0.index(index.0).index(index.1)
    }
}

impl<R, const I: usize, const J: usize> IndexMut<(usize, usize)> for Mat<R, I, J> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.0.index_mut(index.0).index_mut(index.1)
    }
}

impl<R, const I: usize, const J: usize> Neg for Mat<R, I, J>
where
    R: Copy + Zero + Signed,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Neg::neg(&self)
    }
}

impl<'a, R, const I: usize, const J: usize> Neg for &'a Mat<R, I, J> 
where
    R: Copy + Zero + Signed,
{
    type Output = Mat<R, I, J>;

    fn neg(self) -> Self::Output {
        let mut mat: Mat<R, I, J> = Mat::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(i, j)] = -self[(i, j)];
            }
        }
        mat
    }
}

impl<R, const I: usize, const J: usize> InvElem for Mat<R, I, J>
where
    R: Copy + Zero + Inv<Output = R>,
{
    type Output = Self;

    fn inv_elem(self) -> Self::Output {
        InvElem::inv_elem(&self)
    }
}

impl<'a, R, const I: usize, const J: usize> InvElem for &'a Mat<R, I, J>
where
    R: Copy + Zero + Inv<Output = R>,
{
    type Output = Mat<R, I, J>;

    fn inv_elem(self) -> Self::Output {
        let mut mat: Mat<R, I, J> = Mat::zero();
        for i in 0..I {
            for j in 0..J {
                mat[(i, j)] = self[(i, j)].inv();
            }
        }
        mat
    }
}

macro_rules! impl_scalar_matops {
    ($($Op:ident, $op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<R, const I: usize, const J: usize> $Op<R> for Mat<R, I, J>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: R) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, R, const I: usize, const J: usize> $Op<&'a R> for &'a Mat<R, I, J>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: &'a R) -> Self::Output {
                let mut mat = Mat::<R, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self[(i, j)].$op(*rhs);
                    }
                }
                mat
            }
        }

        $(
        impl<const I: usize, const J: usize> $Op<Mat<$tgt, I, J>> for $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: Mat<$tgt, I, J>) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, const I: usize, const J: usize> $Op<&'a Mat<$tgt, I, J>> for &'a $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: &'a Mat<$tgt, I, J>) -> Self::Output {
                let mut mat = Mat::<$tgt, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self.$op(rhs[(i, j)]);
                    }
                }
                mat
            }
        }
        )*

        )+
    }
}

impl_scalar_matops!(
    Add, add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub, sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div, div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

macro_rules! impl_elemwise_matops {
    ($($Op:ident, $op:ident);+ $(;)*) => {
        $(
        impl<R, const I: usize, const J: usize> $Op for Mat<R, I, J>
            where
                R: Num + Copy,
        {
            type Output = Self;

            fn $op(self, rhs: Self) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'a, R, const I: usize, const J: usize> $Op for &'a Mat<R, I, J>
            where
                R: Num + Copy,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: Self) -> Self::Output {
                let mut mat = Mat::<R, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self[(i, j)].$op(rhs[(i, j)]);
                    }
                }
                mat
            }
        }
        )+
    };
}

impl_elemwise_matops!(
    Add, add;
    Sub, sub;
);

macro_rules! impl_matmul {
    ($dim:literal, $tt:tt) => {
        impl_matmul!($dim, $dim, $dim, $dim, $tt);
    };
    ($nl:literal, $ml:literal, $nr:literal, $mr:literal, $tt:tt) => {
        impl<R> Mul<Mat<R, $nr, $mr>> for Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: Mat<R, $nr, $mr>) -> Self::Output {
                (&self).mul(&rhs)
            }
        }

        impl<'a, R> Mul<&'a Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: &'a Mat<R, $nr, $mr>) -> Self::Output {
                self.dot(rhs)
            }
        }

        impl<R> Dot<Mat<R, $nr, $mr>> for Mat<R, $nl, $ml>
            where
                R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: Mat<R, $nr, $mr>) -> Self::Output {
                (&self).dot(&rhs)
            }
        }

        impl<'a, R> Dot<&'a Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
        where
            R: Num + Copy + Sum,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: &'a Mat<R, $nr, $mr>) -> Self::Output {
                let c = abop!(dot, self, rhs, $tt);
                c.into()
            }
        }
    };
}

impl_matmul!(2, 1, 1, 2, [((0), (0)), ((0), (1)), ((1), (0)), ((1), (1))]);

impl_matmul!(
    2,
    [((0, 1), (0, 2)), ((0, 1), (1, 3)), ((2, 3), (0, 2)), ((2, 3), (1, 3)),]
);
impl_matmul!(1, 2, 2, 2, [((0, 1), (0, 2)), ((0, 1), (1, 3))]);
impl_matmul!(2, 2, 2, 1, [((0, 1), (0, 1)), ((2, 3), (0, 1))]);

impl_matmul!(
    3,
    [
        ((0, 1, 2), (0, 3, 6)),
        ((0, 1, 2), (1, 4, 7)),
        ((0, 1, 2), (2, 5, 8)),
        ((3, 4, 5), (0, 3, 6)),
        ((3, 4, 5), (1, 4, 7)),
        ((3, 4, 5), (2, 5, 8)),
        ((6, 7, 8), (0, 3, 6)),
        ((6, 7, 8), (1, 4, 7)),
        ((6, 7, 8), (2, 5, 8)),
    ]
);
impl_matmul!(
    1, 3, 3, 3,
    [
        ((0, 1, 2), (0, 3, 6)),
        ((0, 1, 2), (1, 4, 7)),
        ((0, 1, 2), (2, 5, 8)),
    ]
);
impl_matmul!(
    3, 3, 3, 1,
    [
        ((0, 1, 2), (0, 1, 2)),
        ((3, 4, 5), (0, 1, 2)),
        ((6, 7, 8), (0, 1, 2)),
    ]
);

impl_matmul!(
    4,
    [
        ((0, 1, 2, 3), (0, 4, 8, 12)),
        ((0, 1, 2, 3), (1, 5, 9, 13)),
        ((0, 1, 2, 3), (2, 6, 10, 14)),
        ((0, 1, 2, 3), (3, 7, 11, 15)),
        ((4, 5, 6, 7), (0, 4, 8, 12)),
        ((4, 5, 6, 7), (1, 5, 9, 13)),
        ((4, 5, 6, 7), (2, 6, 10, 14)),
        ((4, 5, 6, 7), (3, 7, 11, 15)),
        ((8, 9, 10, 11), (0, 4, 8, 12)),
        ((8, 9, 10, 11), (1, 5, 9, 13)),
        ((8, 9, 10, 11), (2, 6, 10, 14)),
        ((8, 9, 10, 11), (3, 7, 11, 15)),
        ((12, 13, 14, 15), (0, 4, 8, 12)),
        ((12, 13, 14, 15), (1, 5, 9, 13)),
        ((12, 13, 14, 15), (2, 6, 10, 14)),
        ((12, 13, 14, 15), (3, 7, 11, 15)),
    ]
);
impl_matmul!(
    1, 4, 4, 4,
    [
        ((0, 1, 2, 3), (0, 4, 8, 12)),
        ((0, 1, 2, 3), (1, 5, 9, 13)),
        ((0, 1, 2, 3), (2, 6, 10, 14)),
        ((0, 1, 2, 3), (3, 7, 11, 15)),
    ]
);
impl_matmul!(
    4, 4, 4, 1,
    [
        ((0, 1, 2, 3), (0, 1, 2, 3)),
        ((4, 5, 6, 7), (0, 1, 2, 3)),
        ((8, 9, 10, 11), (0, 1, 2, 3)),
        ((12, 13, 14, 15), (0, 1, 2, 3)),
    ]
);

impl<'a, R> Dot<&'a Mat<R, 2, 1>> for &'a Mat<R, 1, 2>
where
    R: Num + Copy + Sum,
{
    type Output = R;

    fn dot(self, rhs: &'a Mat<R, 2, 1>) -> Self::Output {
        abop!(dot, self, rhs, [((0, 1), (0, 1))])[0]
    }
}

#[cfg(feature = "serde_support")]
impl<R, const I: usize, const J: usize> serde::ser::Serialize for Mat<R, I, J> 
where
    R: serde::ser::Serialize,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error> 
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut state = ser.serialize_seq(Some(I * J))?;
        for row in &self.0 {
            for cell in row {
                state.serialize_element(cell)?;
            }
        }
        state.end()
    }
}

#[cfg(feature = "serde_support")]
impl<'de, R, const I: usize, const J: usize> serde::de::Deserialize<'de> for Mat<R, I, J>
where
    R: Zero + Copy + serde::de::Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MatVisitor<R, const I: usize, const J: usize>(std::marker::PhantomData<[[R; J]; I]>);

        impl<R, const I: usize, const J: usize> Default for MatVisitor<R, I, J> {
            fn default() -> Self {
                MatVisitor(std::marker::PhantomData::default())
            }
        }
        
        impl<'v, R, const I: usize, const J: usize> serde::de::Visitor<'v> for MatVisitor<R, I, J> 
        where
            R: Zero + Copy + serde::de::Deserialize<'v>,
        {
            type Value = Mat<R, I, J>;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "A sequence with {} elements", I * J)
            }

            fn visit_seq<A>(self, mut acc: A) -> Result<Self::Value, A::Error> 
            where
                A: serde::de::SeqAccess<'v>,
            {
                use serde::de::Error;

                match acc.size_hint() {
                    Some(sh) if sh == I * J => (),
                    Some(sh) => {
                        return Err(Error::invalid_length(sh, &self));
                    },
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    },
                }

                let mut mat: Mat<R, I, J> = Mat::zero();

                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = acc.next_element()?.ok_or_else(|| Error::custom(format!("unexpected end of the serialized sequence of length {}", I * J)))?;
                    }
                }
                
                Ok(mat)
            }
        }

        de.deserialize_seq(MatVisitor::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn a2i_1x1() {
        assert_eq!(Mat::<f32, 1, 1>::as_2d_idx(0), (0, 0), "i=0");
    }

    #[test]
    fn a2i_1x2() {
        assert_eq!(Mat::<f32, 1, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 2>::as_2d_idx(1), (0, 1), "i=1");
    }

    #[test]
    fn a2i_1x3() {
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 1, 3>::as_2d_idx(2), (0, 2), "i=2");
    }

    #[test]
    fn a2i_1x4() {
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 1, 4>::as_2d_idx(3), (0, 3), "i=3");
    }

    #[test]
    fn a2i_2x1() {
        assert_eq!(Mat::<f32, 2, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 1>::as_2d_idx(1), (1, 0), "i=1");
    }

    #[test]
    fn a2i_2x2() {
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 2, 2>::as_2d_idx(3), (1, 1), "i=3");
    }

    #[test]
    fn a2i_2x3() {
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 2, 3>::as_2d_idx(5), (1, 2), "i=5");
    }

    #[test]
    fn a2i_2x4() {
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 2, 4>::as_2d_idx(7), (1, 3), "i=7");
    }

    #[test]
    fn a2i_3x1() {
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(1), (1, 0), "i=1");
        assert_eq!(Mat::<f32, 3, 1>::as_2d_idx(2), (2, 0), "i=2");
    }

    #[test]
    fn a2i_3x2() {
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(3), (1, 1), "i=3");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(4), (2, 0), "i=4");
        assert_eq!(Mat::<f32, 3, 2>::as_2d_idx(5), (2, 1), "i=5");
    }

    #[test]
    fn a2i_3x3() {
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(5), (1, 2), "i=5");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(6), (2, 0), "i=6");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(7), (2, 1), "i=7");
        assert_eq!(Mat::<f32, 3, 3>::as_2d_idx(8), (2, 2), "i=8");
    }

    #[test]
    fn a2i_3x4() {
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(7), (1, 3), "i=7");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(8), (2, 0), "i=8");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(9), (2, 1), "i=9");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(10), (2, 2), "i=10");
        assert_eq!(Mat::<f32, 3, 4>::as_2d_idx(11), (2, 3), "i=11");
    }

    #[test]
    fn a2i_4x1() {
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(1), (1, 0), "i=1");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(2), (2, 0), "i=2");
        assert_eq!(Mat::<f32, 4, 1>::as_2d_idx(3), (3, 0), "i=3");
    }

    #[test]
    fn a2i_4x2() {
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(2), (1, 0), "i=2");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(3), (1, 1), "i=3");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(4), (2, 0), "i=4");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(5), (2, 1), "i=5");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(6), (3, 0), "i=6");
        assert_eq!(Mat::<f32, 4, 2>::as_2d_idx(7), (3, 1), "i=7");
    }

    #[test]
    fn a2i_4x3() {
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(3), (1, 0), "i=3");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(4), (1, 1), "i=4");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(5), (1, 2), "i=5");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(6), (2, 0), "i=6");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(7), (2, 1), "i=7");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(8), (2, 2), "i=8");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(9), (3, 0), "i=9");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(10), (3, 1), "i=10");
        assert_eq!(Mat::<f32, 4, 3>::as_2d_idx(11), (3, 2), "i=11");
    }

    #[test]
    fn a2i_4x4() {
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(0), (0, 0), "i=0");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(1), (0, 1), "i=1");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(2), (0, 2), "i=2");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(3), (0, 3), "i=3");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(4), (1, 0), "i=4");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(5), (1, 1), "i=5");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(6), (1, 2), "i=6");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(7), (1, 3), "i=7");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(8), (2, 0), "i=8");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(9), (2, 1), "i=9");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(10), (2, 2), "i=10");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(11), (2, 3), "i=11");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(12), (3, 0), "i=12");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(13), (3, 1), "i=13");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(14), (3, 2), "i=14");
        assert_eq!(Mat::<f32, 4, 4>::as_2d_idx(15), (3, 3), "i=15");
    }

    #[test]
    fn mat_implements_display() {
        let a: Mat<f32, 2, 3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(format!("{}", a), "[[1, 2, 3], [4, 5, 6]]");

        let a: Mat<f32, 1, 2> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1, 2]]");

        let a: Mat<f32, 2, 1> = Mat::from([1.0f32, 2.0]);
        assert_eq!(format!("{}", a), "[[1], [2]]");
    }

    #[test]
    fn mat_implements_from_2d_array() {
        let _: Mat<f32, 2, 2> = Mat::from([[0.0, 1.0], [2.0, 3.0]]);
    }

    #[test]
    fn mat_1x1_implements_from_scalar_value() {
        let _: Mat<f32, 1, 1> = (1.0f32).into();
    }

    #[test]
    fn mat_implements_from_array() {
        let _: Mat<f32, 1, 1> = Mat::from([0.0f32; 1]);
        let _: Mat<f32, 1, 2> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, 1, 3> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, 1, 4> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 2, 1> = Mat::from([0.0f32; 2]);
        let _: Mat<f32, 2, 2> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 2, 3> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, 2, 4> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, 3, 1> = Mat::from([0.0f32; 3]);
        let _: Mat<f32, 3, 2> = Mat::from([0.0f32; 6]);
        let _: Mat<f32, 3, 3> = Mat::from([0.0f32; 9]);
        let _: Mat<f32, 3, 4> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, 4, 1> = Mat::from([0.0f32; 4]);
        let _: Mat<f32, 4, 2> = Mat::from([0.0f32; 8]);
        let _: Mat<f32, 4, 3> = Mat::from([0.0f32; 12]);
        let _: Mat<f32, 4, 4> = Mat::from([0.0f32; 16]);
    }

    #[test]
    fn mat_supports_1d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[2], 3.0f32);
    }

    #[test]
    fn mat_supports_mut_1d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[2] = 5.0f32;
        assert_eq!(m[2], 5.0f32);
    }

    #[test]
    fn mat_supports_2d_indexing() {
        let m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(m[(1, 1)], 4.0f32);
    }

    #[test]
    fn mat_supports_mut_2d_indexing() {
        let mut m: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        m[(1, 1)] = 5.0f32;
        assert_eq!(m[(1, 1)], 5.0f32);
    }

    #[test]
    fn mat_supports_transposition() {
        let a: Mat<f32, 2, 3> = Mat::from([1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b: Mat<f32, 3, 2> = a.t();
        assert_eq!(b, Mat::<f32, 3, 2>::from([1.0f32, 4.0, 2.0, 5.0, 3.0, 6.0]));
    }

    #[test]
    fn mat_provides_zero_constructor() {
        let m: Mat<f32, 2, 2> = Mat::zero();
        assert_eq!(m, Mat::<f32, 2, 2>::from([0.0f32; 4]));

        let m: Mat<f32, 2, 3> = Mat::zero();
        assert_eq!(m, Mat::<f32, 2, 3>::from([0.0f32; 6]));
    }

    #[test]
    fn mat_supports_one_constructor() {
        let m: Mat<f32, 2, 2> = Mat::one();
        assert_eq!(m, Mat::<f32, 2, 2>::from([1.0f32; 4]));

        let m: Mat<f32, 2, 3> = Mat::one();
        assert_eq!(m, Mat::<f32, 2, 3>::from([1.0f32; 6]));
    }

    #[test]
    fn mat_supports_identity_constructor() {
        let m: Mat<f32, 2, 2> = Mat::identity();
        assert_eq!(m, Mat::<f32, 2, 2>::from([1.0f32, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn mat_supports_scalar_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([3.0]));
    }

    #[test]
    fn mat_supports_scalar_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([-1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([1.0]));
    }

    #[test]
    fn mat_supports_scalar_multiplication() {
        let a: Mat<f32, 1, 1> = Mat::from([2.0]);
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Mat::<f32, 1, 1>::from([4.0]));
        assert_eq!(&b * &a, Mat::<f32, 1, 1>::from([4.0]));
    }

    #[test]
    fn mat_supports_scalar_division() {
        let a: Mat<f32, 1, 1> = Mat::from([6.0]);
        let b: f32 = 2.0;
        assert_eq!(&a / &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b / &a, Mat::<f32, 1, 1>::from([2.0 / 6.0]));
    }

    #[test]
    fn mat_supports_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([5.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([5.0]));
    }

    #[test]
    fn mat_supports_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([-1.0]));
    }

    #[test]
    fn mat_supports_dot_product_2x1_1x2() {
        let a: Mat<f32, 2, 1> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 1, 2> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x1() {
        let a: Mat<f32, 1, 2> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), 8.0f32);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x2() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 2> = Mat::from([2.0, 3.0, 4.0, 5.0]);
        let c: Mat<f32, 2, 2> = Mat::from([10.0, 13.0, 22.0, 29.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x2() {
        let a: Mat<f32, 1, 2> = Mat::from([2.0, 3.0]);
        let b: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let c: Mat<f32, 1, 2> = Mat::from([11.0, 16.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x1() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 3.0]);
        let c: Mat<f32, 2, 1> = Mat::from([8.0, 18.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x3() {
        let a: Mat<f32, 3, 3> = Mat::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 3, 3> = Mat::from([36., 42., 48., 81., 96., 111., 126., 150., 174.]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x3_3x3() {
        let a: Mat<f32, 1, 3> = Mat::from([1.0, 2.0, 3.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 1, 3> = Mat::from([36.0, 42.0, 48.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x1() {
        let a: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let b: Mat<f32, 3, 1> = Mat::from([1.0, 2.0, 3.0]);
        let c: Mat<f32, 3, 1> = Mat::from([20.0, 38.0, 56.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x4() {
        let a: Mat<f32, 4, 4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, 4, 4> = Mat::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let c: Mat<f32, 4, 4> = Mat::from([
            100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.,
        ]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x4_4x4() {
        let a: Mat<f32, 1, 4> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let c: Mat<f32, 1, 4> = Mat::from([100.0, 110.0, 120.0, 130.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x1() {
        let a: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let b: Mat<f32, 4, 1> = Mat::from([1.0, 2.0, 3.0, 3.0]);
        let c: Mat<f32, 4, 1> = Mat::from([35.0, 71.0, 107.0, 143.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat2_x_vec2_works_as_premultiplication_of_the_matrix() {
        let m: Mat2<f32> = Mat2::identity();
        let v: Vec2<f32> = Vec2::one();

        assert_eq!(m * v, Vec2::one());
    }

    #[test]
    fn mat3_x_vec3_works_as_premultiplication_of_the_matrix() {
        let m: Mat3<f32> = Mat3::identity();
        let v: Vec3<f32> = Vec3::one();

        assert_eq!(m * v, Vec3::one());
    }

    #[test]
    fn mat4_x_vec4_works_as_premultiplication_of_the_matrix() {
        let m: Mat4<f32> = Mat4::identity();
        let v: Vec4<f32> = Vec4::one();

        assert_eq!(m * v, Vec4::one());
    }

    #[test]
    fn mat_provides_is_nan() {
        let a: Mat<f32, 2, 2> = Mat::from([f32::NAN, 1.0, 1.0, 1.0]);
        assert!(a.is_nan());

        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 1.0, 1.0, 1.0]);
        assert!(!a.is_nan());
    }

    #[test]
    fn mat_provides_col_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 3.0]);
        let b: Mat<f32, 2, 1> = a.col(0);
        assert_eq!(b, Mat::<f32, 2, 1>::from([1.0f32, 3.0]));
    }

    #[test]
    fn mat_provides_row_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 1, 2> = a.row(0);
        assert_eq!(b, Mat::<f32, 1, 2>::from([1.0f32, 2.0]));
    }

    #[test]
    fn mat_provides_subset_method() {
        let a: Mat<f32, 4, 4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, 2, 2> = a.subset::<2, 2>(0, 1);
        assert_eq!(b, Mat::<f32, 2, 2>::from([2.0f32, 3.0, 6.0, 7.0]));
    }

    #[test]
    fn mat_provides_norm_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(a.norm(), 5.477225575051661f32);
    }

    #[test]
    fn mat_probides_diag_method() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(a.diag(), Vec2::new(1.0f32, 4.0f32));
    }

    #[test]
    fn vec2_implements_new() {
        let _: Vec2<f32> = Vec2::new(1.0f32, 2.0f32);
    }

    #[test]
    fn vec3_implements_new() {
        let _: Vec3<f32> = Vec3::new(1.0f32, 2.0f32, 3.0f32);
    }

    #[test]
    fn vec4_implements_new() {
        let _: Vec4<f32> = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
    }

    #[test]
    fn vec2_implements_x_and_y() {
        let v: Vec2<f32> = Vec2::new(1.0, 2.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
    }

    #[test]
    fn vec3_implements_x_y_and_z() {
        let v: Vec3<f32> = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
        assert_eq!(v.z(), 3.0f32);
    }

    #[test]
    fn vec4_implements_x_y_z_and_w() {
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x(), 1.0f32);
        assert_eq!(v.y(), 2.0f32);
        assert_eq!(v.z(), 3.0f32);
        assert_eq!(v.w(), 4.0f32);
    }

    #[test]
    fn mat_implements_serde() {
        let a: Mat<f32, 2, 2> = Mat::identity();

        assert_tokens(
            &a,
            &[
                Token::Seq { len: Some(4) },
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::SeqEnd,
            ],
        );
    }
}
