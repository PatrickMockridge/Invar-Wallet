# Introduction

Invar is a **fork-able, customisable GUI wallet for [DarkWow](https://darkwow.org)**,
inspired by [Electrum](https://electrum.org). It is built in Rust on **egui/eframe** and
embeds the DarkWow `dwow_wallet` crate directly — the wallet is a **full node**: it syncs the
chain over P2P, stores it locally, and AEAD-decrypts its own notes. There is no light client
and no third-party index server.

The single most important thing to understand before reading anything else:

> **Invar has no coins.** It holds *capabilities*.

## The capability model in one page

DarkWow is a layer-1 blockchain whose privacy model is built on **object capabilities**.
Instead of a ledger of coins/UTXOs, the wallet holds **capabilities** — typed compositions of
cryptographic **primitives** that together cover the **barbs** (observable actions) an action
requires.

A concrete example — a native-token transfer note decrypts into a capability composing:

| primitive | barb it provides |
|-----------|------------------|
| `SecretKey` | `↓spend`, `↓derive` |
| `Commitment` | `↓commit` |
| `Nullifier` | `↓nullify` |
| `ContractId` | `↓dispatch` |
| `FuncId` | `↓gate` |
| `AssetId` | `↓denominate` |
| `MerkleNode` | `↓prove-inclusion` |

The barb union is what *makes* the capability a capability: **composition adds structure,
never authority.** A capability is spendable only if its primitives' barbs cover the action's
required barbs. This is the composition kernel DarkWow formalises in Lean4 and mirrors in
Rust (`wallet_construct`); Invar's whole interface is built to expose that model faithfully —
never to flatten it back into "coins".

## Two layers: the kernel and the surface

This book is organised around a single distinction:

- **The kernel** ([kernel.md](kernel.md)) — the *invariant core*. These are the properties
  that must be preserved for Invar to remain a correct, fork-safe wallet. They are not
  stylistic preferences; violating them breaks the wallet's model.
- **The surface** ([extending.md](extending.md)) — what is *built around* the kernel:
  screens, plugins, macros, themes, and forks. Everything in the surface is replaceable and
  extensible without touching the kernel.

The [Architecture](architecture.md) chapter shows how the kernel is implemented; the
[User Guide](user-guide.md) chapter shows how to drive it.
