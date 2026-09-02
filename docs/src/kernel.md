# The Kernel

The **kernel** is Invar's invariant core: the set of properties every screen, verb, plugin,
and fork must preserve. They are not stylistic choices. If a change violates one, it is
wrong — even if it compiles, even if it "works" for a single user.

Each invariant states the rule, *why* it exists, and where it is enforced. The invariants
are grounded in DarkWow's own specifications (`vendor/darkwow/doc/src/arch/{wallet,
composition,manifest}.md`) and in Invar's source under `src/`.

---

## 1. There are no coins — capabilities are the object

**Rule.** The wallet holds typed *capabilities* (compositions of primitives covering an
action's barbs), never coins or UTXOs. "Balance" is a narrow, derived view over native-token
capabilities — not the primary object. Any UI, verb, or plugin that reintroduces "coin/UTXO"
language or a coin-centric data model violates the kernel.

**Why.** DarkWow's privacy model is object-capability based; a capability is *the union of
the barbs of the primitives it composes*. Flattening capabilities into "coins" destroys the
typed structure (which primitives, which barbs, which contract/function) that the wallet
needs to know what an object *can do*.

**Enforced by.** `dwow_wallet`'s `CapRecord` / `get_held_capabilities` (the wallet's held
objects are typed compositions, not amounts); mirrored in Invar's display layer
`src/wallet_service.rs` (`CapView`: `name`, `primitives`, `barbs`, `status`, `trust`).

## 2. The wallet is a full node; state is a pure function

**Rule.** The wallet holds the complete chain locally. Its state is
`WalletState = f(AccountManager, ChainBlocks)` — a *pure function*: identical keys + identical
chain ⇒ byte-identical wallet state, every time. Invar introduces no light-client mode and no
index server.

**Why.** Privacy: the wallet must decrypt its own notes locally, because no third party can be
trusted to find them without learning about them. Determinism: state is replayable and
auditable.

**Enforced by.** `dwow_wallet` (`bin/dww`) syncs via `GetTip`/`GetBlocks` and scans locally;
Invar's `src/wallet_service.rs` runs that lifecycle on a background executor.

## 3. Invar embeds `dwow_wallet`; it never reimplements wallet logic

**Rule.** Scanning, transaction construction, and ZK proving are the DarkWow crate's job.
Invar is a thin shell that *uses* `dwow_wallet` — it does not copy, reimplement, or "simplify"
any of it.

**Why.** Wallet logic is consensus-adjacent and safety-critical; a second implementation
drifts. The invariant keeps Invar correct by construction: it can only be as correct as
`dwow_wallet`, never less.

**Enforced by.** `Cargo.toml` depends on `dwow_wallet` (vendored submodule at
`vendor/darkwow`); `src/wallet_service.rs` calls `Dww::new`, `init_p2p`, `scan_blocks`,
`build_native_transfer`, `invoke_contract`, `broadcast_tx` — never re-derives them.

## 4. Sovereign keys — declared, never auto-generated

**Rule.** Identity comes from an owner-authored `keys.toml` (`[section].wallet_secret`, a
64-hex secret). The wallet *derives* its keypair on boot; it never generates, caches, or
silently substitutes a key. Missing file/section is a hard error.

**Why.** If a wallet could auto-generate a key, a misconfigured deployment would silently
create a throwaway identity (and in a miner, break consensus). "The owner declares their key;
the software only uses it."

**Enforced by.** `dwow_accounts::AccountManager::open`; Invar's onboarding
(`src/config.rs` `generate_new_key` / `restore_from_mnemonic` / `restore_from_hex`) writes
`keys.toml` only at explicit owner request.

## 5. Discovery is AEAD trial-decryption only

**Rule.** The wallet discovers its objects by attempting to decrypt every on-chain
`AeadEncryptedNote` with its held secrets; a successful decrypt *is* the discovery. There is
no address index, no pubkey-metadata scan, no signature-based discovery.

**Why.** Any other channel leaks which notes belong to whom. AEAD trial-decryption is the
only mechanism consistent with the privacy model.

**Enforced by.** `dwow_wallet`'s scan path (`bin/dww/src/scan.rs`); Invar surfaces the result
through `held_capability_views` / `all_capability_views`.

## 6. The manifest is the type declaration

**Rule.** A contract carries its own interface on chain as a TOML **manifest** — functions,
capabilities (with `primitives` + `note_schema`), actions (`requires`/`consumes`/`produces`/
`required_barbs`), circuits (`witness_map`), and parameters. The wallet reads the manifest and
auto-configures. **Adding a contract requires zero wallet code.**

**Why.** Without manifests, every wallet would decompile WASM to discover interfaces. The
manifest makes the capability graph usable generically — and it is the schema, never the data
(who holds what stays encrypted).

**Enforced by.** `dwow_wallet` manifest pipeline (parse → store → resolve → query → invoke);
Invar's `src/wallet_service.rs` `contract_manifest` / `genesis_manifest_views` and the
Contracts screen.

## 7. Trust tiers annotate, never block

**Rule.** Every contract carries a trust tier — Genesis / Self-deployed / Attested /
Unverified. The wallet *displays* the tier and warns, but **never blocks** interaction based
on it. Caveat emptor: the wallet is a navigation tool, not a guardian.

**Why.** The chain is adversarial; a manifest can lie. The correct posture is information,
not policy — the user decides.

**Enforced by.** Invar's `src/contracts.rs` (`trust_tier`, the nine genesis contracts
resolving to `Genesis`); the UI shows the tier as a badge.

## 8. The console is the extensibility core

**Rule.** Plugins and macros *begin life* as `/verb` commands in the console, and graduate
into UI panels. Every feature is expressible as a verb first. A plugin can ship a verb today
and a panel tomorrow through the same seam.

**Why.** A command-line surface is the lowest-friction place for a feature to prove itself;
the verb → panel graduation keeps the UI from ossifying.

**Enforced by.** `src/plugin.rs` (`Plugin` trait: `commands()` + `panels()`),
`src/commands/mod.rs` (`CommandRegistry`, `/help`), `src/macros.rs` (TOML `[macros]`).

## 9. Threading: egui (main) + smol (background), snapshot-driven

**Rule.** The egui UI reads a `ViewModel` snapshot and is **never blocked**. The wallet
service owns the `DwwPtr` and runs sync/scan (and the queued write path) on a background
`smol` executor. The UI and the wallet communicate through a snapshot, not shared mutable
state.

**Why.** ZK proof generation and network I/O are slow; blocking the UI on them makes the
wallet unusable. The snapshot discipline also avoids `smol`-lock deadlocks.

**Enforced by.** `src/wallet_service.rs` (`start_sync` spawns the executor + snapshot loop;
`queue_send` / `queue_invoke` run the write path off-thread) and `src/viewmodel.rs`
(`ViewModel`).

## 10. AGPL-3.0-only

**Rule.** Invar is AGPL-3.0-only, inherited from the embedded `dwow_wallet` crate. A fork
must remain AGPL-3.0-only.

**Why.** Copyleft is the price of embedding the DarkWow wallet; it keeps derivatives open.

**Enforced by.** `Cargo.toml` `license`, `LICENSE`.
