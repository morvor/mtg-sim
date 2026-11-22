//! Rulings batch P215 — fight (CR 701.14): illegal targets, choosing whether to fight as a
//! "may" ability resolves, reflexive triggers, and bonuses checked on resolution.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Gives `id` (followed across zone changes) the keyword until end of turn, as an effect of
/// `by`'s.
pub(crate) fn grant(t: &mut TestGame, by: PlayerId, id: ObjectId, kw: Keyword) {
    run_from(
        t,
        by,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(t.g.current(id))],
    );
}

/// Makes `id` an illegal target for P0's spells and abilities: P1's creatures get hexproof
/// (from P1), P0's get shroud.
pub(crate) fn make_illegal(t: &mut TestGame, id: ObjectId) {
    let c = t.obj_now(id).controller;
    if c == P0 {
        grant(t, P0, id, Keyword::new(KeywordKind::Shroud));
    } else {
        grant(t, c, id, Keyword::new(KeywordKind::Hexproof));
    }
}

fn no_damage(t: &TestGame, ids: &[ObjectId]) {
    for id in ids {
        assert_eq!(damage_on(t, t.g.current(*id)), 0);
    }
}

/// P0 casts `name` targeting P0's `mine` and P1's `theirs` (in that order), makes the
/// targets in `illegal` illegal, and resolves everything. Returns (game, mine, theirs).
fn two_target_spell(
    name: &str,
    mine: &str,
    theirs: &str,
    illegal: &[usize],
) -> (TestGame, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, mine);
    let o = t.battlefield(P1, theirs);
    cast_new(&mut t, P0, name, &[Entity::Object(m), Entity::Object(o)]);
    for i in illegal {
        make_illegal(&mut t, [m, o][*i]);
    }
    t.resolve_all();
    (t, m, o)
}

#[test]
fn mage_duel_with_an_illegal_target() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Mage Duel",
        "If the creature you control is an illegal target as Mage Duel tries to resolve, that creature won’t get +1/+2. If that creature is a legal target, but the one you don’t control isn’t, your creature will still get +1/+2. In both cases, no fight will happen and no creature will deal or be dealt damage."
    );
    let (t, m, o) = two_target_spell("Mage Duel", "Grizzly Bears", "Grizzly Bears", &[0]);
    assert_eq!(t.pt(m), (2, 2));
    no_damage(&t, &[m, o]);
    let (t, m, o) = two_target_spell("Mage Duel", "Grizzly Bears", "Grizzly Bears", &[1]);
    assert_eq!(t.pt(m), (3, 4));
    no_damage(&t, &[m, o]);
}

#[test]
fn hunt_the_hunter_with_an_illegal_opposing_creature_pumps_but_doesnt_fight() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Hunt the Hunter",
        "If the green creature an opponent controls is an illegal target when Hunt the Hunter tries to resolve, but the green creature you control is still a legal target, the creature you control will get +2/+2, but the creatures won't fight. Neither creature will deal or be dealt damage."
    );
    let (t, m, o) = two_target_spell("Hunt the Hunter", "Grizzly Bears", "Grizzly Bears", &[1]);
    assert_eq!(t.pt(m), (4, 4));
    no_damage(&t, &[m, o]);
}

#[test]
fn time_to_feed_with_one_illegal_target_doesnt_fight_or_gain_life() {
    cr!("608.2b", "701.14b", "603.7c");
    ruling!(
        "Time to Feed",
        "Time to Feed has two targets: a creature an opponent controls and a creature you control. If only one of those creatures is a legal target when Time to Feed tries to resolve, the creatures won't fight and neither will deal or be dealt damage. If the creature you don't control is the illegal target, you won't gain life when it dies."
    );
    supported("Time to Feed");
    for illegal in [0usize, 1] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        // Targets: the creature an opponent controls first, then the one you control.
        cast_new(
            &mut t,
            P0,
            "Time to Feed",
            &[Entity::Object(theirs), Entity::Object(mine)],
        );
        make_illegal(&mut t, [theirs, mine][illegal]);
        t.resolve_all();
        no_damage(&t, &[mine, theirs]);
        if illegal == 0 {
            // The Bears dies later this turn: no life gained.
            destroy(&mut t, theirs);
            t.resolve_all();
            assert!(!t.on_battlefield(theirs));
            assert_eq!(t.life(P0), 20);
        }
    }
}

