//! Rulings batch P011 — copies, triggers and multiplayer choices: Psychic Rebuttal's copy
//! (CR 707.10, 707.10c), Pyromancer Ascension's triggers above the spell (CR 603.3), Fear
//! of Burning Alive's trigger per opponent (CR 603.2c), Olórin's Searing Light's choices in
//! APNAP order (CR 101.4), costs paid before anyone can respond (CR 602.2b), and casting
//! with flash from the graveyard (CR 601.3d).

use crate::r_p009_common::{dmg, obj, pl};
use crate::r_p011_common::*;
use crate::r_p012_common::{cast_card, fill_graveyard, trigger_on_top};
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{supported, tokens, watch};
use crate::r_s25_common::{change_copy_targets, spell_copies};
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Psychic Rebuttal -------------------------------------------------------------------------

#[test]
fn psychic_rebuttals_copy_isnt_cast_and_resolves_later() {
    cr!("707.10");
    ruling!(
        "Psychic Rebuttal",
        "If Psychic Rebuttal creates a copy of the spell, you control the copy. That copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger. The copy will then resolve like a normal spell, after players get a chance to cast spells and activate abilities."
    );
    supported("Psychic Rebuttal");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    fill_graveyard(&mut t, P0, "Opt", 2);
    let spike = cast_card(&mut t, P1, "Lava Spike", &[pl(P0)]);
    cast_card(&mut t, P0, "Psychic Rebuttal", &[obj(spike)]);
    // Young Pyromancer's trigger for Psychic Rebuttal.
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(pl(P1))]);
    t.resolve();
    // Rebuttal resolved: the original is countered, the copy waits on the stack.
    assert_eq!(t.zone(spike), Zone::Graveyard(P1));
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P0);
    assert_eq!(t.stack_len(), 1, "no cast trigger for the copy");
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn psychic_rebuttals_copy_keeps_the_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Psychic Rebuttal",
        "If the copied spell is modal (that is, it says “Choose one –” or the like), the copy will have the same mode. You can’t choose a different one."
    );
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Opt", 2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Plains", 1);
    let charm = t.hand(P1, "Boros Charm");
    let charm = t.cast(P1, charm).modes(&[0]).target(pl(P0)).go();
    t.settle();
    cast_card(&mut t, P0, "Psychic Rebuttal", &[obj(charm)]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(pl(P1))]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(modes_on_stack(&t, copies[0]), vec![0]);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseModes { .. })));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 20);
}

// --- Pyromancer Ascension -------------------------------------------------------------------

#[test]
fn pyromancer_ascension_counter_even_if_things_change() {
    cr!("603.3", "113.7a");
    ruling!(
        "Pyromancer Ascension",
        "If Pyromancer Ascension’s first ability triggers, you’ll put a quest counter on it even if the card in your graveyard leaves your graveyard by the time the ability resolves or if the spell you cast is countered."
    );
    supported("Pyromancer Ascension");
    supported("Cancel");
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Pyromancer Ascension");
    let old = t.graveyard(P0, "Lightning Bolt");
    let bolt = cast_card(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    assert!(trigger_on_top(&t));
    crate::r_p127_common::exile_now(&mut t, old);
    cast_card(&mut t, P1, "Cancel", &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0), "countered");
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(asc, "quest"), 1);
}

#[test]
fn pyromancer_ascension_copy_resolves_first_even_if_the_original_is_countered() {
    cr!("603.3", "707.10", "405.5");
    ruling!(
        "Pyromancer Ascension",
        "If either of Pyromancer Ascension’s abilities triggers, it will go on the stack on top of the spell that caused it to trigger. The ability will resolve first. If it’s Pyromancer Ascension’s second ability that triggered, the copy it creates will also resolve before the original spell. The copy is created even if the original spell has been countered."
    );
    // The copy resolves before the original.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Pyromancer Ascension");
    t.g.add_counters(obj(asc), "quest", 2, None);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = cast_card(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    assert!(trigger_on_top(&t));
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.stack.last(), Some(&copies[0]), "the copy is above the original");
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.zone(bolt), Zone::Stack, "the original hasn't resolved yet");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The original is countered in response to the trigger: the copy is still created.
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Pyromancer Ascension");
    t.g.add_counters(obj(asc), "quest", 2, None);
    let bolt = cast_card(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    cast_card(&mut t, P1, "Cancel", &[obj(bolt)]);
    t.resolve();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// --- Fear of Burning Alive ------------------------------------------------------------------

#[test]
fn fear_of_burning_alive_triggers_for_each_opponent_dealt_damage() {
    cr!("603.2c");
    ruling!(
        "Fear of Burning Alive",
        "If a source you control deals noncombat damage to multiple opponents simultaneously while there are four or more card types among cards in your graveyard, Fear of Burning Alive's last ability will trigger once for each opponent dealt noncombat damage this way."
    );
    ruling!(
        "Fear of Burning Alive",
        "Fear of Burning Alive's last ability deals damage equal to the amount of noncombat damage dealt by the source that dealt damage to an opponent."
    );
    supported("Fear of Burning Alive");
    supported("Sizzle");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Fear of Burning Alive");
    delirium(&mut t, P0);
    let w1 = t.battlefield(P1, "Colossal Dreadmaw");
    let w2 = t.battlefield(P2, "Colossal Dreadmaw");
    t.answer_targets(P0, &[obj(w1)]);
    t.answer_targets(P0, &[obj(w2)]);
    t.answer_targets(P0, &[obj(w1)]);
    t.answer_targets(P0, &[obj(w2)]);
    cast_card(&mut t, P0, "Sizzle", &[]);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "noncombat"), 2);
    t.resolve_all();
    assert_eq!(dmg(&t, w1), 3);
    assert_eq!(dmg(&t, w2), 3);
    // Lava Spike (3) at P1 alone: one trigger, 3 damage; without delirium none.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Fear of Burning Alive");
    let w1 = t.battlefield(P1, "Colossal Dreadmaw");
    cast_card(&mut t, P0, "Lava Spike", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(dmg(&t, w1), 0, "no delirium");
}

