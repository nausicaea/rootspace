use either::Either;

#[derive(Debug)]
pub struct TypedSystem<'a, S> {
    pub order: usize,
    pub system: Either<&'a S, S>,
}
