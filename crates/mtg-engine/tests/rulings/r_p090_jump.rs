//! Rulings batch P090 — "target creature gains flying" (jump effects) and their cards:
//! flying given after blocks (CR 509.1h, 702.9b), Eutropia's and Gryffwing Cavalry's
//! targets (CR 115.1, 603.3d, 118.12), Landroval's "attack a player" (CR 508.1, 506.2),
//! Helitrooper's equip cost reduction (CR 702.6, 601.2f), Ajani's abilities (CR 606),
//! and Dropkick Bomber's anthem (CR 120.6).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{damage, has_kw};
use crate::r_s07_common::damage_on;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn dropkick_bomber_leaving_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4c");
    ruling!(
        "Dropkick Bomber",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Goblin you control may become lethal if Dropkick Bomber leaves the battlefield during that turn."
    );
    supported("Dropkick Bomber");
    let mut t = TestGame::new(2);
    let bomber = t.battlefield(P0, "Dropkick Bomber");
    let goblin = t.battlefield(P0, "Raging Goblin");
    assert_eq!(t.pt(goblin), (2, 2));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 1, goblin);
    t.settle();
    assert!(t.on_battlefield(goblin));
    destroy(&mut t, bomber);
    t.settle();
    assert!(!t.on_battlefield(goblin));
}

#[test]
fn eutropia_targets_itself_a_flier_or_the_entering_enchantment_creature() {
    cr!("115.1", "702.9c", "603.2");
    ruling!(
        "Eutropia the Twice-Favored",
        "Eutropia can be the target of its own triggered ability."
    );
    ruling!(
        "Eutropia the Twice-Favored",
        "Eutropia’s ability can target a creature that already has flying. It still gets a +1/+1 counter. Multiple instances of flying on the same creature are redundant."
    );
    ruling!(
        "Eutropia the Twice-Favored",
        "If an enchantment creature enters the battlefield under your control, Eutropia’s ability can target that creature."
    );
    supported("Eutropia the Twice-Favored");
    for which in 0..3 {
        let mut t = TestGame::new(2);
        let eu = t.battlefield(P0, "Eutropia the Twice-Favored");
        let drake = t.battlefield(P0, "Wind Drake");
        let from = t.asked().len();
        // Cyclops of Eternal Fury is an enchantment creature.
        let target = match which {
            0 => Some(eu),
            1 => Some(drake),
            _ => None,
        };
        if let Some(x) = target {
            t.answer_targets(P0, &[Entity::Object(x)]);
        }
        let cyc = t.enter(P0, "Cyclops of Eternal Fury");
        if target.is_none() {
            t.answer_targets(P0, &[Entity::Object(cyc)]);
        }
        t.settle();
        let cands: Vec<Entity> = asked_since(&t, from)
            .into_iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseTargets { candidates, .. } => Some(candidates),
                _ => None,
            })
            .flatten()
            .collect();
        for x in [eu, drake, cyc] {
            assert!(cands.contains(&Entity::Object(x)), "{which}: {cands:?}");
        }
        t.resolve_all();
        let x = target.unwrap_or(cyc);
        assert_eq!(t.counters(x, "+1/+1"), 1, "{which}: {}", t.dump_log());
        assert!(has_kw(&t, x, KeywordKind::Flying));
    }
}

/// P0 attacks P1 with `attacker`, P1 blocks with `blocker`; then P0 casts `spell` on the
/// attacker in the declare blockers step, and combat finishes.
fn flying_after_block(spell: &str) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, bears)]),
    );
    crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
    give_mana_for(&mut t, P0, spell);
    let s = t.hand(P0, spell);
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Flying), "{spell}");
    t.advance_to(P0, Step::EndOfCombat);
    (t, bears, giant)
}

