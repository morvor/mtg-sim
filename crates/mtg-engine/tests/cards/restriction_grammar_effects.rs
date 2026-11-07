//! Restriction effects and statics compiled by the restriction grammar: P/T changes with
//! restrictions for a duration, "don't untap during [whose] next untap step", "If a
//! creature you control attacks, ~ also attacks if able", "attack a player other than
//! you if able", "Each creature dealt damage this way attacks this turn if able", "This
//! spell can't be copied", quoted abilities naming the token, "[spells] can't be cast",
//! "..., and all creatures able to block it do so", "Cast this spell only if you've cast
//! another spell this turn".

use mtg_engine::combat::{
    attack_declaration_legal, attack_options, block_declaration_legal, block_options,
};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn can_activate(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == src))
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.advance_to(ap, Step::DeclareAttackers);
}

#[test]
fn gets_minus_three_and_its_activated_abilities_cant_be_activated_until_your_next_turn() {
    cr!("611.2a", "602.5");
    compiles("Dovin Baan");
    let mut t = TestGame::new(2);
    let dovin = t.battlefield(P0, "Dovin Baan");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P0, dovin, 0, &[Entity::Object(pinger)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(pinger), (-2, 1));
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_activate(&mut t, P1, pinger));
    assert_eq!(t.pt(pinger), (-2, 1));
    t.advance_to(P0, Step::Upkeep);
    assert!(can_activate(&mut t, P1, pinger));
    assert_eq!(t.pt(pinger), (1, 1));
}

#[test]
fn dont_untap_during_their_next_untap_steps() {
    cr!("502.3");
    compiles("Blinding Beam");
    compiles("Reduce // Rubble");
    let mut t = TestGame::new(2);
    let beam = t.hand(P0, "Blinding Beam");
    t.lands(P0, "Plains", 3);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(a);
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.tap(mine);
    // Mode 2: "Creatures don't untap during target player's next untap step."
    t.cast(P0, beam).modes(&[1]).target(P1).go();
    t.resolve();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(a).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(mine).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(a).tapped);
}

#[test]
fn attacks_if_able_if_another_creature_attacks() {
    cr!("508.1d");
    compiles("Ekundu Cyclops");
    compiles("Viashino Bey");
    compiles("War's Toll");
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Ekundu Cyclops");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p1 = Entity::Player(P1);
    assert!(attack_declaration_legal(&t.g, &opts, &[]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(cyclops, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(bears, p1)]));
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(bears, p1), (cyclops, p1)]
    ));

    let mut t = TestGame::new(2);
    let bey = t.battlefield(P0, "Viashino Bey");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(bey, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bey, p1), (bears, p1)]));

    // War's Toll: its controller's opponents' creatures attack together.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "War's Toll");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p0 = Entity::Player(P0);
    assert!(!attack_declaration_legal(&t.g, &opts, &[(x, p0)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(x, p0), (y, p0)]));
    // Not its controller's own creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "War's Toll");
    let mx = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[(mx, p1)]));
}

#[test]
fn attack_each_combat_and_a_player_other_than_you_if_able() {
    cr!("508.1d");
    compiles("Kardur, Doomscourge");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let kardur = t.hand(P0, "Kardur, Doomscourge");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 2);
    t.cast(P0, kardur).go();
    t.resolve_all();
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(!attack_declaration_legal(&t.g, &opts, &[]));
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(bears, Entity::Player(P0))]
    ));
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(bears, Entity::Player(P2))]
    ));
}

#[test]
fn each_creature_dealt_damage_this_way_attacks_this_turn_if_able() {
    cr!("508.1d", "608.2c");
    compiles("Aggravate");
    let mut t = TestGame::new(2);
    let aggravate = t.hand(P0, "Aggravate");
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.cast(P0, aggravate).target(P1).go();
    t.resolve();
    // Entered after the spell resolved: not dealt damage this way.
    let late = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p0 = Entity::Player(P0);
    assert!(!attack_declaration_legal(&t.g, &opts, &[]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p0)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p0), (late, p0)]));
    let _ = late;
    // Next turn it's over.
    t.advance_to(P0, Step::Upkeep);
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn this_spell_cant_be_copied() {
    cr!("707.10");
    compiles("See Double");
    let mut t = TestGame::new(2);
    let see = t.hand(P0, "See Double");
    t.lands(P0, "Island", 4);
    let target = t.battlefield(P1, "Grizzly Bears");
    let spell = t.cast(P0, see).modes(&[1]).target(target).go();
    assert!(mtg_engine::copy::copy_spell(&mut t.g, spell, P0, false).is_none());
    // Another spell can be copied.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let b = t.cast(P0, bolt).target(P1).go();
    assert!(mtg_engine::copy::copy_spell(&mut t.g, b, P0, false).is_some());
}

#[test]
fn a_named_tokens_quoted_ability_refers_to_the_token() {
    cr!("201.5", "509.1b");
    compiles("White Tiger, Ava Ayala");
    let mut t = TestGame::new(2);
    let tiger = t.battlefield(P0, "White Tiger, Ava Ayala");
    t.lands(P0, "Forest", 6);
    t.activate(P0, tiger, 0, &[]).unwrap();
    t.resolve();
    let god = t.named_on_battlefield("The Tiger God");
    assert_eq!(god.len(), 1);
    let god = god[0];
    t.g.objects[god.0 as usize].summoning_sick = false;
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(god, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(x, god)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, god), (y, god)]));
}

#[test]
fn noncreature_spells_with_mana_value_four_or_greater_cant_be_cast() {
    cr!("601.2", "202.3");
    compiles("Gaddock Teeg");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Gaddock Teeg");
    t.lands(P0, "Island", 5);
    let big = t.hand(P0, "Concentrate");
    let small = t.hand(P0, "Divination");
    let x = t.hand(P0, "Blaze");
    let creature = t.hand(P0, "Air Elemental");
    assert!(!castable(&mut t, P0, big));
    assert!(castable(&mut t, P0, small));
    assert!(!castable(&mut t, P0, x));
    assert!(castable(&mut t, P0, creature));
}

#[test]
fn all_creatures_able_to_block_the_enchanted_creature_do_so() {
    cr!("509.1c");
    compiles("Indrik Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = t.battlefield(P0, "Indrik Umbra");
    t.g.attach(umbra, Entity::Object(bears));
    let other = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Llanowar Elves");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(x, bears), (y, bears)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, bears), (y, other)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn cast_only_if_youve_cast_another_spell_this_turn() {
    cr!("601.3");
    compiles("Hewed Stone Retainers");
    compiles("Talara's Battalion");
    compiles("Dream Thief");
    let mut t = TestGame::new(2);
    let golem = t.hand(P0, "Hewed Stone Retainers");
    let elves = t.hand(P0, "Talara's Battalion");
    t.lands(P0, "Forest", 6);
    assert!(!castable(&mut t, P0, golem));
    assert!(!castable(&mut t, P0, elves));
    // A red spell: "another spell", but not "another green spell".
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert!(castable(&mut t, P0, golem));
    assert!(!castable(&mut t, P0, elves));
    let llanowar = t.hand(P0, "Llanowar Elves");
    t.cast(P0, llanowar).go();
    t.resolve();
    assert!(castable(&mut t, P0, elves));

    // Dream Thief's own spell isn't "another blue spell".
    let mut t = TestGame::new(2);
    let thief = t.hand(P0, "Dream Thief");
    t.lands(P0, "Island", 3);
    t.cast(P0, thief).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}
