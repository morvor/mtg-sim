//! CR 702.183 Tiered (`oracle/patterns/k702_179_195.rs`; the modes' additional costs are
//! paid as for spree, CR 601.2b, 601.2f).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn tiered_cards_compile() {
    assert_supported(&[
        "Thunder Magic",
        "Ice Magic",
        "Fire Magic",
        "Tifa's Limit Break",
        "Restoration Magic",
    ]);
    let c = mtg_engine::card::card("Thunder Magic");
    assert!(c.front().chars.has_keyword(KeywordKind::Tiered));
}

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn choose_one_mode_and_pay_its_additional_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "You must choose exactly one of the listed modes and pay its associated additional cost"
    );
    // "Thunder — {0} — 2 damage; Thundara — {3} — 4 damage; Thundaga — {5}{R} — 8 damage."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    t.cast(P0, c).modes(&[0]).target(wurm).go();
    // {R} plus {0}.
    assert_eq!(untapped_lands(&t), 7);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    let c = t.hand(P0, "Thunder Magic");
    t.cast(P0, c).modes(&[2]).target(wurm).go();
    // {R} plus {5}{R}.
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn only_one_mode_and_only_if_its_cost_can_be_paid() {
    cr!("702.183a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    // Two modes can't be chosen.
    assert!(t
        .cast(P0, c)
        .modes(&[0, 1])
        .targets(&[Entity::Object(wurm), Entity::Object(wurm)])
        .try_go()
        .map_or(true, |s| t.g.obj(s).stack.as_ref().unwrap().chosen.len() == 1));
    t.clear_answers();
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    // Thundara costs {R} + {3}: four mana, with three lands it can't be cast.
    assert!(t.cast(P0, c).modes(&[1]).target(wurm).try_go().is_err());
    assert!(t.in_hand(P0, "Thunder Magic"));
    assert_eq!(untapped_lands(&t), 3);
}

#[test]
fn the_mana_value_is_that_of_the_mana_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "The mana value of a spell with tiered is determined only by its mana cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    let spell = t.cast(P0, c).modes(&[2]).target(wurm).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
}

#[test]
fn cast_without_paying_its_mana_cost_still_pays_the_mode_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "you must still choose exactly one mode and pay the associated additional cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![1]),
    );
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(c)],
    );
    // Only Thundara's {3} was paid.
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    // 4 damage: the 6/4 Wurm dies (Thunder's 2 wouldn't have killed it).
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P0, "Thunder Magic"));
}

#[test]
fn a_mode_can_be_chosen_only_if_its_targets_are_available() {
    cr!("702.183a");
    ruling!(
        "Ice Magic",
        "If a mode requires a target, you can select that mode only if there's a legal target available."
    );
    // Every mode of Ice Magic targets a creature: with no creatures, no mode can be
    // chosen and it can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 8);
    let c = t.hand(P0, "Ice Magic");
    assert!(t.cast(P0, c).modes(&[0]).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Ice Magic"));
    // Fire Magic's modes don't target: it can be cast with no creatures around, and Fira
    // deals 2 damage to each creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Fire Magic");
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fire Magic"));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let c = t.hand(P0, "Fire Magic");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
}
