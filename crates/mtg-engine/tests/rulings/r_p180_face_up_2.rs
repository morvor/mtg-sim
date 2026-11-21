//! Rulings batch P180 — "when this creature is turned face up" abilities, continued:
//! Thelonite Hermit, Jeering Instigator, Willbender, Brine Elemental, Culvert Ambusher,
//! Frontline Strategist, Silumgar Assassin and Shaper Parasite.

use crate::r_p180_common::*;
use crate::r_s06_common::{attach_new, damage};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn thelonite_hermits_bonus_ending_can_make_damage_lethal() {
    cr!("704.5g");
    ruling!(
        "Thelonite Hermit",
        "nonlethal damage dealt to a Saproling may become lethal if Thelonite Hermit leaves the battlefield during that turn."
    );
    supported("Thelonite Hermit");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Grizzly Bears");
    let hermit = t.battlefield(P0, "Thelonite Hermit");
    let sap = crate::r_s02_common::create_token(&mut t, P0, "Saproling");
    assert_eq!(t.pt(sap), (2, 2));
    damage(&mut t, src, 1, sap);
    assert!(t.on_battlefield(sap));
    destroy(&mut t, hermit);
    assert!(!t.on_battlefield(sap), "1 damage is lethal for a 1/1");
}

#[test]
fn jeering_instigator_can_take_any_creature_but_not_its_equipment() {
    cr!("301.5d", "302.6");
    ruling!(
        "Jeering Instigator",
        "Gaining control of a creature doesn't cause you to gain control of any Auras or Equipment attached to that creature."
    );
    ruling!(
        "Jeering Instigator",
        "Jeering Instigator's last ability can target any creature, even one that's untapped or one you already control."
    );
    supported("Jeering Instigator");
    // An opponent's untapped, equipped creature.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    let inst = morph(&mut t, P0, "Jeering Instigator");
    t.answer_targets(P0, &[obj(bears)]);
    turn_up(&mut t, P0, inst, &[("Mountain", 1), ("Wastes", 2)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(splitter).controller, P1);
    assert_eq!(t.obj_now(splitter).attached_to, Some(obj(bears)));
    // A tapped, summoning-sick creature you already control: untapped, with haste.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.tap(mine);
    let inst = morph(&mut t, P0, "Jeering Instigator");
    t.answer_targets(P0, &[obj(mine)]);
    turn_up(&mut t, P0, inst, &[("Mountain", 1), ("Wastes", 2)]);
    t.resolve_all();
    assert!(!t.obj_now(mine).tapped);
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, mine));
}

/// P0's face-down Willbender, with lands for its morph cost {1}{U}.
fn willbender(t: &mut TestGame) -> ObjectId {
    let w = morph(t, P0, "Willbender");
    t.lands(P0, "Island", 2);
    w
}

#[test]
fn willbender_cant_target_a_spell_with_more_than_one_target() {
    cr!("115.7a");
    ruling!(
        "Willbender",
        "If a spell or ability targets multiple things, you can't target it with Willbender's triggered ability, even if all but one of those targets have become illegal."
    );
    ruling!(
        "Willbender",
        "If a spell or ability targets the same player or object multiple times, you can't target it with Willbender's ability."
    );
    supported("Willbender");
    let mut t = TestGame::new(2);
    let w2 = willbender(&mut t);
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Hill Giant");
    let c2 = t.battlefield(P1, "Grizzly Bears");
    // Twin Bolt: 2 damage divided between Grizzly Bears and Hill Giant.
    give_mana_for(&mut t, P1, "Twin Bolt");
    let twin = t.hand(P1, "Twin Bolt");
    t.answer_targets(P1, &[obj(a2), obj(b2)]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, twin, mtg_engine::object::CastMethod::Normal)
        .expect("Twin Bolt");
    // Seeds of Strength targets P1's Grizzly Bears three times.
    let seeds = cast_targeting(&mut t, P1, "Seeds of Strength", &[obj(c2), obj(c2), obj(c2)]);
    // Shock has a single target.
    let shock = cast_targeting(&mut t, P1, "Shock", &[obj(a2)]);
    // One of Twin Bolt's targets is gone.
    destroy(&mut t, b2);
    let from = t.asked().len();
    assert!(turn_face_up(&mut t, P0, w2));
    t.settle();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1, "{cands:?}");
    assert!(cands[0].contains(&obj(shock)));
    assert!(!cands[0].contains(&obj(twin)));
    assert!(!cands[0].contains(&obj(t.g.current(twin))));
    assert!(!cands[0].contains(&obj(seeds)));
}

