//! Rulings batch P002 — alternate win conditions (CR 104.2b) and intervening "if" clauses
//! (CR 603.4), including Two-Headed Giant (CR 810.9); "sacrifice a creature or pay"
//! additional costs (CR 601.2b, 601.2f-h); a face-down creature turned face up in combat
//! (CR 506.4, 708.8).

use crate::r_p003_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn won(t: &TestGame, p: PlayerId) -> bool {
    t.g.players[p.idx()].has_won
}

// ---------------------------------------------------------------------------------------
// Approach of the Second Sun.
// ---------------------------------------------------------------------------------------

#[test]
fn approach_of_the_second_sun_does_nothing_until_it_resolves() {
    cr!("104.2b", "608.2", "701.6a");
    ruling!(
        "Approach of the Second Sun",
        "Approach of the Second Sun has no effect until it's resolving. If the second one you cast is countered, you won't win the game."
    );
    supported("Approach of the Second Sun");
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Approach of the Second Sun", "6W", &[]);
    assert_eq!(t.life(P0), 27);
    let second = cast(&mut t, P0, "Approach of the Second Sun", "6W", &[]);
    cast(&mut t, P1, "Cancel", "1UU", &[obj(second)]);
    t.resolve_all();
    assert!(!won(&t, P0) && !t.has_lost(P1));
    assert!(t.g.result.is_none());
    // A third one, cast from the hand, resolves: two others were cast this game.
    cast_resolve(&mut t, P0, "Approach of the Second Sun", "6W", &[]);
    assert!(won(&t, P0));
    assert!(t.has_lost(P1));
}

// ---------------------------------------------------------------------------------------
// Vraska the Unseen's Assassins.
// ---------------------------------------------------------------------------------------

/// P0's Vraska with 7 loyalty uses its −7: three Assassins.
fn assassins(t: &mut TestGame) -> Vec<ObjectId> {
    supported("Vraska the Unseen");
    let v = add(t, P0, "Vraska the Unseen");
    t.g.objects[v.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    activate_resolve(t, P0, v, "Assassin", "", &[]);
    let a = tokens(t, P0);
    assert_eq!(a.len(), 3);
    a
}

/// The active player attacks `p` with `attacker`; the game runs until combat ends or the
/// game does.
fn attack_until_over(t: &mut TestGame, attacker: ObjectId, p: PlayerId) {
    crate::r_s01_common::attack_with(t, &[(attacker, pl(p))]);
    let ok = t.g.run_until(10_000, |g| {
        g.result.is_some() || g.turn.step == Step::EndOfCombat
    });
    assert!(ok);
}

#[test]
fn vraska_s_assassins_make_any_player_they_damage_lose_and_copies_do_too() {
    cr!("104.3e", "603.2", "707.2", "510.2");
    ruling!(
        "Vraska the Unseen",
        "Each Assassin token's triggered ability will trigger whenever it deals combat damage to any player, including you."
    );
    ruling!(
        "Vraska the Unseen",
        "If an effect creates a copy of one of the Assassin creature tokens, the copy will also have the triggered ability."
    );
    supported("Clone");
    // P1 gains control of an Assassin and attacks P0 with it: P0 loses.
    let mut t = TestGame::new(2);
    let a = assassins(&mut t);
    crate::r_s06_common::give_control(&mut t, a[0], P1);
    t.g.objects[a[0].0 as usize].summoning_sick = false;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_until_over(&mut t, a[0], P0);
    assert!(t.has_lost(P0));
    // P1's Clone copying an Assassin attacks P0: P0 loses.
    let mut t = TestGame::new(2);
    let a = assassins(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(a[0])]);
    cast_resolve(&mut t, P1, "Clone", "3U", &[]);
    let clone = t.named_on_battlefield("Clone");
    assert!(clone.is_empty());
    let copy = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).controller == P1 && t.g.obj(*o).chars.has_subtype("Assassin"))
        .expect("the Clone is an Assassin");
    t.g.objects[copy.0 as usize].summoning_sick = false;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_until_over(&mut t, copy, P0);
    assert!(t.has_lost(P0));
    assert!(!t.has_lost(P1));
}

// ---------------------------------------------------------------------------------------
// Intervening "if" clauses: Biovisionary, Hellkite Tyrant, Near-Death Experience.
// ---------------------------------------------------------------------------------------

/// The triggered abilities on the stack whose text contains `text`.
fn triggers_from(t: &TestGame, text: &str) -> usize {
    crate::r_s01_common::triggers_on_stack(t, text)
}

