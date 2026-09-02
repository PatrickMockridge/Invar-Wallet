//! Contract verbs — `/contracts` lists the standard (genesis) contracts.

use crate::commands::{Command, CommandContext, CommandResult};
use crate::plugin::Plugin;

pub struct ContractsPlugin;

impl Plugin for ContractsPlugin {
    fn name(&self) -> &'static str {
        "contracts"
    }

    fn commands(&self) -> Vec<Command> {
        vec![Command {
            name: "contracts",
            help: "list genesis contracts (standard)",
            handler: contracts,
        }]
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
