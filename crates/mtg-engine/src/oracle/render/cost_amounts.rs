//! Cost changes counted "for each" thing (CR 601.2f): "{1} less to cast for each target",
//! "for each 1 life you gained this turn", "for each counter among players and
//! permanents".

use super::*;

impl Renderer<'_> {
    /// (amount, tail) for a cost change by a number the generic forms don't word.
    pub(crate) fn cost_amount_more(&mut self, v: &Value) -> Option<(String, String)> {
        match v {
            // The number of targets the spell being cast has (each instance of the word
            // "target" chosen, CR 601.2c).
            Value::Custom(n) if n == "cost: targets of the spell being cast" => {
                Some(("{1}".into(), " for each target".into()))
            }
            // "for each counter among players and permanents": one count over two kinds
            // of things, said once.
            Value::Sum(parts) if parts.len() == 2 => {
                let mut heads = Vec::new();
                let mut wheres = Vec::new();
                for p in parts {
                    let s = self.value(p);
                    let s = s
                        .strip_prefix("the total number of ")
                        .or_else(|| s.strip_prefix("the number of "))?
                        .to_string();
                    let (head, among) = s.split_once(" among ")?;
                    heads.push(head.to_string());
                    wheres.push(among.to_string());
                }
                if heads[0] != heads[1] {
                    return None;
                }
                let each = effects::singular_head(&format!("{} among x", heads[0]));
                let each = each.strip_suffix(" among x")?.to_string();
                Some((
                    "{1}".into(),
                    format!(" for each {each} among {} and {}", wheres[0], wheres[1]),
                ))
            }
            _ => None,
        }
    }

    /// "for each 1 life you gained this turn": an amount of life counted one by one.
    pub(crate) fn per_life(&mut self, s: &str) -> Option<String> {
        let r = s
            .strip_prefix("the amount of life ")
            .or_else(|| s.strip_prefix("the {opt:total} amount of life "))
            .or_else(|| s.strip_prefix("the total amount of life "))?;
        Some(format!("1 life {r}"))
    }
}
