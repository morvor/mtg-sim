//! Statics restricting targets, casting and activation, and maximum hand size.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
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

// ---------------------------------------------------------------------------
// "can't be the target of"
// ---------------------------------------------------------------------------

#[test]
fn cant_be_the_target_of_spells_or_abilities_your_opponents_control() {
    cr!("115.4", "613.11");
    ruling!(
        "Canopy Cover",
        "your opponents won’t be able to target their own creature"
    );
    compiles("Canopy Cover");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Canopy Cover");
    t.attach(aura, Entity::Object(theirs));
    t.settle();
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let bolt0 = t.hand(P0, "Lightning Bolt");
    let bolt1 = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    // Its controller is an opponent of the Aura's controller: it can't target it (the
    // requested target is refused and another is chosen).
    let _ = t.cast(P1, bolt1).target(theirs).try_go();
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
    t.clear_answers();
    t.g.turn.priority = Some(P0);
    t.cast(P0, bolt0).target(theirs).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn cant_be_the_target_of_spells_only() {
    cr!("115.4");
    ruling!(
        "Spectral Shield",
        "The enchanted creature can still be the target of abilities."
    );
    compiles("Spectral Shield");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Spectral Shield");
    t.attach(aura, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (2, 4));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    let _ = t.cast(P1, bolt).target(bears).try_go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    t.clear_answers();
    t.activate(P1, pinger, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn cant_be_the_targets_of_blue_spells_or_abilities_from_blue_sources() {
    cr!("115.4", "113.7");
    compiles("Spellbane Centaur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spellbane Centaur");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Island", 2);
    t.lands(P1, "Mountain", 1);
    let unsummon = t.hand(P1, "Unsummon");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, unsummon).target(bears).try_go().is_err());
    t.clear_answers();
    assert!(t.cast(P1, bolt).target(bears).try_go().is_ok());
}

#[test]
fn cards_in_graveyards_cant_be_the_targets_of_spells_or_abilities() {
    cr!("115.4");
    ruling!(
        "Ground Seal",
        "Only spells and abilities that target cards in graveyards will be affected."
    );
    compiles("Ground Seal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ground Seal");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let raise = t.hand(P0, "Raise Dead");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, raise).try_go().is_err());
}

// ---------------------------------------------------------------------------
// Casting and activation
// ---------------------------------------------------------------------------

#[test]
fn opponents_cant_cast_spells_from_anywhere_other_than_their_hands() {
    cr!("101.2", "601.2");
    ruling!(
        "Drannith Magistrate",
        "Drannith Magistrate's restriction overrules that permission"
    );
    compiles("Drannith Magistrate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drannith Magistrate");
    t.lands(P1, "Mountain", 5);
    let firebolt = t.graveyard(P1, "Firebolt");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    assert!(t
        .cast(P1, firebolt)
        .target(P0)
        .method(flashback)
        .try_go()
        .is_err());
    t.clear_answers();
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
}

#[test]
fn during_combat_players_cant_cast_spells_or_activate_non_mana_abilities() {
    cr!("602.5", "605.1a");
    compiles("Yuriko, Blade of the Mighty");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yuriko, Blade of the Mighty");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P0, Step::DeclareBlockers);
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    t.clear_answers();
    assert!(t.activate(P1, pinger, 0, &[Entity::Player(P0)]).is_err());
    t.clear_answers();
    // Mana abilities are still fine.
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
    t.set_step(P0, Step::PostcombatMain);
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
}

// ---------------------------------------------------------------------------
// Maximum hand size
// ---------------------------------------------------------------------------

#[test]
fn maximum_hand_size_changes_apply_in_timestamp_order() {
    cr!("402.2", "613.10");
    ruling!(
        "Minamo Scrollkeeper",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    compiles("Minamo Scrollkeeper");
    compiles("Gnat Miser");
    compiles("Thought Devourer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Minamo Scrollkeeper");
    t.settle();
    assert_eq!(t.g.player(P0).max_hand_size, Some(8));
    t.battlefield(P0, "Gnat Miser");
    t.settle();
    assert_eq!(t.g.player(P1).max_hand_size, Some(6));
    assert_eq!(t.g.player(P0).max_hand_size, Some(8));
    t.battlefield(P0, "Thought Devourer");
    t.settle();
    assert_eq!(t.g.player(P0).max_hand_size, Some(4));
}
