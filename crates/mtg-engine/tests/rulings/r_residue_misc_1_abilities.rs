//! Item `rulings-residue-misc-1` (pass C) — activated and triggered abilities of batch
//! P112: Splinter doubling Elite Arcanist's linked ability (CR 607.2a, 603.2d), Inniaz
//! (resolves before blocks), Frodo's checks on resolution (CR 608.2b-ish "if" effects,
//! 611.2c), Smoke Teller's look (CR 708.5), The Wandering Minstrel (intervening "if",
//! CR 603.4; X on resolution), Mirrorwood Treefolk (the next time damage would be dealt,
//! CR 615.1, 510.2), Agrus Kos's copies (CR 707.10c), Yasova's targets checked twice
//! (CR 608.2b), Najeela's extra combat (CR 500.8), Kill-Suit Cultist's replacement
//! (CR 614.1a), Wick's last known information (CR 608.2h), Mizzix's single payment, Acidic
//! Dagger's timing in a second combat (CR 506.8), Feral Encounter's delayed targets
//! (CR 603.7c), Deadshot and Ram Through (illegal targets, CR 608.2b).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::{can_look_at, turn_face_down};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Splinter, Radical Rat and Elite Arcanist
// ---------------------------------------------------------------------------------------

#[test]
fn splinter_doubles_elite_arcanists_exile_and_x_is_the_sum() {
    cr!("607.2a", "603.2d", "107.3", "205.3m");
    ruling!(
        "Splinter, Radical Rat",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. For example, if Elite Arcanist's enters ability triggers twice (probably because it entered as a Ninja due to another effect), two cards are exiled. The value of X in the activation cost of Elite Arcanist's other ability is the sum of the two cards' mana values. As the ability resolves, you create copies of both cards and can cast none, one, or both of the copies in any order."
    );
    supported("Splinter, Radical Rat");
    supported("Elite Arcanist");
    supported("Maskwood Nexus");
    // Maskwood Nexus makes the Arcanist a Ninja: Splinter makes its enters ability trigger
    // twice. Opt and Shock are exiled (X = 2); P0 casts only the Shock's copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Splinter, Radical Rat");
    t.battlefield(P0, "Maskwood Nexus");
    let opt = t.hand(P0, "Opt");
    let shock = t.hand(P0, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(opt)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(shock)]);
    let arc = cast_new(&mut t, P0, "Elite Arcanist", &[]);
    t.resolve_all();
    assert!(t.in_exile("Opt") && t.in_exile("Shock"));
    let arc = t.g.current(arc);
    t.g.objects[arc.0 as usize].summoning_sick = false;
    tap_lands(&mut t, P0);
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate(&mut t, P0, arc), "X is 2, not 1");
    t.lands(P0, "Wastes", 1);
    assert!(can_activate(&mut t, P0, arc));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, arc, "Copy").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Inniaz, Frodo
// ---------------------------------------------------------------------------------------

#[test]
fn inniaz_resolves_before_blockers() {
    cr!("508.1m", "509.1", "608.2f");
    ruling!(
        "Inniaz, the Gale Force",
        "Inniaz’s ability resolves before blockers are chosen."
    );
    supported("Inniaz, the Gale Force");
    let mut t = TestGame::new(2);
    let inniaz = t.battlefield(P0, "Inniaz, the Gale Force");
    let crows: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Storm Crow")).collect();
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[obj(theirs)]);
    t.answer_choose(P0, &[obj(mine)]);
    attack_with(
        &mut t,
        &[(inniaz, pl(P1)), (crows[0], pl(P1)), (crows[1], pl(P1))],
    );
    t.resolve_all();
    // Still the declare attackers step, and control has changed: P1 may block with the
    // Bears it gained.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(controller(&mut t, theirs), P0);
    assert_eq!(controller(&mut t, mine), P1);
}

const FRODO: &str = "Frodo, Sauron's Bane";

