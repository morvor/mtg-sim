//! Rulings batch P174 — choices and values fixed as a spell or ability resolves (CR 608.2,
//! 608.2h): creatures tapped "any number" while resolving, reflexive triggers (CR 603.12),
//! values computed once (X), last known information (CR 608.2h, 113.7a), costs locked in
//! when paid (CR 601.2h), and convoke-like tapping in Heirloom Epic's cost.

use crate::r_p212_common::{back_face, boost};
use crate::r_s01_common::{asked_since, attack_with, give_mana_for, supported, tokens};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s24_common::tapped;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| Entity::Object(*i)).collect()
}

/// P0 attacks P1 with `attacker`; its attack trigger is on the stack.
fn attack(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
}

// --- Tapping any number of creatures as the ability resolves -------------------------------

#[test]
fn attack_triggers_tap_creatures_only_while_resolving() {
    cr!("608.2d", "302.6");
    ruling!(
        "Ledev Champion",
        "You can tap creatures this way only as Ledev Champion's triggered ability is resolving. You can't tap more creatures later in combat to give it +1/+1 again."
    );
    ruling!(
        "Siege Striker",
        "You choose how many and which creatures to tap while Siege Striker's ability is resolving."
    );
    ruling!(
        "Orphans of the Wheat",
        "You can tap any untapped creatures you control, including creatures you haven't controlled continuously since the beginning of your most recent turn, for Orphans of the Wheat's ability."
    );
    for (name, base) in [
        ("Ledev Champion", (2, 2)),
        ("Siege Striker", (1, 1)),
        ("Orphans of the Wheat", (2, 1)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        // Two creatures that came under P0's control this turn.
        let a = t.battlefield_sick(P0, "Grizzly Bears");
        let b = t.battlefield_sick(P0, "Grizzly Bears");
        attack(&mut t, c);
        assert!(!tapped(&t, a) && !tapped(&t, b), "{name}: nothing tapped yet");
        // A third one arrives in response: it can be tapped too.
        let late = t.battlefield_sick(P0, "Grizzly Bears");
        t.answer_choose(P0, &objs(&[a, b, late]));
        t.resolve_all();
        assert!(tapped(&t, a) && tapped(&t, b) && tapped(&t, late), "{name}");
        assert_eq!(t.pt(c), (base.0 + 3, base.1 + 3), "{name}");
        // Later in combat: nothing more to tap for it.
        let more = t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
        assert!(!tapped(&t, more), "{name}");
        assert_eq!(t.pt(c), (base.0 + 3, base.1 + 3), "{name}");
    }
}

#[test]
fn minas_tirith_garrison_taps_humans_while_resolving_and_draws() {
    cr!("608.2d");
    ruling!(
        "Minas Tirith Garrison",
        "You choose how many and which creatures to tap while Minas Tirith Garrison's last ability is resolving."
    );
    supported("Minas Tirith Garrison");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Minas Tirith Garrison");
    let a = t.battlefield_sick(P0, "Alaborn Trooper");
    let b = t.battlefield_sick(P0, "Femeref Scouts");
    let bear = t.battlefield_sick(P0, "Grizzly Bears");
    attack(&mut t, g);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &objs(&[a, b]));
    t.resolve_all();
    assert!(tapped(&t, a) && tapped(&t, b) && !tapped(&t, bear));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn redcap_raiders_taps_at_most_one_creature() {
    cr!("608.2d");
    ruling!(
        "Redcap Raiders",
        "While resolving the ability of Redcap Raiders, you can't tap more than one creature to get +1/+1 more than once."
    );
    supported("Redcap Raiders");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Redcap Raiders");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, r);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[a, b]));
    t.resolve_all();
    assert_eq!(
        [a, b].iter().filter(|x| tapped(&t, **x)).count(),
        1,
        "exactly one creature tapped"
    );
    assert_eq!(t.pt(r), (4, 3));
    assert!(has_kw(&t, r, KeywordKind::Trample));
}

#[test]
fn devout_invocation_taps_as_it_resolves() {
    cr!("608.2d", "608.2c");
    ruling!(
        "Devout Invocation",
        "You choose the creatures to tap as Devout Invocation resolves."
    );
    supported("Devout Invocation");
    let mut t = TestGame::new(2);
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Devout Invocation");
    let s = t.hand(P0, "Devout Invocation");
    t.cast(P0, s).go();
    assert!(!tapped(&t, a) && !tapped(&t, b));
    // A creature that enters while it's on the stack can be tapped for it; one that became
    // tapped in response can't.
    let late = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.objects[b.0 as usize].tapped = true;
    let from = t.asked().len();
    t.answer_choose(P0, &objs(&[a, late]));
    t.resolve_all();
    assert!(tapped(&t, a) && tapped(&t, late));
    let offered = asked_since(&t, from).into_iter().find_map(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => Some(candidates),
        _ => None,
    });
    assert_eq!(offered.map(|c| c.contains(&Entity::Object(b))), Some(false));
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 2);
}

