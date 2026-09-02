# Extending Invar

What can be built around the kernel. Everything in this chapter is *surface*: it extends
Invar without touching the invariants in [kernel.md](kernel.md). If a change would violate a
kernel invariant, it belongs on the DarkWow side (a contract change) or is out of scope.

## 1. Contracts — deploy once, discover everywhere

DarkWow contracts carry their own manifest on chain. Because Invar's wallet is
manifest-driven (kernel invariant #6), **deploying a new contract requires zero Invar code**:

1. Write a contract with a `manifest.toml` (functions, capabilities, actions, parameters).
2. Deploy it (via the DarkWow tooling) — the manifest rides in the `DeployV1` payload.
3. Invar's wallet scans the deploy transaction, stores the manifest, and the contract appears
   in the Contracts screen with its functions/actions/parameters.

You can then invoke it from the console:

```
/contracts                 # list contracts
/contract <name|id>        # show its manifest (functions, capabilities, actions, params)
/invoke <contract> <fn> --params '{"amount": 100, "recipient": "..."}'
```

## 2. Plugins — verbs that graduate into panels

A plugin implements the `Plugin` trait (`src/plugin.rs`) and registers verbs
(`Command`) and, later, panels (`PanelId`). A verb handler gets a read-only `CommandContext`
with wallet access — it is **headless** (no `egui` import), so it is unit-testable and shared
by the console and GUI buttons.

```rust
pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str { "myplugin" }
    fn commands(&self) -> Vec<Command> {
        vec![Command { name: "ping", help: "reply pong", handler: ping }]
    }
}
fn ping(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    ctx.log("pong");
    Ok(())
}
```

Register it in `src/main.rs`:

```rust
registry.register_plugin(&MyPlugin);
```

To "graduate" a verb into a UI panel, add a `PanelId` to `Plugin::panels()` and resolve it to
an egui render function in the app — the registry already exposes both slots.

## 3. Macros — named verb sequences

A macro is a named sequence of verbs in `~/.config/invar/macros.toml`:

```toml
[macros]
sweep = ["/scan", "/balance"]
```

Invoke it as `/sweep` (or bare `sweep`). Macros are how a workflow "begins life" as text
before it becomes a button.

## 4. Screens / panels

Each screen is a function in `src/ui/*.rs` that renders into an `egui::Ui`. To add a screen:

1. Add a `NavScreen` variant in `src/app.rs`.
2. Add a `selectable_value` in the nav panel.
3. Add a `match` arm in `show_screen` that calls your render function.

## 5. Themes

The single visual-customisation point is `src/ui/theme.rs` (`apply`). Every screen inherits
the palette; to retheme, edit `apply` (or replace it with your own) — no per-screen changes.

## 6. Forks

A fork typically:

- **Rebrands** — edit `Cargo.toml` (`name`, `description`), `book.toml` (`title`), and the
  window title in `src/main.rs`.
- **Rethemes** — edit `src/ui/theme.rs`.
- **Adds plugins** — as above.

Because the kernel is preserved, a fork stays correct against DarkWow; the surface is where
the fork differentiates.

## 7. The IRC plugin — a worked example

`src/plugins/irc.rs` is a minimal RFC-1459 IRC client (behind the `irc` cargo feature) that
connects to a DarkWow `darkirc` daemon — which owns the P2P/event-graph/Tor side and exposes
standard IRC on `localhost:6667`. It registers a single `/irc` verb with `connect` / `join` /
`msg` / `nick` / `quit` / `status` subcommands, and runs its reader on a background thread.

It is the canonical example of the console-first pattern: a plugin that begins life purely as
verbs, with a UI panel as the natural next step.
