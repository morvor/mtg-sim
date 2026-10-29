//! Referents of a cost and of another ability (cards4-paper-effects, family
//! `referents`): "the exiled card" of an "Exile the top card of your library" cost,
//! "the exiled creature" of a spell's additional cost, and "the token" another of the
//! card's abilities created.

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn storm_elemental_pumps_only_if_the_exiled_card_is_a_snow_land() {
    cr!("400.7j", "602.2");
    ruling!(
        "Storm Elemental",
        "If you activate Storm Elemental’s third ability and the exiled card is not a snow land, the ability has no effect."
    );
    supported("Storm Elemental");
    for (top, pt) in [("Snow-Covered Forest", (4, 5)), ("Forest", (3, 4))] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let s = t.battlefield(P0, "Storm Elemental");
        t.lands(P0, "Island", 1);
        t.library_top(P0, top);
        t.activate(P0, s, 1, &[]).unwrap();
        t.resolve();
        assert!(t.in_exile(top), "{top}");
        assert_eq!(t.pt(s), pt, "{top}");
    }
}

#[test]
fn soul_exchange_counter_only_if_the_exiled_creature_was_a_thrull() {
    cr!("400.7j", "601.2b", "608.2h");
    supported("Soul Exchange");
    for (fodder, counters) in [("Ostiary Thrull", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let spell = t.hand(P0, "Soul Exchange");
        t.lands(P0, "Swamp", 3);
        let f = t.battlefield(P0, fodder);
        let back = t.graveyard(P0, "Hill Giant");
        t.answer_choose(P0, &[Entity::Object(f)]);
        t.cast(P0, spell).target(Entity::Object(back)).go();
        t.resolve();
        assert!(t.in_exile(fodder), "{fodder}");
        let giant = t.named_on_battlefield("Hill Giant");
        assert_eq!(giant.len(), 1);
        assert_eq!(t.counters(giant[0], "+2/+2"), counters, "{fodder}");
    }
}

fn dance(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[Entity::Object(target)]);
    let d = t.enter(P0, "Dance of Many");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    d
}

fn tokens(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.g.obj(*o).is_token())
        .collect()
}

#[test]
fn dance_of_many_exiles_its_token_when_it_leaves() {
    cr!("607.1d", "603.6c");
    ruling!(
        "Dance of Many",
        "Each Dance of Many is associated only with its token creature. If one Dance leaves the battlefield, only the corresponding token is affected, not all tokens from all instances of Dance of Many."
    );
    supported("Dance of Many");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let d1 = dance(&mut t, bears);
    let _d2 = dance(&mut t, giant);
    assert_eq!(tokens(&t, "Grizzly Bears").len(), 1);
    assert_eq!(tokens(&t, "Hill Giant").len(), 1);
    t.g.destroy(d1, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(tokens(&t, "Grizzly Bears").is_empty());
    assert_eq!(tokens(&t, "Hill Giant").len(), 1);
    assert!(t.on_battlefield(bears));
}

#[test]
fn dance_of_many_is_sacrificed_when_its_token_leaves() {
    cr!("607.1d", "603.6c");
    ruling!(
        "Dance of Many",
        "Destroying the creature that was copied will not cause the token creature to be destroyed as well."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let d = dance(&mut t, bears);
    // The original dying doesn't matter.
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(d));
    let tok = tokens(&t, "Grizzly Bears");
    assert_eq!(tok.len(), 1);
    t.g.destroy(tok[0], None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(d));
    assert!(t.in_graveyard(P0, "Dance of Many"));
}
