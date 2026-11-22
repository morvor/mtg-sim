//! Item `rulings-residue-misc-1` (pass C) — cards chosen, named and exiled: Grimoire Thief
//! and Jester's Scepter countering spells by name (CR 201.2a, 607.2a), Forgotten Lore and
//! Shrouded Lore (each spell's choices are its own, CR 608.2c), Cursed Scroll with an empty
//! hand, Stronghold Gambit's revealed cards (CR 701.20a), Mogg Assassin's two targets
//! (CR 601.2c), Beastie Beatdown's required targets (CR 601.2c) and Amy Pond's suspended
//! cards (CR 702.62b).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Grimoire Thief, controlled by P0, exiles the top three cards of P1's library (`top`,
/// top first). Returns the Thief and the exiled cards.
fn thief(t: &mut TestGame, top: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    let thief = t.battlefield(P0, "Grimoire Thief");
    let cards = stack_library(t, P1, top);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.tap(thief);
    t.g.flush_events();
    t.resolve_all();
    let cards: Vec<ObjectId> = cards.iter().map(|c| t.g.current(*c)).collect();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(t.obj_now(*c).face_down);
    }
    (thief, cards)
}

#[test]
fn grimoire_thief_counters_every_spell_with_those_names_whoever_controls_it() {
    cr!("201.2a", "607.2a", "701.6a");
    ruling!(
        "Grimoire Thief",
        "The last ability counters all spells on the stack with any of the appropriate names, no matter who controls them."
    );
    supported("Grimoire Thief");
    let mut t = TestGame::new(3);
    let (thief, _) = thief(&mut t, &["Shock", "Opt", "Island"]);
    t.advance_to(P0, Step::PrecombatMain);
    // P0's own Shock, P1's Shock and P2's Opt are countered; P2's Lightning Bolt isn't.
    let mine = cast_new(&mut t, P0, "Shock", &[pl(P2)]);
    t.g.turn.priority = Some(P1);
    let theirs = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.g.turn.priority = Some(P2);
    let opt = cast_new(&mut t, P2, "Opt", &[]);
    let bolt = cast_new(&mut t, P2, "Lightning Bolt", &[pl(P0)]);
    t.lands(P0, "Island", 1);
    t.g.turn.priority = Some(P0);
    t.activate(P0, thief, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!resolved(&t, mine) && !resolved(&t, theirs) && !resolved(&t, opt));
    assert!(resolved(&t, bolt));
    assert_eq!((t.life(P0), t.life(P2)), (17, 20));
}

