pub mod approx;
pub mod convert;
pub mod ops;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Unit<T>(T);

impl<T> Unit<T> {
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T> std::fmt::Display for Unit<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
