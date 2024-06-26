Rust libraries and programs for solving [impartial games](https://en.wikipedia.org/wiki/Impartial_game) and calculating their [nimbers](https://en.wikipedia.org/wiki/Nimber), developed by [Piotr Beling](http://pbeling.w8.pl/).
Currently, only the [normal play convention](https://en.wikipedia.org/wiki/Normal_play_convention) is supported, but support for [misère games](https://en.wikipedia.org/wiki/Mis%C3%A8re#Mis%C3%A8re_game) is planned.

Included libraries:
- `igs` ([crate](https://crates.io/crates/igs), [doc](https://docs.rs/igs)) - solves [impartial games](https://en.wikipedia.org/wiki/Impartial_game) under the [normal play convention](https://en.wikipedia.org/wiki/Normal_play_convention);
- `ogs` ([crate](https://crates.io/crates/ogs), [doc](https://docs.rs/ogs)) - solves [octal games](https://en.wikipedia.org/wiki/Octal_game).

Included programs:
- `igsolve` ([crate](https://crates.io/crates/igsolve), [doc](https://docs.rs/igsolve)) - a console-based application for calculating nimbers with `igs`;
- `ogsolve` ([crate](https://crates.io/crates/ogsolve), [doc](https://docs.rs/ogsolve)) - a console-based application for calculating nimbers with `ogs`.

# Installation
Programs can be compiled and installed from sources. To do this, a Rust compiler is needed.
The easiest way to obtain the compiler along with other necessary tools (like `cargo`) is
to use [rustup](https://www.rust-lang.org/tools/install).

Once Rust is installed, to compile and install a program with native optimizations, just execute:

```RUSTFLAGS="-C target-cpu=native" cargo install <program_name>```

for example

```RUSTFLAGS="-C target-cpu=native" cargo install igsolve```

# The results obtained so far
Here I provide the nimbers calculated using my software.

## Cram (under the normal play convention)

|       | 4 | 5 | 6 | 7 |   8   |   9   |   10  |   11  |
|------:|:-:|:-:|:-:|:-:|:-----:|:-----:|:-----:|:-----:|
| **4** | 0 | 2 | 0 | 3 |   0   |   1   |   0   | **1** |
| **5** | - | 0 | 2 | 1 |   1   |   1   | **2** |   0   |
| **6** | - | - | 0 | 5 |   0   | **1** |   0   |   ?   |
| **7** | - | - | - | 1 | **3** | **1** |   ?   |   ?   |

The table shows the nimbers of initial [cram](<https://en.wikipedia.org/wiki/Cram_(game)>) positions for different board sizes computed so far.
Note that the symmetry strategy implies that even-by-even boards are losing and therefore of nimber 0 (the second player can win by responding with moves symmetrical to the center of the board).

To the best of my knowledge, the bold values (for the largest boards: $9 \times 7$, $8 \times 7$, $9 \times 6$, $10 \times 5$, $11 \times 4$) were calculated by me and published here for the first time.

The nimbers of the smaller boards were earlier computed by [Glop](http://sprouts.tuxfamily.org/wiki/doku.php?id=records), which is the solver developed by Lemoine and Viennot.
Nimbers of most boards no larger than $5 \times 7$ were first given by Martin Schneider in his master's thesis entitled *Das spiel juvavum* in 2009.
Uiterwijk reported that the $11 \times 5$ board is losing (and thus of nimber 0) in his paper [*Solving Cram Using Combinatorial Game Theory* (Advances in Computer Games: 16th International Conference, ACG 2019, Macao, China, August 11–13, 2019)](https://dl.acm.org/doi/10.1007/978-3-030-65883-0_8).
I have verified the correctness of all the nimbers in the table with my solver.

# Publications and citations
When using my software for research purposes, please cite the following paper which details the key algorithms used:
* Piotr Beling, Marek Rogalski, *On pruning search trees of impartial games*, Artificial Intelligence, Volume 283, 2020, 103262, ISSN 0004-3702,
https://doi.org/10.1016/j.artint.2020.103262