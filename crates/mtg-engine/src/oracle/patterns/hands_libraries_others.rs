//! Hands, libraries and graveyards — other players' and odd positions:
//!
//! * the bottom card of a library (CR 401.2): "Put the bottom card of your library into
//!   your graveyard." (Grenzo, Dungeon Warden), "Target player puts the bottom card of
//!   their library into their graveyard." (Cellar Door), "exile the bottom card of each
//!   opponent's library face down" (Arvinox, the Mind Flail);
//! * reordering a graveyard (CR 404.2): "reorder your graveyard as you choose" (Fossil
//!   Find), "Reorder your graveyard at random." (Search for Survivors);
//! * "Put target nonland permanent into its owner's library just beneath the top X cards
//!   of that library." (Unexpectedly Absent, Quarry Colossus);
//! * cards revealed at random from a hand (CR 701.20a): "Target player reveals two cards at
//!   random from their hand, then discards each nonland card revealed this way." (Rise //
//!   Fall), "Target creature's controller reveals a card at random from their hand."
//!   (Friendly Fire);
//! * a card from a hand just revealed or looked at: "You may put a creature card from it
//!   onto the battlefield under your control." (Treacherous Urge), "Look at target
//!   opponent's hand and choose a card from it. Put that card on top of that player's
//!   library." (Painful Memories);
//! * "You may then have that player shuffle that library." (Visions).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::zones_dig_effects::{
    beneath_effect, Beneath, REORDER_GRAVEYARD_CHOOSE, REORDER_GRAVEYARD_RANDOM,
};
use crate::oracle::effects::{object_ref, parse_sentence, player_ref, Builder};
use crate::oracle::phrases::end;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// The bottom card of a library
// ---------------------------------------------------------------------------

/// "the bottom card of your library", "the bottom card of their library", "the bottom
/// card of each opponent's library": whose library (`who` for "their").
fn bottom_card<'a>(s: &'a str, who: &PlayerRef) -> Option<(Sel, &'a str)> {
    let r = s.strip_prefix("the bottom card of ")?;
    for (p, w) in [
        ("your library", PlayerRef::You),
        ("their library", who.clone()),
        ("each opponent's library", PlayerRef::EachOpponent),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((Sel::BottomOfLibrary(w, Value::c(1)), rest));
        }
    }
    None
}

fn bottom_card_moves(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "Put the bottom card of your library into your graveyard."
    if let Some(r) = l.strip_prefix("put ") {
        let (what, rest) = bottom_card(r, &PlayerRef::You)?;
        if !matches!(what, Sel::BottomOfLibrary(PlayerRef::You, _)) || rest != " into your graveyard" {
            return None;
        }
        b.it = Sel::Var(vars::IT);
        return Some(Effect::Move {
            what,
            to: Destination::zone(ZoneKind::Graveyard),
        });
    }
    // "exile the bottom card of each opponent's library face down".
    if let Some(r) = l.strip_prefix("exile ") {
        let (what, rest) = bottom_card(r, &PlayerRef::You)?;
        let face_down = match rest {
            "" => false,
            " face down" => true,
            _ => return None,
        };
        b.it = Sel::Var(vars::IT);
        return Some(Effect::Exile {
            what,
            face_down,
            link: false,
        });
    }
    // "Target player puts the bottom card of their library into their graveyard."
    let (subject, r) = l.split_once(" puts ")?;
    let saved = b.targets.len();
    let (who, extra) = player_ref(subject, b)?;
    let parsed = (|| {
        if !extra.trim().is_empty() || matches!(who, PlayerRef::You) {
            return None;
        }
        let (what, rest) = bottom_card(r, &who)?;
        (rest == " into their graveyard").then_some(what)
    })();
    let Some(what) = parsed else {
        b.targets.truncate(saved);
        return None;
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Move {
        what,
        to: Destination::zone(ZoneKind::Graveyard),
    })
}

inventory::submit! { EffectPattern { name: "the bottom card of a library: put it into a graveyard, exile it", priority: 100, parse: bottom_card_moves } }

// ---------------------------------------------------------------------------
// Reordering a graveyard
// ---------------------------------------------------------------------------

