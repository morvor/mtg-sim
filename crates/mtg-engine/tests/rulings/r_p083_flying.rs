//! Rulings batch P083 — flying hate: blocking restrictions (CR 509.1b, 702.9b, 702.17b),
//! targets "creature with flying" (CR 115.1, 608.2b), divided damage (CR 601.2d),
//! "you may" choices on resolution of targeted triggers (CR 603.3d, 608.2d), modes with
//! targets (CR 700.2a), and "enchant creature you control" (CR 303.4, 704.5m).

use crate::r_p083_common::*;
use crate::r_s06_common::give_control;
use crate::r_s21_common::{blocks_now, castable, legal_blocks};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 attacks P0 with `attackers` (from P1's beginning of combat); stops before blocks.
fn p1_attacks(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P1, Step::BeginningOfCombat);
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, player(P0))).collect();
    crate::r_s01_common::attack_with(t, &at);
}

#[test]
fn dense_canopy_reach_blocks_anything() {
    cr!("509.1b", "702.17b", "702.9b");
    ruling!(
        "Dense Canopy",
        "A creature with reach can still block creatures with flying as well as creatures without flying."
    );
    supported("Dense Canopy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dense Canopy");
    let spider = t.battlefield(P0, "Giant Spider");
    let angel = t.battlefield(P0, "Serra Angel");
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    p1_attacks(&mut t, &[drake, bears]);
    assert!(legal_blocks(&mut t, P0, &[(spider, bears)]));
    assert!(legal_blocks(&mut t, P0, &[(spider, drake)]));
    assert!(legal_blocks(&mut t, P0, &[(angel, drake)]));
    assert!(!legal_blocks(&mut t, P0, &[(angel, bears)]), "flying blocks only flying");
}

