//! Rulings batch P071 — cards that give deathtouch (CR 702.2): lord effects and their
//! sources leaving (CR 611.3a, 704.5g, 704.5h), spells that affect only the creatures there
//! as they resolve (CR 611.2c), targets that become illegal (CR 608.2b), granted triggers,
//! and Quietus Spike's half-life loss (CR 107.1a).

use crate::r_p071_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::counters;
use mtg_engine::*;

fn deathtouch(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Deathtouch)
}

/// P0 casts `name` (on the stack) for its mana (any type), targeting `targets`.
fn cast(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    let c = t.hand(P0, name);
    mana(t, P0, ManaType::B, 4);
    mana(t, P0, ManaType::G, 4);
    mana(t, P0, ManaType::R, 4);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let s = t.cast(P0, c).go();
    t.g.players[0].mana_pool.empty();
    s
}

/// Declares `blocks` for `dp` and runs to the declare blockers step, P0 (the active
/// player) with priority.
fn block_with(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(dp, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    let ap = t.g.turn.active;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap)
    });
    assert!(ok, "blockers not declared");
    t.settle();
}

/// Sets loyalty counters on a planeswalker.
fn loyalty(t: &mut TestGame, pw: ObjectId, n: u32) {
    t.g.objects[pw.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
}

// ---------------------------------------------------------------------------
// Lords: Wren's Run Packmaster, Death Baron, Ogre Slumlord
// ---------------------------------------------------------------------------

#[test]
fn wrens_run_packmaster_gives_all_your_wolves_deathtouch() {
    cr!("611.3a", "109.4");
    ruling!(
        "Wren's Run Packmaster",
        "Wren's Run Packmaster gives deathtouch to all Wolf permanents you control, not just the tokens it creates."
    );
    ruling!(
        "Ogre Slumlord",
        "All Rats you control have deathtouch while Ogre Slumlord is on the battlefield, not just the ones created by Ogre Slumlord."
    );
    for (lord, kind) in [
        ("Wren's Run Packmaster", "Watchwolf"),
        ("Ogre Slumlord", "Pack Rat"),
    ] {
        let mut t = TestGame::new(2);
        let lord = t.battlefield(P0, lord);
        let mine = t.battlefield(P0, kind);
        let theirs = t.battlefield(P1, kind);
        assert!(deathtouch(&t, mine), "{kind}");
        assert!(!deathtouch(&t, theirs), "{kind}");
        t.g.destroy(lord, None);
        t.settle();
        assert!(!deathtouch(&t, mine), "{kind}");
    }
}

#[test]
fn wrens_run_packmaster_wolf_damage_destroys_even_if_packmaster_leaves_at_the_same_time() {
    cr!("702.2b", "704.5h", "704.3");
    ruling!(
        "Wren's Run Packmaster",
        "A Wolf's deathtouch ability will apply if both Wren's Run Packmaster and that Wolf are on the battlefield at the time the Wolf deals damage. The creature dealt damage by the Wolf will be destroyed the next time state-based actions are checked, even if the Wolf or Wren's Run Packmaster leaves the battlefield."
    );
    // The Wolf (Watchwolf, 3/3) is blocked by Craw Wurm (6/4); Packmaster (5/5) by
    // another Craw Wurm. Combat damage kills the Wolf and the Packmaster; the Wurm dealt 3
    // (nonlethal) damage by the Wolf is destroyed too.
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Wren's Run Packmaster");
    let wolf = t.battlefield(P0, "Watchwolf");
    let w1 = t.battlefield(P1, "Craw Wurm");
    let w2 = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(pm, Entity::Player(P1)), (wolf, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(w1, wolf), (w2, pm)]);
    assert!(!t.on_battlefield(pm), "{}", t.dump_log());
    assert!(!t.on_battlefield(wolf));
    assert!(!t.on_battlefield(w1), "deathtouch damage: {}", t.dump_log());
    assert!(!t.on_battlefield(w2), "lethal damage");
}

