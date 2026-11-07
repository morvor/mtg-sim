//! Cost grammar IV (cards3-costs-mana-replacements).
//!
//! - "The next instant or sorcery spell you cast this turn costs {X} less to cast, where X
//!   is the number of Wizards you control as this ability resolves." (Kaza, Roil Chaser;
//!   Maelstrom Muse): the reduction a resolving ability grants the next spell is locked in
//!   as it resolves anyway (CR 608.2h; `next_spell::exec_next_spell`), so "as this ability
//!   resolves" only says so.
//! - "You gain 3 life plus an additional 3 life for each additional {1}{G} you paid."
//!   (Taste of Paradise): after "As an additional cost to cast this spell, you may pay
//!   {1}{G} any number of times.", the number of times that cost was paid (CR 601.2b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};

/// "... costs {X} less to cast, where X is [value] as this ability resolves".
fn next_spell_x_as_resolves(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l.strip_suffix(" as this ability resolves")?;
    if !head.contains(" spell you cast ") || !head.contains(" less to cast, where x is ") {
        return None;
    }
    let saved = b.targets.len();
    let e = parse_clause(head, b);
    let ok = matches!(&e, Some(Effect::NextSpell { mods, .. })
        if mods.iter().all(|m| matches!(m, Modification::AddAbility(a)
            if matches!(&a.kind, AbilityKind::Static(s)
                if matches!(&s.effect, StaticEffect::CostModifier(cm)
                    if matches!(cm.change, CostChange::ReduceGeneric(_)))))));
    if !ok || b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    e
}

inventory::submit! { EffectPattern { name: "costs 4: the next spell costs {X} less, where X is ... as this ability resolves", priority: 100, parse: next_spell_x_as_resolves } }

/// "you gain N life plus an additional M life for each additional [mana] you paid".
fn gain_for_each_additional_paid(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you gain ")?;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start().strip_prefix("life plus an additional ")?;
    let (m, r) = parse_number(r)?;
    let mana = r
        .trim_start()
        .strip_prefix("life for each additional ")?
        .strip_suffix(" you paid")?;
    if !mana.starts_with('{') || mana.contains(' ') {
        return None;
    }
    // The repeatable additional cost it counts (`spell_cost_grammar`).
    let name = format!("pay {mana} any number of times");
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(&format!("as an additional cost to cast ~, you may {name}."))
        && !raw.contains(&format!("as an additional cost to cast this spell, you may {name}."))
    {
        return None;
    }
    let (Value::Const(_), Value::Const(k)) = (&n, &m) else {
        return None;
    };
    let times = Value::Custom(crate::kw::spell_cost_grammar::paid_times(&name).into());
    let extra = match k {
        1 => times,
        k => Value::Mul(Box::new(Value::c(*k)), Box::new(times)),
    };
    Some(Effect::GainLife {
        who: PlayerRef::You,
        n: Value::Sum(vec![n, extra]),
    })
}

inventory::submit! { EffectPattern { name: "costs 4: you gain N life plus M for each additional [mana] you paid", priority: 100, parse: gain_for_each_additional_paid } }
