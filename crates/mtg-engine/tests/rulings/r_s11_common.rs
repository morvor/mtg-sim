//! Shared helpers for the tests of rulings batch S11 (`r_s11_*.rs`): magecraft, manifest,
//! manifest dread, mayhem, megamorph, melee, menace, mentor, mill, miracle, mobilize,
//! modular, monstrosity. (The helpers of batches S01–S07 are used too.)

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::facedown::REVEALED;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Number of triggered abilities of `source` put on the stack this turn.
pub fn triggered_from(t: &TestGame, source: ObjectId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::AbilityTriggeredOnStack { source: s, .. } if *s == source))
        .count()
}

/// Number of copies of spells put onto the stack this turn (CR 707.10).
pub fn spells_copied(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::SpellCopied { .. }))
        .count()
}

/// `p` manifests the top card of their library (CR 701.40a) as a resolving effect would.
/// Returns the manifested permanent.
pub fn manifest_top(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let top = t.g.library_top(p).expect("empty library");
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::KeywordAction {
            action: KeywordAction::Manifest,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let m = t.g.current(top);
    assert!(t.obj(m).face_down && t.on_battlefield(m), "not manifested");
    m
}

/// Puts the real card `name` on top of `p`'s library and manifests it.
pub fn manifest_card(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.library_top(p, name);
    manifest_top(t, p)
}

/// Whether turning `obj` face up is among `p`'s legal actions now (the special action).
pub fn can_turn_face_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::TurnFaceUp { obj }))
}

/// `p` turns `obj` face up with the special action (CR 116.2b). Whether it was legal.
pub fn turn_face_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let r =
        t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }));
    t.g.flush_events();
    r.is_ok()
}

/// The face-down objects revealed to all players this turn (CR 708.9, 701.40g).
pub fn revealed_this_turn(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == REVEALED => *obj,
            _ => None,
        })
        .collect()
}

/// Whether any permanent was turned face up this turn.
pub fn turned_face_up_this_turn(t: &TestGame) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::TurnedFaceUp { .. }))
}

/// Whether `id` is a face-down 2/2 creature with no name, mana cost, creature types, or
/// abilities, colorless with mana value 0 (CR 708.2a, 701.40a).
pub fn is_plain_face_down_2_2(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj(id);
    o.face_down
        && o.chars.name.is_empty()
        && (o.chars.power, o.chars.toughness) == (Some(2), Some(2))
        && o.chars.card_types == CardTypeSet::single(CardType::Creature)
        && o.chars.subtypes.is_empty()
        && o.chars.mana_cost.is_none()
        && o.chars.abilities.is_empty()
        && o.chars.colors == ColorSet::NONE
        && t.g.mana_value_of(id) == 0
}

/// Empties `p`'s library (of the harness's filler cards).
pub fn empty_library(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].library.clear();
}
