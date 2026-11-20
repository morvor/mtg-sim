//! Item `rulings-residue-keywords` — fight interactions (CR 701.14): illegal targets and
//! sources that left (608.2b, 701.14b), what happens during one resolution (no priority
//! between, triggers after it), simultaneity, choices made as a "may" resolves, fights
//! as attackers, copies, and replacement effects around fights.

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Grizzly Bears and P1's Hill Giant, with a Colossal Dreadmaw for ferocious and
/// delirium in P0's graveyard; P0 casts `name` on the two, makes the targets in `illegal`
/// (0: P0's, 1: P1's) illegal, and resolves everything. Returns (game, mine, theirs).
fn two_targets(
    name: &str,
    illegal: usize,
    entered_this_turn: bool,
) -> (TestGame, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    delirium_graveyard(&mut t, P0);
    let mine = if entered_this_turn {
        t.enter(P0, "Grizzly Bears")
    } else {
        t.battlefield(P0, "Grizzly Bears")
    };
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, name, &[obj(mine), obj(theirs)]);
    make_illegal(&mut t, [mine, theirs][illegal]);
    t.resolve_all();
    (t, mine, theirs)
}

#[test]
fn savage_punch_bonus_depends_only_on_its_own_target() {
    cr!("608.2b", "701.14b", "207.2c");
    ruling!(
        "Savage Punch",
        "Assuming the ferocious ability applies, if just the creature you control is an illegal target when Savage Punch tries to resolve, it won’t get +2/+2 until end of turn. If the creature you control is a legal target but the creature you don’t control isn’t, the creature you control will get +2/+2 until end of turn."
    );
    let (mut t, mine, theirs) = two_targets("Savage Punch", 0, false);
    assert_eq!(pt_now(&mut t, mine), (2, 2));
    assert_eq!(damage_on(&t, mine) + damage_on(&t, theirs), 0);
    let (mut t, mine, theirs) = two_targets("Savage Punch", 1, false);
    assert_eq!(pt_now(&mut t, mine), (4, 4));
    // No fight: neither deals damage.
    assert_eq!(damage_on(&t, mine) + damage_on(&t, theirs), 0);
}

#[test]
fn beastie_beatdown_counters_need_the_creature_you_control_to_be_legal() {
    cr!("608.2b", "207.2c");
    ruling!(
        "Beastie Beatdown",
        "If either creature is an illegal target as Beastie Beatdown tries to resolve, the creature you control won't deal damage. If the creature you control is an illegal target, you won't put two +1/+1 counters on it even if you have delirium."
    );
    let (t, mine, theirs) = two_targets("Beastie Beatdown", 0, false);
    assert_eq!(p1(&t, mine), 0);
    assert_eq!(damage_on(&t, theirs), 0);
    let (t, mine, theirs) = two_targets("Beastie Beatdown", 1, false);
    assert_eq!(p1(&t, mine), 2);
    assert_eq!(damage_on(&t, theirs), 0);
}

#[test]
fn malamet_battle_glyph_counter_without_the_fight() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Malamet Battle Glyph",
        "If the target creature you control entered the battlefield this turn and is still a legal target as Malamet Battle Glyph resolves but the target creature you don't control isn't, you will still put a +1/+1 counter on the creature you control. The creatures will not fight each other."
    );
    let (t, mine, theirs) = two_targets("Malamet Battle Glyph", 1, true);
    assert_eq!(p1(&t, mine), 1);
    assert_eq!(damage_on(&t, mine) + damage_on(&t, theirs), 0);
}

