# swim-marlin

`swim-marlin` is a [Swim](https://gitlab.com/spade-lang/swim) subcommand that
makes it easy to manage Marlin tests in a Swim project. See `swim marlin --help`
for more information.

Install with `cargo install swim-marlin`.

Overview:

- `swim marlin init` inside a Swim project sets up Marlin
- `swim marlin add <testname>` creates a new empty test in your tests directory
- `swim marlin test <pattern>` runs all tests containing `<pattern>` in their
   name on multiple threads
- `swim marlin check` checks for an invalid configuration
