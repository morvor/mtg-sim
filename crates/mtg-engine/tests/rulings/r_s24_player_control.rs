//! Rulings batch S24 — controlling another player (CR 723): Emrakul, the Promised End and
//! Sorin Markov's last ability.

use crate::r_s01_common::supported;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::{can_see, decider};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 casts Emrakul, the Promised End targeting P1 ("When you cast this spell, you gain
/// control of target opponent during that player's next turn."), and it resolves.
fn emrakul(t: &mut TestGame) {
    supported("Emrakul, the Promised End");
    let emrakul = t.hand(P0, "Emrakul, the Promised End");
    t.lands(P0, "Wastes", 13);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, emrakul).go();
    t.resolve_all();
}

/// P0 activates Sorin Markov's "−7: You control target player during that player's next
/// turn." targeting P1, and it resolves.
fn sorin(t: &mut TestGame) {
    supported("Sorin Markov");
    let sorin = t.battlefield(P0, "Sorin Markov");
    t.g.add_counters(Entity::Object(sorin), counters::LOYALTY, 3, None);
    let uid = t.obj_now(sorin).chars.abilities[2].uid;
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, sorin, uid).expect("activate −7");
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn the_controlled_player_is_still_active_and_keeps_control_of_their_objects() {
    cr!("723.1", "723.3", "723.5");
    ruling!(
        "Emrakul, the Promised End",
        "The player you’re controlling is still the active player during that turn."
    );
    ruling!(
        "Emrakul, the Promised End",
        "You only control the player. You don’t control any of that player’s permanents, spells, or abilities."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    emrakul(&mut t);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    assert_eq!(t.g.turn.active, P1);
    // P0 has P1 cast P1's Lightning Bolt: it's P1's spell.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.advance();
    let spell = *t.g.stack.last().expect("the Bolt was cast");
    assert_eq!(t.g.obj(spell).controller, P1);
    assert_eq!(t.g.obj(bears).controller, P1);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn the_controller_cant_make_the_controlled_player_concede() {
    cr!("723.6");
    ruling!(
        "Emrakul, the Promised End",
        "You can’t make the affected player concede. That player may choose to concede at any time, even while you’re controlling that player."
    );
    let mut t = TestGame::new(2);
    emrakul(&mut t);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    // P0 answers "concede" for P1: nothing happens.
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(!t.has_lost(P1));
    // P1 may still concede.
    t.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
}

#[test]
fn the_controller_sees_what_the_controlled_player_sees() {
    cr!("723.4");
    ruling!(
        "Sorin Markov",
        "While controlling another player, you can see all cards in the game that player can see. This includes cards in that player’s hand, face-down cards that player controls, and any cards in that player’s library the player may look at."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    assert!(!can_see(&t.g, P0, in_hand));
    sorin(&mut t);
    assert!(!can_see(&t.g, P0, in_hand));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    assert!(can_see(&t.g, P0, in_hand));
    // Not after that turn.
    t.advance_to(P0, Step::Upkeep);
    assert!(!can_see(&t.g, P0, in_hand));
}

#[test]
fn the_controller_cant_see_the_sideboard_or_choose_a_card_from_outside_the_game() {
    cr!("723.4");
    ruling!(
        "Sorin Markov",
        "Controlling a player doesn’t allow you to look at that player’s sideboard. If an effect instructs that player to choose a card from outside the game, you can’t have that player choose any card."
    );
    let mut t = TestGame::new(2);
    let outside = t.custom(P1, (*card("Lava Spike")).clone(), Zone::Outside(P1));
    let wish = t.hand(P1, "Burning Wish");
    t.lands(P1, "Mountain", 2);
    sorin(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(decider(&t.g, P1), P0);
    assert!(!can_see(&t.g, P0, outside));
    // P0 has P1 cast Burning Wish ("You may reveal a sorcery card you own from outside the
    // game and put it into your hand."): no card is chosen.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: wish,
            method: CastMethod::Normal,
        }),
    );
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert!(!t.in_hand(P1, "Lava Spike"));
    assert_eq!(t.zone(outside), Zone::Outside(P1));
}
