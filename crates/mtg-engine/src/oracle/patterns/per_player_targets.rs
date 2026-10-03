//! "For each opponent, gain control of up to one target creature that player controls
//! until end of turn." (Mass Mutiny), "For each player, exile up to one target non-Saga,
//! nonland permanent that player controls until ~ leaves the battlefield." (Battle at the
//! Helvault), "for each opponent, you may cast up to one target instant or sorcery card
//! from that player's graveyard without paying its mana cost" (Diluvian Primordial):
//! targets chosen for each player (`TargetSpec::per_player`, see
//! `src/per_player_targets.rs`).
//!
//! "That player" in the target phrase becomes an internal phrase the object phrase parser
//! reads as the player the target is chosen for (`PlayerRel::Iterated`); the rest of the
//! clause is parsed as usual and acts on every target chosen ("Untap those creatures").
//! Only when "that player" appears nowhere else: an effect that does something to or for
//! each player individually ("and that player gains life equal to its power") isn't
//! one effect on all the targets.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

const THAT_PLAYER: [(&str, &str); 4] = [
    ("that player controls", "the iterated player controls"),
    ("that opponent controls", "the iterated player controls"),
    (
        "from that player's graveyard",
        "from the iterated player's graveyard",
    ),
    ("in that player's graveyard", "in the iterated player's graveyard"),
];

fn for_each_player_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (players, rest) = if let Some(r) = l.strip_prefix("for each opponent, ") {
        (PlayerFilter::Opponent, r)
    } else if let Some(r) = l.strip_prefix("for each player, ") {
        (PlayerFilter::Any, r)
    } else if let Some(r) = l.strip_prefix("for each other player, ") {
        (PlayerFilter::NotYou, r)
    } else {
        return None;
    };
    if !rest.contains("target ") {
        return None;
    }
    let mut s = rest.to_string();
    let mut replaced = false;
    for (from, to) in THAT_PLAYER {
        if s.contains(from) {
            s = s.replace(from, to);
            replaced = true;
        }
    }
    if !replaced
        || s.contains("that player")
        || s.contains("that opponent")
        || s.contains(" they ")
        || s.contains(" their ")
    {
        return None;
    }
    let before = b.targets.len();
    let e = parse_clause(&s, b)?;
    // One instance of the word "target", an object one described relative to the player.
    if b.targets.len() != before + 1 {
        return None;
    }
    let spec = &mut b.targets[before];
    let TargetKind::Object(f) = &spec.what else {
        return None;
    };
    if !mentions_iterated(f) || spec.together.is_some() || spec.related_to.is_some() {
        return None;
    }
    spec.per_player = Some(players);
    spec.text = spec.text.replace("the iterated player", "that player");
    Some(e)
}

/// Whether every object the filter matches is described relative to the player.
fn mentions_iterated(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(PlayerRel::Iterated) | Filter::OwnedBy(PlayerRel::Iterated) => true,
        Filter::And(v) => v.iter().any(mentions_iterated),
        Filter::Or(v) => !v.is_empty() && v.iter().all(mentions_iterated),
        _ => false,
    }
}

inventory::submit! { EffectPattern { name: "for each opponent/player, ... target ... that player controls", priority: 70, parse: for_each_player_targets } }
