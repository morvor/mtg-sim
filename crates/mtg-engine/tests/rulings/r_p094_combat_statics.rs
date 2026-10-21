//! Rulings batch P094 — combat and static-ability interactions: evasion granted after
//! blocks (Hazoret, Cindering Cutthroat), "with power N" attack triggers (Adanto Vanguard,
//! Dreadhorde Invasion), Ian the Reckless's intervening "if" and last known information,
//! Howlsquad Heavy's mana ability, and Crovax's color-dependent +1/+1 and -1/-1.

use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, attach_new, damage, has_kw};
use mtg_engine::keywords::KeywordKind;
use crate::r_s16_common::set_speed;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0's `attacker` attacks P1 and is blocked by `blocker`; in the declare blockers step
/// `then` runs (and the stack resolves), then combat finishes.
fn blocked_then(
    t: &mut TestGame,
    attacker: ObjectId,
    blocker: ObjectId,
    then: impl FnOnce(&mut TestGame),
) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    then(t);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn unblockable_or_menace_after_blocks_doesnt_unblock() {
    cr!("509.1h", "509.1b", "702.110b");
    ruling!(
        "Hazoret, Godseeker",
        "Hazoret’s activated ability must be used prior to declaring blockers to be effective. Activating it targeting a creature that has already been blocked will not cause that creature to become unblocked."
    );
    ruling!(
        "Cindering Cutthroat",
        "Once Cindering Cutthroat has been blocked by a single creature, activating its last ability won't cause it to become unblocked."
    );
    supported("Hazoret, Godseeker");
    // Hazoret: Grizzly Bears blocked by a Llanowar Elves, then made unblockable.
    let mut t = TestGame::new(2);
    let hazoret = t.battlefield(P0, "Hazoret, Godseeker");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    blocked_then(&mut t, bears, elves, |t| {
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(t, P0, hazoret, "can't be blocked").unwrap();
        t.resolve_all();
        assert!(t.obj_now(hazoret).tapped);
    });
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // Cindering Cutthroat blocked by one creature, then gains menace.
    let mut t = TestGame::new(2);
    let cut = t.battlefield(P0, "Cindering Cutthroat");
    t.lands(P0, "Swamp", 2);
    let wall = t.battlefield(P1, "Wall of Stone");
    blocked_then(&mut t, cut, wall, |t| {
        activate_containing(t, P0, cut, "menace").unwrap();
        t.resolve_all();
        assert!(has_kw(t, cut, KeywordKind::Menace));
    });
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 3);
}

#[test]
fn adanto_vanguard_attack_triggers_see_its_boosted_power() {
    cr!("508.1", "508.3a", "611.3a");
    ruling!(
        "Adanto Vanguard",
        "An ability that triggers whenever a creature with a certain power attacks uses Adanto Vanguard's power after it has +2/+0 to determine whether it triggers."
    );
    supported("Adanto Vanguard");
    supported("Raid Bombardment");
    // Raid Bombardment: "Whenever a creature you control with power 2 or less attacks,
    // this enchantment deals 1 damage to the player or planeswalker it's attacking."
    for (name, triggers) in [("Adanto Vanguard", 0), ("Grizzly Bears", 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Raid Bombardment");
        let a = t.battlefield(P0, name);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "deals 1 damage"), triggers, "{name}");
        if name == "Adanto Vanguard" {
            assert_eq!(t.pt(a), (3, 1));
        }
    }
}

