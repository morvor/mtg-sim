//! CR 400.7: "An object that moves from one zone to another becomes a new object with no
//! memory of, or relation to, its previous existence." An instruction about an object
//! that has left does nothing to it, nor to the new object it became. The actions.rs
//! primitives assert (in debug builds) that they're given live ids, so these paths, which
//! the liveness assertions found passing ids of objects that had left, now leave such
//! objects out themselves (`Game::present`).

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` is offered an activated ability of `source`.
fn offers_activation(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: s, .. } if *s == source))
}

#[test]
fn doubling_the_counters_on_a_creature_that_left_does_nothing() {
    cr!("400.7", "701.10e");
    // Primordial Hydra: "At the beginning of your upkeep, double the number of +1/+1
    // counters on this creature." It's destroyed with the trigger on the stack.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Primordial Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 3, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.g.destroy(hydra, None));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Primordial Hydra"));
    let card = t.g.current(hydra);
    assert_eq!(t.counters(card, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // With the Hydra still there, the counters double.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Primordial Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 3, None);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 6, "{}", t.dump_log());
}

#[test]
fn a_source_sacrificed_for_mana_cant_also_be_sacrificed_as_its_cost() {
    cr!("400.7", "601.2g", "601.2h", "602.2b");
    // Brass Secretary: "{2}, Sacrifice this creature: Draw a card." Ashnod's Altar:
    // "Sacrifice a creature: Add {C}{C}." The Secretary is the only creature: sacrificing
    // it for the {2} (a mana ability activated before the costs are paid) leaves nothing
    // to pay "Sacrifice this creature" with.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashnod's Altar");
    let secretary = t.battlefield(P0, "Brass Secretary");
    let hand = t.hand_size(P0);
    assert!(!offers_activation(&mut t, P0, secretary));
    assert!(t.activate(P0, secretary, 0, &[]).is_err());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
    assert!(t.on_battlefield(secretary));
}

#[test]
fn a_creature_that_left_cant_be_sacrificed_as_a_cost_its_ability_asks_for() {
    cr!("400.7", "118.3", "118.12a");
    use mtg_engine::events::MoveCause;
    use mtg_engine::object::Zone;
    // Withercrown: "Enchanted creature has base power 0 and has "At the beginning of your
    // upkeep, you lose 1 life unless you sacrifice this creature."" The Bears return to
    // their owner's hand with the trigger on the stack: the card in the hand is a new
    // object, which can't be sacrificed, so P0 loses 1 life though choosing to sacrifice.
    let setup = |leaves: bool| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let crown = t.battlefield(P1, "Withercrown");
        assert!(t.g.attach(crown, Entity::Object(bears)));
        t.g.recompute();
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        if leaves {
            let back = t.g.move_object(bears, Zone::Hand(P0), MoveCause::Effect, None);
            assert!(back.is_some());
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        t
    };
    let t = setup(true);
    assert_eq!(t.life(P0), 19, "{}", t.dump_log());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Still there, it's sacrificed instead.
    let t = setup(false);
    assert_eq!(t.life(P0), 20, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_source_sacrificed_for_mana_cant_have_counters_removed_as_its_cost() {
    cr!("400.7", "601.2g", "601.2h", "602.2b");
    interp!("automatic-mana-payment");
    // Thopter Squadron: "{1}, Remove a +1/+1 counter from this creature: Create a 1/1
    // colorless Thopter artifact creature token with flying. Activate only as a sorcery."
    // With Ashnod's Altar the only mana, sacrificing the Squadron for the {1} leaves no
    // counter to remove: the ability can't be activated, so it isn't offered.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ashnod's Altar");
    let squadron = t.battlefield(P0, "Thopter Squadron");
    t.g.add_counters(Entity::Object(squadron), "+1/+1", 3, None);
    assert!(!offers_activation(&mut t, P0, squadron));
    assert!(t.activate(P0, squadron, 0, &[]).is_err());
    assert!(t.on_battlefield(squadron));
    assert_eq!(t.counters(squadron, "+1/+1"), 3);
    // With another creature for the Altar, that one is sacrificed for the {1}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ashnod's Altar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let squadron = t.battlefield(P0, "Thopter Squadron");
    t.g.add_counters(Entity::Object(squadron), "+1/+1", 3, None);
    assert!(offers_activation(&mut t, P0, squadron));
    assert!(t.activate(P0, squadron, 0, &[]).is_ok(), "{}", t.dump_log());
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(squadron));
    assert_eq!(t.counters(squadron, "+1/+1"), 2);
    // With a land for the {1}, it can, and the Squadron stays.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ashnod's Altar");
    let squadron = t.battlefield(P0, "Thopter Squadron");
    t.g.add_counters(Entity::Object(squadron), "+1/+1", 3, None);
    t.lands(P0, "Wastes", 1);
    assert!(t.activate(P0, squadron, 0, &[]).is_ok());
    t.resolve_all();
    assert!(t.on_battlefield(squadron));
    assert_eq!(t.counters(squadron, "+1/+1"), 2);
    let tokens = t.g.battlefield.iter().filter(|o| !t.obj(**o).is_card()).count();
    assert_eq!(tokens, 1, "{}", t.dump_log());
}

#[test]
fn a_face_down_permanent_sacrificed_for_mana_isnt_turned_face_up() {
    cr!("400.7", "702.37e", "116.2b", "733.1");
    interp!("automatic-mana-payment");
    use mtg_engine::keywords::KeywordKind;
    use mtg_engine::object::CastMethod;
    // Rattleclaw Mystic: "Morph {2}". Cast face down with three Islands, then turned face
    // up with Ashnod's Altar ("Sacrifice a creature: Add {C}{C}.") the only mana.
    let setup = |bears: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Island", 3);
        let card = t.hand(P0, "Rattleclaw Mystic");
        t.cast(P0, card)
            .method(CastMethod::FaceDown(KeywordKind::Morph))
            .go();
        t.resolve();
        let mystic = t.g.current(card);
        assert!(t.obj(mystic).face_down && t.on_battlefield(mystic));
        t.battlefield(P0, "Ashnod's Altar");
        let other = bears.then(|| t.battlefield(P0, "Grizzly Bears"));
        t.g.turn.priority = Some(P0);
        let turn_up = Action::Special(SpecialAction::TurnFaceUp { obj: mystic });
        let offered = t.g.legal_actions(P0).contains(&turn_up);
        let r = t.g.perform_action(P0, turn_up);
        (t, mystic, other, offered, r)
    };
    // Only the Mystic itself could pay: sacrificing it leaves nothing to turn face up, so
    // the action isn't offered, and taking it anyway is reversed: it stays face down.
    let (t, mystic, _, offered, r) = setup(false);
    assert!(!offered);
    assert!(r.is_err(), "{}", t.dump_log());
    assert!(t.on_battlefield(mystic) && t.obj(mystic).face_down);
    // Another creature pays for it.
    let (t, mystic, bears, offered, r) = setup(true);
    assert!(offered);
    assert!(r.is_ok(), "{}", t.dump_log());
    assert!(!t.on_battlefield(bears.unwrap()));
    assert!(t.on_battlefield(mystic) && !t.obj(mystic).face_down);
    assert_eq!(t.obj(mystic).chars.name.as_str(), "Rattleclaw Mystic");
}

#[test]
fn a_face_down_permanent_returned_for_its_own_morph_cost_isnt_turned_face_up() {
    cr!("400.7", "702.37e", "733.1");
    use mtg_engine::keywords::KeywordKind;
    use mtg_engine::object::CastMethod;
    // Raven Guild Initiate: "Morph—Return a Bird you control to its owner's hand." Arcane
    // Adaptation naming Bird makes the face-down Initiate a Bird, so it can be the Bird
    // returned for its own cost: then it's a new object in its owner's hand, which can't
    // be turned face up, and the action is reversed (CR 733.1: "starts to take an action
    // but can't legally complete it").
    let setup = |return_itself: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        crate::r613_audit_common::choose_creature_type(&mut t, P0, "Bird");
        t.enter(P0, "Arcane Adaptation");
        t.resolve_all();
        t.lands(P0, "Island", 3);
        let card = t.hand(P0, "Raven Guild Initiate");
        t.cast(P0, card)
            .method(CastMethod::FaceDown(KeywordKind::Morph))
            .go();
        t.resolve();
        let initiate = t.g.current(card);
        assert!(t.obj(initiate).face_down && t.on_battlefield(initiate));
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.recompute();
        assert!(t.obj(initiate).chars.subtypes.iter().any(|s| s.as_str() == "Bird"));
        let returned = if return_itself { initiate } else { bears };
        t.answer_choose(P0, &[Entity::Object(returned)]);
        t.g.turn.priority = Some(P0);
        let r = t.g.perform_action(
            P0,
            Action::Special(SpecialAction::TurnFaceUp { obj: initiate }),
        );
        (t, initiate, bears, r)
    };
    let (t, initiate, bears, r) = setup(true);
    assert!(r.is_err(), "{}", t.dump_log());
    assert!(t.on_battlefield(initiate) && t.obj(initiate).face_down);
    assert!(t.on_battlefield(bears));
    // Another Bird returned: it's turned face up.
    let (t, initiate, bears, r) = setup(false);
    assert!(r.is_ok(), "{}", t.dump_log());
    assert!(t.on_battlefield(initiate) && !t.obj(initiate).face_down);
    assert_eq!(t.obj(initiate).chars.name.as_str(), "Raven Guild Initiate");
    assert!(!t.on_battlefield(bears));
}

#[test]
fn a_special_action_that_doesnt_act_on_its_source_may_sacrifice_it() {
    cr!("116.2d", "701.21a");
    // Volrath's Curse: "That creature's controller may sacrifice a permanent of their
    // choice for that player to ignore this effect until end of turn." P1 controls the
    // Curse on its own Bears and sacrifices the Curse itself: nothing in the action needs
    // the Curse once the cost is paid, so the action stands (unlike turning a permanent
    // face up, which needs the permanent, CR 733.1).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let curse = t.battlefield(P1, "Volrath's Curse");
    assert!(t.g.attach(curse, Entity::Object(bears)));
    t.g.recompute();
    t.g.turn.priority = Some(P1);
    let special = t
        .g
        .legal_actions(P1)
        .into_iter()
        .find(|a| matches!(a, Action::Special(SpecialAction::Static { source, .. }) if *source == curse))
        .expect("the special action");
    t.answer_choose(P1, &[Entity::Object(curse)]);
    assert!(t.g.perform_action(P1, special).is_ok(), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Volrath's Curse"));
    assert!(t.on_battlefield(bears));
    // Damping Engine: "That player may sacrifice a permanent of their choice for that
    // player to ignore this effect until end of turn." P0 has the most permanents and
    // sacrifices the Engine itself.
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Damping Engine");
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    let special = t
        .g
        .legal_actions(P0)
        .into_iter()
        .find(|a| matches!(a, Action::Special(SpecialAction::Other { obj: Some(o), .. }) if *o == engine))
        .expect("the special action");
    t.answer_choose(P0, &[Entity::Object(engine)]);
    assert!(t.g.perform_action(P0, special).is_ok(), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Damping Engine"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
}
