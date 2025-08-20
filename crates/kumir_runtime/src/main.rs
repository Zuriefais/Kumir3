use std::sync::Arc;

use kumir_runtime::{Runtime, Target, console_runtime_requirements::ConsoleRuntimeRequirements};
use log::info;

pub fn main() {
    env_logger::init();
    info!("Starting runtime");
    let mut target = Target::init(
        Arc::new(ConsoleRuntimeRequirements {}),
        kumir_runtime::Lang::Kumir,
        include_str!("test.kum").to_string(),
    )
    .unwrap();
    target.run().unwrap();
}
