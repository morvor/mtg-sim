//! CR 607.5: abilities an object acquires from a continuously reevaluated effect (a
//! static ability's "have", "has all activated abilities of") are linked only to the
//! abilities acquired with them: "They can't be linked to any other ability, regardless of
//! what other abilities the object may currently have or may have had in the past."
//!
//! Their links used to be hashes of the ability's link and the granting objects' ids into
//! 0x8000–0xbfff, so two grants from objects whose ids differ by 0x4000 got the same link.
//! They're now allocated per object acquiring them (`acquired_links`), so these tests put
//! the second source 0x4000 ids after the first.

use mtg_engine::acquired_links::ACQUIRED_LINK_BASE;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Makes the next object's id `id` by adding objects that are in no zone.
fn next_id_is(t: &mut TestGame, id: u32) {
    let mut filler = t.g.objects[0].clone();
    filler.zone = Zone::Nowhere;
    filler.next = None;
    while (t.g.objects.len() as u32) < id {
        filler.id = ObjectId(t.g.objects.len() as u32);
        t.g.objects.push(filler.clone());
    }
}

/// The links of `id`'s abilities whose text contains `text`, in order.
fn links(t: &mut TestGame, id: ObjectId, text: &str) -> Vec<u16> {
    t.g.recompute();
    t.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(text))
        .map(|a| a.link)
        .collect()
}

fn acquired(l: u16) -> bool {
    (ACQUIRED_LINK_BASE..0xc000).contains(&l)
}

#[test]
fn abilities_two_objects_give_one_object_have_distinct_links() {
    cr!("607.5", "613.1f");
    let mut t = TestGame::new(2);
    // "Creatures you control have "{T}: Add one mana of any color.""
    let first = t.battlefield(P0, "Cryptolith Rite");
    next_id_is(&mut t, first.0 + 0x4000);
    let second = t.battlefield(P0, "Cryptolith Rite");
    assert_eq!(second.0, first.0 + 0x4000);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let l = links(&mut t, bears, "Add one mana of any color");
    assert_eq!(l.len(), 2, "{l:?}");
    assert!(l.iter().all(|x| acquired(*x)), "{l:?}");
    assert_ne!(l[0], l[1], "one grant from each Rite, not linked to each other");
    // The same each time characteristics are computed.
    assert_eq!(links(&mut t, bears, "Add one mana of any color"), l);
    // A new object starts over (CR 400.7) and keeps the abilities apart too.
    let back = t.g.move_object(bears, Zone::Hand(P0), MoveCause::Effect, None);
    let back = back.expect("returned to hand");
    let again = t
        .g
        .move_object(back, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .expect("back on the battlefield");
    let l2 = links(&mut t, again, "Add one mana of any color");
    assert_eq!(l2.len(), 2);
    assert_ne!(l2[0], l2[1]);
}

#[test]
fn abilities_gained_from_two_objects_keep_their_links_apart() {
    cr!("607.5", "613.1f");
    let mut t = TestGame::new(2);
    // "As long as this creature is on the battlefield, it has all activated abilities of
    // all creature cards in all graveyards."
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    // "{B}{G}: Exile target creature blocking or blocked by Sisters of Stone Death." /
    // "{2}{B}: Put a creature card exiled with Sisters of Stone Death onto the battlefield
    // under your control." (linked, CR 607.2a)
    let first = t.graveyard(P0, "Sisters of Stone Death");
    next_id_is(&mut t, first.0 + 0x4000);
    let second = t.graveyard(P1, "Sisters of Stone Death");
    assert_eq!(second.0, first.0 + 0x4000);
    let exile = links(&mut t, ooze, "Exile target creature blocking");
    let put = links(&mut t, ooze, "Put a creature card exiled with");
    assert_eq!((exile.len(), put.len()), (2, 2), "{exile:?} {put:?}");
    assert!(exile.iter().chain(&put).all(|x| acquired(*x)));
    // Each Sisters' pair stays linked on the Ooze, and only to each other.
    assert_eq!(exile[0], put[0]);
    assert_eq!(exile[1], put[1]);
    assert_ne!(exile[0], exile[1]);
}
