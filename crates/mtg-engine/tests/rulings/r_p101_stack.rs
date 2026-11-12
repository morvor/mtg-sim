//! Rulings batch P101 — spells on the stack: gaining control of one (Commandeer, CR
//! 110.2, 115.7, 608.2b, 608.2n, 702.27a), putting one into its owner's library (Subtlety:
//! not countering it, CR 701.6a), copies keeping X and the division of damage (CR 707.10,
//! 601.2d), and abilities that trigger on casting resolving first (Mizzix, Chalice of the
//! Void: CR 603.3, 601.2i, 603.4).

use crate::r_p003_common::divide;
use crate::r_p101_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Commandeer (paid from a pool) on `spell`, answering `new` to choosing new
/// targets (one target answer per slot); Commandeer resolves.
fn commandeer(t: &mut TestGame, spell: ObjectId, new: Option<&[Entity]>) {
    mana(t, P0, "5UU");
    let c = t.hand(P0, "Commandeer");
    t.g.turn.priority = Some(P0);
    t.cast(P0, c).target(obj(spell)).go();
    t.answer_yes(P0, new.is_some());
    for e in new.unwrap_or(&[]) {
        t.answer_targets(P0, &[*e]);
    }
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
}

#[test]
fn commandeer_makes_you_the_spells_you_for_targets_and_effects() {
    cr!("110.2", "115.7", "608.2b", "109.5");
    ruling!(
        "Commandeer",
        "After Commandeer resolves, you control the targeted spell. Any instance of \"you\" in that spell's text now refers to you, \"an opponent\" refers to one of your opponents, and so on."
    );
    ruling!(
        "Commandeer",
        "If you gain control of an instant or sorcery spell with Commandeer, it will still be put into its owner's graveyard when it resolves."
    );
    // P1's Divination: P0 draws two; it goes to P1's graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = cast_new(&mut t, P1, "Divination", &[]);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    commandeer(&mut t, div, None);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 2, h1));
    assert!(t.in_graveyard(P1, "Divination"));
    assert!(!t.in_graveyard(P0, "Divination"));
    // P1's Ranger's Guile ("Target creature you control gets +1/+1 and gains hexproof")
    // on P1's Grizzly Bears: under P0's control, the Bears is no longer a creature "you"
    // control. Kept, the spell doesn't resolve; P0 may change it to P0's Hill Giant.
    for change in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        let guile = cast_new(&mut t, P1, "Ranger's Guile", &[obj(bears)]);
        let new = [obj(giant)];
        commandeer(&mut t, guile, change.then_some(&new[..]));
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Ranger's Guile"));
        if change {
            assert_eq!(t.pt(giant), (4, 4));
            assert_eq!(t.pt(bears), (2, 2));
        } else {
            assert!(!resolved(&t, guile), "an illegal target");
            assert_eq!(t.pt(bears), (2, 2));
            assert_eq!(t.pt(giant), (3, 3));
        }
    }
}

#[test]
fn commandeer_reevaluates_target_restrictions_for_you_before_new_targets() {
    cr!("115.7", "608.2b", "608.2c");
    ruling!(
        "Commandeer",
        "The change of control happens before new targets are chosen, so any targeting restrictions such as \"target opponent\" or \"target creature you control\" are now made in reference to you, not the spell's original controller."
    );
    // P1's Ranger's Guile on P1's Bears: P0's new target must be a creature P0 controls;
    // P1's other creature isn't offered.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Raging Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let guile = cast_new(&mut t, P1, "Ranger's Guile", &[obj(bears)]);
    let from = t.asked().len();
    commandeer(&mut t, guile, Some(&[obj(giant)]));
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    let last = cands.last().expect("asked for a new target");
    assert!(last.contains(&obj(giant)));
    assert!(!last.contains(&obj(other)));
    assert!(!last.contains(&obj(bears)));
}

