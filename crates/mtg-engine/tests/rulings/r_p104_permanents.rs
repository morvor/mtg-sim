//! Rulings batch P104 — planeswalkers and permanents that mill: Estrid, the Masked (her
//! Mask tokens and her ultimate), Grist's reflexive -2, The Master of Keys' escape, and
//! Abhorrent Oculus's additional cost.

use crate::r_p104_common::*;
use crate::r_p177_common::set_loyalty;
use crate::r_p189_common::bounce;
use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s06_common::attach_new;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn estrid(t: &mut TestGame, loyalty: u32) -> ObjectId {
    supported("Estrid, the Masked");
    let e = t.battlefield(P0, "Estrid, the Masked");
    set_loyalty(t, e, loyalty);
    e
}

fn masks(t: &TestGame) -> Vec<ObjectId> {
    t.named_on_battlefield("Mask")
}

/// Estrid's -1 targeting `target`; returns the new Mask (if any).
fn mask_on(t: &mut TestGame, e: ObjectId, target: ObjectId) -> Option<ObjectId> {
    let before = masks(t);
    let cur = t.g.current(e);
    t.g.objects[cur.0 as usize].activations_this_turn.clear();
    t.activate(P0, e, 1, &[obj(target)]).unwrap();
    t.resolve_all();
    masks(t).into_iter().find(|m| !before.contains(m))
}

#[test]
fn estrid_plus_two_untaps_only_enchanted_permanents() {
    cr!("303.4", "701.26b");
    ruling!(
        "Estrid, the Masked",
        "An enchanted permanent is one with an Aura attached to it. Enchantment permanents you control that don't have Auras attached to them aren't enchanted."
    );
    let mut t = TestGame::new(2);
    let e = estrid(&mut t, 3);
    let a = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Rancor", obj(a));
    let b = t.battlefield(P0, "Grizzly Bears");
    let ench = t.battlefield(P0, "Glorious Anthem");
    for id in [a, b, ench] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    t.activate(P0, e, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(a).tapped);
    assert!(t.obj_now(b).tapped);
    assert!(t.obj_now(ench).tapped);
}

#[test]
fn estrid_mask_cant_target_estrid() {
    cr!("115.1", "702.5a");
    ruling!(
        "Estrid, the Masked",
        "Estrid's second ability can't target Estrid, but the Mask token may legally enchant Estrid if you find a way to move it onto her."
    );
    let mut t = TestGame::new(2);
    let e = estrid(&mut t, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let mask = mask_on(&mut t, e, bears).expect("a Mask");
    let cands = target_candidates(&t, P0, from);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&obj(e))));
    assert!(cands[0].contains(&obj(bears)));
    // The Mask can enchant Estrid (enchant permanent).
    assert!(t.g.attach(mask, obj(e)));
    t.settle();
    assert_eq!(t.obj_now(mask).attached_to, Some(obj(t.g.current(e))));
    assert!(t.on_battlefield(mask));
}

#[test]
fn estrid_mask_isnt_created_if_it_cant_be_attached() {
    cr!("303.4f", "702.16c");
    ruling!(
        "Estrid, the Masked",
        "Estrid's second ability might target a permanent that can't have the Aura token attached to it. In this case, the token isn't created at all."
    );
    supported("Azorius First-Wing");
    let mut t = TestGame::new(2);
    let e = estrid(&mut t, 3);
    let wing = t.battlefield(P1, "Azorius First-Wing");
    assert_eq!(mask_on(&mut t, e, wing), None);
    assert!(t.g.permanents().all(|o| !o.is_token()));
    // No token was created (nothing could see one being created).
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::TokenCreated { .. })));
}

#[test]
fn stacked_umbra_armor_masks_only_the_outer_one_is_destroyed() {
    cr!("702.89a", "616.1");
    ruling!(
        "Estrid, the Masked",
        "If an Aura with umbra armor is itself enchanted by another Aura with umbra armor, destroying either the first Aura or the permanent it enchants will result in only the second Aura being destroyed instead."
    );
    for destroy_creature in [true, false] {
        let mut t = TestGame::new(2);
        let e = estrid(&mut t, 5);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let inner = mask_on(&mut t, e, bears).expect("first Mask");
        let outer = mask_on(&mut t, e, inner).expect("second Mask");
        assert_eq!(t.obj_now(outer).attached_to, Some(obj(inner)));
        destroy(&mut t, if destroy_creature { bears } else { inner });
        assert!(t.on_battlefield(bears));
        assert!(t.on_battlefield(inner));
        assert!(!t.g.is_live(outer) || !t.on_battlefield(outer));
    }
}

/// Estrid's -7 with the cards `gy` in P0's graveyard (nothing useful is milled).
fn estrid_ultimate(t: &mut TestGame, gy: &[&str]) -> Vec<ObjectId> {
    let e = estrid(t, 7);
    let ids = gy.iter().map(|n| t.graveyard(P0, n)).collect();
    t.activate(P0, e, 2, &[]).unwrap();
    t.resolve();
    ids
}

