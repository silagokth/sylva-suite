# sylva &emsp; [![Rustc Version 1.88+]][rustc] [![MiniZinc Version 2.9.0+]][minizinc] [![riscv-gnu-toolchain 2026.03.13+]][riscv-gnu-toolchain] [![bender Version 0.30.0+]][bender] [![Questa Version 2023.4+]][questa] 

[Rustc Version 1.88+]: https://img.shields.io/badge/rustc-1.88+-lightgray.svg?e&logo=rust&logoColor=white
[rustc]: https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/
[MiniZinc Version 2.9.0+]: https://img.shields.io/badge/minizinc-2.9.0+-lightgray.svg?e&logo=minizinc&logoColor=white
[minizinc]: https://www.minizinc.org/
[Questa Version 2023.4+]: https://img.shields.io/badge/questa*-2023.4+-lightgray.svg?e&logo=questa&logoColor=white
[questa]: https://www.altera.com/products/development-tools/quartus-prime/questa
[riscv-gnu-toolchain 2026.03.13+]: https://img.shields.io/badge/riscv--gnu--toolchain-2026.03.13+-lightgray.svg?e&logo=gnu&logoColor=white
[riscv-gnu-toolchain]: https://github.com/riscv-collab/riscv-gnu-toolchain
[bender Version 0.30.0+]: https://img.shields.io/badge/bender-0.30.0+-lightgray.svg?e&logo=github&logoColor=white 
[bender]: https://github.com/pulp-platform/bender


Tool suite for Application-Level Synthesis (ALS).

## Overview

Sylva is the ALS layer of the [SiLago](https://silago.eecs.kth.se) framework. It
takes an application modelled as a **Homogeneous Synchronous Dataflow (HSDF)
graph**, together with a library of pre-characterised algorithm implementations
(**AlImp**s, produced by the Vesyla HLS tool), and maps it onto a layout-aware
network of hardware blocks. The generated system — a host processor plus a network
of AlImps connected by output buffers, transporters and input buffers — is built
from the RTL in [`sylva-components`](https://github.com/silagokth/sylva-components)
(included here as a git submodule).

The flow has two phases, each implemented as a binary:

1. **Design Space Exploration** (`sv-dse`) — binding, placement, routing,
   NoC wire synthesis and GLIC synthesis, optionally verified by the GLIC
   simulator (`sv-sim`). Produces an intermediate representation (`db.bin`).
2. **Assembly** (`sv-asm`) — memory synthesis, re-routing, NoC resynthesis, TLB
   code generation, transporter code generation and control synthesis. Turns the
   abstract result into a hardware-realisable design (`db_asm.bin`).

The four worked examples are `minimum`, `copy`, `sobel` and `lenet5`.

### Repository layout

| Path | Contents |
| --- | --- |
| `modules/` | Rust workspace: `sv-lib` (shared model), `sv-dse`, `sv-asm`, `sv-sim`, and the `config` example generator. |
| `config/` | Input configuration files for the current run. |
| `examples/` | Compiled example application binaries (created by `setup.sh`). |
| `bin/` | Built tool binaries (`sv-dse`, `sv-asm`, `sv-sim`, `config`). |
| `sylva-components/` | RTL components submodule used by Assembly's control synthesis. |

### Getting the sources

```bash
git clone --recurse-submodules https://github.com/silagokth/sylva-suite.git
# or, if already cloned:
git submodule update --init --recursive
```

## Dependencies

   ```bash
   apt install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev
   ```

## Compile and Install

1. Build Sylva

   ```bash
   ./setup.sh
   ```

3. Create test example (minimum, copy, sobel, and lenet5)  
   ```bash
   ./bin/config --name {example} --output ./config/
   ```

## Quick Run

   To run Sylva Design-Space-Exploration
   ```bash
   ./run_dse.sh
   ```

   To run Sylva assembly
   ```bash
   ./run_asm.sh
   ```

## Usage

This project is developed and tested on Linux (Ubuntu). It relies on standard Unix utilities and may not work on non-Unix systems without modification.

### Operating System
- Ubuntu 22.04 / 24.04 (or compatible Linux distribution)
  
### Requirements

- `bash`
- `coreutils` (provides `cp`, `mv`, `rm`, etc.)
- `make`
- [rustc](https://www.rust-lang.org/)
- [minizinc](https://www.minizinc.org/)
- [riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [bender](https://github.com/pulp-platform/bender)
- [vsim](https://www.altera.com/products/development-tools/quartus-prime/questa) 

### Commands

#### `sv-dse`

```shell
Arguments to get the configuration files and output directory

Usage: sv-dse [OPTIONS] --graph <GRAPH> --constraint <CONSTRAINT_FILE> --library <ALIMP_LIB> --parameter <HYPER_PARAMETER> --technology <TECHNOLOGY_CONSTRAINT> --output <OUTPUT>

Options:
      --cpu <CPU_LIMIT>                     Set CPU limit
      --memory <MEMORY_LIMIT>               Set memory limit in GB
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

#### `sv-asm`

```shell
Arguments to get the configuration files and output directory

Usage: sv-asm [OPTIONS] --intermediate-representation <IR_OBJECT> --sylva-components <SYLVA_COMPONENTS> --binary <OUTPUT>

Options:
      --cpu <CPU_LIMIT>                          Set CPU limit
      --memory <MEMORY_LIMIT>                    Set memory limit in GB
  -i, --intermediate-representation <IR_OBJECT>  Input Intermediate Representation Object
      --sylva-components <SYLVA_COMPONENTS>      Path to the Sylva components
  -o, --binary <OUTPUT>                          output directory
  -h, --help                                     Print help
  -V, --version                                  Print version
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

> Note: `sv-asm` additionally needs `--sylva-components <PATH>` pointing at the
> `sylva-components` submodule, which it uses during control synthesis. The
> provided `run_asm.sh` passes `./sylva-components/`.

## Documentation

Conceptual documentation of each synthesis step, the input file formats and the
hardware architecture lives in the SiLago documentation under
**ToolChain → Sylva ALS** (<https://silago.eecs.kth.se/docs>).
