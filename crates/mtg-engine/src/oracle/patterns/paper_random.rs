//! Random hand cards in older Oracle wordings:
//!
//! - "Look at a card at random in target player's hand." (Urza's Bauble): see
//!   `kw/look_at_random.rs`;
//! - "Target opponent reveals X cards at random from their hand. Then that player
//!   discards all cards with that name revealed this way." (Nebuchadnezzar: X is the
//!   ability's X; only the revealed cards with the chosen name are discarded);
//! - "Discard any number of cards at random." (Rites of Initiation: the number is chosen
//!   as the spell resolves, then that many cards are discarded at random).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// "look at a card at random in target player's hand" (also "target opponent's").
fn look_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("look at a card at random in ")?;
    let subj = r.strip_suffix("'s hand")?;
    if !subj.starts_with("target ") {
        return None;
    }
    let saved = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    let PlayerRef::Target(slot) = who else {
        b.targets.truncate(saved);
        return None;
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Custom(SmolStr::new(
        crate::kw::look_at_random::effect_name(slot),
    )))
}

inventory::submit! { EffectPattern { name: "look at a card at random in target player's hand", priority: 300, parse: look_at_random } }

/// "target opponent reveals x cards at random from their hand".
fn reveal_x_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let subj = l.strip_suffix(" reveals x cards at random from their hand")?;
    if !subj.starts_with("target ") {
        return None;
    }
    let saved = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    if !rest.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
        b.targets.truncate(saved);
        return None;
    }
    let Some(name) = crate::kw::reveal_at_random::effect_name_x(&who) else {
        b.targets.truncate(saved);
        return None;
    };
    b.it_player = who;
    Some(Effect::Custom(SmolStr::new(name)))
}

inventory::submit! { EffectPattern { name: "target opponent reveals X cards at random from their hand", priority: 300, parse: reveal_x_at_random } }

/// Adds "revealed this way" to the first discard of an effect: only the revealed cards.
fn only_revealed(e: &mut Effect) -> bool {
    match e {
        Effect::Discard { filter, .. } => {
            *filter = Filter::and(vec![
                filter.clone(),
                Filter::In(Box::new(Sel::Var(vars::IT))),
            ]);
            true
        }
        Effect::Seq(v) => v.iter_mut().any(only_revealed),
        Effect::AsPlayer { effect, .. } => only_revealed(effect),
        _ => false,
    }
}

/// "[player] discards all cards with that name revealed this way": the revealed cards
/// (`vars::IT` of the reveal at random) with the chosen name.
fn discards_revealed_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l.strip_suffix(" revealed this way")?;
    if !head.ends_with(" discards all cards with that name") {
        return None;
    }
    let mut e = parse_clause(head, b)?;
    only_revealed(&mut e).then_some(e)
}

inventory::submit! { EffectPattern { name: "discards all cards with that name revealed this way", priority: 300, parse: discards_revealed_this_way } }

/// "discard any number of cards at random": a number chosen as it resolves, then that
/// many cards at random.
fn discard_any_number_at_random(l: &str, _b: &mut Builder) -> Option<Effect> {
    // Only the controller's own instruction: "Target player discards any number of
    // cards at random" (reworded for "you") doesn't say who chooses the number.
    if end(l) != "discard any number of cards at random"
        || !crate::oracle::raw_text()
            .to_lowercase()
            .contains("discard any number of cards at random")
    {
        return None;
    }
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number {
                min: 0,
                max: crate::choices::MAX_CHOSEN_NUMBER,
            },
        },
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::Chosen,
            random: true,
            filter: Filter::Any,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "discard any number of cards at random", priority: 300, parse: discard_any_number_at_random } }
