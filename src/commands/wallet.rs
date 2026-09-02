//! Wallet verbs — `/balance`, `/caps`, `/address`, `/addresses`, `/sync`.

use crate::commands::{Command, CommandContext, CommandResult};
use crate::plugin::Plugin;

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn name(&self) -> &'static str {
        "wallet"
    }

    fn commands(&self) -> Vec<Command> {
        vec![
            Command {
                name: "balance",
                help: "show native-token balance",
                handler: balance,
            },
            Command {
                name: "caps",
                help: "list held capabilities (--all includes spent)",
                handler: caps,
            },
            Command {
                name: "address",
                help: "show the default receiving address",
                handler: address,
            },
            Command {
                name: "addresses",
                help: "list derived addresses",
                handler: addresses,
            },
            Command {
                name: "sync",
                help: "show sync status",
                handler: sync,
            },
            Command {
                name: "send",
                help: "send native DRKW: /send <amount> <recipient-address>",
                handler: send,
            },
        ]
    }
}

fn balance(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    match w.capability_balance() {
        Ok(b) if b.is_empty() => ctx.log("no native balance"),
        Ok(b) => {
            for (asset, amount) in b {
                ctx.log(format!("{asset}\t{amount}"));
            }
        }
        Err(e) => ctx.log(format!("error: {e}")),
    }
    Ok(())
}

fn caps(ctx: &mut CommandContext, args: &[String]) -> CommandResult {
    let all = args.iter().any(|a| a == "--all");
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    let result = if all {
        w.all_capability_views()
    } else {
        w.held_capability_views()
    };
    match result {
        Ok(caps) if caps.is_empty() => ctx.log("no held capabilities"),
        Ok(caps) => {
            for c in caps {
                ctx.log(format!(
                    "{} value={} {} [{}]",
                    c.status, c.value, c.name, c.contract_name
                ));
            }
        }
        Err(e) => ctx.log(format!("error: {e}")),
    }
    Ok(())
}

fn address(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    match w.default_address() {
        Ok(a) => ctx.log(a),
        Err(e) => ctx.log(format!("error: {e}")),
    }
    Ok(())
}

fn addresses(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    match w.addresses() {
        Ok(addrs) => {
            for (i, a) in addrs.iter().enumerate() {
                ctx.log(format!("{i}: {a}"));
            }
        }
        Err(e) => ctx.log(format!("error: {e}")),
    }
    Ok(())
}

fn sync(ctx: &mut CommandContext, _args: &[String]) -> CommandResult {
    let Some(w) = ctx.wallet.as_ref() else {
        ctx.log("wallet not open");
        return Ok(());
    };
    let height = match w.chain_height() {
        Ok(h) => format!("chain height: {h}"),
        Err(e) => format!("chain height: {e}"),
    };
    let peers = w.peer_count();
    let p2p = if w.p2p_ready() { "ready" } else { "not initialized" };
    let synced = w.is_synced();

    ctx.log(height);
    ctx.log(format!("peers: {peers}"));
    ctx.log(format!("p2p: {p2p}"));
    ctx.log(format!("synced: {synced}"));
    Ok(())
}

fn send(ctx: &mut CommandContext, args: &[String]) -> CommandResult {
    let (amount, recipient) = match (args.first(), args.get(1)) {
        (Some(a), Some(r)) => match a.parse::<u64>() {
            Ok(amount) => (amount, r.clone()),
            Err(e) => {
                ctx.log(format!("invalid amount: {e}"));
                return Ok(());
            }
        },
        _ => {
            ctx.log("usage: /send <amount> <recipient-address>");
            return Ok(());
        }
    };

    match ctx.wallet.clone() {
        Some(w) => {
            w.queue_send(amount, recipient);
            ctx.log("send queued…");
        }
        None => ctx.log("wallet not open"),
    }
    Ok(())
}
