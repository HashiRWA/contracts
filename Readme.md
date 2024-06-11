### To Build the contracts for cosmwasm :

```
$ cargo build --target wasm32-unknown-unknown --release
```

### To Check the health of the built contract :

```
cosmwasm-check ./target/wasm32-unknown-unknown/release/hashirwa_contracts.wasm
```

### The incremental build process that produces that red underline in the IDE is a result of cached information about the program and library.

```
cargo clean
```
