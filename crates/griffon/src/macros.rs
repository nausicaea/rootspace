/// Logs a message at the trace level hidden behind the feature flag "dbg-gfx".
#[macro_export(local_inner_macros)]
macro_rules! trace_gfx {
    // trace_gfx!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace_gfx!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "dbg-gfx")]
        {
            tracing::log::log!(target: $target, tracing::log::Level::Trace, $($arg)+)
        }
    };

    // trace_gfx!("a {} event", "log")
    ($($arg:tt)+) => {
        #[cfg(feature = "dbg-gfx")]
        {
            tracing::log::log!(tracing::log::Level::Trace, $($arg)+)
        }
    };
}