#[test]
fn death_baron_bonus_applies_once_and_to_itself_only_as_a_skeleton() {
    cr!("613.4c", "702.2f");
    ruling!(
        "Death Baron",
        "A creature that's both a Skeleton and a Zombie will get the bonus only once."
    );
    ruling!(
        "Death Baron",
        "Death Baron doesn't normally affect itself. If you manage to turn it into a Skeleton, however, then it will give itself +1/+1 and deathtouch."
    );
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Death Baron");
    let co = t.battlefield(P0, "Changeling Outcast");
    assert_eq!(t.pt(co), (2, 2));
    assert!(deathtouch(&t, co));
    assert_eq!(t.pt(baron), (2, 2));
    assert!(!deathtouch(&t, baron));
    // Maskwood Nexus: every creature type, Skeleton included.
    t.battlefield(P0, "Maskwood Nexus");
    t.g.recompute();
    assert_eq!(t.pt(baron), (3, 3));
    assert!(deathtouch(&t, baron));
    assert_eq!(t.pt(co), (2, 2));
}

#[test]
fn death_baron_leaving_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Death Baron",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Skeleton or Zombie you control may become lethal if Death Baron leaves the battlefield during that turn."
    );
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Death Baron");
    let wc = t.battlefield(P0, "Walking Corpse");
    assert_eq!(t.pt(wc), (3, 3));
    let shock_src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(shock_src, Entity::Object(wc), 2, false);
    t.settle();
    assert!(t.on_battlefield(wc));
    t.g.destroy(baron, None);
    t.settle();
    assert!(!t.on_battlefield(wc), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Totentanz, Tyvar the Bellicose
// ---------------------------------------------------------------------------

#[test]
fn totentanz_dying_with_other_creatures_triggers_for_each() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Totentanz, Swarm Piper",
        "If Totentanz, Swarm Piper dies at the same time as one or more other nontoken creatures you control, Totentanz's ability triggers for each of them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Totentanz, Swarm Piper");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let wog = t.hand(P0, "Wrath of God");
    mana(&mut t, P0, ManaType::W, 4);
    t.cast(P0, wog).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Rat").len(), 3, "{}", t.dump_log());
}

#[test]
fn tyvar_the_bellicose_new_tyvar_grants_a_new_ability_that_can_trigger_again() {
    cr!("400.7", "603.4", "704.5j");
    ruling!(
        "Tyvar the Bellicose",
        "If Tyvar leaves the battlefield and returns to the battlefield in the same turn, or if Tyvar leaves and another Tyvar appears, the triggered ability granted by the new one is different than the triggered ability granted by the old one. The one granted by the new Tyvar may trigger for each creature you control, even if the ability granted by the old Tyvar triggered for those creatures that turn. (Note that creatures you control won't have the ability granted by the old one anymore, and if you control multiple Tyvars, you won't have the opportunity to activate mana abilities before the \"legend rule\" gets you down to one Tyvar.)"
    );
    let mut t = TestGame::new(2);
    let tyvar = t.battlefield(P0, "Tyvar the Bellicose");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    // A second Tyvar: the legend rule leaves one (the new one); the Elf has one granted
    // ability, which hasn't triggered this turn.
    let new = t.enter(P0, "Tyvar the Bellicose");
    t.answer_choose(P0, &[o(new)]);
    t.settle();
    assert_eq!(t.named_on_battlefield("Tyvar the Bellicose").len(), 1);
    assert!(!t.on_battlefield(tyvar), "{}", t.dump_log());
    let elf = t.g.current(elf);
    let granted = t
        .g
        .obj(elf)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains("mana ability of this creature resolves"))
        .count();
    assert_eq!(granted, 1);
    t.g.objects[elf.0 as usize].tapped = false;
    t.activate(P0, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2, "{}", t.dump_log());
    // Once more this turn: no.
    t.g.objects[elf.0 as usize].tapped = false;
    t.activate(P0, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2);
}

