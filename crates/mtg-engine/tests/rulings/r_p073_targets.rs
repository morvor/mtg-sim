//! Rulings batch P073 — targets checked as spells and abilities resolve (CR 608.2b):
//! fight spells with one illegal target (Outmuscle, Blizzard Brawl), abilities whose
//! only target became illegal (Hydaelyn, Arwen, Renegade's Getaway, Oath of Eorl),
//! modal spells (Collective Resistance; CR 700.2, 707.10), "until ~ leaves" exile
//! (Sheltered by Ghosts; CR 610.3c), and "for as long as you control ~" (Aegis Angel).

use crate::r_p073_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, damage, give_control};
use crate::r_s13_common::add;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Fight spells: Outmuscle, Blizzard Brawl
// ---------------------------------------------------------------------------

/// P0 casts `name` targeting `mine` and `theirs`, then `theirs` (or `mine` when
/// `mine_illegal`) is bounced to its owner's hand in response, and the spell resolves.
fn fight_with_one_illegal(t: &mut TestGame, name: &str, mine: ObjectId, theirs: ObjectId, mine_illegal: bool) {
    // Four Forests: Outmuscle's adamant is on.
    t.lands(P0, "Forest", 4);
    let c = t.hand(P0, name);
    t.cast(P0, c).target(obj(mine)).target(obj(theirs)).go();
    let gone = if mine_illegal { mine } else { theirs };
    t.g.move_object(gone, Zone::Hand(t.obj_now(gone).owner), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn fight_spells_with_an_illegal_target_dont_fight() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Outmuscle",
        "If either target is an illegal target as Outmuscle tries to resolve, neither creature will deal or be dealt damage."
    );
    ruling!(
        "Outmuscle",
        "If the creature you control is an illegal target as Outmuscle tries to resolve, no creature gets a +1/+1 counter. If that creature is a legal target but the creature you don't control isn't, it still gets a +1/+1 counter and gains indestructible if appropriate."
    );
    ruling!(
        "Blizzard Brawl",
        "If the creature you control is still a legal target as Blizzard Brawl tries to resolve but the target creature you don’t control isn’t, the creature you control will still get the bonuses until end of turn."
    );
    supported("Outmuscle");
    supported("Blizzard Brawl");
    // Outmuscle with {G}{G}{G}{G} (adamant): theirs bounced; mine gets the counter and
    // indestructible and takes no damage.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    fight_with_one_illegal(&mut t, "Outmuscle", mine, theirs, false);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert!(kw(&t, mine, INDESTRUCTIBLE));
    assert_eq!(t.obj_now(mine).damage, 0);
    // Mine bounced: no counter anywhere, no damage.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    fight_with_one_illegal(&mut t, "Outmuscle", mine, theirs, true);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.obj_now(theirs).damage, 0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Blizzard Brawl with three snow permanents: theirs bounced, mine keeps +1/+0 and
    // indestructible.
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Forest", 3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Blizzard Brawl");
    t.cast(P0, c).target(obj(mine)).target(obj(theirs)).go();
    t.g.move_object(theirs, Zone::Hand(P1), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 2));
    assert!(kw(&t, mine, INDESTRUCTIBLE));
    assert_eq!(t.obj_now(mine).damage, 0);
}

#[test]
fn fight_spells_need_both_targets() {
    cr!("601.2c", "115.1");
    ruling!(
        "Outmuscle",
        "You can't cast Outmuscle unless you choose both a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Blizzard Brawl",
        "You can’t cast Blizzard Brawl unless you choose both a creature you control and one you don’t control as targets."
    );
    for name in ["Outmuscle", "Blizzard Brawl"] {
        // Only a creature of mine.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        crate::r_s25_common::lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        assert!(!crate::r_p079_common::can_cast_now(&mut t, P0, c), "{name}");
        // Only theirs.
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Grizzly Bears");
        crate::r_s25_common::lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        assert!(!crate::r_p079_common::can_cast_now(&mut t, P0, c), "{name}");
        // Both.
        t.battlefield(P0, "Grizzly Bears");
        assert!(crate::r_p079_common::can_cast_now(&mut t, P0, c), "{name}");
    }
}

// ---------------------------------------------------------------------------
// Abilities whose only target became illegal
// ---------------------------------------------------------------------------

