//! "{T}: Add {R}. If that mana is spent on a Dragon creature spell, it gains haste until
//! end of turn." (Carnelian Orb of Dragonkind), "If any of that mana is spent on a
//! creature spell, it gains haste until end of turn." (Generator Servant), "If that mana
//! is spent on a creature spell, it gains haste." (Hall of the Bandit Lord): an
//! additional effect of the mana that affects the spell it's spent on (CR 106.6), applied
//! as the mana is spent (no delayed triggered ability), once for each mana (CR 106.6a).
//! An effect that grants the spell an ability continues to apply to the permanent that
//! spell becomes (CR 400.7a).
//!
//! ("If that mana is spent to cast a creature spell, that creature enters with ..." is
//! `etb_replacement_grammar`'s.)

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The spell the mana must be spent on: "a creature spell", "a Dragon creature spell".
fn spell_phrase(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let f = match parse_object_phrase(s) {
        Some((f, false, rest)) if end(rest).is_empty() => f,
        _ => super::mana_restrictions::spell_alternatives(s)?,
    };
    Some(Filter::and(vec![f, Filter::Spell]))
}

/// Whether an effect only grants the spell abilities or changes its characteristics (what
/// carries over to the permanent it becomes, CR 400.7a).
fn grants_to_spell(e: &Effect) -> bool {
    match e {
        Effect::Modify { what, .. } => matches!(what, Sel::TriggerSpell),
        Effect::Seq(v) => !v.is_empty() && v.iter().all(grants_to_spell),
        _ => false,
    }
}

fn mana_spent_rider(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let (idx, marker) = [
        ". if that mana is spent on ",
        ". if any of that mana is spent on ",
        ". if that mana is spent to cast ",
        ". if any of that mana is spent to cast ",
    ]
    .into_iter()
    .find_map(|m| lower.find(m).map(|i| (i, m)))?;
    let head = &block[..idx + 1];
    let tail = end(&lower[idx + marker.len()..]);
    let (spell_s, eff) = tail.split_once(", ")?;
    if !eff.starts_with("it gains ") {
        return None;
    }
    let spell_filter = spell_phrase(spell_s)?;
    let body =
        crate::oracle::effects::parse_trigger_body(eff, ctx, Sel::TriggerSpell, PlayerRef::You)?;
    if !body.targets.is_empty() || body.modal.is_some() || !grants_to_spell(&body.effect) {
        return None;
    }
    let abilities = crate::oracle::parse_ability(head, ctx)?;
    let mut out = Vec::new();
    for a in abilities {
        let AbilityKind::Activated(act) = &a.kind else {
            return None;
        };
        if !matches!(act.body.effect, Effect::AddMana { .. }) {
            return None;
        }
        let mut act = act.clone();
        act.body.effect = Effect::AddManaWithSpentTrigger {
            add: Box::new(act.body.effect.clone()),
            spell_filter: spell_filter.clone(),
            body: Box::new(body.clone()),
            abilities: false,
            additional: true,
        };
        act.is_mana_ability = act.body.targets.is_empty() && !act.is_loyalty;
        out.push(AbilityDef::new(AbilityKind::Activated(act), block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "mana spent rider: if that mana is spent on a spell, it gains [ability]", priority: 150, parse: mana_spent_rider } }
