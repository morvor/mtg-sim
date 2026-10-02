//! Rulings batch P146 — phrases compiled for this batch: the intervening-if condition "if
//! you gained or lost life this turn" (Starlit Soothsayer, Star Charter; CR 603.4) and the
//! trigger "whenever one or more cards leave your graveyard during your turn" (Kheru
//! Goldkeeper, Attuned Hunter, Thran Vigil, Kishla Skimmer; CR 603.10a), with every card
//! the new phrases made compile.

use crate::r_p146_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to P0's end step; returns the triggered abilities put on the stack then.
fn into_end_step(t: &mut TestGame) -> usize {
    t.advance_to(P0, Step::End);
    t.settle();
    stacked_triggers(t)
}

#[test]
fn starlit_soothsayer_needs_life_gained_or_lost_before_the_end_step() {
    cr!("603.4", "513.1");
    ruling!(
        "Starlit Soothsayer",
        "If you haven’t gained or lost life during the turn when your end step begins, Starlit Soothsayer’s last ability won’t trigger at all."
    );
    supported("Starlit Soothsayer");
    // "At the beginning of your end step, if you gained or lost life this turn, surveil
    // 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlit Soothsayer");
    assert_eq!(into_end_step(&mut t), 0);
    // Gaining life during the end step is too late.
    gain_life(&mut t, P0, 2);
    assert_eq!(stacked_triggers(&t), 0);
    // Losing life earlier in the turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlit Soothsayer");
    lose_life(&mut t, P0, 1);
    assert_eq!(into_end_step(&mut t), 1);
    let from = n_asked(&t);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Surveil { .. })));
    // Another player's life changes don't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlit Soothsayer");
    lose_life(&mut t, P1, 1);
    assert_eq!(into_end_step(&mut t), 0);
}

#[test]
fn starlit_soothsayer_cares_about_gaining_or_losing_not_the_total() {
    cr!("603.4", "119.3");
    ruling!(
        "Starlit Soothsayer",
        "Starlit Soothsayer’s last ability cares whether you gained or lost life this turn, not how your life total changed."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlit Soothsayer");
    gain_life(&mut t, P0, 2);
    lose_life(&mut t, P0, 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(into_end_step(&mut t), 1);
    // Gaining only.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlit Soothsayer");
    gain_life(&mut t, P0, 1);
    assert_eq!(into_end_step(&mut t), 1);
}

#[test]
fn star_charter_needs_life_gained_or_lost_this_turn() {
    cr!("603.4", "513.1", "701.20a");
    ruling!(
        "Star Charter",
        "If you haven’t gained or lost life during the turn when your end step begins, Star Charter’s last ability won’t trigger at all."
    );
    ruling!(
        "Star Charter",
        "Star Charter’s last ability cares whether you gained or lost life this turn, not how your life total changed."
    );
    supported("Star Charter");
    // "At the beginning of your end step, if you gained or lost life this turn, look at
    // the top four cards of your library. You may reveal a creature card with power 3 or
    // less from among them and put it into your hand. Put the rest on the bottom of your
    // library in a random order."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Star Charter");
    assert_eq!(into_end_step(&mut t), 0);
    gain_life(&mut t, P0, 1);
    assert_eq!(stacked_triggers(&t), 0);
    // Gained 2 and lost 2: it triggers; the Hill Giant (power 3) is taken, the Craw Wurm
    // (power 6) isn't eligible.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Star Charter");
    let top = stack_library(
        &mut t,
        P0,
        &["Craw Wurm", "Hill Giant", "Lightning Bolt", "Forest"],
    );
    gain_life(&mut t, P0, 2);
    lose_life(&mut t, P0, 2);
    assert_eq!(into_end_step(&mut t), 1);
    t.answer_choose(P0, &[obj(top[1])]);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(top[1])), Zone::Hand(P0));
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.contains(&obj(t.g.current(top[0]))));
    // The other three are now the bottom three cards.
    let lib = &t.g.player(P0).library;
    let bottom: Vec<ObjectId> = lib[..3].to_vec();
    for i in [0, 2, 3] {
        assert!(bottom.contains(&t.g.current(top[i])));
    }
}

