//! Contract verbs — `/contracts` lists the standard contracts, `/contract` shows a manifest.

use crate::commands::{Command, CommandContext, CommandResult};
use crate::plugin::Plugin;

pub struct ContractsPlugin;

impl Plugin for ContractsPlugin {
    fn name(&self) -> &'static str {
        "contracts"
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            Command {
                name: "contracts",
                help: "list genesis contracts (standard)",
                handler: contracts,
            },
            Command {
                name: "contract",
                help: "show a contract's manifest: /contract <name|id>",
                handler: contract,
            },
        ]
    }
}

fn contracts(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    for c in w.contract_views() {
        ctx.log(format!("{} [{}] {}", c.name, c.trust, c.contract_id));
    }
    Ok(())
}

fn contract(ctx: &mut CommandContext, args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    let Some(arg) = args.first() else {
        ctx.log("usage: /contract <name|id>");
        return Ok(());
    };
    let cid = crate::contracts::genesis_id_b58(arg).unwrap_or_else(|| arg.clone());

    match w.contract_manifest(&cid) {
        Ok(Some(m)) => {
            ctx.log(format!(
                "{} [{}] {} v{} — {}",
                m.name, m.trust, m.category, m.version, m.description
            ));
            for f in &m.functions {
                ctx.log(format!("  fn {f}"));
            }
            for c in &m.capabilities {
                ctx.log(format!("  cap {c}"));
            }
            for a in &m.actions {
                ctx.log(format!("  action {a}"));
            }
            for p in &m.parameters {
                ctx.log(format!("  param {p}"));
            }
        }
        Ok(None) => ctx.log(format!("no manifest for {arg}")),
        Err(e) => ctx.log(format!("error: {e}")),
    }
    Ok(())
}
