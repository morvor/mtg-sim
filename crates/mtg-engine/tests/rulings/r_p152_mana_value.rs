//! Rulings batch P152 — abilities scaling with damage dealt or with mana value: added
//! phases and steps (CR 500.8, 500.10), mana value (CR 202.3), repeated damage (CR 120.3,
//! 704.3), reflexive and delayed triggers (CR 603.7, 603.12), and creatures put onto the
//! battlefield attacking (CR 508.4).

use crate::r_p152_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// The steps of the rest of `p`'s current turn (until another player's turn starts).
fn rest_of_turn_steps(t: &mut TestGame, p: PlayerId) -> Vec<Step> {
    let mut log: Vec<Step> = vec![];
    let ok = t.g.run_until(20_000, |g| {
        if g.turn.active == p && g.turn.stage != Stage::PreGame {
            log = g.turn.step_log.clone();
            false
        } else {
            true
        }
    });
    assert!(ok);
    log
}

/// Candidates of the entity choices asked of `p` since decision `from`.
fn entity_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Obeka: additional upkeep steps after this phase
// ---------------------------------------------------------------------------------------

#[test]
fn obekas_upkeeps_follow_the_combat_phase_then_the_turn_goes_on() {
    cr!("500.10", "500.8");
    ruling!(
        "Obeka, Splitter of Seconds",
        "The beginning phases containing the additional upkeep steps occur after the current combat phase ends."
    );
    ruling!(
        "Obeka, Splitter of Seconds",
        "the game proceeds to the next phase as normal, which is likely to be your postcombat main phase"
    );
    supported("Obeka, Splitter of Seconds");
    let mut t = TestGame::new(2);
    let obeka = t.battlefield(P0, "Obeka, Splitter of Seconds");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(obeka, Entity::Player(P1))], &[]);
    let steps = rest_of_turn_steps(&mut t, P0);
    let after: Vec<Step> = steps
        .into_iter()
        .skip_while(|s| *s != Step::CombatDamage)
        .collect();
    assert_eq!(
        after,
        vec![
            Step::CombatDamage,
            Step::EndOfCombat,
            Step::Upkeep,
            Step::Upkeep,
            Step::PostcombatMain,
            Step::End,
            Step::Cleanup,
        ]
    );
}

#[test]
fn obeka_a_later_added_combat_phase_comes_first() {
    cr!("500.8");
    ruling!(
        "Obeka, Splitter of Seconds",
        "If multiple phases are added to the same point in your turn, the most recently created phase happens first."
    );
    supported("Obeka, Splitter of Seconds");
    supported("Savage Beating");
    let mut t = TestGame::new(2);
    let obeka = t.battlefield(P0, "Obeka, Splitter of Seconds");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(obeka, Entity::Player(P1))], &[]);
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    // During end of combat, Savage Beating's second mode adds a combat phase after this
    // one.
    let sb = t.hand(P0, "Savage Beating");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, sb).modes(&[1]).go();
    t.resolve_all();
    let steps = rest_of_turn_steps(&mut t, P0);
    let after: Vec<Step> = steps
        .into_iter()
        .skip_while(|s| *s != Step::EndOfCombat)
        .collect();
    assert_eq!(
        &after[..4],
        &[
            Step::EndOfCombat,
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::EndOfCombat,
        ],
        "{after:?}"
    );
    assert_eq!(
        &after[4..],
        &[
            Step::Upkeep,
            Step::Upkeep,
            Step::PostcombatMain,
            Step::End,
            Step::Cleanup
        ],
        "{after:?}"
    );
}

// ---------------------------------------------------------------------------------------
// Mana value
// ---------------------------------------------------------------------------------------

#[test]
fn lozhan_can_target_players_and_noncommander_creatures() {
    cr!("115.4", "903.3");
    ruling!(
        "Lozhan, Dragons' Legacy",
        "\"Any target that isn't a commander\" includes any player, as well as any creature or planeswalker that isn't a commander."
    );
    supported("Lozhan, Dragons' Legacy");
    let mut t = commander_game();
    t.battlefield(P0, "Lozhan, Dragons' Legacy");
    let cmd = t.battlefield(P1, "Hill Giant");
    t.g.objects[cmd.0 as usize].is_commander = true;
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    let n = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Shivan Dragon", &[]);
    t.settle();
    let cands = target_candidates(&t, P0, n);
    assert!(cands[0].contains(&Entity::Player(P1)) && cands[0].contains(&Entity::Player(P0)));
    assert!(cands[0].contains(&obj(bears)));
    assert!(!cands[0].contains(&obj(cmd)));
    t.resolve();
    assert_eq!(t.life(P1), 34, "40 - the Dragon's mana value 6");
}

