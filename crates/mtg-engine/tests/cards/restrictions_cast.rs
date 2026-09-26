//! Temporary casting prohibitions from resolving spells and abilities (CR 601.3).

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

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card));
    t.g.turn.priority = saved;
    ok
}

#[test]
fn your_opponents_cant_cast_spells_this_turn() {
    cr!("601.3");
    ruling!("Silence", "The only thing Silence stops is casting spells.");
    compiles("Silence");
    compiles("Orim's Chant");
    let mut t = TestGame::new(2);
    let silence = t.hand(P0, "Silence");
    t.lands(P0, "Plains", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let forest = t.hand(P1, "Forest");
    t.lands(P1, "Mountain", 1);
    let mine = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.set_step(P1, Step::Upkeep);
    t.cast(P0, silence).go();
    t.resolve();
    assert!(!castable(&mut t, P1, bolt));
    assert!(castable(&mut t, P0, mine));
    // P1 can still play lands.
    t.set_step(P1, Step::PrecombatMain);
    t.play_land(P1, forest).unwrap();
    // Next turn, P1 can cast spells again.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(castable(&mut t, P1, bolt));
}

#[test]
fn its_controller_cant_cast_spells_even_if_not_countered() {
    cr!("601.3");
    ruling!(
        "Render Silent",
        "The spell won’t be countered when Render Silent resolves, but that spell’s controller won’t be able to cast spells for the rest of the turn."
    );
    compiles("Render Silent");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let render = t.hand(P0, "Render Silent");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let decay = t.hand(P1, "Abrupt Decay");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, decay).target(bears).go();
    t.cast(P0, render).target(spell).go();
    t.resolve();
    // Abrupt Decay can't be countered: it's still on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(!castable(&mut t, P1, bolt));
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!castable(&mut t, P1, bolt));
}

#[test]
fn the_countered_spells_controller_cant_cast_spells() {
    cr!("601.3", "701.6a");
    let mut t = TestGame::new(2);
    let render = t.hand(P0, "Render Silent");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let mine = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, bears).go();
    t.cast(P0, render).target(spell).go();
    t.resolve();
    // The spell is countered and its controller (not Render Silent's) is silenced.
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!castable(&mut t, P1, bolt));
    assert!(castable(&mut t, P0, mine));
}

#[test]
fn defending_player_cant_cast_spells_this_turn() {
    cr!("601.3", "508.5");
    ruling!(
        "Xantid Swarm",
        "only the player Xantid Swarm attacked is affected"
    );
    compiles("Xantid Swarm");
    let mut t = TestGame::new(3);
    let swarm = t.battlefield(P0, "Xantid Swarm");
    let b1 = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let b2 = t.hand(P2, "Lightning Bolt");
    t.lands(P2, "Mountain", 1);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(swarm, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(!castable(&mut t, P1, b1));
    assert!(castable(&mut t, P2, b2));
}
