//! Rulings batch P007 — triggered damage: additional triggers (CR 603.2d), simultaneous
//! triggers (CR 603.3b), reflexive triggers (CR 603.12), delayed triggers (CR 603.7),
//! "is dealt damage" triggers (CR 120.3), and spell-copy triggers (CR 707.10).

use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, custom_card, supported};
use crate::r_s02_common::destroy;
use crate::r_s07_common::damage_on;
use crate::r_s25_common::{cast_new, spell_copies};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn annie_doubles_the_trigger_but_not_enters_with_counters() {
    cr!("603.2d", "614.1c", "603.3d");
    ruling!(
        "Annie Joins Up",
        "Abilities that apply “as [this creature] enters the battlefield” or “as [this creature] is turned face up” are also unaffected."
    );
    ruling!(
        "Annie Joins Up",
        "Annie Joins Up’s last ability doesn’t copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    supported("Annie Joins Up");
    supported("Morlun, Devourer of Spiders");
    // Morlun, X = 2, in a three-player game: two counters (not four), and the "enters"
    // trigger twice, each instance with its own target opponent.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Annie Joins Up");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P2)]);
    cast_x(
        &mut t,
        P0,
        "Morlun, Devourer of Spiders",
        NORMAL,
        Some(2),
        &[],
    );
    t.resolve();
    let morlun = t.named_on_battlefield("Morlun, Devourer of Spiders")[0];
    assert_eq!(
        t.counters(morlun, "+1/+1"),
        2,
        "the replacement isn't doubled"
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
}

#[test]
fn annie_doubles_face_up_triggers_only_of_legendary_creatures() {
    cr!("603.2d", "708.8");
    ruling!(
        "Annie Joins Up",
        "Abilities that apply “when [this creature] is turned face up” will trigger an additional time only if that creature is legendary once it has been turned face up."
    );
    let text = "Morph {R}\nWhen this creature is turned face up, it deals 1 damage to any target.";
    for legendary in [true, false] {
        let tl = if legendary {
            "Legendary Creature — Goblin"
        } else {
            "Creature — Goblin"
        };
        let def = custom_card("Test Morph", tl, "{2}{R}", Some((2, 2)), text);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Annie Joins Up");
        let m = t.custom(P0, def, Zone::Battlefield);
        assert!(mtg_engine::facedown::turn_face_down(&mut t.g, m));
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[pl(P1)]);
        assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, m, P0));
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P1), if legendary { 18 } else { 19 }, "{legendary}");
    }
}

#[test]
fn hedge_mage_triggers_are_simultaneous_optional_and_targeted() {
    cr!("603.3b", "603.3d", "603.5");
    ruling!(
        "Duergar Hedge-Mage",
        "Both abilities trigger at the same time. You can put them on the stack in any order."
    );
    ruling!(
        "Duergar Hedge-Mage",
        "Both abilities are optional; you choose whether to use them when they resolve. If an ability has a target, you must choose a target even if you don't plan to use the ability."
    );
    supported("Duergar Hedge-Mage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 2);
    let art = t.battlefield(P1, "Ornithopter");
    let ench = t.battlefield(P1, "Tainted Remedy");
    t.answer_targets(P0, &[obj(art)]);
    t.answer_targets(P0, &[obj(ench)]);
    let from = t.asked().len();
    t.enter(P0, "Duergar Hedge-Mage");
    t.settle();
    // Both on the stack, each with its target, after the controller ordered them.
    assert_eq!(t.stack_len(), 2);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    // Decline the top one, accept the other.
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.resolve_all();
    let gone = [art, ench]
        .iter()
        .filter(|o| !t.on_battlefield(**o))
        .count();
    assert_eq!(gone, 1, "one ability used, the other declined");
}

#[test]
fn spiteful_sliver_trigger_is_noncombat_and_once_per_simultaneous_damage() {
    cr!("510.2", "120.3", "603.2c");
    ruling!(
        "Spiteful Sliver",
        "Damage dealt by the Sliver due to its triggered ability isn’t combat damage, even if it was combat damage that caused that ability to trigger."
    );
    ruling!(
        "Spiteful Sliver",
        "If a Sliver is dealt damage by multiple sources at once (such as by two creatures blocking it), its ability triggers once and one target is dealt that much damage."
    );
    supported("Spiteful Sliver");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Spiteful Sliver");
    let a = t.battlefield(P1, "Savannah Lions");
    let b = t.battlefield(P1, "Savannah Lions");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(ogre, pl(P1))]);
    t.answer_targets(P0, &[pl(P1)]);
    block_and_finish(&mut t, P1, &[(a, ogre), (b, ogre)]);
    assert_eq!(t.life(P1), 16, "one trigger, 4 damage");
    let sliver_damage: Vec<bool> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage {
                    target: Entity::Player(p),
                    combat,
                    ..
                } if *p == P1 => Some(*combat),
                _ => None,
            })
            .collect();
    assert_eq!(sliver_damage, vec![false]);
}

