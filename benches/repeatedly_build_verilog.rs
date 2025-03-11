use std::fs;

use camino::Utf8Path;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use marlin::{
    verilator::{VerilatorRuntime, VerilatorRuntimeOptions},
    verilog::prelude::*,
};

#[verilog(src = "examples/verilog-project/src/main.sv", name = "main")]
pub struct Main;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("rebuild_and_test_verilog", |b| {
        b.iter(|| {
            let bench_artifacts_directory =
                Utf8Path::new(env!("CARGO_TARGET_TMPDIR"))
                    .join("bench_artifacts");
            let _ = fs::remove_dir_all(&bench_artifacts_directory);

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

            let _main = black_box(&main);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