#[test]
fn spells_whose_bonus_is_conditional_still_fight() {
    cr!("608.2c", "701.14a");
    ruling!(
        "Joust",
        "The target creature you control gets +2/+1 only if it's a Knight as Joust resolves. It'll still fight if it doesn't get +2/+1."
    );
    ruling!(
        "Savage Swipe",
        "The target creature you control gets +2/+2 only if its power is exactly 2 as Savage Swipe resolves. It’ll still fight if it doesn’t get +2/+2."
    );
    ruling!(
        "Ancient Animus",
        "The creature you control doesn't have to be legendary. It simply won't receive a +1/+1 counter before it fights if it isn't legendary."
    );
    ruling!(
        "Duel for Dominance",
        "Only the +1/+1 counter is contingent on having three or more creatures with different powers. The target creatures will still fight even if you don't have three or more creatures with different powers when Duel for Dominance resolves."
    );
    // (spell, my creature, its P/T after the spell): no bonus, and the creatures fought —
    // P1's Hill Giant (3/3) is dealt my creature's power, mine is dealt 3.
    for (spell, mine, pt) in [
        ("Joust", "Hill Giant", (3, 3)),
        ("Savage Swipe", "Hill Giant", (3, 3)),
        ("Ancient Animus", "Hill Giant", (3, 3)),
        ("Duel for Dominance", "Hill Giant", (3, 3)),
    ] {
        let (t, m, o) = two_target_spell(spell, mine, "Hill Giant", &[]);
        assert!(!t.on_battlefield(m), "{spell}");
        assert!(!t.on_battlefield(o), "{spell}");
        let lki = t.g.obj(m);
        assert_eq!((lki.power(), lki.toughness()), pt, "{spell}");
    }
    // With the condition met, the bonus applies first.
    let (t, m, o) = two_target_spell("Joust", "Benalish Knight", "Grizzly Bears", &[]);
    assert_eq!(t.pt(m), (4, 3));
    assert!(!t.on_battlefield(o));
    let (t, m, o) = two_target_spell("Savage Swipe", "Grizzly Bears", "Grizzly Bears", &[]);
    assert_eq!(t.pt(m), (4, 4));
    assert!(!t.on_battlefield(o));
}

#[test]
fn smell_fear_with_one_legal_target_still_proliferates() {
    cr!("608.2b", "701.14b", "701.34a");
    ruling!(
        "Smell Fear",
        "If you choose two creatures as targets with this spell and only one target is legal as the spell resolves, no fight will occur and no damage will be dealt, but you will still proliferate."
    );
    supported("Smell Fear");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(mine), counters::PLUS1, 1, None);
    cast_new(
        &mut t,
        P0,
        "Smell Fear",
        &[Entity::Object(mine), Entity::Object(theirs)],
    );
    make_illegal(&mut t, theirs);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 2);
    no_damage(&t, &[mine, theirs]);
}

#[test]
fn smell_fear_needs_a_creature_you_control_but_not_a_second_target() {
    cr!("601.2c", "115.1", "701.34a");
    ruling!(
        "Smell Fear",
        "You cannot cast Smell Fear without any targets just to proliferate. You must choose at least one creature you control as a target to cast this spell. If you don't choose a creature you don't control as a target, no fight will occur."
    );
    supported("Smell Fear");
    // No creature of P0's: can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Smell Fear");
    let sf = t.hand(P0, "Smell Fear");
    assert!(!castable(&mut t, P0, sf));
    // With only the creature P0 controls as a target: it proliferates, no fight.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(mine), counters::PLUS1, 1, None);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Smell Fear");
    let sf = t.hand(P0, "Smell Fear");
    t.cast(P0, sf)
        .targets(&[Entity::Object(mine)])
        .targets(&[])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 2);
    no_damage(&t, &[mine, theirs]);
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s21_common::castable(t, p, card)
}