#[test]
fn kheru_goldkeeper_triggers_once_per_event_and_only_on_your_turn() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Kheru Goldkeeper",
        "If multiple cards leave your graveyard at the same time, Kheru Goldkeeper’s triggered ability will trigger only once."
    );
    supported("Kheru Goldkeeper");
    // "Whenever one or more cards leave your graveyard during your turn, create a Treasure
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kheru Goldkeeper");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Forest");
    exile_together(&mut t, &[a, b]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    // Another event: another Treasure.
    let c = t.graveyard(P0, "Hill Giant");
    move_to(&mut t, c, Zone::Exile);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 2);
    // During the opponent's turn: nothing. Nor for the opponent's graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kheru Goldkeeper");
    t.set_step(P1, Step::PrecombatMain);
    let a = t.graveyard(P0, "Grizzly Bears");
    move_to(&mut t, a, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.graveyard(P1, "Grizzly Bears");
    move_to(&mut t, b, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
}

#[test]
fn kheru_goldkeeper_renew() {
    cr!("602.5d", "113.6j", "117.3b");
    ruling!(
        "Kheru Goldkeeper",
        "If a card with a renew ability is put into your graveyard during your turn, you can activate that ability if it’s legal to do so before any other player can take any actions."
    );
    // "Renew — {2}{B}{G}{U}, Exile this card from your graveyard: Put two +1/+1 counters
    // and a flying counter on target creature. Activate only as a sorcery." Exiling it
    // from the graveyard during P0's turn also makes another Goldkeeper's ability
    // trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kheru Goldkeeper");
    let k = t.battlefield(P0, "Kheru Goldkeeper");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, k);
    let card = t.g.current(k);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // P0 has priority first, in their main phase with an empty stack.
    assert!(can_activate(&mut t, P0, card));
    t.activate(P0, card, 0, &[obj(bears)]).unwrap();
    assert_eq!(t.zone(card), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
    assert_eq!(treasures(&t, P0), 1);
    // Not as an instant.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.battlefield(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Kheru Goldkeeper");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P0, card));
}

#[test]
fn attuned_hunter_triggers_once_per_event_on_your_turn() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Attuned Hunter",
        "If multiple cards leave your graveyard at the same time, Attuned Hunter’s last ability will trigger only once."
    );
    supported("Attuned Hunter");
    // Trample; "Whenever one or more cards leave your graveyard during your turn, put a
    // +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Attuned Hunter");
    assert!(t.obj(h).has_keyword(KeywordKind::Trample));
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Forest");
    exile_together(&mut t, &[a, b]);
    t.resolve_all();
    assert_eq!(t.counters(h, "+1/+1"), 1);
    t.set_step(P1, Step::PrecombatMain);
    let c = t.graveyard(P0, "Forest");
    move_to(&mut t, c, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.counters(h, "+1/+1"), 1);
}