#[test]
fn biovisionary_checks_four_on_trigger_and_resolution_and_token_copies_count() {
    cr!("603.4", "104.2b", "111.10");
    ruling!(
        "Biovisionary",
        "If the ability does trigger, but you don’t control four or more creatures named Biovisionary when the ability tries to resolve, the ability will do nothing."
    );
    ruling!(
        "Biovisionary",
        "If you don’t control four or more creatures named Biovisionary at the beginning of the end step, the ability won’t trigger."
    );
    ruling!("Biovisionary", "Token creatures that are a copy of Biovisionary will count.");
    supported("Biovisionary");
    supported("Cackling Counterpart");
    // Three Biovisionaries and a token copy: four triggers; win.
    let mut t = TestGame::new(2);
    let bios: Vec<ObjectId> = (0..3).map(|_| add(&mut t, P0, "Biovisionary")).collect();
    cast_resolve(&mut t, P0, "Cackling Counterpart", "1UU", &[obj(bios[0])]);
    assert_eq!(t.named_on_battlefield("Biovisionary").len(), 4);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, "four or more creatures named"), 4);
    t.resolve();
    assert!(won(&t, P0));
    // Four triggers, then one Biovisionary dies: nothing.
    let mut t = TestGame::new(2);
    let bios: Vec<ObjectId> = (0..4).map(|_| add(&mut t, P0, "Biovisionary")).collect();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, "four or more creatures named"), 4);
    kill(&mut t, bios[0]);
    t.resolve_all();
    assert!(!won(&t, P0));
    assert!(t.g.result.is_none());
    // Three: no triggers at all.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        add(&mut t, P0, "Biovisionary");
    }
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, "four or more creatures named"), 0);
}

#[test]
fn hellkite_tyrant_checks_twenty_artifacts_on_trigger_and_resolution() {
    cr!("603.4", "104.2b");
    ruling!(
        "Hellkite Tyrant",
        "If you don't control twenty or more artifacts at the beginning of your upkeep, Hellkite Tyrant's last ability won't trigger. If it does trigger, it will check again when it tries to resolve."
    );
    supported("Hellkite Tyrant");
    for (n, lose_one, wins) in [(20, false, true), (20, true, false), (19, false, false)] {
        let mut t = TestGame::new(2);
        add(&mut t, P0, "Hellkite Tyrant");
        let arts: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P0, "Ornithopter")).collect();
        t.set_step(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_from(&t, "twenty or more artifacts"), usize::from(n >= 20));
        if lose_one {
            kill(&mut t, arts[0]);
        }
        t.resolve_all();
        assert_eq!(won(&t, P0), wins, "{n} {lose_one}");
    }
}

#[test]
fn near_death_experience_intervening_if_and_two_headed_giant() {
    cr!("603.4", "810.9", "104.2b");
    ruling!(
        "Near-Death Experience",
        "This ability has an “intervening ‘if’ clause.” That means (1) the ability triggers only if you have exactly 1 life as your upkeep begins, and (2) the ability does nothing if your life total is anything other than 1 by the time it resolves."
    );
    ruling!(
        "Near-Death Experience",
        "In a Two-Headed Giant game, anything that cares about your life total checks your team’s life total. You’ll win the game if your team has exactly 1 life."
    );
    supported("Near-Death Experience");
    // (life at the upkeep, life gained in response, triggers, wins)
    for (life, gain, triggers, wins) in [(1, 0, 1, true), (1, 3, 1, false), (2, 0, 0, false)] {
        let mut t = TestGame::new(2);
        add(&mut t, P0, "Near-Death Experience");
        t.set_step(P1, Step::PrecombatMain);
        t.g.players[0].life = life;
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_from(&t, "exactly 1 life"), triggers);
        if gain > 0 {
            t.g.gain_life(P0, gain);
        }
        t.resolve_all();
        assert_eq!(won(&t, P0), wins, "{life} {gain}");
    }
    // Two-Headed Giant: P0's team (P0 and P1) has exactly 1 life.
    let mut t = crate::r_p050_common::two_headed_giant();
    add(&mut t, P0, "Near-Death Experience");
    t.g.players[0].life = 1;
    t.g.players[1].life = 1;
    assert_eq!(t.life(P1), 1);
    t.set_step(P2, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_from(&t, "exactly 1 life"), 1);
    t.resolve_all();
    assert!(won(&t, P0) && won(&t, P1));
    assert!(t.has_lost(P2) && t.has_lost(P3));
}

#[test]
fn coalition_victory_counts_every_type_and_color_of_each_permanent() {
    cr!("104.2b", "205.3i", "105.2");
    ruling!(
        "Coalition Victory",
        "If a single land has multiple types and/or a single creature is multiple colors, it will count all those types and/or colors."
    );
    supported("Coalition Victory");
    for forest in [true, false] {
        let mut t = TestGame::new(2);
        add(&mut t, P0, "Tundra");
        add(&mut t, P0, "Badlands");
        if forest {
            add(&mut t, P0, "Forest");
        }
        add(&mut t, P0, "Sliver Queen");
        cast_resolve(&mut t, P0, "Coalition Victory", "3WUBRG", &[]);
        assert_eq!(won(&t, P0), forest);
    }
}

// ---------------------------------------------------------------------------------------
// "As an additional cost to cast this spell, sacrifice a creature or pay {N}."
// ---------------------------------------------------------------------------------------

