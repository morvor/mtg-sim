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

// ---------------------------------------------------------------------------
// The enchanted creature's controller
// ---------------------------------------------------------------------------

#[test]
fn amounts_of_the_enchanted_creatures_controller() {
    cr!("613.4c", "611.3a");
    ruling!(
        "Death's Approach",
        "The value of X will change as the number of creature cards in the enchanted creature"
    );
    compiles("Death's Approach");
    compiles("Righteous Authority");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = t.battlefield(P0, "Death's Approach");
    t.attach(aura, Entity::Object(giant));
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Island");
    t.graveyard(P0, "Grizzly Bears");
    t.settle();
    // One creature card in its controller's graveyard.
    assert_eq!(t.pt(giant), (2, 2));
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.settle();
    assert!(!t.on_battlefield(giant));
    // "for each card in its controller's hand"
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ra = t.battlefield(P0, "Righteous Authority");
    t.attach(ra, Entity::Object(bears));
    t.hand(P1, "Island");
    t.hand(P1, "Island");
    t.hand(P0, "Island");
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn as_long_as_its_controller_controls_another_creature() {
    cr!("611.3a");
    compiles("Favorable Destiny");
    compiles("Predator's Gambit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fd = t.battlefield(P0, "Favorable Destiny");
    t.attach(fd, Entity::Object(bears));
    let pg = t.battlefield(P0, "Predator's Gambit");
    t.attach(pg, Entity::Object(bears));
    // P0's own creatures don't count: only its controller's.
    t.battlefield(P0, "Hill Giant");
    t.settle();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Shroud));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Intimidate));
    t.battlefield(P1, "Llanowar Elves");
    t.settle();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Shroud));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Intimidate));
}

#[test]
fn enchanted_creatures_controller_cant_cast_creature_spells() {
    cr!("101.2", "601.2");
    ruling!(
        "Brand of Ill Omen",
        "is any spell with the type Creature, even if it has other types"
    );
    compiles("Brand of Ill Omen");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let brand = t.battlefield(P0, "Brand of Ill Omen");
    t.attach(brand, Entity::Object(bears));
    t.settle();
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Mountain", 1);
    let elves = t.hand(P1, "Llanowar Elves");
    let bolt = t.hand(P1, "Lightning Bolt");
    let mine = t.hand(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, elves).try_go().is_err());
    t.clear_answers();
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, mine).try_go().is_ok());
}

#[test]
fn opponents_can_cast_spells_only_at_sorcery_speed() {
    cr!("307.1", "101.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Time Raveler");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    // During P0's turn P1 can't cast the instant.
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    t.clear_answers();
    // In P1's own main phase with an empty stack it can.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, shock).target(P0).try_go().is_ok());
}

#[test]
fn players_cant_draw_cards() {
    cr!("121.1", "101.2");
    ruling!(
        "Maralen of the Mornsong",
        "no player can lose the game due to being instructed to draw a card with an empty library"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maralen of the Mornsong");
    t.library_top(P1, "Island");
    let before = t.hand_size(P1);
    t.g.draw_cards(P1, 2);
    t.settle();
    assert_eq!(t.hand_size(P1), before);
    // Drawing from an empty library doesn't happen either.
    let lib: Vec<_> = t.g.player(P0).library.clone();
    for c in lib {
        t.g.move_object(
            c,
            mtg_engine::object::Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
}
