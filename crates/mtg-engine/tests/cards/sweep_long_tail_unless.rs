//! "Unless [a player] pays" with scaled costs (Countervailing Winds, Rethink, Extravagant
//! Spirit), and "becomes the target" triggers that refer to the targeting spell or
//! ability: Frost Titan, Shimmering Glasskite, Bonecrusher Giant.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// P1 casts Lightning Bolt at P0 with `lands` Mountains; P0 answers with Countervailing
/// Winds with `yard` cards in their graveyard, and `later` more cards are put there while
/// Winds is on the stack. Returns P0's life afterwards.
fn winds(lands: usize, yard: usize, later: usize, pay: bool) -> i32 {
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", lands);
    for _ in 0..yard {
        t.graveyard(P0, "Grizzly Bears");
    }
    let bolt = t.hand(P1, "Lightning Bolt");
    let winds = t.hand(P0, "Countervailing Winds");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.cast(P0, winds).target(spell).go();
    for _ in 0..later {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer_yes(P1, pay);
    t.resolve_all();
    t.life(P0)
}

#[test]
fn countervailing_winds_costs_one_per_card_in_graveyard() {
    cr!("118.12a");
    ruling!(
        "Countervailing Winds",
        "Count the number of cards in your graveyard as Countervailing Winds resolves"
    );
    assert_supported("Countervailing Winds");
    // Two cards (Winds itself, still on the stack, doesn't count): P1 has two untapped
    // Mountains left and pays.
    assert_eq!(winds(3, 2, 0, true), 17);
    // Three cards: P1 can't pay {3} with two Mountains, so Bolt is countered.
    assert_eq!(winds(3, 3, 0, true), 20);
    // Counted as Winds resolves: two when it was cast, three by the time it resolves.
    assert_eq!(winds(3, 2, 1, true), 20);
    // P1 may decline to pay.
    assert_eq!(winds(3, 2, 0, false), 20);
    // An empty graveyard: {0}, which P1 can pay.
    assert_eq!(winds(1, 0, 0, true), 17);
}

#[test]
fn rethink_costs_the_spells_mana_value() {
    cr!("118.12a");
    ruling!("Rethink", "The spell’s controller gets the option to pay when this spell resolves.");
    assert_supported("Rethink");
    for (lands, countered) in [(4, false), (3, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P0, "Island", 3);
        t.lands(P1, "Forest", lands);
        let bears = t.hand(P1, "Grizzly Bears");
        let rethink = t.hand(P0, "Rethink");
        let spell = t.cast(P1, bears).go();
        t.cast(P0, rethink).target(spell).go();
        t.answer_yes(P1, true);
        t.resolve_all();
        // Grizzly Bears has mana value 2: P1 needs two more untapped lands.
        assert_eq!(
            t.in_graveyard(P1, "Grizzly Bears"),
            countered,
            "{}",
            t.dump_log()
        );
    }
}

#[test]
fn extravagant_spirit_costs_one_per_card_in_hand() {
    cr!("118.12a");
    assert_supported("Extravagant Spirit");
    for (cards, lands, survives) in [(2, 2, true), (3, 2, false)] {
        let mut t = TestGame::new(2);
        let spirit = t.battlefield(P0, "Extravagant Spirit");
        t.lands(P0, "Island", lands);
        t.set_step(P1, Step::End);
        // Cards in hand at P0's upkeep (the draw step comes after).
        for _ in 0..cards {
            t.hand(P0, "Grizzly Bears");
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.on_battlefield(spirit), survives, "{}", t.dump_log());
    }
}

#[test]
fn frost_titan_taxes_opponents_spells_that_target_it() {
    cr!("603.2", "118.12a");
    ruling!(
        "Frost Titan",
        "affects each spell (including Aura spells), activated ability, and triggered ability"
    );
    assert_supported("Frost Titan");
    for (lands, countered) in [(1, true), (3, false)] {
        let mut t = TestGame::new(2);
        let titan = t.battlefield(P0, "Frost Titan");
        t.lands(P1, "Mountain", lands);
        let shock = t.hand(P1, "Shock");
        t.answer_yes(P1, true);
        t.cast(P1, shock).target(titan).go();
        t.resolve_all();
        assert_eq!(t.obj_now(titan).damage == 0, countered, "{}", t.dump_log());
        assert!(t.in_graveyard(P1, "Shock"));
    }
}

#[test]
fn frost_titan_ignores_its_controllers_spells() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Frost Titan");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(titan).go();
    t.resolve_all();
    assert_eq!(t.obj_now(titan).damage, 2);
}

