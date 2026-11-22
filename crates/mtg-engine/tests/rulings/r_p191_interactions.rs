//! Rulings batch P191 — interactions: abilities that trigger an additional time (Echoes of
//! Eternity, CR 603.2d), token creation replacements and who controls the tokens (CR
//! 614.1, 111.2), redirection shields (CR 615), layers 7b–7c and counters (CR 613.4),
//! requirements to block (CR 509.1c), divided counters and illegal targets (CR 608.2b),
//! casting during resolution (CR 608.2g), loyalty abilities gained and linked (CR 607.2),
//! planeswalkers entering with loyalty (CR 306.5b), Two-Headed Giant (CR 810), and
//! controlling another player's turn (CR 723).

use crate::r_p003_common::{giant_growth, kill, put};
use crate::r_p007_common::divide;
use crate::r_p008_common::hit;
use crate::r_p014_common::thg;
use crate::r_p015_common::{attacking, move_to};
use crate::r_p018_common::modify;
use crate::r_p076_common::mana;
use crate::r_p191_common::*;
use crate::r_s01_common::{
    attack_with, block_and_finish, stack_library, supported, tokens, with_subtype,
};
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, attach_new, attached_to, has_kw};
use mtg_engine::ability::Modification;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::player_control::decider;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Echoes of Eternity
// ---------------------------------------------------------------------------------------

#[test]
fn echoes_of_eternity_copies_multiply_with_more_echoes() {
    cr!("603.2d", "707.10");
    ruling!(
        "Echoes of Eternity",
        "If you control two copies of Echoes of Eternity and you cast a colorless spell, each of their second abilities will trigger. Then, each of their first abilities will cause the other's second ability to trigger an additional time. In total, you'll get four copies of the spell in addition to the original spell. If you control three copies of Echoes of Eternity, you'll end up with nine copies of the spell plus the original."
    );
    supported("Echoes of Eternity");
    for (echoes, copies) in [(1usize, 1usize), (2, 4), (3, 9)] {
        let mut t = TestGame::new(2);
        for _ in 0..echoes {
            t.battlefield(P0, "Echoes of Eternity");
        }
        let thopter = t.hand(P0, "Ornithopter");
        t.cast(P0, thopter).go();
        t.settle();
        assert_eq!(t.stack_len(), 1 + echoes * echoes, "{echoes}");
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Ornithopter").len(),
            1 + copies,
            "{echoes}"
        );
    }
}

#[test]
fn echoes_of_eternity_colorless_permanent_triggers_once_more_per_echoes() {
    cr!("603.2d");
    ruling!(
        "Echoes of Eternity",
        "If you control two copies of Echoes of Eternity, abilities of colorless spells and other colorless permanents you control trigger three times. A third Echoes of Eternity causes such abilities to trigger four times"
    );
    supported("Pierce Strider");
    for echoes in [1usize, 2, 3] {
        let mut t = TestGame::new(2);
        for _ in 0..echoes {
            t.battlefield(P0, "Echoes of Eternity");
        }
        for _ in 0..=echoes {
            t.answer_targets(P0, &[pl(P1)]);
        }
        enter(&mut t, P0, "Pierce Strider");
        assert_eq!(t.stack_len(), 1 + echoes, "{echoes}");
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - 3 * (1 + echoes as i32), "{echoes}");
    }
}

