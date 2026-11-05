//! Rulings batch P055 — additional combat phases (CR 500.8, 505.1a): Relentless Assault,
//! Waves of Aggression, Seize the Day, Finest Hour and Hellkite Charger; attack
//! requirements per turn and per combat (CR 508.1d); and abilities that count as they
//! resolve (Krenko, Marrow-Gnawer, Kyren Flamewright) or need their creature to survive
//! (Necropolis Regent).

use crate::r_p054_common::{cast_how, o, pool};
use crate::r_p079_common::back_face;
use crate::r_s01_common::{custom_card, supported, tokens, triggers_on_stack};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The number of times `step` has begun this turn.
fn count(t: &TestGame, step: Step) -> usize {
    t.g.turn.step_log.iter().filter(|s| **s == step).count()
}

/// Advances to the `n`th occurrence of `step` in the current turn (the active player
/// has priority), panicking if the turn ends first.
fn to_nth(t: &mut TestGame, step: Step, n: usize) {
    let turn = t.g.turn.number;
    let ap = t.g.turn.active;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.number != turn
            || (g.turn.step == step
                && g.turn.stage == Stage::Priority
                && g.turn.priority == Some(ap)
                && g.turn.step_log.iter().filter(|s| **s == step).count() == n)
    });
    assert!(ok && t.g.turn.number == turn, "no {step:?} #{n} this turn");
}

/// Queues `p`'s next attack declaration.
fn declare(t: &mut TestGame, p: PlayerId, attackers: &[ObjectId], at: PlayerId) {
    t.answer(
        p,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.iter().map(|a| (*a, Entity::Player(at))).collect()),
    );
}

/// Whether `id` attacked in the declaration of the `n`th combat (counting from 1) this
/// turn: whether it is attacking during that combat's end of combat step.
fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attacker(t.g.current(id)).is_some())
}

/// `p` casts `name` (from a full pool) with `targets`, via Leyline of Anticipation when
/// it isn't a main phase of `p`'s turn.
fn cast_sorcery(t: &mut TestGame, p: PlayerId, name: &str, targets: &[&[Entity]]) -> ObjectId {
    if t.named_on_battlefield("Leyline of Anticipation")
        .iter()
        .all(|l| t.obj_now(*l).controller != p)
    {
        t.battlefield(p, "Leyline of Anticipation");
    }
    cast_how(t, p, name, CastMethod::Normal, targets, None, None)
}

