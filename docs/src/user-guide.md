# User Guide

How to build, run, and drive Invar.

## Build and run

```bash
git clone --recurse-submodules https://github.com/PatrickMockridge/Invar
cd invar

# DarkWow's ZK circuit binaries (.zk.bin) are generated, not committed:
make -C vendor/darkwow contracts

cargo run
```

The first build is large (halo2 / ZK dependencies). To build without the IRC plugin:

```bash
cargo run --no-default-features
```

## First run — onboarding

On first launch Invar shows the onboarding wizard:

1. **Network** — `darkwow-testnet` (public testnet), `darkwow-devnet` (local), `testnet`,
   `localnet`, or `mainnet`.
2. **Wallet name** — the `keys.toml` section that selects this wallet's identity.
3. **Password** — the SQLCipher password for the wallet database (not stored; prompted).
4. **Identity** — create a new key, restore from a BIP39 mnemonic, restore from a 64-hex
   secret, or open an existing `keys.toml`.
5. **Create / Open**.

Invar then opens the wallet, initialises it (seeding the nine genesis contracts + native-token
circuits), and begins syncing in the background.

> Your key is *declared* in `keys.toml`, never auto-generated and never stored in the wallet
> database. Back it up.

## The screens

- **Overview** — live sync status (height / peers / synced), your address, and the native
  balance.
- **Capabilities** — the objects you hold (not "coins"): each shows its name, value, contract,
  trust tier, primitives and barbs, and spend-state.
- **Activity** — the capability lifecycle grouped by spend-state
  (Unspent → Pending → Processing → Spent).
- **Addresses** — your default receiving address + its QR code, and derived addresses.
- **Contracts** — the nine genesis contracts (marked `GENESIS`) and their on-chain manifests
  (functions, capabilities, actions, parameters).
- **Send** — send native DRKW to a base58 address.
- **Console** — the command surface (see below).

## The console

The persistent command bar (top of every screen) and the Console screen share one command
interface. Type `/help` to list everything.

| Verb | Purpose |
|------|---------|
| `/help` | list verbs and macros |
| `/version` | show the Invar version |
| `/balance` | native-token balance |
| `/caps` | held capabilities (`--all` includes spent) |
| `/address` / `/addresses` | default / derived addresses |
| `/sync` | sync status (height, peers, synced) |
| `/send <amount> <address>` | queue a native DRKW transfer |
| `/contracts` | list genesis contracts |
| `/contract <name\|id>` | show a contract's manifest |
| `/invoke <contract> <fn> [--params <json>]` | invoke a contract action |
| `/irc …` | IRC client (if the `irc` feature is on) |

Macros defined in `~/.config/invar/macros.toml` are invocable by name (e.g. `/sweep`).

## Sending and receiving

- **Receive** — share your default address (Addresses screen). Others encrypt a note to it;
  after your wallet syncs + scans, the resulting capability appears in the Capabilities screen.
- **Send** — enter an amount and the recipient's base58 address on the Send screen (or
  `/send`). The transfer is built and broadcast on a background thread; the txid is logged in
  the console. Sending requires a synced chain and a spendable native-token capability.

## Capabilities and trust tiers, in plain language

- **Capability** — a thing you hold that lets you *do* something (spend, vote, redeem, …).
  It is not just an amount; it is the set of permissions (barbs) its parts give you.
- **Trust tier** — how much you should trust a contract: `GENESIS` (built-in), `OWN` (you
  deployed it), `ATTESTED` (vouched for), `UNVERIFIED` (everything else). Invar shows the tier
  but never stops you from interacting — it warns, and you decide.