// --- Olórin's Searing Light: choices in turn order ------------------------------------------

#[test]
fn olorins_searing_light_opponents_choose_in_turn_order() {
    cr!("101.4", "608.2e");
    ruling!(
        "Olórin's Searing Light",
        "First, the active player (or, if it's not an opponent's turn, the next opponent in turn order) chooses which creature they'll exile, then each other opponent in turn order does the same, knowing choices made before their choice. Then all those creatures are exiled simultaneously."
    );
    ruling!(
        "Olórin's Searing Light",
        "If a player controls two or more creatures tied for the greatest power among creatures they control, that player chooses one of them to exile."
    );
    supported("Olórin's Searing Light");
    // (active player, expected order of choosers)
    for (active, order) in [(P0, [P1, P2, P3]), (P2, [P2, P3, P1])] {
        let mut t = TestGame::new(4);
        let mut giants = Vec::new();
        for p in [P1, P2, P3] {
            t.battlefield(p, "Grizzly Bears");
            let a = t.battlefield(p, "Hill Giant");
            let b = t.battlefield(p, "Hill Giant");
            giants.push((p, a, b));
            t.answer_choose(p, &[obj(b)]);
        }
        // When each player chooses: how many of the chosen Giants are already gone.
        let seen = [P1, P2, P3].map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::ChooseEntities { .. }),
                |g| g.exile.len(),
            )
        });
        if active != P0 {
            t.set_step(active, Step::PrecombatMain);
        }
        let from = t.asked().len();
        cast_card(&mut t, P0, "Olórin's Searing Light", &[]);
        t.resolve_all();
        let choosers: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(choosers, order.to_vec(), "active {active:?}");
        // Only the tied Giants were offered.
        for (p, d) in &t.asked()[from..] {
            if let Decision::ChooseEntities { candidates, .. } = d {
                let (_, a, b) = giants.iter().find(|(q, ..)| q == p).unwrap();
                assert_eq!(candidates.len(), 2, "{p:?}");
                assert!(candidates.contains(&obj(*a)) && candidates.contains(&obj(*b)));
            }
        }
        // Nothing was exiled until everyone had chosen; then each chosen Giant was.
        for s in &seen {
            assert_eq!(*s.lock().unwrap(), vec![0], "active {active:?}");
        }
        for (_, a, b) in &giants {
            assert!(t.on_battlefield(*a));
            assert!(!t.on_battlefield(*b));
        }
    }
}

// --- Costs paid before responses ------------------------------------------------------------

#[test]
fn scuttletide_discard_is_part_of_the_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Scuttletide",
        "Discarding a card is part of the cost of Scuttletide's first ability. There is no time for players to respond to the ability before the discarded card is in the graveyard."
    );
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Scuttletide");
    // A Crab already there (discarding a Forest: artifact, creature, land), to see
    // delirium apply at once.
    t.graveyard(P0, "Ornithopter");
    t.lands(P0, "Island", 2);
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    t.activate(P0, st, 0, &[]).unwrap();
    t.resolve_all();
    let crab = tokens(&t, P0)[0];
    assert_eq!(t.pt(crab), (0, 3));
    let opt = t.hand(P0, "Opt");
    t.answer_choose(P0, &[obj(opt)]);
    let from = t.asked().len();
    t.activate(P0, st, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1, "the ability is on the stack");
    assert!(t.in_graveyard(P0, "Opt"), "already discarded");
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p == P0 || !matches!(d, Decision::Priority { .. })));
    t.g.recompute();
    assert_eq!(t.pt(crab), (1, 4), "delirium with the discarded instant");
}

// --- Casting with flash from the graveyard --------------------------------------------------

#[test]
fn graveyard_shift_counts_itself_before_it_is_cast() {
    cr!("601.3d", "702.34a");
    ruling!(
        "Graveyard Shift",
        "If an effect allows you to cast Graveyard Shift from your graveyard, you may cast it any time you could cast an instant if you have five or more mana values among cards in your graveyard immediately before you begin to cast it, even if moving it to the stack would reduce that number to four."
    );
    supported("Snapcaster Mage");
    // Values 0, 1, 2, 3 and Graveyard Shift's 5; on P1's turn.
    for (divination, ok) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let mut names = vec!["Ornithopter", "Llanowar Elves", "Grizzly Bears"];
        if divination {
            names.push("Divination");
        }
        for n in names {
            t.graveyard(P0, n);
        }
        let gs = t.graveyard(P0, "Graveyard Shift");
        t.set_step(P1, Step::PrecombatMain);
        t.answer_targets(P0, &[obj(gs)]);
        t.enter(P0, "Snapcaster Mage");
        t.resolve_all();
        t.lands(P0, "Swamp", 5);
        let bears = t.g.find_in_zone(Zone::Graveyard(P0), "Grizzly Bears")[0];
        let r = t
            .cast(P0, gs)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .target(bears)
            .try_go();
        assert_eq!(r.is_ok(), ok, "Divination {divination}: {r:?}");
        if ok {
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
            assert!(t.in_exile("Graveyard Shift"));
        }
    }
}
