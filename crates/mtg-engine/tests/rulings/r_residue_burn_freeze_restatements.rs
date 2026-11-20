//! Item `rulings-residue-burn-freeze` — rulings of batch P009 that restate what the card's
//! own text says (targets, costs, choices): one table, each case with its card, a minimal
//! setup and the asserted outcome, citing its rules and its ruling. Also Chandra,
//! Flameshaper's copy ability (CR 707.2), an interaction kept here with its card's other
//! ruling.

use crate::r_residue_burn_freeze_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// A restatement ruling: its card and the check (setup, outcome, citations).
type Case = (&'static str, fn());

/// The decisions asked of `p` since `from` that are of the given kind.
fn asked_kind(t: &TestGame, from: usize, p: PlayerId, kind: fn(&Decision) -> bool) -> usize {
    asked_since(t, from)
        .iter()
        .filter(|(q, d)| *q == p && kind(d))
        .count()
}

#[test]
fn restatement_rulings() {
    let cases: &[Case] = &[
        ("Soul of Shandalar", || {
            cr!("115.1", "601.2c", "602.2b");
            ruling!(
                "Soul of Shandalar",
                "To activate either ability, you must target a player, but you can choose to not target any creatures."
            );
            supported("Soul of Shandalar");
            // From the battlefield, then from the graveyard: only P1 is targeted.
            for from_graveyard in [false, true] {
                let mut t = TestGame::new(2);
                t.set_step(P0, Step::PrecombatMain);
                let soul = if from_graveyard {
                    t.graveyard(P0, "Soul of Shandalar")
                } else {
                    t.battlefield(P0, "Soul of Shandalar")
                };
                let bears = t.battlefield(P1, "Grizzly Bears");
                t.lands(P0, "Mountain", 5);
                t.answer_targets(P0, &[pl(P1)]);
                t.answer_targets(P0, &[]);
                let i = usize::from(from_graveyard);
                t.activate(P0, soul, i, &[])
                    .unwrap_or_else(|e| panic!("graveyard {from_graveyard}: {e:?}"));
                t.resolve_all();
                assert_eq!(t.life(P1), 17, "graveyard {from_graveyard}");
                assert_eq!(dmg(&t, bears), 0);
            }
        }),
        ("Chandra, Pyromaster", || {
            cr!("115.1", "606.3");
            ruling!(
                "Chandra, Pyromaster",
                "To activate the first ability, you must target a player but you can choose to not target any creatures."
            );
            supported("Chandra, Pyromaster");
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let c = pw(&mut t, P0, "Chandra, Pyromaster", 4);
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.answer_targets(P0, &[pl(P1)]);
            t.answer_targets(P0, &[]);
            loyalty(&mut t, P0, c, 0, &[]);
            t.resolve_all();
            assert_eq!(t.life(P1), 19);
            assert_eq!(dmg(&t, bears), 0);
            assert_eq!(t.counters(c, "loyalty"), 5);
        }),
        ("Sarkhan the Mad", || {
            cr!("606.4");
            ruling!(
                "Sarkhan the Mad",
                "Unlike previous planeswalkers, Sarkhan the Mad has no way to raise his loyalty."
            );
            supported("Sarkhan the Mad");
            let c = mtg_engine::card::card("Sarkhan the Mad");
            let costs: Vec<i32> = c
                .front()
                .chars
                .abilities
                .iter()
                .filter_map(|a| match &a.kind {
                    AbilityKind::Activated(x) if x.is_loyalty => x.cost.loyalty(),
                    _ => None,
                })
                .collect();
            assert_eq!(costs, vec![0, -2, -4]);
        }),
        ("Sarkhan the Mad", || {
            cr!("115.1", "120.3");
            ruling!(
                "Sarkhan the Mad",
                "You choose a single target as you activate the third ability. All your Dragons deal damage to that player."
            );
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
            t.battlefield(P0, "Shivan Dragon");
            t.battlefield(P0, "Shivan Dragon");
            let from = t.asked().len();
            loyalty(&mut t, P0, s, 2, &[pl(P1)]);
            assert_eq!(
                asked_kind(&t, from, P0, |d| matches!(
                    d,
                    Decision::ChooseTargets { .. }
                )),
                1
            );
            t.resolve_all();
            assert_eq!(t.life(P1), 10, "5 from each Dragon");
        }),
        ("Nahiri's Wrath", || {
            cr!("115.3", "601.2c");
            ruling!(
                "Nahiri's Wrath",
                "You can't target the same creature or planeswalker multiple times to deal more damage to it."
            );
            supported("Nahiri's Wrath");
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let wurm = t.battlefield(P1, "Craw Wurm");
            t.hand(P0, "Hill Giant");
            t.hand(P0, "Hill Giant");
            t.lands(P0, "Mountain", 3);
            let wrath = t.hand(P0, "Nahiri's Wrath");
            t.answer(P0, DecisionKind::X, Answer::Number(2));
            t.answer_targets(P0, &[obj(wurm), obj(wurm)]);
            let spell = t.cast(P0, wrath).x(2).try_go();
            // X = 2 but only one creature: at most one target can be chosen, and the Wurm
            // named twice isn't accepted.
            let maxes: Vec<u32> = t
                .asked()
                .iter()
                .filter_map(|(_, d)| match d {
                    Decision::ChooseTargets { max, .. } => Some(*max),
                    _ => None,
                })
                .collect();
            assert_eq!(maxes, vec![1]);
            let spell = spell.expect("cast with no targets");
            assert!(crate::r_s25_common::targets_of(&t, spell).is_empty());
            // Targeted once, it's dealt the 8 damage once.
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let wurm = t.battlefield(P1, "Craw Wurm");
            t.hand(P0, "Hill Giant");
            t.hand(P0, "Hill Giant");
            t.lands(P0, "Mountain", 3);
            let wrath = t.hand(P0, "Nahiri's Wrath");
            t.answer(P0, DecisionKind::X, Answer::Number(2));
            t.answer_targets(P0, &[obj(wurm)]);
            t.cast(P0, wrath).x(2).go();
            t.resolve_all();
            let dealt: Vec<u32> = damage_events(&t, 0)
                .into_iter()
                .filter(|d| d.1 == obj(wurm))
                .map(|d| d.2)
                .collect();
            assert_eq!(dealt, vec![8]);
        }),
        ("Wolf of Devil's Breach", || {
            cr!("603.3d", "608.2", "601.2c");
            ruling!(
                "Wolf of Devil's Breach",
                "You choose the target creature or planeswalker as the triggered ability of Wolf of Devil’s Breach is put onto the stack. You choose which card to discard, if any, as the ability resolves. While players may respond to the triggered ability once you’ve chosen a target, no player may take actions between the time you discard the card and the time damage is dealt."
            );
            supported("Wolf of Devil's Breach");
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let wolf = t.battlefield(P0, "Wolf of Devil's Breach");
            let wurm = t.battlefield(P1, "Craw Wurm");
            let giant = t.hand(P0, "Hill Giant");
            t.lands(P0, "Mountain", 2);
            t.answer_targets(P0, &[obj(wurm)]);
            attack_with(&mut t, &[(wolf, pl(P1))]);
            // The trigger is on the stack with its target; nothing discarded yet.
            assert_eq!(t.stack_len(), 1);
            assert_eq!(t.zone(giant), Zone::Hand(P0));
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(giant)]);
            let from = t.asked().len();
            t.g.resolve_top();
            t.g.flush_events();
            // While it resolved, no one got priority: the discard and the damage happen
            // together.
            assert_eq!(
                asked_kind(&t, from, P0, |d| matches!(d, Decision::Priority { .. }))
                    + asked_kind(&t, from, P1, |d| matches!(d, Decision::Priority { .. })),
                0
            );
            assert!(t.in_graveyard(P0, "Hill Giant"));
            t.settle();
            assert!(!t.on_battlefield(wurm), "4 damage");
        }),
        ("Chandra, Flameshaper", || {
            cr!("601.2d", "606.3", "115.1");
            ruling!(
                "Chandra, Flameshaper",
                "You choose the targets and how damage will be divided as you activate Chandra's last ability. Each chosen target must receive at least 1 damage."
            );
            supported("Chandra, Flameshaper");
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let c = pw(&mut t, P0, "Chandra, Flameshaper", 6);
            let a = t.battlefield(P1, "Grizzly Bears");
            let b = t.battlefield(P1, "Craw Wurm");
            t.answer_targets(P0, &[obj(a), obj(b)]);
            divide(&mut t, P0, &[2, 6]);
            let from = t.asked().len();
            loyalty(&mut t, P0, c, 2, &[]);
            // Asked as it's activated, with at least 1 for each target.
            let divisions: Vec<u32> = asked_since(&t, from)
                .into_iter()
                .filter_map(|(_, d)| match d {
                    Decision::Divide {
                        min_each, total, ..
                    } => {
                        assert_eq!(total, 8);
                        Some(min_each)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(divisions, vec![1]);
            t.resolve_all();
            assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
        }),
        ("Galvanic Discharge", || {
            cr!("107.14");
            ruling!(
                "Galvanic Discharge",
                "You may pay zero {E}. You will get {E}{E}{E}, but Galvanic Discharge won't deal any damage."
            );
            supported("Galvanic Discharge");
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.answer(P0, DecisionKind::X, Answer::Number(0));
            cast_new(&mut t, P0, "Galvanic Discharge", &[obj(bears)]);
            t.resolve_all();
            assert_eq!(energy(&t, P0), 3);
            assert_eq!(dmg(&t, bears), 0);
        }),
        ("Searing Blaze", || {
            cr!("601.2c", "115.1");
            ruling!(
                "Searing Blaze",
                "You must choose two targets as you cast Searing Blaze: a player and a creature that player controls. If you can't (because there are no creatures on the battlefield, perhaps), then you can't cast the spell."
            );
            supported("Searing Blaze");
            for creature in [false, true] {
                let mut t = TestGame::new(2);
                t.set_step(P0, Step::PrecombatMain);
                let bears = creature.then(|| t.battlefield(P1, "Grizzly Bears"));
                t.lands(P0, "Mountain", 2);
                let blaze = t.hand(P0, "Searing Blaze");
                t.answer_targets(P0, &[pl(P1)]);
                if let Some(b) = bears {
                    t.answer_targets(P0, &[obj(b)]);
                }
                let r = t.cast(P0, blaze).try_go();
                assert_eq!(r.is_ok(), creature, "creature {creature}");
                if let Some(b) = bears {
                    t.resolve_all();
                    assert_eq!((t.life(P1), dmg(&t, b)), (19, 1));
                }
            }
        }),
        ("Charge of the Forever-Beast", || {
            cr!("601.2b", "701.20a", "608.2h");
            ruling!(
                "Charge of the Forever-Beast",
                "You must reveal exactly one card to cast Charge of the Forever-Beast; you can’t cast it without revealing a card, and you can’t reveal additional cards. The revealed card remains revealed until Charge of the Forever-Beast resolves."
            );
            supported("Charge of the Forever-Beast");
            // No creature card in hand: it can't be cast.
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let wurm = t.battlefield(P1, "Craw Wurm");
            t.lands(P0, "Forest", 3);
            t.hand(P0, "Shock");
            let charge = t.hand(P0, "Charge of the Forever-Beast");
            t.answer_targets(P0, &[obj(wurm)]);
            assert!(t.cast(P0, charge).try_go().is_err());
            // Two creature cards: exactly one is revealed; it stays revealed until the
            // spell resolves.
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let wurm = t.battlefield(P1, "Craw Wurm");
            t.lands(P0, "Forest", 3);
            let giant = t.hand(P0, "Hill Giant");
            let bears = t.hand(P0, "Grizzly Bears");
            let charge = t.hand(P0, "Charge of the Forever-Beast");
            t.answer_choose(P0, &[obj(giant), obj(bears)]);
            t.answer_targets(P0, &[obj(wurm)]);
            let from = t.asked().len();
            let r = t.cast(P0, charge).try_go();
            let sizes: Vec<(u32, u32)> = asked_since(&t, from)
                .into_iter()
                .filter_map(|(_, d)| match d {
                    Decision::ChooseEntities { min, max, .. } => Some((min, max)),
                    _ => None,
                })
                .collect();
            assert_eq!(sizes, vec![(1, 1)], "exactly one card");
            if r.is_err() {
                // The two-card answer was refused: reveal one.
                t.clear_answers();
                t.answer_choose(P0, &[obj(giant)]);
                t.answer_targets(P0, &[obj(wurm)]);
                t.cast(P0, charge).go();
            }
            let revealed: Vec<ObjectId> = [giant, bears]
                .into_iter()
                .filter(|c| mtg_engine::reveal::is_revealed(&t.g, *c))
                .collect();
            assert_eq!(revealed.len(), 1);
            let power = if revealed[0] == giant { 3 } else { 2 };
            t.resolve_all();
            assert_eq!(dmg(&t, wurm), power);
            assert!(!mtg_engine::reveal::is_revealed(&t.g, revealed[0]));
        }),
        ("Whirling Catapult", || {
            cr!("602.2b", "118.3");
            ruling!(
                "Whirling Catapult",
                "If your library has fewer than two cards in it, you can't activate the ability."
            );
            supported("Whirling Catapult");
            for cards in [1usize, 2] {
                let mut t = TestGame::new(2);
                t.set_step(P0, Step::PrecombatMain);
                t.g.players[P0.idx()].library.clear();
                stack_library(&mut t, P0, &vec!["Shock"; cards]);
                let cat = t.battlefield(P0, "Whirling Catapult");
                t.lands(P0, "Wastes", 2);
                assert_eq!(t.activate(P0, cat, 0, &[]).is_ok(), cards == 2, "{cards}");
            }
        }),
    ];
    for (name, check) in cases {
        let _ = name;
        check();
    }
}

#[test]
fn chandra_flameshapers_token_copies_only_the_copiable_values() {
    cr!("707.2", "707.9b", "111.1", "702.122a");
    ruling!(
        "Chandra, Flameshaper",
        "The token created by Chandra's second ability copies exactly what was printed on the original creature and nothing else, with the listed exceptions (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on. If it is a Vehicle, it is not crewed."
    );
    supported("Chandra, Flameshaper");
    // A tapped Grizzly Bears with a +1/+1 counter, Holy Strength and Giant Growth.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Flameshaper", 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 1);
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", obj(bears));
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    tap(&mut t, bears);
    assert_eq!(t.pt(bears), (7, 8));
    loyalty(&mut t, P0, c, 1, &[obj(bears)]);
    t.resolve_all();
    let token = tokens(&t, P0);
    assert_eq!(token.len(), 1);
    let tok = token[0];
    assert_eq!(t.pt(tok), (2, 2));
    assert!(!is_tapped(&t, tok));
    assert_eq!(t.counters(tok, "+1/+1"), 0);
    assert!(t.g.attachments_of(obj(tok)).is_empty());
    assert!(crate::r_s06_common::has_kw(
        &t,
        tok,
        mtg_engine::keywords::KeywordKind::Haste
    ));
    // A crewed Vehicle: the token isn't crewed (not a creature).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Flameshaper", 6);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let crew = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[crew]));
    t.resolve_all();
    t.g.recompute();
    assert!(t.obj_now(copter).is_creature());
    loyalty(&mut t, P0, c, 1, &[obj(copter)]);
    t.resolve_all();
    let token = tokens(&t, P0);
    assert_eq!(token.len(), 1);
    assert!(!t.obj_now(token[0]).is_creature());
    assert!(t.obj_now(token[0]).chars.is(CardType::Artifact));
}
