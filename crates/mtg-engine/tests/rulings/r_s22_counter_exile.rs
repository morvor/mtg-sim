//! Rulings batch S22 — Thranduil's Decree ("Counter target spell. If a permanent spell is
//! countered this way, exile it instead of putting it into its owner's graveyard. You may
//! cast that card without paying its mana cost for as long as it remains exiled."): the
//! card is cast without paying its mana cost, an alternative cost, so no other
//! alternative cost — casting it face down with morph included — can be used; additional
//! costs can (or must) be paid (CR 118.9a, 702.37c).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 casts `name` in their main phase (with mana for it); P0 counters it with
/// Thranduil's Decree. Returns the card (followed into exile), with P0 back in their
/// main phase.
fn decree(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Thranduil's Decree");
    t.set_step(P1, Step::PrecombatMain);
    let card = t.hand(P1, name);
    // A card for P1 to discard if the spell's cost needs one (Mardu Outrider).
    t.hand(P1, "Island");
    give_mana_for(t, P1, name);
    let spell = t.cast(P1, card).go();
    add_mana(t, P0, ManaType::U, 6);
    let d = t.hand(P0, "Thranduil's Decree");
    t.cast(P0, d).target(Entity::Object(spell)).go();
    t.resolve();
    t.clear_answers();
    let card = t.g.current(spell);
    t.set_step(P0, Step::PrecombatMain);
    t.g.player_mut(P0).mana_pool = Default::default();
    t.g.player_mut(P1).mana_pool = Default::default();
    card
}

#[test]
fn thranduils_decree_the_card_cant_be_cast_face_down_but_additional_costs_are_paid() {
    cr!("118.9a", "118.8a", "702.37c", "601.2b", "400.7j");
    ruling!(
        "Thranduil's Decree",
        "Because you're already casting the card using an alternative cost (by casting it without paying its mana cost), you can't pay any other alternative costs for the card, including casting it face down using the morph ability. You can pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    // Spined Basher ({2}{B}, morph {2}{B}): only cast face up, without paying its mana
    // cost — not face down for {3}.
    let mut t = TestGame::new(2);
    let basher = decree(&mut t, "Spined Basher");
    assert_eq!(t.zone(basher), Zone::Exile);
    t.lands(P0, "Swamp", 3);
    assert_eq!(legal_cast_methods(&mut t, P0, basher), vec![CastMethod::Free]);
    t.cast(P0, basher).method(CastMethod::Free).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Spined Basher");
    assert_eq!(b.len(), 1);
    assert!(!t.obj_now(b[0]).face_down);
    assert_eq!(t.obj_now(b[0]).controller, P0);
    assert_eq!(tapped_lands(&t, P0), 0);

    // Skyclave Relic's kicker {3} can be paid: two token copies.
    let mut t = TestGame::new(2);
    let relic = decree(&mut t, "Skyclave Relic");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, relic).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.named_on_battlefield("Skyclave Relic").len(), 3);

    // Mardu Outrider's discard must be paid ("As an additional cost to cast this spell,
    // discard a card.").
    let mut t = TestGame::new(2);
    let outrider = decree(&mut t, "Mardu Outrider");
    assert!(t
        .cast(P0, outrider)
        .method(CastMethod::Free)
        .try_go()
        .is_err());
    t.hand(P0, "Forest");
    t.cast(P0, outrider).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.named_on_battlefield("Mardu Outrider").len(), 1);
}

#[test]
fn thranduils_decree_only_a_permanent_spell_is_exiled() {
    cr!("701.6a", "614.15", "110.4b");
    supported("Thranduil's Decree");
    // Lightning Bolt isn't a permanent spell: it's countered into P1's graveyard.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    add_mana(&mut t, P0, ManaType::U, 6);
    let d = t.hand(P0, "Thranduil's Decree");
    t.cast(P0, d).target(Entity::Object(spell)).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    let card = t.g.current(spell);
    assert!(legal_cast_methods(&mut t, P0, card).is_empty());
}