fn reorder_graveyard(l: &str, _b: &mut Builder) -> Option<Effect> {
    let name = match end(l) {
        "reorder your graveyard as you choose" => REORDER_GRAVEYARD_CHOOSE,
        "reorder your graveyard at random" => REORDER_GRAVEYARD_RANDOM,
        _ => return None,
    };
    Some(Effect::Custom(SmolStr::new(name)))
}

inventory::submit! { EffectPattern { name: "reorder your graveyard (as you choose / at random)", priority: 100, parse: reorder_graveyard } }

// ---------------------------------------------------------------------------
// Just beneath the top X cards of a library
// ---------------------------------------------------------------------------

/// "put target nonland permanent into its owner's library just beneath the top X cards
/// of that library[, where X is the number of Plains you control]".
fn beneath_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("put ")?;
    let (obj, r) = r.split_once(" into its owner's library just beneath the top ")?;
    let saved = b.targets.len();
    let parsed = (|| {
        let (what, rest) = object_ref(obj, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        let (n, r) = crate::oracle::phrases::parse_number(r)?;
        let r = r.trim_start().strip_prefix("cards of that library")?;
        let n = match r.strip_prefix(", where x is ") {
            Some(v) if matches!(n, Value::X) => {
                let (v, rest) = super::r107_numbers::value_phrase(v, b)?;
                if !rest.trim().is_empty() {
                    return None;
                }
                v
            }
            Some(_) => return None,
            None if r.is_empty() => {
                // X is the spell's.
                if matches!(n, Value::X) && (!b.ctx.is_spell() || b.in_trigger) {
                    return None;
                }
                n
            }
            None => return None,
        };
        beneath_effect(&Beneath { what, n })
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "put [object] into its owner's library just beneath the top X cards", priority: 100, parse: beneath_top } }

// ---------------------------------------------------------------------------
// Cards revealed at random from a hand
// ---------------------------------------------------------------------------

/// "Target player reveals two cards at random from their hand[, then discards each
/// nonland card revealed this way]", "Target creature's controller reveals a card at
/// random from their hand."
fn reveal_at_random_n(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) = l.split_once(" reveals ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let n = n.as_const()?.max(0) as u32;
    let r = r
        .trim_start()
        .strip_prefix(if n == 1 { "card" } else { "cards" })?
        .strip_prefix(" at random from their hand")?;
    let saved = b.targets.len();
    let who = if let Some(x) = subject.strip_prefix("target creature's controller") {
        if !x.is_empty() {
            return None;
        }
        let slot = b.add_target(
            TargetSpec::one(
                TargetKind::Object(Filter::and(vec![
                    Filter::Type(crate::types::CardType::Creature),
                    Filter::InZone(ZoneKind::Battlefield),
                ])),
                "target creature",
            ),
            "target creature",
        );
        b.it = Sel::Target(slot);
        PlayerRef::ControllerOf(Box::new(Sel::Target(slot)))
    } else {
        let (who, extra) = player_ref(subject, b)?;
        if !extra.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
            b.targets.truncate(saved);
            return None;
        }
        who
    };
    // Single-card reveals by a target player are `reveal_at_random.rs`'s.
    if n == 1 && matches!(who, PlayerRef::Target(_)) && r.is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    let Some(name) = crate::kw::reveal_at_random::effect_name_n(&who, n) else {
        b.targets.truncate(saved);
        return None;
    };
    b.it_player = who.clone();
    let reveal = Effect::Custom(SmolStr::new(name));
    let effect = match r {
        "" => reveal,
        ", then discards each nonland card revealed this way" => {
            let f = Filter::and(vec![
                Filter::In(Box::new(Sel::Var(vars::IT))),
                Filter::not(Filter::Type(crate::types::CardType::Land)),
            ]);
            Effect::seq(vec![
                reveal,
                Effect::Discard {
                    who,
                    n: Value::CountSel(Box::new(Sel::All(f.clone()))),
                    random: false,
                    filter: f,
                },
            ])
        }
        _ => {
            b.targets.truncate(saved);
            return None;
        }
    };
    // "the revealed card" (Friendly Fire).
    if n == 1 {
        b.named
            .push(("the revealed card".to_string(), Sel::Var(vars::IT)));
    }
    Some(effect)
}