#[test]
fn echoes_of_eternity_color_changes_after_triggering_dont_matter() {
    cr!("603.2d", "603.3", "105.2c");
    ruling!(
        "Echoes of Eternity",
        "Once an ability of a permanent you control has triggered, making it colorless won't cause that ability to trigger an additional time. Similarly, once an ability of a colorless permanent has triggered an additional time due to Echoes of Eternity's first ability, giving that permanent one or more colors won't remove that additional triggered ability from the stack."
    );
    // A green Elvish Visionary's trigger; it's made colorless afterward: still one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let elf = enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(t.stack_len(), 1);
    modify(&mut t, elf, vec![Modification::SetColors(ColorSet::NONE)]);
    assert_eq!(t.obj(elf).chars.colors, ColorSet::NONE);
    assert_eq!(t.stack_len(), 1);
    // A colorless Pierce Strider's two triggers; it's made green afterward: still two.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    let strider = enter(&mut t, P0, "Pierce Strider");
    assert_eq!(t.stack_len(), 2);
    modify(
        &mut t,
        strider,
        vec![Modification::SetColors(ColorSet::single(Color::Green))],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn echoes_of_eternity_doesnt_change_replacement_effects() {
    cr!("603.2d", "614.1c", "702.43a");
    ruling!(
        "Echoes of Eternity",
        "Replacement effects are unaffected by Echoes of Eternity's first ability. For example, a colorless creature that enters the battlefield under your control with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let worker = enter(&mut t, P0, "Arcbound Worker");
    assert_eq!(t.counters(worker, "+1/+1"), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn echoes_of_eternity_copy_isnt_cast() {
    cr!("707.10", "603.2");
    ruling!(
        "Echoes of Eternity",
        "The copy created by Echoes of Eternity's last ability is put directly onto the stack. It isn't cast, so any abilities that apply \"when you cast [this spell]\" won't trigger."
    );
    // Devourer of Destiny ("When you cast this spell, exile target permanent that's one
    // or more colors."): its cast trigger triggers twice (it's a colorless spell's), the
    // copy's never.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Raging Goblin");
    let dev = t.hand(P0, "Devourer of Destiny");
    mana(&mut t, P0, ManaType::C, 7);
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(b)]);
    t.answer_targets(P0, &[obj(c)]);
    t.answer_targets(P0, &[obj(c)]);
    t.cast(P0, dev).go();
    t.settle();
    assert_eq!(stacked_triggers(&t, "exile target").len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(c));
    assert_eq!(t.named_on_battlefield("Devourer of Destiny").len(), 2);
}

// ---------------------------------------------------------------------------------------
// Queen Allenal of Ruadach
// ---------------------------------------------------------------------------------------

#[test]
fn queen_allenal_counts_itself() {
    cr!("604.3", "613.4a");
    ruling!(
        "Queen Allenal of Ruadach",
        "Queen Allenal of Ruadach’s first ability counts itself, so its power and toughness will always be at least 1/1."
    );
    supported("Queen Allenal of Ruadach");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Queen Allenal of Ruadach");
    assert_eq!(t.pt(q), (1, 1));
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(q), (2, 2));
}

#[test]
fn queen_allenal_soldier_shares_the_creation_instructions_not_the_abilities() {
    cr!("614.1a", "508.4");
    ruling!(
        "Queen Allenal of Ruadach",
        "The additional Soldier token won’t have any abilities the other tokens were created with. Anything else specified in the effect creating the tokens (such as tapped, attacking, “That token gains haste,” or “Exile that token at end of combat”) applies to both the original tokens and the Soldier."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Queen Allenal of Ruadach");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    attack_with(&mut t, &[(geist, pl(P1))]);
    t.resolve_all();
    let angel = with_subtype(&t, P0, "Angel");
    let soldier = with_subtype(&t, P0, "Soldier");
    assert_eq!((angel.len(), soldier.len()), (1, 1));
    let (angel, soldier) = (angel[0], soldier[0]);
    for id in [angel, soldier] {
        assert!(t.obj(id).tapped);
        assert_eq!(attacking(&t, id), Some(pl(P1)));
    }
    assert!(has_kw(&t, angel, KeywordKind::Flying));
    assert!(!has_kw(&t, soldier, KeywordKind::Flying));
    assert_eq!(t.pt(soldier), (1, 1));
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
    assert!(!t.on_battlefield(soldier));
}

#[test]
fn queen_allenal_and_effects_on_who_gets_the_tokens() {
    cr!("614.1a", "616.1", "111.2");
    ruling!(
        "Queen Allenal of Ruadach",
        "If an effect changes under whose control a token would be created, that effect applies before Queen Allenal’s effect applies. If an effect changes under whose control a token would enter the battlefield, that effect applies after Queen Allenal’s effect is able to be applied."
    );
    supported("Crafty Cutpurse");
    supported("Gather Specimens");
    // Crafty Cutpurse (P1's): P0's Goblin token is created under P1's control instead, so
    // P0's Queen Allenal doesn't add a Soldier.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Queen Allenal of Ruadach");
    enter(&mut t, P1, "Crafty Cutpurse");
    t.resolve_all();
    let c = t.hand(P0, "Goblin Offensive");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, c).x(1).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Goblin").len(), 1);
    assert!(with_subtype(&t, P0, "Goblin").is_empty());
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(with_subtype(&t, P1, "Soldier").is_empty());
    // The same with the Queen on P1's side: she adds a Soldier for P1.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Queen Allenal of Ruadach");
    enter(&mut t, P1, "Crafty Cutpurse");
    t.resolve_all();
    let c = t.hand(P0, "Goblin Offensive");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, c).x(1).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Goblin").len(), 1);
    assert_eq!(with_subtype(&t, P1, "Soldier").len(), 1);
    // Gather Specimens (P1's): P0's Queen applies (Goblin and Soldier), then both enter
    // under P1's control.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Queen Allenal of Ruadach");
    let gs = t.hand(P1, "Gather Specimens");
    mana(&mut t, P1, ManaType::U, 3);
    mana(&mut t, P1, ManaType::C, 3);
    t.cast(P1, gs).go();
    t.resolve_all();
    let c = t.hand(P0, "Goblin Offensive");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, c).x(1).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Goblin").len(), 1);
    assert_eq!(with_subtype(&t, P1, "Soldier").len(), 1);
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Razia, Boros Archangel
// ---------------------------------------------------------------------------------------

/// Razia's ability: the next 3 damage to `from` this turn is dealt to `to` instead.
fn razia(t: &mut TestGame, from: ObjectId, to: ObjectId) -> ObjectId {
    let r = t.battlefield(P0, "Razia, Boros Archangel");
    t.activate(P0, r, 0, &[obj(from), obj(to)]).unwrap();
    t.resolve_all();
    r
}

#[test]
fn razia_redirection_needs_both_targets_but_not_razia() {
    cr!("614.9", "611.2a");
    ruling!(
        "Razia, Boros Archangel",
        "If either target creature leaves the battlefield before damage is dealt, that damage won’t be redirected. It doesn’t matter if Razia leaves the battlefield."
    );
    supported("Razia, Boros Archangel");
    let shock_src = |t: &mut TestGame| t.battlefield(P1, "Raging Goblin");
    // Razia leaves: still redirected.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let r = razia(&mut t, mine, theirs);
    kill(&mut t, r);
    let src = shock_src(&mut t);
    hit(&mut t, src, obj(mine), 3);
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    // The other creature leaves: not redirected.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    razia(&mut t, mine, theirs);
    move_to(&mut t, theirs, Zone::Hand(P1));
    let src = shock_src(&mut t);
    hit(&mut t, src, obj(mine), 2);
    assert_eq!(t.obj(mine).damage, 2);
}

#[test]
fn razia_shield_redirects_three_damage_from_any_sources_over_time() {
    cr!("614.9", "614.1a");
    ruling!(
        "Razia, Boros Archangel",
        "The 3 damage doesn’t have to be from the same source, and it doesn’t have to be all dealt at once. If only 2 damage is redirected, the next 1 damage will also be redirected."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    razia(&mut t, mine, theirs);
    let a = t.battlefield(P1, "Raging Goblin");
    let b = t.battlefield(P1, "Grizzly Bears");
    hit(&mut t, a, obj(mine), 2);
    assert_eq!(t.obj(mine).damage, 0);
    assert_eq!(t.obj(theirs).damage, 2);
    // The next 1 of 2 is redirected; the other 1 is dealt to the Giant.
    hit(&mut t, b, obj(mine), 2);
    assert_eq!(t.obj(theirs).damage, 3);
    assert_eq!(t.obj(mine).damage, 1);
    hit(&mut t, a, obj(mine), 1);
    assert_eq!(t.obj(theirs).damage, 3);
    assert_eq!(t.obj(mine).damage, 2);
}

// ---------------------------------------------------------------------------------------
// Omnibian, Godhead of Awe
// ---------------------------------------------------------------------------------------

#[test]
fn omnibian_sets_base_power_below_counters_and_pumps() {
    cr!("613.4b", "613.4c", "613.3");
    ruling!(
        "Omnibian",
        "If the target creature has any counters on it that modify its power and toughness (such as a +1/+1 counter), those counters will continue to affect the creature after it becomes 3/3. In the example given, it will be 4/4."
    );
    ruling!(
        "Omnibian",
        "If the target creature is affected by any effects that modify its power or toughness without setting them to specific values (such as from Giant Growth), Omnibian’s effect will be applied before those effects. In the example given, the creature will be 6/6."
    );
    let mut t = TestGame::new(2);
    let frog = t.battlefield(P0, "Omnibian");
    let a = t.battlefield(P1, "Craw Wurm");
    put(&mut t, a, "+1/+1", 1);
    t.activate(P0, frog, 0, &[obj(a)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert!(t.obj(a).chars.subtypes.iter().any(|s| s == "Frog"));
    // Giant Growth first, then Omnibian: still 6/6.
    let mut t = TestGame::new(2);
    let frog = t.battlefield(P0, "Omnibian");
    let b = t.battlefield(P1, "Craw Wurm");
    giant_growth(&mut t, b);
    t.activate(P0, frog, 0, &[obj(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(b), (6, 6));
}

#[test]
fn two_godheads_of_awe_make_each_other_one_one() {
    cr!("613.4b", "613.1");
    ruling!(
        "Godhead of Awe",
        "If two Godheads of Awe are on the battlefield, they'll each make the other one 1/1."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Godhead of Awe");
    let b = t.battlefield(P1, "Godhead of Awe");
    assert_eq!(t.pt(a), (1, 1));
    assert_eq!(t.pt(b), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Dominus of Fealty, Giant Ambush Beetle
// ---------------------------------------------------------------------------------------

#[test]
fn dominus_of_fealty_on_your_own_permanent_still_asks() {
    cr!("608.2c", "603.5", "118.12");
    interp!("may-gain-control-if-you-do-follows-the-choice");
    ruling!(
        "Dominus of Fealty",
        "If you already control the targeted permanent when the ability resolves, you still need to choose whether or not to apply the effect. If you choose to apply it, the permanent untaps and gains haste. If you don’t, it doesn’t."
    );
    supported("Dominus of Fealty");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dominus of Fealty");
        let bears = t.battlefield_sick(P0, "Grizzly Bears");
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.g.tap(bears);
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, yes);
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        assert_eq!(t.obj(bears).tapped, !yes, "{yes}");
        assert_eq!(has_kw(&t, bears, KeywordKind::Haste), yes, "{yes}");
    }
}

#[test]
fn assault_suit_on_a_creature_that_player_already_controls_still_untaps_it() {
    cr!("608.2c", "118.12", "301.5d");
    interp!("may-gain-control-if-you-do-follows-the-choice");
    supported("Assault Suit");
    supported("Control Magic");
    // P0's Assault Suit stays on its creature after P1's Control Magic takes it. At P1's
    // upkeep, P0 may have P1 gain control of the equipped creature: choosing to apply the
    // effect counts as doing it, though P1 already controls it, as for Dominus of Fealty.
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let suit = attach_new(&mut t, P0, "Assault Suit", bears);
        attach_new(&mut t, P1, "Control Magic", bears);
        assert_eq!(t.obj(bears).controller, P1);
        assert_eq!(attached_to(&t, suit), Some(obj(bears)));
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::Upkeep);
        t.g.tap(bears);
        t.answer_yes(P0, yes);
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        assert_eq!(t.obj(bears).controller, P1, "{yes}");
        assert_eq!(t.obj(bears).tapped, !yes, "{yes}");
    }
}

#[test]
fn giant_ambush_beetle_requirement_does_nothing_if_the_creature_cant_block_it() {
    cr!("509.1c", "509.1b", "509.1d");
    interp!("default-combat-declarations");
    ruling!(
        "Giant Ambush Beetle",
        "If you choose to have the targeted creature block but that creature isn't able to block Giant Ambush Beetle (for example, because the targeted creature can block only creatures with flying), the requirement to block does nothing. The targeted creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    ruling!(
        "Giant Ambush Beetle",
        "Tapped creatures, creatures that can't block as the result of an effect, creatures with unpaid costs to block (such as those from War Cadence), and creatures that aren't controlled by the defending player are exempt from effects that would require them to block. Such creatures can be targeted by Giant Ambush Beetle's ability, but the requirement to block does nothing."
    );
    // (the target, its controller, tapped, under Pacifism, War Cadence's cost, exempt):
    // the last case, an untapped Grizzly Bears free to block, must block it.
    for (blocker_name, owner, tapped, pacifism, cadence, exempt) in [
        ("Cloud Sprite", P1, false, false, false, true),
        ("Grizzly Bears", P1, true, false, false, true),
        ("Grizzly Bears", P1, false, true, false, true),
        ("Grizzly Bears", P2, false, false, false, true),
        ("Grizzly Bears", P1, false, false, true, true),
        ("Grizzly Bears", P1, false, false, false, false),
    ] {
        let case = (blocker_name, owner, tapped, pacifism, cadence);
        let mut t = TestGame::new(3);
        let target = t.battlefield(owner, blocker_name);
        if tapped {
            t.g.tap(target);
        }
        if pacifism {
            attach_new(&mut t, P0, "Pacifism", target);
        }
        if cadence {
            let wc = t.battlefield(P0, "War Cadence");
            mana(&mut t, P0, ManaType::R, 1);
            mana(&mut t, P0, ManaType::C, 1);
            t.answer(P0, DecisionKind::X, Answer::Number(1));
            t.activate(P0, wc, 0, &[]).unwrap();
            t.resolve_all();
            t.lands(P1, "Forest", 1);
        }
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(target)]);
        let beetle = enter(&mut t, P0, "Giant Ambush Beetle");
        t.resolve_all();
        attack_with(&mut t, &[(beetle, pl(P1))]);
        // P1 declares no blockers; a declaration that disobeys the requirement isn't kept.
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), if exempt { 16 } else { 20 }, "{case:?}");
        assert_eq!(t.on_battlefield(target), exempt, "{case:?}");
    }
}

// ---------------------------------------------------------------------------------------
// Broodlord, Arsenal Thresher
// ---------------------------------------------------------------------------------------

#[test]
fn broodlord_counters_for_an_illegal_target_are_lost() {
    cr!("601.2d", "608.2b", "702.156a");
    ruling!(
        "Broodlord",
        "If some of the creatures are illegal targets as Broodlord's triggered ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost. They won't be put instead on a legal target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let bl = t.hand(P0, "Broodlord");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 6);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[1, 2]);
    t.cast(P0, bl).x(3).go();
    t.resolve();
    assert!(t.on_battlefield(bl));
    assert_eq!(t.counters(bl, "+1/+1"), 3);
    move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(bl, "+1/+1"), 3);
}

#[test]
fn arsenal_threshers_entering_together_may_reveal_the_same_cards() {
    cr!("614.12", "614.1c", "701.20a");
    ruling!(
        "Arsenal Thresher",
        "If multiple Arsenal Threshers are entering at the same time (for example, if Warp World is resolving), you may reveal the same artifact cards from your hand for each of them."
    );
    supported("Arsenal Thresher");
    let mut t = TestGame::new(2);
    let o = t.hand(P0, "Ornithopter");
    let s = t.hand(P0, "Sol Ring");
    let lib = stack_library(&mut t, P0, &["Arsenal Thresher", "Arsenal Thresher"]);
    let c = t.hand(P0, "Genesis Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::U, 3);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    t.answer_choose(P0, &[obj(o), obj(s)]);
    t.answer_choose(P0, &[obj(o), obj(s)]);
    t.cast(P0, c).go();
    t.resolve_all();
    let th = t.named_on_battlefield("Arsenal Thresher");
    assert_eq!(th.len(), 2);
    for id in th {
        assert_eq!(t.counters(id, "+1/+1"), 2);
    }
}

// ---------------------------------------------------------------------------------------
// Mindleech Mass, Bruna, Djeru and Hazoret, Hugs, Crackling Drake
// ---------------------------------------------------------------------------------------

#[test]
fn mindleech_mass_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "601.2", "601.3");
    ruling!(
        "Mindleech Mass",
        "The spell you cast via the triggered ability is cast as part of the resolution of that ability. Timing restrictions based on the card's type are ignored. Other restrictions, such as \"Cast [this spell] only during an opponent's turn,\" are not."
    );
    supported("Mindleech Mass");
    let mut t = TestGame::new(2);
    let mass = t.battlefield(P0, "Mindleech Mass");
    let div = t.hand(P1, "Divination");
    let reset = t.hand(P1, "Reset");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    attack_with(&mut t, &[(mass, pl(P1))]);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![obj(div)]));
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // Reset ("Cast this spell only during an opponent's turn after their upkeep step.")
    // wasn't offered; Divination, a sorcery, was cast during combat.
    for c in entity_choices(&t, from) {
        assert!(!c.contains(&obj(reset)), "Reset was offered");
    }
    assert!(t.in_graveyard(P1, "Divination"));
    assert!(t.in_hand(P1, "Reset"));
}

