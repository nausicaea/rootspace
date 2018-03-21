use ecs::world::Error as RootEcsError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unspecified Error")]
    UnspecifiedError,
    #[fail(display = "{}", _0)]
    EcsError(#[cause] RootEcsError),
}

impl From<RootEcsError> for Error {
    fn from(value: RootEcsError) -> Self {
        Error::EcsError(value)
    }
}