#[test]
fn estrid_ultimate_enchantments_enter_together_and_see_each_other() {
    cr!("603.6a", "603.2");
    ruling!(
        "Estrid, the Masked",
        "All of the non-Aura enchantments put onto the battlefield with Estrid's third ability enter at the same time. If any have triggered abilities that trigger on something else entering the battlefield, they'll see each other."
    );
    supported("Eidolon of Blossoms");
    let mut t = TestGame::new(2);
    let ids = estrid_ultimate(&mut t, &["Eidolon of Blossoms", "Eidolon of Blossoms"]);
    for id in &ids {
        assert!(t.on_battlefield(*id));
    }
    // Each Eidolon triggers for itself and for the other.
    assert_eq!(triggers_on_stack(&t, "draw a card"), 4);
}

#[test]
fn estrid_ultimate_auras_may_enchant_the_returned_enchantments() {
    cr!("303.4f");
    ruling!(
        "Estrid, the Masked",
        "Aura cards returned with Estrid's third ability may enchant non-Aura enchantments put onto the battlefield with that ability."
    );
    let mut t = TestGame::new(2);
    let ids = estrid_ultimate(&mut t, &["Eidolon of Blossoms", "Rancor"]);
    t.resolve_all();
    assert!(t.on_battlefield(ids[0]));
    assert!(t.on_battlefield(ids[1]));
    assert_eq!(t.obj_now(ids[1]).attached_to, Some(obj(t.g.current(ids[0]))));
}

#[test]
fn grist_minus_two_works_without_grist() {
    cr!("603.12", "113.7a");
    ruling!(
        "Grist, the Hunger Tide",
        "Grist's second loyalty ability doesn't require a target. If you choose to sacrifice a creature as it resolves, the reflexive triggered ability triggers and you'll choose a target creature or planeswalker to destroy. This will happen even if Grist isn't on the battlefield when the loyalty ability resolves."
    );
    supported("Grist, the Hunger Tide");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grist, the Hunger Tide");
    set_loyalty(&mut t, g, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.activate(P0, g, 1, &[]).unwrap();
    // No target was chosen on activation.
    assert!(target_candidates(&t, P0, from).is_empty());
    bounce(&mut t, g);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn escaped_enchantment_returns_to_the_graveyard_and_can_escape_again() {
    cr!("702.138a", "702.138b");
    ruling!(
        "The Master of Keys",
        "After an escaped spell resolves, it returns to its owner's graveyard if it's not a permanent spell. If it is a permanent spell, it enters and will return to its owner's graveyard if it dies later. It can escape again."
    );
    supported("The Master of Keys");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Master of Keys");
    let anthem = t.graveyard(P0, "Glorious Anthem");
    for _ in 0..6 {
        t.graveyard(P0, "Island");
    }
    t.lands(P0, "Plains", 6);
    let m = legal_cast_methods(&mut t, P0, anthem);
    assert_eq!(m.len(), 1, "{m:?}");
    t.cast(P0, anthem).method(m[0].clone()).go();
    assert_eq!(t.g.exile.len(), 3);
    t.resolve_all();
    assert!(t.on_battlefield(anthem));
    destroy(&mut t, anthem);
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    let again = t.g.find_in_zone(Zone::Graveyard(P0), "Glorious Anthem")[0];
    assert_eq!(legal_cast_methods(&mut t, P0, again).len(), 1);
}

#[test]
fn escaping_a_room_casts_one_half_plus_the_exile_cost() {
    cr!("709.3", "709.3a", "702.138a");
    ruling!(
        "The Master of Keys",
        "If you're casting a Room with escape, you choose which half to cast, then pay the appropriate cost plus exiling three cards. The same is true for split cards, adventurer cards, and prototype cards."
    );
    supported("Bottomless Pool // Locker Room");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Master of Keys");
    let room = t.graveyard(P0, "Bottomless Pool // Locker Room");
    for _ in 0..3 {
        t.graveyard(P0, "Island");
    }
    t.lands(P0, "Island", 1);
    let m = legal_cast_methods(&mut t, P0, room);
    assert!(!m.is_empty(), "{m:?}");
    // With one Island, only the {U} half (Bottomless Pool) is castable.
    t.cast(P0, room).method(m[0].clone()).go();
    assert_eq!(t.g.exile.len(), 3);
    t.resolve_all();
    assert!(t.on_battlefield(room));
    let room_now = t.g.current(room);
    assert_eq!(mtg_engine::rooms::unlocked(&t.g, room_now), [true, false]);
}

#[test]
fn abhorrent_oculus_additional_cost_applies_to_free_casts() {
    cr!("118.9d", "601.2f");
    ruling!(
        "Abhorrent Oculus",
        "Abhorrent Oculus's additional cost must be paid even if it's cast \"without paying its mana cost\" or for any alternative cost."
    );
    supported("Abhorrent Oculus");
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let o = t.hand(P0, "Abhorrent Oculus");
    for _ in 0..5 {
        t.graveyard(P0, "Island");
    }
    assert!(legal_cast_methods(&mut t, P0, o).is_empty());
    t.graveyard(P0, "Island");
    let m = legal_cast_methods(&mut t, P0, o);
    assert!(!m.is_empty());
    t.cast(P0, o).method(m[0].clone()).go();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.exile.len(), 6);
}
