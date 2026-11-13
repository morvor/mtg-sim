//! Rulings batch S25 — a copy of a spell copies the decisions made for the original,
//! including which additional or alternative costs were paid, but its controller pays no
//! costs for it and no mana was spent on it (CR 707.10, 601.2b, 601.2f).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn optional_costs_asked_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count()
}

/// After `setup`, P0 casts Burst Lightning ("deals 2 damage to any target. If this spell
/// was kicked, it deals 4 damage instead.") kicked at P1; `start` puts on the stack what
/// copies it (nothing when a triggered ability already does), which resolves. The copy
/// is kicked too — its controller isn't asked about costs — and deals 4 damage.
fn kicked_burst_lightning_copy(
    setup: impl FnOnce(&mut TestGame),
    start: impl FnOnce(&mut TestGame, ObjectId),
) {
    supported("Burst Lightning");
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    let burst = t.cast(P0, card).kicked(true).target(Entity::Player(P1)).go();
    start(&mut t, burst);
    let from = t.asked().len();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    assert_eq!(optional_costs_asked_since(&t, from), 0);
}

#[test]
fn a_cast_trigger_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Double Vision",
        "You can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Double Vision");
    kicked_burst_lightning_copy(
        |t| {
            t.battlefield(P0, "Double Vision");
        },
        |_, _| {},
    );
}

#[test]
fn flare_s_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Flare of Duplication",
        "The controller of a copy can't choose to pay any alternative or additional costs for the copy. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    supported("Flare of Duplication");
    kicked_burst_lightning_copy(
        |_| {},
        |t, burst| {
            cast_new(t, P0, "Flare of Duplication", &[Entity::Object(burst)]);
        },
    );
}

#[test]
fn mica_s_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Mica, Reader of Ruins",
        "You can't choose to pay any additional costs for a copied spell. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Mica, Reader of Ruins");
    // "Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If
    // you do, copy that spell and you may choose new targets for the copy."
    kicked_burst_lightning_copy(
        |t| {
            t.battlefield(P0, "Mica, Reader of Ruins");
            let thopter = t.battlefield(P0, "Ornithopter");
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[Entity::Object(thopter)]);
        },
        |_, _| {},
    );
}

#[test]
fn a_doublecast_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Doublecast",
        "You can’t choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Doublecast");
    kicked_burst_lightning_copy(
        |t| {
            cast_new(t, P0, "Doublecast", &[]);
            t.resolve_all();
        },
        |_, _| {},
    );
}

#[test]
fn a_fury_storm_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Fury Storm",
        "The controller of a copy can’t choose to pay any alternative or additional costs for the copy. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    supported("Fury Storm");
    kicked_burst_lightning_copy(
        |_| {},
        |t, burst| {
            cast_new(t, P0, "Fury Storm", &[Entity::Object(burst)]);
            // Its cast trigger (a copy per commander cast: none) resolves first.
            t.resolve();
        },
    );
}

#[test]
fn reiterate_s_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Reiterate",
        "You can't choose to pay any alternative or additional costs for the copy. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    supported("Reiterate");
    kicked_burst_lightning_copy(
        |_| {},
        |t, burst| {
            // Reiterate's own buyback isn't paid.
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
            cast_new(t, P0, "Reiterate", &[Entity::Object(burst)]);
        },
    );
}

