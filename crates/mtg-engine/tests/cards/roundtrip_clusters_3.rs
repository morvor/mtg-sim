//! roundtrip-clusters-3 (docs/agents/items/roundtrip-clusters-3.md): parser bugs the round
//! trip found in its clusters, and cards whose rendering differs in wording, checked in play.

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