#[test]
fn shimmering_glasskite_counters_only_the_first_each_turn() {
    cr!("603.2", "701.6a");
    assert_supported("Shimmering Glasskite");
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Shimmering Glasskite");
    t.lands(P1, "Mountain", 2);
    let s1 = t.hand(P1, "Shock");
    let s2 = t.hand(P1, "Shock");
    t.cast(P1, s1).target(kite).go();
    t.resolve_all();
    assert_eq!(t.obj_now(kite).damage, 0, "{}", t.dump_log());
    t.cast(P1, s2).target(kite).go();
    t.resolve_all();
    assert_eq!(t.obj_now(kite).damage, 2, "{}", t.dump_log());
}

#[test]
fn bonecrusher_giant_damages_the_spells_controller() {
    cr!("603.2");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "It resolves even if that spell is countered."
    );
    assert_supported("Bonecrusher Giant // Stomp");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Bonecrusher Giant // Stomp");
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 2);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(giant).go();
    // The trigger goes on the stack above Shock; Shock is countered before it resolves.
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let negate = t.hand(P0, "Negate");
    t.cast(P0, negate).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}

#[test]
fn frost_titan_taxes_activated_abilities_too() {
    cr!("603.2", "118.12a", "701.6a");
    ruling!(
        "Frost Titan",
        "affects each spell (including Aura spells), activated ability, and triggered ability"
    );
    for (lands, countered) in [(0, true), (2, false)] {
        let mut t = TestGame::new(2);
        let titan = t.battlefield(P0, "Frost Titan");
        let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
        t.lands(P1, "Mountain", lands);
        t.answer_yes(P1, true);
        t.activate(P1, sorcerer, 0, &[Entity::Object(titan)]).unwrap();
        t.resolve_all();
        assert_eq!(t.obj_now(titan).damage == 0, countered, "{}", t.dump_log());
        // The cost was paid either way.
        assert!(t.obj_now(sorcerer).tapped);
    }
}

#[test]
fn kira_counters_the_first_spell_or_ability_targeting_each_creature_each_turn() {
    cr!("603.2", "613.1f");
    assert_supported("Kira, Great Glass-Spinner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kira, Great Glass-Spinner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.lands(P1, "Mountain", 2);
    // The first targeting (an ability) is countered...
    t.activate(P1, sorcerer, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0, "{}", t.dump_log());
    // ...the second one this turn isn't.
    let s1 = t.hand(P1, "Shock");
    t.cast(P1, s1).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn retromancer_damages_the_controller_of_a_targeting_ability() {
    cr!("603.2");
    assert_supported("Retromancer");
    let mut t = TestGame::new(2);
    let retro = t.battlefield(P0, "Retromancer");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[Entity::Object(retro)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(retro).damage, 1);
}

#[test]
fn lava_runner_makes_the_targeting_player_sacrifice_a_land() {
    cr!("603.2", "701.21a");
    assert_supported("Lava Runner");
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Lava Runner");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Forest", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(runner).go();
    t.resolve_all();
    // P1 (who controlled Shock) sacrificed one of their two lands; P0 lost none.
    let lands_of = |p: PlayerId| {
        ["Mountain", "Forest"]
            .iter()
            .flat_map(|n| t.named_on_battlefield(n))
            .filter(|&o| t.obj_now(o).controller == p)
            .count()
    };
    assert_eq!(lands_of(P1), 1, "{}", t.dump_log());
    assert_eq!(lands_of(P0), 2);
    assert!(t.in_graveyard(P0, "Lava Runner"));
}