#[test]
fn urge_to_feed_chooses_vampires_on_resolution() {
    cr!("608.2d", "608.2c");
    ruling!(
        "Urge to Feed",
        "You don't choose which untapped Vampire creatures you control to tap (if any) until Urge to Feed resolves."
    );
    supported("Urge to Feed");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Noble");
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Urge to Feed");
    t.cast(P0, s).target(victim).go();
    assert!(!tapped(&t, v));
    // A Vampire that arrives in response can be tapped too.
    let late = t.battlefield_sick(P0, "Highborn Vampire");
    t.answer_choose(P0, &objs(&[v, late]));
    t.resolve_all();
    assert!(!t.on_battlefield(victim));
    assert_eq!(t.counters(v, "+1/+1"), 1);
    assert_eq!(t.counters(late, "+1/+1"), 1);
}

#[test]
fn jaddi_lifestrider_may_tap_itself() {
    cr!("608.2d", "603.3");
    ruling!(
        "Jaddi Lifestrider",
        "You may tap Jaddi Lifestrider itself this way, assuming it’s still on the battlefield untapped by this time."
    );
    supported("Jaddi Lifestrider");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let j = t.enter(P0, "Jaddi Lifestrider");
    t.settle();
    assert!(!tapped(&t, j) && !tapped(&t, b), "nothing chosen as it triggers");
    t.answer_choose(P0, &objs(&[j, b]));
    t.resolve_all();
    assert!(tapped(&t, j) && tapped(&t, b));
    assert_eq!(t.life(P0), 24);
}

// --- Reflexive triggers -------------------------------------------------------------------