inventory::submit! { EffectPattern { name: "[player] reveals N cards at random from their hand", priority: 99, parse: reveal_at_random_n } }

// ---------------------------------------------------------------------------
// A card from a hand just revealed or looked at
// ---------------------------------------------------------------------------

/// The variable for the card put from a hand.
const FROM_HAND: Var = vars::USER + 2330;

/// The player whose hand the effect revealed or looked at last.
fn hand_owner(e: &Effect) -> Option<PlayerRef> {
    match e {
        Effect::RevealHand { who } | Effect::LookAtHand { who } => Some(who.clone()),
        Effect::Seq(v) => v.last().and_then(hand_owner),
        _ => None,
    }
}

/// "You may put a creature card from it onto the battlefield under your control." after
/// a hand was revealed or looked at (Treacherous Urge): a card of that player's hand,
/// chosen (or none) as the instruction is performed; "that creature" and "it" afterward.
fn put_from_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(owner) = hand_owner(prev) else {
        return false;
    };
    let Some(r) = end(l).strip_prefix("you may put ") else {
        return false;
    };
    let Some((desc, dest)) = r.split_once(" from it ") else {
        return false;
    };
    if dest != "onto the battlefield under your control" {
        return false;
    }
    let Some(desc) = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an ")) else {
        return false;
    };
    let Some(f) = super::card_flow_search::card_filter(desc, b) else {
        return false;
    };
    let choose = Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedByPlayer(Box::new(owner)),
        ]),
        count: Value::c(1),
        up_to: true,
        store: None,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Store {
            var: FROM_HAND,
            sel: choose,
        },
        Effect::Move {
            what: Sel::Var(FROM_HAND),
            to: Destination::battlefield().under_your_control(),
        },
    ]);
    // "That creature gains haste. Sacrifice it at the beginning of the next end step."
    b.it = Sel::Var(vars::IT);
    b.named
        .push(("that creature".to_string(), Sel::Var(vars::IT)));
    true
}

inventory::submit! { FollowupPattern { name: "you may put a [card] from it (a hand) onto the battlefield", priority: 95, apply: put_from_it } }

/// "Look at target opponent's hand and choose a card from it." (which Word of Command's
/// grammar reads as its first step) followed by an instruction about that card that
/// isn't Word of Command's: the card is chosen from the hand looked at (Painful
/// Memories: "Put that card on top of that player's library.").
fn chose_from_looked_at_hand(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::Custom(name) = &*prev else {
        return false;
    };
    let Some(slot) = name
        .strip_prefix(crate::player_control::WORD_OF_COMMAND)
        .and_then(|r| r.strip_prefix("choose:"))
        .and_then(|s| s.parse::<u8>().ok())
    else {
        return false;
    };
    if l.contains("control that player") || l.contains("plays that card") {
        return false;
    }
    let who = PlayerRef::Target(slot);
    let choose = Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(vec![
            Filter::Card,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedByPlayer(Box::new(who.clone())),
        ]),
        count: Value::c(1),
        up_to: false,
        store: None,
    };
    let saved = (b.it.clone(), b.named.len());
    b.it = Sel::Var(FROM_HAND);
    b.it_player = who.clone();
    for n in ["that card", "the chosen card"] {
        b.named.push((n.to_string(), Sel::Var(FROM_HAND)));
    }
    let Some(then) = parse_sentence(l, b) else {
        b.it = saved.0;
        b.named.truncate(saved.1);
        return false;
    };
    *prev = Effect::seq(vec![
        Effect::LookAtHand { who },
        Effect::Store {
            var: FROM_HAND,
            sel: choose,
        },
        then,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "look at a hand and choose a card from it: an instruction about that card", priority: 59, apply: chose_from_looked_at_hand } }

