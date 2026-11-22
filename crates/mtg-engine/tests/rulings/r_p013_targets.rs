//! Rulings batch P013 — spells castable from exile or a graveyard whose targets become
//! illegal (CR 608.2b): with every target illegal the spell doesn't resolve (an Adventure
//! then goes to the graveyard, CR 715.4); with one of two, the rest still happens. And
//! Sever the Bloodline's "all other creatures with the same name" (CR 201.2, 111.4).

use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn bounce(t: &mut TestGame, id: ObjectId) {
    let owner = t.obj_now(id).owner;
    crate::r_p127_common::move_to(t, id, Zone::Hand(owner));
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn tapping_two_targets_with_one_illegal_taps_the_other() {
    cr!("608.2b", "715.4", "115.3");
    ruling!(
        "Feeling of Dread",
        "If Feeling of Dread targets two creatures, and one of them is an illegal target by the time Feeling of Dread resolves, the other creature will still be tapped."
    );
    ruling!(
        "Ardenvale Tactician",
        "If two targets are chosen for Dizzying Swoop and one of them becomes an illegal target but not the other, the legal target will be tapped and Dizzying Swoop will be exiled. If both targets are illegal, Dizzying Swoop will be put into your graveyard."
    );
    supported("Feeling of Dread");
    supported("Ardenvale Tactician");
    for card in ["Feeling of Dread", "Ardenvale Tactician"] {
        for both_illegal in [false, true] {
            let mut t = TestGame::new(2);
            let a = t.battlefield(P1, "Grizzly Bears");
            let b = t.battlefield(P1, "Hill Giant");
            t.lands(P0, "Plains", 2);
            let c = t.hand(P0, card);
            let method = if card == "Feeling of Dread" {
                CastMethod::Normal
            } else {
                CastMethod::Half(1)
            };
            t.cast(P0, c).method(method).targets(&[obj(a), obj(b)]).go();
            bounce(&mut t, a);
            if both_illegal {
                bounce(&mut t, b);
            }
            t.resolve_all();
            if !both_illegal {
                assert!(tapped(&t, b), "{card}");
            }
            let expect = match (card, both_illegal) {
                ("Ardenvale Tactician", false) => Zone::Exile,
                _ => Zone::Graveyard(P0),
            };
            assert_eq!(t.zone(c), expect, "{card} both illegal: {both_illegal}");
        }
    }
}

#[test]
fn spells_with_their_only_target_illegal_do_nothing() {
    cr!("608.2b", "701.6");
    ruling!(
        "Risk Factor",
        "If the target player is an illegal target by the time Risk Factor tries to resolve, the spell doesn't resolve. You don't draw three cards."
    );
    ruling!(
        "Traumatic Critique",
        "If the target is illegal as Traumatic Critique tries to resolve, it won't resolve and none of its effects will happen. You won't draw or discard."
    );
    supported("Risk Factor");
    supported("Traumatic Critique");
    // Risk Factor: P1 gains hexproof (Leyline of Sanctity) in response.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Risk Factor");
    let rf = t.hand(P0, "Risk Factor");
    t.cast(P0, rf).target(P1).go();
    t.battlefield(P1, "Leyline of Sanctity");
    let hand = t.hand_size(P0);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 20);
    // Control: without hexproof, P1 declines the damage and P0 draws three.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Risk Factor");
    let rf = t.hand(P0, "Risk Factor");
    t.cast(P0, rf).target(P1).go();
    let hand = t.hand_size(P0);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Traumatic Critique (X = 1) at a creature that leaves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.hand(P0, "Forest");
    let tc = t.hand(P0, "Traumatic Critique");
    t.cast(P0, tc).x(1).target(bears).go();
    bounce(&mut t, bears);
    let (hand, lib) = (t.hand_size(P0), t.library_size(P0));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, lib));
    assert!(t.in_hand(P0, "Forest"));
}