#[test]
fn relentless_assault_attack_triggers_and_requirements_in_each_combat() {
    cr!("505.1a", "500.8", "508.1d", "508.1m");
    ruling!(
        "Relentless Assault",
        "Abilities that trigger when a creature attacks, blocks, or is blocked will trigger during each combat that turn."
    );
    // Its first two sentences ("each turn") are outdated: the Ixalan Oracle changes
    // (2017-09-28) made every such card attack "each combat" (exempted in
    // docs/rulings-exemptions/rulings-P055.tsv; logged in docs/RULING_DISCREPANCIES.md).
    ruling!(
        "Relentless Assault",
        "A creature that attacks each turn if able must attack only once that turn. It is not forced into each attack that turn. However, a creature that attacks each combat if able must attack again if it can."
    );
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    // "Whenever Hero of Bladehold attacks, create two 1/1 white Soldier creature tokens
    // that are tapped and attacking."
    let hero = t.battlefield(P0, "Hero of Bladehold");
    // "attacks each combat if able"
    let berserkers = t.battlefield(P0, "Berserkers of Blood Ridge");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[hero, berserkers], P1);
    // Second combat: P0 declares no attackers; the engine replaces the illegal
    // declaration with one meeting the requirements.
    declare(&mut t, P0, &[], P1);
    to_nth(&mut t, Step::DeclareAttackers, 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    for c in [hero, berserkers] {
        assert!(attacking(&t, c));
    }
    t.advance_to(P0, Step::PostcombatMain);
    cast_sorcery(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    for c in [hero, berserkers] {
        assert!(!t.obj_now(c).tapped, "untapped");
    }
    to_nth(&mut t, Step::DeclareAttackers, 2);
    t.resolve_all();
    // Berserkers attacks again although P0 declared no attackers; Hero didn't attack, so
    // no new tokens.
    assert!(attacking(&t, berserkers));
    assert!(!attacking(&t, hero));
    assert_eq!(tokens(&t, P0).len(), 2);
    // Hero attacking in an additional combat triggers again.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[hero], P1);
    declare(&mut t, P0, &[hero], P1);
    to_nth(&mut t, Step::DeclareAttackers, 1);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    cast_sorcery(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    to_nth(&mut t, Step::DeclareAttackers, 2);
    t.resolve_all();
    assert!(attacking(&t, hero));
    assert_eq!(tokens(&t, P0).len(), 4);
}

#[test]
fn relentless_assault_on_an_opponents_turn_gives_them_the_combat() {
    cr!("505.1a", "500.8", "506.2");
    ruling!(
        "Relentless Assault",
        "If you manage to cast this during a main phase of your opponent's turn, that opponent's creatures will untap and that opponent will be able to attack again. It will not allow you to attack during their turn."
    );
    ruling!(
        "Seize the Day",
        "If you somehow cast Seize the Day during an opponent's main phase, that opponent attacks during the resulting combat phase, since it's still that player's turn. If you somehow cast it not during a main phase at all, all that happens is you untap the target creature."
    );
    for spell in ["Relentless Assault", "Seize the Day"] {
        supported(spell);
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P0, "Grizzly Bears");
        let mine = t.battlefield(P1, "Hill Giant");
        t.set_step(P0, Step::BeginningOfCombat);
        declare(&mut t, P0, &[theirs], P1);
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.life(P1), 18);
        assert!(t.obj_now(theirs).tapped);
        // P1 casts it (as though it had flash) in P0's postcombat main phase.
        let tgt = [o(theirs)];
        let targets: Vec<&[Entity]> = if spell == "Seize the Day" {
            vec![&tgt]
        } else {
            vec![]
        };
        let from = t.asked().len();
        cast_sorcery(&mut t, P1, spell, &targets);
        t.resolve_all();
        assert!(!t.obj_now(theirs).tapped, "{spell}");
        declare(&mut t, P0, &[theirs], P1);
        declare(&mut t, P1, &[mine], P0);
        to_nth(&mut t, Step::EndOfCombat, 2);
        // P0 attacked again; P1 was never asked to declare attackers.
        assert_eq!(t.life(P1), 16, "{spell}");
        assert_eq!(t.life(P0), 20, "{spell}");
        assert!(t.asked()[from..]
            .iter()
            .all(|(p, d)| !(matches!(d, Decision::DeclareAttackers { .. }) && *p == P1)));
        to_nth(&mut t, Step::PostcombatMain, 2);
    }
    // Seize the Day not in a main phase (P0's end of combat step): it only untaps.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[bear], P1);
    t.advance_to(P0, Step::EndOfCombat);
    cast_sorcery(&mut t, P0, "Seize the Day", &[&[o(bear)]]);
    t.resolve_all();
    assert!(!t.obj_now(bear).tapped);
    let combats = count(&t, Step::BeginningOfCombat);
    let mains = count(&t, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    assert_eq!(count(&t, Step::BeginningOfCombat), combats);
    assert_eq!(count(&t, Step::PostcombatMain), mains + 1);
}

#[test]
fn waves_of_aggression_untaps_every_attacker_this_turn_and_adds_phases_only_in_a_main_phase() {
    cr!("505.1a", "500.8", "506.4");
    ruling!(
        "Waves of Aggression",
        "Waves of Aggression untaps all creatures that attacked this turn, not just those that attacked during the most recent combat phase."
    );
    ruling!(
        "Waves of Aggression",
        "If it's somehow not a main phase when Waves of Aggression resolves, all it does is untap all creatures that attacked that turn. No new phases are created."
    );
    supported("Seize the Day");
    // First combat: the Bears attack. Seize the Day (on the Giant) adds a second, where
    // only the Giant attacks. Waves of Aggression untaps both.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let idle = t.battlefield(P0, "Craw Wurm");
    t.g.tap(idle);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[bear], P1);
    declare(&mut t, P0, &[giant], P1);
    t.advance_to(P0, Step::PostcombatMain);
    cast_sorcery(&mut t, P0, "Seize the Day", &[&[o(giant)]]);
    t.resolve_all();
    to_nth(&mut t, Step::PostcombatMain, 2);
    assert!(t.obj_now(bear).tapped && t.obj_now(giant).tapped);
    let combats = count(&t, Step::BeginningOfCombat);
    cast_sorcery(&mut t, P0, "Waves of Aggression", &[]);
    t.resolve_all();
    assert!(!t.obj_now(bear).tapped && !t.obj_now(giant).tapped);
    // A creature that didn't attack stays tapped.
    assert!(t.obj_now(idle).tapped);
    to_nth(&mut t, Step::BeginningOfCombat, combats + 1);
    // Not in a main phase (end of combat): it untaps, and no phases are added.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[bear], P1);
    t.advance_to(P0, Step::EndOfCombat);
    cast_sorcery(&mut t, P0, "Waves of Aggression", &[]);
    t.resolve_all();
    assert!(!t.obj_now(bear).tapped);
    let combats = count(&t, Step::BeginningOfCombat);
    let mains = count(&t, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    assert_eq!(count(&t, Step::BeginningOfCombat), combats);
    assert_eq!(count(&t, Step::PostcombatMain), mains + 1);
}

#[test]
fn seize_the_day_targets_and_illegal_target() {
    cr!("608.2b", "505.1a");
    ruling!(
        "Seize the Day",
        "Seize the Day can target a creature that's already untapped."
    );
    ruling!(
        "Seize the Day",
        "If the target creature becomes an illegal target for Seize the Day, the spell doesn't resolve. There isn't an additional combat phase or main phase."
    );
    for bounced in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        assert!(!t.obj_now(bear).tapped);
        cast_sorcery(&mut t, P0, "Seize the Day", &[&[o(bear)]]);
        if bounced {
            t.lands(P1, "Island", 1);
            let un = t.hand(P1, "Unsummon");
            t.cast(P1, un).target(bear).go();
            t.resolve();
        }
        t.resolve_all();
        t.advance_to(P0, Step::End);
        let combats = count(&t, Step::BeginningOfCombat);
        assert_eq!(combats, if bounced { 1 } else { 2 }, "bounced: {bounced}");
    }
}