#[test]
fn frodos_abilities_have_no_duration() {
    cr!("611.2c", "613.1d");
    ruling!(
        "Frodo, Sauron's Bane",
        "None of these abilities have a duration. If one of them resolves, it will remain in effect until the game ends, Frodo, Sauron's Bane leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    supported(FRODO);
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, FRODO);
    t.advance_to(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    // Two turns later it's still a 2/3 Halfling Scout with lifelink.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(pt_now(&mut t, f), (2, 3));
    assert!(t.obj_now(f).chars.has_subtype("Scout"));
    assert!(!t.obj_now(f).chars.has_subtype("Citizen"));
    assert!(has(&mut t, f, KeywordKind::Lifelink));
}

#[test]
fn frodos_abilities_check_its_types_as_they_resolve() {
    cr!("608.2c", "602.1");
    ruling!(
        "Frodo, Sauron's Bane",
        "You can activate Frodo, Sauron's Bane's abilities regardless of what creature types it is. Each of those abilities checks Frodo's creature types when that ability resolves. If Frodo, Sauron's Bane isn't the appropriate creature type at the time, that ability does nothing."
    );
    // The Scout ability activated while Frodo is a Citizen does nothing.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, FRODO);
    t.advance_to(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, f, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(f).chars.has_subtype("Rogue"));
    assert!(t.obj_now(f).chars.has_subtype("Citizen"));
    // Activated first, beneath the Citizen ability: by the time it resolves Frodo is a
    // Scout, so it becomes a Rogue.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, FRODO);
    t.advance_to(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, f, 1, &[]).unwrap();
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(f).chars.has_subtype("Rogue"));
    assert!(!t.obj_now(f).chars.has_subtype("Scout"));
}

// ---------------------------------------------------------------------------------------
// Smoke Teller
// ---------------------------------------------------------------------------------------

#[test]
fn smoke_teller_looks_privately_as_it_resolves() {
    cr!("708.5", "701.20e");
    interp!("looked-at-face-down-permanent-stays-seen");
    ruling!(
        "Smoke Teller",
        "Smoke Teller’s ability lets you look at the creature as the ability resolves. It doesn’t let you look at it later, although you should be given a reasonable amount of time to look at it. In a multiplayer game, be sure not to reveal the card to other players."
    );
    supported("Smoke Teller");
    let mut t = TestGame::new(3);
    let teller = t.battlefield(P0, "Smoke Teller");
    let theirs = t.battlefield(P1, "Serra Angel");
    assert!(turn_face_down(&mut t.g, theirs));
    t.g.recompute();
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    let ab = t.activate(P0, teller, 0, &[obj(theirs)]).unwrap();
    // Before it resolves, P0 can't look.
    assert!(ab.is_some());
    assert!(!can_look_at(&t.g, P0, theirs));
    t.resolve_all();
    // P0 has seen it (the engine keeps showing it to P0 while it stays the same
    // face-down permanent); P2 never sees it, and it isn't revealed.
    assert!(can_look_at(&t.g, P0, theirs));
    assert!(!can_look_at(&t.g, P2, theirs));
    assert!(!mtg_engine::reveal::is_revealed(&t.g, theirs));
    // Once it's a new object (turned face down again after leaving), the look is over.
    crate::r_p046_common::bounce(&mut t, theirs);
    let again = t.battlefield(P1, "Serra Angel");
    assert!(turn_face_down(&mut t.g, again));
    t.g.recompute();
    assert!(!can_look_at(&t.g, P0, again));
}

#[test]
fn smoke_teller_on_your_own_face_down_creature_gains_nothing() {
    cr!("708.5", "115.1");
    ruling!(
        "Smoke Teller",
        "You could target a creature you control with Smoke Teller’s ability, but there’s no benefit to doing so. Remember, you may look at face-down spells you control on the stack and face-down permanents you control at any time."
    );
    let mut t = TestGame::new(2);
    let teller = t.battlefield(P0, "Smoke Teller");
    let mine = t.battlefield(P0, "Serra Angel");
    assert!(turn_face_down(&mut t.g, mine));
    t.g.recompute();
    assert!(can_look_at(&t.g, P0, mine));
    assert!(!can_look_at(&t.g, P1, mine));
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    t.activate(P0, teller, 0, &[obj(mine)]).unwrap();
    t.resolve_all();
    assert!(can_look_at(&t.g, P0, mine));
    assert!(t.obj_now(mine).face_down);
}

// ---------------------------------------------------------------------------------------
// The Wandering Minstrel
// ---------------------------------------------------------------------------------------

const MINSTREL: &str = "The Wandering Minstrel";

