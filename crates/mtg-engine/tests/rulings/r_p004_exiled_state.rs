//! Rulings batch P004 — what happens to a permanent exiled by a banishing or flickering
//! effect: Auras go to the graveyard (CR 704.5m), Equipment stays unattached (704.5n),
//! counters cease to exist (122.2), and the card returns as a new object (400.7).

use crate::r_p004_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// `p`'s Grizzly Bears with a +1/+1 counter, `p`'s Holy Strength and Bonesplitter attached.
/// Returns (bears, aura, equipment).
fn decorated_bears(t: &mut TestGame, p: PlayerId) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(p, "Grizzly Bears");
    let aura = t.battlefield(p, "Holy Strength");
    assert!(t.g.attach(aura, obj(bears)));
    let eq = t.battlefield(p, "Bonesplitter");
    assert!(t.g.attach(eq, obj(bears)));
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 5));
    (bears, aura, eq)
}

/// Asserts the state right after the decorated Bears was exiled.
fn assert_cleaned_up(t: &TestGame, who: &str, p: PlayerId, bears: ObjectId, eq: ObjectId) {
    assert!(t.in_exile("Grizzly Bears"), "{who}: not exiled");
    assert!(!t.on_battlefield(bears), "{who}");
    assert!(t.in_graveyard(p, "Holy Strength"), "{who}: the Aura stayed");
    assert!(t.on_battlefield(eq), "{who}: the Equipment left");
    assert_eq!(t.obj_now(eq).attached_to, None, "{who}: still attached");
}

/// Asserts the Bears came back as a new object without counters or attachments.
fn assert_new_object(t: &TestGame, who: &str, bears: ObjectId, eq: ObjectId) {
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{who}: didn't return");
    assert_ne!(back[0], bears, "{who}: same object");
    assert_eq!(t.counters(back[0], counters::PLUS1), 0, "{who}: kept its counter");
    assert_eq!(t.pt(back[0]), (2, 2), "{who}");
    assert_eq!(t.obj_now(eq).attached_to, None, "{who}: reattached");
}

#[test]
fn banished_permanents_lose_auras_equipment_and_counters_and_return_as_new_objects() {
    cr!("610.3", "400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Chained to the Rocks",
        "Auras attached to the exiled creature will be put into their owners' graveyards (unless they have bestow). Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Glass Casket",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Gelatinous Cube",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Werefox Bodyguard",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Dimensional Exile",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Fairgrounds Warden",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Faith Unbroken",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Silkwrap",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Colossal Whale",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Stasis Snare",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Hixus, Prison Warden",
        "Auras attached to the exiled creatures will be put into their owners' graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    ruling!(
        "Cast Out",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Ixalan's Binding",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Borrowed Time",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Leyline Binding",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "In the Trenches",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Static Net",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Hieromancer's Cage",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Thopter Arrest",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Banishing Light",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Citizen's Arrest",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Prison Realm",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Seam Rip",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Stormplain Detainment",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    ruling!(
        "Angel of Sanctions",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Henchbots",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    for name in [
        "Chained to the Rocks",
        "Glass Casket",
        "Gelatinous Cube",
        "Werefox Bodyguard",
        "Dimensional Exile",
        "Fairgrounds Warden",
        "Faith Unbroken",
        "Silkwrap",
        "Colossal Whale",
        "Stasis Snare",
        "Hixus, Prison Warden",
        "Cast Out",
        "Ixalan's Binding",
        "Borrowed Time",
        "Leyline Binding",
        "In the Trenches",
        "Static Net",
        "Hieromancer's Cage",
        "Thopter Arrest",
        "Banishing Light",
        "Citizen's Arrest",
        "Prison Realm",
        "Seam Rip",
        "Stormplain Detainment",
        "Angel of Sanctions",
        "Henchbots",
    ] {
        let mut t = TestGame::new(2);
        let (bears, _aura, eq) = decorated_bears(&mut t, P1);
        if name == "Henchbots" {
            t.g.tap(bears);
        }
        if name == "Angel of Sanctions" {
            t.answer_yes(P0, true);
        }
        let b = banish_trigger(&mut t, name, bears);
        t.resolve_all();
        assert_cleaned_up(&t, name, P1, bears, eq);
        destroy_now(&mut t, b);
        t.g.flush_events();
        t.settle();
        assert_new_object(&t, name, bears, eq);
    }
}

