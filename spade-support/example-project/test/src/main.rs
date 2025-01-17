use snafu::Whatever;
use spade::{spade, SpadeRuntime};

#[spade(src = "src/main.spade", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    colog::init();

    let mut runtime = SpadeRuntime::new("artifacts".into(), true, true)?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);
    assert_eq!(main.out, 42); // hardcoded into Spade source

    Ok(())
}