#[test]
fn tyvar_the_bellicose_trigger_from_paying_for_a_spell_resolves_first() {
    cr!("605.3b", "603.3", "601.2g");
    ruling!(
        "Tyvar the Bellicose",
        "If the mana ability that caused the granted triggered ability to trigger was activated while casting a spell or activating an ability, the triggered ability will resolve (and +1/+1 counters will be added) before that spell or ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tyvar the Bellicose");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(elf).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    assert!(t.obj_now(elf).tapped);
    t.resolve();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(elf), (2, 2));
    t.resolve();
    assert_eq!(t.pt(elf), (5, 5));
}

// ---------------------------------------------------------------------------
// Spells: Undercity Uprising, Bladebrand, Pain 101, Deadly Allure, Ruthless Instincts,
// Touch of Moonglove
// ---------------------------------------------------------------------------

/// P0 casts Undercity Uprising: its targets are P0's `mine` and P1's `theirs`.
fn uprising(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) -> ObjectId {
    cast(t, "Undercity Uprising", &[o(mine), o(theirs)])
}

#[test]
fn undercity_uprising_with_both_targets_illegal_does_nothing() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Undercity Uprising",
        "If both target creatures are illegal targets by the time Undercity Uprising tries to resolve, the spell doesn’t resolve. No creatures deal damage or gain deathtouch."
    );
    let mut t = TestGame::new(2);
    let other = t.battlefield(P0, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    uprising(&mut t, mine, theirs);
    t.g.destroy(mine, None);
    t.g.destroy(theirs, None);
    t.settle();
    t.resolve_all();
    assert!(!deathtouch(&t, other));
    assert!(t.in_graveyard(P0, "Undercity Uprising"));
}

#[test]
fn undercity_uprising_with_one_target_illegal_gives_deathtouch_without_a_fight() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Undercity Uprising",
        "If one of the two target creatures is an illegal target by the time Undercity Uprising tries to resolve, no creature deals or is dealt damage. Creatures you control still gain deathtouch."
    );
    // P1's creature gains hexproof (an illegal target): P0's Hill Giant gets deathtouch
    // and deals no damage.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    uprising(&mut t, mine, theirs);
    t.g.destroy(theirs, None);
    t.settle();
    let theirs2 = t.battlefield(P1, "Craw Wurm");
    t.resolve_all();
    assert!(deathtouch(&t, mine));
    assert_eq!(t.obj_now(mine).damage, 0);
    assert_eq!(t.obj_now(theirs2).damage, 0);
    // P0's creature leaves instead: the other gets no damage either.
    let mut t = TestGame::new(2);
    let other = t.battlefield(P0, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    uprising(&mut t, mine, theirs);
    t.g.destroy(mine, None);
    t.settle();
    t.resolve_all();
    assert!(deathtouch(&t, other));
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.obj_now(theirs).damage, 0);
}

#[test]
fn undercity_uprising_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Undercity Uprising",
        "The first part of Undercity Uprising’s effect affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain deathtouch."
    );
    ruling!(
        "Vraska, Scheming Gorgon",
        "Vraska’s first and last abilities affect only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +1/+0 or gain abilities."
    );
    ruling!(
        "Ajani, Inspiring Leader",
        "Ajani’s last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain flying or double strike."
    );
    ruling!(
        "Cleaver Riot",
        "Cleaver Riot affects only creatures you control at the time it resolves. It won’t affect creatures that come under your control later in the turn."
    );
    // Undercity Uprising.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    uprising(&mut t, mine, theirs);
    t.resolve_all();
    assert!(deathtouch(&t, mine));
    let later = t.battlefield(P0, "Grizzly Bears");
    assert!(!deathtouch(&t, later));
    // Vraska's +2 and −10, Ajani's −10, Cleaver Riot.
    for which in ["Vraska +2", "Vraska -10", "Ajani", "Cleaver Riot"] {
        let mut t = TestGame::new(2);
        let before = t.battlefield(P0, "Grizzly Bears");
        match which {
            "Vraska +2" | "Vraska -10" => {
                let v = t.battlefield(P0, "Vraska, Scheming Gorgon");
                loyalty(&mut t, v, 10);
                let i = if which == "Vraska +2" { 0 } else { 2 };
                t.activate(P0, v, i, &[]).unwrap();
            }
            "Ajani" => {
                let a = t.battlefield(P0, "Ajani, Inspiring Leader");
                loyalty(&mut t, a, 10);
                t.activate(P0, a, 2, &[]).unwrap();
            }
            _ => {
                cast(&mut t, "Cleaver Riot", &[]);
            }
        }
        t.resolve_all();
        let later = t.battlefield(P0, "Grizzly Bears");
        let check = |t: &TestGame, id: ObjectId| match which {
            "Vraska +2" => t.pt(id) == (3, 2),
            "Vraska -10" => deathtouch(t, id),
            "Ajani" => {
                t.obj_now(id).has_keyword(KeywordKind::Flying)
                    && t.obj_now(id).has_keyword(KeywordKind::DoubleStrike)
            }
            _ => t.obj_now(id).has_keyword(KeywordKind::DoubleStrike),
        };
        assert!(check(&t, before), "{which}: {}", t.dump_log());
        assert!(!check(&t, later), "{which}");
    }
}

