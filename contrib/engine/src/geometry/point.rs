use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point<N> {
    x: N,
    y: N,
}

impl<N> Point<N> {
    pub fn new(x: N, y: N) -> Self {
        Point { x, y }
    }
}

impl<N> Point<N>
where
    N: Clone,
{
    pub fn x(&self) -> N {
        self.x.clone()
    }

    pub fn y(&self) -> N {
        self.y.clone()
    }
}

impl<N> From<(N, N)> for Point<N> {
    fn from(value: (N, N)) -> Self {
        Point { x: value.0, y: value.1 }
    }
}

impl<N> From<[N; 2]> for Point<N>
where
    N: Clone,
{
    fn from(value: [N; 2]) -> Self {
        Point {
            x: value[0].clone(),
            y: value[1].clone(),
        }
    }
}

impl<N> fmt::Display for Point<N>
where
    N: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: Point<u32> = Point::new(2, 3);
    }

    #[test]
    fn clone() {
        let p: Point<u32> = Point::new(1, 10);
        let _q: Point<u32> = p.clone();
    }

    #[test]
    fn equality() {
        let p: Point<u32> = Point::new(5, 4);

        assert_eq!(p, Point { x: 5, y: 4 });
    }

    #[test]
    fn accessors() {
        let p: Point<u32> = Point::new(2, 9);

        assert_eq!(p.x(), 2);
        assert_eq!(p.y(), 9);
    }

    #[test]
    fn from_tuple() {
        let _p: Point<f32> = Into::into((2f32, 6f32));
    }

    #[test]
    fn from_array() {
        let _p: Point<u32> = Into::into([2u32; 2]);
    }
}
