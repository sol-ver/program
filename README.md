# Intent-based trading on Solana?
![Build and Test](https://github.com/sol-ver/program/actions/workflows/ci.yml/badge.svg)

Thank to the power of Solana and Pinocchio, we create a program that enables decentralized intent-based trading on the Solana blockchain.

## Design
1. Stateless order
- Orders are represented as intents without storing state on-chain.
- This reduces on-chain storage costs and increases scalability.

2. Dutch auction mechanism
- Orders are fulfilled using a Dutch auction mechanism.
- This allows for dynamic pricing based on market demand.
- Solvers can compete to fulfill orders at the best price and timing.

3. Decentralized fulfillment
- Any participant can act as a solver to fulfill orders.
- This promotes decentralization and reduces reliance on centralized entities.
- Order will be published on-chain, and solvers can monitor and fulfill them.

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
