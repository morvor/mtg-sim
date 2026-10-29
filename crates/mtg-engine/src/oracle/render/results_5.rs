//! Rendering round-5 results grammar (`patterns::results_5`): "Each of them enters with
//! an additional +1/+1 counter on it if it's a creature and an additional loyalty counter
//! on it if it's a planeswalker." after a move onto the battlefield (Semester's End).

use super::*;

impl Renderer<'_> {
    /// `[AddReplacement(enter with counters, those objects of a type)..., Move]`: the move,
    /// then the counters each of them enters with.
    pub(crate) fn enters_with_if(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let mut parts = Vec::new();
        let mut moved: Option<&Sel> = None;
        for (i, e) in v.iter().enumerate() {
            match e {
                Effect::AddReplacement {
                    def:
                        ReplacementDef {
                            event: ReplacementEvent::EntersBattlefield(Filter::And(fs)),
                            action: ReplacementAction::EnterWithCounters(kind, Value::Const(1)),
                            self_replacement: false,
                            optional: false,
                        },
                    duration: Duration::EndOfTurn,
                    uses: None,
                } if fs.len() == 2 => {
                    let Filter::In(sel) = &fs[0] else {
                        return None;
                    };
                    if moved.is_some_and(|m| format!("{m:?}") != format!("{sel:?}")) {
                        return None;
                    }
                    moved = Some(sel);
                    let n = self.noun_det(&fs[1], nouns::Det::A);
                    parts.push(format!("an additional {kind} counter on it if it's {n}"));
                }
                Effect::Move { what, to }
                    if !parts.is_empty()
                        && to.zone == ZoneKind::Battlefield
                        && moved.is_some_and(|m| format!("{m:?}") == format!("{what:?}")) =>
                {
                    let m = self.effect(e);
                    let s = format!("{m}. Each of them enters with {}", parts.join(" and "));
                    return Some((s, i + 1));
                }
                _ => return None,
            }
        }
        None
    }
}

impl Renderer<'_> {
    /// Values about noted information (`kw/noted_info.rs`).
    pub(crate) fn noted_value(&mut self, name: &str) -> Option<String> {
        use crate::kw::noted_info as n;
        if name == n::GREATEST_NOTED_THIS_TURN {
            return Some("the greatest number noted for it this turn".into());
        }
        let slot: u8 = name.strip_prefix(n::NOTED_TYPES_OF_TARGET)?.parse().ok()?;
        let t = self.target_mention(slot, players::Case::Obj);
        Some(format!("the number of creature types noted for {t}"))
    }
}