#[test]
fn bruna_takes_only_auras_that_could_enchant_it() {
    cr!("701.3a", "701.3b", "608.2d");
    ruling!(
        "Bruna, Light of Alabaster",
        "Only Auras that could legally enchant Bruna may be attached to it, whether they're being moved from other permanents or being put onto the battlefield. An Aura with \"enchant land,\" for example, could not be attached to Bruna this way."
    );
    supported("Bruna, Light of Alabaster");
    let mut t = TestGame::new(2);
    let bruna = t.battlefield(P0, "Bruna, Light of Alabaster");
    let forest = t.battlefield(P0, "Forest");
    let seas = attach_new(&mut t, P0, "Spreading Seas", forest);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hs = attach_new(&mut t, P0, "Holy Strength", bears);
    let wild = t.hand(P0, "Wild Growth");
    let hs2 = t.hand(P0, "Holy Strength");
    let from = t.asked().len();
    attack_with(&mut t, &[(bruna, pl(P1))]);
    t.answer_choose(P0, &[obj(hs)]);
    t.answer_choose(P0, &[obj(hs2)]);
    t.resolve_all();
    let offered: Vec<Entity> = entity_choices(&t, from).into_iter().flatten().collect();
    assert!(!offered.contains(&obj(seas)));
    assert!(!offered.contains(&obj(wild)));
    assert_eq!(attached_to(&t, seas), Some(obj(forest)));
    assert_eq!(attached_to(&t, hs), Some(obj(bruna)));
    assert!(t.in_hand(P0, "Wild Growth"));
    let hs2 = t.g.current(hs2);
    assert!(t.on_battlefield(hs2));
    assert_eq!(attached_to(&t, hs2), Some(obj(bruna)));
}