#[test]
fn dreadhorde_invasion_lifelink_depends_on_power_as_it_attacks() {
    cr!("508.1", "603.2", "611.2c");
    ruling!(
        "Dreadhorde Invasion",
        "If the token's power changes later in the turn after it has attacked, including while Dreadhorde Invasion's second ability is on the stack, it won't cause the token to gain or lose lifelink."
    );
    supported("Dreadhorde Invasion");
    supported("Giant Growth");
    for (counters, gets) in [(6u32, true), (5, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dreadhorde Invasion");
        // A 0/0 Zombie Army with counters, from the upkeep amass.
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        let army = crate::r_s05_common::tokens_with_subtype(&t, P0, "Army")[0];
        t.g.add_counters(obj(army), "+1/+1", counters - 1, None);
        t.g.objects[army.0 as usize].summoning_sick = false;
        t.g.recompute();
        t.advance_to(P0, Step::PrecombatMain);
        attack_with(&mut t, &[(army, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "lifelink"), gets as usize);
        // Its power changes while the ability (if any) is on the stack.
        if gets {
            t.g.remove_counters(obj(army), "+1/+1", 3);
        } else {
            t.lands(P0, "Forest", 1);
            let gg = t.hand(P0, "Giant Growth");
            t.cast(P0, gg).target(obj(army)).go();
        }
        t.resolve_all();
        let lifelink = has_kw(&t, army, KeywordKind::Lifelink);
        assert_eq!(lifelink, gets, "{counters}");
    }
}

/// Ian the Reckless attacks P1 under P0's control (modified by `modify`); returns the
/// number of Ian triggers on the stack.
fn ian_attacks(t: &mut TestGame, modify: impl FnOnce(&mut TestGame, ObjectId)) -> ObjectId {
    supported("Ian the Reckless");
    let ian = t.battlefield(P0, "Ian the Reckless");
    modify(t, ian);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(t, &[(ian, Entity::Player(P1))]);
    ian
}

#[test]
fn ian_the_reckless_checks_modified_on_attack_and_on_resolution() {
    cr!("603.4", "700.9");
    ruling!(
        "Ian the Reckless",
        "If Ian the Reckless isn’t modified when it attacks, its ability won’t trigger at all. When its ability tries to resolve, if it isn’t modified at that time, the ability won’t resolve."
    );
    // Unmodified: no trigger.
    let mut t = TestGame::new(2);
    ian_attacks(&mut t, |_, _| {});
    assert_eq!(triggers_on_stack(&t, "damage equal to its power"), 0);
    // Modified by Short Sword, which is destroyed in response: no effect.
    let mut t = TestGame::new(2);
    let mut sword = None;
    ian_attacks(&mut t, |t, ian| sword = Some(attach_new(t, P0, "Short Sword", ian)));
    assert_eq!(triggers_on_stack(&t, "damage equal to its power"), 1);
    destroy(&mut t, sword.unwrap());
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Control: still modified (a +1/+1 counter), so 3 damage each.
    let mut t = TestGame::new(2);
    ian_attacks(&mut t, |t, ian| {
        t.g.add_counters(obj(ian), "+1/+1", 1, None);
        t.g.recompute();
    });
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
}

#[test]
fn ian_the_reckless_uses_last_known_power_if_it_left() {
    cr!("608.2h", "603.4", "113.7a");
    ruling!(
        "Ian the Reckless",
        "If Ian the Reckless leaves the battlefield before its last ability resolves, as long as it was modified when it was last on the battlefield, use its power as it last existed on the battlefield to determine how much damage that ability deals."
    );
    let mut t = TestGame::new(2);
    let ian = ian_attacks(&mut t, |t, ian| {
        attach_new(t, P0, "Short Sword", ian);
    });
    assert_eq!(triggers_on_stack(&t, "damage equal to its power"), 1);
    move_to(&mut t, ian, Zone::Hand(P0));
    t.resolve_all();
    // It was a modified 3/2 as it last existed.
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
    // Modified by two +1/+1 counters (a 4/3), then destroyed.
    let mut t = TestGame::new(2);
    let ian = ian_attacks(&mut t, |t, ian| {
        t.g.add_counters(obj(ian), "+1/+1", 2, None);
        t.g.recompute();
    });
    destroy(&mut t, ian);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
}

#[test]
fn howlsquad_heavy_mana_ability_doesnt_use_the_stack() {
    cr!("605.3a", "605.3b", "605.1a");
    ruling!(
        "Howlsquad Heavy",
        "Mana abilities don’t use the stack, which means that they can’t be responded to. Notably, this means that a player can’t respond to this creature’s last ability by destroying one of your Goblins."
    );
    supported("Howlsquad Heavy");
    let mut t = TestGame::new(2);
    let heavy = t.battlefield(P0, "Howlsquad Heavy");
    set_speed(&mut t, P0, 4);
    t.battlefield(P0, "Goblin Piker");
    t.battlefield(P0, "Goblin Piker");
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, heavy, "for each Goblin"));
    assert_eq!(t.stack_len(), 0);
    // Heavy and two Pikers: {R}{R}{R}, with no chance for anyone to respond.
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
}

