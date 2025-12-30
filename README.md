# Intent-based trading on Solana?
![Build and Test](https://github.com/sol-ver/program/actions/workflows/ci.yml/badge.svg)


Thank to the power of Solana and Pinocchio, we create a program that enables decentralized intent-based trading on the Solana blockchain.

## Build and test
1. Build
```sh
cargo build-bpf
```

2. Test
```sh
cargo test-sbf
```

## Entrypoint 
1. Initialize order
- Create a new intent-based order

2. Cancel order
- Cancel an existing order

3. Fulfill order
- Fulfill an existing order