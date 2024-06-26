`ogsolve` is the console-based program (by Piotr Beling) for solving [octal games](https://en.wikipedia.org/wiki/Octal_game).

Please run the program with the `--help` switch to see the available options.

# Installation

The program can be compiled and installed from sources. To do this, a Rust compiler is needed.
The easiest way to obtain the compiler along with other necessary tools (like `cargo`) is
to use [rustup](https://www.rust-lang.org/tools/install).

Once Rust is installed, to compile and install the program with native optimizations, just execute:

```RUSTFLAGS="-C target-cpu=native" cargo install ogsolve```