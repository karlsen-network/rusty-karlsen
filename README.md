# Karlsen On Rust

[![Build Status](https://github.com/karlsen-network/rusty-karlsen/actions/workflows/ci.yaml/badge.svg)](https://github.com/karlsen-network/rusty-karlsen/actions/workflows/ci.yaml)
[![GitHub release](https://img.shields.io/github/v/release/karlsen-network/rusty-karlsen.svg)](https://github.com/karlsen-network/rusty-karlsen/releases)
[![GitHub license](https://img.shields.io/github/license/karlsen-network/rusty-karlsen.svg)](https://github.com/karlsen-network/rusty-karlsen/blob/main/LICENSE)
[![GitHub downloads](https://img.shields.io/github/downloads/karlsen-network/rusty-karlsen/total.svg)](https://github.com/karlsen-network/rusty-karlsen/releases)
[![Join the Karlsen Discord Server](https://img.shields.io/discord/1169939685280337930.svg?label=&logo=discord&logoColor=ffffff)](https://discord.gg/ZPZRvgMJDT)

Welcome to the Rust-based implementation of the Karlsen full-node and
its ancillary libraries. This production release serves as a drop-in
replacement to the established [Golang node](https://github.com/karlsen-network/karlsend),
introducing developers to the possibilities of Rust in the Karlsen
network's context.

We invite developers and blockchain enthusiasts to collaborate, test,
and optimize our Rust implementation. Each line of code here is an
opportunity to contribute to the open-source blockchain movement,
shaping a platform designed for scalability and speed without
compromising on decentralization.

Your feedback, contributions, and issue reports will be integral to
evolving this codebase from its Alpha phase into a mature and reliable
node in the Karlsen network.

## Overview

Karlsen is a fork of [Kaspa](https://github.com/kaspanet/rusty-kaspa)
introducing a GPU-centric fork as a solution to the dominance of ASIC
mining farms, aiming to empower small-scale miners and enhance
decentralization. We focus on bridging the gap between blockchain
technology, decentralized finance and the real world of payment systems
and traditional finance.

With Kaspa, our approach is one of friendly cohabitation. We operate
under the same protocol, enjoy similar advantages, and face
unidentified issues. Any significant improvements that need to be made
in the primary codebase will be shared back.

## Small Scale Miners

The Karlsen Network team believes in decentralization and small-scale
miners. We will ensure long-term GPU-friendly mining.

### Hashing Function

We initially started with `kHeavyHash` and `blake3` modifications
on-top. This algorithm is called `KarlsenHashv1`.

`KarlsenHashv1` is currently used in [mainnet](https://github.com/karlsen-network/karlsend/releases/tag/v1.1.0)
and can be mined using the following miners maintained by the Karlsen
developers:

* Karlsen [CPU miner](https://github.com/karlsen-network/karlsend) from Golang `karlsend`
* Karlsen [GPU miner](https://github.com/karlsen-network/karlsen-miner)
  as reference implementation of `kHeavyHash` with `blake3`.

The following third-party miners are available and have added
`KarlsenHashv1`:

* [lolMiner](https://github.com/Lolliedieb/lolMiner-releases)
* [Team Red Miner](https://github.com/todxx/teamredminer)
* [SRBMiner](https://github.com/doktor83/SRBMiner-Multi)
* [BzMiner](https://github.com/bzminer/bzminer)
* [Rigel](https://github.com/rigelminer/rigel)
* [GMiner](https://github.com/develsoftware/GMinerRelease)

`KarlsenHashv2` will become active via hardfork at DAA score `26.962.009`.
It is based on [FishHash](https://github.com/iron-fish/fish-hash/blob/main/FishHash.pdf)
written from scratch in our Rust node implementation. It is FPGA/ASIC
resistent. It is the worlds first implementation of FishHash in Rust in
`mainnet` in a 1bps blockchain.

`KarlsenHashv2` is currently used in [mainnet](https://github.com/karlsen-network/karlsend/releases/tag/v2.1.0)
and can be mined using the following miners maintained by the Karlsen
developers:

* Karlsen [CPU miner](https://github.com/karlsen-network/karlsend/releases/tag/v2.1.0) from Golang `karlsend`
* Karlsen [GPU miner](https://github.com/karlsen-network/karlsen-miner/releases/tag/v2.0.0)
  as bleeding edge and unoptimized reference implementation of
  `KarlsenHashv2`. Please follow the steps in the [README.md](https://github.com/karlsen-network/karlsen-miner/blob/main/README.md)
  to generate a DAG file.

The following third-party miners are available and have added
`KarlsenHashv2`:

* [SRBMiner](https://github.com/doktor83/SRBMiner-Multi)

## Smart Contracts

The Karlsen Network team is launching an R&D project to connect the
Karlsen blockchain with other blockchain ecosystems using a smart
contract layer based on the [Cosmos SDK](https://v1.cosmos.network/sdk).

This initiative aims to enhance interoperability, efficiency, and
innovation in the blockchain space. By leveraging the Cosmos SDK's
advanced features, we'll facilitate seamless transactions across
different networks, offering new opportunities for users and
developers.

### Cosmos Hub

[Cosmos](https://cosmos.network/) is a highly attractive ecosystem due
to its innovative approach to blockchain interoperability, scalability,
and usability. By enabling different blockchains to seamlessly
communicate and exchange value, Cosmos opens up vast opportunities for
businesses and developers to build and deploy decentralized
applications that can operate across multiple blockchain environments.
This interoperability fosters a more connected and efficient digital
economy, potentially driving adoption and usage across various sectors,
including finance, supply chain, and beyond.

By connecting to Cosmos, we will open the door of a web3 ecosystem
connected to a complete network of other blockchain project, making
Karlsen Network more competitive and adaptable in the rapidly evolving
landscape of decentralized technologies.

### Karlsen Sidechain

The creation of the Karlsen [sidechain](https://github.com/john-light/sidechains)
with fast transaction times, smart contract capabilities, and a dual
coin model, designed to integrate seamlessly with the Cosmos ecosystem
and utilize the Karlsen (KLS) across all interconnected platforms,
signifies a strategic advancement in Karlsen Network.

This sidechain will not only enable quick and efficient
inter-blockchain transactions but also support complex decentralized
applications through its smart contract functionality.

The ability to use Karlsen (KLS) across the entire ecosystem will
ensure a unified and streamlined user experience, promoting greater
adoption and utility within the Cosmos network. This sidechain aims
to enhance scalability, foster innovation, and provide a flexible and
user-centric blockchain solution that meets the diverse needs of
developers, users, and investors within the Cosmos ecosystem.

## Installation

### Building on Linux
  
1. Install general prerequisites

   ```bash
   sudo apt install curl git build-essential libssl-dev pkg-config
   ```

2. Install Protobuf (required for gRPC)
  
   ```bash
   sudo apt install protobuf-compiler libprotobuf-dev #Required for gRPC
   ```

3. Install the clang toolchain (required for RocksDB and WASM secp256k1
   builds)

   ```bash
   sudo apt-get install clang-format clang-tidy \
   clang-tools clang clangd libc++-dev \
   libc++1 libc++abi-dev libc++abi1 \
   libclang-dev libclang1 liblldb-dev \
   libllvm-ocaml-dev libomp-dev libomp5 \
   lld lldb llvm-dev llvm-runtime \
   llvm python3-clang
   ```

3. Install the [rust toolchain](https://rustup.rs/)
     
   If you already have rust installed, update it by running:
   `rustup update`

4. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

4. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

5. Clone the repo

   ```bash
   git clone https://github.com/karlsen-network/rusty-karlsen
   cd rusty-karlsen
   ```

### Building on Windows

1. [Install Git for Windows](https://gitforwindows.org/) or an alternative Git distribution.

2. Install [Protocol Buffers](https://github.com/protocolbuffers/protobuf/releases/download/v21.10/protoc-21.10-win64.zip) and add the `bin` directory to your `Path`

3. Install [LLVM-15.0.6-win64.exe](https://github.com/llvm/llvm-project/releases/download/llvmorg-15.0.6/LLVM-15.0.6-win64.exe)

   Add the `bin` directory of the LLVM installation
   (`C:\Program Files\LLVM\bin`) to PATH
    
   Set `LIBCLANG_PATH` environment variable to point to the `bin`
   directory as well

   **IMPORTANT:** Due to C++ dependency configuration issues, LLVM
   `AR` installation on Windows may not function correctly when
   switching between WASM and native C++ code compilation (native
   `RocksDB+secp256k1` vs WASM32 builds of `secp256k1`). Unfortunately,
   manually setting `AR` environment variable also confuses C++ build
   toolchain (it should not be set for native but should be set for
   WASM32 targets). Currently, the best way to address this, is as
   follows: after installing LLVM on Windows, go to the target `bin`
   installation directory and copy or rename `LLVM_AR.exe` to `AR.exe`.
  
4. Install the [rust toolchain](https://rustup.rs/)
     
   If you already have rust installed, update it by running:
   `rustup update`

5. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

6. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

7. Clone the repo

   ```bash
   git clone https://github.com/karlsen-network/rusty-karlsen
   cd rusty-karlsen
   ```

### Building on Mac OS

1. Install Protobuf (required for gRPC)

   ```bash
   brew install protobuf
   ```

2. Install llvm.
  
   The default XCode installation of `llvm` does not support WASM
   build targets. To build WASM on MacOS you need to install `llvm`
   from homebrew (at the time of writing, the llvm version for MacOS
   is 16.0.1).

   ```bash
   brew install llvm
   ```

   **NOTE:** Homebrew can use different keg installation locations
   depending on your configuration. For example:

   - `/opt/homebrew/opt/llvm` -> `/opt/homebrew/Cellar/llvm/16.0.1`
   - `/usr/local/Cellar/llvm/16.0.1`

   To determine the installation location you can use `brew list llvm`
   command and then modify the paths below accordingly:

   ```bash
   % brew list llvm
   /usr/local/Cellar/llvm/16.0.1/bin/FileCheck
   /usr/local/Cellar/llvm/16.0.1/bin/UnicodeNameMappingGenerator
   ...
   ```

   If you have `/opt/homebrew/Cellar`, then you should be able to use
   `/opt/homebrew/opt/llvm`.

   Add the following to your `~/.zshrc` file:

   ```bash
   export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
   export LDFLAGS="-L/opt/homebrew/opt/llvm/lib"
   export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
   export AR=/opt/homebrew/opt/llvm/bin/llvm-ar
   ```

   Reload the `~/.zshrc` file

   ```bash
   source ~/.zshrc
   ```

3. Install the [rust toolchain](https://rustup.rs/)
     
   If you already have rust installed, update it by running:
   `rustup update`

4. Install wasm-pack

   ```bash
   cargo install wasm-pack
   ```

4. Install wasm32 target

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

5. Clone the repo

   ```bash
   git clone https://github.com/karlsen-network/rusty-karlsen
   cd rusty-karlsen
   ```

### Building WASM32 SDK

Rust WebAssembly (WASM) refers to the use of the Rust programming
language to write code that can be compiled into WebAssembly, a binary
instruction format that runs in web browsers and NodeJs. This allows
for easy development using JavaScript and TypeScript programming
languages while retaining the benefits of Rust.

WASM SDK components can be built from sources by running:

- `./build-release` - build a full release package (includes both
  release and debug builds for web and nodejs targets)
- `./build-docs` - build TypeScript documentation
- `./build-web` - release web build
- `./build-web-dev` - development web build
- `./build-nodejs` - release nodejs build
- `./build-nodejs-dev` - development nodejs build

**IMPORTANT:** do not use `dev` builds in production. They are
significantly larger, slower and include debug symbols.

#### Requirements

- NodeJs (v20+): https://nodejs.org/en
- TypeDoc: https://typedoc.org/

#### Builds & documentation

- Release builds: https://github.com/karlsen-network/rusty-karlsen/releases
- Developer TypeScript documentation is available from [Kaspa](https://kaspa.aspectron.org/docs/)

## Karlsen CLI + Wallet

`karlsen-cli` crate provides cli-driven RPC interface to the node and
a terminal interface to the Rusty Karlsen Wallet runtime. These wallets
are compatible with WASM SDK Wallet API and Karlsen NG projects.

```bash
cd cli
cargo run --release
```

## Local Web Wallet

Run an http server inside of `wallet/wasm/web` folder. If you don't
have once, you can use the following:

```bash
cd wallet/wasm/web
cargo install basic-http-server
basic-http-server
```

The *basic-http-server* will serve on port 4000 by default, so open
your web browser and load http://localhost:4000

The framework is compatible with all major desktop and mobile browsers.

## Running the node

**Start a mainnet node**

```bash
cargo run --release --bin karlsend
```

**Start a testnet node**

```bash
cargo run --release --bin karlsend -- --testnet
```
Using a configuration file

```bash
cargo run --release --bin karlsend -- --configfile /path/to/configfile.toml
# or
cargo run --release --bin karlsend -- -C /path/to/configfile.toml
```

- The config file should be a list of \<CLI argument\> = \<value\>
  separated by newlines.
- Whitespace around the `=` is fine, `arg=value` and `arg = value`
  are both parsed correctly.
- Values with special characters like `.` or `=` will require quoting
  the value i.e \<CLI argument\> = "\<value\>".
- Arguments with multiple values should be surrounded with brackets
  like `addpeer = ["10.0.0.1", "1.2.3.4"]`.

For example:

```
testnet = true
utxoindex = false
disable-upnp = true
perf-metrics = true
appdir = "some-dir"
netsuffix = 11
addpeer = ["10.0.0.1", "1.2.3.4"]
```

Pass the `--help` flag to view all possible arguments

```bash
cargo run --release --bin karlsend -- --help
```

## wRPC

wRPC subsystem is disabled by default in `karlsend` and can be enabled
via:

JSON protocol:

```bash
--rpclisten-json = <interface:port>
```

Borsh protocol:

```bash
--rpclisten-borsh = <interface:port>
```

### Sidenote

Rusty Karlsen integrates an optional wRPC subsystem. wRPC is a
high-performance, platform-neutral, Rust-centric, WebSocket-framed
RPC implementation that can use [Borsh](https://borsh.io/) and JSON
protocol encoding.

JSON protocol messaging is similar to JSON-RPC 1.0, but differs from
the specification due to server-side notifications.

[Borsh](https://borsh.io/) encoding is meant for inter-process
communication. When using [Borsh](https://borsh.io/) both client and
server should be built from the same codebase.

JSON protocol is based on Karlsen data structures and is
data-structure-version agnostic. You can connect to the JSON endpoint
using any WebSocket library. Built-in RPC clients for JavaScript and
TypeScript capable of running in web browsers and Node.js are
available as a part of the Karlsen WASM framework.

**wRPC to gRPC Proxy is deprecated and no longer supported.**

## Mining

Mining is currently supported only on testnet, so once you've setup a
test node, follow these instructions.

1. Download and unzip the latest binaries bundle of [karlsen-network/karlsend](https://github.com/karlsen-network/karlsend/releases).

2. In a separate terminal run the karlsen-network/karlsend miner:

   ```
   karlsenminer --testnet --miningaddr karlsentest:qrcqat6l9zcjsu7swnaztqzrv0s7hu04skpaezxk43y4etj8ncwfk308jlcew
   ```

This will create and feed a DAG with the miner getting block templates
from the node and submitting them back when mined. The node processes
and stores the blocks while applying all currently implemented logic.
Execution can be stopped and resumed, the data is persisted in a
database. You can replace the above mining address with your own
address.

## Benchmarking & Testing

### Simulation framework (Simpa)

Logging in `karlsend` and `simpa` can be [filtered](https://docs.rs/env_logger/0.10.0/env_logger/#filtering-results)
by either:

The current codebase supports a full in-process network simulation,
building an actual DAG over virtual time with virtual delay and
benchmarking validation time (following the simulation generation).
To see the available commands

```bash 
cargo run --release --bin simpa -- --help
```

The following command will run a simulation to produce 1000 blocks
with communication delay of 2 seconds and 8 BPS (blocks per second)
while attempting to fill each block with up to 200 transactions.

```bash
cargo run --release --bin simpa -- -t=200 -d=2 -b=8 -n=1000
```

### Heap Profiling

Heap-profiling in `karlsend` and `simpa` can be done by enabling
`heap` feature and profile using the `--features` argument.

```bash
cargo run --bin karlsend --profile heap --features=heap
```

It will produce `{bin-name}-heap.json` file in the root of the workdir,
that can be inspected by the [dhat-viewer](https://github.com/unofficial-mirror/valgrind/tree/master/dhat)

### Tests

**Run unit and most integration tests**

```bash
cd rusty-karlsen
cargo test --release
// or install nextest and run
```

**Using nextest**

```bash
cd rusty-karlsen
cargo nextest run --release
```

### Benchmarks

```bash
cd rusty-karlsen
cargo bench
```

### Logging

Logging in `karlsend` and `simpa` can be [filtered](https://docs.rs/env_logger/0.10.0/env_logger/#filtering-results)
by either:

1. Defining the environment variable `RUST_LOG`
2. Adding the --loglevel argument like in the following example:

   ```
   (cargo run --bin karlsend -- --loglevel info,karlsen_rpc_core=trace,karlsen_grpc_core=trace,consensus=trace,karlsen_core=trace) 2>&1 | tee ~/rusty-karlsen.log
   ```

   In this command we set the `loglevel` to `INFO`.