#[test]
fn mogg_maniac_is_dealt_only_the_damage_assigned_to_it() {
    cr!("702.19b", "510.1c");
    ruling!(
        "Mogg Maniac",
        "If it blocks a trample creature, it only gets dealt damage equal to what is assigned to it. This means it will likely only get to deal 1 damage to the opponent."
    );
    supported("Mogg Maniac");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let maniac = t.battlefield(P1, "Mogg Maniac");
    attack_with(&mut t, &[(maw, pl(P1))]);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 5]));
    t.answer_targets(P1, &[pl(P0)]);
    block_and_finish(&mut t, P1, &[(maniac, maw)]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn several_simultaneous_events_trigger_once_each() {
    cr!("603.2c", "603.3d");
    ruling!(
        "Mogg Bombers",
        "If multiple creatures enter at the same time, the ability triggers once for each of them. You may choose a different target player or planeswalker for each time it triggers."
    );
    ruling!(
        "Cavalcade of Calamity",
        "If multiple creatures you control with power 1 or less attack, Cavalcade of Calamity’s ability will trigger for each of those creatures. Each ability will cause Cavalcade of Calamity to deal 1 damage to the appropriate player or planeswalker."
    );
    supported("Mogg Bombers");
    supported("Cavalcade of Calamity");
    // Mogg Bombers: Raise the Alarm makes two Soldiers at once; two triggers, one at P1 and
    // one at P1's planeswalker.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mogg Bombers");
    let walker = pw(&mut t, P1, "Vivien, Arkbow Ranger", 4);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(walker)]);
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "two triggers");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(walker, "loyalty"), 1);
    // Cavalcade: two 1-power attackers, one at P1 and one at P1's planeswalker.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalcade of Calamity");
    let a = t.battlefield(P0, "Memnite");
    let b = t.battlefield(P0, "Memnite");
    let walker = pw(&mut t, P1, "Vivien, Arkbow Ranger", 4);
    attack_with(&mut t, &[(a, pl(P1)), (b, obj(walker))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(walker, "loyalty"), 3);
}

#[test]
fn ral_triggers_for_each_copy_of_a_spell() {
    cr!("707.10", "702.40a", "603.2");
    ruling!(
        "Ral, Storm Conduit",
        "If an effect copies a spell multiple times, as Finale of Promise may, Ral's first ability triggers that many times."
    );
    supported("Ral, Storm Conduit");
    // Grapeshot after two other spells: cast (1) + two storm copies (2).
    let mut t = TestGame::new(2);
    pw(&mut t, P0, "Ral, Storm Conduit", 4);
    for _ in 0..3 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "two Shocks and two Ral triggers");
    for _ in 0..8 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Grapeshot", &[pl(P1)]);
    t.settle();
    t.resolve_all();
    // Grapeshot and its two copies (3) and three Ral triggers (3).
    assert_eq!(t.life(P1), 8);
}

#[test]
fn ral_doesnt_trigger_for_copying_a_card_only_for_casting_the_copy() {
    cr!("707.12", "603.2");
    ruling!(
        "Ral, Storm Conduit",
        "If an effect copies a card rather than a spell (such as that of God-Eternal Kefnet), this doesn't cause Ral's first ability to trigger. That ability will trigger if you cast the copy, however."
    );
    supported("Isochron Scepter");
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        pw(&mut t, P0, "Ral, Storm Conduit", 4);
        let shock = t.hand(P0, "Shock");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(shock)]);
        let scepter = t.enter(P0, "Isochron Scepter");
        t.resolve_all();
        assert_eq!(t.zone(shock), Zone::Exile);
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, true);
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[pl(P1)]);
        t.activate(P0, scepter, 0, &[]).expect("Scepter");
        t.resolve();
        if cast {
            assert_eq!(t.stack_len(), 2, "the copy and one Ral trigger");
            t.resolve_all();
            assert_eq!(t.life(P1), 17);
        } else {
            assert_eq!(t.stack_len(), 0, "copying the card doesn't trigger Ral");
            assert_eq!(t.life(P1), 20);
        }
    }
}

