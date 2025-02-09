use marlin::spade::prelude::*;
use snafu::Whatever;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = SpadeRuntime::new(SpadeRuntimeOptions::default(), true)?;

    // testing code here...

    Ok(())
}