#[test]
fn commandeer_on_buyback_and_storm_spells() {
    cr!("702.27a", "702.40a", "707.10", "110.2");
    ruling!(
        "Commandeer",
        "If you Commandeer a spell for which buyback has been paid, the card returns to its owner's hand."
    );
    ruling!(
        "Commandeer",
        "If the targeted spell has a triggered ability that copies it (for example, replicate or storm), the copies will be controlled by the player who cast that spell."
    );
    supported("Whispers of the Muse");
    // P1's Whispers of the Muse with buyback: P0 draws, the card returns to P1's hand.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let w = t.hand(P1, "Whispers of the Muse");
    mana(&mut t, P1, "6U");
    t.answer(
        P1,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, w).go();
    let h0 = t.hand_size(P0);
    commandeer(&mut t, spell, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert!(t.in_hand(P1, "Whispers of the Muse"), "{}", t.dump_log());
    // P1's Grapeshot (storm count 1): P0 Commandeers it above the storm trigger; the copy
    // is P1's.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shot = cast_new(&mut t, P1, "Grapeshot", &[pl(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "Grapeshot and its storm trigger");
    commandeer(&mut t, shot, Some(&[pl(P1)]));
    // The storm trigger resolves: P1 controls the copy (P1 chooses its target: Bears).
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(bears)]);
    t.resolve();
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj_now(copies[0]).controller, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "P0's Grapeshot");
    assert_eq!(dmg(&t, bears), 1, "P1's copy");
}

/// Answers a top-or-bottom choice with the option naming `word`.
fn choose_position(d: &Decision, word: &str) -> Option<mtg_engine::decision::Answer> {
    match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.to_lowercase().contains(word))
            .map(mtg_engine::decision::Answer::Index),
        _ => None,
    }
}
fn choose_top(_: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    choose_position(d, "top")
}
fn choose_bottom(_: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    choose_position(d, "bottom")
}

#[test]
fn subtlety_puts_a_creature_spell_into_its_library_by_its_owners_choice() {
    cr!("701.6a", "115.1a", "401.4", "702.12b");
    ruling!(
        "Subtlety",
        "A spell that's put into its owner's library doesn't resolve, but it isn't countered. This will work on spells that say they can't be countered."
    );
    ruling!(
        "Subtlety",
        "Subtlety's triggered ability targets a spell on the stack. It can't target creatures or planeswalkers on the battlefield."
    );
    ruling!(
        "Subtlety",
        "The owner of the spell is the one who chooses whether it goes on the top or bottom of their library. All players will know this information."
    );
    supported("Subtlety");
    // P1 casts Carnage Tyrant (can't be countered); P0 flashes in Subtlety.
    for top in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let tyrant = cast_new(&mut t, P1, "Carnage Tyrant", &[]);
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(tyrant)]);
        crate::r_s03_common::respond(&mut t, P1, if top { choose_top } else { choose_bottom });
        let s = cast(&mut t, P0, "Subtlety", "2UU", &[]);
        t.resolve();
        assert!(t.on_battlefield(s));
        let cands = target_candidates(&t, P0, from);
        assert!(cands
            .iter()
            .all(|c| !c.contains(&obj(bears)) && !c.contains(&obj(s))));
        assert!(cands.iter().any(|c| c.contains(&obj(tyrant))));
        t.resolve_all();
        let lib = &t.g.player(P1).library;
        let at = if top { *lib.last().unwrap() } else { lib[0] };
        assert_eq!(t.obj(at).chars.name, "Carnage Tyrant", "top = {top}");
        assert!(
            t.asked()[from..]
                .iter()
                .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. })),
            "the owner chooses: {:?}",
            t.asked()[from..].to_vec()
        );
        assert!(!t
            .g
            .turn_events
            .iter()
            .any(|e| matches!(e, mtg_engine::events::Event::Countered { .. })));
    }
}

