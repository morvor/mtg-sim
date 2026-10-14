//! "Choose a color. Add an amount of mana of that color equal to your devotion to that
//! color." (Nykthos, Shrine to Nyx; Nyx Lotus; CR 700.5), and player counters gotten
//! without a subject after "you may" ("you may get two rad counters", CR 122.1).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn devotion_to_chosen_color(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "add an amount of mana of that color equal to your devotion to that color" {
        return false;
    }
    if !matches!(
        prev,
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Color,
        }
    ) {
        return false;
    }
    *prev = Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::DevotionToChosenColor,
        restriction: None,
    };
    true
}

inventory::submit! { FollowupPattern { name: "game rules: choose a color, add mana equal to your devotion to it", priority: 100, apply: devotion_to_chosen_color } }

/// "get two rad counters" after "you may" (Tato Farmer): you get them.
fn get_player_counters(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("get ")?;
    let (n, r) = parse_number(r)?;
    let kind = r
        .trim_start()
        .strip_suffix(" counters")
        .or_else(|| r.trim_start().strip_suffix(" counter"))?;
    if !matches!(kind, "rad" | "poison" | "experience" | "ticket") {
        return None;
    }
    Some(Effect::AddPlayerCounters {
        who: PlayerRef::You,
        kind: kind.into(),
        n,
    })
}

inventory::submit! { EffectPattern { name: "game rules: get N [kind] counters (after you may)", priority: 100, parse: get_player_counters } }

/// "If target player has fewer than nine poison counters, they get a number of poison
/// counters equal to the difference." (Vraska, Betrayal's Sting): up to that many.
fn up_to_n_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if target player has fewer than ")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = r.trim_start().split_once(" counters, they get a number of ")?;
    if r != format!("{kind} counters equal to the difference") {
        return None;
    }
    if !matches!(kind, "poison" | "rad" | "energy" | "experience") {
        return None;
    }
    let slot = b.add_target(
        TargetSpec::player(PlayerFilter::Any, "target player"),
        "target player",
    );
    let who = PlayerRef::Target(slot);
    b.it_player = who.clone();
    let have = Value::PlayerCounters(who.clone(), kind.into());
    Some(Effect::If {
        cond: Condition::Compare(have.clone(), Cmp::Lt, n.clone()),
        then: Box::new(Effect::AddPlayerCounters {
            who,
            kind: kind.into(),
            n: Value::Diff(Box::new(n), Box::new(have)),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "game rules: if target player has fewer than N counters, they get the difference", priority: 100, parse: up_to_n_counters } }
