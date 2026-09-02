//! IRC client plugin — a minimal RFC-1459 client that connects to a DarkWow `darkirc`
//! daemon (which owns the P2P/event-graph/Tor side and exposes standard IRC on
//! `localhost:6667`). The plugin "begins life" as `/irc` verbs; an IRC panel can graduate
//! from here later.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, OnceLock};

use crate::commands::{Command, CommandContext, CommandResult};
use crate::plugin::Plugin;

/// Format an IRC protocol line (no `\r\n`; the caller appends it). Pure + unit-testable.
pub fn irc_line(cmd: &str, args: &[&str], trailing: Option<&str>) -> String {
    let mut s = cmd.to_string();
    for a in args {
        s.push(' ');
        s.push_str(a);
    }
    if let Some(t) = trailing {
        s.push_str(" :");
        s.push_str(t);
    }
    s
}

struct IrcClient {
    /// Shared write handle (commands + PONG replies).
    stream: Arc<Mutex<TcpStream>>,
    _reader: std::thread::JoinHandle<()>,
    nick: String,
}

struct IrcState {
    client: Option<IrcClient>,
    log: Vec<String>,
}

static IRC: OnceLock<Mutex<IrcState>> = OnceLock::new();

fn state() -> &'static Mutex<IrcState> {
    IRC.get_or_init(|| {
        Mutex::new(IrcState {
            client: None,
            log: Vec::new(),
        })
    })
}

pub struct IrcPlugin;

impl Plugin for IrcPlugin {
    fn name(&self) -> &'static str {
        "irc"
    }

    fn commands(&self) -> Vec<Command> {
        vec![Command {
            name: "irc",
            help: "IRC: /irc connect <host:port> <nick> | join <ch> | msg <t> <m> | nick <n> | quit | status",
            handler: irc,
        }]
    }
}

fn irc(ctx: &mut CommandContext, args: &[String]) -> CommandResult {
    let Some(sub) = args.first() else {
        ctx.log("usage: /irc <connect|join|msg|nick|quit|status> ...");
        return Ok(());
    };
    match sub.as_str() {
        "connect" => connect(ctx, &args[1..]),
        "join" => join(ctx, &args[1..]),
        "msg" => msg(ctx, &args[1..]),
        "nick" => nick(ctx, &args[1..]),
        "quit" => quit(ctx),
        "status" => status(ctx),
        _ => ctx.log(format!("unknown /irc subcommand: {sub}")),
    }
    Ok(())
}

/// Send one raw IRC line (without CRLF) to the server, if connected.
fn send(line: &str) -> Result<(), String> {
    let stream = {
        let st = state().lock().unwrap();
        match &st.client {
            Some(c) => c.stream.clone(),
            None => return Err("not connected — run /irc connect <host:port> <nick>".into()),
        }
    };
    let mut s = stream.lock().unwrap();
    writeln!(s, "{line}").map_err(|e| e.to_string())
}

fn connect(ctx: &mut CommandContext, args: &[String]) {
    if args.len() < 2 {
        ctx.log("usage: /irc connect <host:port> <nick>");
        return;
    }
    let (host, port) = match args[0].split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(6667)),
        None => (args[0].clone(), 6667),
    };
    let nick = args[1].clone();

    let mut stream = match TcpStream::connect((host.as_str(), port)) {
        Ok(s) => s,
        Err(e) => {
            ctx.log(format!("connect failed: {e}"));
            return;
        }
    };
    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            ctx.log(format!("stream clone: {e}"));
            return;
        }
    };

    if writeln!(stream, "NICK {nick}").is_err()
        || writeln!(stream, "USER {nick} 0 * :Invar").is_err()
    {
        ctx.log("failed to register with the IRC server");
        return;
    }

    let write_arc = Arc::new(Mutex::new(stream));
    let pong_arc = write_arc.clone();
    let reader = std::thread::spawn(move || {
        let buf = BufReader::new(read_stream);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    if let Some(token) = l.strip_prefix("PING ") {
                        let _ = writeln!(pong_arc.lock().unwrap(), "PONG {token}");
                        continue;
                    }
                    state().lock().unwrap().log.push(l);
                }
                Err(_) => break,
            }
        }
    });

    {
        let mut st = state().lock().unwrap();
        st.client = Some(IrcClient {
            stream: write_arc,
            _reader: reader,
            nick: nick.clone(),
        });
        st.log.push(format!("connected to {host}:{port} as {nick}"));
    }
    ctx.log(format!("connected to {host}:{port} as {nick}"));
}

fn join(ctx: &mut CommandContext, args: &[String]) {
    let Some(ch) = args.first() else {
        ctx.log("usage: /irc join <channel>");
        return;
    };
    match send(&irc_line("JOIN", &[ch], None)) {
        Ok(()) => ctx.log(format!("joining {ch}")),
        Err(e) => ctx.log(format!("error: {e}")),
    }
}

fn msg(ctx: &mut CommandContext, args: &[String]) {
    if args.len() < 2 {
        ctx.log("usage: /irc msg <target> <message...>");
        return;
    }
    let target = &args[0];
    let text = args[1..].join(" ");
    match send(&irc_line("PRIVMSG", &[target], Some(&text))) {
        Ok(()) => ctx.log(format!("-> {target}: {text}")),
        Err(e) => ctx.log(format!("error: {e}")),
    }
}

fn nick(ctx: &mut CommandContext, args: &[String]) {
    let Some(n) = args.first() else {
        ctx.log("usage: /irc nick <nick>");
        return;
    };
    match send(&irc_line("NICK", &[n], None)) {
        Ok(()) => {
            let mut st = state().lock().unwrap();
            if let Some(c) = &mut st.client {
                c.nick = n.clone();
            }
            ctx.log(format!("nick -> {n}"));
        }
        Err(e) => ctx.log(format!("error: {e}")),
    }
}

fn quit(ctx: &mut CommandContext) {
    let _ = send(&irc_line("QUIT", &[], Some("bye")));
    state().lock().unwrap().client = None;
    ctx.log("disconnected");
}

fn status(ctx: &mut CommandContext) {
    let st = state().lock().unwrap();
    match &st.client {
        Some(c) => ctx.log(format!("connected as {}", c.nick)),
        None => ctx.log("not connected"),
    }
    for line in st.log.iter().rev().take(20).rev() {
        ctx.log(line.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_irc_lines() {
        assert_eq!(irc_line("NICK", &["alice"], None), "NICK alice");
        assert_eq!(irc_line("JOIN", &["#dev"], None), "JOIN #dev");
        assert_eq!(
            irc_line("PRIVMSG", &["#dev"], Some("hello world")),
            "PRIVMSG #dev :hello world"
        );
        assert_eq!(irc_line("QUIT", &[], Some("bye")), "QUIT :bye");
    }
}
