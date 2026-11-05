//! Rulings batch P073 — granting hexproof: hexproof in response to a targeted spell (CR
//! 608.2b), hexproof on a creature you don't control (CR 702.11b), Auras on permanents
//! you don't control (Curator's Ward), haste from Equipment (Swiftfoot Boots), conditions
//! checked on resolution (Karametra's Blessing, Blacksmith's Skill, Built to Last), and
//! Slippery Bogbonder's counter moves.

use crate::r_p073_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::{crew, spell_targets};
use crate::r_s06_common::attach_new;
use crate::r_s09_common::legal_attack;
use crate::r_s13_common::add;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hexproof_in_response_makes_the_only_target_illegal() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Sheltering Word",
        "If a creature gains hexproof in response to a spell or ability controlled by an opponent that targets only it, that spell or ability doesn’t resolve for having an illegal target when it tries to resolve."
    );
    ruling!(
        "Celestial Armor",
        "If a creature gains hexproof in response to a spell or ability controlled by an opponent that only targets that creature, then when that spell or ability tries to resolve, all of its targets will be illegal."
    );
    supported("Sheltering Word");
    supported("Celestial Armor");
    for response in ["Sheltering Word", "Celestial Armor"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_p079_common::cast_card(&mut t, P1, "Lightning Bolt", &[obj(bears)]).unwrap();
        // Sheltering Word targets the creature; Celestial Armor's enters trigger does.
        t.answer_targets(P0, &[obj(bears)]);
        crate::r_p079_common::cast_card(&mut t, P0, response, &[]).unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{response}");
        assert_eq!(t.obj_now(bears).damage, 0, "{response}: the Bolt didn't resolve");
        assert!(kw(&t, bears, HEXPROOF));
        assert!(t.in_graveyard(P1, "Lightning Bolt"));
        if response == "Sheltering Word" {
            assert_eq!(t.life(P0), 22, "life equal to its toughness");
        }
    }
}

#[test]
fn celestial_armor_doesnt_unblock_an_already_blocked_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "Celestial Armor",
        "Attaching Celestial Armor to a creature that has already been blocked won't cause it to become unblocked, even if one or more of the blocking creatures don't have flying or reach."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, pl(P1))], &[(wall, bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_p079_common::cast_card(&mut t, P0, "Celestial Armor", &[]).unwrap();
    t.resolve_all();
    assert!(kw(&t, bears, FLYING));
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert_eq!(t.obj_now(wall).damage, 4);
}

#[test]
fn mask_of_avacyn_on_an_opponents_creature_stops_your_targeting() {
    cr!("702.11b", "303.4e");
    ruling!(
        "Mask of Avacyn",
        "If Mask of Avacyn somehow becomes attached to a creature an opponent controls, that creature can't be the target of spells or abilities you control."
    );
    supported("Mask of Avacyn");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Mask of Avacyn", obj(theirs));
    assert!(kw(&t, theirs, HEXPROOF));
    assert_eq!(t.pt(theirs), (3, 4));
    assert!(!spell_targets(&mut t, P0, "Shock").contains(&obj(theirs)));
    assert!(spell_targets(&mut t, P1, "Shock").contains(&obj(theirs)));
}

#[test]
fn curators_ward_on_a_permanent_you_dont_control_draws_for_you() {
    cr!("603.10a", "303.4e");
    ruling!(
        "Curator's Ward",
        "If Curator’s Ward is attached to a historic permanent you don’t control, you draw two cards when that permanent leaves the battlefield, not that permanent’s controller."
    );
    supported("Curator's Ward");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, "Curator's Ward", obj(theirs));
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    destroy_now(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn swiftfoot_boots_haste_must_last_until_attackers_are_declared() {
    cr!("302.6", "702.10b", "508.1a");
    ruling!(
        "Swiftfoot Boots",
        "If a creature enters the battlefield under your control and gains haste, but then loses it before attacking, it won't be able to attack that turn. This means that you can't use one Swiftfoot Boots to allow two new creatures to attack in the same turn."
    );
    supported("Swiftfoot Boots");
    let mut t = TestGame::new(2);
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Hill Giant");
    let boots = attach_new(&mut t, P0, "Swiftfoot Boots", obj(a));
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(a, pl(P1))]));
    assert!(!legal_attack(&mut t, &[(b, pl(P1))]));
    // Moved to the other creature: the first one can't attack any more.
    assert!(t.g.attach(boots, obj(b)));
    t.g.recompute();
    assert!(!legal_attack(&mut t, &[(a, pl(P1))]));
    assert!(legal_attack(&mut t, &[(b, pl(P1))]));
    assert!(!legal_attack(&mut t, &[(a, pl(P1)), (b, pl(P1))]));
}

