//! Command registry — headless (no `egui` imports).
//!
//! Resolves `/verb` invocations and macros against the registered commands, then
//! dispatches to a handler. Built-in verbs live here and in `wallet.rs` / `contracts.rs`
//! (added in later milestones); plugins contribute more via [`crate::plugin::Plugin`].

use std::collections::HashMap;
use std::sync::Arc;

use crate::plugin::Plugin;
use crate::wallet_service::WalletService;

pub mod contracts;
pub mod wallet;

pub type CommandResult = Result<(), String>;

/// Context handed to a command handler: an output sink and an optional read-only handle to
/// the wallet (so console verbs can query the same data the GUI screens show).
pub struct CommandContext {
    pub out: Vec<String>,
    pub wallet: Option<Arc<WalletService>>,
}

impl CommandContext {
    pub fn new() -> Self {
        Self {
            out: Vec::new(),
            wallet: None,
        }
    }

    /// Append a line of output.
    pub fn log(&mut self, line: impl Into<String>) {
        self.out.push(line.into());
    }
}

/// A console verb: its name, one-line help, and a handler.
pub struct Command {
    pub name: &'static str,
    pub help: &'static str,
    pub handler: fn(&mut CommandContext, args: &[String]) -> CommandResult,
}

/// The registry of verbs + macros. `execute_line` is the single dispatch entry point.
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    macros: HashMap<String, Vec<String>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            macros: HashMap::new(),
        }
    }

    pub fn register(&mut self, cmd: Command) {
        self.commands.insert(cmd.name.to_string(), cmd);
    }

    pub fn register_plugin(&mut self, plugin: &dyn Plugin) {
        for cmd in plugin.commands() {
            self.register(cmd);
        }
    }

    pub fn set_macros(&mut self, macros: HashMap<String, Vec<String>>) {
        self.macros = macros;
    }

    /// Sorted verb names, for `/help` and autocomplete.
    pub fn command_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.commands.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Sorted macro names.
    pub fn macro_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.macros.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Parse and execute one line of console input (with or without a leading `/`).
    pub fn execute_line(&self, input: &str, ctx: &mut CommandContext) {
        let input = input.trim();
        if input.is_empty() {
            return;
        }

        let tokens: Vec<String> = input.split_whitespace().map(str::to_string).collect();
        let verb = tokens[0].trim_start_matches('/').to_string();
        let args = &tokens[1..];

        // `/help` lists everything, so it needs registry access — handled inline.
        if verb == "help" {
            self.print_help(ctx);
            return;
        }

        // Macro expansion: run each step as its own line.
        if let Some(steps) = self.macros.get(&verb) {
            for step in steps.clone() {
                self.execute_line(&step, ctx);
            }
            return;
        }

        match self.commands.get(&verb) {
            Some(cmd) => {
                if let Err(e) = (cmd.handler)(ctx, args) {
                    ctx.log(format!("error: {e}"));
                }
            }
            None => ctx.log(format!("unknown verb: /{verb} (try /help)")),
        }
    }

    fn print_help(&self, ctx: &mut CommandContext) {
        ctx.log("verbs:");
        for name in self.command_names() {
            let help = self.commands.get(name).map(|c| c.help).unwrap_or("");
            ctx.log(format!("  /{name:<16} {help}"));
        }
        ctx.log("  /help            list verbs and macros");
        if !self.macros.is_empty() {
            ctx.log("macros:");
            for name in self.macro_names() {
                let steps = self.macros.get(name).map(|s| s.join(" ; ")).unwrap_or_default();
                ctx.log(format!("  /{name:<16} {steps}"));
            }
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The built-in core plugin: `/help` (handled inline) and `/version`.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn name(&self) -> &'static str {
        "core"
    }

    fn commands(&self) -> Vec<Command> {
        vec![Command {
            name: "version",
            help: "show Invar version",
            handler: |ctx, _args| {
                ctx.log(format!("Invar {}", env!("CARGO_PKG_VERSION")));
                Ok(())
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_registered_command() {
        let mut reg = CommandRegistry::new();
        reg.register(Command {
            name: "ping",
            help: "reply pong",
            handler: |ctx, args| {
                ctx.log(format!("pong {}", args.join(" ")));
                Ok(())
            },
        });

        let mut ctx = CommandContext::new();
        reg.execute_line("/ping hello", &mut ctx);
        assert_eq!(ctx.out, vec!["pong hello"]);
    }

    #[test]
    fn expands_macros() {
        let mut reg = CommandRegistry::new();
        reg.register(Command {
            name: "balance",
            help: "show balance",
            handler: |ctx, _| {
                ctx.log("balance: 0");
                Ok(())
            },
        });
        reg.set_macros(
            [("sweep".to_string(), vec!["/balance".to_string()])]
                .into_iter()
                .collect(),
        );

        let mut ctx = CommandContext::new();
        reg.execute_line("sweep", &mut ctx);
        assert_eq!(ctx.out, vec!["balance: 0"]);
    }

    #[test]
    fn unknown_verb() {
        let reg = CommandRegistry::new();
        let mut ctx = CommandContext::new();
        reg.execute_line("/nope", &mut ctx);
        assert_eq!(ctx.out, vec!["unknown verb: /nope (try /help)"]);
    }
}