#[test]
fn x_in_a_mana_cost_counts_as_zero_for_pyromancy_and_wand_of_ith() {
    cr!("202.3e");
    ruling!("Pyromancy", "A card with X in the cost is treated as X=0.");
    ruling!(
        "Wand of Ith",
        "Any X in the mana cost of a card is zero for purposes of the Wand."
    );
    supported("Pyromancy");
    supported("Wand of Ith");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Pyromancy");
    t.hand(P0, "Fireball");
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, pyro, "Discard a card at random", &[Entity::Player(P1)]).expect("Pyromancy");
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "Fireball's mana value is 1");
    // Wand of Ith: P1 pays life equal to Fireball's mana value (1) to keep it.
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Wand of Ith");
    t.hand(P1, "Fireball");
    pool(&mut t, P0, &[ManaType::C], 3);
    t.answer_yes(P1, true);
    act(&mut t, P0, wand, "reveals a card at random", &[Entity::Player(P1)]).expect("Wand");
    t.resolve_all();
    assert!(t.in_hand(P1, "Fireball"));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn saruman_gives_a_goblin_orc_one_ward() {
    cr!("702.21a", "613.1f");
    ruling!(
        "Saruman, the White Hand",
        "A creature that is both a Goblin and an Orc still only gets one instance of ward {2}"
    );
    supported("Saruman, the White Hand");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Saruman, the White Hand");
    let goblin = t.battlefield(P0, "Goblin Piker");
    crate::r_p117_common::modify(
        &mut t,
        goblin,
        vec![mtg_engine::ability::Modification::AddSubtypes(vec![
            "Orc".into()
        ])],
    );
    assert!(t.obj_now(goblin).chars.has_subtype("Orc"));
    let wards = t
        .obj_now(goblin)
        .chars
        .abilities
        .iter()
        .filter(|a| a.keyword().is_some_and(|k| k.kind == KeywordKind::Ward))
        .count();
    assert_eq!(wards, 1);
}

#[test]
fn kindle_the_carnage_damage_iterations_are_separate_and_deaths_wait() {
    cr!("120.3", "704.3", "701.19c");
    ruling!(
        "Kindle the Carnage",
        "A creature that’s been dealt lethal damage by an iteration of this effect will remain on the battlefield and be dealt more damage by any further iterations."
    );
    ruling!(
        "Kindle the Carnage",
        "A single regeneration shield is sufficient to save a particular creature"
    );
    ruling!(
        "Kindle the Carnage",
        "All the damage has the same source. However, each iteration of this process is a separate instance of damage."
    );
    ruling!(
        "Kindle the Carnage",
        "Any damage prevention or regeneration abilities must be activated before Kindle the Carnage starts resolving"
    );
    supported("Kindle the Carnage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    // The regeneration shield, before Kindle the Carnage resolves.
    t.activate(P1, skel, 0, &[]).expect("regenerate");
    t.resolve_all();
    let kindle = cast_new(&mut t, P0, "Kindle the Carnage", &[]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0, "two iterations");
    let hits = |id: ObjectId| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Damage { target: Entity::Object(o), source, .. }
                if *o == id && *source == kindle))
            .count()
    };
    assert_eq!(hits(bears), 2, "dealt damage again after lethal damage");
    assert_eq!(hits(wurm), 2);
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(wurm), "4 damage each");
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_graveyard(P1, "Craw Wurm"));
    assert!(t.on_battlefield(skel), "one shield saved it from both instances");
    assert!(t.obj_now(skel).tapped);
}