#[test]
fn bladebrand_with_an_illegal_target_draws_no_card() {
    cr!("608.2b");
    ruling!(
        "Bladebrand",
        "If the target creature is an illegal target by the time Bladebrand tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast(&mut t, "Bladebrand", &[o(bears)]);
    let hand = t.hand_size(P0);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Bladebrand"));
}

#[test]
fn pain_101_token_triggers_but_isnt_returned() {
    cr!("111.7", "704.5d", "603.6c");
    ruling!(
        "Pain 101",
        "If the target creature is a token, the granted ability will still trigger when it dies, but you won't return the token to the battlefield."
    );
    let mut t = TestGame::new(2);
    let raise = t.hand(P0, "Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, raise).go();
    t.resolve_all();
    let token = with_subtype(&t, P0, "Soldier")[0];
    cast(&mut t, "Pain 101", &[o(token)]);
    t.resolve_all();
    assert!(deathtouch(&t, token));
    t.g.destroy(token, None);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the granted ability triggers: {}", t.dump_log());
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    // A nontoken creature comes back tapped.
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast(&mut t, "Pain 101", &[o(bears)]);
    t.resolve_all();
    t.g.destroy(bears, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    assert!(t.g.obj(back[0]).tapped);
}

#[test]
fn deadly_allure_attacker_must_be_blocked_by_at_least_one_able_creature() {
    cr!("509.1c", "702.2b");
    ruling!(
        "Deadly Allure",
        "If the target creature is attacking, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    cast(&mut t, "Deadly Allure", &[o(bears)]);
    t.resolve_all();
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    // P1 tries not to block (then to block the Giant): the Wall must block Grizzly Bears.
    block_with(&mut t, P1, &[(wall, giant)]);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blockers_of(bears), vec![wall], "{}", t.dump_log());
}

#[test]
fn ruthless_instincts_first_mode_needs_a_nonattacking_creature_that_can_attack_later() {
    cr!("115.1", "508.1a", "702.17b");
    ruling!(
        "Ruthless Instincts",
        "If you choose the first mode, the target creature must not be attacking. However, it can attack later in the turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    let ri = t.hand(P0, "Ruthless Instincts");
    mana(&mut t, P0, ManaType::G, 3);
    t.cast(P0, ri).modes(&[0]).target(bears).go();
    t.resolve_all();
    assert!(deathtouch(&t, bears));
    assert!(!t.obj_now(bears).tapped);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.is_attacking(bears), "{}", t.dump_log());
    // Now attacking: not a legal target for the first mode (Hill Giant, not attacking,
    // is).
    let giant = t.battlefield(P0, "Hill Giant");
    let ri = t.hand(P0, "Ruthless Instincts");
    mana(&mut t, P0, ManaType::G, 3);
    let from = t.asked().len();
    t.cast(P0, ri).modes(&[0]).target(giant).go();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&o(giant)));
    assert!(!offered.contains(&o(bears)), "{offered:?}");
}

#[test]
fn touch_of_moonglove_life_loss_is_by_the_dead_creatures_controller() {
    cr!("603.10a", "702.2b");
    ruling!(
        "Touch of Moonglove",
        "The controller of the creature dealt damage by the target creature is the one who loses 2 life, not the controller of the target creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    cast(&mut t, "Touch of Moonglove", &[o(bears)]);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn onyx_mage_twice_still_destroys_once() {
    cr!("702.2d", "704.5h", "701.19a");
    ruling!(
        "Onyx Mage",
        "Multiple instances of deathtouch on the same creature provide no additional benefit. A creature dealt damage by a creature with multiple instances of deathtouch is destroyed only once."
    );
    // Cudgel Troll (4/3) with one regeneration shield is dealt 2 damage by a Grizzly
    // Bears with two instances of deathtouch: one shield is enough.
    let mut t = TestGame::new(2);
    let om = t.battlefield(P0, "Onyx Mage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        mana(&mut t, P0, ManaType::B, 2);
        t.activate(P0, om, 0, &[o(bears)]).unwrap();
        t.resolve_all();
    }
    let n = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.to_lowercase().contains("deathtouch"))
        .count();
    assert!(n >= 2, "two instances: {n}");
    let troll = t.battlefield(P1, "Cudgel Troll");
    mana(&mut t, P1, ManaType::G, 1);
    t.g.turn.priority = Some(P1);
    t.activate(P1, troll, 0, &[]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(troll, bears)]);
    assert!(t.on_battlefield(troll), "{}", t.dump_log());
    assert!(t.obj_now(troll).tapped, "regenerated");
}