#[test]
fn djeru_and_hazoret_keeps_attacking_untapped_after_losing_haste_and_vigilance() {
    cr!("506.4", "508.1f", "702.10b");
    ruling!(
        "Djeru and Hazoret",
        "Once Djeru and Hazoret has legally attacked during the turn it came under your control, causing it to lose haste by adding cards to your hand won’t cause it to stop attacking. Similarly, causing it to lose vigilance after it has attacked won’t cause it to become tapped."
    );
    let mut t = TestGame::new(2);
    let dj = t.battlefield_sick(P0, "Djeru and Hazoret");
    assert!(has_kw(&t, dj, KeywordKind::Haste));
    attack_with(&mut t, &[(dj, pl(P1))]);
    assert_eq!(attacking(&t, dj), Some(pl(P1)));
    assert!(!t.obj(dj).tapped);
    t.resolve_all();
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.g.recompute();
    assert!(!has_kw(&t, dj, KeywordKind::Haste));
    assert!(!has_kw(&t, dj, KeywordKind::Vigilance));
    assert_eq!(attacking(&t, dj), Some(pl(P1)));
    assert!(!t.obj(dj).tapped);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn hugs_extra_land_adds_up_with_exploration() {
    cr!("305.2", "305.2b");
    ruling!(
        "Hugs, Grisly Guardian",
        "The effect of Hugs's last ability that allows you to play an additional land is cumulative with similar effects. For example, if you control Hugs as well as Exploration (an enchantment with \"You may play an additional land on each of your turns.\"), you'll be able to play three lands during each of your turns."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hugs, Grisly Guardian");
    t.battlefield(P0, "Exploration");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        assert!(t.play_land(P0, *l).is_ok());
    }
    assert!(t.play_land(P0, lands[3]).is_err());
}

#[test]
fn crackling_drake_power_is_defined_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Crackling Drake",
        "The ability that defines Crackling Drake's power works in all zones."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Divination");
    t.exile(P0, "Cancel");
    t.graveyard(P1, "Lightning Bolt");
    for zone in [
        Zone::Hand(P0),
        Zone::Graveyard(P0),
        Zone::Library(P0),
        Zone::Exile,
    ] {
        let d = t.custom(P0, (*card("Crackling Drake")).clone(), zone);
        t.g.recompute();
        assert_eq!(t.obj(d).power(), 3, "{zone:?}");
    }
}

