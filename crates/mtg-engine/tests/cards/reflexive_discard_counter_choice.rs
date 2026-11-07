//! Two small oracle patterns:
//!
//! * "When you discard a [quality] card this way, [effect]" — a reflexive triggered
//!   ability that triggers only if the preceding discard discarded such a card (CR 603.12;
//!   `src/oracle/patterns/r603_discard_this_way.rs`): Evie Frye, Fiery Encore, Teo.
//! * "Put your choice of a [kind], [kind], or [kind] counter on [object]" (CR 122.1;
//!   `src/oracle/patterns/counters_your_choice.rs`): Owen Grady, Assaultron Dominator.

use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn discard_this_way_and_counter_choice_cards_compile() {
    assert_compiles(&[
        "Evie Frye",
        "Fiery Encore",
        "Teo, Spirited Glider",
        "Owen Grady, Raptor Trainer",
        "Assaultron Dominator",
        "T-45 Power Armor",
        "The Night of the Doctor",
    ]);
}

#[test]
fn evie_frye_makes_a_creature_unblockable_only_if_she_discarded_a_creature_card() {
    cr!("603.12");
    // "{1}, {T}: Draw a card, then discard a card. When you discard a creature card this
    // way, target creature you control can't be blocked this turn."
    let setup = |t: &mut TestGame, in_hand: &str| -> (ObjectId, ObjectId, ObjectId) {
        let evie = t.battlefield(P0, "Evie Frye");
        let giant = t.battlefield(P0, "Hill Giant");
        let blocker = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", 1);
        t.library_top(P0, "Island");
        let card = t.hand(P0, in_hand);
        t.answer_choose(P0, &[Entity::Object(card)]);
        (evie, giant, blocker)
    };
    let can_block = |t: &mut TestGame, giant: ObjectId, blocker: ObjectId| {
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(giant, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::DeclareAttackers);
        let opts = block_options(&t.g, &[P1]);
        block_declaration_legal(&t.g, &opts, &[(blocker, giant)])
    };
    // A creature card discarded: the Giant can't be blocked this turn.
    let mut t = TestGame::new(2);
    let (evie, giant, blocker) = setup(&mut t, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.activate(P0, evie, 0, &[]).expect("Evie Frye");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Island"));
    assert!(!can_block(&mut t, giant, blocker));
    // A land card discarded: no reflexive trigger.
    let mut t = TestGame::new(2);
    let (evie, giant, blocker) = setup(&mut t, "Forest");
    t.activate(P0, evie, 0, &[]).expect("Evie Frye");
    t.resolve();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.stack_len(), 0);
    assert!(can_block(&mut t, giant, blocker));
}

#[test]
fn fiery_encore_deals_damage_equal_to_the_discarded_cards_mana_value() {
    cr!("603.12");
    // "Discard a card, then draw a card. When you discard a nonland card this way, Fiery
    // Encore deals damage equal to that card's mana value to target creature or
    // planeswalker."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let dragon = t.hand(P0, "Shivan Dragon");
    t.library_top(P0, "Island");
    t.lands(P0, "Mountain", 5);
    let encore = t.hand(P0, "Fiery Encore");
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.cast(P0, encore).go();
    t.resolve_all();
    // Shivan Dragon's mana value is 6: the 6/4 Craw Wurm dies.
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    // A land discarded instead: nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let land = t.hand(P0, "Forest");
    t.lands(P0, "Mountain", 5);
    let encore = t.hand(P0, "Fiery Encore");
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.cast(P0, encore).go();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn owen_grady_puts_the_chosen_kind_of_counter_on_a_dinosaur() {
    cr!("122.1b", "602.5d");
    // "{T}: Put your choice of a reach, menace, trample, or haste counter on target
    // Dinosaur. Activate only as a sorcery."
    let mut t = TestGame::new(2);
    let owen = t.battlefield(P0, "Owen Grady, Raptor Trainer");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    // The options are reach, menace, trample, haste: menace.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, owen, 0, &[Entity::Object(dino)])
        .expect("Owen Grady");
    t.resolve_all();
    assert_eq!(t.counters(dino, "menace"), 1);
    assert_eq!(t.counters(dino, "reach"), 0);
    assert!(t.obj_now(dino).chars.has_keyword(KeywordKind::Menace));
    // Only as a sorcery.
    let mut t = TestGame::new(2);
    let owen = t.battlefield(P0, "Owen Grady, Raptor Trainer");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P0, owen, 0, &[Entity::Object(dino)]).is_err());
}