#[test]
fn hydaelyn_doesnt_draw_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal",
        "If the target creature is an illegal target as Hydaelyn, the Mothercrystal's last ability tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card even if that creature is legendary."
    );
    supported("Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::Upkeep);
        crate::r_s17_common::enter_transformed(
            &mut t,
            P0,
            "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal",
        );
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        t.answer_targets(P0, &[obj(isamaru)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let hand = t.hand_size(P0);
        if illegal {
            destroy_now(&mut t, isamaru);
        }
        t.resolve_all();
        if illegal {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(t.counters(isamaru, "+1/+1"), 1);
            assert!(kw(&t, isamaru, INDESTRUCTIBLE));
        }
    }
}

#[test]
fn arwen_with_an_illegal_target_gets_no_counters() {
    cr!("608.2b", "602.2b");
    ruling!(
        "Arwen, Mortal Queen",
        "If the target of Arwen's second ability is illegal as it tries to resolve, the ability does nothing. You won't get to put any counters on Arwen."
    );
    supported("Arwen, Mortal Queen");
    let mut t = TestGame::new(2);
    let arwen = t.enter(P0, "Arwen, Mortal Queen");
    assert_eq!(t.counters(arwen, "indestructible"), 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, arwen, "Remove an indestructible counter").unwrap();
    assert_eq!(t.counters(arwen, "indestructible"), 0, "removed as a cost");
    destroy_now(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(arwen, "+1/+1"), 0);
    assert_eq!(t.counters(arwen, "lifelink"), 0);
}

#[test]
fn arwen_dies_to_earlier_damage_once_the_counter_is_removed() {
    cr!("602.2b", "704.5g", "117.5");
    ruling!(
        "Arwen, Mortal Queen",
        "You remove the indestructible counter from Arwen as a cost to activate its second ability. If Arwen already received 2 damage earlier in the turn, it will be destroyed due to having lethal damage before you get to put a +1/+1 counter on it."
    );
    let mut t = TestGame::new(2);
    let arwen = t.enter(P0, "Arwen, Mortal Queen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 2, arwen);
    assert!(t.on_battlefield(arwen));
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, arwen, "Remove an indestructible counter").unwrap();
    t.settle();
    assert!(!t.on_battlefield(arwen), "destroyed before the ability resolves");
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "lifelink"), 1);
}

#[test]
fn renegades_getaway_makes_no_servo_without_its_target() {
    cr!("608.2b", "111.1");
    ruling!(
        "Renegade's Getaway",
        "If the target permanent becomes an illegal target, Renegade’s Getaway doesn’t resolve and none of its effects happen. You won’t create a Servo token."
    );
    ruling!(
        "Renegade's Getaway",
        "There’s no way to have Renegade’s Getaway’s effect create a Servo token and give that token indestructible."
    );
    supported("Renegade's Getaway");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Renegade's Getaway", &[obj(bears)]);
    let servos = crate::r_s05_common::tokens_with_subtype(&t, P0, "Servo");
    assert_eq!(servos.len(), 1);
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert!(!kw(&t, servos[0], INDESTRUCTIBLE), "the target was chosen before it existed");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p079_common::cast_card(&mut t, P0, "Renegade's Getaway", &[obj(bears)]).unwrap();
    destroy_now(&mut t, bears);
    t.resolve_all();
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Servo").is_empty());
}

/// P0's Oath of Eorl with two lore counters gets its third, with `target` chosen for
/// chapter III (none if `None`), and the chapter ability goes on the stack.
fn oath_chapter_three(t: &mut TestGame, target: Option<ObjectId>) -> ObjectId {
    let oath = t.battlefield(P0, "Oath of Eorl");
    add(t, oath, "lore", 2);
    t.resolve_all();
    t.answer_targets(P0, &target.map(obj).into_iter().collect::<Vec<_>>());
    add(t, oath, "lore", 1);
    t.settle();
    oath
}

#[test]
fn oath_of_eorl_monarch_needs_its_target_only_if_one_was_chosen() {
    cr!("608.2b", "115.1d", "725.1");
    ruling!(
        "Oath of Eorl",
        "When Oath of Eorl's final chapter ability triggers, you can choose not to target a Human just to become the monarch. However, if you do choose a target, and that target is illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't become the monarch."
    );
    supported("Oath of Eorl");
    // No target: monarch.
    let mut t = TestGame::new(2);
    let human = t.battlefield(P0, "Benalish Marshal");
    oath_chapter_three(&mut t, None);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert_eq!(t.counters(human, "indestructible"), 0);
    // A target that becomes illegal: not the monarch.
    let mut t = TestGame::new(2);
    let human = t.battlefield(P0, "Benalish Marshal");
    oath_chapter_three(&mut t, Some(human));
    destroy_now(&mut t, human);
    t.resolve_all();
    assert_eq!(t.g.monarch, None);
    // A legal target: both.
    let mut t = TestGame::new(2);
    let human = t.battlefield(P0, "Benalish Marshal");
    oath_chapter_three(&mut t, Some(human));
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert_eq!(t.counters(human, "indestructible"), 1);
}

#[test]
fn sheltered_by_ghosts_leaving_first_exiles_nothing() {
    cr!("610.3b");
    ruling!(
        "Sheltered by Ghosts",
        "If Sheltered by Ghosts leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled at all."
    );
    supported("Sheltered by Ghosts");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(theirs)]);
    crate::r_p079_common::cast_card(&mut t, P0, "Sheltered by Ghosts", &[]).unwrap();
    t.resolve(); // the Aura resolves; its enters trigger goes on the stack
    assert_eq!(t.stack_len(), 1);
    let aura = t.named_on_battlefield("Sheltered by Ghosts")[0];
    destroy_now(&mut t, aura);
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
}

// ---------------------------------------------------------------------------
// Collective Resistance
// ---------------------------------------------------------------------------