#[test]
fn primal_might_with_x_0_just_fights() {
    cr!("107.3a", "701.14a");
    ruling!(
        "Primal Might",
        "You can cast Primal Might with 0 as the value of X to just have the two target creatures fight."
    );
    supported("Primal Might");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let pm = t.hand(P0, "Primal Might");
    t.cast(P0, pm).x(0).target(mine).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(damage_on(&t, mine), 2);
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn pit_fight_second_target_can_be_your_own_creature_but_not_the_same_one() {
    cr!("115.3", "701.14a");
    ruling!(
        "Pit Fight",
        "The second target of Pit Fight can be another creature you control, but it can’t be the same creature as the first target."
    );
    supported("Pit Fight");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let pf = t.hand(P0, "Pit Fight");
    let from = t.asked().len();
    t.cast(P0, pf).target(giant).target(bears).go();
    let cands = target_candidates(&t, P0, from);
    assert!(!cands[1].contains(&Entity::Object(giant)));
    assert!(cands[1].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(damage_on(&t, giant), 2);
}

#[test]
fn undercity_uprising_deathtouch_fight_destroys() {
    cr!("702.2b", "701.14a", "704.5h");
    ruling!(
        "Undercity Uprising",
        "Remember that a source with deathtouch has to deal damage, but not necessarily combat damage, to destroy another creature. As long as the fighting creature you control has power greater than 0 (and the damage isn’t prevented), the fighting creature you don’t control will be destroyed."
    );
    let (t, m, o) = two_target_spell("Undercity Uprising", "Llanowar Elves", "Craw Wurm", &[]);
    assert!(!t.on_battlefield(o));
    assert!(!t.on_battlefield(m));
}

#[test]
fn apex_altisaur_illegal_target_or_altisaur_gone() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Apex Altisaur",
        "If the target creature is an illegal target when Apex Altisaur's last ability tries to resolve, the ability doesn't resolve. If it's a legal target but Apex Altisaur is no longer on the battlefield when the ability resolves (perhaps because it has already died from picking too many fights) the target creature won't deal or be dealt damage."
    );
    supported("Apex Altisaur");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        let apex = enter(&mut t, P0, "Apex Altisaur");
        assert_eq!(t.stack_len(), 1);
        if leave {
            move_to(&mut t, apex, Zone::Hand(P0));
        } else {
            make_illegal(&mut t, bears);
        }
        t.resolve_all();
        assert!(t.on_battlefield(bears));
        no_damage(&t, &[bears]);
        if !leave {
            assert_eq!(damage_on(&t, apex), 0);
        }
    }
}

