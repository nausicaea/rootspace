use ecs::{Reg, VecStorage, Resources};
use log::LevelFilter;
use fern::Dispatch;

/// The following test is used to examine a memory access bug in the scene loading
/// functionality of World, using a type with a VecStorage storage type.
///
/// # Expected Behavior
///
/// This test should _not_ throw a segmentation fault, but should instead panic.
#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Error(\"Not a registered type: BogusType\", line: 1, column: 12)")]
fn test_no_segfault() {
    Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{} @{}: {}", record.level(), record.target(), message)))
        .level(LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    pub type TestRegistry = Reg![
        VecStorage<usize>,
    ];

    let mut d = serde_json::Deserializer::from_str("{\"BogusType\":null}");

    Resources::deserialize::<TestRegistry, _>(&mut d)
        .unwrap();
}
