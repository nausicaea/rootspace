use super::Mat;

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