#[test]
fn wandering_minstrel_checks_five_towns_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "The Wandering Minstrel",
        "The Wandering Minstrel's second ability checks at the moment it would trigger to see if you control five or more Towns. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control five or more Towns at that time, the ability won't resolve and none of its effects will happen."
    );
    supported(MINSTREL);
    supported("Capital City");
    for (towns, lose_one, token) in [(4, false, false), (5, true, false), (5, false, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, MINSTREL);
        let ts = t.lands(P0, "Capital City", towns);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "Elemental"), usize::from(towns >= 5));
        if lose_one {
            destroy(&mut t, ts[0]);
        }
        t.resolve_all();
        assert_eq!(
            tokens_of(&t, P0, "Elemental"),
            usize::from(token),
            "{towns} {lose_one}"
        );
    }
}

#[test]
fn wandering_minstrel_x_is_counted_once_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "The Wandering Minstrel",
        "The value of X is calculated only once, as The Wandering Minstrel's last ability resolves."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, MINSTREL);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Capital City", 2);
    t.advance_to(P0, Step::PrecombatMain);
    tap_lands(&mut t, P0);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, m, 0, &[]).unwrap();
    // A third Town in response counts; a fourth after it resolves doesn't.
    t.lands(P0, "Capital City", 1);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, bears), (5, 5));
    t.lands(P0, "Capital City", 1);
    assert_eq!(pt_now(&mut t, bears), (5, 5));
}

// ---------------------------------------------------------------------------------------
// Mirrorwood Treefolk, Kill-Suit Cultist
// ---------------------------------------------------------------------------------------

#[test]
fn mirrorwood_treefolk_redirects_all_simultaneous_combat_damage() {
    cr!("615.1a", "510.2", "614.9");
    ruling!(
        "Mirrorwood Treefolk",
        "The ability applies to the next time damage is dealt. Normally this means damage from one source at one time, but during combat it is possible for multiple sources to damage the Treefolk at one time, in which case damage from all of those sources is redirected."
    );
    supported("Mirrorwood Treefolk");
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Mirrorwood Treefolk");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.advance_to(P0, Step::PrecombatMain);
    crate::r_s03_common::to_blockers(&mut t, &[(tree, pl(P1))], &[(a, tree), (b, tree)]);
    tap_lands(&mut t, P0);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, tree, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17, "both blockers' damage");
    assert!(t.on_battlefield(tree));
    assert_eq!(damage_on(&t, tree), 0);
}

