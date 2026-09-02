//! Load user-defined macros (named sequences of verbs) from TOML.
//!
//! A macro is a `[macros]` table mapping a name to an ordered list of verb invocations:
//!
//! ```toml
//! [macros]
//! sweep = ["/scan", "/balance"]
//! ```
//!
//! Invoking `/sweep` expands to `/scan` then `/balance`. This is how a workflow
//! "begins life" as console text before it graduates into a button/panel.

use std::collections::HashMap;

/// Parse a `[macros]` table from TOML text into `name -> [verb lines...]`.
pub fn load_macros(toml_text: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let value: toml::Value = toml::from_str(toml_text).map_err(|e| e.to_string())?;

    let mut out = HashMap::new();
    let Some(table) = value.get("macros").and_then(|v| v.as_table()) else {
        return Ok(out);
    };

    for (name, steps) in table {
        let arr = steps
            .as_array()
            .ok_or_else(|| format!("macro '{name}' must be an array of strings"))?;
        let mut expanded = Vec::with_capacity(arr.len());
        for step in arr {
            let s = step
                .as_str()
                .ok_or_else(|| format!("macro '{name}' steps must be strings"))?;
            expanded.push(s.to_string());
        }
        out.insert(name.clone(), expanded);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_macros() {
        let m = load_macros("[macros]\nsweep = [\"/scan\", \"/balance\"]\n").unwrap();
        assert_eq!(m["sweep"], vec!["/scan", "/balance"]);
    }

    #[test]
    fn empty_without_table() {
        let m = load_macros("").unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn rejects_non_array() {
        assert!(load_macros("[macros]\nbad = 1\n").is_err());
    }
}
