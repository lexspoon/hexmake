Hexmake is set up as a standard Rust crate, so all of
the Cargo commands for a Rust crate will work.

Install Rust itself using
[the instructions on the Rust web site][rust-install].

Edit the code using any Rust tooling that you like.
I use the Cursor IDE with Rust-analyzer.

There are scripts in the top-level [scripts](../scripts)
folder that have the right command for some common tasks:

* `./scripts/install`. Build Hexmake and install it on your
  PATH.
* `./scripts/test`. Build Hexmake and run its test suite.


The integration tests assume that `cc` is on your PATH
and will act like a gcc, Clang, or similar Unix C compilers.

[rust-install]: https://rust-lang.org/tools/install/


