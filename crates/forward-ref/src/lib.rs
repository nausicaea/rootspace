#![warn(clippy::pedantic)]

/// Forward an implementation of a unary operator to combinations of references
#[macro_export]
macro_rules! forward_ref_unop {
    (impl $Op:ident, $op:ident for $Lhs:ty, $Output:ty) => {
        impl $Op for $Lhs {
            type Output = $Output;

            fn $op(self) -> Self::Output {
                $Op::$op(&self)
            }
        }
    };
    (impl<$(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Output:ty) => {
        impl<$(const $N: usize),+> $Op for $Lhs {
            type Output = $Output;

            fn $op(self) -> Self::Output {
                $Op::$op(&self)
            }
        }
    };
    (impl<$R:ident: $Bound:ident> $Op:ident, $op:ident for $Lhs:ty, $Output:ty) => {
        impl<$R> $Op for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self) -> Self::Output {
                $Op::$op(&self)
            }
        }
    };
    (impl<$R:ident: $Bound:ident, $(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Output:ty) => {
        impl<$R, $(const $N: usize),+> $Op for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self) -> Self::Output {
                $Op::$op(&self)
            }
        }
    };
}

/// Forward an implementation of a binary operator to combinations of references
#[macro_export]
macro_rules! forward_ref_binop {
    (impl $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(&self, &rhs)
            }
        }

        impl<'b> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                $Op::$op(&self, rhs)
            }
        }

        impl<'a> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(self, &rhs)
            }
        }
    };
    (impl<$(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$(const $N: usize),+> $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(&self, &rhs)
            }
        }

        impl<'b, $(const $N: usize),+> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                $Op::$op(&self, rhs)
            }
        }

        impl<'a, $(const $N: usize),+> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(self, &rhs)
            }
        }
    };
    (impl<$R:ident> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R> $Op<$Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(&self, &rhs)
            }
        }

        impl<'b, $R> $Op<&'b $Rhs> for $Lhs {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                $Op::$op(&self, rhs)
            }
        }

        impl<'a, $R> $Op<$Rhs> for &'a $Lhs {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(self, &rhs)
            }
        }
    };
    (impl<$R:ident: $Bound:ident> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R> $Op<$Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(&self, &rhs)
            }
        }

        impl<'b, $R> $Op<&'b $Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                $Op::$op(&self, rhs)
            }
        }

        impl<'a, $R> $Op<$Rhs> for &'a $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(self, &rhs)
            }
        }
    };
    (impl<$R:ident: $Bound:ident, $(const $N:ident: usize),+ $(,)*> $Op:ident, $op:ident for $Lhs:ty, $Rhs:ty, $Output:ty) => {
        impl<$R, $(const $N: usize),+> $Op<$Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(&self, &rhs)
            }
        }

        impl<'b, $R, $(const $N: usize),+> $Op<&'b $Rhs> for $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: &'b $Rhs) -> Self::Output {
                $Op::$op(&self, rhs)
            }
        }

        impl<'a, $R, $(const $N: usize),+> $Op<$Rhs> for &'a $Lhs
        where
            $R: $Bound,
        {
            type Output = $Output;

            fn $op(self, rhs: $Rhs) -> Self::Output {
                $Op::$op(self, &rhs)
            }
        }
    };
}
