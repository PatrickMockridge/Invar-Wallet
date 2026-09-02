# Architecture

How the kernel is implemented. Invar is a thin shell over `dwow_wallet`; this chapter maps
the components, the two-worlds threading model, and the data flow.

## Components

| Module | Role |
|--------|------|
| `src/wallet_service.rs` | Owns the `dwow_wallet::DwwPtr`; runs the background lifecycle (P2P sync, auto-scan, snapshot loop) and the write path. |
| `src/viewmodel.rs` | The UI-facing snapshot (`ViewModel`) and display views (`CapView`, `ContractView`, `ContractManifestView`). |
| `src/plugin.rs` | The `Plugin` trait — the extension seam (headless, egui-free). |
| `src/commands/` | `CommandRegistry`, `CommandContext`, and the built-in verb plugins (`wallet`, `contracts`). |
| `src/macros.rs` | Loads TOML `[macros]` (named verb-sequences). |
| `src/ui/` | The egui screens: `onboarding`, `capabilities`, `activity`, `addresses` (+QR), `contracts`, `theme`. |
| `src/plugins/` | Optional feature-gated plugins (`irc`). |
| `src/config.rs` | `InvarConfig` + `keys.toml` generation/restore (sovereign keys). |
| `src/contracts.rs` | The nine genesis contracts and trust-tier resolution. |

## Two worlds: egui and smol

Invar runs two runtime worlds that never block each other:

- **egui** (main thread) — immediate-mode rendering. Each frame it reads a `ViewModel`
  snapshot and paints the current screen.
- **smol** (background threads) — the async runtime. A `smol::Executor` runs `init_p2p`,
  `sync_task::run_wallet_sync`, an auto-scan loop, and a snapshot loop that publishes
  `height` / `peers` / `synced` into the `ViewModel`.

The bridge is a `parking_lot::RwLock<ViewModel>`. The wallet service writes snapshots; the UI
reads them. This mirrors the wallet CLI's own async runtime (`bin/dww/src/main.rs`).

The UI never calls `smol::block_on` on `Dww`; instead it queries the service's synchronous
read methods (which block_on internally for a *read* of an already-synced value) or reads the
snapshot.

## The write path

`send` and `invoke` are **queued**, not executed on the UI thread:

1. A verb or the Send screen calls `WalletService::queue_send` / `queue_invoke`.
2. A background thread runs `send_native_blocking` / `invoke_contract_blocking`
   (`build_native_transfer` → `broadcast_tx` → `mark_tx_exercise`, or
   `invoke_contract` → `broadcast_tx` → `mark_tx_exercise`).
3. The txid or error is pushed to the console log in the `ViewModel`.

This keeps ZK proof generation and network I/O off the UI thread.

## Data flow

```
             ┌───────────────────────────────────────────┐
             │  smol executor (background threads)        │
             │  init_p2p → run_wallet_sync → auto-scan     │
             │  snapshot loop (every 2s)                   │
             │  queued write path (queue_send/invoke)      │
             └───────────────┬───────────────────────────┘
                             │ writes ViewModel snapshot
                             ▼
   ┌─────────────────────────────────────────────────────┐
   │  ViewModel (parking_lot::RwLock)                     │
   │  sync_height · peer_count · synced · console_log      │
   └───────────────┬─────────────────────────────────────┘
                   │ egui reads each frame
                   ▼
   ┌─────────────────────────────────────────────────────┐
   │  egui (main thread)                                  │
   │  nav → screen render (Overview/Capabilities/.../Console) │
   │  command bar → CommandRegistry.execute_line → verbs   │
   └─────────────────────────────────────────────────────┘
```

## Console and extensibility

The command bar + Console screen share one `CommandRegistry`. A verb handler receives a
`CommandContext` with a read-only wallet handle (`Arc<WalletService>`) and returns output
lines. Plugins register verbs (and later panels) at startup in `src/main.rs`. This is the
"console-first" seam — the same handlers back both the console and GUI buttons.
