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