#[test]
fn flare_of_duplications_copy_keeps_x() {
    cr!("707.10", "107.3", "707.2");
    ruling!(
        "Flare of Duplication",
        "If the spell that's copied has an X whose value was determined as it was cast (like Kozilek's Command does), the copy will have the same value of X."
    );
    supported("Flare of Duplication");
    let mut t = TestGame::new(2);
    let blaze = cast_x(&mut t, P0, "Blaze", 3, &[pl(P1)]);
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    cast(&mut t, P0, "Flare of Duplication", "1RR", &[obj(blaze)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn deflecting_swat_keeps_the_division_of_damage_and_counters() {
    cr!("601.2d", "115.7", "608.2c");
    ruling!(
        "Deflecting Swat",
        "If the target spell has damage divided as it was cast (like Mythos of Vadrok), the division can't be changed although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    // P1's Forked Bolt (2 damage divided among one or two targets): 1 to P0 and 1 to P0's
    // Grizzly Bears; P0 changes the targets to P1 and P1's Raging Goblin.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.set_step(P1, Step::PrecombatMain);
    divide(&mut t, P1, &[1, 1]);
    t.answer(
        P1,
        DecisionKind::Targets,
        Answer::Entities(vec![pl(P0), obj(bears)]),
    );
    let bolt = cast_new(&mut t, P1, "Forked Bolt", &[]);
    cast(&mut t, P0, "Deflecting Swat", "2R", &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(goblin)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19), "{}", t.dump_log());
    assert!(!t.on_battlefield(goblin));
    assert_eq!(dmg(&t, bears), 0);
    // Blessings of Nature (distribute four +1/+1 counters): 3 on Grizzly Bears, 1 on
    // Raging Goblin, both P1's; P0 changes the first target to P0's Hill Giant: 3 there.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    divide(&mut t, P1, &[3, 1]);
    t.answer(
        P1,
        DecisionKind::Targets,
        Answer::Entities(vec![obj(bears), obj(goblin)]),
    );
    let bless = cast_new(&mut t, P1, "Blessings of Nature", &[]);
    cast(&mut t, P0, "Deflecting Swat", "2R", &[obj(bless)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(goblin)]);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
    assert_eq!(t.counters(goblin, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn mizzixs_trigger_comes_after_the_spell_is_cast_and_resolves_before_it() {
    cr!("601.2i", "603.3", "601.2f", "122.1f");
    ruling!(
        "Mizzix of the Izmagnus",
        "You finish casting a spell, including paying its costs, before Mizzix's first ability triggers. The ability that gives you the experience counter will resolve before the spell that caused it to trigger."
    );
    supported("Mizzix of the Izmagnus");
    // With no experience counters, Divination (3) costs all of {2}{U}; then the trigger
    // resolves first: one experience counter while Divination is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mizzix of the Izmagnus");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    let spell = t.cast(P0, div).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.resolve();
    assert_eq!(
        t.g.player(P0)
            .counters
            .get("experience")
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
}

#[test]
fn chalice_of_the_void_compares_counters_as_the_spell_is_cast() {
    cr!("603.2", "603.4", "608.2c");
    ruling!(
        "Chalice of the Void",
        "The number of charge counters on Chalice of the Void matters only at the time the spell is cast. Changing the number of charge counters on Chalice of the Void after a spell has been cast won't change whether the ability triggers or counters the spell."
    );
    // One counter; P1 casts Opt (1); a second counter is added in response: still
    // countered.
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    put(&mut t, chalice, "charge", 1);
    t.set_step(P1, Step::PrecombatMain);
    let opt = cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    put(&mut t, chalice, "charge", 1);
    t.resolve_all();
    assert_eq!(t.zone(opt), Zone::Graveyard(P1));
    assert!(!resolved(&t, opt));
    // Two counters; P1 casts Opt: no trigger; one counter is removed: it resolves.
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    put(&mut t, chalice, "charge", 2);
    t.set_step(P1, Step::PrecombatMain);
    let opt = cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p151_common::remove_counters(&mut t, chalice, "charge", 1);
    t.resolve_all();
    assert!(resolved(&t, opt));
}