#[test]
fn flying_after_blocks_doesnt_undo_the_block() {
    cr!("509.1h", "702.9b", "506.4");
    ruling!(
        "Mighty Leap",
        "Giving a creature flying after it’s already been blocked won’t change or undo that block. If you want to affect what can block the creature, you must cast Mighty Leap during the declare attackers step at the latest."
    );
    ruling!(
        "Aspirant's Ascent",
        "Giving an attacking creature flying after it has become blocked will not cause it to become unblocked, even if the blocking creature can't normally block creatures with flying."
    );
    for spell in ["Mighty Leap", "Aspirant's Ascent"] {
        supported(spell);
        let (t, bears, giant) = flying_after_block(spell);
        assert_eq!(t.life(P1), 20, "{spell}: the Bears stayed blocked");
        assert!(
            damage_on(&t, giant) > 0 || !t.on_battlefield(giant),
            "{spell}"
        );
        let _ = bears;
    }
    // Cast in the declare attackers step, Mighty Leap does change what can block: the
    // Hill Giant (no flying or reach) can't block the Bears any more.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    let block = [(giant, bears)];
    assert!(legal_blocks(&mut t, P1, &block));
    give_mana_for(&mut t, P0, "Mighty Leap");
    let s = t.hand(P0, "Mighty Leap");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &block));
}

#[test]
fn helitrooper_reduces_generic_equip_costs_only() {
    cr!("702.6a", "601.2f", "118.7a");
    ruling!(
        "Helitrooper",
        "Helitrooper's last ability reduces only the amount of generic mana you pay for equip abilities that target Helitrooper. For example, it will reduce {2} to {0}, but it will have no effect on an equip cost of {G}."
    );
    ruling!(
        "Helitrooper",
        "Reconfigure is not an equip ability, and reconfigure costs are not reduced by Helitrooper's last ability. Equip variants or equip abilities with restrictions, like the \"equip legendary creature\" ability of Wrecking Ball Arm, are still equip abilities, and their costs will be reduced by Helitrooper's last ability."
    );
    supported("Helitrooper");
    // (equipment, equip ability index, lands, lands tapped, makes Helitrooper legendary)
    let cases = [
        // Short Sword: Equip {1}: free.
        ("Short Sword", 0, 0, 0, false),
        // Greatsword of Tyr: Equip {W}: still {W}.
        ("Greatsword of Tyr", 0, 1, 1, false),
        // Wrecking Ball Arm: Equip legendary creature {3}: {1}.
        ("Wrecking Ball Arm", 0, 3, 1, true),
        // Wrecking Ball Arm: Equip {7}: {5}.
        ("Wrecking Ball Arm", 1, 7, 5, false),
    ];
    for (eq, idx, lands, tapped, legendary) in cases {
        supported(eq);
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Helitrooper");
        if legendary {
            supported("Leyline of Singularity");
            t.battlefield(P0, "Leyline of Singularity");
        }
        let e = t.battlefield(P0, eq);
        t.lands(P0, "Plains", lands);
        t.set_step(P0, Step::PrecombatMain);
        let r = t.activate(P0, e, idx, &[Entity::Object(h)]);
        assert!(r.is_ok(), "{eq} {idx}: {r:?}");
        t.resolve_all();
        assert_eq!(t.obj_now(e).attached_to, Some(Entity::Object(h)), "{eq}");
        assert_eq!(tapped_lands(&t, P0), tapped, "{eq} {idx}");
    }
    // Lizard Blades: Reconfigure {2} isn't reduced.
    supported("Lizard Blades");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Helitrooper");
    let blades = t.battlefield(P0, "Lizard Blades");
    t.lands(P0, "Plains", 2);
    t.set_step(P0, Step::PrecombatMain);
    let r = t.activate(P0, blades, 0, &[Entity::Object(h)]);
    assert!(r.is_ok(), "{r:?}");
    t.resolve_all();
    assert_eq!(t.obj_now(blades).attached_to, Some(Entity::Object(h)));
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn landroval_triggers_per_player_attacked_by_two_creatures() {
    cr!("508.1", "506.2", "603.2c");
    ruling!(
        "Landroval, Horizon Witness",
        "In a multiplayer game, Landroval, Horizon Witness's triggered ability triggers once for each player you attack with at least two creatures."
    );
    ruling!(
        "Landroval, Horizon Witness",
        "Two or more creatures you control must attack the same player in order for Landroval, Horizon Witness's ability to trigger. Creatures you control that attack planeswalkers that player controls or battles that player is protecting won't count toward Landroval, Horizon Witness's trigger condition."
    );
    supported("Landroval, Horizon Witness");
    // Two at P1, two at P2: two triggers.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Landroval, Horizon Witness");
    let c: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    to_beginning_of_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[
            (c[0], Entity::Player(P1)),
            (c[1], Entity::Player(P1)),
            (c[2], Entity::Player(P2)),
            (c[3], Entity::Player(P2)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "gains flying"), 2, "{}", t.dump_log());
    // One at P1, one at P1's planeswalker, one at P2: none.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Landroval, Horizon Witness");
    let pw = t.battlefield(P1, "Liliana of the Veil");
    let c: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    to_beginning_of_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[
            (c[0], Entity::Player(P1)),
            (c[1], Entity::Object(pw)),
            (c[2], Entity::Player(P2)),
        ],
    );
    assert_eq!(t.g.combat.as_ref().map(|c| c.attackers.len()), Some(3));
    assert_eq!(triggers_on_stack(&t, "gains flying"), 0);
    // One at P1, one at a battle P1 protects: none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Landroval, Horizon Witness");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    let c: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    to_beginning_of_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(c[0], Entity::Player(P1)), (c[1], Entity::Object(battle))],
    );
    assert_eq!(t.g.combat.as_ref().map(|c| c.attackers.len()), Some(2));
    assert_eq!(triggers_on_stack(&t, "gains flying"), 0);
}