#[test]
fn collective_resistance_modes_with_illegal_targets() {
    cr!("608.2b", "702.120a");
    ruling!(
        "Collective Resistance",
        "If you chose more than one mode and the target of one of those modes becomes illegal, the other targets will still be affected. If all of the targets become illegal, Collective Resistance won't resolve."
    );
    supported("Collective Resistance");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Ornithopter");
    let ench = t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let c = t.hand(P0, "Collective Resistance");
    let spell = t
        .cast(P0, c)
        .modes(&[0, 1, 2])
        .target(obj(relic))
        .target(obj(ench))
        .target(obj(bears))
        .go();
    // {1}{G} plus escalate {G} for each of the two extra modes.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    t.g.move_object(relic, Zone::Hand(P1), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(crate::r_s07_common::resolved(&t, spell));
    assert!(t.in_graveyard(P1, "Propaganda"));
    assert!(kw(&t, bears, INDESTRUCTIBLE) && kw(&t, bears, HEXPROOF));
    // All targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, "Collective Resistance");
    let spell = t
        .cast(P0, c)
        .modes(&[0, 2])
        .target(obj(relic))
        .target(obj(bears))
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.g.move_object(relic, Zone::Hand(P1), mtg_engine::events::MoveCause::Effect, None);
    t.g.move_object(bears, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(!crate::r_s07_common::resolved(&t, spell));
}

#[test]
fn collective_resistance_copy_keeps_its_modes() {
    cr!("707.10", "700.2g", "707.10c");
    ruling!(
        "Collective Resistance",
        "If Collective Resistance is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Collective Resistance");
    let spell = t.cast(P0, c).modes(&[2]).target(obj(bears)).go();
    crate::r_p079_common::cast_card(&mut t, P0, "Twincast", &[obj(spell)]).unwrap();
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    let from = t.asked().len();
    t.resolve();
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(crate::r_s26_common::modes_on_stack(&t, copies[0]), vec![2]);
    // New targets were offered, new modes weren't.
    assert_eq!(
        crate::r_s07_common::count_asked(&t, from, |d| matches!(d, Decision::ChooseModes { .. })),
        0
    );
    assert_eq!(
        crate::r_s07_common::count_asked(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })),
        1
    );
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE) && kw(&t, giant, INDESTRUCTIBLE));
    assert!(t.on_battlefield(t.named_on_battlefield("Ornithopter")[0]));
}

#[test]
fn collective_resistance_triggers_wait_until_it_finishes_resolving() {
    cr!("608.2", "603.3", "700.2");
    ruling!(
        "Collective Resistance",
        "No player can cast spells or activate abilities in between the modes of a resolving spell. Any abilities that trigger won't be put onto the stack until Collective Resistance is done resolving."
    );
    // Perilous Myr is destroyed by the first mode; its trigger is put on the stack (and
    // targeted) only after the third mode gave the Bears hexproof.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Perilous Myr");
    let ench = t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let c = t.hand(P0, "Collective Resistance");
    t.cast(P0, c)
        .modes(&[0, 1, 2])
        .target(obj(myr))
        .target(obj(ench))
        .target(obj(bears))
        .go();
    let from = t.asked().len();
    t.resolve();
    assert!(t.in_graveyard(P1, "Perilous Myr"));
    let cands = crate::r_s02_common::target_candidates(&t, P1, from);
    assert_eq!(cands.len(), 1);
    assert!(!cands[0].contains(&obj(bears)), "the Bears had hexproof already");
    assert!(cands[0].contains(&pl(P0)));
}

// ---------------------------------------------------------------------------
// Aegis Angel
// ---------------------------------------------------------------------------

#[test]
fn aegis_angel_destroyed_together_with_the_permanent() {
    cr!("611.2b", "702.12b");
    ruling!(
        "Aegis Angel",
        "If Aegis Angel and the permanent would be destroyed simultaneously, only Aegis Angel will be destroyed."
    );
    ruling!("Aegis Angel", "If Aegis Angel leaves the battlefield, you no longer control it.");
    supported("Aegis Angel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let angel = crate::r_s05_common::enter(&mut t, P0, "Aegis Angel");
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(!t.on_battlefield(angel));
    assert!(t.on_battlefield(bears));
    // The Angel left: the Bears no longer has indestructible.
    assert!(!kw(&t, bears, INDESTRUCTIBLE));
    assert!(destroy_now(&mut t, bears));
}

#[test]
fn aegis_angel_effect_ends_for_good_when_control_changes() {
    cr!("611.2b");
    ruling!(
        "Aegis Angel",
        "If another player gains control of Aegis Angel, the permanent will no longer have indestructible, even if you regain control of Aegis Angel."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let angel = crate::r_s05_common::enter(&mut t, P0, "Aegis Angel");
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    give_control(&mut t, angel, P1);
    assert!(!kw(&t, bears, INDESTRUCTIBLE));
    give_control(&mut t, angel, P0);
    assert!(!kw(&t, bears, INDESTRUCTIBLE));
}