#[test]
fn mawloc_exiles_the_fought_creature_if_it_dies_later_even_after_mawloc_left() {
    cr!("614.1a", "701.14a");
    ruling!(
        "Mawloc",
        "The creature Mawloc fights will be exiled instead of going to the graveyard if it would die on the same turn for any reason, not just because of the damage from fighting Mawloc. This is true even if Mawloc has left the battlefield by that time."
    );
    supported("Mawloc");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let m = t.hand(P0, "Mawloc");
    t.cast(P0, m).x(0).go();
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.resolve_all();
    let mawloc = t.g.current(m);
    // Mawloc (2/2) died fighting the 6/4 Wurm.
    assert!(!t.on_battlefield(mawloc));
    assert!(t.on_battlefield(wurm));
    assert_eq!(damage_on(&t, wurm), 2);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    assert!(!t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn the_tarrasque_fights_before_blockers() {
    cr!("508.1m", "701.14a", "509.1a");
    ruling!(
        "The Tarrasque",
        "The fight happens in the declare attackers step before blockers are declared. Unless the creature it is fighting survives, that creature won't be able to block that combat."
    );
    supported("The Tarrasque");
    let mut t = TestGame::new(2);
    let tarrasque = t.battlefield(P0, "The Tarrasque");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    attack_with(&mut t, &[(tarrasque, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(damage_on(&t, tarrasque), 6);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 10);
}

#[test]
fn strax_controller_chooses_the_creature_of_the_random_player() {
    cr!("701.14a", "603.12", "115.1");
    ruling!(
        "Strax, Sontaran Nurse",
        "The player is chosen at random, but Strax's controller gets to choose which of their creatures it fights."
    );
    supported("Strax, Sontaran Nurse");
    let mut t = TestGame::new(2);
    let strax = t.battlefield(P0, "Strax, Sontaran Nurse");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Wastes", 2);
    let from = t.asked().len();
    crate::r_s06_common::activate_containing(&mut t, P0, strax, "random").expect("activate");
    t.resolve_all();
    // Whichever player was chosen, Strax's controller chose the target.
    let p0 = target_candidates(&t, P0, from);
    assert_eq!(p0.len(), 1);
    assert!(target_candidates(&t, P1, from).is_empty());
    let c = &p0[0];
    assert!(
        (c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(elves)))
            || (c.contains(&Entity::Object(elves)) && !c.contains(&Entity::Object(bears)))
    );
    assert!(!c.contains(&Entity::Object(strax)));
    assert!(!t.on_battlefield(bears) || !t.on_battlefield(elves));
}

/// The "you may have it fight" cards: the target is chosen as the trigger goes on the
/// stack; the choice to fight is made on resolution.
#[test]
fn may_fight_triggers_choose_the_target_first_and_whether_to_fight_on_resolution() {
    cr!("603.3d", "608.2d", "701.14a");
    ruling!(
        "Somberwald Stag",
        "You choose the target of the triggered ability as it goes on the stack, but you choose whether or not to fight as that ability resolves."
    );
    ruling!(
        "Affectionate Indrik",
        "You choose the target of the triggered ability as it goes on the stack, but you choose whether the creatures fight as that ability resolves."
    );
    ruling!(
        "Atzocan Archer",
        "You must choose another target creature (if able) as Atzocan Archer’s triggered ability is put on the stack. You choose as that ability resolves whether those creatures fight."
    );
    ruling!(
        "Foe-Razer Regent",
        "You choose the target of the first triggered ability as it goes on the stack, but you choose whether or not to fight as that ability resolves."
    );
    // (Foe-Razer Regent's last ability isn't supported yet; its enters trigger is.)
    for name in [
        "Somberwald Stag",
        "Affectionate Indrik",
        "Atzocan Archer",
        "Foe-Razer Regent",
    ] {
        if name != "Foe-Razer Regent" {
            supported(name);
        }
        for fight in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.answer_targets(P0, &[Entity::Object(bears)]);
            let me = enter(&mut t, P0, name);
            // The target was chosen as the trigger went on the stack, before any choice.
            assert_eq!(t.stack_len(), 1, "{name}");
            assert!(!t
                .asked()
                .iter()
                .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
            t.answer_yes(P0, fight);
            t.resolve_all();
            assert!(t
                .asked()
                .iter()
                .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
            // The Bears is dealt damage equal to the creature's power.
            let dealt = if t.on_battlefield(bears) {
                damage_on(&t, bears)
            } else {
                2
            };
            assert_eq!(dealt > 0, fight, "{name}");
            assert_eq!(damage_on(&t, me), if fight { 2 } else { 0 }, "{name}");
        }
    }
    // Atzocan Archer must target another creature if able: with only P0's own creature
    // around, it's the target.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let archer = enter(&mut t, P0, "Atzocan Archer");
    let c = target_candidates(&t, P0, from);
    assert_eq!(c.len(), 1);
    assert!(c[0].contains(&Entity::Object(mine)) && !c[0].contains(&Entity::Object(archer)));
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn kraul_harpooner_target_or_no_target_first_fight_choice_on_resolution() {
    cr!("603.3d", "608.2d", "115.1");
    ruling!(
        "Kraul Harpooner",
        "You choose the target of the triggered ability (or that it has no target) as it goes on the stack, but you choose whether the creatures fight as that ability resolves."
    );
    supported("Kraul Harpooner");
    for fight in [false, true] {
        let mut t = TestGame::new(2);
        let bird = t.battlefield(P1, "Storm Crow");
        t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bird)]);
        let kh = enter(&mut t, P0, "Kraul Harpooner");
        assert_eq!(t.stack_len(), 1);
        t.answer_yes(P0, fight);
        t.resolve_all();
        assert_eq!(t.pt(kh), (4, 2));
        assert_eq!(!t.on_battlefield(bird), fight);
    }
    // No target: it still gets +X/+0.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let kh = enter(&mut t, P0, "Kraul Harpooner");
    t.resolve_all();
    assert_eq!(t.pt(kh), (4, 2));
}

#[test]
fn ulrich_target_chosen_on_trigger_fight_chosen_on_resolution() {
    cr!("603.3d", "608.2d");
    ruling!(
        "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        "You choose the target of the Ulrich, Uncontested Alpha's first ability as it goes on the stack, but you choose whether or not to fight as that ability resolves."
    );
    supported("Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha");
    for fight in [false, true] {
        let mut t = TestGame::new(2);
        let u = t.battlefield(
            P0,
            "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        );
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        crate::r_s17_common::transform(&mut t, u);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        t.answer_yes(P0, fight);
        t.resolve_all();
        assert_eq!(!t.on_battlefield(bears), fight);
        assert_eq!(damage_on(&t, u), if fight { 2 } else { 0 });
    }
}

#[test]
fn kioras_emblem_exists_when_the_octopuses_enter() {
    cr!("114.1", "603.2");
    ruling!(
        "Kiora, Master of the Depths",
        "You will have the emblem by the time the Octopus creature tokens enter the battlefield—they’ll be looking to fight!"
    );
    supported("Kiora, Master of the Depths");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kiora, Master of the Depths");
    t.g.objects[k.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 8);
    crate::r_s06_common::activate_containing(&mut t, P0, k, "emblem").expect("activate");
    t.resolve();
    // Three emblem triggers, one for each Octopus.
    assert_eq!(triggers_on_stack(&t, "fight"), 3);
}

#[test]
fn ace_nitro_9_counter_depends_on_the_target() {
    cr!("603.12", "608.2b", "115.1");
    ruling!(
        "Ace, Fearless Rebel",
        "If you choose no target for Ace's Nitro-9 ability, you will put a +1/+1 counter on it. However, if you choose a target and it is an illegal target as the ability tries to resolve, you won't put a +1/+1 counter on Ace. In either case, no fight will happen and no creature will deal or be dealt damage."
    );
    supported("Ace, Fearless Rebel");
    for with_target in [false, true] {
        let mut t = TestGame::new(2);
        let ace = t.battlefield(P0, "Ace, Fearless Rebel");
        let thopter = t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(thopter)]);
        if with_target {
            t.answer_targets(P0, &[Entity::Object(bears)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        attack_with(&mut t, &[(ace, Entity::Player(P1))]);
        // Resolve the attack trigger (sacrificing the Ornithopter); the reflexive trigger
        // goes on the stack.
        t.resolve();
        assert!(!t.on_battlefield(thopter));
        assert_eq!(t.stack_len(), 1);
        if with_target {
            make_illegal(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.counters(ace, counters::PLUS1),
            if with_target { 0 } else { 1 }
        );
        no_damage(&t, &[ace, bears]);
    }
}

#[test]
fn curse_of_the_werefox_reflexive_trigger_targets_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Curse of the Werefox",
        "You don't choose a target for the creature to fight at the time you cast this spell. Rather, a second \"reflexive\" ability triggers when you create a Monster Role token attached to the creature that was targeted by the spell. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Curse of the Werefox");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Curse of the Werefox", &[Entity::Object(mine)]);
    // One target chosen while casting.
    assert_eq!(target_candidates(&t, P0, from).len(), 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    // The reflexive trigger is on the stack, targeting the Bears; P1 may respond.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from).len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(mine), (4, 4));
}

#[test]
fn curse_of_the_werefox_without_the_role_there_is_no_fight() {
    cr!("603.12", "303.4i", "702.16c");
    ruling!(
        "Curse of the Werefox",
        "If you can't attach a Monster Role token to the target creature you control when Curse of the Werefox resolves, the reflexive trigger that causes the fight won't happen."
    );
    supported("Curse of the Werefox");
    // The target gains protection from enchantments in response: it's still a legal
    // target for the sorcery, but a Role (an Aura) can't be attached to it.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Curse of the Werefox", &[Entity::Object(mine)]);
    grant(
        &mut t,
        P0,
        mine,
        Keyword::with_filter(
            KeywordKind::Protection,
            Filter::Type(mtg_engine::types::CardType::Enchantment),
        ),
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    no_damage(&t, &[mine, bears]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn vorinclex_grand_evolution_chapter_iii_affects_only_creatures_then() {
    cr!("611.2c", "714.2b");
    ruling!(
        "Vorinclex // The Grand Evolution",
        "The set of creatures that gain the activated fight ability is determined as the chapter III ability resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t gain that ability. Notably, Vorinclex won’t be back on the battlefield yet, so it won’t gain the ability either."
    );
    supported("Vorinclex // The Grand Evolution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let saga =
        crate::r_s17_common::enter_transformed(&mut t, P0, "Vorinclex // The Grand Evolution");
    t.resolve_all();
    crate::r_s29_common::put_counters(&mut t, saga, counters::LORE, 1);
    t.resolve_all();
    crate::r_s29_common::put_counters(&mut t, saga, counters::LORE, 1);
    t.resolve_all();
    let fights = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .any(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("fights"))
    };
    assert!(fights(&mut t, bears));
    let later = enter(&mut t, P0, "Llanowar Elves");
    assert!(!fights(&mut t, later));
    let v = t.named_on_battlefield("Vorinclex");
    assert_eq!(v.len(), 1);
    assert!(!fights(&mut t, v[0]));
}
