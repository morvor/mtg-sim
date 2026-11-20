//! CR 400.7: "An object that moves from one zone to another becomes a new object with no
//! memory of, or relation to, its previous existence." An instruction about an object
//! that has left does nothing to it, nor to the new object it became. The actions.rs
//! primitives assert (in debug builds) that they're given live ids, so these paths, which
//! the liveness assertions found passing ids of objects that had left, now leave such
//! objects out themselves (`Game::present`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

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
    assert!(t.activate(P0, secretary, 0, &[]).is_err());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
    assert!(t.on_battlefield(secretary));
}

#[test]
fn a_source_sacrificed_for_mana_cant_have_counters_removed_as_its_cost() {
    cr!("400.7", "601.2g", "601.2h", "602.2b");
    // Thopter Squadron: "{1}, Remove a +1/+1 counter from this creature: Create a 1/1
    // colorless Thopter artifact creature token with flying. Activate only as a sorcery."
    // With Ashnod's Altar the only mana, sacrificing the Squadron for the {1} leaves no
    // counter to remove: the ability can't be activated.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ashnod's Altar");
    let squadron = t.battlefield(P0, "Thopter Squadron");
    t.g.add_counters(Entity::Object(squadron), "+1/+1", 3, None);
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
    use mtg_engine::decision::{Action, SpecialAction};
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
        let r = t.g.perform_action(
            P0,
            Action::Special(SpecialAction::TurnFaceUp { obj: mystic }),
        );
        (t, mystic, other, r)
    };
    // Only the Mystic itself could pay: sacrificing it leaves nothing to turn face up, so
    // the action is reversed and it stays face down.
    let (t, mystic, _, r) = setup(false);
    assert!(r.is_err(), "{}", t.dump_log());
    assert!(t.on_battlefield(mystic) && t.obj(mystic).face_down);
    // Another creature pays for it.
    let (t, mystic, bears, r) = setup(true);
    assert!(r.is_ok(), "{}", t.dump_log());
    assert!(!t.on_battlefield(bears.unwrap()));
    assert!(t.on_battlefield(mystic) && !t.obj(mystic).face_down);
    assert_eq!(t.obj(mystic).chars.name.as_str(), "Rattleclaw Mystic");
}