// ---------------------------------------------------------------------------------------
// Emrakul, the Promised End (multiplayer)
// ---------------------------------------------------------------------------------------

/// `p` casts Emrakul, the Promised End targeting `target` and it resolves.
fn emrakul(t: &mut TestGame, p: PlayerId, target: PlayerId) {
    let e = t.hand(p, "Emrakul, the Promised End");
    mana(t, p, ManaType::C, 13);
    t.answer_targets(p, &[pl(target)]);
    t.cast(p, e).go();
    t.resolve_all();
}

#[test]
fn emrakul_extra_turn_never_begins_if_the_player_lost() {
    cr!("723.1", "800.4a", "800.4k", "500.7");
    ruling!(
        "Emrakul, the Promised End",
        "If the targeted player loses the game while you control their turn during a multiplayer game, the extra turn never begins."
    );
    let mut t = TestGame::new(3);
    emrakul(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    t.g.lose_life(P1, 20);
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P1));
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(t.g.turn.active, P2);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.turn.active, P0);
}

#[test]
fn emrakul_later_control_effect_wins_but_each_gives_an_extra_turn() {
    cr!("723.1a", "500.7");
    ruling!(
        "Emrakul, the Promised End",
        "Multiple player-controlling effects that affect the same player overwrite each other. The last one to be created is the one that works. If multiple players have cast Emrakul and targeted the same player, each ability’s effect will create an extra turn."
    );
    let mut t = TestGame::new(3);
    emrakul(&mut t, P0, P2);
    t.advance_to(P1, Step::PrecombatMain);
    emrakul(&mut t, P1, P2);
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P1);
    // P2's turn (controlled by P1), then two extra turns for P2, neither controlled.
    let mut turns = vec![];
    for _ in 0..3 {
        let n = t.g.turn.number;
        t.g.run_until(10_000, |g| {
            g.turn.number != n && g.turn.step == Step::Upkeep
        });
        turns.push((t.g.turn.active, decider(&t.g, t.g.turn.active)));
    }
    assert_eq!(turns, vec![(P2, P2), (P2, P2), (P0, P0)]);
}