#[test]
fn kill_suit_cultist_destroys_instead_and_no_damage_is_dealt() {
    cr!("614.1a", "614.6");
    ruling!(
        "Kill-Suit Cultist",
        "This ability causes a creature to be destroyed instead of being dealt damage. Damage replaced this way isn’t dealt, and effects that would happen based on this damage (such as Demonfire’s “exile it” effect) don’t happen."
    );
    supported("Kill-Suit Cultist");
    supported("Demonfire");
    let mut t = TestGame::new(2);
    let cultist = t.battlefield(P0, "Kill-Suit Cultist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    t.activate(P0, cultist, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    let s = t.hand(P0, "Demonfire");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, s).x(1).target(obj(bears)).go();
    t.resolve_all();
    // Destroyed (in the graveyard, not exiled), and never dealt damage.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.in_exile("Grizzly Bears"));
    let dealt = t.dump_log().contains("deals 1 damage to Object");
    assert!(!dealt);
}

// ---------------------------------------------------------------------------------------
// Agrus Kos, Eternal Soldier
// ---------------------------------------------------------------------------------------

const AGRUS: &str = "Agrus Kos, Eternal Soldier";

/// The controllers of the abilities on the stack, bottom first.
fn stack_controllers(t: &TestGame) -> Vec<PlayerId> {
    t.g.stack.iter().map(|id| t.g.obj(*id).controller).collect()
}

#[test]
fn agrus_kos_copies_are_its_controllers_above_the_original() {
    cr!("707.10c", "603.3b");
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "The controller of Agrus Kos controls all the copies, no matter which player controlled the original ability. That player chooses the order the copies are put on the stack. The original ability will be on the stack beneath those copies and will resolve last."
    );
    supported(AGRUS);
    let mut t = TestGame::new(2);
    let agrus = t.battlefield(P0, AGRUS);
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.lands(P0, "Plains", 2);
    t.advance_to(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.activate(P1, pyro, 0, &[obj(agrus)]).unwrap();
    // The trigger resolves: two copies, P0's, above P1's original.
    t.resolve();
    assert_eq!(stack_controllers(&t), vec![P1, P0, P0]);
    t.resolve();
    t.resolve();
    assert_eq!(damage_on(&t, agrus), 0, "the original hasn't resolved yet");
    t.resolve_all();
    assert_eq!(damage_on(&t, agrus), 1);
}

#[test]
fn agrus_kos_triggers_on_any_players_ability_targeting_only_it() {
    cr!("603.2", "115.10");
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "The last ability triggers whenever Agrus Kos, Eternal Soldier becomes the target of an ability that targets it and no other object or player. It doesn't matter who controls the ability."
    );
    // P0's own Pyromancer: it triggers.
    let mut t = TestGame::new(2);
    let agrus = t.battlefield(P0, AGRUS);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    t.activate(P0, pyro, 0, &[obj(agrus)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the ability and Agrus Kos's trigger");
    t.resolve_all();
    // Inferno Titan's enters ability dividing among Agrus Kos and the Bears: no trigger.
    let mut t = TestGame::new(2);
    let agrus = t.battlefield(P1, AGRUS);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(agrus), obj(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.enter(P0, "Inferno Titan");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "only the Titan's ability");
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Yasova Dragonclaw, Wick, Mizzix
// ---------------------------------------------------------------------------------------

#[test]
fn yasova_checks_powers_on_the_stack_and_on_resolution_only() {
    cr!("608.2b", "115.1");
    ruling!(
        "Yasova Dragonclaw",
        "The powers of the target creature and Yasova Dragonclaw are checked as you put the ability on the stack and again as the ability resolves. Once the ability resolves, it doesn’t matter what happens to either creature’s power."
    );
    supported("Yasova Dragonclaw");
    for pump_first in [true, false] {
        let mut t = TestGame::new(2);
        let y = t.battlefield(P0, "Yasova Dragonclaw");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Island", 3);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(bears)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        if pump_first {
            // In response, the Bears become 5/5: no longer less than Yasova's 4.
            t.g.turn.priority = Some(P1);
            cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
            t.resolve();
        }
        t.resolve_all();
        let stolen = controller(&mut t, bears) == P0;
        assert_eq!(stolen, !pump_first);
        if !pump_first {
            // Afterward, pumping the Bears doesn't undo it.
            t.g.turn.priority = Some(P1);
            cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
            t.resolve_all();
            assert_eq!(controller(&mut t, bears), P0);
        }
        let _ = y;
    }
}

#[test]
fn wick_uses_the_sacrificed_snails_last_known_power() {
    cr!("608.2h", "118.3");
    ruling!(
        "Wick, the Whorled Mind",
        "Use the power of the sacrificed Snail as it last existed on the battlefield to determine how much damage to deal and how many cards to draw. While Wick may have given the Snail up, Wick won’t let you down."
    );
    supported("Wick, the Whorled Mind");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let wick = enter(&mut t, P0, "Wick, the Whorled Mind");
    t.resolve_all();
    let snails = crate::r_s05_common::tokens_with_subtype(&t, P0, "Snail");
    assert_eq!(snails.len(), 1);
    let snail = snails[0];
    cast_new(&mut t, P0, "Giant Growth", &[obj(snail)]);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, snail), (4, 4));
    tap_lands(&mut t, P0);
    fill_pool(&mut t, P0, 15);
    t.answer_choose(P0, &[obj(snail)]);
    let hand = t.hand_size(P0);
    t.activate(P0, wick, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn mizzix_pays_once_for_one_copy() {
    cr!("707.10", "118.3");
    ruling!(
        "Mizzix, Replica Rider",
        "You may only pay the cost once, as the triggered ability resolves. You can't pay it more than once to get additional copies."
    );
    supported("Mizzix, Replica Rider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mizzix, Replica Rider");
    let tt = t.graveyard(P0, "Think Twice");
    t.advance_to(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 15);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.cast(P0, tt)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Flashback,
        ))
        .go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    let pays = count_asked(&t, from, |d| matches!(d, Decision::YesNo { .. }));
    assert_eq!(pays, 1);
    assert_eq!(t.hand_size(P0), hand + 2, "the spell and one copy");
}

