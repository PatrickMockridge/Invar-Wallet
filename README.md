# Invar

A fork-able, customisable GUI wallet for [DarkWow](https://darkwow.org), inspired by
[Electrum](https://electrum.org). Built in Rust on **egui/eframe**, embedding the DarkWow
`dwow_wallet` crate directly (full-node wallet — no light client).

> **Invar uses DarkWow's capability model, not a coin/UTXO model.** There are no "coins":
> the wallet holds typed **capabilities** (compositions of primitives — `SecretKey`,
> `Commitment`, `Nullifier`, `ContractId`, `FuncId`, `AssetId`, `MerkleNode` — covering an
> action's **barbs**). See `doc/src/arch/wallet.md` in the DarkWow tree for the full model.

## Build

```bash
git clone --recurse-submodules https://github.com/PatrickMockridge/Invar
cd invar

# DarkWow's ZK circuit binaries (.zk.bin) are generated, not committed.
# Build them first (compiles the `zkas` compiler, then emits all proofs):
make -C vendor/darkwow contracts

cargo build            # first build is large (halo2 / ZK deps)
cargo run              # launch the GUI
```

### Vendored DarkWow

The DarkWow crates are unpublished, and `dwow_wallet` lives in a subdirectory (`bin/dww`)
that a Cargo `git` dependency cannot target. Invar therefore vendors the DarkWow repo as a
**git submodule** at `vendor/darkwow` and references it by `path`:

```bash
git submodule add https://github.com/PatrickMockridge/DarkWow vendor/darkwow
```

For local development without git, you can instead symlink an existing checkout:

```bash
ln -s /path/to/darkwow vendor/darkwow
```

## License

AGPL-3.0-only (inherited from the embedded `dwow_wallet` crate). See [LICENSE](LICENSE).

## Extending Invar

See the `src/commands/` and `src/plugin.rs` — plugins are native Rust modules that register
`/verb` commands (and later, UI panels). Macros are TOML `[macros]` entries: named sequences
of verbs.