#[test]
fn flickered_permanents_lose_auras_equipment_and_counters() {
    cr!("400.7", "704.5m", "704.5n", "122.2", "610.3");
    ruling!(
        "Personify",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Wispweaver Angel",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    ruling!(
        "Ghostway",
        "Auras attached to the exiled creatures will be put into their owners' graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    ruling!(
        "Morningtide's Light",
        "Auras attached to the exiled creatures will be put into their owners' graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    ruling!(
        "Identity Thief",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    // Personify and Wispweaver Angel: exile, then return right away.
    for name in ["Personify", "Wispweaver Angel"] {
        let mut t = TestGame::new(2);
        let (bears, _aura, eq) = decorated_bears(&mut t, P0);
        if name == "Personify" {
            cast_now(&mut t, P0, name, &[obj(bears)]);
        } else {
            t.answer_targets(P0, &[obj(bears)]);
            t.answer_yes(P0, true);
            t.enter(P0, name);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Holy Strength"), "{name}");
        assert_new_object(&t, name, bears, eq);
    }
    // Ghostway and Morningtide's Light: exiled until the next end step.
    for name in ["Ghostway", "Morningtide's Light"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let (bears, _aura, eq) = decorated_bears(&mut t, P0);
        let targets: &[Entity] = if name == "Ghostway" { &[] } else { &[obj(bears)] };
        cast_now(&mut t, P0, name, targets);
        t.resolve_all();
        assert_cleaned_up(&t, name, P0, bears, eq);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_new_object(&t, name, bears, eq);
    }
    // Identity Thief: exiled until the next end step.
    let mut t = TestGame::new(2);
    let (bears, _aura, eq) = decorated_bears(&mut t, P1);
    let thief = t.battlefield(P0, "Identity Thief");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    crate::r_s01_common::attack_with(&mut t, &[(thief, Entity::Player(P1))]);
    t.resolve_all();
    assert_cleaned_up(&t, "Identity Thief", P1, bears, eq);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // (Identity Thief itself is a copy of Grizzly Bears until end of turn.)
    let thief = t.g.current(thief);
    let back: Vec<_> = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .filter(|&b| b != thief)
        .collect();
    assert_eq!(back.len(), 1, "didn't return");
    assert_ne!(back[0], bears);
    assert_eq!(t.counters(back[0], counters::PLUS1), 0);
    assert_eq!(t.obj_now(eq).attached_to, None);
}

#[test]
fn every_permanent_exiled_by_a_multi_exile_banisher_is_cleaned_up() {
    cr!("610.3", "704.5m", "704.5n", "122.2");
    ruling!(
        "Deputy of Detention",
        "Auras attached to the exiled permanents will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist."
    );
    ruling!(
        "Bronzebeak Foragers",
        "Auras attached to the exiled permanents will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist. When the cards return to the battlefield, they will be new objects with no connections to the cards that were exiled."
    );
    // Deputy of Detention: the target and the other permanent with the same name.
    let mut t = TestGame::new(2);
    let (bears, _, eq) = decorated_bears(&mut t, P1);
    let other = t.battlefield(P1, "Grizzly Bears");
    let aura2 = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(aura2, obj(other)));
    t.g.add_counters(obj(other), counters::PLUS1, 2, None);
    let dep = banish_trigger(&mut t, "Deputy of Detention", bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.g.player(P1).graveyard.len(), 2, "both Auras");
    assert!(t.on_battlefield(eq));
    assert_eq!(t.obj_now(eq).attached_to, None);
    destroy_now(&mut t, dep);
    t.g.flush_events();
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 2);
    for b in back {
        assert_eq!(t.counters(b, counters::PLUS1), 0);
    }
    // Bronzebeak Foragers: one target per opponent.
    let mut t = TestGame::new(3);
    let (bears, _, eq) = decorated_bears(&mut t, P1);
    let giant = t.battlefield(P2, "Hill Giant");
    t.g.add_counters(obj(giant), counters::PLUS1, 1, None);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    let f = t.enter(P0, "Bronzebeak Foragers");
    t.resolve_all();
    assert_cleaned_up(&t, "Bronzebeak Foragers", P1, bears, eq);
    assert!(t.in_exile("Hill Giant"));
    destroy_now(&mut t, f);
    t.g.flush_events();
    t.settle();
    assert_new_object(&t, "Bronzebeak Foragers", bears, eq);
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.counters(g[0], counters::PLUS1), 0);
    assert_eq!(t.obj_now(g[0]).controller, P2);
}