// ---------------------------------------------------------------------------------------
// Two-Headed Giant: Dragon Broodmother, Slitherwisp, Nicol Bolas, Dragon-God
// ---------------------------------------------------------------------------------------

#[test]
fn dragon_broodmother_triggers_once_per_team_upkeep_and_can_be_devoured() {
    cr!("810.2", "503.1a", "702.82a");
    ruling!(
        "Dragon Broodmother",
        "In a Two-Headed Giant game, Dragon Broodmother’s second ability triggers only once at the beginning of each team’s upkeep, not once for each player."
    );
    ruling!(
        "Dragon Broodmother",
        "Putting a token creature with devour onto the battlefield works the same as putting any other creature with devour onto the battlefield. The token may devour Dragon Broodmother itself!"
    );
    supported("Dragon Broodmother");
    let mut t = thg();
    let mother = t.battlefield(P0, "Dragon Broodmother");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The token devours Dragon Broodmother.
    t.answer_choose(P0, &[obj(mother)]);
    t.resolve_all();
    assert!(!t.on_battlefield(mother));
    let dragons = with_subtype(&t, P0, "Dragon");
    assert_eq!(dragons.len(), 1);
    assert_eq!(t.counters(dragons[0], "+1/+1"), 2);
    // Next team's upkeep: no Broodmother, no trigger. With another Broodmother: once.
    t.battlefield(P0, "Dragon Broodmother");
    t.advance_to(P2, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn slitherwisp_in_two_headed_giant() {
    cr!("810.9", "810.2");
    ruling!(
        "Slitherwisp",
        "In a Two-Headed Giant game, Slitherwisp’s last ability causes the opposing team to lose 2 life and you to draw one card."
    );
    let mut t = thg();
    t.battlefield(P0, "Slitherwisp");
    let viper = t.hand(P0, "Ambush Viper");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let life = t.life(P2);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.cast(P0, viper).go();
    t.resolve_all();
    assert_eq!(t.life(P2), life - 2);
    assert_eq!(t.life(P3), life - 2);
    assert_eq!(t.hand_size(P0), h0 - 1 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

/// Nicol Bolas, Dragon-God for `p` with `loyalty` loyalty counters.
fn bolas(t: &mut TestGame, p: PlayerId, loyalty: u32) -> ObjectId {
    crate::r_p007_common::pw(t, p, "Nicol Bolas, Dragon-God", loyalty)
}

#[test]
fn nicol_bolas_ultimate_in_two_headed_giant_takes_the_whole_team() {
    cr!("810.8a", "104.3e");
    ruling!(
        "Nicol Bolas, Dragon-God",
        "In a Two-Headed Giant game, if one player on a team loses the game, the entire team loses the game, even if the other player controls a legendary creature or planeswalker."
    );
    supported("Nicol Bolas, Dragon-God");
    let mut t = thg();
    let b = bolas(&mut t, P0, 8);
    t.battlefield(P2, "Isamaru, Hound of Konda");
    activate_containing(&mut t, P0, b, "loses the game").unwrap();
    t.resolve_all();
    assert!(t.has_lost(P2));
    assert!(t.has_lost(P3));
    assert!(!t.has_lost(P0));
}

#[test]
fn nicol_bolas_draw_triggers_wait_for_the_exiles() {
    cr!("608.2", "603.3");
    ruling!(
        "Nicol Bolas, Dragon-God",
        "If any abilities trigger as you draw a card while resolving Nicol Bolas's first loyalty ability, those abilities don't resolve until after your opponents have exiled a card or permanent."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Parun");
    let b = bolas(&mut t, P0, 4);
    let bears = t.hand(P1, "Grizzly Bears");
    activate_containing(&mut t, P0, b, "You draw a card").unwrap();
    t.answer_choose(P1, &[obj(bears)]);
    t.g.resolve_top();
    // The opponent exiled a card; the draw trigger waits.
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.g.stack.is_empty());
    t.answer_targets(P0, &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn nicol_bolas_linked_abilities_he_gains_see_only_their_own_exiles() {
    cr!("607.2a", "607.5");
    ruling!(
        "Nicol Bolas, Dragon-God",
        "If an ability of a planeswalker exiles cards and another loyalty ability of that planeswalker references the exiled cards, those abilities are linked. If Nicol Bolas gains both abilities from the same planeswalker, the instances he has are similarly linked. Cards exiled by any other abilities Nicol Bolas has (such as his own first loyalty ability) won't be seen by those linked abilities."
    );
    supported("Ashiok, Nightmare Weaver");
    // Three Craw Wurms (mana value 6) are exiled: by Ashiok's own +2, by Bolas's own +1,
    // and by the +2 Bolas has from Ashiok. Ashiok's −X through Bolas with X = 6 sees only
    // the last one.
    let mut t = TestGame::new(2);
    let ashiok = crate::r_p007_common::pw(&mut t, P1, "Ashiok, Nightmare Weaver", 3);
    let b = bolas(&mut t, P0, 6);
    // Ashiok's own +2 (P1's, on P0's library).
    let by_ashiok = stack_library(&mut t, P0, &["Craw Wurm"])[0];
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[pl(P0)]);
    activate_containing(&mut t, P1, ashiok, "Exile the top three").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(by_ashiok), Zone::Exile);
    // Bolas's +1: P1 exiles a Craw Wurm from their hand.
    t.set_step(P0, Step::PrecombatMain);
    let by_plus_one = t.hand(P1, "Craw Wurm");
    t.answer_choose(P1, &[obj(by_plus_one)]);
    activate_containing(&mut t, P0, b, "You draw a card").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(by_plus_one), Zone::Exile);
    // Next turn: Bolas uses Ashiok's +2 on P1's library.
    let by_bolas_ashiok = stack_library(&mut t, P1, &["Craw Wurm"])[0];
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, b, "Exile the top three").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(by_bolas_ashiok), Zone::Exile);
    // Next turn: Ashiok's −X through Bolas, X = 6.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let seen = obj(t.g.current(by_bolas_ashiok));
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(6));
    activate_containing(&mut t, P0, b, "onto the battlefield").unwrap();
    t.resolve_all();
    for c in entity_choices(&t, from) {
        assert_eq!(c, vec![seen]);
    }
    let found = t.g.current(by_bolas_ashiok);
    assert!(t.on_battlefield(found));
    assert_eq!(t.obj(found).controller, P0);
    assert_eq!(t.zone(by_ashiok), Zone::Exile);
    assert_eq!(t.zone(by_plus_one), Zone::Exile);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

// ---------------------------------------------------------------------------------------
// Vraska, Nissa, Jace
// ---------------------------------------------------------------------------------------

#[test]
fn vraska_poison_difference_then_replacement_effects() {
    cr!("614.1a");
    ruling!(
        "Vraska, Betrayal's Sting",
        "In most cases, the target of Vraska's last loyalty ability will end up with nine poison counters. However, once you calculate what the difference is, replacement effects can change how many poison counters the player actually gets. For example, if the player controls Melira, the Living Cure, they will end up getting one poison counter if they had eight or fewer to begin with."
    );
    supported("Vraska, Betrayal's Sting");
    supported("Melira, the Living Cure");
    for (melira, after) in [(false, 9), (true, 4)] {
        let mut t = TestGame::new(2);
        let v = crate::r_p007_common::pw(&mut t, P0, "Vraska, Betrayal's Sting", 9);
        t.g.add_counters(pl(P1), counters::POISON, 3, None);
        t.g.flush_events();
        if melira {
            t.battlefield(P1, "Melira, the Living Cure");
        }
        t.answer_targets(P0, &[pl(P1)]);
        activate_containing(&mut t, P0, v, "poison").unwrap();
        t.resolve_all();
        assert_eq!(t.g.player(P1).counter(counters::POISON), after, "{melira}");
    }
}

#[test]
fn vraska_treasure_keeps_supertypes_only() {
    cr!("205.1a", "613.1d");
    ruling!(
        "Vraska, Betrayal's Sting",
        "The target of Vraska's second loyalty ability will lose any other subtypes and card types it previously had and will be only a Treasure artifact. It will retain any supertypes it had."
    );
    let mut t = TestGame::new(2);
    let v = crate::r_p007_common::pw(&mut t, P0, "Vraska, Betrayal's Sting", 6);
    let dog = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.answer_targets(P0, &[obj(dog)]);
    activate_containing(&mut t, P0, v, "Treasure").unwrap();
    t.resolve_all();
    let c = &t.obj(dog).chars;
    assert!(c.is(CardType::Artifact));
    assert!(!c.is(CardType::Creature));
    assert_eq!(c.card_types.count(), 1);
    assert_eq!(
        c.subtypes.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        ["Treasure"]
    );
    assert!(c.supertypes.contains(Supertype::Legendary));
}

/// `name` cast by P0 with `phyrexian` of its Phyrexian symbols paid with 2 life, with
/// Doubling Season on the battlefield, P0 applying compleated first or not; returns its
/// loyalty once it resolves.
fn compleated_with_doubling_season(
    name: &str,
    generic: u32,
    colored: (ManaType, u32),
    phyrexian: usize,
    compleated_first: bool,
) -> u32 {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let pw = t.hand(P0, name);
    mana(&mut t, P0, colored.0, colored.1);
    mana(&mut t, P0, ManaType::C, generic);
    for _ in 0..phyrexian {
        t.answer(P0, DecisionKind::Option, Answer::Index(2));
    }
    // The options: Doubling Season's effect, then compleated.
    t.answer(
        P0,
        DecisionKind::Replacement,
        Answer::Index(if compleated_first { 1 } else { 0 }),
    );
    let from = t.asked().len();
    t.cast(P0, pw).go();
    t.resolve_all();
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseReplacement { options } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1, "{name}");
    assert!(options[0][0].starts_with("Doubling Season"), "{name}");
    assert!(options[0][1].contains("Compleated"), "{name}");
    t.counters(pw, "loyalty")
}