// ---------------------------------------------------------------------------
// Hunted Nightmare, Vedalken Infuser, Ghirapur Guide, Bow of Nylea
// ---------------------------------------------------------------------------

#[test]
fn hunted_nightmare_target_opponent_chooses_the_creature_if_any() {
    cr!("608.2d", "122.1");
    ruling!(
        "Hunted Nightmare",
        "The target opponent chooses which of their creatures receives a deathtouch counter."
    );
    ruling!(
        "Hunted Nightmare",
        "If the target opponent controls no creatures, they won’t put a deathtouch counter on any creature."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[o(b)]);
    let from = t.asked().len();
    let hn = t.enter(P0, "Hunted Nightmare");
    t.resolve_all();
    assert_eq!(t.counters(b, "deathtouch"), 1, "{}", t.dump_log());
    assert_eq!(t.counters(a, "deathtouch"), 0);
    assert!(deathtouch(&t, b));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.counters(hn, "deathtouch"), 0);
    // No creatures: no counter anywhere (not on P0's creatures either).
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hn = t.enter(P0, "Hunted Nightmare");
    t.resolve_all();
    assert_eq!(t.counters(mine, "deathtouch"), 0);
    assert_eq!(t.counters(hn, "deathtouch"), 0);
}

#[test]
fn vedalken_infuser_puts_a_charge_counter_on_any_artifact() {
    cr!("122.1", "115.1");
    ruling!(
        "Vedalken Infuser",
        "You may target and put a charge counter on any artifact, even if it doesn't have an ability that references charge counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Infuser");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[o(thopter)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(thopter, "charge"), 1, "{}", t.dump_log());
}

#[test]
fn ghirapur_guide_doesnt_undo_a_block() {
    cr!("509.1b", "506.4");
    ruling!(
        "Ghirapur Guide",
        "Once a creature with power 2 or less has blocked a creature, activating Ghirapur Guide's ability won't change or undo that block."
    );
    ruling!(
        "Ghirapur Guide",
        "Once a creature with power 3 or greater has blocked the target creature, changing the power of the blocking creature won't change or undo that block."
    );
    // Grizzly Bears blocks Hill Giant, then the Guide's ability targets the Giant.
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P0, "Ghirapur Guide");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_with(&mut t, P1, &[(bears, giant)]);
    mana(&mut t, P0, ManaType::G, 3);
    t.activate(P0, guide, 0, &[o(giant)]).unwrap();
    t.resolve_all();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(giant));
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
    // The Giant is targeted first; P1's Craw Wurm (6 power) blocks; it gets −5/−0.
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P0, "Ghirapur Guide");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    mana(&mut t, P0, ManaType::G, 3);
    t.activate(P0, guide, 0, &[o(giant)]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_with(&mut t, P1, &[(wurm, giant)]);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(giant));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        wurm,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::Const(-5),
            mtg_engine::ability::Value::Const(0),
        )],
    );
    assert_eq!(t.pt(wurm).0, 1);
    assert_eq!(t.g.combat.as_ref().unwrap().blockers_of(giant), vec![wurm]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
}