/// "You may then have that player shuffle that library." (Visions): you decide whether
/// the player whose library you looked at shuffles it (the "you may" is read around
/// this).
fn may_have_shuffle(l: &str, b: &mut Builder) -> Option<Effect> {
    // "You may" is read around it (`effects::parse_sentence`).
    if !matches!(
        end(l),
        "then have that player shuffle that library"
            | "have that player shuffle that library"
            | "have that player shuffle their library"
    ) {
        return None;
    }
    // The target player whose library the text is about.
    let who = match &b.it_player {
        PlayerRef::Target(_) => b.it_player.clone(),
        _ => {
            let players: Vec<usize> = b
                .targets
                .iter()
                .enumerate()
                .filter(|(_, t)| matches!(t.what, TargetKind::Player(_)))
                .map(|(i, _)| i)
                .collect();
            match players[..] {
                [i] if b.targets.len() == 1 => PlayerRef::Target(i as u8),
                _ => return None,
            }
        }
    };
    Some(Effect::Shuffle { who })
}

/// "An opponent chooses a card at random in your graveyard." (Search for Survivors): a
/// card of your graveyard at random ("it").
fn chooses_at_random_in_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "an opponent chooses a card at random in your graveyard" {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Store {
        var: vars::IT,
        sel: Sel::AtRandom {
            filter: Filter::and(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            count: Value::c(1),
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "an opponent chooses a card at random in your graveyard", priority: 100, parse: chooses_at_random_in_graveyard } }

inventory::submit! { EffectPattern { name: "you may have that player shuffle that library", priority: 100, parse: may_have_shuffle } }

// ---------------------------------------------------------------------------
// Other players' graveyards and cards
// ---------------------------------------------------------------------------

/// "exile up to one card of each card type from defending player's graveyard" (Grime
/// Gorger): any number of that graveyard's cards, each standing for a different card type
/// it has (`TargetGroup::OnePerCardType`).
fn exile_one_of_each_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile up to one card of each card type from ")?;
    let whose = r.strip_suffix(" graveyard")?;
    let who = match whose {
        "your" => PlayerRef::You,
        "defending player's" => PlayerRef::DefendingPlayer,
        _ => return None,
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedByPlayer(Box::new(who)),
                Filter::Together(TargetGroup::OnePerCardType),
            ]),
            count: Value::c(crate::types::CardType::ALL.len() as i32),
            up_to: true,
            store: None,
        },
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "exile up to one card of each card type from a graveyard", priority: 100, parse: exile_one_of_each_type } }

/// "Target opponent exiles a creature they control and their graveyard." (Strategic
/// Betrayal): that player chooses the creature; their whole graveyard is exiled.
fn exiles_creature_and_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subject, r) = end(l).split_once(" exiles a ")?;
    let desc = r.strip_suffix(" they control and their graveyard")?;
    let saved = b.targets.len();
    let parsed = (|| {
        let (who, extra) = player_ref(subject, b)?;
        if !extra.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
            return None;
        }
        let (f, _, rest) = crate::oracle::phrases::parse_object_phrase(desc)?;
        if !rest.trim().is_empty() || f.zone().is_some() {
            return None;
        }
        let rel = match &who {
            PlayerRef::Target(n) => PlayerRel::Target(*n),
            _ => return None,
        };
        Some(Effect::seq(vec![
            Effect::Exile {
                what: Sel::Choose {
                    chooser: who.clone(),
                    filter: Filter::and(vec![
                        f,
                        Filter::InZone(ZoneKind::Battlefield),
                        Filter::ControlledBy(rel),
                    ]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
                face_down: false,
                link: false,
            },
            Effect::Exile {
                what: Sel::All(Filter::and(vec![
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedByPlayer(Box::new(who)),
                ])),
                face_down: false,
                link: false,
            },
        ]))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "[player] exiles a [permanent] they control and their graveyard", priority: 100, parse: exiles_creature_and_graveyard } }

/// "that player reveals the chosen card" (Vendilion Clique): the card chosen from their
/// hand is revealed.
fn reveals_the_chosen_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("that player reveals ")
        .or_else(|| l.strip_prefix("reveal "))?;
    if r != "the chosen card" {
        return None;
    }
    let (what, rest) = object_ref(r, b)?;
    if !rest.trim().is_empty() || !matches!(what, Sel::Var(_)) {
        return None;
    }
    crate::kw::zones_dig_effects::reveal_effect(&what)
}

inventory::submit! { EffectPattern { name: "that player reveals the chosen card", priority: 100, parse: reveals_the_chosen_card } }
