# Gossip Glomers

This project contains my solutions for the [Gossip Glomers: Distributed Systems programming challenge](https://fly.io/dist-sys/).
It is a hybrid setup with the first few solutions being in Rust and a few in Golang. There's also a utility script that builds, and runs
the solutions against [Maelstrom](https://github.com/jepsen-io/maelstrom).

## Setup

TLDR; Install [OpenJDK](https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md#jdk), [GraphViz](https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md#graphviz), and [Gnuplot](https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md#gnuplot).

**Note**: You do not even have to install **Maelstrom** explicitly. If you're running a test for the first time, the `./run` util should set that up for you automatically.

## Usage

To run the Maelstrom tests, you can use the `./run` executable:

```sh
./run 1 # Runs the echo challenge.
./run 2 3a 3b # Runs the Unique Id Generation challenge as well as the first two parts of the Broadcast challenge.
```

More info available: `./run --help`

```
usage: MaelstromInvoker [-h] CHALLENGE_ID [CHALLENGE_ID ...]

Run the maelstrom tests for a given fly.io dist-sys challenge. Currently, the following challenges are implemented: - 01: Echo - 02: Unique ID Generation - 03a: Broadcast (Single Node) -
03b: Broadcast (Multi-Node)

positional arguments:
  CHALLENGE_ID  The ids of the challenges to run.

optional arguments:
  -h, --help    show this help message and exit
```

## Contributing

Issues, pull requests, and Github stars are always appreciated!