#[test]
fn bow_of_nylea_mode_is_chosen_on_activation() {
    cr!("602.2b", "601.2b", "700.2a");
    ruling!(
        "Bow of Nylea",
        "You choose which mode you're using as you activate the ability."
    );
    let mut t = TestGame::new(2);
    let bow = t.battlefield(P0, "Bow of Nylea");
    mana(&mut t, P0, ManaType::G, 2);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.activate(P0, bow, 0, &[]).unwrap();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. })));
    let ab = *t.g.stack.last().unwrap();
    let chosen: Vec<_> = t.g.obj(ab).stack.as_ref().unwrap().chosen.iter().map(|c| c.mode).collect();
    assert_eq!(chosen, vec![Some(2)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert_eq!(t.life(P0), 23);
}

// ---------------------------------------------------------------------------
// Quietus Spike
// ---------------------------------------------------------------------------

/// P0's Grizzly Bears, equipped with `n` Quietus Spikes, attacks `defender` unblocked.
fn spiked(t: &mut TestGame, n: usize, defender: PlayerId) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..n {
        let s = t.battlefield(P0, "Quietus Spike");
        t.g.attach(s, Entity::Object(bears));
    }
    attack_with(t, &[(bears, Entity::Player(defender))]);
    bears
}

#[test]
fn quietus_spike_loss_is_half_the_life_after_combat_damage_as_it_resolves() {
    cr!("107.1a", "603.3", "608.2h");
    ruling!(
        "Quietus Spike",
        "That player loses half their life after combat damage has been subtracted from the player's life total. The amount of life the player loses is determined as the triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    spiked(&mut t, 1, P1);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 1);
    // P1 gains 5 life in response: 23, loses 12.
    t.g.gain_life(P1, 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 11, "{}", t.dump_log());
}

#[test]
fn quietus_spikes_triggering_together_halve_the_remainder_each_time() {
    cr!("107.1a", "603.3b");
    ruling!(
        "Quietus Spike",
        "If multiple Quietus Spikes trigger at the same time, that player loses half their life when the first ability resolves, then loses half of the remainder when the next ability resolves, and so on. The player does not lose the same amount each time."
    );
    let mut t = TestGame::new(2);
    spiked(&mut t, 2, P1);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // 20 − 2 = 18; −9 = 9; −5 = 4.
    assert_eq!(t.life(P1), 4, "{}", t.dump_log());
}

#[test]
fn quietus_spike_in_two_headed_giant_halves_the_team_life() {
    cr!("810.9", "810.9a", "107.1a");
    ruling!(
        "Quietus Spike",
        "In a Two-Headed Giant game, after combat damage is dealt, Quietus Spike looks at that player's life total (which is the same as the team's life total) when determining how much life the team will lose, which basically means the team's life total is halved. Here's an example of the math: The team has 19 life, so the player has 19 life. Quietus Spike causes the team to lose 10 life (19 divided by 2, rounded up). The team's life total becomes 9 (19 minus 10)."
    );
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    // The team P2/P3 has 21 life; the Bears deal 2 to P2: 19; then −10: 9.
    let have = t.life(P2);
    t.g.lose_life(P2, (have - 21) as u32);
    t.settle();
    assert_eq!((t.life(P2), t.life(P3)), (21, 21));
    spiked(&mut t, 1, P2);
    block_and_finish(&mut t, P2, &[]);
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (9, 9), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Tyvar Kell, Kaya, Vraska
// ---------------------------------------------------------------------------

#[test]
fn tyvar_kell_emblem_haste_carries_from_the_elf_spell_to_the_permanent() {
    cr!("611.2c", "400.7a", "302.6");
    ruling!(
        "Tyvar Kell",
        "The emblem will cause the Elf spell to gain haste. That change carries through to the permanent that the spell becomes."
    );
    let mut t = TestGame::new(2);
    let tk = t.battlefield(P0, "Tyvar Kell");
    loyalty(&mut t, tk, 6);
    t.activate(P0, tk, 2, &[]).unwrap();
    t.resolve_all();
    let elf = t.hand(P0, "Llanowar Elves");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, elf).go();
    t.resolve_all();
    assert!(t.on_battlefield(elf));
    assert!(t.obj_now(elf).summoning_sick);
    assert!(t.obj_now(elf).has_keyword(KeywordKind::Haste), "{}", t.dump_log());
    // It can tap for mana right away.
    t.activate(P0, t.g.current(elf), 0, &[]).unwrap();
    assert!(t.obj_now(elf).tapped);
}

#[test]
fn tyvar_kell_targets_elves_tapped_or_not_but_not_himself() {
    cr!("115.1", "205.3j", "306.1");
    ruling!(
        "Tyvar Kell",
        "The first loyalty ability can target an Elf that's already untapped."
    );
    ruling!(
        "Tyvar Kell",
        "Tyvar may look like an Elf, but he's not a creature with the creature type Elf, so he doesn't give himself the mana ability he gives Elves you control and he can't be the target of the first loyalty ability."
    );
    let mut t = TestGame::new(2);
    let tk = t.battlefield(P0, "Tyvar Kell");
    let elf = t.battlefield(P0, "Llanowar Elves");
    // Tyvar has no mana ability; the Elf has its own and Tyvar's.
    assert!(t.obj_now(tk).chars.abilities.iter().all(|a| !a.is_mana_ability()));
    let mana_abilities = t
        .obj_now(elf)
        .chars
        .abilities
        .iter()
        .filter(|a| a.is_mana_ability())
        .count();
    assert_eq!(mana_abilities, 2);
    let from = t.asked().len();
    t.answer_targets(P0, &[o(elf)]);
    t.activate(P0, tk, 0, &[]).unwrap();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&o(elf)));
    assert!(!offered.contains(&o(tk)));
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    assert!(deathtouch(&t, elf));
    assert!(!t.obj_now(elf).tapped);
}

