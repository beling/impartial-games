`igsolve` is the console-based program (by Piotr Beling) for solving [impartial games](https://en.wikipedia.org/wiki/Impartial_game).
Currently, only the [normal play convention](https://en.wikipedia.org/wiki/Normal_play_convention) is supported, but support for [misère games](https://en.wikipedia.org/wiki/Mis%C3%A8re#Mis%C3%A8re_game) is planned.

Please run the program with the` --help` switch to see the available options (including the list of supported games).

# Installation

The program can be compiled and installed from sources. To do this, a Rust compiler is needed.
The easiest way to obtain the compiler along with other necessary tools (like `cargo`) is
to use [rustup](https://www.rust-lang.org/tools/install).

Once Rust is installed, to compile and install the program with native optimizations, just execute:

```RUSTFLAGS="-C target-cpu=native" cargo install igsolve```