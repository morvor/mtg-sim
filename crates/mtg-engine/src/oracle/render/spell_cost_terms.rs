//! Spell cost changes qualified by who casts the spell or when (`kw/spell_cost_grammar.rs`,
//! CR 601.2f): "The second spell you cast each turn costs {1} less to cast.", "Each spell
//! costs {3} more to cast except during its controller's turn.", "Spells with the chosen
//! name enchanted player casts cost {2} more to cast."

use super::*;
use crate::kw::spell_cost_grammar as scg;

/// The spell filter without the quality `name`, if it has it.
fn without(f: &Filter, name: &str) -> Option<Filter> {
    let Filter::And(v) = f else {
        return None;
    };
    let is_it = |x: &Filter| matches!(x, Filter::Custom(n) if n == name);
    if !v.iter().any(is_it) {
        return None;
    }
    let rest: Vec<Filter> = v.iter().filter(|x| !is_it(x)).cloned().collect();
    Some(if rest.len() == 1 {
        rest.into_iter().next().unwrap_or(Filter::Spell)
    } else {
        Filter::And(rest)
    })
}

impl Renderer<'_> {
    /// The cost change, if its spells are qualified in a way this module words.
    pub(crate) fn qualified_cost_modifier(&mut self, cm: &CostModifier) -> Option<String> {
        let CostTarget::Spells(f) = &cm.applies_to else {
            return None;
        };
        // The spell being cast is the event the cost change looks at: "that spell"
        // (`kw/spell_cost_grammar.rs`).
        if !self.event_scope {
            return Some(self.in_event_scope(|r| r.cost_modifier(cm)));
        }
        let with = |base: Filter| {
            let mut cm2 = cm.clone();
            cm2.applies_to = CostTarget::Spells(base);
            cm2
        };
        // "The second spell you cast each turn costs {1} less to cast": the second spell
        // its caster casts this turn.
        if let Some(base) = without(f, scg::SECOND_THIS_TURN) {
            let s = self.cost_modifier(&with(base.clone()));
            let n = self.noun(&base, Num::One);
            let n = if n.contains("spell") {
                n
            } else {
                "spell".to_string()
            };
            let who = if cm.who == PlayerRel::Any {
                "a player".to_string()
            } else {
                self.rel_subject(cm.who)
            };
            let i = s.find(" cost ")?;
            return Some(format!("the second {n} {who} cast each turn{}", &s[i..]));
        }
        // "Each spell costs {3} more to cast except during its controller's turn".
        if let Some(base) = without(f, scg::CASTER_NOT_ACTIVE) {
            let s = self.cost_modifier(&with(base));
            return Some(format!("{s} except during its controller's turn"));
        }
        // "Spells with the chosen name enchanted player casts cost {2} more to cast".
        if let Some(base) = without(f, scg::CASTER_ENCHANTED) {
            if cm.who != PlayerRel::Any {
                return None;
            }
            let s = self.cost_modifier(&with(base));
            let i = s.find(" cost ")?;
            return Some(format!("{} enchanted player casts{}", &s[..i], &s[i..]));
        }
        None
    }
}