#[test]
fn willbender_chooses_the_new_target_on_resolution_and_must_change_it_if_possible() {
    cr!("115.7a", "608.2c");
    ruling!(
        "Willbender",
        "You don't choose the new target for the spell until Willbender's triggered ability resolves. You must change the target if possible."
    );
    supported("Willbender");
    // Doom Blade on P0's Grizzly Bears: the only other nonblack creature is Willbender,
    // so that's the new target, even though the Bears are gone by then.
    for bears_gone in [false, true] {
        let mut t = TestGame::new(2);
        let w = willbender(&mut t);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Vampire Nighthawk");
        let blade = cast_targeting(&mut t, P1, "Doom Blade", &[obj(bears)]);
        t.answer_targets(P0, &[obj(blade)]);
        assert!(turn_face_up(&mut t, P0, w));
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if bears_gone {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Willbender"), "{bears_gone}");
        if !bears_gone {
            assert!(t.on_battlefield(bears));
        }
    }
    // No other legal target: Duress keeps targeting the only opponent.
    let mut t = TestGame::new(2);
    let w = willbender(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    let duress = cast_targeting(&mut t, P1, "Duress", &[Entity::Player(P0)]);
    t.answer_targets(P0, &[obj(duress)]);
    assert!(turn_face_up(&mut t, P0, w));
    t.settle();
    t.resolve();
    assert_eq!(
        t.g.obj(t.g.current(duress))
            .stack
            .as_ref()
            .map(|s| s.chosen[0].targets.clone()),
        Some(vec![vec![Entity::Player(P0)]])
    );
}

/// Turns `n` face-down Brine Elementals of P0's face up.
fn brine_elementals(t: &mut TestGame, n: usize) {
    supported("Brine Elemental");
    for _ in 0..n {
        let b = morph(t, P0, "Brine Elemental");
        turn_up(t, P0, b, &[("Island", 2), ("Wastes", 5)]);
        t.resolve_all();
    }
}

#[test]
fn brine_elemental_skips_are_cumulative() {
    cr!("614.10", "614.10a", "502.3");
    ruling!(
        "Brine Elemental",
        "Skipping your \"next\" untap step is cumulative."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let land = t.battlefield(P1, "Forest");
    brine_elementals(&mut t, 2);
    for (turn, untapped) in [(1, false), (2, false), (3, true)] {
        t.g.tap(land);
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(!t.obj_now(land).tapped, untapped, "P1's turn {turn}");
        t.advance_to(P0, Step::Upkeep);
    }
}

#[test]
fn doesnt_untap_during_next_untap_step_waits_for_one_that_isnt_skipped() {
    cr!("614.10a", "502.3");
    ruling!(
        "Brine Elemental",
        "If an effect says that something doesn't untap during its controller's next untap step, that effect applies to the next untap step that isn't skipped."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    // Frost Breath: "Tap up to two target creatures. Those creatures don't untap during
    // their controller's next untap step."
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Frost Breath", &[obj(bears)]);
    assert!(t.obj_now(bears).tapped);
    brine_elementals(&mut t, 1);
    t.g.tap(land);
    // P1's next untap step is skipped.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(land).tapped && t.obj_now(bears).tapped);
    // The following one: the land untaps, the Bears don't.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(land).tapped);
    assert!(t.obj_now(bears).tapped);
    // Then the Bears untap.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn culvert_ambushers_target_doesnt_block_if_its_tapped_or_cant_block() {
    cr!("509.1c");
    ruling!(
        "Culvert Ambusher",
        "If the target creature is tapped or is affected by a spell or ability that says it can't block, then it doesn't block."
    );
    supported("Culvert Ambusher");
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let attacker = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if tapped {
            t.g.tap(bears);
        }
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Culvert Ambusher");
        t.resolve_all();
        // P1 declares no blockers; a block the requirement demands is made.
        t.attack(&[(attacker, Entity::Player(P1))], &[]);
        if tapped {
            assert_eq!(t.life(P1), 17, "no block");
        } else {
            assert_eq!(t.life(P1), 20, "it blocked");
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        }
    }
}

#[test]
fn frontline_strategist_checks_creature_types_as_damage_is_dealt() {
    cr!("615.1");
    ruling!(
        "Frontline Strategist",
        "The creature type is checked for “non-Soldier” at the time the damage would be dealt."
    );
    supported("Frontline Strategist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fs = morph(&mut t, P0, "Frontline Strategist");
    turn_up(&mut t, P0, fs, &[("Plains", 1)]);
    t.resolve_all();
    // Creatures that arrive afterwards are affected too: a non-Soldier deals no combat
    // damage, a Soldier does.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let soldier = t.battlefield(P0, "Elite Vanguard");
    t.attack(
        &[(bears, Entity::Player(P1)), (soldier, Entity::Player(P1))],
        &[],
    );
    assert_eq!(t.life(P1), 18);
}

#[test]
fn silumgar_assassins_block_restriction_is_checked_only_as_blockers_are_declared() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Silumgar Assassin",
        "The powers of Silumgar Assassin and the creature trying to block it are checked only as blockers are declared."
    );
    supported("Silumgar Assassin");
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Silumgar Assassin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(assassin, Entity::Player(P1))]);
    assert!(!can_block(&mut t, giant, assassin), "power 3 > 2");
    assert!(can_block(&mut t, bears, assassin));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, assassin)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    cast_resolve(&mut t, P1, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert!(t.in_graveyard(P0, "Silumgar Assassin"));
}

#[test]
fn shaper_parasite_chooses_the_bonus_as_it_resolves() {
    cr!("608.2c");
    ruling!(
        "Shaper Parasite",
        "You choose whether the targeted creature gets +2/-2 or -2/+2 as the ability resolves."
    );
    supported("Shaper Parasite");
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let parasite = morph(&mut t, P0, "Shaper Parasite");
        t.answer_targets(P0, &[obj(bears)]);
        let from = t.asked().len();
        turn_up(&mut t, P0, parasite, &[("Island", 1), ("Wastes", 2)]);
        assert_eq!(t.stack_len(), 1);
        let chose = |t: &TestGame| {
            t.asked()[from..]
                .iter()
                .filter(|(_, d)| {
                    matches!(d, Decision::ChooseOption { .. } | Decision::ChooseModes { .. })
                })
                .count()
        };
        assert_eq!(chose(&t), 0, "no choice yet");
        t.answer(P0, DecisionKind::Any, Answer::Index(pick));
        t.resolve_all();
        assert_eq!(chose(&t), 1);
        if pick == 0 {
            assert!(t.in_graveyard(P1, "Grizzly Bears"), "4/0");
        } else {
            assert_eq!(t.pt(bears), (0, 4));
        }
    }
}
