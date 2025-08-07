# sylva &emsp; [![Rustc Version 1.88+]][rustc] [![MiniZinc Version 2.9.3+]][minizinc]

[Rustc Version 1.88+]: https://img.shields.io/badge/rustc-1.88+-lightgray.svg?e&logo=rust&logoColor=white
[rustc]: https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/
[MiniZinc Version 2.9.3+]: https://img.shields.io/badge/minizinc-2.9.3+-lightgray.svg?e&logo=minizinc&logoColor=white
[minizinc]: https://www.python.org/downloads/release/python-360/

Tool suite for Application Level Synthesis.

## Compile and Install

1. Build Sylva

   ```bash
   ./run.sh
   ```

3. Create test example (minimum, copy, sobel, lenet5)

   ```bash
   ./bin/config --name {example} --output ./config/
   ```

## Usage

### Requirements

- [rustc](https://www.rust-lang.org/)
- [minizinc](https://www.minizinc.org/)
`
### Commands

#### `sylva`

```shell
Arguments to get the configuration files and output directory


Usage: sylva --graph <GRAPH> --constraint <CONSTRAINT_FILE> --library <ALIMP_LIB> --parameter <HYPER_PARAMETER> --technology <TECHNOLOGY_CONSTRAINT> --output <OUTPUT>

Options:
  -g, --graph <GRAPH>                       SDF graph file
  -c, --constraint <CONSTRAINT_FILE>        global constraint file
  -l, --library <ALIMP_LIB>                 alimp library file
  -p, --parameter <HYPER_PARAMETER>         hyper parameter file
  -t, --technology <TECHNOLOGY_CONSTRAINT>  technology constraint file
  -o, --output <OUTPUT>                     output directory
  -h, --help                                Print help
  -V, --version                             Print version
```

#### `sv-sim`

```shell
Arguments to locate JSON files

Usage: sv-sim --dir <DIR>

Options:
      --dir <DIR>  Directory containing the files
  -h, --help       Print help
  -V, --version    Print version
```

#### `config`

```shell
Usage: config --name <NAME> --output <DIR>

Options:
  -n, --name <NAME>   name of the generating example
  -o, --output <DIR>  output directory
  -h, --help          Print help
  -V, --version       Print version
```