#[test]
fn meanders_guide_targets_with_a_reflexive_trigger() {
    cr!("603.12", "603.3d");
    ruling!(
        "Meanders Guide",
        "You don't choose a target for Meanders Guide's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you tap another untapped Merfolk you control this way."
    );
    supported("Meanders Guide");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Meanders Guide");
    let m = t.battlefield(P0, "Maritime Guard");
    let card = t.graveyard(P0, "Grizzly Bears");
    attack(&mut t, g);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[m]));
    // Resolve only the attack trigger: the reflexive trigger is put on the stack, and only
    // now is its target chosen.
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.resolve();
    assert!(tapped(&t, m));
    assert_eq!(t.stack_len(), 1, "the reflexive trigger is on the stack");
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    // Players may respond to it as normal; then it resolves.
    t.resolve_all();
    assert!(!t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn teferi_akosa_taps_creatures_as_it_resolves_then_reflexively_targets() {
    cr!("603.12", "608.2d", "606.3");
    ruling!(
        "Invasion of New Phyrexia",
        "You activate Teferi Akosa’s last ability without choosing a target or tapping any creatures."
    );
    // (With no creatures tapped, the reflexive ability still triggers, with X = 0.)
    supported("Invasion of New Phyrexia // Teferi Akosa of Zhalfir");
    for n in [2usize, 0] {
        let mut t = TestGame::new(2);
        let tef = back_face(&mut t, P0, "Invasion of New Phyrexia // Teferi Akosa of Zhalfir");
        t.g.add_counters(Entity::Object(tef), counters::LOYALTY, 5, None);
        let mine: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        let bears = t.battlefield(P1, "Grizzly Bears");
        let thopter = t.battlefield(P1, "Ornithopter");
        activate_containing(&mut t, P0, tef, "Tap any number").expect("activates");
        assert!(mine.iter().all(|m| !tapped(&t, *m)), "nothing tapped on activation");
        t.answer_choose(P0, &objs(&mine[..n]));
        // X = number tapped: 2 lets the Bears be chosen; 0 only a mana value 0 permanent.
        let target = if n == 2 { bears } else { thopter };
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.resolve_all();
        assert_eq!(mine.iter().filter(|m| tapped(&t, **m)).count(), n);
        assert!(!t.on_battlefield(target), "X = {n}: shuffled away");
        assert!(t.on_battlefield(if n == 2 { thopter } else { bears }));
    }
}

// --- Values determined once, as the ability resolves --------------------------------------

#[test]
fn impelled_giant_checks_power_as_it_resolves_and_uses_lki() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Impelled Giant",
        "The power of the creature you tap is checked as the ability resolves, not as the creature is tapped, and not at any later time should the power of that creature change."
    );
    supported("Impelled Giant");
    // Boosted in response: the new power counts; boosted afterwards: no change.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Impelled Giant");
    let piker = t.battlefield(P0, "Goblin Piker");
    activate_containing(&mut t, P0, g, "Tap an untapped red creature").unwrap();
    assert!(tapped(&t, piker));
    boost(&mut t, piker, 3, 0);
    t.resolve_all();
    assert_eq!(t.pt(g), (8, 3));
    boost(&mut t, piker, 1, 0);
    assert_eq!(t.pt(g), (8, 3));
    // Gone by then: its power as it last existed on the battlefield.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Impelled Giant");
    let piker = t.battlefield(P0, "Goblin Piker");
    activate_containing(&mut t, P0, g, "Tap an untapped red creature").unwrap();
    boost(&mut t, piker, 1, 0);
    destroy(&mut t, piker);
    t.resolve_all();
    assert_eq!(t.pt(g), (6, 3));
}

#[test]
fn burn_at_the_stake_counts_creatures_tapped_as_the_cost() {
    cr!("601.2h", "608.2h");
    ruling!(
        "Burn at the Stake",
        "The amount of damage Burn at the Stake deals won’t change if any creatures tapped to cast it become untapped or leave the battlefield before it resolves."
    );
    supported("Burn at the Stake");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Burn at the Stake");
    let s = t.hand(P0, "Burn at the Stake");
    t.answer_choose(P0, &objs(&[a, b]));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, s).target(P1).go();
    assert!(tapped(&t, a) && tapped(&t, b));
    destroy(&mut t, a);
    t.g.objects[b.0 as usize].tapped = false;
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn tyvar_x_is_calculated_once() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Tyvar, the Pummeler",
        "The value of X is calculated only once, as Tyvar's last ability resolves."
    );
    supported("Tyvar, the Pummeler");
    let mut t = TestGame::new(2);
    let ty = t.battlefield(P0, "Tyvar, the Pummeler");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, ty, "{3}{G}{G}").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (12, 10));
    assert_eq!(t.pt(ty), (9, 9));
    // The greatest power changes afterwards: X doesn't.
    boost(&mut t, ty, 10, 0);
    assert_eq!(t.pt(wurm), (12, 10));
}