#[test]
fn bower_passage_reach_still_blocks_and_late_flying_doesnt_matter() {
    cr!("509.1b", "509.1h", "702.17b", "702.9b");
    ruling!(
        "Bower Passage",
        "Creatures with reach can still block creatures you control."
    );
    ruling!(
        "Bower Passage",
        "Bower Passage won’t affect a blocking creature that gains flying after blockers are declared."
    );
    supported("Bower Passage");
    supported("Jump");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Bower Passage");
    let spider = t.battlefield(P0, "Giant Spider");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Craw Wurm");
    p1_attacks(&mut t, &[attacker]);
    assert!(legal_blocks(&mut t, P0, &[(spider, attacker)]));
    assert!(!legal_blocks(&mut t, P0, &[(angel, attacker)]));
    // The Bears block, then gain flying from Jump: they stay blocking and deal damage.
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(bears, attacker)]));
    t.advance_to(P1, Step::DeclareBlockers);
    cast_resolve(&mut t, P0, "Jump", &[obj(bears)]);
    assert!(t.obj_now(bears).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(blocks_now(&t), vec![(bears, attacker)]);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(damage_on(&t, attacker), 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn dizzying_gaze_only_on_your_own_creature() {
    cr!("303.4a", "303.4d", "704.5m", "115.1");
    ruling!(
        "Dizzying Gaze",
        "Dizzying Gaze can’t be cast targeting an opponent’s creature. And this card is put into the graveyard if ever on a creature you do not control."
    );
    supported("Dizzying Gaze");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Trying to target P1's creature: the spell doesn't get that target (with no legal
    // choice the engine falls back to the only legal one, P0's Bears).
    let gaze = cast_new(&mut t, P0, "Dizzying Gaze", &[obj(theirs)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, gaze), vec![obj(mine)]);
    t.resolve_all();
    assert_eq!(t.obj_now(gaze).attached_to, Some(obj(mine)));
    let gaze = t.named_on_battlefield("Dizzying Gaze");
    assert_eq!(gaze.len(), 1);
    // P1 gains control of the enchanted Bears: the Aura is put into the graveyard.
    give_control(&mut t, mine, P1);
    t.settle();
    assert!(t.in_graveyard(P0, "Dizzying Gaze"));
}

// ---------------------------------------------------------------------------------
// Spells that target creatures with flying.
// ---------------------------------------------------------------------------------

#[test]
fn aerial_volley_divides_among_one_to_three_flyers() {
    cr!("601.2d", "608.2b", "115.1");
    ruling!(
        "Aerial Volley",
        "You choose how many targets Aerial Volley has and how the damage is divided as you cast the spell. Each target must receive at least 1 damage."
    );
    ruling!(
        "Aerial Volley",
        "If some (but not all) of the targets become illegal before Aerial Volley tries to resolve, the original division of damage still applies, but no damage is dealt to illegal targets. If all targets become illegal, Aerial Volley won't resolve."
    );
    supported("Aerial Volley");
    // One target: all 3 damage.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    cast_resolve(&mut t, P0, "Aerial Volley", &[obj(angel)]);
    assert_eq!(damage_on(&t, angel), 3);
    // Two targets, 2 and 1; the first becomes illegal (it loses... it leaves the
    // battlefield): the second still gets 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Serra Angel");
    let b = t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Forest", 1);
    let volley = t.hand(P0, "Aerial Volley");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, volley).targets(&[obj(a), obj(b)]).go();
    cast_resolve(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, b), 1);
    // A creature without flying can't be a target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    let targets = spell_targets(&mut t, P0, "Aerial Volley");
    assert!(targets.contains(&obj(angel)));
    assert!(!targets.contains(&obj(bears)));
    // All targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    cast_new(&mut t, P0, "Aerial Volley", &[obj(angel)]);
    cast_resolve(&mut t, P1, "Unsummon", &[obj(angel)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Aerial Volley"));
}

#[test]
fn pistus_strike_and_aerial_predation_need_their_target() {
    cr!("608.2b", "115.1", "601.2c");
    ruling!(
        "Pistus Strike",
        "If the creature is an illegal target when Pistus Strike tries to resolve, the spell doesn’t resolve. No player gets a poison counter."
    );
    ruling!(
        "Aerial Predation",
        "If the creature with flying is an illegal target when Aerial Predation tries to resolve, it won't resolve and none of its effects will happen. You won't gain 2 life."
    );
    ruling!(
        "Aerial Predation",
        "You must be able to target a creature with flying to cast Aerial Predation."
    );
    supported("Pistus Strike");
    supported("Aerial Predation");
    for name in ["Pistus Strike", "Aerial Predation"] {
        // The target returns to its owner's hand in response.
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P1, "Serra Angel");
        cast_new(&mut t, P0, name, &[obj(angel)]);
        cast_resolve(&mut t, P1, "Unsummon", &[obj(angel)]);
        assert!(t.in_graveyard(P0, name));
        assert_eq!(t.g.player(P1).poison(), 0, "{name}");
        assert_eq!(t.life(P0), 20, "{name}");
        // With its target, it does everything.
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P1, "Serra Angel");
        cast_resolve(&mut t, P0, name, &[obj(angel)]);
        assert!(t.in_graveyard(P1, "Serra Angel"));
        if name == "Pistus Strike" {
            assert_eq!(t.g.player(P1).poison(), 1);
        } else {
            assert_eq!(t.life(P0), 22);
        }
    }
    // No creature with flying: Aerial Predation can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Aerial Predation");
    let c = t.hand(P0, "Aerial Predation");
    assert!(!castable(&mut t, P0, c));
    t.battlefield(P1, "Wind Drake");
    assert!(castable(&mut t, P0, c));
}

#[test]
fn pistus_strike_poisons_even_if_the_creature_survives() {
    cr!("608.2c", "701.19c", "702.12b");
    ruling!(
        "Pistus Strike",
        "If the creature with flying regenerates or has indestructible, its controller will still get a poison counter."
    );
    supported("Darksteel Gargoyle");
    // Indestructible.
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P1, "Darksteel Gargoyle");
    cast_resolve(&mut t, P0, "Pistus Strike", &[obj(gargoyle)]);
    assert!(t.on_battlefield(gargoyle));
    assert_eq!(t.g.player(P1).poison(), 1);
    // Regenerated: a regeneration shield from Mossbridge... (Regenerate via an effect).
    supported("Mending Touch");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    cast_resolve(&mut t, P1, "Mending Touch", &[obj(drake)]);
    cast_resolve(&mut t, P0, "Pistus Strike", &[obj(drake)]);
    assert!(t.on_battlefield(drake));
    assert_eq!(t.g.player(P1).poison(), 1);
}

