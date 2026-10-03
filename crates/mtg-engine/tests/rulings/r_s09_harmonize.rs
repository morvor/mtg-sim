//! Rulings batch S09 — harmonize (CR 702.180): "You may cast this card from your
//! graveyard by paying [cost] and tapping up to one untapped creature you control rather
//! than paying its mana cost", the tapped creature's power reduces the generic mana, and
//! the spell is exiled whenever it would leave the stack.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Stage;
use mtg_engine::*;

const HARMONIZE: CastMethod = CastMethod::Keyword(KeywordKind::Harmonize);

#[test]
fn a_harmonize_card_put_into_the_graveyard_can_be_cast_before_anyone_else_acts() {
    cr!("702.180a", "117.3b", "608.2n");
    ruling!(
        "Unending Whisper",
        "If a card with harmonize is put into your graveyard during your turn, you can cast it if it’s legal to do so before any other player can take any actions."
    );
    supported("Unending Whisper");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let w = t.hand(P0, "Unending Whisper");
    t.cast(P0, w).go();
    // Both players pass; the spell resolves and is put into P0's graveyard.
    let ok = t.g.run_until(100, |g| g.stack.is_empty());
    assert!(ok);
    let w = t.g.current(w);
    assert_eq!(t.zone(w), Zone::Graveyard(P0));
    // P0 receives priority first and can cast it with harmonize right away.
    assert_eq!(t.g.turn.stage, Stage::Priority);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(can_cast(&mut t, P0, w, HARMONIZE));
    let asked = t.asked().len();
    t.cast(P0, w).method(HARMONIZE).go();
    assert!(asked_since(&t, asked)
        .iter()
        .all(|(p, d)| *p == P0 || !matches!(d, Decision::Priority { .. })));
    t.resolve_all();
    assert!(t.in_exile("Unending Whisper"));
}

#[test]
fn harmonize_casts_from_the_graveyard_tapping_up_to_one_creature_and_exiles_the_spell() {
    cr!("702.180a", "702.180b", "601.2f");
    ruling!(
        "Wild Ride",
        "“Harmonize [cost]” means “You may cast this card from your graveyard by paying [cost] and tapping up to one untapped creature you control rather than paying the spell’s mana cost,”"
    );
    supported("Wild Ride");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 3);
    let ride = t.graveyard(P0, "Wild Ride");
    // Tapping two creatures isn't allowed: the choice is of at most one creature.
    let asked = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, ride).method(HARMONIZE).target(elves).go();
    let maxes: Vec<u32> = asked_since(&t, asked)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { prompt, max, .. } if prompt.contains("harmonize") => {
                Some(*max)
            }
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![1]);
    // {4}{R} reduced by the Bears' power 2: {2}{R}, all three Mountains.
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(elves).tapped);
    assert_eq!(tapped_lands(&t, P0), 3);
    // The mana value is still that of the mana cost {R}.
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert_eq!(t.pt(elves), (4, 1));
    // It's exiled instead of going to the graveyard.
    assert!(t.in_exile("Wild Ride"));
    assert!(!t.in_graveyard(P0, "Wild Ride"));
}

#[test]
fn a_harmonize_card_that_was_never_cast_can_be_cast_from_the_graveyard() {
    cr!("702.180a");
    ruling!(
        "Unending Whisper",
        "You can cast a spell using harmonize even if it was somehow put into your graveyard without having been cast."
    );
    supported("Unending Whisper");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.library_top(P0, "Unending Whisper");
    // Milled, not cast.
    t.g.mill(P0, 1);
    t.g.flush_events();
    let w = t.g.current(w);
    assert_eq!(t.zone(w), Zone::Graveyard(P0));
    assert!(t.g.history.spells_cast.is_empty());
    let hand = t.hand_size(P0);
    t.cast(P0, w).method(HARMONIZE).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Unending Whisper"));
}