/// The creature cards Eldritch Evolution's search offered after sacrificing `sac`.
fn evolution_candidates(t: &mut TestGame, sac: ObjectId) -> Vec<String> {
    t.answer_choose(P0, &[obj(sac)]);
    let n = t.asked().len();
    cast_new(t, P0, "Eldritch Evolution", &[]);
    t.resolve_all();
    let cands = entity_candidates(t, P0, n);
    let last = cands.last().cloned().unwrap_or_default();
    let mut names: Vec<String> = last
        .iter()
        .filter_map(|e| match e {
            Entity::Object(o) => Some(t.g.obj(*o).chars.name.to_string()),
            _ => None,
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

#[test]
fn eldritch_evolution_uses_the_sacrificed_creatures_mana_value() {
    cr!("202.3", "202.3e", "707.2");
    ruling!(
        "Eldritch Evolution",
        "If the mana cost includes {X}, X is considered to be 0."
    );
    ruling!(
        "Eldritch Evolution",
        "A token that isn't a copy of another permanent has a mana value of 0. A token that is a copy or a creature that has become a copy of another permanent has the mana value of what it's copying."
    );
    supported("Eldritch Evolution");
    let library = |t: &mut TestGame| {
        for n in ["Grizzly Bears", "Hill Giant", "Craw Wurm"] {
            t.library_top(P0, n);
        }
    };
    // A plain token: mana value 0, up to 2.
    let mut t = TestGame::new(2);
    library(&mut t);
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    assert_eq!(evolution_candidates(&mut t, tok), vec!["Grizzly Bears"]);
    // Endless One with counters: X is 0, up to 2.
    let mut t = TestGame::new(2);
    library(&mut t);
    let endless = t.battlefield(P0, "Endless One");
    put_counters(&mut t, endless, "+1/+1", 4);
    assert_eq!(evolution_candidates(&mut t, endless), vec!["Grizzly Bears"]);
    // A token copy of Hill Giant: mana value 4, up to 6.
    let mut t = TestGame::new(2);
    library(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    let mp = t.battlefield(P0, "Mirrorpool");
    pool(&mut t, P0, &[ManaType::C], 5);
    act(&mut t, P0, mp, "Create a token that's a copy", &[obj(giant)]).expect("Mirrorpool");
    t.resolve_all();
    let copy = tokens(&t, P0)[0];
    assert_eq!(
        evolution_candidates(&mut t, copy),
        vec!["Craw Wurm", "Grizzly Bears", "Hill Giant"]
    );
}

#[test]
fn life_animated_lands_need_to_have_been_controlled_since_the_turn_began() {
    cr!("302.6", "305.1");
    ruling!(
        "Life // Death",
        "Notably, if you turn a land into a creature on the turn it entered the battlefield, you won't be able to tap it for mana."
    );
    supported("Life // Death");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Forest");
    let new = t.hand(P0, "Forest");
    t.play_land(P0, new).expect("land");
    let life = t.hand(P0, "Life // Death");
    t.cast(P0, life).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert!(t.obj_now(new).is(mtg_engine::types::CardType::Creature));
    assert!(t.obj_now(old).tapped, "paid for Life");
    let o = t.g.current(old);
    t.g.untap(o);
    assert!(t.activate(P0, new, 0, &[]).is_err(), "summoning sick");
    assert!(t.activate(P0, old, 0, &[]).is_ok());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, new));
}

#[test]
fn kellan_lets_you_cast_only_a_permanent_spell() {
    cr!("110.4b", "601.3");
    ruling!(
        "Kellan, the Kid",
        "A permanent spell is an artifact, battle, creature, enchantment, or planeswalker spell."
    );
    supported("Kellan, the Kid");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kellan, the Kid");
    let shock = t.hand(P0, "Shock");
    let thopter = t.hand(P0, "Ornithopter");
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let n = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve();
    let cands = entity_candidates(&t, P0, n);
    let offered: Vec<Entity> = cands.into_iter().flatten().collect();
    assert!(offered.contains(&obj(thopter)), "{offered:?}");
    assert!(!offered.contains(&obj(shock)));
    t.resolve_all();
    assert_eq!(count_named(&t, "Ornithopter"), 1);
    assert!(t.in_hand(P0, "Shock"));
}

#[test]
fn karn_stops_activated_but_not_triggered_abilities_of_opponents_artifacts() {
    cr!("602.5", "603.1");
    ruling!(
        "Karn, the Great Creator",
        "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by Karn."
    );
    supported("Karn, the Great Creator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karn, the Great Creator");
    let stone = t.battlefield(P1, "Mind Stone");
    let boiler = t.battlefield(P1, "Plague Boiler");
    let mine = t.battlefield(P0, "Mind Stone");
    assert!(t.activate(P1, stone, 0, &[]).is_err(), "mana ability too");
    assert!(t.activate(P0, mine, 0, &[]).is_ok(), "yours are fine");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(boiler, "plague"), 1, "the upkeep trigger still works");
}

#[test]
fn the_eleventh_doctor_unblockability_survives_a_power_boost() {
    cr!("611.2c", "509.1b");
    ruling!(
        "The Eleventh Doctor",
        "After The Eleventh Doctor's last ability resolves, that creature can't be blocked this turn even if its power later increases"
    );
    supported("The Eleventh Doctor");
    let mut t = TestGame::new(2);
    let doc = t.battlefield(P0, "The Eleventh Doctor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, doc, "can't be blocked", &[obj(bears)]).expect("Doctor");
    t.resolve_all();
    crate::r_p119_common::giant_growth(&mut t, bears);
    assert_eq!(t.pt(bears).0, 5);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.g.recompute();
    let opts = mtg_engine::combat::block_options(&t.g, &[P1]);
    let can = opts
        .iter()
        .any(|(b, atk)| *b == blocker && atk.contains(&t.g.current(bears)));
    assert!(!can);
    block_and_finish(&mut t, P1, &[(blocker, bears)]);
    assert_eq!(t.life(P1), 15, "unblocked");
}

#[test]
fn viviens_delayed_trigger_resolves_before_the_creature_spell() {
    cr!("603.7a", "608.1");
    ruling!(
        "Vivien, Monsters' Advocate",
        "the delayed triggered ability it creates will put a creature card onto the battlefield before the creature spell that caused the ability to trigger resolves."
    );
    supported("Vivien, Monsters' Advocate");
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien, Monsters' Advocate");
    t.g.objects[vivien.0 as usize].counters.insert("loyalty".into(), 5);
    let bears = t.library_top(P0, "Grizzly Bears");
    act(&mut t, P0, vivien, "When you next cast", &[]).expect("-2");
    t.resolve_all();
    let giant = cast_new(&mut t, P0, "Hill Giant", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    assert_eq!(t.zone(giant), Zone::Stack, "the Giant is still a spell");
    t.resolve_all();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn cemetery_desecrator_chooses_mode_and_target_after_exiling() {
    cr!("603.12", "700.2b");
    ruling!(
        "Cemetery Desecrator",
        "You choose the mode and target as you put that second ability on the stack, not before you exile a card."
    );
    supported("Cemetery Desecrator");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Hill Giant");
    let victim = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(card)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(victim)]);
    let n = t.asked().len();
    t.enter(P0, "Cemetery Desecrator");
    t.resolve_all();
    let asked = &t.asked()[n..];
    let exile_at = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .expect("exile choice");
    let mode_at = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .expect("mode choice");
    assert!(exile_at < mode_at);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.on_battlefield(victim), "-4/-4");
}

#[test]
fn counterpoint_still_lets_you_cast_if_the_spell_cant_be_countered() {
    cr!("701.6b", "608.2c");
    ruling!(
        "Counterpoint",
        "if the target spell is still a legal target but can’t be countered, Counterpoint will still resolve"
    );
    supported("Counterpoint");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let shock = t.graveyard(P0, "Shock");
    t.set_step(P1, Step::PrecombatMain);
    let decay = cast_new(&mut t, P1, "Abrupt Decay", &[obj(thopter)]);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Counterpoint");
    let cp = t.hand(P0, "Counterpoint");
    t.answer_targets(P0, &[obj(decay)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast_with(P0, cp, &[]).expect("Counterpoint");
    t.resolve_all();
    assert!(!t.on_battlefield(thopter), "Abrupt Decay resolved");
    assert_eq!(t.life(P1), 18, "Shock cast from the graveyard");
}

#[test]
fn yuriko_entering_attacking_doesnt_trigger_attack_abilities() {
    cr!("508.4", "702.49a");
    ruling!(
        "Yuriko, the Tiger's Shadow",
        "Although the ninjutsu ability has the creature enter the battlefield attacking, it was never declared as an attacking creature"
    );
    supported("Yuriko, the Tiger's Shadow");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let yuriko = t.hand(P0, "Yuriko, the Tiger's Shadow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "Hellrider for the Bears");
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, yuriko, 0, &[]).expect("ninjutsu");
    t.resolve_all();
    assert_eq!(count_named(&t, "Yuriko, the Tiger's Shadow"), 1);
    assert_eq!(t.life(P1), 19, "no attack trigger for Yuriko");
}