#[test]
fn ajani_caller_of_the_pride_abilities() {
    cr!("606.4", "115.1", "608.2h");
    ruling!(
        "Ajani, Caller of the Pride",
        "You can activate the first ability without a target. You'll still put one loyalty counter on Ajani when you activate the ability."
    );
    ruling!(
        "Ajani, Caller of the Pride",
        "The number of Cat tokens you put onto the battlefield is equal to your life total when the third ability resolves."
    );
    supported("Ajani, Caller of the Pride");
    // +1 with no creature to target.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Caller of the Pride");
    let loyalty = t.counters(ajani, "loyalty");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, ajani, 0, &[]).expect("+1 without a target");
    assert_eq!(t.counters(ajani, "loyalty"), loyalty + 1);
    t.resolve_all();
    // −8: P0 is at 20 when it's activated and loses 15 in response: 5 Cats.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Caller of the Pride");
    crate::r_s13_common::add(&mut t, ajani, "loyalty", 8);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, ajani, 2, &[]).expect("-8");
    t.g.lose_life(P0, 15);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Cat").len(),
        5
    );
}

#[test]
fn gryffwing_cavalry_target_first_then_the_cost() {
    cr!("603.3d", "118.12", "601.2c");
    ruling!(
        "Gryffwing Cavalry",
        "You must choose a target for Gryffwing Cavalry's last ability before you choose whether or not to pay the cost. If there is no legal target, you won't have the opportunity to pay the cost."
    );
    supported("Gryffwing Cavalry");
    let pay_offers = |t: &TestGame, from: usize| {
        asked_since(t, from)
            .into_iter()
            .filter(|(p, d)| {
                *p == P0 && matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })
            })
            .count()
    };
    // Alone (it has flying): no legal target, no payment offered.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gryffwing Cavalry");
    t.lands(P0, "Plains", 2);
    to_beginning_of_combat(&mut t, P0);
    let from = t.asked().len();
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(pay_offers(&t, from), 0);
    assert_eq!(tapped_lands(&t, P0), 0);
    // With Grizzly Bears attacking too: the Bears are targeted, then P0 pays.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gryffwing Cavalry");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[(g, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    let order: Vec<&str> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { .. } if *p == P0 => Some("target"),
            Decision::YesNo { .. } | Decision::OptionalCost { .. } if *p == P0 => Some("pay"),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["target", "pay"]);
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(tapped_lands(&t, P0), 2);
}
