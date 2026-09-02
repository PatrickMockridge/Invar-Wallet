//! Plugin trait — the extension seam.
//!
//! This module is intentionally **egui-free** so plugins and their commands can be
//! unit-tested headlessly and reused by both the console and GUI buttons. A plugin
//! starts life as a set of console verbs and graduates into the UI by adding a
//! [`PanelId`] (which the app resolves to an `egui` panel elsewhere).

use crate::commands::Command;

/// Identifier for a UI panel a plugin can register. The app maps these ids to
/// actual `egui` panels, keeping `plugin.rs` decoupled from the UI crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanelId(pub &'static str);

/// A plugin contributes verbs (commands) and optional UI panels.
///
/// ```ignore
/// pub struct WalletPlugin;
/// impl Plugin for WalletPlugin {
///     fn name(&self) -> &'static str { "wallet" }
///     fn commands(&self) -> Vec<Command> { vec![/* /balance, /caps, ... */] }
///     fn panels(&self) -> Vec<PanelId> { vec![PanelId("capabilities")] }
/// }
/// ```
pub trait Plugin {
    /// Stable plugin name.
    fn name(&self) -> &'static str;

    /// The console verbs this plugin registers.
    fn commands(&self) -> Vec<Command>;

    /// UI panels this plugin registers (empty = console-only plugin).
    fn panels(&self) -> Vec<PanelId> {
        Vec::new()
    }
}
