//! "[Effect] instead if [condition]." and "If [condition], instead [effect]." after an
//! earlier instruction, including as an ability-word paragraph of an instant or sorcery
//! ("Morbid — That creature gets -13/-13 until end of turn instead if a creature died this
//! turn."): the earlier instruction is replaced when the condition holds as the spell
//! resolves (CR 608.2c). Patterns in `src/oracle/patterns/damage_removal_instead.rs` and
//! `src/oracle/patterns/r113_spell_ability_word_followup.rs`.

use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn instead_if_cards_compile() {
    assert_compiles(&[
        "Galvanic Blast",
        "Tragic Slip",
        "Cabal Ritual",
        "Stitch Together",
        "Precognitive Perception",
        "Hunger of the Howlpack",
        "Brimstone Volley",
        "Unholy Heat",
        "Might Beyond Reason",
        "Traverse the Ulvenwald",
        "Join the Dead",
        "Cinder Strike",
    ]);
}

#[test]
fn galvanic_blast_deals_four_damage_with_metalcraft() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let blast = t.hand(P0, "Galvanic Blast");
    t.cast(P0, blast).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let blast = t.hand(P0, "Galvanic Blast");
    t.cast(P0, blast).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn tragic_slip_gives_that_creature_minus_thirteen_with_morbid() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    let slip = t.hand(P0, "Tragic Slip");
    t.cast(P0, slip).target(wurm).go();
    t.resolve();
    assert_eq!(t.pt(wurm), (5, 3));
    // A creature dies; the next one gets -13/-13 instead (not both).
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    let giant = t.battlefield(P1, "Hill Giant");
    let slip = t.hand(P0, "Tragic Slip");
    t.cast(P0, slip).target(giant).go();
    t.g.settle();
    t.g.resolve_top();
    t.g.recompute();
    assert_eq!(t.pt(giant), (-10, -10));
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn cabal_ritual_adds_five_mana_with_threshold() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let ritual = t.hand(P0, "Cabal Ritual");
    t.cast(P0, ritual).go();
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 3);
    t.g.players[0].mana_pool.empty();
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 2);
    let ritual = t.hand(P0, "Cabal Ritual");
    t.cast(P0, ritual).go();
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 5);
}

#[test]
fn stitch_together_returns_the_card_to_the_battlefield_instead_with_threshold() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let stitch = t.hand(P0, "Stitch Together");
    t.cast(P0, stitch).target(giant).go();
    t.resolve();
    assert_eq!(t.zone(giant), Zone::Hand(P0));
    // With seven or more cards in the graveyard, the same target goes to the battlefield.
    let wurm = t.graveyard(P0, "Craw Wurm");
    for _ in 0..6 {
        t.graveyard(P0, "Forest");
    }
    t.lands(P0, "Swamp", 2);
    let stitch = t.hand(P0, "Stitch Together");
    t.cast(P0, stitch).target(wurm).go();
    t.resolve();
    assert_eq!(t.zone(wurm), Zone::Battlefield);
}

#[test]
fn precognitive_perception_scries_first_during_the_main_phase() {
    cr!("608.2c", "207.2c", "701.22a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let hand = t.hand_size(P0);
    let pp = t.hand(P0, "Precognitive Perception");
    t.cast(P0, pp).go();
    t.resolve();
    // Cast in the main phase: scry 3 (a decision), then draw three cards.
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Scry { .. })));
    assert_eq!(t.hand_size(P0), hand + 3);
    // Cast during the opponent's turn: just draw three.
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::Upkeep);
    t.lands(P0, "Island", 5);
    let pp = t.hand(P0, "Precognitive Perception");
    let hand = t.hand_size(P0);
    t.cast(P0, pp).go();
    t.resolve();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Scry { .. })));
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

#[test]
fn brimstone_volley_deals_five_damage_instead_with_morbid() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let volley = t.hand(P0, "Brimstone Volley");
    t.cast(P0, volley).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // A creature died this turn: 5 damage instead of 3 (not both).
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    t.lands(P0, "Mountain", 3);
    let volley = t.hand(P0, "Brimstone Volley");
    t.cast(P0, volley).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn hunger_of_the_howlpack_puts_three_counters_instead_with_morbid() {
    cr!("608.2c", "207.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let hunger = t.hand(P0, "Hunger of the Howlpack");
    t.cast(P0, hunger).target(bears).go();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // A creature died this turn: three counters instead of one (not four).
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.destroy(giant, None);
    t.settle();
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    let hunger = t.hand(P0, "Hunger of the Howlpack");
    t.cast(P0, hunger).target(elves).go();
    t.resolve();
    assert_eq!(t.counters(elves, "+1/+1"), 3);
}

#[test]
fn cinder_strike_itself_deals_the_greater_damage_instead() {
    cr!("608.2c");
    // "As an additional cost to cast this spell, you may blight 1. Cinder Strike deals 2
    // damage to target creature. It deals 4 damage to that creature instead if this
    // spell's additional cost was paid." "It" is Cinder Strike, not the creature: a
    // lifelinking target doesn't gain its controller life by damaging itself.
    for blight in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let hawk = t.battlefield(P1, "Vampire Nighthawk");
        t.lands(P0, "Mountain", 1);
        let strike = t.hand(P0, "Cinder Strike");
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Bool(blight),
        );
        if blight {
            t.answer_choose(P0, &[Entity::Object(bears)]);
        }
        t.cast(P0, strike).target(hawk).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "blighted: {blight}");
        assert_eq!(t.in_graveyard(P1, "Vampire Nighthawk"), blight);
        if !blight {
            assert_eq!(t.g.obj(hawk).damage, 2);
        }
    }
}
