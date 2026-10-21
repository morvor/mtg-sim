//! CR 101.4: players are taken in APNAP order — the active player, then the others in
//! turn order — never by where they sit. Found by the seat symmetry property
//! (docs/PROPERTIES.md): a player's opponents were listed in seat order, so when the
//! engine declared attackers for a player whose declaration broke a requirement
//! (CR 508.1d), the attack went to the opponent in the lowest seat.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn opponents_are_listed_from_the_active_player_in_turn_order() {
    cr!("101.4", "102.3");
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    assert_eq!(t.g.opponents(P2), vec![P3, P0, P1]);
    assert_eq!(t.g.opponents(P0), vec![P2, P3, P1]);
}

#[test]
fn a_required_attack_the_engine_declares_goes_to_the_next_player_in_turn_order() {
    cr!("101.4", "508.1d");
    let mut t = TestGame::new(3);
    // "Juggernaut attacks each combat if able."
    let juggernaut = t.battlefield(P1, "Juggernaut");
    t.set_step(P1, Step::PrecombatMain);
    // P1 declares no attackers, breaking the requirement: the engine declares a legal
    // attack instead, and its first choice of defender is the next player in turn order.
    t.answer(P1, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P1, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().expect("combat");
    let a = c.attackers.iter().find(|a| a.id == juggernaut).expect("it attacks");
    assert_eq!(a.target, Some(Entity::Player(P2)));
}

#[test]
fn choices_at_random_take_the_players_in_apnap_order() {
    // Found by seat symmetry: "target opponent chosen at random" shuffled the candidates
    // as listed, in seat order, so the same random draw picked a different player
    // depending on where the players sat.
    cr!("101.4");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    let mut v = vec![
        Entity::Object(a),
        Entity::Player(P0),
        Entity::Object(b),
        Entity::Player(P1),
        Entity::Player(P2),
    ];
    t.g.sort_apnap(&mut v);
    assert_eq!(
        v,
        vec![
            Entity::Player(P1),
            Entity::Player(P2),
            Entity::Player(P0),
            Entity::Object(b),
            Entity::Object(a),
        ]
    );
}

#[test]
fn targets_that_must_go_together_are_replaced_without_regard_to_seats() {
    // Found by seat symmetry: when chosen targets didn't go together ("two target cards
    // from an opponent's graveyard" from two graveyards), the group chosen instead was
    // the first among the candidates in seat order.
    cr!("101.4", "601.2c");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    // "{1}{B}, {T}: Exile two target cards from an opponent's graveyard. ..."
    let tracker = t.battlefield(P1, "Deadeye Tracker");
    t.lands(P1, "Swamp", 2);
    let p0 = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Hill Giant"),
    ];
    let p2 = [
        t.graveyard(P2, "Grizzly Bears"),
        t.graveyard(P2, "Hill Giant"),
    ];
    t.answer(
        P1,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Object(p0[0]), Entity::Object(p2[0])]),
    );
    let ab = t.activate(P1, tracker, 0, &[]).unwrap().expect("on the stack");
    let targets = &t.g.obj(ab).stack.as_ref().unwrap().chosen[0].targets[0];
    // P2 is the first opponent after P1 in turn order.
    assert_eq!(
        targets,
        &vec![Entity::Object(p2[0]), Entity::Object(p2[1])],
        "P0's cards: {p0:?}"
    );
}
