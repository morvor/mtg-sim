//! Edicts with choices (CR 701.21a, 101.4): several kinds of permanents at once (each
//! permanent fills one kind, as many kinds as possible), qualifiers after "of their
//! choice", "that many", players described by what they control, and optional edicts for
//! several players ("any opponent may sacrifice ... If a player does, ...").

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The candidates of the choices `p` was asked to make since `from`.
fn choices_asked(t: &TestGame, from: usize, p: PlayerId) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn structural_collapse_an_artifact_land_fills_only_one_kind() {
    cr!("701.21a", "101.4");
    ruling!(
        "Structural Collapse",
        "If the player controls an artifact land, they can’t choose to sacrifice that permanent for both the artifact and the land."
    );
    compiles("Structural Collapse");
    // The artifact land is the only artifact: it's the artifact, and another land goes.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let den = t.battlefield(P1, "Ancient Den");
    let island = t.battlefield(P1, "Island");
    let swamp = t.battlefield(P1, "Swamp");
    let spell = t.hand(P0, "Structural Collapse");
    t.answer_choose(P1, &[Entity::Object(island)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(den));
    assert_eq!(
        [island, swamp].iter().filter(|o| t.on_battlefield(**o)).count(),
        1
    );
    assert!(!t.on_battlefield(island));
    assert_eq!(t.life(P1), 18);
    // With another artifact and no other land, the artifact land must be the land: only
    // the other artifact may be chosen as the artifact.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let den = t.battlefield(P1, "Ancient Den");
    let thopter = t.battlefield(P1, "Ornithopter");
    let spell = t.hand(P0, "Structural Collapse");
    let from = t.asked().len();
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let asked = choices_asked(&t, from, P1);
    assert_eq!(asked.first(), Some(&vec![Entity::Object(thopter)]));
    assert!(!t.on_battlefield(den));
    assert!(!t.on_battlefield(thopter));
}

#[test]
fn release_each_player_sacrifices_one_of_each_kind() {
    cr!("701.21a", "101.4");
    ruling!(
        "Catch // Release",
        "You must choose the artifact creature as the artifact and the nonartifact creature as the creature"
    );
    ruling!(
        "Catch // Release",
        "then all the chosen permanents are sacrificed simultaneously"
    );
    compiles("Catch // Release");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Plains", 1);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let other = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Catch // Release");
    let from = t.asked().len();
    t.cast(P0, spell).method(CastMethod::Half(1)).go();
    t.resolve();
    // The artifact creature is the only artifact: it's the artifact. Then one of the
    // other creatures (not the artifact creature again).
    let asked = choices_asked(&t, from, P1);
    assert_eq!(asked[0], vec![Entity::Object(thopter)]);
    assert!(!asked[1].contains(&Entity::Object(thopter)));
    assert!(!t.on_battlefield(thopter));
    assert!(!t.on_battlefield(forest));
    assert_eq!(
        [bears, other].iter().filter(|o| t.on_battlefield(**o)).count(),
        1
    );
    // P0 sacrificed a land too.
    assert_eq!(t.g.permanents_controlled_by(P0).len(), 5);
    // The active player chooses first, then each other player in turn order.
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    let first_p1 = order.iter().position(|p| *p == P1).expect("P1 chose");
    assert!(order[..first_p1].contains(&P0), "{order:?}");
    assert!(order[first_p1..].iter().all(|p| *p == P1), "{order:?}");
}