#[test]
fn kaya_doubled_tokens_are_exact_copies_of_the_created_ones() {
    cr!("111.1", "614.1a");
    ruling!(
        "Kaya, Geist Hunter",
        "The tokens that Kaya's second ability creates are exact copies of the tokens that were being created in the first place."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kaya, Geist Hunter");
    t.activate(P0, k, 1, &[]).unwrap();
    t.resolve_all();
    let raise = t.hand(P0, "Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, raise).go();
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 4, "{}", t.dump_log());
    for s in soldiers {
        let o = t.g.obj(s);
        assert!(o.is_token());
        assert_eq!(o.chars.name, t.g.obj(with_subtype(&t, P0, "Soldier")[0]).chars.name);
        assert_eq!((o.power(), o.toughness()), (1, 1));
        assert!(o.chars.colors.contains(mtg_engine::types::Color::White));
    }
}

#[test]
fn vraska_granted_trigger_counts_any_damage_and_ends_the_game_when_opponents_are_gone() {
    cr!("104.2a", "603.3b", "120.3");
    ruling!(
        "Vraska, Scheming Gorgon",
        "The triggered ability Vraska grants to creatures with her third ability triggers on any damage those creatures deal, including noncombat damage. If multiple players are dealt damage simultaneously by affected creatures, you choose the order in which those triggers resolve. If all your opponents have lost the game, you’ll win the game before any more of the triggers resolve."
    );
    // Noncombat damage: Prodigal Pyromancer pings P1.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vraska, Scheming Gorgon");
    let pp = t.battlefield(P0, "Prodigal Pyromancer");
    loyalty(&mut t, v, 10);
    t.activate(P0, v, 2, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, pp, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P1), "{}", t.dump_log());
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    // Three players: two creatures deal combat damage to P1 and P2 at once; P0 orders the
    // triggers; P0 wins when both have lost.
    let mut t = TestGame::new(3);
    let v = t.battlefield(P0, "Vraska, Scheming Gorgon");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    loyalty(&mut t, v, 10);
    t.activate(P0, v, 2, &[]).unwrap();
    t.resolve_all();
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
            (c, Entity::Player(P2)),
        ],
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.answer(P2, DecisionKind::Blockers, Answer::Blockers(vec![]));
    // The Bears' trigger (P1) on top, then the Wurm's (P2), then the Giant's (P2).
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 2, 0]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    t.resolve();
    assert!(t.has_lost(P1) && !t.has_lost(P2), "{}", t.dump_log());
    assert!(!t.g.is_over());
    t.resolve();
    assert!(t.has_lost(P2), "{}", t.dump_log());
    // P0 has won with the Giant's trigger still unresolved.
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    assert!(!t
        .dump_log()
        .contains("Resolving Hill Giant ability"));
}