// ---------------------------------------------------------------------------------------
// Combat timing: Najeela, Acidic Dagger, Feral Encounter
// ---------------------------------------------------------------------------------------

/// Advances from the end of combat to the next step where a player gets priority.
fn step_after_end_of_combat(t: &mut TestGame) -> Step {
    t.advance_to(P0, Step::EndOfCombat);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step != Step::EndOfCombat && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.g.turn.step
}

#[test]
fn najeela_extra_combat_has_no_main_phase_before_it() {
    cr!("500.8", "506.1", "702.6a");
    ruling!(
        "Najeela, the Blade-Blossom",
        "There is no main phase before the additional combat phase. This means that, for example, you can't activate an equip ability between combats."
    );
    supported("Najeela, the Blade-Blossom");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Najeela, the Blade-Blossom");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    tap_lands(&mut t, P0);
    fill_pool(&mut t, P0, 15);
    t.activate(P0, n, 0, &[]).unwrap();
    t.resolve_all();
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    assert_eq!(step_after_end_of_combat(&mut t), Step::BeginningOfCombat);
    fill_pool(&mut t, P0, 15);
    assert!(
        !can_activate(&mut t, P0, sword),
        "no equipping between combats"
    );
}

#[test]
fn acidic_dagger_only_before_blockers_of_the_first_combat() {
    cr!("506.8b", "506.8d");
    ruling!(
        "Acidic Dagger",
        "If a turn has multiple combat phases, the ability can only be activated before the beginning of the Declare Blockers Step of the first combat phase in that turn."
    );
    supported("Acidic Dagger");
    supported("Savage Beating");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dagger = t.battlefield(P0, "Acidic Dagger");
    t.lands(P0, "Wastes", 4);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_activate(&mut t, P0, dagger));
    crate::r_s03_common::to_blockers(&mut t, &[(bears, pl(P1))], &[]);
    assert!(!can_activate(&mut t, P0, dagger));
    lands_for_cost(&mut t, P0, "Savage Beating");
    let beating = t.hand(P0, "Savage Beating");
    t.cast(P0, beating).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(step_after_end_of_combat(&mut t), Step::BeginningOfCombat);
    assert!(
        !can_activate(&mut t, P0, dagger),
        "second combat, before blockers"
    );
}

#[test]
fn feral_encounter_needs_no_targets_until_its_delayed_trigger() {
    cr!("603.7c", "601.2c");
    ruling!(
        "Feral Encounter",
        "Casting Feral Encounter doesn't require any targets. As you put the delayed triggered ability on the stack at the beginning of the next combat phase this turn, you choose the target creature you control and up to one target creature you don't control."
    );
    supported("Feral Encounter");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t
        .g
        .permanents()
        .all(|o| o.controller != P0 || !o.is_creature()));
    t.answer_choose(P0, &[]);
    cast_new(&mut t, P0, "Feral Encounter", &[]);
    t.resolve_all();
    // A creature arrives after it resolved; it's chosen at the beginning of combat.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
}

// ---------------------------------------------------------------------------------------
// Deadshot, Ram Through
// ---------------------------------------------------------------------------------------

#[test]
fn deadshot_may_target_an_already_tapped_creature() {
    cr!("701.26a", "115.1");
    ruling!(
        "Deadshot",
        "Deadshot can target a creature which is already tapped as its first target, and it will still damage the second target. This is because it taps that creature as an effect, not as a cost."
    );
    supported("Deadshot");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Deadshot", &[obj(giant), obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn ram_through_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "702.19c");
    ruling!(
        "Ram Through",
        "If either creature is an illegal target as Ram Through tries to resolve, the creature you control won’t deal damage to any creature or player."
    );
    supported("Ram Through");
    for mine_leaves in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        grant_kw(&mut t, wurm, KeywordKind::Trample);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.advance_to(P0, Step::PrecombatMain);
        cast_new(&mut t, P0, "Ram Through", &[obj(wurm), obj(bears)]);
        crate::r_p046_common::bounce(&mut t, if mine_leaves { wurm } else { bears });
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "mine leaves {mine_leaves}");
        if mine_leaves {
            assert!(t.on_battlefield(bears));
            assert_eq!(damage_on(&t, bears), 0);
        }
    }
}
