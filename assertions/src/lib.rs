//! This crate provides assertions for testing `Result` and `Option` types.
#![deny(missing_docs)]

/// Asserts that the supplied expression is a `Option::Some(_)` value. Otherwise,
/// panics and prints an appropriate message.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// let some: Option<u32> = Some(100);
/// assert_some!(some);
/// ```
#[macro_export]
macro_rules! assert_some {
    ($option:expr) => {
        assert!($option.is_some(), "Expected 'Some(_)', got 'None'");
    };
    ($option:expr,) => {
        assert_some!($option);
    };
}

/// Asserts that the supplied expression is a `Option::None` value. Otherwise,
/// panics and prints an appropriate message.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// let none: Option<u32> = None;
/// assert_none!(none);
/// ```
#[macro_export]
macro_rules! assert_none {
    ($option:expr) => {
        assert!(
            $option.is_none(),
            "Expected 'None', got 'Some({:?})'",
            $option.unwrap()
        );
    };
    ($option:expr,) => {
        assert_none!($option);
    };
}

/// Asserts that the supplied expression is a `Result::Ok(_)` value. Otherwise,
/// panics and prints an appropriate message.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// let ok: Result<u32, String> = Ok(100);
/// assert_ok!(ok);
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        assert!(
            $result.is_ok(),
            "Expected 'Ok(_)', got 'Err({:?})'",
            $result.unwrap_err()
        );
    };
    ($result:expr,) => {
        assert_ok!($result);
    };
}

/// Asserts that the supplied expression is a `Result::Err(_)` value. Otherwise,
/// panics and prints an appropriate message.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// let err: Result<u32, String> = Err("Hello".into());
/// assert_err!(err);
/// ```
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        assert!(
            $result.is_err(),
            "Expected 'Err(_)', got 'Ok({:?})'",
            $result.unwrap()
        );
    };
    ($result:expr,) => {
        assert_err!($result);
    };
}

/// Asserts that the supplied expression is a `Result::Ok(_)` value. Functions the same as
/// `assert_ok!`, but does not require that the error type have the `Debug` trait.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// struct NoDebug(u32);
///
/// let ok: Result<u32, NoDebug> = Ok(100);
/// assert_ok2!(ok);
/// ```
#[macro_export]
macro_rules! assert_ok2 {
    ($result:expr) => {
        assert!(
            $result.is_ok(),
            "Expected 'Ok(_)', got 'Err(_)'"
        );
    };
    ($result:expr,) => {
        assert_ok2!($result);
    }
}

/// Asserts that the supplied expression is a `Result::Err(_)` value. Functions the same as
/// `assert_err!`, but does not require that the success type have the `Debug` trait.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate assertions;
///
/// struct NoDebug(u32);
///
/// let err: Result<u32, NoDebug> = Err(NoDebug(100));
/// assert_err2!(err);
/// ```
#[macro_export]
macro_rules! assert_err2 {
    ($result:expr) => {
        assert!(
            $result.is_err(),
            "Expected 'Err(_)', got 'Ok(_)'"
        );
    };
    ($result:expr,) => {
        assert_err2!($result);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn assert_some() {
        let some: Option<u32> = Some(100);

        assert_some!(some);
    }

    #[test]
    #[should_panic(expected = "Expected 'Some(_)', got 'None'")]
    fn assert_some_fail() {
        let none: Option<u32> = None;

        assert_some!(none);
    }

    #[test]
    fn assert_none() {
        let none: Option<u32> = None;

        assert_none!(none);
    }

    #[test]
    #[should_panic(expected = "Expected 'None', got 'Some(100)'")]
    fn assert_none_fail() {
        let some: Option<u32> = Some(100);

        assert_none!(some);
    }

    #[test]
    fn assert_ok() {
        let ok: Result<u32, String> = Ok(100);

        assert_ok!(ok);
    }

    #[test]
    #[should_panic(expected = "Expected 'Ok(_)', got 'Err(\"Hello\")'")]
    fn assert_ok_fail() {
        let err: Result<u32, String> = Err("Hello".into());

        assert_ok!(err);
    }

    #[test]
    fn assert_err() {
        let err: Result<u32, String> = Err("Hello".into());

        assert_err!(err);
    }

    #[test]
    #[should_panic(expected = "Expected 'Err(_)', got 'Ok(100)'")]
    fn assert_err_fail() {
        let ok: Result<u32, String> = Ok(100);

        assert_err!(ok);
    }

    #[test]
    fn assert_ok2() {
        struct NoDebug(u32);

        let ok: Result<u32, NoDebug> = Ok(100);

        assert_ok2!(ok);
    }

    #[test]
    #[should_panic(expected = "Expected 'Ok(_)', got 'Err(_)'")]
    fn assert_ok2_fail() {
        struct NoDebug(u32);

        let err: Result<u32, NoDebug> = Err(NoDebug(100));

        assert_ok2!(err);
    }

    #[test]
    fn assert_err2() {
        struct NoDebug(u32);

        let err: Result<u32, NoDebug> = Err(NoDebug(100));

        assert_err2!(err);
    }

    #[test]
    #[should_panic(expected = "Expected 'Err(_)', got 'Ok(_)'")]
    fn assert_err2_fail() {
        struct NoDebug(u32);

        let ok: Result<u32, NoDebug> = Ok(100);

        assert_err2!(ok);
    }
}
