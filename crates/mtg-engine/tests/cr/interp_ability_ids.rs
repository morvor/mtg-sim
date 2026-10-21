//! Register entry `ability-ids-depend-on-the-process`: ability ids come from a counter the
//! whole process shares, so the same game played twice in one process can number its
//! abilities differently; options are told apart by their place in the list.

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A game where Quicksilver Elemental has gained Arc-Slogger's ability, the legal actions
/// of its controller, and the place of the gained ability among them.
fn play() -> (TestGame, Vec<Action>, usize) {
    let mut t = TestGame::new(2);
    // "{U}: This creature gains all activated abilities of target creature until end of
    // turn. ..."
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let slogger = t.battlefield(P1, "Arc-Slogger");
    t.lands(P0, "Island", 2);
    t.g.recompute();
    let gain = t
        .obj(qe)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("gains all activated"))
        .unwrap();
    t.activate(P0, qe, gain, &[Entity::Object(slogger)])
        .expect("activates");
    t.resolve();
    t.g.recompute();
    let gained = t
        .obj(qe)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("Exile the top ten"))
        .map(|a| a.uid)
        .expect("gained Arc-Slogger's ability");
    let actions = t.g.legal_actions(P0);
    let i = actions
        .iter()
        .position(|a| matches!(a, Action::Activate { source, ability } if *source == qe && *ability == gained))
        .expect("the gained ability can be activated");
    (t, actions, i)
}

#[test]
fn the_same_game_numbers_a_gained_ability_differently_but_lists_it_in_the_same_place() {
    interp!("ability-ids-depend-on-the-process");
    let (mut first, first_actions, i) = play();
    let (mut second, second_actions, j) = play();
    // The ability the effect gave is a new ability each time, with a new id from the
    // process-wide counter...
    let id = |a: &Action| match a {
        Action::Activate { ability, .. } => *ability,
        _ => 0,
    };
    assert_ne!(id(&first_actions[i]), id(&second_actions[j]));
    // ...but it's the same option: same place, and every other option is the same.
    assert_eq!(i, j);
    assert_eq!(first_actions.len(), second_actions.len());
    for (k, (a, b)) in first_actions.iter().zip(&second_actions).enumerate() {
        if k != i {
            assert_eq!(a, b);
        }
    }
    // Answering with the option's place does the same thing in both games.
    for (t, actions, k) in [
        (&mut first, &first_actions, i),
        (&mut second, &second_actions, j),
    ] {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.g.turn.priority = Some(P0);
        t.g.perform_action(P0, actions[k].clone()).expect("legal");
        t.resolve();
        assert_eq!(t.life(P1), 18);
    }
}