#[test]
fn reflexive_triggers_go_on_the_stack_after_the_action() {
    cr!("603.12");
    ruling!(
        "Fireblade Artist",
        "Fireblade Artist's triggered ability goes on the stack without a target. While that ability is resolving, you may sacrifice a creature. When you do, the reflexive triggered ability triggers and you pick a target opponent or planeswalker to be dealt damage."
    );
    ruling!(
        "Torrent Sculptor // Flamethrower Sonata",
        "Flamethrower Sonata goes on the stack without a target. While it is resolving, you discard a card, then draw a card. When you discard an instant or sorcery card this way, the reflexive triggered ability triggers and you pick a target to be dealt damage."
    );
    supported("Fireblade Artist");
    // Fireblade Artist: the upkeep trigger has no target; after the sacrifice, the
    // reflexive trigger waits on the stack (players may respond) before the damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fireblade Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert!(crate::r_s25_common::targets_of(&t, top).is_empty());
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(
        t.stack_len(),
        1,
        "the reflexive trigger waits for responses"
    );
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // Flamethrower Sonata: discard Lava Spike (mana value 1); the reflexive trigger is on
    // the stack after the spell resolved, then deals 1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spike = t.hand(P0, "Lava Spike");
    t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(spike)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Torrent Sculptor // Flamethrower Sonata");
    let s = t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve();
    assert!(crate::r_s07_common::resolved(&t, s));
    assert_eq!(t.stack_len(), 1, "the reflexive trigger");
    assert_eq!(damage_on(&t, bears), 0);
    t.resolve();
    assert_eq!(damage_on(&t, bears), 1);
}

#[test]
fn fiery_encore_copies_each_resolve_their_reflexive_trigger_first() {
    cr!("603.12", "702.40a", "608.2");
    ruling!(
        "Fiery Encore",
        "Each copy will resolve one at a time. As each copy resolves, if you discarded a nonland card, the reflexive triggered ability triggers and you pick a target to be dealt damage. That ability will resolve before you deal with the next copy of Fiery Encore."
    );
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    let a = t.hand(P0, "Lava Spike");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(maw)]);
    let fe = cast_new(&mut t, P0, "Fiery Encore", &[]);
    // The storm trigger resolves: one copy.
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    // The copy resolved: its reflexive trigger is above the original Fiery Encore.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], fe);
    t.resolve();
    assert_eq!(damage_on(&t, maw), 1);
    assert_eq!(t.g.stack, vec![fe]);
}

#[test]
fn devouring_tendrils_delayed_trigger_any_death() {
    cr!("603.7c");
    ruling!(
        "Devouring Tendrils",
        "Devouring Tendrils creates a delayed triggered ability that triggers whenever the target creature or planeswalker you don’t control dies this turn, even if it didn’t die because of the damage dealt as Devouring Tendrils resolved."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Devouring Tendrils", &[obj(bears), obj(maw)]);
    t.resolve_all();
    assert!(t.on_battlefield(maw));
    assert_eq!(t.life(P0), 20);
    destroy(&mut t, maw);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn chandras_incinerator_counts_damage_not_life_change() {
    cr!("601.2f", "120.3");
    ruling!(
        "Chandra's Incinerator",
        "Chandra's Incinerator counts all noncombat damage dealt to your opponents, not the difference in their life totals since the turn began."
    );
    supported("Chandra's Incinerator");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(src, pl(P1), 5, false);
    t.g.flush_events();
    t.g.gain_life(P1, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P1), 18);
    t.lands(P0, "Mountain", 1);
    let card = t.hand(P0, "Chandra's Incinerator");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Chandra's Incinerator").len(), 1);
}
