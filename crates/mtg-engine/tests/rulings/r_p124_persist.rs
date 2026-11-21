//! Rulings batch P124 — persist (CR 702.79): +1/+1 and -1/-1 counters annihilating (CR
//! 704.5q), the look-back at the creature as it last existed (CR 603.10a), several
//! instances, and the returned card being a new object (CR 400.7).

use crate::r_p124_common::*;
use crate::r_s06_common::attached_to;
use crate::r_s17_common::enter_transformed;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters::{MINUS1, PLUS1};
use mtg_engine::*;

/// The creature (followed across zone changes) dies by an effect that destroys it.
fn kill(t: &mut TestGame, id: ObjectId) {
    destroy(t, id);
    t.resolve_all();
}

#[test]
fn counters_annihilate_so_persist_can_return_it_again() {
    cr!("704.5q", "702.79a", "122.3");
    ruling!(
        "Obstinate Gargoyle",
        "If a creature has +1/+1 counters and -1/-1 counters on it, state-based actions remove the same number of each so that it has only one kind of those counters on it. A creature's persist ability can bring it back again if its -1/-1 counters are removed this way."
    );
    // Obstinate Gargoyle (persist) and, under Isilu, Carrier of Twilight ("Each other
    // nontoken creature you control has persist."), Grizzly Bears.
    for (giver, creature) in [
        (None, "Obstinate Gargoyle"),
        (Some("Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight"), "Grizzly Bears"),
    ] {
        let mut t = TestGame::new(2);
        if let Some(g) = giver {
            supported(g);
            enter_transformed(&mut t, P0, g);
        }
        let id = t.battlefield(P0, creature);
        kill(&mut t, id);
        let id = t.g.current(id);
        assert!(t.on_battlefield(id), "{creature}: persisted");
        assert_eq!(t.counters(id, MINUS1), 1);
        give_plus1(&mut t, id, 1);
        assert_eq!((t.counters(id, MINUS1), t.counters(id, PLUS1)), (0, 0));
        kill(&mut t, id);
        let id = t.g.current(id);
        assert!(t.on_battlefield(id), "{creature}: persisted again");
        assert_eq!(t.counters(id, MINUS1), 1);
    }
}

#[test]
fn lethal_minus1_counters_on_a_creature_with_plus1_counters_dont_let_it_persist() {
    cr!("704.5q", "704.3", "702.79a", "603.10a");
    ruling!(
        "Lesser Masticore",
        "If a creature with persist that has +1/+1 counters on it receives enough -1/-1 counters to cause it to be destroyed by lethal damage or put into its owner's graveyard for having 0 or less toughness, persist won't trigger and the card won't return to the battlefield."
    );
    // A 2/2 Obstinate Gargoyle with a +1/+1 counter gets four -1/-1 counters: it's put
    // into the graveyard at the same time the counters would annihilate.
    for name in ["Obstinate Gargoyle", "Lesser Masticore"] {
        supported(name);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        give_plus1(&mut t, id, 1);
        give_minus1(&mut t, id, 4);
        t.resolve_all();
        assert!(t.named_on_battlefield(name).is_empty(), "{name}");
        assert!(t.in_graveyard(P0, name), "{name}");
    }
}

#[test]
fn several_instances_of_persist_return_it_once() {
    cr!("702.79a", "603.2c", "400.7");
    ruling!(
        "Cauldron Haze",
        "If a creature has multiple instances of persist, the result is largely the same as having just one instance of persist."
    );
    ruling!(
        "Antler Skulkin",
        "If a creature has multiple instances of persist, the result is largely the same as having just one instance of persist."
    );
    // Kitchen Finks (green-white, persist) gains another persist.
    for card in ["Cauldron Haze", "Antler Skulkin"] {
        supported(card);
        let mut t = TestGame::new(2);
        let finks = t.battlefield(P0, "Kitchen Finks");
        if card == "Antler Skulkin" {
            let skulkin = t.battlefield(P0, card);
            t.lands(P0, "Wastes", 2);
            t.activate(P0, skulkin, 0, &[obj(finks)]).unwrap();
            t.resolve_all();
        } else {
            t.answer_targets(P0, &[obj(finks)]);
            cast_resolve(&mut t, P0, card, &[]);
        }
        destroy(&mut t, finks);
        assert_eq!(t.stack_len(), 2, "{card}: two persist triggers");
        t.resolve_all();
        let all = t.named_on_battlefield("Kitchen Finks");
        assert_eq!(all.len(), 1, "{card}: returned once");
        assert_eq!(t.counters(all[0], MINUS1), 1);
    }
}

#[test]
fn a_creature_that_gained_persist_returns_without_it() {
    cr!("702.79a", "400.7", "611.2c");
    ruling!(
        "Antler Skulkin",
        "If a nontoken creature that gains persist this way is put into a graveyard, that card will be returned to the battlefield with a -1/-1 counter on it. However, because it’s a new object with no relation to its previous existence, that permanent will not have persist."
    );
    ruling!(
        "Cauldron Haze",
        "If a nontoken creature that gains persist this way is put into a graveyard, that card will be returned to the battlefield with a -1/-1 counter on it. However, because it’s a new object with no relation to its previous existence, that permanent will not have persist."
    );
    ruling!(
        "Cauldron of Souls",
        "If a nontoken creature that gains persist this way is put into a graveyard, that card will be returned to the battlefield with a -1/-1 counter on it. However, because it’s a new object with no relation to its previous existence, the returned creature will not have persist."
    );
    for card in ["Antler Skulkin", "Cauldron Haze", "Cauldron of Souls"] {
        supported(card);
        let mut t = TestGame::new(2);
        let lions = t.battlefield(P0, "Benalish Knight");
        match card {
            "Cauldron Haze" => {
                t.answer_targets(P0, &[obj(lions)]);
                cast_resolve(&mut t, P0, card, &[]);
            }
            _ => {
                let src = t.battlefield(P0, card);
                t.lands(P0, "Wastes", 2);
                t.activate(P0, src, 0, &[obj(lions)]).unwrap();
                t.resolve_all();
            }
        }
        assert!(has_kw(&mut t, lions, KeywordKind::Persist), "{card}");
        kill(&mut t, lions);
        let back = t.g.current(lions);
        assert!(t.on_battlefield(back), "{card}");
        assert_eq!(t.counters(back, MINUS1), 1);
        assert!(!has_kw(&mut t, back, KeywordKind::Persist), "{card}");
    }
}

#[test]
fn a_persisted_creature_is_a_new_object() {
    cr!("400.7", "702.79a", "704.5m", "704.5n");
    ruling!(
        "Persistent Constrictor",
        "Once persist returns the creature, it's considered a new object with no relation to the object that it was. Auras that were attached to it won't return to the battlefield. Equipment that was attached to it will remain unattached. Any counters that were on it won't be put on the new creature."
    );
    supported("Persistent Constrictor");
    let mut t = TestGame::new(2);
    let snake = t.battlefield(P0, "Persistent Constrictor");
    let splitter = crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", snake);
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", snake);
    give_plus1(&mut t, snake, 2);
    kill(&mut t, snake);
    let back = t.g.current(snake);
    assert!(t.on_battlefield(back));
    assert_ne!(back, snake);
    assert_eq!(
        (t.counters(back, MINUS1), t.counters(back, PLUS1)),
        (1, 0)
    );
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_ne!(t.zone(aura), Zone::Battlefield);
    assert!(t.on_battlefield(splitter));
    assert_eq!(attached_to(&t, splitter), None);
}
