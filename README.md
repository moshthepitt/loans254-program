# Loans254

Loans254 is a smart contract that lives on the Solana blockchain and whose purpose is to enable collaterized lending.

More information can be found on the [companion UI repo](https://github.com/moshthepitt/kenyaloans-defi).

## Environment Setup

1. Install Rust from https://rustup.rs/
2. Install Solana v1.5.0 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

## Build and test for program compiled natively

```sh
$ cargo build
$ cargo test
```

## Build and test the program compiled for BPF

```sh
$ cargo build-bpf
$ cargo test-bpf
```
