use std::fmt;

use glium::Rect as GliumRect;
use num_traits::{NumCast, RefNum};
use rusttype::{Point as RusttypePoint, Rect as RusttypeRect};

use super::point::Point;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect<N> {
    min: Point<N>,
    max: Point<N>,
}

impl<N> Rect<N> {
    pub fn new(min_x: N, min_y: N, max_x: N, max_y: N) -> Self {
        Rect {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }

    pub fn min(&self) -> &Point<N> {
        &self.min
    }

    pub fn max(&self) -> &Point<N> {
        &self.max
    }
}

impl<N> fmt::Display for Rect<N>
where
    N: fmt::Display + Num + Clone,
    for<'r> &'r N: RefNum<N>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (w, h) = (&self.max.x() - &self.min.x(), &self.max.y() - &self.min.y());
        write!(f, "({}, {:.2} x {:.2})", self.min, w, h)
    }
}

impl<N> Rect<N>
where
    N: Num + Clone,
    for<'r> &'r N: RefNum<N>,
{
    pub fn dimensions(&self) -> [N; 2] {
        [&self.max.x() - &self.min.x(), &self.max.y() - &self.min.y()]
    }
}

impl From<GliumRect> for Rect<u32> {
    fn from(value: GliumRect) -> Self {
        Rect {
            min: Point::new(value.left, value.bottom),
            max: Point::new(value.left + value.width, value.bottom + value.height),
        }
    }
}

impl From<Rect<u32>> for GliumRect {
    fn from(value: Rect<u32>) -> Self {
        GliumRect {
            left: value.min.x(),
            bottom: value.min.y(),
            width: value.max.x() - value.min.x(),
            height: value.max.y() - value.min.y(),
        }
    }
}

impl<N> From<RusttypeRect<N>> for Rect<N>
where
    N: Num + PartialOrd,
{
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
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }
}

impl<N> Into<RusttypeRect<N>> for Rect<N>
where
    N: Num + Clone,
{
    fn into(self) -> RusttypeRect<N> {
        RusttypeRect {
            min: RusttypePoint {
                x: self.min.x(),
                y: self.min.y(),
            },
            max: RusttypePoint {
                x: self.max.x(),
                y: self.max.y(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rusttype::point as rusttype_point;

    use super::*;

    #[test]
    fn new() {
        let _r: Rect<u32> = Rect::new(1, 2, 20, 20);
    }

    #[test]
    fn clone() {
        let r: Rect<u32> = Rect::new(1, 2, 20, 20);
        let _s: Rect<u32> = r.clone();
    }

    #[test]
    fn equality() {
        let r: Rect<u32> = Rect::new(2, 3, 21, 20);

        assert_eq!(
            r,
            Rect {
                min: Point::new(2, 3),
                max: Point::new(21, 20)
            }
        );
    }

    #[test]
    fn accessors() {
        let r: Rect<u32> = Rect::new(2, 3, 5, 3);

        assert_eq!(r.min(), &Point::new(2, 3));
        assert_eq!(r.max(), &Point::new(5, 3));
        assert_eq!(r.dimensions(), [3, 0]);
    }

    #[test]
    fn from_glium_rect() {
        let r = GliumRect {
            left: 1,
            bottom: 2,
            width: 4,
            height: 5,
        };
        let s: Rect<u32> = r.into();
        assert_eq!(s, Rect::new(1, 2, 5, 7));
    }

    #[test]
    fn into_glium_rect() {
        let r = Rect::new(4u32, 2u32, 10u32, 11u32);
        let s: GliumRect = r.into();
        assert_eq!(
            s,
            GliumRect {
                left: 4,
                bottom: 2,
                width: 6,
                height: 9
            }
        );
    }

    #[test]
    fn from_rusttype_rect() {
        let r = RusttypeRect {
            min: rusttype_point(1.0f32, 2.0f32),
            max: rusttype_point(32.0f32, 32.0f32),
        };
        let s: Rect<f32> = r.into();
        assert_eq!(s, Rect::new(1.0f32, 2.0f32, 32.0f32, 32.0f32));
    }

    #[test]
    fn into_rusttype_rect() {
        let r = Rect::new(1u32, 2u32, 3u32, 4u32);
        let s: RusttypeRect<u32> = r.into();
        assert_eq!(
            s,
            RusttypeRect {
                min: rusttype_point(1, 2),
                max: rusttype_point(3, 4)
            }
        );
    }
}