#[test]
fn grimoire_thief_turns_face_up_only_its_own_cards() {
    cr!("607.2a", "406.3");
    ruling!(
        "Grimoire Thief",
        "The only cards you turn face up are the cards exiled with the Grimoire Thief that was sacrificed."
    );
    let mut t = TestGame::new(2);
    let (a, first) = thief(&mut t, &["Shock", "Island", "Island"]);
    let (_b, second) = thief(&mut t, &["Opt", "Forest", "Forest"]);
    t.advance_to(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    let opt = cast_new(&mut t, P1, "Opt", &[]);
    t.lands(P0, "Island", 1);
    t.g.turn.priority = Some(P0);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(first.iter().all(|c| !t.obj_now(*c).face_down));
    assert!(second.iter().all(|c| t.obj_now(*c).face_down));
    // The other Thief's Opt isn't one of the names.
    assert!(resolved(&t, opt));
}

#[test]
fn jesters_scepter_may_pay_with_any_card_and_target_any_spell() {
    cr!("601.2c", "602.2b", "608.2c");
    ruling!(
        "Jester's Scepter",
        "When Jester’s Scepter’s second ability is activated, any of the exiled cards may be put into its owner’s graveyard, and any spell may be targeted, whether or not the card and the spell have the same name. If they don’t have the same name, the effect does nothing. The spell’s controller and the owner of the card put into a graveyard this way may be different."
    );
    supported("Jester's Scepter");
    let mut t = TestGame::new(3);
    let cards = stack_library(
        &mut t,
        P1,
        &["Island", "Shock", "Forest", "Forest", "Forest"],
    );
    t.answer_targets(P0, &[pl(P1)]);
    let s = t.enter(P0, "Jester's Scepter");
    t.g.flush_events();
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    // P2's Opt, paid for with P1's Island: no shared name, Opt resolves; the Island still
    // goes to P1's graveyard.
    t.g.turn.priority = Some(P2);
    let opt = cast_new(&mut t, P2, "Opt", &[]);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(t.g.current(cards[0]))]);
    t.g.turn.priority = Some(P0);
    t.activate(P0, s, 0, &[obj(opt)]).unwrap();
    t.resolve_all();
    assert!(resolved(&t, opt));
    assert_eq!(t.zone(cards[0]), Zone::Graveyard(P1));
    // P2's Shock, paid for with P1's Shock: countered.
    t.g.untap(t.g.current(s));
    t.g.turn.priority = Some(P2);
    let shock = cast_new(&mut t, P2, "Shock", &[pl(P0)]);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(t.g.current(cards[1]))]);
    t.g.turn.priority = Some(P0);
    t.activate(P0, s, 0, &[obj(shock)]).unwrap();
    t.resolve_all();
    assert!(!resolved(&t, shock));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn forgotten_and_shrouded_lore_know_only_their_own_choices() {
    cr!("608.2c", "400.7");
    ruling!(
        "Forgotten Lore",
        "The spell has no knowledge of cards chosen for other spells named Forgotten Lore."
    );
    ruling!(
        "Shrouded Lore",
        "The spell has no knowledge of cards chosen for other spells named Shrouded Lore."
    );
    for (name, land) in [("Forgotten Lore", "Forest"), ("Shrouded Lore", "Swamp")] {
        supported(name);
        let mut t = TestGame::new(2);
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Craw Wurm");
        t.advance_to(P0, Step::PrecombatMain);
        // The first spell: P1 chooses the Bears, P0 pays, P1 chooses the Wurm, P0 stops:
        // the Wurm goes to P0's hand.
        t.lands(P0, land, 2);
        let first = t.hand(P0, name);
        t.answer_choose(P1, &[obj(a)]);
        t.answer_yes(P0, true);
        t.answer_choose(P1, &[obj(b)]);
        t.answer_yes(P0, false);
        t.cast(P0, first).target(P1).go();
        t.resolve_all();
        assert!(t.in_hand(P0, "Craw Wurm"), "{name}");
        // The second spell: the Bears, chosen for the first one, may be chosen again.
        t.graveyard(P0, "Hill Giant");
        t.lands(P0, land, 1);
        let second = t.hand(P0, name);
        let from = t.asked().len();
        t.answer_choose(P1, &[obj(a)]);
        t.answer_yes(P0, false);
        t.cast(P0, second).target(P1).go();
        t.resolve_all();
        let offered = asked_since(&t, from).into_iter().any(|(p, d)| {
            p == P1
                && matches!(&d, Decision::ChooseEntities { candidates, .. }
                    if candidates.contains(&obj(a)))
        });
        assert!(offered, "{name}: the Bears can be chosen again");
        assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
    }
}