#[test]
fn omega_x_is_calculated_once() {
    cr!("608.2h", "603.3");
    ruling!(
        "Omega, Heartless Evolution",
        "The value of X is calculated only once, as Omega's ability resolves."
    );
    supported("Omega, Heartless Evolution");
    let mut t = TestGame::new(2);
    t.lands(P0, "Evolving Wilds", 2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(victim)]);
    t.enter(P0, "Omega, Heartless Evolution");
    t.settle();
    // A nonbasic land arrives in response: X is 3 as it resolves.
    t.battlefield(P0, "Evolving Wilds");
    t.resolve_all();
    assert!(tapped(&t, victim));
    assert_eq!(t.counters(victim, counters::STUN), 3);
    assert_eq!(t.life(P0), 23);
    // Later changes don't matter.
    t.battlefield(P0, "Evolving Wilds");
    assert_eq!(t.counters(victim, counters::STUN), 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn congregation_gryff_x_is_determined_as_it_resolves() {
    cr!("608.2h", "702.171a");
    ruling!(
        "Congregation Gryff",
        "The value of X is determined as Congregation Gryff’s ability resolves."
    );
    supported("Congregation Gryff");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Congregation Gryff");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &objs(&[giant]));
    activate_containing(&mut t, P0, g, "Saddle").expect("saddle");
    t.resolve_all();
    attack(&mut t, g);
    // Another Mount arrives in response: X = 2.
    t.battlefield(P0, "Congregation Gryff");
    t.resolve_all();
    assert_eq!(t.pt(g), (3, 6));
}

#[test]
fn veteran_warleader_chooses_the_ability_as_it_resolves() {
    cr!("608.2d");
    ruling!(
        "Veteran Warleader",
        "You choose which ability Veteran Warleader gains as the last ability resolves, not as you activate it."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Veteran Warleader");
    t.battlefield(P0, "Expedition Envoy");
    let from = t.asked().len();
    activate_containing(&mut t, P0, w, "Tap another untapped Ally").unwrap();
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseOption { .. });
    assert!(!asked_since(&t, from).iter().any(|(_, d)| is_choice(d)));
    let from = t.asked().len();
    t.resolve_all();
    assert!(asked_since(&t, from).iter().any(|(_, d)| is_choice(d)));
    t.g.recompute();
    assert!([
        KeywordKind::FirstStrike,
        KeywordKind::Vigilance,
        KeywordKind::Trample
    ]
    .iter()
    .any(|k| has_kw(&t, w, *k)));
}

#[test]
fn crashing_wave_distributes_stun_counters_among_any_tapped_opposing_creatures() {
    cr!("608.2d", "601.2d");
    ruling!(
        "Crashing Wave",
        "You decide how to distribute the stun counters as Crashing Wave resolves, and you can put the counters on any tapped creatures your opponents control, not just creatures tapped by Crashing Wave."
    );
    supported("Crashing Wave");
    let mut t = TestGame::new(2);
    // Already tapped before Crashing Wave: X = 0, no targets.
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let untapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[a.0 as usize].tapped = true;
    t.g.objects[b.0 as usize].tapped = true;
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Crashing Wave");
    t.cast(P0, s).x(0).go();
    t.resolve_all();
    assert_eq!(
        t.counters(a, counters::STUN) + t.counters(b, counters::STUN),
        3,
        "{}",
        t.dump_log()
    );
    assert_eq!(t.counters(untapped, counters::STUN), 0);
}

#[test]
fn cryptic_gateway_card_shares_a_type_with_each_tapped_creature() {
    cr!("205.3", "608.2d");
    ruling!(
        "Cryptic Gateway",
        "The two creatures you tap to activate the ability don’t necessarily have to share a creature type, but the creature card you put onto the battlefield must share a creature type with each of them."
    );
    supported("Cryptic Gateway");
    // An Elf Warrior and a Human Soldier: a Human Warrior can be put in, an Elf Ranger not.
    for (card, ok) in [("Kyoshi Warrior Guard", true), ("Elvish Ranger", false)] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Cryptic Gateway");
        t.battlefield(P0, "Elvish Warrior");
        t.battlefield(P0, "Alaborn Trooper");
        let c = t.hand(P0, card);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(c)]);
        activate_containing(&mut t, P0, g, "Tap two untapped creatures").unwrap();
        t.resolve_all();
        assert_eq!(!t.named_on_battlefield(card).is_empty(), ok, "{card}");
    }
}

