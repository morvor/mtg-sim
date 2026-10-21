//! Riders on a spell being cast, from a triggered ability or a "next spell" effect
//! (CR 611.2f, 608.2n, 614.1a, 400.7a):
//!
//! * "The next time you cast an instant or sorcery spell from your hand this turn, put that
//!   card into your hand instead of into your graveyard as it resolves." (Soulfire Grand
//!   Master);
//! * "exile that card instead of putting it into your graveyard as it resolves. If you do,
//!   return it to your hand at the beginning of the next end step." (Feather, the
//!   Redeemed);
//! * "you may exile that spell instead of putting it into its owner's graveyard as it
//!   resolves" after mana spent on it (Forger's Foundry: exiled with the source, see
//!   `kw/exile_as_it_resolves_linked.rs`);
//! * "it enters with an additional +1/+1 counter on it and gains hexproof until your next
//!   turn" (Jade Orb of Dragonkind): both instructions about the spell.
//!
//! See `kw/spell_destination_markers.rs`.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::spell_destination_markers::{marker, EXILE_THEN_HAND, TO_HAND};
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// "The next time you cast an instant or sorcery spell from your hand this turn, put that
/// card into your hand instead of into your graveyard as it resolves."
fn p_next_time_to_hand(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("the next time you cast ")?;
    let (spell, rest) = r.split_once(" this turn, ")?;
    if rest != "put that card into your hand instead of into your graveyard as it resolves" {
        return None;
    }
    let (spell, from_hand) = match spell.strip_suffix(" from your hand") {
        Some(s) => (s, true),
        None => (spell, false),
    };
    let spell = spell
        .strip_prefix("an ")
        .or_else(|| spell.strip_prefix("a "))?;
    let mut parts = super::spell_cost_grammar::spell_noun(spell, true)?;
    // Only an instant or sorcery resolves into a graveyard.
    let s = format!("{parts:?}");
    if !(s.contains("Instant") || s.contains("Sorcery")) || s.contains("Creature") {
        return None;
    }
    if from_hand {
        parts.push(Filter::CastFrom(ZoneKind::Hand));
    }
    Some(Effect::NextSpell {
        filter: Filter::and(parts),
        mods: vec![marker(TO_HAND)],
        expires: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "spell destination riders: the next time you cast a spell, put it into your hand as it resolves", priority: 100, parse: p_next_time_to_hand } }

/// "exile that card instead of putting it into your graveyard as it resolves" in an ability
/// that triggers on casting a spell: that spell (a replacement effect of where it goes).
fn p_exile_trigger_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let l = end(l.trim());
    // "you may exile that spell instead of putting it into its owner's graveyard as it
    // resolves" (after mana spent on it): exiled with the source.
    if matches!(
        l,
        "you may exile that spell instead of putting it into its owner's graveyard as it resolves"
            | "you may exile that spell instead of putting it into your graveyard as it resolves"
    ) {
        return Some(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Custom(
                crate::kw::exile_as_it_resolves_linked::EFFECT.into(),
            )),
        });
    }
    // (The same after "you may" was read as the choice.)
    if matches!(
        l,
        "exile that spell instead of putting it into its owner's graveyard as it resolves"
            | "exile that spell instead of putting it into your graveyard as it resolves"
    ) {
        return Some(Effect::Custom(
            crate::kw::exile_as_it_resolves_linked::EFFECT.into(),
        ));
    }
    if l != "exile that card instead of putting it into your graveyard as it resolves" {
        return None;
    }
    Some(Effect::Modify {
        what: Sel::TriggerSpell,
        mods: vec![crate::kw::plot::exile_as_it_resolves(false)],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "spell destination riders: exile the triggering spell as it resolves", priority: 100, parse: p_exile_trigger_spell } }

/// "If you do, return it to your hand at the beginning of the next end step." after that.
fn f_return_at_end(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim()) != "if you do, return it to your hand at the beginning of the next end step"
    {
        return false;
    }
    let Effect::Modify { what: Sel::TriggerSpell, mods, .. } = prev else {
        return false;
    };
    let is_exile = |m: &Modification| {
        matches!(m, Modification::AddAbility(a) if a.text == crate::kw::plot::EXILE_AS_IT_RESOLVES)
    };
    if mods.len() != 1 || !is_exile(&mods[0]) {
        return false;
    }
    mods[0] = marker(EXILE_THEN_HAND);
    true
}

inventory::submit! { FollowupPattern { name: "spell destination riders: if you do, return it to your hand at the next end step", priority: -5, apply: f_return_at_end } }

/// "it enters with an additional +1/+1 counter on it and gains hexproof until your next
/// turn" about a spell being cast: both instructions.
fn p_enters_with_and_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let l = end(l.trim());
    let (subject, r) = [("it ", "it"), ("that creature ", "that creature")]
        .iter()
        .find_map(|(p, s)| l.strip_prefix(p).map(|r| (*s, r)))?;
    let r = r.strip_prefix("enters with ")?;
    let (counters, gains) = r.split_once(" and gains ")?;
    let first = parse_effect_text(&format!("{subject} enters with {counters}."), b)?;
    let second = parse_effect_text(&format!("{subject} gains {gains}."), b)?;
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { EffectPattern { name: "spell destination riders: it enters with counters and gains [keyword]", priority: 100, parse: p_enters_with_and_gains } }
