//! cards5-referents-statics, family `history`: what happened earlier in the game, read by
//! an ability (CR 603.10a, 700.4).

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

#[test]
fn krovikan_vampire_returns_the_creatures_it_damaged_that_died() {
    cr!("603.4", "603.10a", "700.4");
    ruling!(
        "Krovikan Vampire",
        "it doesn’t matter whether the damage was what caused the creatures to die"
    );
    supported("Krovikan Vampire");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // The Giant survives 1 damage from the Vampire, then is destroyed; the Elves die
    // without being dealt damage by it.
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.g.deal_damage(kv, Entity::Object(bears), 2, false);
    t.settle();
    t.g.destroy(giant, None);
    t.g.destroy(elves, None);
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for (name, back) in [("Hill Giant", true), ("Grizzly Bears", true), ("Llanowar Elves", false)] {
        let on: Vec<ObjectId> = t.named_on_battlefield(name);
        assert_eq!(on.len(), usize::from(back), "{name}");
        if back {
            assert_eq!(t.obj_now(on[0]).controller, P0, "{name}");
        }
    }
    // Losing control of the Vampire (it leaves the battlefield): they're sacrificed.
    t.g.destroy(kv, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn krovikan_vampire_does_not_trigger_without_such_a_death() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.settle();
    let stack_before = t.stack_len();
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), stack_before);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P1);
}
