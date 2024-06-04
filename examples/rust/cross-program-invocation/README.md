# Rust Summit Belgrade June 2024

## Running the tests

Use `cargo test-sbf` to run the tests and not `cargo test`.

## Useful links

- Rust summit presentation code: https://github.com/andreisilviudragnea/solana-program-library/tree/rust-summit
- BPF info: https://solana.com/docs/programs/faq#berkeley-packet-filter-bpf
- Solana programs memory map: https://solana.com/docs/programs/faq#memory-map
- Solana rBPF virtual machine: https://github.com/solana-labs/rbpf
- Solana program input deserialization
  logic: https://github.com/anza-xyz/agave/blob/63c16b65cc872c08a69af7082a6c8a82b31dfd10/sdk/program/src/entrypoint.rs#L277
- Solana rBPF memory map input
  start: https://github.com/solana-labs/rbpf/blob/4dc039f4ee7409838c7f230558aebf6869c32db9/src/ebpf.rs#L54
- Heap size: https://solana.com/docs/programs/faq#heap-size
- Rust heap: https://solana.com/docs/programs/lang-rust#heap
- C heap: https://solana.com/docs/programs/lang-c#heap
- Compute budget: https://solana.com/docs/core/fees#compute-budget
- Compute units: https://solana.com/docs/core/fees#compute-units
- Compute unit limit: https://solana.com/docs/core/fees#compute-unit-limit
- How to optimize compute units: https://solana.com/developers/guides/advanced/how-to-optimize-compute
- Accounts key points: https://solana.com/docs/core/accounts#key-points
- Linked list allocator: https://crates.io/crates/linked-list-allocator
- Allocator-api2 crate: https://crates.io/crates/allocator-api2