/// P0 casts Sever the Bloodline at `target`.
fn sever(t: &mut TestGame, target: ObjectId) -> ObjectId {
    lands_for_cost(t, P0, "Sever the Bloodline");
    let s = t.hand(P0, "Sever the Bloodline");
    t.cast(P0, s).target(target).go()
}

#[test]
fn sever_the_bloodline_with_an_illegal_target_exiles_nothing() {
    cr!("608.2b");
    ruling!(
        "Sever the Bloodline",
        "If the target creature is an illegal target by the time Sever the Bloodline tries to resolve, the spell won’t resolve. You won’t exile any creatures at all."
    );
    ruling!(
        "Sever the Bloodline",
        "If the targeted creature is an illegal target by the time Sever the Bloodline resolves, it won't resolve and none of its effects will happen. No creatures will be exiled."
    );
    supported("Sever the Bloodline");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    sever(&mut t, a);
    bounce(&mut t, a);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn sever_the_bloodline_exiles_same_named_creatures_on_the_battlefield_only() {
    cr!("608.2c", "201.2", "702.11b", "109.2");
    ruling!(
        "Sever the Bloodline",
        "Only creatures on the battlefield will be exiled. In other zones, they're \"creature cards,\" not \"creatures.\""
    );
    ruling!(
        "Sever the Bloodline",
        "Sever the Bloodline has only one target. Other creatures with the same name will be exiled even if they have hexproof or protection."
    );
    supported("Gladecover Scout");
    // P0 targets their own Gladecover Scout (hexproof); P1's is exiled too, but not the
    // Scouts in a graveyard or hand.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Gladecover Scout");
    let theirs = t.battlefield(P1, "Gladecover Scout");
    let gy = t.graveyard(P1, "Gladecover Scout");
    let hand = t.hand(P1, "Gladecover Scout");
    sever(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.zone(mine), Zone::Exile);
    assert_eq!(t.zone(theirs), Zone::Exile);
    assert_eq!(t.zone(gy), Zone::Graveyard(P1));
    assert_eq!(t.zone(hand), Zone::Hand(P1));
    // Two White Knights (protection from black): one loses its abilities (Turn to Frog)
    // and is targeted; the other is exiled despite its protection.
    supported("White Knight");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "White Knight");
    let b = t.battlefield(P1, "White Knight");
    lands_for_cost(&mut t, P0, "Turn to Frog");
    let frog = t.hand(P0, "Turn to Frog");
    t.cast(P0, frog).target(a).go();
    t.resolve_all();
    sever(&mut t, a);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn sever_the_bloodline_uses_the_name_of_the_face_up_and_token_names() {
    cr!("201.2", "712.8d", "111.4");
    ruling!(
        "Sever the Bloodline",
        "A double-faced creature only has the name of the face that's up. For example, if Village Ironsmith is targeted by Sever the Bloodline, Ironfang wouldn't be exiled."
    );
    ruling!(
        "Sever the Bloodline",
        "Unless a token is a copy of another permanent or was explicitly given a name by the effect that created it, its name is the subtypes it was given when it was created plus the word “Token.”"
    );
    supported("Village Ironsmith");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Village Ironsmith");
    let b = t.battlefield(P1, "Village Ironsmith");
    assert!(mtg_engine::dfc::transform(&mut t.g, b));
    t.g.recompute();
    assert_eq!(t.obj_now(b).chars.name, "Ironfang");
    sever(&mut t, a);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert!(t.on_battlefield(b));
    // Soldier tokens are all named "Soldier Token"; a Goblin token isn't.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P1, "Raise the Alarm");
    let ra = t.hand(P1, "Raise the Alarm");
    t.cast(P1, ra).go();
    t.resolve_all();
    let goblin = crate::r_s02_common::create_token(&mut t, P1, "Goblin");
    let soldiers = t.named_on_battlefield("Soldier Token");
    assert_eq!(soldiers.len(), 2);
    sever(&mut t, soldiers[0]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Soldier Token").is_empty());
    assert!(t.on_battlefield(goblin));
}