#[test]
fn bayou_groff_and_lash_of_the_balrog_sacrifice_as_a_cost_no_response_possible() {
    cr!("601.2h", "601.2i", "117.1c");
    ruling!(
        "Bayou Groff",
        "Players can respond to Bayou Groff only after it’s been cast and all its costs have been paid."
    );
    ruling!(
        "Lash of the Balrog",
        "Players can respond to Lash of the Balrog only after it's been cast and all its costs have been paid."
    );
    supported("Bayou Groff");
    supported("Lash of the Balrog");
    for (name, cost, target) in [("Bayou Groff", "1G", false), ("Lash of the Balrog", "B", true)] {
        let mut t = TestGame::new(2);
        let bears = add(&mut t, P0, "Grizzly Bears");
        let giant = add(&mut t, P1, "Hill Giant");
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        let c = t.hand(P0, name);
        mana(&mut t, P0, cost);
        if target {
            t.answer_targets(P0, &[obj(giant)]);
        }
        // Nothing is asked of P1 while P0 casts it; once cast, the Bears are already gone.
        let from = t.asked().len();
        let spell = t.cast(P0, c).go();
        assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0), "{name}");
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert!(!t.on_battlefield(bears));
        assert_eq!(t.g.stack, vec![spell]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), !target);
    }
}

#[test]
fn bayou_groff_and_lash_of_the_balrog_exactly_one_of_the_costs() {
    cr!("601.2b", "601.2f", "601.2h");
    ruling!(
        "Bayou Groff",
        "You must sacrifice exactly one creature or pay an extra {3} to cast this spell; you can’t cast it without paying one of those costs, and you can’t pay either multiple times."
    );
    ruling!(
        "Lash of the Balrog",
        "You must sacrifice exactly one creature or pay an extra {4} to cast this spell; you can't cast it without paying one of those costs, and you can't pay either multiple times."
    );
    for (name, cost, extra, target) in [
        ("Bayou Groff", "1G", "3", false),
        ("Lash of the Balrog", "B", "4", true),
    ] {
        // Two creatures, sacrificing: exactly one goes, no extra mana is spent.
        let mut t = TestGame::new(2);
        let giant = add(&mut t, P1, "Hill Giant");
        add(&mut t, P0, "Grizzly Bears");
        add(&mut t, P0, "Llanowar Elves");
        let c = t.hand(P0, name);
        mana(&mut t, P0, cost);
        mana(&mut t, P0, extra);
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        if target {
            t.answer_targets(P0, &[obj(giant)]);
        }
        t.cast(P0, c).go();
        assert_eq!(t.graveyard_size(P0), 1, "{name}");
        assert_eq!(
            t.g.players[0].mana_pool.total() as u32,
            extra.parse::<u32>().unwrap(),
            "{name}"
        );
        // Paying: no creature goes.
        let mut t = TestGame::new(2);
        let giant = add(&mut t, P1, "Hill Giant");
        let bears = add(&mut t, P0, "Grizzly Bears");
        let c = t.hand(P0, name);
        mana(&mut t, P0, cost);
        mana(&mut t, P0, extra);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        if target {
            t.answer_targets(P0, &[obj(giant)]);
        }
        t.cast(P0, c).go();
        assert!(t.on_battlefield(bears), "{name}");
        assert_eq!(t.g.players[0].mana_pool.total(), 0, "{name}");
        // Neither possible: it can't be cast.
        let mut t = TestGame::new(2);
        let giant = add(&mut t, P1, "Hill Giant");
        let c = t.hand(P0, name);
        mana(&mut t, P0, cost);
        if target {
            t.answer_targets(P0, &[obj(giant)]);
        }
        assert!(t.cast(P0, c).try_go().is_err(), "{name}");
        assert!(t.in_hand(P0, name));
    }
}

// ---------------------------------------------------------------------------------------
// Crude Rampart: "Defender. Morph {4}{W}."
// ---------------------------------------------------------------------------------------

#[test]
fn crude_rampart_turned_face_up_while_attacking_stays_attacking() {
    cr!("708.8", "506.4", "702.3b");
    ruling!(
        "Crude Rampart",
        "If it attacks while face down and is turned face up during combat, this will not remove it from combat."
    );
    supported("Crude Rampart");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Crude Rampart");
    mana(&mut t, P0, "3");
    t.cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let r = t.g.current(card);
    t.g.objects[r.0 as usize].summoning_sick = false;
    crate::r_s01_common::attack_with(&mut t, &[(r, pl(P1))]);
    mana(&mut t, P0, "4W");
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: r }))
        .expect("turn face up");
    t.g.flush_events();
    let r = t.g.current(r);
    assert!(!t.obj_now(r).face_down);
    assert!(t.obj_now(r).has_keyword(KeywordKind::Defender));
    assert!(t.g.is_attacking(r));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}