#[test]
fn earth_rumble_fight_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "701.14b", "603.12");
    ruling!(
        "Earth Rumble",
        "If either target is an illegal target as the fight ability tries to resolve, neither creature will deal or be dealt damage."
    );
    supported("Earth Rumble");
    // 0, 1: that target is illegal; 2: both are legal (the fight happens: the reflexive
    // ability triggers once the land was earthbent).
    for illegal in 0..3 {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, "Forest");
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, "Earth Rumble", &[obj(land)]);
        // The reflexive ability's targets: P0's Giant, then P1's Bears.
        t.answer_targets(P0, &[obj(mine)]);
        t.answer_targets(P0, &[obj(theirs)]);
        t.resolve();
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        if illegal == 2 {
            t.resolve_all();
            assert!(!t.on_battlefield(theirs));
            assert_eq!(damage_on(&t, mine), 2);
            continue;
        }
        make_illegal(&mut t, [mine, theirs][illegal]);
        t.resolve_all();
        assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
        assert_eq!(damage_on(&t, mine) + damage_on(&t, theirs), 0);
    }
}

#[test]
fn scab_clan_giant_that_left_or_an_illegal_target_means_no_fight() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Scab-Clan Giant",
        "If Scab-Clan Giant leaves the battlefield before its ability resolves, no fight will happen and neither creature will deal damage. Also, if the target creature is an illegal target when the ability tries to resolve, it won't resolve and no fight will happen."
    );
    supported("Scab-Clan Giant");
    for case in 0..2 {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Hill Giant");
        let giant = t.enter(P0, "Scab-Clan Giant");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if case == 0 {
            move_to(&mut t, giant, Zone::Hand(P0));
        } else {
            make_illegal(&mut t, theirs);
        }
        t.resolve_all();
        assert_eq!(damage_on(&t, theirs), 0);
        if case == 1 {
            assert_eq!(damage_on(&t, giant), 0);
        }
    }
}

#[test]
fn foe_razer_regent_doesnt_fight_an_illegal_target_so_nothing_triggers() {
    cr!("608.2b", "701.14b", "603.2");
    ruling!(
        "Foe-Razer Regent",
        "If the target creature you don’t control becomes an illegal target of the first triggered ability before it resolves, Foe-Razer Regent won’t fight and its last ability won’t trigger."
    );
    supported("Foe-Razer Regent");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(theirs)]);
        t.answer_yes(P0, true);
        let regent = t.enter(P0, "Foe-Razer Regent");
        t.settle();
        if illegal {
            make_illegal(&mut t, theirs);
        }
        t.resolve_all();
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let expected = if illegal { 0 } else { 2 };
        assert_eq!(p1(&t, regent), expected, "illegal: {illegal}");
        assert_eq!(t.on_battlefield(theirs), illegal);
    }
}

#[test]
fn outmuscle_indestructible_saves_the_creature_after_the_fight() {
    cr!("704.3", "702.12b", "608.2c");
    ruling!(
        "Outmuscle",
        "Because state-based actions don't destroy a creature with lethal damage marked on it until a player would receive priority, gaining indestructible from Outmuscle will save your creature if it's dealt lethal damage in the fight."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Three green mana spent: adamant.
    t.lands(P0, "Forest", 4);
    let o = t.hand(P0, "Outmuscle");
    t.cast(P0, o).target(mine).target(theirs).go();
    t.resolve_all();
    // The Bears (3/3 with its counter) took 3 damage in the fight, then became
    // indestructible before state-based actions were checked.
    assert!(t.on_battlefield(mine));
    assert_eq!(damage_on(&t, mine), 3);
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn unnatural_aggression_exiles_the_creature_if_it_dies_later_in_the_turn() {
    cr!("614.1a", "701.14a");
    ruling!(
        "Unnatural Aggression",
        "If the creature an opponent controls survives the fight but would die later in the turn for another reason, it will be exiled instead, whether your creature dealt it damage or not."
    );
    supported("Unnatural Aggression");
    // P0's Hill Giant (3 damage) and P0's Ornithopter (no damage) each fight a Craw Wurm
    // (6/4) that survives.
    for mine_name in ["Hill Giant", "Ornithopter"] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, mine_name);
        let theirs = t.battlefield(P1, "Craw Wurm");
        cast_new(
            &mut t,
            P0,
            "Unnatural Aggression",
            &[obj(mine), obj(theirs)],
        );
        t.resolve_all();
        assert!(t.on_battlefield(theirs));
        destroy(&mut t, theirs);
        assert_eq!(t.zone(theirs), Zone::Exile, "{mine_name}");
    }
}

