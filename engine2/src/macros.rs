#[macro_export(local_inner_macros)]
macro_rules! spam {
    // spam!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // spam!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "dbg")]
        log::log!(target: $target, log::Level::Trace, $($arg)+)
    };

    // spam!("a {} event", "log")
    ($($arg:tt)+) => {
        #[cfg(feature = "dbg")]
        log::log!(log::Level::Trace, $($arg)+)
    };
}