#[test]
fn finest_hour_adds_one_combat_right_after_the_first() {
    cr!("603.4", "500.8", "702.83a");
    ruling!(
        "Finest Hour",
        "Finest Hour’s second ability has an “intervening ‘if’ clause.” The ability won’t trigger at all unless it’s the first combat phase of the turn."
    );
    ruling!(
        "Finest Hour",
        "Unlike other similar cards, Finest Hour doesn’t create a main phase in between the two combat phases. The second combat phase will begin immediately after the current one ends."
    );
    supported("Finest Hour");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Finest Hour");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[bear], P1);
    declare(&mut t, P0, &[bear], P1);
    to_nth(&mut t, Step::DeclareAttackers, 1);
    t.settle();
    // Exalted and the untap-and-combat ability.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.obj_now(bear).tapped);
    to_nth(&mut t, Step::DeclareAttackers, 2);
    t.settle();
    // Second combat: only exalted triggers.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.obj_now(bear).tapped);
    t.advance_to(P0, Step::End);
    // 3 then 4 damage (one exalted bonus, then two, until end of turn); the second
    // combat followed the first directly; no third combat.
    assert_eq!(t.life(P1), 13);
    let log = t.g.turn.step_log.clone();
    let first_end = log.iter().position(|s| *s == Step::EndOfCombat).unwrap();
    assert_eq!(log[first_end + 1], Step::BeginningOfCombat);
    assert_eq!(count(&t, Step::BeginningOfCombat), 1);
    assert_eq!(count(&t, Step::DeclareAttackers), 2);
    assert_eq!(count(&t, Step::PostcombatMain), 1);
}