#[test]
fn compleated_and_other_loyalty_replacements_both_apply() {
    cr!("702.150a", "616.1", "616.1e", "616.1f", "306.5b");
    interp!("compleated-is-ordered-with-other-replacements");
    ruling!(
        "Nissa, Ascended Animist",
        "Other replacement effects that would change the number of loyalty counters Nissa enters with will apply as normal."
    );
    ruling!(
        "Jace, the Perfected Mind",
        "Other replacement effects that would change the number of loyalty counters Jace enters with will apply as normal."
    );
    // The controller orders the two replacement effects (CR 616.1). Jace: 5, two fewer
    // for the life paid, then doubled: 6; doubled first: 8.
    let jace = |first| {
        compleated_with_doubling_season("Jace, the Perfected Mind", 2, (ManaType::U, 1), 1, first)
    };
    assert_eq!(jace(true), 6);
    assert_eq!(jace(false), 8);
    // Nissa: 7, four fewer, then doubled: 6; doubled first: 10.
    let nissa = |first| {
        compleated_with_doubling_season("Nissa, Ascended Animist", 3, (ManaType::G, 2), 2, first)
    };
    assert_eq!(nissa(true), 6);
    assert_eq!(nissa(false), 10);
}

#[test]
fn nissa_bonuses_and_token_size_are_fixed_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Nissa, Ascended Animist",
        "The power and toughness bonus granted to creatures by Nissa's last ability is determined only once, as that ability resolves. It won't increase or decrease if the number of Forests you control later changes. Similarly, it applies only to creatures you control at the time it resolves. Creatures that come under your control after it resolves will not have the bonus."
    );
    ruling!(
        "Nissa, Ascended Animist",
        "The token created by Nissa's first ability has its power and toughness set only once, at the time the ability resolves. It doesn't change later as loyalty counters are added to or removed from Nissa."
    );
    supported("Nissa, Ascended Animist");
    // +1: an X/X Horror where X is Nissa's loyalty as it resolves (5 + 1).
    let mut t = TestGame::new(2);
    let n = crate::r_p007_common::pw(&mut t, P0, "Nissa, Ascended Animist", 5);
    activate_containing(&mut t, P0, n, "Horror").unwrap();
    t.resolve_all();
    let horror = with_subtype(&t, P0, "Horror");
    assert_eq!(horror.len(), 1);
    assert_eq!(t.pt(horror[0]), (6, 6));
    put(&mut t, n, "loyalty", 3);
    assert_eq!(t.pt(horror[0]), (6, 6));
    // −7 with two Forests: +2/+2, fixed.
    let mut t = TestGame::new(2);
    let n = crate::r_p007_common::pw(&mut t, P0, "Nissa, Ascended Animist", 7);
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, n, "for each Forest").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    t.lands(P0, "Forest", 1);
    assert_eq!(t.pt(bears), (4, 4));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!has_kw(&t, late, KeywordKind::Trample));
}

#[test]
fn jace_minus_two_checks_every_graveyard() {
    cr!("608.2c");
    ruling!(
        "Jace, the Perfected Mind",
        "Jace's second loyalty ability doesn't care which player's graveyard has twenty or more cards in it. It could be the target player's or another player's."
    );
    supported("Jace, the Perfected Mind");
    for (yours, drawn) in [(20usize, 3usize), (19, 1)] {
        let mut t = TestGame::new(2);
        let j = crate::r_p007_common::pw(&mut t, P0, "Jace, the Perfected Mind", 5);
        for _ in 0..yours {
            t.graveyard(P0, "Grizzly Bears");
        }
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[pl(P1)]);
        activate_containing(&mut t, P0, j, "mills three cards. Then").unwrap();
        t.resolve_all();
        assert_eq!(t.graveyard_size(P1), 3);
        assert_eq!(t.hand_size(P0), hand + drawn, "{yours}");
    }
}