#[test]
fn the_last_agni_kai_excess_damage_with_deathtouch() {
    cr!("120.4a", "120.10");
    ruling!(
        "The Last Agni Kai",
        "In some cases, the creature you control that is fighting may have deathtouch. Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage, so any amount greater than that will cause excess damage to be dealt, even if the total amount of damage isn't greater than the opponent's creature's toughness."
    );
    supported("The Last Agni Kai");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    grant_kw(&mut t, mine, KeywordKind::Deathtouch);
    cast_new(&mut t, P0, "The Last Agni Kai", &[obj(mine), obj(theirs)]);
    t.resolve();
    // 3 damage to a 6/4 from a deathtouch source: 2 of it is excess.
    assert_eq!(mana(&t, P0, ManaType::R), 2);
    assert!(!t.on_battlefield(theirs));
}

/// Guild Feud's upkeep trigger with `theirs` and `mine` on top of the libraries (each
/// chosen if it's a creature card). Returns the cards.
fn guild_feud(t: &mut TestGame, theirs: &str, mine: &str) -> (ObjectId, ObjectId) {
    supported("Guild Feud");
    t.battlefield(P0, "Guild Feud");
    t.advance_to(P1, Step::End);
    let th = stack_library(t, P1, &[theirs, "Island", "Island"])[0];
    let mi = stack_library(t, P0, &[mine, "Forest", "Forest"])[0];
    t.answer_choose(P1, &[obj(th)]);
    t.answer_choose(P0, &[obj(mi)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (th, mi)
}

#[test]
fn guild_feud_fight_happens_before_enter_triggers_and_without_priority() {
    cr!("701.14a", "603.3", "608.2c", "117.3b");
    ruling!(
        "Guild Feud",
        "Any abilities that trigger when the two creatures enter will be put on the stack after the fight, even if one or both creatures die."
    );
    ruling!(
        "Guild Feud",
        "Players can't cast spells or activate abilities between the creatures being put onto the battlefield and them fighting each other. Notably, you can't respond with a spell like Giant Growth."
    );
    let mut t = TestGame::new(2);
    let (th, mi) = guild_feud(&mut t, "Elvish Visionary", "Hill Giant");
    let hand = t.hand_size(P1);
    let from = t.asked().len();
    t.g.resolve_top();
    // The whole resolution, entering and fighting, offered no one priority.
    assert_eq!(count_asked(&t, from, is_priority), 0);
    let giant = t.g.current(mi);
    assert_eq!(damage_on(&t, giant), 1);
    t.g.flush_events();
    t.settle();
    // The Visionary died in the fight; its enter trigger is put on the stack only now.
    assert_eq!(t.zone(th), Zone::Graveyard(P1));
    assert_eq!(triggers_on_stack(&t, "draw"), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn guild_feud_with_one_creature_doesnt_fight() {
    cr!("701.14a", "608.2c");
    ruling!(
        "Guild Feud",
        "If no creatures or only one creature is put onto the battlefield this way, no fight happens."
    );
    let mut t = TestGame::new(2);
    let (th, mi) = guild_feud(&mut t, "Hill Giant", "Forest");
    t.resolve_all();
    assert!(t.on_battlefield(th));
    assert_eq!(damage_on(&t, t.g.current(th)), 0);
    assert_eq!(t.zone(mi), Zone::Graveyard(P0));
}

/// P1 controls Grizzly Bears and Hill Giant; P0 (with `extra` on the battlefield) casts
/// Ezuri's Predation. Returns the game after the spell (only) resolved.
fn predation(extra: &str) -> (TestGame, ObjectId, ObjectId, usize) {
    supported("Ezuri's Predation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, extra);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Ezuri's Predation", &[]);
    let start = t.g.turn_events.len();
    t.resolve();
    (t, bears, giant, start)
}

#[test]
fn ezuris_predation_fights_at_once_and_triggers_wait_for_it() {
    cr!("701.14a", "603.3", "608.2c");
    ruling!(
        "Ezuri's Predation",
        "Each of the \"fights\" happens at the same time."
    );
    ruling!(
        "Ezuri's Predation",
        "Players can't cast spells or activate any abilities in between the Beasts entering the battlefield and fighting the other creatures. If the Beasts entering the battlefield cause any abilities to trigger, those abilities will be put onto the stack after Ezuri's Predation is finished resolving."
    );
    let (mut t, bears, giant, start) = predation("Soul Warden");
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    // All four fight damage events form one batch (no boundary between them).
    let evs = &t.g.turn_events[start..];
    let dmg: Vec<usize> = evs
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, Event::Damage { .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(dmg.len(), 4);
    assert!(!evs[dmg[0]..=dmg[3]]
        .iter()
        .any(|e| matches!(e, Event::BatchBoundary)));
    // Soul Warden's two triggers (one per Beast) are on the stack only now.
    assert_eq!(t.life(P0), 20);
    assert_eq!(triggers_on_stack(&t, "gain"), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn ezuris_predation_extra_tokens_dont_fight() {
    cr!("701.14a", "614.1a");
    ruling!(
        "Ezuri's Predation",
        "If Ezuri's Predation creates more than one token for any given creature (due to an effect such as the one Doubling Season creates), the extra tokens won't fight any creature."
    );
    let (t, _, _, _) = predation("Doubling Season");
    let beasts = with_subtype(&t, P0, "Beast");
    assert_eq!(beasts.len(), 4);
    let damaged = beasts.iter().filter(|b| damage_on(&t, **b) > 0).count();
    assert_eq!(damaged, 2);
}

#[test]
fn grothama_fights_happen_before_blocks() {
    cr!("508.1m", "506.4", "701.14a", "509.1a");
    ruling!(
        "Grothama, All-Devouring",
        "Creatures fight Grothama before blockers are declared. Any creatures that die in the fight can't be blocked. Any attacking creatures that survive the fight will still be attacking, may be blocked as normal, and will deal damage to the player or planeswalker they're attacking if unblocked."
    );
    supported("Grothama, All-Devouring");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grothama, All-Devouring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    grant_kw(&mut t, giant, KeywordKind::Indestructible);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "fight"), 2);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    // The Bears died before blockers were declared; the Giant survived and still attacks.
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert!(is_attacking(&t, giant));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn grothama_once_defeated_cant_be_fought_by_the_rest() {
    cr!("701.14b", "603.3b", "608.2b");
    ruling!(
        "Grothama, All-Devouring",
        "If multiple creatures attack, each desired fight happens individually in the order the attacking players choose. Once Grothama has been defeated, any remaining attacking creatures can't fight it."
    );
    let mut t = TestGame::new(2);
    let gro = t.battlefield(P1, "Grothama, All-Devouring");
    damage(&mut t, gro, 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    // P0 put the two triggers on the stack in an order of P0's choice.
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    let hand = t.hand_size(P0);
    t.resolve_all();
    // The first fight defeated Grothama (and that Bears); the other Bears didn't fight.
    assert!(!t.on_battlefield(gro));
    let survivors = [a, b].iter().filter(|x| t.on_battlefield(**x)).count();
    assert_eq!(survivors, 1);
    // P0's Bears dealt Grothama 2 damage this turn: P0 drew 2.
    assert_eq!(t.hand_size(P0), hand + 2);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn frontier_siege_and_cherished_hatchling_decide_to_fight_on_resolution() {
    cr!("603.3d", "608.2d", "603.3");
    ruling!(
        "Frontier Siege",
        "For the \"Dragons\" ability, you decide whether the creature with flying will fight the target creature you don't control as the ability resolves."
    );
    ruling!(
        "Cherished Hatchling",
        "For the triggered ability that the entering Dinosaur gains, you choose a target when the ability goes on the stack, but you don't choose until that ability resolves whether those creatures fight."
    );
    supported("Frontier Siege");
    supported("Cherished Hatchling");
    let yes_no = |d: &Decision| matches!(d, Decision::YesNo { .. });
    let targets = |d: &Decision| matches!(d, Decision::ChooseTargets { .. });
    for fight in [true, false] {
        // Frontier Siege (Dragons): Wind Drake enters.
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.enter(P0, "Frontier Siege");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(theirs)]);
        let from = t.asked().len();
        t.enter(P0, "Wind Drake");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        assert!(was_asked(&t, from, targets));
        assert!(!was_asked(&t, from, yes_no));
        t.answer_yes(P0, fight);
        t.resolve_all();
        assert!(was_asked(&t, from, yes_no));
        assert_eq!(t.on_battlefield(theirs), !fight);

        // Cherished Hatchling: it died, then a Dinosaur spell enters.
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Cherished Hatchling");
        destroy(&mut t, h);
        t.resolve_all();
        let theirs = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, "Colossal Dreadmaw", &[]);
        // The Hatchling's cast trigger gives the spell the ability; then the spell resolves.
        t.resolve();
        t.answer_targets(P0, &[obj(theirs)]);
        let from = t.asked().len();
        t.resolve();
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        assert!(was_asked(&t, from, targets));
        assert!(!was_asked(&t, from, yes_no));
        t.answer_yes(P0, fight);
        t.resolve_all();
        assert!(was_asked(&t, from, yes_no));
        assert_eq!(t.on_battlefield(theirs), !fight);
    }
}

#[test]
fn boxing_ring_needs_a_creature_that_fought_and_is_still_there() {
    cr!("602.5b", "701.14a");
    ruling!(
        "Boxing Ring",
        "If the only creature you controlled that fought this turn died due to that fight, you can't activate Boxing Ring's last ability. Only winners get prizes!"
    );
    supported("Boxing Ring");
    for (name, survives) in [("Grizzly Bears", false), ("Watchwolf", true)] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P0, "Boxing Ring");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(theirs)]);
        let mine = t.enter(P0, name);
        t.resolve_all();
        assert!(!t.on_battlefield(theirs));
        assert_eq!(t.on_battlefield(mine), survives);
        assert_eq!(can_activate(&mut t, P0, ring), survives, "{name}");
    }
}

#[test]
fn barroom_brawl_copy_may_target_a_creature_that_will_die() {
    cr!("707.10c", "608.2b", "704.3", "115.7");
    ruling!(
        "Barroom Brawl",
        "The opponent to your left may choose the creature they control that was part of the original fight as the first target for their copy, even if it would die after the original spell is done resolving. If it does die, however, it will be an illegal target as the copy tries to resolve."
    );
    supported("Barroom Brawl");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P1 copies it, targeting its own Bears (lethally damaged, not yet dead) and P0's Giant.
    t.answer_yes(P1, true);
    change_copy_targets(&mut t, P1, &[Some(obj(theirs)), Some(obj(giant))]);
    cast_new(&mut t, P0, "Barroom Brawl", &[obj(mine), obj(theirs)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "P1's copy: {}", t.dump_log());
    // The copy has the targets P1 chose: its own Bears, lethally damaged but still on the
    // battlefield as it was chosen, and P0's Giant.
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(
        crate::r_s25_common::targets_of(&t, copy),
        vec![obj(theirs), obj(giant)]
    );
    // State-based actions put both Bears into graveyards before the copy resolves.
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    t.resolve_all();
    // The first target is illegal: no fight, the Giant is dealt no damage.
    assert_eq!(damage_on(&t, giant), 0);
}

fn change_copy_targets(t: &mut TestGame, p: PlayerId, new: &[Option<Entity>]) {
    crate::r_s25_common::change_copy_targets(t, p, new)
}
