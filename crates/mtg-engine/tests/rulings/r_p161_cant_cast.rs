//! Rulings batch P161 — conditional "can't cast spells" statics: Wardscale Dragon ("As
//! long as this creature is attacking, defending player can't cast spells.") and
//! Conqueror's Flail ("As long as this Equipment is attached to a creature, your opponents
//! can't cast spells during your turn."), plus the Flail's color count (CR 105.1).

use crate::r_p160_common::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` can cast a Shock at `at` now (it resolves if so).
fn can_shock(t: &mut TestGame, p: PlayerId, at: Entity) -> bool {
    lands_for_cost(t, p, "Shock");
    let shock = t.hand(p, "Shock");
    t.answer_targets(p, &[at]);
    let ok = t.cast(p, shock).try_go().is_ok();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

/// P0 attacks with Wardscale Dragon (attacking `target`); returns in the declare attackers
/// step.
fn dragon_attacks(t: &mut TestGame, target: Entity) -> ObjectId {
    let dragon = t.battlefield(P0, "Wardscale Dragon");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(dragon, target)]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    dragon
}

#[test]
fn wardscale_dragon_defending_player_can_still_activate_and_take_special_actions() {
    cr!("116.2b", "602.1", "101.2");
    ruling!(
        "Wardscale Dragon",
        "The defending player can still activate abilities or take special actions, such as turning a face-down creature face up."
    );
    supported("Wardscale Dragon");
    supported("Zombie Cutthroat");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    // P1 has a face-down Zombie Cutthroat (morph — pay 5 life) and a Prodigal Sorcerer.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 3);
    let cut = t.hand(P1, "Zombie Cutthroat");
    t.cast(P1, cut)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let cut = t.g.current(cut);
    assert!(t.obj(cut).face_down);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P0, Step::PrecombatMain);
    dragon_attacks(&mut t, Entity::Player(P1));
    // No spells...
    assert!(!can_shock(&mut t, P1, Entity::Player(P0)));
    // ... but abilities and special actions.
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    t.g.turn.priority = Some(P1);
    assert!(t
        .g
        .perform_action(P1, Action::Special(SpecialAction::TurnFaceUp { obj: cut }))
        .is_ok());
    t.g.recompute();
    assert!(!t.obj_now(cut).face_down);
    // The attacker can still cast spells.
    assert!(can_shock(&mut t, P0, Entity::Player(P1)));
    // After combat, the defending player can cast spells again.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_shock(&mut t, P1, Entity::Player(P0)));
}

#[test]
fn wardscale_dragon_defending_player_is_the_one_it_attacks() {
    cr!("506.2", "802.2a", "101.2");
    ruling!(
        "Wardscale Dragon",
        "“Defending player” refers to the player Wardscale Dragon is attacking or the controller of the planeswalker Wardscale Dragon is attacking. In multiplayer formats that allow attacking multiple players, your other opponents can still cast spells, even if you control other creatures attacking those players or planeswalkers those players control."
    );
    supported("Wardscale Dragon");
    supported("Oko, Thief of Crowns");
    // Attacking P1 in a three-player game: P1 can't cast spells, P2 can, even with
    // another creature attacking P2.
    let mut t = TestGame::new(3);
    let dragon = t.battlefield(P0, "Wardscale Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (dragon, Entity::Player(P1)),
            (bears, Entity::Player(P2)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!can_shock(&mut t, P1, Entity::Player(P0)));
    assert!(can_shock(&mut t, P2, Entity::Player(P0)));
    // Attacking P1's planeswalker: P1 is the defending player.
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P1, "Oko, Thief of Crowns");
    dragon_attacks(&mut t, Entity::Object(oko));
    assert!(!can_shock(&mut t, P1, Entity::Player(P0)));
}

#[test]
fn conquerors_flail_counts_at_most_five_colors() {
    cr!("105.1", "105.2", "301.5");
    ruling!(
        "Conqueror's Flail",
        "The five colors are white, blue, black, red, and green. Conqueror's Flail can't give a creature more than +5/+5. (Gold, artifact, and colorless aren't colors.)"
    );
    supported("Conqueror's Flail");
    let mut t = TestGame::new(2);
    let flail = t.battlefield(P0, "Conqueror's Flail");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate_resolve(&mut t, P0, flail, 0, &[Entity::Object(bears)]);
    // Green (the Bears) and colorless (the Flail, Wastes): +1/+1.
    assert_eq!(t.pt(bears), (3, 3));
    t.battlefield(P0, "Azorius Guildmage");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    for name in ["Bereavement", "Havoc", "Serra Angel", "Crystalline Resonance"] {
        t.battlefield(P0, name);
    }
    t.g.recompute();
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn conquerors_flail_stops_opponents_casting_during_your_turn() {
    cr!("101.2", "611.3a");
    supported("Conqueror's Flail");
    let mut t = TestGame::new(2);
    let flail = t.battlefield(P0, "Conqueror's Flail");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Unattached: no restriction.
    assert!(can_shock(&mut t, P1, Entity::Player(P0)));
    t.lands(P0, "Wastes", 2);
    activate_resolve(&mut t, P0, flail, 0, &[Entity::Object(bears)]);
    assert!(!can_shock(&mut t, P1, Entity::Player(P0)));
    assert!(can_shock(&mut t, P0, Entity::Player(P1)));
    // On P1's turn, P1 can.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_shock(&mut t, P1, Entity::Player(P0)));
}
