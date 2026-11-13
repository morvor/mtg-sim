//! Rulings batch P123 — "[creatures] can't block this turn" for a whole group of
//! creatures: such an effect doesn't change characteristics, so the group is checked as
//! blockers are declared (CR 611.2c, 509.1b) — unless the effect grants an ability, which
//! locks in the affected set (Incite Hysteria). Also Void Winnower's mana-value checks
//! (CR 202.3), Mark for Death's block requirement (CR 509.1c) and blocks already declared
//! (CR 506.4).

use crate::r_p123_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

/// `apply` makes P0's "creatures without flying can't block this turn" effect happen.
/// Then P1's Grizzly Bears can't block and its Wind Drake can; the Drake losing flying
/// and a creature without flying entering later can't block either, and (for an effect
/// on "your opponents") neither can a creature P1 gains control of later.
fn without_flying_cant_block(name: &str, apply: impl FnOnce(&mut TestGame)) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    let ours = t.battlefield(P0, "Hill Giant");
    apply(&mut t);
    t.resolve_all();
    assert!(!can_block(&mut t, bears), "{name}");
    assert!(can_block(&mut t, drake), "{name}: it has flying");
    modify_until_eot(
        &mut t,
        drake,
        vec![Modification::RemoveKeyword(KeywordKind::Flying)],
    );
    assert!(!can_block(&mut t, drake), "{name}: it lost flying");
    let late = t.enter(P1, "Craw Wurm");
    assert!(!can_block(&mut t, late), "{name}: it entered later");
    crate::r_p015_common::gain_control(&mut t, P1, ours);
    assert!(!can_block(&mut t, ours), "{name}: it came under P1's control later");
}

#[test]
fn creatures_without_flying_cant_block_is_checked_as_blockers_are_declared() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Magmatic Chasm",
        "Because Magmatic Chasm’s effect doesn’t change the characteristics of any permanents, the set of creatures affected by Magmatic Chasm is constantly updated. Creatures without flying that enter the battlefield later in the turn won’t be able to block."
    );
    ruling!(
        "Destructive Tampering",
        "Because the effect of Destructive Tampering's second mode doesn't change the characteristics of any permanents, the set of creatures affected by it is constantly updated."
    );
    ruling!(
        "Seismic Elemental",
        "No creature without flying will be able to block that turn, including creatures that lose flying after Seismic Elemental’s ability resolves and creatures without flying that enter the battlefield after that ability resolves."
    );
    ruling!(
        "Seismic Stomp",
        "No creature without flying will be able to block that turn, including creatures that lose flying after Seismic Stomp resolves and creatures without flying that enter the battlefield after Seismic Stomp resolves."
    );
    ruling!(
        "Stoneshock Giant",
        "Your opponents can't assign any creature without flying to block that turn, even if that creature had flying or wasn't on the battlefield under one of your opponent's control when Stoneshock Giant's ability resolved."
    );
    without_flying_cant_block("Magmatic Chasm", |t| {
        cast_card(t, P0, "Magmatic Chasm");
    });
    without_flying_cant_block("Seismic Stomp", |t| {
        cast_card(t, P0, "Seismic Stomp");
    });
    without_flying_cant_block("Destructive Tampering", |t| {
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_card(t, P0, "Destructive Tampering");
    });
    without_flying_cant_block("Seismic Elemental", |t| {
        t.enter(P0, "Seismic Elemental");
    });
    without_flying_cant_block("Stoneshock Giant", |t| {
        let giant = t.battlefield(P0, "Stoneshock Giant");
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 6);
        activate_containing(t, P0, giant, "onstrosity").expect("activated");
    });
}

#[test]
fn stoneshock_giant_doesnt_undo_blocks() {
    cr!("506.4", "509.1b");
    ruling!(
        "Stoneshock Giant",
        "If blockers have already been declared when Stoneshock Giant's last ability resolves, those blocks won't change or become undone."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Stoneshock Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 6);
        activate_containing(t, P0, giant, "onstrosity").expect("activated");
    });
}

#[test]
fn hero_of_oxid_ridge_checks_power_as_blockers_are_declared() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Hero of Oxid Ridge",
        "Creatures with power 1 or less can’t block even if they weren’t on the battlefield when the last ability resolved."
    );
    ruling!(
        "Hero of Oxid Ridge",
        "The power of each creature is checked only when blocking creatures are declared. If a creature has power 1 or less at that time, it can’t block, even if its power was greater than 1 when the last ability resolved."
    );
    supported("Hero of Oxid Ridge");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Oxid Ridge");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(hero, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!can_block(&mut t, elves));
    assert!(can_block(&mut t, bears));
    // Power changes after the ability resolved count.
    pump(&mut t, bears, -1, 0);
    assert!(!can_block(&mut t, bears), "power 1 now");
    pump(&mut t, elves, 3, 0);
    assert!(can_block(&mut t, elves), "power 4 now");
    let late = t.enter(P1, "Llanowar Elves");
    assert!(!can_block(&mut t, late), "entered later");
    // The declaration: the Bears can't block, the Giant and the pumped Elves can.
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, hero), (elves, hero)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(blocking(&t, giant) && blocking(&t, elves));
}