#[test]
fn cursed_scroll_with_no_cards_in_hand_names_a_card_and_deals_no_damage() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Cursed Scroll",
        "If you have no cards in hand, you still have to name a card. Then you fail to reveal one (since there aren’t any). Since the named card wasn’t revealed, no damage is dealt."
    );
    supported("Cursed Scroll");
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Cursed Scroll");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 3);
    let n = t.hand_size(P0);
    for _ in 0..n {
        let c = t.g.players[0].hand[0];
        move_to(&mut t, c, Zone::Graveyard(P0));
    }
    assert_eq!(t.hand_size(P0), 0);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Name, Answer::Text("Shock".into()));
    t.activate(P0, scroll, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::NameCard { .. }
    )));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn stronghold_gambit_leaves_the_other_revealed_cards_in_hand_unrevealed() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Stronghold Gambit",
        "Creature cards which are revealed but are not the lowest cost, or any non-creature cards revealed remain in their owners’ hands and stop being revealed."
    );
    ruling!(
        "Stronghold Gambit",
        "You don’t have to choose a creature card, but choosing a non creature does nothing except maybe bluff your opponent."
    );
    supported("Stronghold Gambit");
    let mut t = TestGame::new(3);
    let bolt = t.hand(P0, "Lightning Bolt");
    let elves = t.hand(P1, "Llanowar Elves");
    let bears = t.hand(P2, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Stronghold Gambit");
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_choose(P1, &[obj(elves)]);
    t.answer_choose(P2, &[obj(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    // P0's Lightning Bolt does nothing; P1's Elves (the lowest creature) enter; P2's
    // Bears stay in hand, and neither the Bolt nor the Bears are revealed any more.
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_hand(P0, "Lightning Bolt") && t.in_hand(P2, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(!mtg_engine::reveal::is_revealed(&t.g, bolt));
    assert!(!mtg_engine::reveal::is_revealed(&t.g, bears));
}

#[test]
fn mogg_assassins_targets_are_both_chosen_before_the_flip_you_first() {
    cr!("601.2c", "602.2b", "705.1");
    ruling!(
        "Mogg Assassin",
        "You and your opponent both pick the target creatures on announcing, which is before the coin is flipped. You pick first."
    );
    supported("Mogg Assassin");
    let mut t = TestGame::new(2);
    let mogg = t.battlefield(P0, "Mogg Assassin");
    let mine = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P0, "Llanowar Elves");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P1, &[obj(theirs)]);
    let from = t.asked().len();
    let ab = t.activate(P0, mogg, 0, &[obj(mine)]).expect("activate");
    // Both targets are on the stack before anything resolves, and no coin was flipped.
    let targets = crate::r_s25_common::targets_of(&t, ab.expect("on the stack"));
    assert_eq!(targets, vec![obj(mine), obj(theirs)]);
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseTargets { .. }
    )));
    assert!(crate::r_p017_common::flip_results(&t).is_empty());
    t.g.dice.loaded_coins.push_back(true);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
}

#[test]
fn beastie_beatdown_needs_both_targets() {
    cr!("601.2c", "115.1a");
    ruling!(
        "Beastie Beatdown",
        "You can't cast Beastie Beatdown unless you choose a creature you control and a creature an opponent controls as targets."
    );
    supported("Beastie Beatdown");
    for theirs in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        if theirs {
            t.battlefield(P1, "Llanowar Elves");
        }
        t.advance_to(P0, Step::PrecombatMain);
        lands_for_cost(&mut t, P0, "Beastie Beatdown");
        let c = t.hand(P0, "Beastie Beatdown");
        assert_eq!(
            can_cast(&mut t, P0, c, mtg_engine::object::CastMethod::Normal),
            theirs
        );
    }
}

#[test]
fn amy_pond_chooses_only_a_suspended_card() {
    cr!("702.62b", "122.1");
    ruling!(
        "Amy Pond",
        "A suspended card is a card that (1) has suspend, (2) is in exile, and (3) has one or more time counters on it."
    );
    supported("Amy Pond");
    let mut t = TestGame::new(2);
    let amy = t.battlefield(P0, "Amy Pond");
    // Suspended: Rift Bolt in exile with time counters. Not: a Rift Bolt in exile without
    // any, a Grizzly Bears in exile with a time counter, a Rift Bolt in hand.
    let suspended = t.exile(P0, "Rift Bolt");
    crate::r_p108_common::put_counters(&mut t, suspended, mtg_engine::types::counters::TIME, 3);
    let bare = t.exile(P0, "Rift Bolt");
    let bears = t.exile(P0, "Grizzly Bears");
    crate::r_p108_common::put_counters(&mut t, bears, mtg_engine::types::counters::TIME, 3);
    let in_hand = t.hand(P0, "Rift Bolt");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(suspended)]);
    t.advance_to(P0, Step::PrecombatMain);
    t.attack(&[(amy, pl(P1))], &[]);
    t.resolve_all();
    let candidates: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(candidates.contains(&obj(suspended)));
    for c in [bare, bears, in_hand] {
        assert!(!candidates.contains(&obj(c)));
    }
    assert_eq!(t.counters(suspended, mtg_engine::types::counters::TIME), 1);
}
