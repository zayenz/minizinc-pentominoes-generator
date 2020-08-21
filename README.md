# Instance generator for pentominoes-like placement problems in MiniZinc


[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/zayenz/minizinc-pentominoes-generator//workflows/CI/badge.svg
[actions-url]: https://github.com/zayenz/minizinc-pentominoes-generator//actions?query=workflow%3A%22CI%22


Simple instance generator for pentominoes-like problems in MiniZinc.

The project contains a single executable minizinc-pentominoes-generator, that is invoked as follows

```
$ $ minizinc-pentominoes-generator --help
Usage: minizinc-pentominoes-generator --size <size> --tiles <tiles> [--seed <seed>] [-d] [--strategy <strategy>]

Generate instances for pentominoes-like MiniZinc problems

Options:
  --size            the width and height of the board
  --tiles           the number of tiles
  --seed            the random number seed to use (if absent, use system
                    entropy)
  -d, --debug       debug print the generated board
  --strategy        strategy to use for generating the board (source (default),
                    target, close, and far)
  --help            display usage information
```

The `model/` folder contains a model for the problem, and the `data/` folder contains a set of instances.

---

## Generated instances

The script folder contains a simple script to generate a set of instances, with the current output in the `data/` folder.
The script can be modified to generate different sets of instances.

---

## Installation

Clone this repository and build with a recent (>=1.42) version of Rust.

---

## License

Copyright 2020 <a href="https://zayenz.se" target="_blank">Mikael Zayenz Lagerkvist</a>.

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
[https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)>
or the MIT license <LICENSE-MIT or
[https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)>,
at your option. Files in the project may not be copied, modified, or
distributed except according to those terms.