#[test]
fn awe_for_the_guilds_monocolored_checked_as_blockers_are_declared() {
    cr!("105.4", "611.2c", "509.1b");
    ruling!(
        "Awe for the Guilds",
        "A monocolored creature is exactly one color. (Colorless creatures aren’t monocolored.)"
    );
    ruling!(
        "Awe for the Guilds",
        "No monocolored creature will be able to block that turn, including monocolored creatures that enter the battlefield after Awe for the Guilds resolves. A creature that’s monocolored when Awe for the Guilds resolves but somehow becomes multicolored before blockers are declared will be able to block."
    );
    supported("Awe for the Guilds");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let wolf = t.battlefield(P1, "Watchwolf");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_resolve_at(&mut t, P0, "Awe for the Guilds", &[]);
    assert!(!can_block(&mut t, bears));
    assert!(can_block(&mut t, thopter), "colorless isn't monocolored");
    assert!(can_block(&mut t, wolf), "multicolored");
    let late = t.enter(P1, "Hill Giant");
    assert!(!can_block(&mut t, late), "entered later");
    modify_until_eot(
        &mut t,
        elves,
        vec![Modification::AddColors(ColorSet::single(Color::Blue))],
    );
    assert!(can_block(&mut t, elves), "multicolored now");
}

#[test]
fn incite_hysteria_locks_in_the_creatures_affected() {
    cr!("611.2c", "105.4");
    ruling!(
        "Incite Hysteria",
        "The targeted creature and the creatures that share a color with it at the time Incite Hysteria resolves gain an ability that says they can’t block this turn. It doesn’t matter if those creatures change colors before blockers are declared."
    );
    supported("Incite Hysteria");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve_at(&mut t, P0, "Incite Hysteria", &[obj(bears)]);
    assert!(!can_block(&mut t, bears) && !can_block(&mut t, elves));
    assert!(can_block(&mut t, giant));
    let red = ColorSet::single(Color::Red);
    let green = ColorSet::single(Color::Green);
    modify_until_eot(&mut t, elves, vec![Modification::SetColors(red)]);
    assert!(!can_block(&mut t, elves), "still has the ability");
    modify_until_eot(&mut t, giant, vec![Modification::SetColors(green)]);
    assert!(can_block(&mut t, giant), "didn't share a color as it resolved");
    let late = t.enter(P1, "Llanowar Elves");
    assert!(can_block(&mut t, late), "entered later");
}

#[test]
fn void_winnower_mana_values() {
    cr!("202.3e", "107.3b", "708.4", "601.3");
    ruling!(
        "Void Winnower",
        "For creatures on the battlefield with {X} in their mana costs, the value for X is 0. Mistcutter Hydra (a creature with mana cost {X}{G}) could block, but Endless One could not."
    );
    ruling!(
        "Void Winnower",
        "For spells with {X} in their mana costs, use the value chosen for X to determine if the spell's mana value is even or not. For example, your opponent could cast Endless One (with mana cost {X}) with X equal to 5, but not with X equal to 6."
    );
    ruling!(
        "Void Winnower",
        "The mana value of a face-down creature spell or a face-down creature is 0. Void Winnower will stop face-down creature spells from being cast and face-down creatures from blocking."
    );
    supported("Void Winnower");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Void Winnower");
    let hydra = t.battlefield(P1, "Mistcutter Hydra");
    let one = t.battlefield(P1, "Endless One");
    assert!(can_block(&mut t, hydra));
    assert!(!can_block(&mut t, one));
    // Casting Endless One with X = 6 (even) or 5 (odd).
    for (x, ok) in [(6, false), (5, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Void Winnower");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Wastes", x as usize);
        let card = t.hand(P1, "Endless One");
        t.g.turn.priority = Some(P1);
        let r = t.cast(P1, card).x(x).try_go();
        assert_eq!(r.is_ok(), ok, "X = {x}");
    }
    // Face down: a face-down creature can't block; a face-down spell can't be cast.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let fd = crate::r_s12_common::morph(&mut t, P1, "Exalted Angel");
    t.battlefield(P0, "Void Winnower");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_block(&mut t, fd));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 3);
    let card = t.hand(P1, "Exalted Angel");
    assert!(!can_cast(
        &mut t,
        P1,
        card,
        CastMethod::FaceDown(KeywordKind::Morph)
    ));
}

/// P0 casts Mark for Death on P1's `target` in its main phase.
fn mark(t: &mut TestGame, target: ObjectId) {
    supported("Mark for Death");
    cast_resolve_at(t, P0, "Mark for Death", &[obj(target)]);
}

#[test]
fn mark_for_death_requirement_when_it_cant_block() {
    cr!("509.1c");
    ruling!(
        "Mark for Death",
        "If the creature can’t block (perhaps because it has become tapped or it doesn’t have flying and all attacking creatures do), then the requirement to block does nothing."
    );
    // It became tapped.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.g.tap(bears);
    mark(&mut t, bears);
    assert!(!t.obj_now(bears).tapped, "Mark for Death untaps it");
    t.g.tap(bears);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(other, goblin)]), "other creatures can't block");
    // All attackers fly.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mark(&mut t, bears);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(drake, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mark_for_death_doesnt_force_paying_costs_to_block() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Mark for Death",
        "If there are any costs required for that creature to block (such as the one imposed by War Cadence), the creature’s controller isn’t required to pay those costs."
    );
    supported("War Cadence");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let cadence = t.battlefield(P0, "War Cadence");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    mark(&mut t, bears);
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, cadence, "pays").expect("activated");
    t.resolve_all();
    // P1 declares no blocks (its default): legal, the Bears don't block.
    to_combat(&mut t, P0);
    to_blockers(&mut t, &[(goblin, Entity::Player(P1))], &[]);
    assert!(!blocking(&t, bears));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
}

#[test]
fn mark_for_death_controller_chooses_which_attacker_to_block() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Mark for Death",
        "That creature’s controller still decides which attacking creature it blocks."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mark(&mut t, bears);
    to_combat(&mut t, P0);
    to_blockers(
        &mut t,
        &[(goblin, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(bears, giant)],
    );
    assert_eq!(blocks_now(&t), vec![(bears, giant)]);
}
