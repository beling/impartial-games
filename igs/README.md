`igs` (*impartial game solver*) is the Rust library by Piotr Beling for solving [impartial games](https://en.wikipedia.org/wiki/Impartial_game). Currently, only the [normal play convention](https://en.wikipedia.org/wiki/Normal_play_convention) is supported, but support for [misère games](https://en.wikipedia.org/wiki/Mis%C3%A8re#Mis%C3%A8re_game) is planned.

`igs` can determine both an outcome class (i.e. a player with a winning strategy) and a [nimber](https://en.wikipedia.org/wiki/Nimber) of any game position. The solver is highly configurable and can use many advanced techniques to speed up calculations, including:
*  Pruning branches of search tree using the methods described in:
   *  P. Beling, M, Rogalski, *On pruning search trees of impartial games*, Artificial Intelligence 283 (2020), doi: [10.1016/j.artint.2020.103262](https://doi.org/10.1016/j.artint.2020.103262);
   *  J. Lemoine, S. Viennot, *Nimbers are inevitable*, Theoretical Computer Science 462 (2012) 70–79, doi: [10.1016/j.tcs.2012.09.002](https://doi.org/10.1016/j.tcs.2012.09.002).
*  Independent analysis of the components of decomposable game positions through the [Sprague–Grundy theorem](https://en.wikipedia.org/wiki/Sprague%E2%80%93Grundy_theorem).
*  A [transposition table](https://en.wikipedia.org/wiki/Transposition_table) that uses hashing (various implementations are available, including very compact ones) and can optionally be periodically saved to disk, allowing the calculation to be resumed after interruption.
*  An [endgame database](https://en.wikipedia.org/wiki/Endgame_tablebase) that uses very little space thanks to methods based on [perfect hashing](https://en.wikipedia.org/wiki/Perfect_hash_function), [huffman compression](https://en.wikipedia.org/wiki/Huffman_coding) or integer compression.
*  Game-specific methods, such as heuristic move sorting.

`igs` has built-in support for the following games: [Cram](<https://en.wikipedia.org/wiki/Cram_(game)>), [Chomp](https://en.wikipedia.org/wiki/Chomp) (2 models), [Grundy's game](https://en.wikipedia.org/wiki/Grundy%27s_game). Adding support for other games comes down to implementing the appropriate trait.