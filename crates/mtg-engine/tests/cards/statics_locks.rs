//! Statics that lock players or creatures: opponents can't cast or activate, player
//! hexproof, can't gain life, no maximum hand size, one spell each turn, "can't attack
//! or block, and its activated abilities can't be activated", evasion by comparison.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn during_your_turn_opponents_cant_cast_or_activate() {
    cr!("101.2", "602.5");
    ruling!(
        "Grand Abolisher",
        "doesn't affect triggered abilities or static abilities"
    );
    compiles("Grand Abolisher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grand Abolisher");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    let land = t.battlefield(P1, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    t.clear_answers();
    assert!(t
        .activate(P1, pinger, 0, &[Entity::Player(P0)])
        .is_err());
    t.clear_answers();
    // Lands aren't artifacts, creatures, or enchantments.
    assert!(t.activate(P1, land, 0, &[]).is_ok());
    // Triggered abilities of P1's creatures still trigger and resolve.
    t.battlefield(P1, "Soul Warden");
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.move_object(
        bears,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    // On P1's own turn, no restriction.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
}

#[test]
fn you_have_hexproof() {
    cr!("702.11c", "613.10");
    compiles("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Sanctity");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    // The only other target is P1 (or nothing else): P0 can't be chosen.
    let _ = t.cast(P1, bolt).target(P0).try_go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn your_opponents_cant_gain_life() {
    cr!("101.2");
    compiles("Erebos, God of the Dead");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erebos, God of the Dead");
    t.g.gain_life(P1, 5);
    t.g.gain_life(P0, 5);
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn you_have_no_maximum_hand_size() {
    cr!("402.2", "613.10");
    ruling!(
        "Spellbook",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    compiles("Spellbook");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spellbook");
    t.settle();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    assert_eq!(t.g.player(P1).max_hand_size, Some(7));
    // Null Profusion ("your maximum hand size is two") first, then Spellbook: no
    // maximum; the other way around: two.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Null Profusion");
    t.battlefield(P0, "Spellbook");
    t.settle();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spellbook");
    t.battlefield(P0, "Null Profusion");
    t.settle();
    assert_eq!(t.g.player(P0).max_hand_size, Some(2));
}

#[test]
fn each_player_cant_cast_more_than_one_spell_each_turn() {
    cr!("101.2", "601.2");
    ruling!(
        "Rule of Law",
        "If you cast a spell that was countered, you can’t cast another spell during the same turn."
    );
    compiles("Rule of Law");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rule of Law");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Island", 2);
    let a = t.hand(P0, "Lightning Bolt");
    let b = t.hand(P0, "Shock");
    let counter = t.hand(P1, "Counterspell");
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.cast(P0, a).target(P1).go();
    // P1's first spell this turn: allowed.
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    // The countered Bolt was still cast this turn.
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, b).target(P1).try_go().is_err());
    // Next turn, P0 can cast a spell again.
    t.clear_answers();
    t.advance_to(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, b).target(P1).try_go().is_ok());
}

#[test]
fn arrested_creature_cant_attack_block_or_activate() {
    cr!("508.1c", "509.1b", "602.5");
    compiles("Arrest");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let aura = t.battlefield(P0, "Arrest");
    t.attach(aura, Entity::Object(elves));
    t.settle();
    t.set_step(P1, Step::PrecombatMain);
    // Mana abilities too.
    assert!(t.activate(P1, elves, 0, &[]).is_err());
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.can_attack(elves));
    assert!(!t.can_block_at_all(elves));
}

#[test]
fn activated_abilities_of_creatures_your_opponents_control() {
    cr!("602.5", "605.1a");
    ruling!(
        "Linvala, Keeper of Silence",
        "No abilities of creatures your opponents control can be activated, including mana abilities."
    );
    compiles("Linvala, Keeper of Silence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Linvala, Keeper of Silence");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P1, theirs, 0, &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, mine, 0, &[]).is_ok());
}

#[test]
fn cant_be_blocked_except_by_creatures_with_flying() {
    cr!("509.1b");
    ruling!(
        "Treetop Rangers",
        "Creatures with reach (such as Giant Spider) don't actually have flying"
    );
    compiles("Treetop Rangers");
    for (blocker, damage) in [("Giant Spider", 2), ("Suntail Hawk", 0)] {
        let mut t = TestGame::new(2);
        let rangers = t.battlefield(P0, "Treetop Rangers");
        let b = t.battlefield(P1, blocker);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(rangers, Entity::Player(P1))], &[(b, rangers)]);
        assert_eq!(t.life(P1), 20 - damage, "{blocker}");
    }
}

#[test]
fn creatures_with_less_power_cant_block_creatures_you_control() {
    cr!("509.1b");
    ruling!(
        "Champion of Lambholt",
        "Champion of Lambholt's first ability applies even if it isn't attacking."
    );
    compiles("Champion of Lambholt");
    for (blocker, damage) in [("Llanowar Elves", 2), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        let champ = t.battlefield(P0, "Champion of Lambholt");
        t.g.add_counters(Entity::Object(champ), "+1/+1", 1, None);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, blocker);
        t.set_step(P0, Step::BeginningOfCombat);
        // Champion is 2/2: Elves (power 1) can't block the attacking Bears.
        t.attack(&[(bears, Entity::Player(P1))], &[(b, bears)]);
        assert_eq!(t.life(P1), 20 - damage, "{blocker}");
    }
}