// ---------------------------------------------------------------------------
// Double strike: Paladin Class, Armed // Dangerous, Embercleave, Blade Historian
// ---------------------------------------------------------------------------

#[test]
fn paladin_class_cost_increases_are_cumulative() {
    cr!("601.2f", "613.1");
    ruling!(
        "Paladin Class",
        "Additional costs from Paladin Class cards are cumulative. For example, if you control two Paladin Class cards, spells your opponents cast cost {2} more to cast."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Paladin Class");
    t.battlefield(P0, "Paladin Class");
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, ManaType::R, 2);
    assert!(!castable(&mut t, P1, bolt));
    mana(&mut t, P1, ManaType::R, 1);
    assert!(castable(&mut t, P1, bolt));
    // Not in P1's own turn.
    t.set_step(P1, Step::PrecombatMain);
    crate::r_p069_common::no_mana(&mut t, P1);
    mana(&mut t, P1, ManaType::R, 1);
    assert!(castable(&mut t, P1, bolt));
}

#[test]
fn dangerous_forces_only_creatures_able_to_block() {
    cr!("509.1c", "702.9b");
    ruling!(
        "Armed // Dangerous",
        "Dangerous doesn’t give any creatures the ability to block the target creature. It just forces those creatures that are already able to block the creature to do so."
    );
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    let d = t.hand(P0, "Armed // Dangerous");
    mana(&mut t, P0, ManaType::G, 4);
    t.cast(P0, d)
        .method(mtg_engine::object::CastMethod::Half(1))
        .target(drake)
        .go();
    t.resolve_all();
    attack_with(&mut t, &[(drake, Entity::Player(P1))]);
    block_with(&mut t, P1, &[]);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blockers_of(drake), vec![spider], "{}", t.dump_log());
    assert!(c.blocking(bears).is_empty());
}

#[test]
fn embercleave_cant_cost_less_than_rr() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Embercleave",
        "Embercleave's second ability can't reduce its cost below {R}{R}."
    );
    let mut t = TestGame::new(2);
    let mut atk = vec![];
    for _ in 0..7 {
        let c = t.battlefield(P0, "Grizzly Bears");
        atk.push((c, Entity::Player(P1)));
    }
    attack_with(&mut t, &atk);
    let e = t.hand(P0, "Embercleave");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 3);
    assert!(!castable(&mut t, P0, e));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(castable(&mut t, P0, e));
    crate::r_p069_common::no_mana(&mut t, P0);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_targets(P0, &[o(atk[0].0)]);
    t.cast(P0, e).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
}

#[test]
fn blade_historian_leaving_after_first_strike_damage_stops_regular_damage() {
    cr!("702.4c", "702.4d", "510.4");
    ruling!(
        "Blade Historian",
        "If Blade Historian leaves the battlefield after first-strike combat damage has been dealt but before regular combat damage (perhaps because it attacked and was destroyed by first-strike combat damage), attacking creatures you control will lose double strike. A creature without double strike won’t deal regular combat damage if it already dealt first-strike damage that turn."
    );
    let mut t = TestGame::new(2);
    let bh = t.battlefield(P0, "Blade Historian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(bh, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(t.life(P1), 16);
    t.g.destroy(bh, None);
    t.settle();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
}