#[test]
fn perilous_predicament_an_artifact_creature_and_a_nonartifact_creature() {
    cr!("701.21a", "101.4");
    ruling!(
        "Perilous Predicament",
        "If each creature an opponent controls is an artifact creature, that player sacrifices only one creature."
    );
    compiles("Perilous Predicament");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Perilous Predicament");
    t.answer_choose(P1, &[Entity::Object(thopter)]);
    t.answer_choose(P1, &[Entity::Object(elves)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.on_battlefield(thopter));
    assert!(!t.on_battlefield(elves));
    assert!(t.on_battlefield(bears));
    // Only artifact creatures: one creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    let spell = t.hand(P0, "Perilous Predicament");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!([a, b].iter().filter(|o| t.on_battlefield(**o)).count(), 1);
}

#[test]
fn witch_king_sacrifices_a_creature_that_dealt_combat_damage_to_you() {
    cr!("701.21a", "510.2");
    compiles("Witch-king of Angmar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Witch-king of Angmar");
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(attacker, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(idle));
}

#[test]
fn phyrexian_obliterator_that_sources_controller_sacrifices_that_many() {
    cr!("701.21a", "120.3");
    compiles("Phyrexian Obliterator");
    let mut t = TestGame::new(2);
    let obliterator = t.battlefield(P0, "Phyrexian Obliterator");
    t.lands(P1, "Mountain", 4);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(obliterator).go();
    t.resolve_all();
    assert!(t.on_battlefield(obliterator));
    // Three of P1's four lands are sacrificed.
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
}

#[test]
fn tectonic_hellion_each_player_who_controls_the_most_lands() {
    cr!("701.21a", "101.4");
    ruling!(
        "Tectonic Hellion",
        "If more than one player controls the most lands, each of those players sacrifices two lands."
    );
    compiles("Tectonic Hellion");
    let mut t = TestGame::new(3);
    let hellion = t.battlefield(P0, "Tectonic Hellion");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Forest", 3);
    t.lands(P2, "Island", 3);
    t.attack(&[(hellion, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.g.permanents_controlled_by(P0).len(), 3);
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
    assert_eq!(t.g.permanents_controlled_by(P2).len(), 1);
}

#[test]
fn desecration_demon_any_opponent_may_sacrifice() {
    cr!("701.21a", "101.4");
    ruling!(
        "Desecration Demon",
        "Each opponent in turn order may choose to sacrifice a creature, even if an opponent already chose to sacrifice a creature that combat."
    );
    compiles("Desecration Demon");
    let mut t = TestGame::new(3);
    let demon = t.battlefield(P0, "Desecration Demon");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(b1)]);
    t.answer_choose(P2, &[Entity::Object(b2)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(b1));
    assert!(!t.on_battlefield(b2));
    assert!(t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 1);
    // No one sacrifices: nothing happens.
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Desecration Demon");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(b1));
    assert!(!t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 0);
}

#[test]
fn clackbridge_troll_if_a_player_does() {
    cr!("701.21a", "101.4");
    ruling!(
        "Clackbridge Troll",
        "you don't gain more life or draw more cards if more than one creature was sacrificed"
    );
    compiles("Clackbridge Troll");
    let mut t = TestGame::new(3);
    let troll = t.battlefield(P0, "Clackbridge Troll");
    let g1 = t.battlefield(P1, "Grizzly Bears");
    let g2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(g1)]);
    t.answer_choose(P2, &[Entity::Object(g2)]);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(g1) && !t.on_battlefield(g2));
    assert!(t.obj_now(troll).tapped);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn labyrinth_raptor_defending_player_sacrifices_a_blocker() {
    cr!("701.21a", "509.1");
    ruling!(
        "Labyrinth Raptor",
        "The defending player sacrifices a blocking creature before combat damage is dealt."
    );
    compiles("Labyrinth Raptor");
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Labyrinth Raptor");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P1, "Llanowar Elves");
    let from = t.asked().len();
    t.attack(&[(raptor, Entity::Player(P1))], &[(a, raptor), (b, raptor)]);
    let asked = choices_asked(&t, from, P1);
    assert!(asked
        .iter()
        .any(|c| c.contains(&Entity::Object(a)) && !c.contains(&Entity::Object(idle))));
    assert!(t.on_battlefield(idle));
    assert!(!(t.on_battlefield(a) && t.on_battlefield(b)));
}
