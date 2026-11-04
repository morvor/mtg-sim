//! Rulings batch P005 — attack and block triggers: "while" conditions checked only as the
//! ability triggers (CR 508.1m), targets checked on resolution (CR 608.2b), attaching to
//! an illegal object (CR 701.3b), Augusta's untap-and-tap (CR 506.4b), and Plargg casting
//! during resolution (CR 608.2g).

use crate::r_p079_common::back_face;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s02_common::destroy;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BIG: &str = "Rumbling Baloth"; // vanilla 4/4

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

/// An old permanent of `p` (not summoning sick).
fn old(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].summoning_sick = false;
    t.g.objects[id.0 as usize].entered_turn = 0;
    id
}

/// P1 attacks P0 with `attacker`; P0 blocks it with `blocker`; the declare blockers step
/// has begun and its triggers are on the stack.
fn p1_attacks_and_p0_blocks(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) {
    use mtg_engine::turn::Stage;
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.settle();
}

#[test]
fn while_conditions_are_checked_only_as_the_creature_attacks_or_blocks() {
    cr!("508.1m", "509.2a");
    ruling!(
        "Ravening Warg",
        "Ravening Warg's triggered ability will check when it attacks to see if you control a creature with power 4 or greater. If you don't, the ability won't trigger at all. It will not check again when it resolves, so the ability will resolve even if you no longer control a creature with power 4 or greater."
    );
    ruling!(
        "Wargling",
        "Wargling's triggered ability will check when it attacks to see if you control a creature with power 4 or greater. If you don't, the ability won't trigger at all. It will not check again when it resolves, so the ability will resolve even if you no longer control a creature with power 4 or greater."
    );
    ruling!(
        "Picnic Ruiner // Stolen Goodies",
        "If you controlled a creature with power 4 or greater when you declared Picnic Ruiner as an attacker, it doesn't matter whether you still control one as its ability resolves. Picnic Ruiner will still gain double strike until end of turn."
    );
    ruling!(
        "Burning Sun Cavalry",
        "If you controlled a Dinosaur when you declared Burning Sun Cavalry as an attacker or blocker, it doesn't matter whether you still control one as its ability resolves. Burning Sun Cavalry will still get +1/+1 until end of turn."
    );
    // (card, the creature that enables it)
    let cases = [
        ("Ravening Warg", BIG),
        ("Wargling", BIG),
        ("Picnic Ruiner // Stolen Goodies", BIG),
        ("Burning Sun Cavalry", "Colossal Dreadmaw"),
    ];
    for (name, enabler) in cases {
        supported(name);
        // Without the enabler: no trigger.
        let mut t = TestGame::new(2);
        let a = old(&mut t, P0, name);
        old(&mut t, P0, "Grizzly Bears");
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 0, "{name}: no trigger");
        // With it, gone before the ability resolves: the ability still does its thing.
        let mut t = TestGame::new(2);
        let a = old(&mut t, P0, name);
        let bears = old(&mut t, P0, "Grizzly Bears");
        let e = old(&mut t, P0, enabler);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1, "{name}: triggered");
        destroy(&mut t, e);
        t.resolve_all();
        match name {
            "Ravening Warg" => assert_eq!(t.life(P0), 22),
            "Wargling" => {
                assert_eq!(t.pt(a), (3, 2));
                assert!(has_kw(&t, a, KeywordKind::Trample));
                assert!(has_kw(&t, bears, KeywordKind::Trample));
            }
            "Picnic Ruiner // Stolen Goodies" => {
                assert!(has_kw(&t, a, KeywordKind::DoubleStrike))
            }
            _ => assert_eq!(t.pt(a), (3, 3)),
        }
    }
    // Burning Sun Cavalry as a blocker.
    let mut t = TestGame::new(2);
    let c = old(&mut t, P0, "Burning Sun Cavalry");
    let dino = old(&mut t, P0, "Colossal Dreadmaw");
    let att = old(&mut t, P1, "Grizzly Bears");
    p1_attacks_and_p0_blocks(&mut t, att, c);
    assert_eq!(t.stack_len(), 1, "blocking triggered");
    destroy(&mut t, dino);
    t.resolve_all();
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn item_shopkeep_needs_an_attacking_equipped_target_only_on_resolution() {
    cr!("608.2b", "611.2c");
    ruling!(
        "Item Shopkeep",
        "If the target creature isn't both attacking and equipped when Item Shopkeep's ability tries to resolve, it won't resolve and none of its effects will happen. However, in cases where it does resolve, it doesn't matter what happens to any of the Equipment on that creature. It'll still have menace even if it stops being equipped."
    );
    supported("Item Shopkeep");
    for unequip_before in [true, false] {
        let mut t = TestGame::new(2);
        old(&mut t, P0, "Item Shopkeep");
        let bears = old(&mut t, P0, "Grizzly Bears");
        let sword = t.battlefield(P0, "Bonesplitter");
        assert!(t.g.attach(sword, obj(bears)));
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        if unequip_before {
            destroy(&mut t, sword);
            t.resolve_all();
            assert!(!has_kw(&t, bears, KeywordKind::Menace), "fizzled");
        } else {
            t.resolve_all();
            destroy(&mut t, sword);
            assert!(has_kw(&t, bears, KeywordKind::Menace), "kept menace");
        }
    }
}