#[test]
fn enters_trigger_targeting_a_flyer_is_mandatory() {
    cr!("603.3d", "115.1");
    ruling!(
        "Deadshot Minotaur",
        "The \"enters\" ability is mandatory. If you're the only player who controls a creature with flying, you must target one of those creatures."
    );
    supported("Deadshot Minotaur");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Deadshot Minotaur");
    t.resolve_all();
    assert_eq!(damage_on(&t, angel), 3);
}

/// The "you may" choices P0 was asked since decision `from`.
fn may_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn target_chosen_on_the_trigger_damage_chosen_on_resolution() {
    cr!("603.3d", "608.2d", "603.5");
    ruling!(
        "Geistcatcher's Rig",
        "The target creature with flying is chosen when the ability triggers and goes on the stack. You choose whether or not Geistcatcher’s Rig will deal 4 damage to it when then ability resolves."
    );
    ruling!(
        "Stingerfling Spider",
        "You choose the target creature with flying when the ability goes on the stack. When it resolves, you choose whether or not to destroy the creature."
    );
    ruling!(
        "Tajuru Archer",
        "You target a creature when the ability triggers. You decide whether to have Tajuru Archer deal damage to that creature as the ability resolves."
    );
    for name in ["Geistcatcher's Rig", "Stingerfling Spider", "Tajuru Archer"] {
        supported(name);
        for yes in [false, true] {
            let mut t = TestGame::new(2);
            let angel = t.battlefield(P1, "Serra Angel");
            t.answer_targets(P0, &[obj(angel)]);
            t.enter(P0, name);
            t.settle();
            // On the stack with its target, and no "you may" asked yet.
            assert_eq!(t.stack_len(), 1, "{name}");
            let ab = *t.g.stack.last().unwrap();
            assert_eq!(crate::r_s25_common::targets_of(&t, ab), vec![obj(angel)]);
            let from = t.asked().len();
            assert_eq!(may_asked(&t, 0), 0, "{name}: not asked on the trigger");
            t.answer_yes(P0, yes);
            t.resolve_all();
            assert_eq!(may_asked(&t, from), 1, "{name}: asked on resolution");
            let hit = !t.on_battlefield(angel) || damage_on(&t, angel) > 0;
            assert_eq!(hit, yes, "{name}: yes = {yes}");
        }
    }
}

#[test]
fn branching_bolt_modes_need_legal_targets() {
    cr!("700.2a", "601.2b", "601.2c");
    ruling!(
        "Branching Bolt",
        "You may choose just the first mode (targeting a creature with flying), just the second mode (targeting a creature without flying), or both modes (targeting a creature with flying and a creature without flying). You can’t choose a mode unless there’s a legal target for it."
    );
    supported("Branching Bolt");
    // Both modes.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Branching Bolt");
    let bb = t.hand(P0, "Branching Bolt");
    t.cast(P0, bb)
        .modes(&[0, 1])
        .target(obj(drake))
        .target(obj(bears))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wind Drake"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Just the second mode.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Branching Bolt");
    let bb = t.hand(P0, "Branching Bolt");
    t.cast(P0, bb).modes(&[1]).targets(&[obj(bears)]).go();
    t.resolve_all();
    assert!(t.on_battlefield(drake));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // No creature with flying: the first mode can't be chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Branching Bolt");
    let bb = t.hand(P0, "Branching Bolt");
    let from = t.asked().len();
    t.cast(P0, bb).modes(&[0, 1]).targets(&[obj(bears)]).go();
    let offered: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![1]], "only the second mode is available");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}
