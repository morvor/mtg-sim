//! cards5-referents-statics, family `permissions-attach`: the permanent an Aura became
//! attached to as the referent of the trigger (CR 603.2, 611.2b).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// P0 (with Eriette) casts `aura` on P1's Grizzly Bears (mana value 2).
fn eriette(aura: &str, lands: &str, n: usize) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Eriette, the Beguiler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, lands, n);
    let a = t.hand(P0, aura);
    t.cast(P0, a).target(Entity::Object(bears)).go();
    t.resolve_all();
    let a = t.named_on_battlefield(aura)[0];
    (t, bears, a)
}

#[test]
fn eriette_steals_what_an_aura_of_greater_or_equal_mana_value_enchants() {
    cr!("611.2b", "603.2");
    supported("Eriette, the Beguiler");
    // Pacifism (2) on the Bears (2): P0 controls them while it's attached.
    let (mut t, bears, pacifism) = eriette("Pacifism", "Plains", 2);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.g.destroy(pacifism, None);
    t.settle();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
    // Rancor (1) on the Bears: nothing.
    let (t, bears, _) = eriette("Rancor", "Forest", 1);
    assert_eq!(t.obj_now(bears).controller, P1);
}

/// P0 casts Finale of Promise for X targeting Lightning Bolt and Divination in their
/// graveyard; the Bolt (and its copies) at P1.
fn finale(x: i64) -> TestGame {
    use mtg_engine::decision::Answer;
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    for _ in 0..10 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Mountain", x as usize + 2);
    let f = t.hand(P0, "Finale of Promise");
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.cast(P0, f)
        .target(Entity::Object(bolt))
        .target(Entity::Object(div))
        .go();
    t.answer_yes(P0, true);
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.resolve_all();
    t
}

#[test]
fn finale_of_promise_casts_both_and_exiles_them_and_copies_for_ten() {
    cr!("608.2g", "707.10", "614.1a");
    supported("Finale of Promise");
    let t = finale(3);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Divination"));
    // X = 10: each is copied twice.
    let t = finale(10);
    assert_eq!(t.life(P1), 20 - 9);
    assert_eq!(t.hand_size(P0), 6);
}