#[test]
fn raubahn_cant_attach_equipment_to_a_creature_it_cant_equip() {
    cr!("701.3b", "702.16d");
    ruling!(
        "Raubahn, Bull of Ala Mhigo",
        "If the target Equipment can't legally be attached to the target attacking creature when Raubahn's last ability resolves, the Equipment won't move."
    );
    supported("Raubahn, Bull of Ala Mhigo");
    for (attacker, moves) in [("Tel-Jilad Chosen", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        let r = old(&mut t, P0, "Raubahn, Bull of Ala Mhigo");
        let a = old(&mut t, P0, attacker);
        let holder = old(&mut t, P0, "Hill Giant");
        let sword = t.battlefield(P0, "Bonesplitter");
        assert!(t.g.attach(sword, obj(holder)));
        t.answer_targets(P0, &[obj(sword)]);
        t.answer_targets(P0, &[obj(a)]);
        attack_with(&mut t, &[(r, Entity::Player(P1)), (a, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        let on = t.obj_now(sword).attached_to;
        let expected = if moves { obj(a) } else { obj(holder) };
        assert_eq!(on, Some(expected), "{attacker}");
    }
}

#[test]
fn ogre_head_helm_sacrifices_the_equipped_creature() {
    cr!("702.151b");
    ruling!(
        "Ogre-Head Helm",
        "If the equipped creature deals combat damage, you may sacrifice the equipped creature, not Ogre-Head Helm."
    );
    supported("Ogre-Head Helm");
    let mut t = TestGame::new(2);
    let helm = old(&mut t, P0, "Ogre-Head Helm");
    let bears = old(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, helm, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(helm).attached_to, Some(obj(bears)));
    t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    assert!(
        t.in_graveyard(P0, "Grizzly Bears"),
        "the creature was sacrificed"
    );
    assert!(t.on_battlefield(helm), "not the Helm");
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn augusta_untaps_every_creature_then_you_choose_which_to_tap() {
    cr!("506.4b", "603.2");
    ruling!(
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Augusta’s last ability untaps all creatures you control, not just the ones that attacked this turn. Then you choose which creatures to tap, if any."
    );
    ruling!(
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Untapping an attacking creature doesn’t remove it from combat. It’s still attacking normally."
    );
    let mut t = TestGame::new(2);
    back_face(
        &mut t,
        P0,
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
    );
    let a = old(&mut t, P0, "Grizzly Bears");
    let b = old(&mut t, P0, "Hill Giant");
    let idle = old(&mut t, P0, "Llanowar Elves");
    assert!(t.g.tap(idle));
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    // Tap only Hill Giant again.
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert!(
        !t.obj_now(idle).tapped,
        "a creature that didn't attack untapped too"
    );
    assert!(!t.obj_now(a).tapped);
    assert!(t.obj_now(b).tapped, "the chosen one tapped");
    // Both still attack: the untapped one (2/3 with Augusta's +0/+1) and the tapped one
    // (4/3 with +1/+0) deal their damage.
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}

#[test]
fn augustas_bonuses_dont_care_whether_she_is_tapped() {
    cr!("611.3a");
    ruling!(
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Despite the term “other tapped creatures,” Augusta’s first ability doesn’t care whether Augusta is tapped or untapped. The same is true for the second ability. Both abilities apply as long as Augusta is on the battlefield."
    );
    for augusta_tapped in [false, true] {
        let mut t = TestGame::new(2);
        let aug = back_face(
            &mut t,
            P0,
            "Plargg, Dean of Chaos // Augusta, Dean of Order",
        );
        let base = t.pt(aug);
        let tapped = old(&mut t, P0, "Grizzly Bears");
        let untapped = old(&mut t, P0, "Hill Giant");
        assert!(t.g.tap(tapped));
        if augusta_tapped {
            assert!(t.g.tap(aug));
        }
        t.g.recompute();
        assert_eq!(t.pt(tapped), (3, 2), "{augusta_tapped}");
        assert_eq!(t.pt(untapped), (3, 4), "{augusta_tapped}");
        assert_eq!(t.pt(aug), base, "not herself");
    }
}

#[test]
fn plargg_casts_the_revealed_card_ignoring_timing() {
    cr!("608.2g");
    ruling!(
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Plargg, Dean of Chaos’s second ability allows you to cast the spell during that ability’s resolution, ignoring its normal timing restrictions."
    );
    let mut t = TestGame::new(2);
    let plargg = old(
        &mut t,
        P0,
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
    );
    stack_library(&mut t, P0, &["Divination"]);
    t.lands(P0, "Mountain", 5);
    // During P1's upkeep: a sorcery couldn't normally be cast.
    t.set_step(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.activate(P0, plargg, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand + 2);
}
