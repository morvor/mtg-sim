//! Opus (an ability word, see `kw/opus.rs`): "[N] or more mana was spent to cast that
//! spell", the spell that caused the ability to trigger.

use crate::ability::*;
use crate::kw::opus::MANA_SPENT_ON_THAT_SPELL;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{ConditionPattern, FollowupPattern};
use crate::oracle::phrases::parse_number;

/// Whether the condition `c` is one parsed here: its "that spell" is the spell that caused
/// the ability to trigger, not something the previous sentence refers to.
pub fn refers_to_the_trigger_spell(c: &str) -> bool {
    mana_spent_on_that_spell(c).is_some()
}

fn mana_spent_on_that_spell(c: &str) -> Option<Condition> {
    let c = c.trim().trim_end_matches(['.', ',']);
    let r = c.strip_suffix(" or more mana was spent to cast that spell")?;
    let (n, rest) = parse_number(r)?;
    if !rest.trim().is_empty() {
        return None;
    }
    n.as_const()?;
    Some(Condition::Compare(
        Value::Custom(MANA_SPENT_ON_THAT_SPELL.into()),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! {
    ConditionPattern {
        name: "opus: N or more mana was spent to cast that spell",
        priority: 100,
        parse: mana_spent_on_that_spell,
    }
}

/// "If eight or more mana was spent to cast that spell, sacrifice ~ and it deals that much
/// damage to each opponent." (Tellah, Great Sage): "that much" is the amount of mana spent
/// to cast that spell, and "it" is the sacrificed permanent, which deals the damage as it
/// last existed on the battlefield (CR 608.2h) — as it does if it had already left the
/// battlefield before the ability resolved. Appended to the previous sentence's effect.
fn that_much_damage_as_mana_spent(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some((c, rest)) = l.strip_prefix("if ").and_then(|r| r.split_once(", ")) else {
        return false;
    };
    let Some(cond) = mana_spent_on_that_spell(c) else {
        return false;
    };
    let (sacrifice, to) = if let Some(r) =
        rest.strip_prefix("sacrifice ~ and it deals that much damage to ")
    {
        (true, r)
    } else if let Some(r) = rest.strip_prefix("~ deals that much damage to ") {
        (false, r)
    } else {
        return false;
    };
    let to = match to {
        "each opponent" => Sel::Players(PlayerRef::EachOpponent),
        "each player" => Sel::Players(PlayerRef::EachPlayer),
        _ => return false,
    };
    let damage = Effect::DealDamage {
        source: Sel::This,
        amount: Value::Custom(MANA_SPENT_ON_THAT_SPELL.into()),
        to,
    };
    let then = if sacrifice {
        Effect::seq(vec![Effect::SacrificeObjects { what: Sel::This }, damage])
    } else {
        damage
    };
    let before = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        before,
        Effect::If {
            cond,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! {
    FollowupPattern {
        name: "opus: if N or more mana was spent to cast that spell, ~ deals that much damage",
        priority: 100,
        apply: that_much_damage_as_mana_spent,
    }
}