/// `n` Mountains for P0.
fn mountains(t: &mut TestGame, n: usize) {
    t.lands(P0, "Mountain", n);
}

#[test]
fn hellkite_charger_pays_on_resolution_and_adds_a_combat_right_after() {
    cr!("603.5", "608.2c", "500.8");
    ruling!(
        "Hellkite Charger",
        "You decide whether to pay {5}{R}{R} as Hellkite Charger’s ability resolves."
    );
    ruling!(
        "Hellkite Charger",
        "If you pay {5}{R}{R}, the new combat phase immediately follows the current combat phase. There is no main phase in between."
    );
    ruling!(
        "Hellkite Charger",
        "Hellkite Charger’s ability may trigger multiple times in the same turn, since its own ability gives it multiple chances to attack. Each time it resolves, you may create an additional combat phase."
    );
    supported("Hellkite Charger");
    let mut t = TestGame::new(2);
    let charger = t.battlefield(P0, "Hellkite Charger");
    mountains(&mut t, 14);
    t.set_step(P0, Step::BeginningOfCombat);
    for _ in 0..3 {
        declare(&mut t, P0, &[charger], P1);
    }
    to_nth(&mut t, Step::DeclareAttackers, 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Nothing has been asked about paying while the ability waits on the stack.
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    let pay_asked = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }));
    assert!(pay_asked, "asked on resolution");
    assert!(!t.obj_now(charger).tapped);
    // Second combat: it attacks again, triggers again, and P0 pays again.
    to_nth(&mut t, Step::DeclareAttackers, 2);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    to_nth(&mut t, Step::DeclareAttackers, 3);
    t.settle();
    // Third: it triggers, but P0 has no mana left.
    t.resolve_all();
    t.advance_to(P0, Step::End);
    assert_eq!(t.life(P1), 20 - 3 * 5);
    assert_eq!(count(&t, Step::DeclareAttackers), 3);
    assert_eq!(count(&t, Step::PostcombatMain), 1);
    let log = t.g.turn.step_log.clone();
    let main = log.iter().position(|s| *s == Step::PostcombatMain).unwrap();
    assert!(log[..main].iter().all(|s| s.is_combat()));
}

#[test]
fn two_hellkite_chargers_make_two_combats_and_untap_only_as_they_resolve() {
    cr!("500.8", "506.4");
    ruling!(
        "Hellkite Charger",
        "If two Hellkite Chargers attack at the same time, both of their abilities trigger. If you pay {5}{R}{R} for each, two new combat phases will be created. However, all attacking creatures untap as those abilities resolve, not as the combat phases start. Any creature that attacks in the second combat phase will remain tapped during the third combat phase."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hellkite Charger");
    let b = t.battlefield(P0, "Hellkite Charger");
    let bear = t.battlefield(P0, "Grizzly Bears");
    mountains(&mut t, 14);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[a, b, bear], P1);
    // Second combat: only the Bears attack. Third: P0 tries the Bears again.
    declare(&mut t, P0, &[bear], P1);
    declare(&mut t, P0, &[bear], P1);
    to_nth(&mut t, Step::DeclareAttackers, 1);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    for c in [a, b, bear] {
        assert!(!t.obj_now(c).tapped);
    }
    to_nth(&mut t, Step::DeclareAttackers, 2);
    t.settle();
    assert!(attacking(&t, bear));
    assert_eq!(t.stack_len(), 0, "the Chargers didn't attack");
    let combats = count(&t, Step::BeginningOfCombat);
    to_nth(&mut t, Step::BeginningOfCombat, combats + 1);
    // Third combat: the Bears are still tapped and can't attack.
    assert!(t.obj_now(bear).tapped);
    assert!(!crate::r_s02_common::can_attack(&mut t, bear));
    assert!(crate::r_s02_common::can_attack(&mut t, a));
    to_nth(&mut t, Step::EndOfCombat, 3);
    assert!(!attacking(&t, bear));
    // 5 + 5 + 2, then 2.
    assert_eq!(t.life(P1), 20 - 12 - 2);
}

// ---------------------------------------------------------------------------
// Counting as the ability resolves
// ---------------------------------------------------------------------------