#[test]
fn crovax_damage_can_become_lethal_when_it_comes_or_goes() {
    cr!("704.5g", "611.3a", "120.6");
    ruling!(
        "Crovax, Ascendant Hero",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to white creatures may become lethal if Crovax leaves the battlefield during that turn, and nonlethal damage dealt to nonwhite creatures may become lethal if Crovax enters the battlefield during that turn."
    );
    supported("Crovax, Ascendant Hero");
    let mut t = TestGame::new(2);
    let crovax = t.battlefield(P0, "Crovax, Ascendant Hero");
    let lions = t.battlefield(P1, "Savannah Lions");
    let src = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(lions), (3, 2));
    damage(&mut t, src, 1, lions);
    assert!(t.on_battlefield(lions));
    move_to(&mut t, crovax, Zone::Hand(P0));
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    // A nonwhite Hill Giant with 2 damage dies as Crovax enters.
    damage(&mut t, src, 2, src);
    assert!(t.on_battlefield(src));
    t.enter(P0, "Crovax, Ascendant Hero");
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn crovax_return_ability_only_on_the_battlefield() {
    cr!("113.6", "602.1");
    ruling!(
        "Crovax, Ascendant Hero",
        "Crovax's last ability can be activated only while it's on the battlefield."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Crovax, Ascendant Hero");
    let in_gy = t.graveyard(P0, "Crovax, Ascendant Hero");
    assert!(!can_activate(&mut t, P0, in_hand));
    assert!(!can_activate(&mut t, P0, in_gy));
    let on_bf = t.battlefield(P0, "Crovax, Ascendant Hero");
    assert!(can_activate(&mut t, P0, on_bf));
    activate_containing(&mut t, P0, on_bf, "Return").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.zone(on_bf), Zone::Hand(P0));
}

#[test]
fn crovax_that_isnt_white_shrinks_itself() {
    cr!("611.3a", "105.3", "613.4c");
    ruling!(
        "Crovax, Ascendant Hero",
        "If Crovax, Ascendant Hero stops being white, it gives itself -1/-1."
    );
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let crovax = t.battlefield(P0, "Crovax, Ascendant Hero");
    assert_eq!(t.pt(crovax), (4, 4));
    t.lands(P0, "Island", 1);
    let wisps = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(obj(crovax)).go();
    t.resolve_all();
    assert_eq!(t.pt(crovax), (3, 3));
}

#[test]
fn second_crovax_legend_rule_and_shrunken_creatures_simultaneously() {
    cr!("704.3", "704.5j", "704.5f");
    ruling!(
        "Crovax, Ascendant Hero",
        "If a second Crovax, Ascendant Hero comes under your control, you'll put one into its owner's graveyard due to the \"legend rule\" at the same time that any creatures getting -2/-2 are put into their owner's graveyard for having 0 or less toughness."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crovax, Ascendant Hero");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(bears), (1, 1));
    // If the legend rule went first, the Bears would be back at 1/1 and survive.
    t.enter(P0, "Crovax, Ascendant Hero");
    t.settle();
    assert_eq!(t.named_on_battlefield("Crovax, Ascendant Hero").len(), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}


#[test]
fn modified_creature_leaving_is_judged_with_its_last_attachments() {
    // Regression for "modified" (CR 700.9) on an object that left the battlefield: it's
    // judged with the permanents attached to it as it last existed there (CR 608.2h),
    // even after its Equipment became unattached.
    cr!("700.9", "603.10a", "608.2h");
    for (name, text) in [
        ("Guardian of the Forgotten", "manifest"),
        ("Akki Ember-Keeper", "Spirit"),
        ("Ondu Knotmaster // Throw a Line", "two +1/+1 counters"),
        ("Costume Closet", "leaves the battlefield"),
        ("One with the Kami", "Spirit"),
    ] {
        supported(name);
        for (how, expect) in [("equipment", 1), ("their aura", 0), ("none", 0)] {
            let mut t = TestGame::new(2);
            if name == "One with the Kami" {
                // Enchanting another creature of P0's.
                let other = t.battlefield(P0, "Hill Giant");
                attach_new(&mut t, P0, name, other);
            } else {
                t.battlefield(P0, name);
            }
            let bears = t.battlefield(P0, "Grizzly Bears");
            match how {
                "equipment" => {
                    attach_new(&mut t, P0, "Short Sword", bears);
                }
                "their aura" => {
                    attach_new(&mut t, P1, "Pacifism", bears);
                }
                _ => {}
            }
            destroy(&mut t, bears);
            assert_eq!(triggers_on_stack(&t, text), expect, "{name}: {how}");
        }
    }
}
