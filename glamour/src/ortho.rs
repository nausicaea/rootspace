#[derive(Debug, PartialEq, Clone)]
pub struct Ortho<R> {
    _r: std::marker::PhantomData<R>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrthoBuilder;