#[test]
fn krenko_and_marrow_gnawer_count_as_their_abilities_resolve() {
    cr!("608.2h", "602.2");
    ruling!(
        "Krenko, Mob Boss",
        "The ability counts each Goblin you control, including Krenko itself, not just the tokens it creates."
    );
    ruling!(
        "Marrow-Gnawer",
        "Marrow-Gnawer's ability counts the number of Rats on the battlefield when it resolves, so you don't count the sacrificed Rat."
    );
    supported("Krenko, Mob Boss");
    supported("Marrow-Gnawer");
    // Krenko alone: one token. With a Goblin Piker: two.
    for (piker, made) in [(false, 1), (true, 2)] {
        let mut t = TestGame::new(2);
        let krenko = t.battlefield(P0, "Krenko, Mob Boss");
        if piker {
            t.battlefield(P0, "Goblin Piker");
        }
        t.activate(P0, krenko, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), made, "Piker: {piker}");
    }
    // Marrow-Gnawer and two Plague Rats; one Rat is sacrificed: two Rats remain.
    let mut t = TestGame::new(2);
    let gnawer = t.battlefield(P0, "Marrow-Gnawer");
    let rat = t.battlefield(P0, "Plague Rats");
    t.battlefield(P0, "Plague Rats");
    t.answer_choose(P0, &[o(rat)]);
    t.activate(P0, gnawer, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Plague Rats"));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn kyren_flamewrights_bonus_applies_to_the_creatures_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Invasion of Mercadia // Kyren Flamewright",
        "The set of creatures affected by Kyren Flamewright’s ability is determined as the ability resolves. The Elemental tokens will get the bonuses, but creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn will not."
    );
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let kyren = back_face(&mut t, P0, "Invasion of Mercadia // Kyren Flamewright");
    t.g.objects[kyren.0 as usize].summoning_sick = false;
    assert_eq!(t.obj_now(kyren).chars.name, "Kyren Flamewright");
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Mountain", 4);
    t.hand(P0, "Island");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, kyren, 0, &[]).unwrap();
    t.resolve_all();
    let elementals = tokens(&t, P0);
    assert_eq!(elementals.len(), 2);
    for e in &elementals {
        assert_eq!(t.pt(*e), (2, 1));
        assert!(t
            .obj_now(*e)
            .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    }
    assert_eq!(t.pt(bear), (3, 2));
    // Later: a creature P0 begins to control, and a land that becomes a creature.
    let later = t.enter(P0, "Hill Giant");
    t.activate(P0, vault, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    assert_eq!(t.pt(later), (3, 3));
    assert_eq!(t.pt(vault), (2, 2));
    assert!(!t
        .obj_now(later)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn necropolis_regents_creature_dies_before_its_counters_arrive() {
    cr!("510.2", "704.5g", "603.3");
    ruling!(
        "Necropolis Regent",
        "If that creature deals combat damage to a player at the same time it's dealt lethal damage (perhaps because it has trample and was blocked), it will die before the triggered ability resolves and puts +1/+1 counters on it."
    );
    supported("Necropolis Regent");
    for blocked in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Necropolis Regent");
        // A 6/6 trampler; blocked by a 6/4 Craw Wurm it assigns 4 to the Wurm, 2 to P1.
        let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.set_step(P0, Step::BeginningOfCombat);
        declare(&mut t, P0, &[dreadmaw], P1);
        if blocked {
            t.answer(
                P1,
                DecisionKind::Blockers,
                Answer::Blockers(vec![(wurm, dreadmaw)]),
            );
            t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![4, 2]));
        }
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        if blocked {
            assert_eq!(t.life(P1), 18);
            assert!(t.in_graveyard(P0, "Colossal Dreadmaw"));
            assert_eq!(triggers_on_stack(&t, "counter"), 1);
            t.resolve_all();
            assert_eq!(t.zone(dreadmaw), Zone::Graveyard(P0));
        } else {
            assert_eq!(t.life(P1), 14);
            t.resolve_all();
            assert_eq!(t.counters(dreadmaw, "+1/+1"), 6);
        }
    }
}
