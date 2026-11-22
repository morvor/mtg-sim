//! roundtrip-clusters-3 (docs/agents/items/roundtrip-clusters-3.md): parser bugs the round
//! trip found in its clusters, and cards whose rendering differs in wording, checked in play.

use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn adventure_awaits_draws_only_if_no_card_was_put_into_your_hand() {
    cr!("608.2c");
    // "You may reveal a creature card from among them and put it into your hand. ... If you
    // didn't put a card into your hand this way, draw a card."
    for take in [true, false] {
        let mut t = TestGame::new(2);
        for _ in 0..4 {
            t.library_top(P0, "Forest");
        }
        let bears = t.library_top(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 2);
        let spell = t.hand(P0, "Adventure Awaits");
        let chosen = if take {
            vec![Entity::Object(bears)]
        } else {
            vec![]
        };
        t.answer_choose(P0, &chosen);
        t.cast(P0, spell).go();
        t.resolve_all();
        // The Bears, or (none taken) a card drawn instead.
        assert_eq!(t.hand_size(P0), 1, "take {take}\n{}", t.dump_log());
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), take);
    }
}

#[test]
fn sorcerous_spyglass_looks_at_the_hand_without_revealing_it() {
    cr!("701.20a");
    // "As this artifact enters, look at an opponent's hand, then choose any card name": only
    // you see the cards; they aren't revealed (to every player).
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.enter(P0, "Sorcerous Spyglass");
    t.settle();
    assert!(t.g.reveals.revealed.is_empty(), "{}", t.dump_log());
}

#[test]
fn akoum_hellkite_checks_the_land_that_entered_not_the_target() {
    cr!("603.2", "608.2c");
    // "Whenever a land you control enters, ~ deals 1 damage to any target. If that land is
    // a Mountain, ~ deals 2 damage instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akoum Hellkite");
    let mountain = t.hand(P0, "Mountain");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    if let Err(e) = t.play_land(P0, mountain) {
        panic!("play: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}

#[test]
fn arashin_war_beast_triggers_once_for_two_blockers() {
    cr!("603.2c", "510.2");
    ruling!(
        "Arashin War Beast",
        "will trigger only once per combat damage step, no matter how many creatures are blocking it"
    );
    // (Kept as it was: once per source; checked because the rendering changed.)
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Arashin War Beast");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.attack(&[(beast, Entity::Player(P1))], &[(a, beast), (b, beast)]);
    t.resolve_all();
    // One manifested card (a face-down 2/2).
    let face_down = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.obj_now(**o).face_down)
        .count();
    assert_eq!(face_down, 1, "{}", t.dump_log());
}


/// Whether a card named `name` was revealed this turn.
fn revealed(t: &TestGame, name: &str) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name: n, obj: Some(o), .. }
            if n == mtg_engine::reveal::REVEALED && t.g.obj(*o).name() == name)
    })
}

fn in_library(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.find_in_zone(Zone::Library(p), name).len()
}

#[test]
fn colossus_is_revealed_and_shuffled_into_its_library_when_sacrificed_as_a_cost() {
    cr!("701.20a", "614.1a", "614.6");
    // Darksteel Colossus: "If ~ would be put into a graveyard from anywhere, reveal ~ and
    // shuffle it into its owner's library instead." The reveal was dropped. Sacrificed to
    // pay a cost, it's still sacrificed (the cost is paid) and goes to the library.
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Viscera Seer");
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let before = in_library(&t, P0, "Darksteel Colossus");
    t.answer_choose(P0, &[Entity::Object(colossus)]);
    t.activate(P0, seer, 0, &[]).expect("the cost can be paid");
    assert!(!t.on_battlefield(colossus));
    assert_eq!(in_library(&t, P0, "Darksteel Colossus"), before + 1);
    assert!(!t.in_graveyard(P0, "Darksteel Colossus"));
    assert!(revealed(&t, "Darksteel Colossus"), "{}", t.dump_log());
    // The ability (Scry 1) resolves: the cost was paid.
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

#[test]
fn progenitus_is_revealed_and_shuffled_into_its_library_when_it_dies() {
    cr!("701.20a", "614.1a", "700.4");
    let mut t = TestGame::new(2);
    let progenitus = t.battlefield(P0, "Progenitus");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(!t.on_battlefield(progenitus));
    assert!(!t.in_graveyard(P0, "Progenitus"));
    assert_eq!(in_library(&t, P0, "Progenitus"), 1);
    assert!(revealed(&t, "Progenitus"), "{}", t.dump_log());
}

#[test]
fn nexus_of_fate_is_revealed_and_shuffled_into_its_library_as_it_resolves() {
    cr!("701.20a", "614.1a", "608.2n");
    // "Take an extra turn after this one. If ~ would be put into a graveyard from anywhere,
    // reveal ~ and shuffle it into its owner's library instead."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let nexus = t.hand(P0, "Nexus of Fate");
    t.cast(P0, nexus).go();
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Nexus of Fate"));
    assert_eq!(in_library(&t, P0, "Nexus of Fate"), 1);
    assert!(revealed(&t, "Nexus of Fate"), "{}", t.dump_log());
}

#[test]
fn animal_magnetism_puts_the_card_the_opponent_chose_onto_the_battlefield() {
    cr!("608.2c", "110.2a");
    // "Reveal the top five cards of your library. An opponent chooses a creature card from
    // among them. Put that card onto the battlefield and the rest into your graveyard."
    // (Rendered "an opponent puts ..."; checked that the choice only picks the card.)
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let elves = t.library_top(P0, "Llanowar Elves");
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    t.lands(P0, "Forest", 5);
    let spell = t.hand(P0, "Animal Magnetism");
    t.answer_choose(P1, &[Entity::Object(elves)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let on_bf: Vec<_> = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.obj_now(**o).name() == "Llanowar Elves")
        .collect();
    assert_eq!(on_bf.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(*on_bf[0]).controller, P0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.find_in_zone(Zone::Graveyard(P0), "Forest").len(), 3);
}
