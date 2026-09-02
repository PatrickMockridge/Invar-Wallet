# Glossary

**Capability** — an object the wallet holds, defined by the primitives it composes and the
barbs those primitives provide. The unit of authority in DarkWow (and thus Invar). Not a coin.

**Primitive** — a cryptographic name type (`SecretKey`, `PublicKey`, `Nullifier`,
`Commitment`, `ContractId`, `FuncId`, `AssetId`, `MerkleNode`, `OwnedSecretKey`,
`MiningRecipient`), each carrying a fixed set of barbs.

**Barb** — an observable action a primitive can exhibit (`↓spend`, `↓derive`, `↓verify`,
`↓encrypt`, `↓nullify`, `↓commit`, `↓dispatch`, `↓gate`, `↓denominate`, `↓prove-inclusion`,
`↓mine`). The "↓" reads "can be used to".

**Composition** — the operation that unions the barb sets of a capability's primitives. A
capability is valid for an action iff that union covers the action's required barbs.
"Composition adds structure, never authority."

**Manifest** — a TOML document, embedded on chain at deploy time, that declares a contract's
functions, capabilities, actions, circuits, and parameters. The wallet's type declaration for
that contract.

**Note** — an `AeadEncryptedNote`; the encrypted payload that, when decrypted with a held
secret, yields a capability's primitives.

**Trust tier** — `Genesis` / `Self-deployed` / `Attested` / `Unverified`; a displayed,
advisory classification of a contract. Never blocks interaction.

**Verb** — a `/slash` command in the console, e.g. `/balance`. The unit of extensibility.

**Macro** — a named sequence of verbs in `~/.config/invar/macros.toml`.

**Plugin** — a native Rust module implementing the `Plugin` trait, contributing verbs and
(optionally) panels.

**Full node** — the wallet stores the whole chain locally and derives its state from it; no
light client, no index server.

**ViewModel** — the snapshot of wallet state the egui UI reads each frame, published by the
background wallet service.
