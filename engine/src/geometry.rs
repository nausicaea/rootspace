use glium::Rect as GliumRect;
use std::fmt;
use num_traits::{Num, RefNum};
use rusttype::{Point as RusttypePoint, Rect as RusttypeRect};

#[derive(Debug, Clone)]
pub struct Point<N> {
    x: N,
    y: N,
}

impl<N> fmt::Display for Point<N>
where
    N: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone)]
pub struct Rect<N> {
    min: Point<N>,
    max: Point<N>,
}

impl<N> fmt::Display for Rect<N>
where
    N: fmt::Display + Num,
    for<'r> &'r N: RefNum<N>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (w, h) = (&self.max.x - &self.min.x, &self.max.y - &self.min.y);
        write!(f, "({}, {} x {})", self.min, w, h)
    }
}

impl<N> Rect<N>
where
    N: Num,
    for<'r> &'r N: RefNum<N>,
{
    pub fn dimensions(&self) -> [N; 2] {
        [&self.max.x - &self.min.x, &self.max.y - &self.min.y]
    }
}

impl From<GliumRect> for Rect<u32> {
    fn from(value: GliumRect) -> Self {
        Rect {
            min: Point {
                x: value.left,
                y: value.bottom,
            },
            max: Point {
                x: value.left + value.width,
                y: value.bottom + value.height,
            },
        }
    }
}

impl From<Rect<u32>> for GliumRect {
    fn from(value: Rect<u32>) -> Self {
        GliumRect {
            left: value.min.x,
            bottom: value.min.y,
            width: value.max.x - value.min.x,
            height: value.max.y - value.min.y,
        }
    }
}

impl<N: Num + PartialOrd> From<RusttypeRect<N>> for Rect<N> {
    fn from(value: RusttypeRect<N>) -> Self {
        let (min_x, max_x) = if value.min.x < value.max.x {
            (value.min.x, value.max.x)
        } else {
            (value.max.x, value.min.x)
        };

        let (min_y, max_y) = if value.min.y < value.max.y {
            (value.min.y, value.max.y)
        } else {
            (value.max.y, value.min.y)
        };

        Rect {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }
}

impl<N: Num> Into<RusttypeRect<N>> for Rect<N> {
    fn into(self) -> RusttypeRect<N> {
        RusttypeRect {
            min: RusttypePoint {
                x: self.min.x,
                y: self.min.y,
            },
            max: RusttypePoint {
                x: self.max.x,
                y: self.max.y,
            },
        }
    }
}
