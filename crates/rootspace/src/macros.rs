/// Logs a message at the trace level hidden behind the feature flag "dbg-loop".
#[macro_export(local_inner_macros)]
macro_rules! trace_loop {
    // trace_loop!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace_loop!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "dbg-loop")]
        {
            tracing::log::log!(target: $target, tracing::log::Level::Trace, $($arg)+)
        }
    };

    // trace_loop!("a {} event", "log")
    ($($arg:tt)+) => {
        #[cfg(feature = "dbg-loop")]
        {
            tracing::log::log!(tracing::log::Level::Trace, $($arg)+)
        }
    };
}
