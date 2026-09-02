# Invar

A fork-able, customisable GUI wallet for [DarkWow](https://darkwow.org), inspired by
[Electrum](https://electrum.org). Built in Rust on **egui/eframe**, embedding the DarkWow
`dwow_wallet` crate directly (full-node wallet — no light client).

> **Invar uses DarkWow's capability model, not a coin/UTXO model.** There are no "coins":
> the wallet holds typed **capabilities** (compositions of primitives — `SecretKey`,
> `Commitment`, `Nullifier`, `ContractId`, `FuncId`, `AssetId`, `MerkleNode` — covering an
> action's **barbs**). See `vendor/darkwow/doc/src/arch/wallet.md` for the full model.

## Features

- **Full-node wallet** — syncs the DarkWow chain over P2P and scans locally (no light client,
  no index server).
- **Capability browser** — your held capabilities (typed: primitives, barbs, spend-state,
  trust tier), never flattened to "coins".
- **Activity** — the capability spend-state lifecycle (Unspent → Pending → Processing → Spent).
- **Contracts** — the nine genesis contracts and their on-chain manifests (functions,
  capabilities, actions, parameters).
- **Send & invoke** — native DRKW transfer and generic manifest-driven contract invocation,
  built and broadcast off the UI thread.
- **Addresses + QR** — your receiving address with a QR code.
- **Console** — a `/verb` command surface (`/help`, `/balance`, `/caps`, `/sync`, `/send`,
  `/contracts`, `/invoke`, …) that plugins and macros extend.
- **Themes & plugins** — retheme in one file; add features as plugins without touching the
  wallet core.
- **Optional IRC** — an IRC client plugin (behind the `irc` cargo feature) for `darkirc`.

## Build

```bash
git clone --recurse-submodules https://github.com/PatrickMockridge/Invar
cd invar

# DarkWow's ZK circuit binaries (.zk.bin) are generated, not committed.
# Build them first (compiles the `zkas` compiler, then emits all proofs):
make -C vendor/darkwow contracts

cargo build            # first build is large (halo2 / ZK deps)
cargo test             # unit tests (wallet open, sync, verbs, manifests, …)
cargo run              # launch the GUI
```

The IRC plugin is on by default; build without it:

```bash
cargo run --no-default-features
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

## Documentation

The [Invar Book](docs/) (mdBook) defines the wallet's **kernel** (the invariant core) and
everything that can be built around it, for developers and users:

```bash
cd docs && mdbook build   # → docs/book (HTML)
mdbook serve docs         # live preview
```

## Extending Invar

Invar's console is the extensibility core: plugins and macros begin life as `/verb` commands
and graduate into UI panels.

### Add a verb / plugin

Plugins implement the `Plugin` trait (`src/plugin.rs`) and register `Command` verbs
(`src/commands/mod.rs`). A command handler receives a `CommandContext` with read-only wallet
access. Register your plugin in `src/main.rs`:

```rust
pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str { "myplugin" }
    fn commands(&self) -> Vec<Command> {
        vec![Command { name: "ping", help: "reply pong", handler: ping }]
    }
}
fn ping(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    ctx.log("pong"); Ok(())
}
```

Commands are headless (no `egui` import), so they're unit-testable and shared by the console
and GUI buttons.

### Macros

Named sequences of verbs, defined in `~/.config/invar/macros.toml`:

```toml
[macros]
sweep = ["/scan", "/balance"]
```

Invoke as `/sweep` (or bare `sweep`) from the console.

### Retheme

Edit `src/ui/theme.rs` (`apply`) — every screen inherits the palette. This is the single
visual-customisation point.

## License

AGPL-3.0-only (inherited from the embedded `dwow_wallet` crate). See [LICENSE](LICENSE).
