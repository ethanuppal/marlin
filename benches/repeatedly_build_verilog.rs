use std::fs;

use camino::Utf8Path;
use divan::Bencher;
use marlin::verilator::{VerilatorRuntime, VerilatorRuntimeOptions};
use marlin::verilog::prelude::*;

#[verilog(src = "examples/verilog-project/src/main.sv", name = "main")]
pub struct Main;

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 5, skip_ext_time = true)]
fn rebuild_and_test_verilog(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let bench_artifacts_directory =
                Utf8Path::new(env!("CARGO_TARGET_TMPDIR"))
                    .join("bench_artifacts");
            let _ = fs::remove_dir_all(&bench_artifacts_directory);
        })
        .bench_local_values(|_| {
            let bench_artifacts_directory =
                Utf8Path::new(env!("CARGO_TARGET_TMPDIR"))
                    .join("bench_artifacts");

            let runtime = VerilatorRuntime::new(
                &bench_artifacts_directory,
                &["examples/verilog-project/src/main.sv".as_ref()],
                &[],
                [],
                VerilatorRuntimeOptions::default(),
            )
            .unwrap();

            let mut main = runtime.create_model::<Main>().unwrap();

            main.medium_input = u32::MAX;
            assert_eq!(main.medium_output, 0);
            main.eval();
            assert_eq!(main.medium_output, u32::MAX);
        });
}
