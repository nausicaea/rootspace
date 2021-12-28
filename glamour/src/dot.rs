use num_traits::Num;

pub trait Dot<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> Self::Output;
}

impl<R> Dot for [R; 2]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        (&self).dot(&rhs)
    }
}

impl<'a, R> Dot for &'a [R; 2]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        self[0] * rhs[0] + self[1] * rhs[1]
    }
}

impl<R> Dot for [R; 3]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        (&self).dot(&rhs)
    }
}

impl<'a, R> Dot for &'a [R; 3]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2]
    }
}

impl<R> Dot for [R; 4]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        (&self).dot(&rhs)
    }
}

impl<'a, R> Dot for &'a [R; 4]
where
    R: Num + Copy,
{
    type Output = R;

    fn dot(self, rhs: Self) -> Self::Output {
        self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2] + self[3] * rhs[3]
    }
}