#[test]
fn thran_vigil_artifact_or_creature_cards_on_your_turn() {
    cr!("603.2c", "603.10a", "115.1");
    supported("Thran Vigil");
    // "Whenever one or more artifact and/or creature cards leave your graveyard during
    // your turn, put a +1/+1 counter on target creature you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thran Vigil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A land card: no.
    let land = t.graveyard(P0, "Forest");
    move_to(&mut t, land, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
    // An artifact card and a creature card together: once.
    let a = t.graveyard(P0, "Ornithopter");
    let c = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    exile_together(&mut t, &[a, c]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // On the opponent's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    let c = t.graveyard(P0, "Hill Giant");
    move_to(&mut t, c, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
}

#[test]
fn kishla_skimmer_once_each_turn_on_your_turn() {
    cr!("603.2c", "603.10a");
    supported("Kishla Skimmer");
    // Flying; "Whenever a card leaves your graveyard during your turn, draw a card. This
    // ability triggers only once each turn."
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kishla Skimmer");
    assert!(t.obj(k).has_keyword(KeywordKind::Flying));
    // During the opponent's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    let a = t.graveyard(P0, "Forest");
    move_to(&mut t, a, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
    // On P0's turn: once, even for two cards and a second event.
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    let b = t.graveyard(P0, "Forest");
    let c = t.graveyard(P0, "Forest");
    exile_together(&mut t, &[b, c]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    let d = t.graveyard(P0, "Forest");
    move_to(&mut t, d, Zone::Exile);
    assert_eq!(stacked_triggers(&t), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn prosper_pact_boon_triggers_for_any_card_played_from_exile() {
    cr!("603.2", "601.2i", "305.1", "715.3d");
    ruling!(
        "Prosper, Tome-Bound",
        "The Pact Boon ability triggers whenever you play any cards from exile, not just those exiled with the Mystic Arcanum ability."
    );
    supported("Prosper, Tome-Bound");
    // "Pact Boon — Whenever you play a card from exile, create a Treasure token."
    // Casting an adventurer card from exile after its Adventure.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Prosper, Tome-Bound");
    assert!(t.obj(p).has_keyword(KeywordKind::Deathtouch));
    t.lands(P0, "Mountain", 4);
    let m = t.hand(P0, "Merchant of the Vale // Haggle");
    t.cast(P0, m)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.answer_yes(P0, false);
    t.resolve_all();
    // Cast from hand: no Treasure.
    assert_eq!(treasures(&t, P0), 0);
    let m = t.g.current(m);
    assert_eq!(t.zone(m), Zone::Exile);
    t.cast(P0, m).go();
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.named_on_battlefield("Merchant of the Vale").len(), 1);
    // A card cast from the hand: nothing.
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn prosper_mystic_arcanum_and_playing_a_land_from_exile() {
    cr!("603.2", "305.1", "611.2a");
    // "Mystic Arcanum — At the beginning of your end step, exile the top card of your
    // library. Until the end of your next turn, you may play that card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prosper, Tome-Bound");
    let forest = t.library_top(P0, "Forest");
    end_step(&mut t, P0);
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), Zone::Exile);
    // Not during the opponent's turn (it's a land); in P0's next turn it's played, and
    // Pact Boon triggers.
    t.advance_to(P0, Step::PrecombatMain);
    t.play_land(P0, forest).unwrap();
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    // A land played from the hand: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prosper, Tome-Bound");
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).unwrap();
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
    // The permission lasts until the end of P0's next turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prosper, Tome-Bound");
    let bolt = t.library_top(P0, "Lightning Bolt");
    end_step(&mut t, P0);
    let bolt = t.g.current(bolt);
    t.lands(P0, "Mountain", 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(can_cast(
        &mut t,
        P0,
        bolt,
        mtg_engine::object::CastMethod::Normal
    ));
    t.advance_to(P0, Step::End);
    assert!(can_cast(
        &mut t,
        P0,
        bolt,
        mtg_engine::object::CastMethod::Normal
    ));
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_cast(
        &mut t,
        P0,
        bolt,
        mtg_engine::object::CastMethod::Normal
    ));
}

const SIDEQUEST: &str = "Sidequest: Hunt the Mark // Yiazmat, Ultimate Mark";

#[test]
fn sidequest_hunt_the_mark_checks_as_the_end_step_begins() {
    cr!("603.4", "513.1", "700.4");
    ruling!(
        "Sidequest: Hunt the Mark // Yiazmat, Ultimate Mark",
        "Sidequest: Hunt the Mark's last ability checks at the moment it would trigger to see if a creature died under an opponent's control this turn."
    );
    supported(SIDEQUEST);
    // "At the beginning of your end step, if a creature died under an opponent's control
    // this turn, create a Treasure token. Then if you control three or more Treasures,
    // transform this enchantment."
    // No creature died: no trigger, and a creature dying during the end step is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, SIDEQUEST);
    assert_eq!(into_end_step(&mut t), 0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(stacked_triggers(&t), 0);
    // Only P0's own creature died: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, SIDEQUEST);
    let mine = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, mine);
    assert_eq!(into_end_step(&mut t), 0);
    // An opponent's creature died earlier: a Treasure (one Treasure: no transformation).
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, SIDEQUEST);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    assert_eq!(into_end_step(&mut t), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.obj(s).chars.name, "Sidequest: Hunt the Mark");
}

#[test]
fn sidequest_hunt_the_mark_enters_destroys_and_transforms_into_yiazmat() {
    cr!("701.27a", "712.8e", "608.2c");
    // "When this enchantment enters, destroy up to one target creature."
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    let s = t.enter(P0, SIDEQUEST);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // With two Treasures already, the third makes it transform.
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Treasure");
    end_step(&mut t, P0);
    assert_eq!(treasures(&t, P0), 3);
    assert_eq!(t.obj(s).chars.name, "Yiazmat, Ultimate Mark");
    assert!(t.obj(s).is(mtg_engine::types::CardType::Creature));
    // Yiazmat: "{1}{B}, Sacrifice another creature or artifact: Yiazmat gains
    // indestructible until end of turn. Tap it."
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let tr = tokens(&t, P0)[0];
    t.answer_choose(P0, &[obj(tr)]);
    activate_resolve(&mut t, P0, s, 0, &[]);
    assert!(!t.on_battlefield(tr));
    assert!(t.obj(s).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj(s).tapped);
}