#[test]
fn hapatras_mark_needs_no_minus_counters() {
    cr!("702.11b");
    ruling!(
        "Hapatra's Mark",
        "You can target a creature with no -1/-1 counters on it. It will still gain hexproof."
    );
    supported("Hapatra's Mark");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Hapatra's Mark", &[obj(bears)]);
    assert!(kw(&t, bears, HEXPROOF));
    assert_eq!(t.pt(bears), (2, 2));
}

/// The "How many ... counters to move from X" prompts asked since decision `from`.
fn move_prompts(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseNumber { prompt, .. } if prompt.contains("to move") => {
                Some(prompt.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn slippery_bogbonder_moves_chosen_counters_on_resolution() {
    cr!("122.5", "608.2c");
    ruling!(
        "Slippery Bogbonder",
        "You can't move counters from the target creature to itself."
    );
    ruling!(
        "Slippery Bogbonder",
        "You choose which counters to move as Slippery Bogbonder's triggered ability resolves."
    );
    ruling!("Slippery Bogbonder", "You may choose to move zero counters.");
    supported("Slippery Bogbonder");
    // The target's own counters aren't offered; counters put on another creature after
    // the trigger was put on the stack are offered as it resolves.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, target, "+1/+1", 2);
    t.answer_targets(P0, &[obj(target)]);
    crate::r_s05_common::enter(&mut t, P0, "Slippery Bogbonder");
    assert_eq!(t.stack_len(), 1);
    add(&mut t, giant, "+1/+1", 3);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.resolve_all();
    let prompts = move_prompts(&t, from);
    assert_eq!(prompts.len(), 1, "{prompts:?}");
    assert!(prompts[0].contains("Hill Giant"));
    assert_eq!(t.counters(target, "+1/+1"), 5);
    assert_eq!(t.counters(target, "hexproof"), 1);
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    // Zero counters may be moved.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, "+1/+1", 2);
    t.answer_targets(P0, &[obj(target)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    crate::r_s05_common::enter(&mut t, P0, "Slippery Bogbonder");
    t.resolve_all();
    assert_eq!(t.counters(target, "hexproof"), 1);
    assert_eq!(t.counters(target, "+1/+1"), 0);
    assert_eq!(t.counters(giant, "+1/+1"), 2);
}

#[test]
fn karametras_blessing_checks_enchanted_only_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Karametra's Blessing",
        "Whether the target creature is enchanted or an enchantment is checked only while Karametra’s Blessing is resolving."
    );
    supported("Karametra's Blessing");
    // Enchanted in response: it gains hexproof and indestructible, and keeps them after
    // the Aura leaves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p079_common::cast_card(&mut t, P0, "Karametra's Blessing", &[obj(bears)]).unwrap();
    let aura = attach_new(&mut t, P0, "Holy Strength", obj(bears));
    t.resolve_all();
    assert!(kw(&t, bears, HEXPROOF) && kw(&t, bears, INDESTRUCTIBLE));
    destroy_now(&mut t, aura);
    assert!(kw(&t, bears, HEXPROOF) && kw(&t, bears, INDESTRUCTIBLE));
    assert_eq!(t.pt(bears), (4, 4));
    // Enchanted when cast but not as it resolves: only +2/+2; enchanted later: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", obj(bears));
    crate::r_p079_common::cast_card(&mut t, P0, "Karametra's Blessing", &[obj(bears)]).unwrap();
    destroy_now(&mut t, aura);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!kw(&t, bears, HEXPROOF) && !kw(&t, bears, INDESTRUCTIBLE));
    attach_new(&mut t, P0, "Holy Strength", obj(bears));
    assert!(!kw(&t, bears, HEXPROOF) && !kw(&t, bears, INDESTRUCTIBLE));
}

#[test]
fn blacksmiths_skill_and_built_to_last_check_artifact_creature_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Blacksmith's Skill",
        "Blacksmith's Skill checks whether the permanent is an artifact creature when it resolves. If the target isn't an artifact creature when the spell is cast but becomes one before the spell resolves, it will get +2/+2."
    );
    ruling!(
        "Built to Last",
        "An artifact creature gains indestructible in addition to getting +2/+2, not instead of it."
    );
    supported("Blacksmith's Skill");
    supported("Built to Last");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p079_common::cast_card(&mut t, P0, "Blacksmith's Skill", &[obj(copter)]).unwrap();
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (5, 5));
    assert!(kw(&t, copter, HEXPROOF) && kw(&t, copter, INDESTRUCTIBLE));
    // Built to Last on an artifact creature and on a plain creature.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Built to Last", &[obj(thopter)]);
    cast_resolve(&mut t, P0, "Built to Last", &[obj(bears)]);
    assert_eq!(t.pt(thopter), (2, 4));
    assert!(kw(&t, thopter, INDESTRUCTIBLE));
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!kw(&t, bears, INDESTRUCTIBLE));
}