#[test]
fn a_copy_of_a_kicked_creature_spell_is_kicked() {
    cr!("707.10", "707.10f", "702.33d");
    ruling!(
        "Archmage of Echoes",
        "The copy remembers any decisions that were made for the original spell as it was cast, including values chosen for X in its mana cost and whether any alternative or additional costs were chosen."
    );
    supported("Archmage of Echoes");
    supported("Faerie Squadron");
    // "Whenever you cast a Faerie or Wizard permanent spell, copy it." Faerie Squadron:
    // "Kicker {3}{U}. If this creature was kicked, it enters with two +1/+1 counters on it
    // and with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archmage of Echoes");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Faerie Squadron");
    let from = t.asked().len();
    t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    let squadrons = t.named_on_battlefield("Faerie Squadron");
    assert_eq!(squadrons.len(), 2);
    for s in squadrons {
        assert_eq!(t.counters(s, counters::PLUS1), 2);
        assert!(t
            .obj_now(s)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    }
    assert_eq!(optional_costs_asked_since(&t, from), 1);
}

#[test]
fn a_copy_had_no_colored_mana_spent_on_it() {
    cr!("707.10", "601.2h");
    ruling!(
        "Repel Intruders",
        "If this spell is copied, the copy will not have had any colors of mana paid for it, no matter what colors were spent on the original spell."
    );
    supported("Repel Intruders");
    // "Create two 1/1 white Kithkin Soldier creature tokens if {W} was spent to cast this
    // spell. Counter up to one target creature spell if {U} was spent to cast this spell."
    let mut t = TestGame::new(2);
    let intruders = cast_new(&mut t, P0, "Repel Intruders", &[]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(intruders)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    // Only the original ({W} was spent on it) made Kithkin.
    assert_eq!(creature_tokens(&t, P0), 2);
}

#[test]
fn a_copied_mythos_gets_no_bonus() {
    cr!("707.10", "601.2h");
    ruling!(
        "Mythos of Brokkos",
        "If an effect copies the Mythos spell, no mana was spent to cast the copy, so the copy won’t receive the bonus."
    );
    supported("Mythos of Brokkos");
    // "If {U}{B} was spent to cast this spell, search your library for a card, put that
    // card into your graveyard, then shuffle. Return up to two permanent cards from your
    // graveyard to your hand."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let card = t.hand(P0, "Mythos of Brokkos");
    let mythos = t.cast(P0, card).go();
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(mythos)]);
    let library = t.library_size(P0);
    // The copy resolves first: no search.
    t.resolve();
    t.resolve();
    assert_eq!(t.library_size(P0), library);
    // The original searches.
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 1);
}

/// Resolves the ability on top of the stack after `copier` copies it (P0 keeps targets).
fn copy_top_ability_with(t: &mut TestGame, copier: ObjectId, needle: &str) {
    let top = *t.g.stack.last().unwrap();
    t.answer_targets(P0, &[Entity::Object(top)]);
    activate_containing(t, P0, copier, needle).unwrap();
    keep_copy_targets(t, P0);
    t.resolve();
}

#[test]
fn a_copied_ability_uses_the_sacrificed_creature_of_the_original() {
    cr!("707.10", "602.2b");
    ruling!(
        "Adric, Mathematical Genius",
        "You can't choose to pay any activation costs for the copy. However, effects based on those costs that were paid for the original ability are copied as though those same costs were paid for the copy."
    );
    supported("Adric, Mathematical Genius");
    supported("Bloodshot Cyclops");
    // Adric: "{2}{U}, {T}: Copy target activated or triggered ability you control."
    // Bloodshot Cyclops: "{T}, Sacrifice a creature: This creature deals damage equal to
    // the sacrificed creature's power to any target."
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let cyclops = t.battlefield(P0, "Bloodshot Cyclops");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, cyclops, "Sacrifice").unwrap();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    copy_top_ability_with(&mut t, adric, "Copy target");
    t.resolve_all();
    // Both deal 3 (the Hill Giant's power); only one creature was sacrificed.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_copied_ability_uses_the_cost_choices_of_the_original() {
    cr!("707.10", "602.2b");
    ruling!(
        "Abstruse Archaic",
        "If the activated ability's cost contains a choice, such as a creature to sacrifice or a number of counters to remove, the copy uses that same information. You can't pay the cost again, even if you want to."
    );
    supported("Abstruse Archaic");
    supported("Diamond Valley");
    // Abstruse Archaic: "{1}, {T}: Copy target activated or triggered ability you control
    // from a colorless source." Diamond Valley (a colorless land): "{T}, Sacrifice a
    // creature: You gain life equal to the sacrificed creature's toughness."
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let valley = t.battlefield(P0, "Diamond Valley");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, valley, "Sacrifice").unwrap();
    copy_top_ability_with(&mut t, archaic, "Copy target");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
}

#[test]
fn the_peregrine_dynamo_copy_uses_the_cost_choices_of_the_original() {
    cr!("707.10", "602.2b");
    ruling!(
        "The Peregrine Dynamo",
        "If the activated ability’s cost contains a choice, such as a creature to sacrifice or a number of counters to remove, the copy uses that same information. You can’t pay the cost again, even if you want to."
    );
    supported("The Peregrine Dynamo");
    supported("Miren, the Moaning Well");
    // The Peregrine Dynamo: "{1}, {T}: Copy target activated or triggered ability you
    // control from another legendary source that's not a commander." Miren, the Moaning
    // Well (a legendary land): "{3}, {T}, Sacrifice a creature: You gain life equal to the
    // sacrificed creature's toughness."
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let miren = t.battlefield(P0, "Miren, the Moaning Well");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, miren, "Sacrifice").unwrap();
    copy_top_ability_with(&mut t, dynamo, "Copy target");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
