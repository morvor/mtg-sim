//! "During your turn, as long as ~ has one or more loyalty counters on him, he's a 3/4
//! Ninja creature and has hexproof." (Kaito, Bane of Nightmares): a static ability with a
//! turn condition and a second condition (CR 611.3a, 604.1). It's compiled as the static
//! ability with the turn condition alone, then the second condition is added to it.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn two_conditions(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let (turn, rest) = l.split_once(", as long as ")?;
    super::statics::turn_condition(turn)?;
    let (cond, rest) = rest.split_once(", ")?;
    let rest = ["he's ", "she's ", "it's "]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    // "on him" / "on her": the object is the permanent itself.
    let cond = cond.replace(" on him", " on it").replace(" on her", " on it");
    let extra = crate::oracle::statics::parse_condition(&cond, ctx)?;
    let mut abilities = crate::oracle::statics::parse_static(&format!("{turn}, ~ is {rest}"), ctx)?;
    for a in &mut abilities {
        let a = std::sync::Arc::make_mut(a);
        let AbilityKind::Static(st) = &mut a.kind else {
            return None;
        };
        st.condition = Some(match st.condition.take() {
            Some(c) => Condition::And(vec![c, extra.clone()]),
            None => extra.clone(),
        });
        a.text = text.to_string();
    }
    Some(abilities)
}

inventory::submit! { StaticPattern { name: "during your turn, as long as [condition], ~ is [predicates]", priority: 1090, parse: two_conditions } }