#[test]
fn heirloom_epic_taps_at_most_the_mana_in_its_cost() {
    cr!("602.2b", "601.2f");
    ruling!(
        "Heirloom Epic",
        "You can’t tap more creatures than the amount of mana in the cost of Heirloom Epic’s ability."
    );
    // Five creatures, no lands: P0 is offered at most four to tap, and four pay it all.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Heirloom Epic");
    let cs: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let from = t.asked().len();
    t.answer_choose(P0, &objs(&cs[..4]));
    activate_containing(&mut t, P0, e, "Draw a card").expect("four creatures pay {4}");
    let max = asked_since(&t, from).into_iter().find_map(|(_, d)| match d {
        Decision::ChooseEntities { max, candidates, .. } if candidates.len() == 5 => Some(max),
        _ => None,
    });
    assert_eq!(max, Some(4), "no more creatures than the mana in the cost");
    t.resolve_all();
    assert_eq!(cs.iter().filter(|c| tapped(&t, **c)).count(), 4);
    assert!(!tapped(&t, cs[4]));
    assert_eq!(t.hand_size(P0), 1);
    // Suppression Field: "Activated abilities cost {2} more ...": six creatures pay it.
    supported("Suppression Field");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Suppression Field");
    let e = t.battlefield(P0, "Heirloom Epic");
    let cs: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.answer_choose(P0, &objs(&cs));
    activate_containing(&mut t, P0, e, "Draw a card").expect("six creatures pay {6}");
    t.resolve_all();
    assert!(cs.iter().all(|c| tapped(&t, *c)));
    assert_eq!(t.hand_size(P0), 1);
    let _ = tokens;
    let _ = Answer::Default;
}

#[test]
fn yotia_declares_war_reflexive_trigger_triggers_with_zero_artifacts_tapped() {
    cr!("603.12");
    ruling!(
        "Yotia Declares War",
        "If you choose to tap zero artifacts as the second chapter ability resolves, the reflexive triggered ability still triggers"
    );
    supported("Yotia Declares War");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Yotia Declares War");
    let a = t.battlefield(P0, "Ornithopter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(Entity::Object(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.settle();
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.resolve();
    assert!(!tapped(&t, a));
    assert_eq!(t.stack_len(), 1, "the reflexive trigger, targeting the Wurm");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 0);
    assert!(t.on_battlefield(wurm));
}

#[test]
fn when_you_do_after_tapping_any_number_triggers_but_after_one_or_more_needs_one() {
    cr!("603.12");
    // "Tap any number of ...": done even with none chosen, so the reflexive ability
    // triggers anyway; "tap one or more ..." with none chosen isn't done, and neither is
    // declining "you may tap any number of ...".
    use mtg_engine::card::Layout;
    use mtg_engine::oracle::{self, CompileContext};
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: "Test Sorcery",
        full_name: "Test Sorcery",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    for (text, unconditional) in [
        ("Tap any number of untapped creatures you control. When you do, you gain that much life.", true),
        ("Tap one or more untapped creatures you control. When you do, you gain that much life.", false),
        ("You may tap any number of untapped creatures you control. When you do, you gain that much life.", false),
    ] {
        let compiled = oracle::compile(text, &ctx);
        assert!(compiled.unsupported.is_empty(), "{text}: {:?}", compiled.unsupported);
        let ast = format!("{:?}", compiled.abilities);
        assert!(ast.contains("Reflexive"), "{text}: {ast}");
        assert_eq!(!ast.contains("PrevHappened"), unconditional, "{text}: {ast}");
    }
}
