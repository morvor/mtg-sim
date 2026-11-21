//! roundtrip-gaps-2 (docs/agents/items/roundtrip-gaps-2.md): cards whose rendered
//! abilities differ in wording from their Oracle text, checked in play.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// How many of `p`'s lands are tapped.
fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| {
            let o = t.obj_now(**o);
            o.controller == p && o.chars.card_types.contains(CardType::Land) && o.tapped
        })
        .count()
}

#[test]
fn dargo_counts_sacrifices_this_way_and_other_ones_this_turn_once_each() {
    cr!("601.2b", "601.2f", "601.2h");
    // "As an additional cost to cast this spell, you may sacrifice any number of artifacts
    // and/or creatures. This spell costs {2} less to cast for each permanent sacrificed
    // this way and {2} less to cast for each other artifact or creature you've sacrificed
    // this turn."
    let mut t = TestGame::new(2);
    let early = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 7);
    t.g.sacrifice(early, P0);
    t.settle();
    let dargo = t.hand(P0, "Dargo, the Shipwrecker");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    if let Err(e) = t.cast_with(P0, dargo, &[]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    // {6}{R} minus {4} (two sacrificed this way) minus {2} (one other this turn).
    assert_eq!(tapped_lands(&t, P0), 1);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn phabine_counts_each_revealed_card_once() {
    cr!("701.20a");
    // "each player reveals the top card of their library. For each land card revealed this
    // way, you create a 1/1 green and white Citizen creature token. Then creatures you
    // control get +1/+1 until end of turn for each nonland card revealed this way."
    let mut t = TestGame::new(2);
    let phabine = t.battlefield(P0, "Phabine, Boss's Confidant");
    t.library_top(P0, "Forest");
    t.library_top(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let (p, q) = t.pt(phabine);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Citizen Token").len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(phabine), (p + 1, q + 1));
}

#[test]
fn devouring_rage_adds_three_power_per_spirit_sacrificed() {
    cr!("601.2b", "601.2h");
    // "As an additional cost to cast this spell, you may sacrifice any number of Spirits.
    // Target creature gets +3/+0 until end of turn. For each Spirit sacrificed this way,
    // that creature gets an additional +3/+0 until end of turn."
    let mut t = TestGame::new(2);
    let s1 = t.battlefield(P0, "Kami of Ancient Law");
    let s2 = t.battlefield(P0, "Spectral Sailor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let rage = t.hand(P0, "Devouring Rage");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    if let Err(e) = t.cast_with(P0, rage, &[Entity::Object(bears)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(!t.on_battlefield(s1) && !t.on_battlefield(s2));
    assert_eq!(t.pt(bears), (2 + 9, 2));
}

#[test]
fn ixhel_lets_you_play_each_poisoned_opponents_exiled_card() {
    cr!("608.2c");
    // "At the beginning of your end step, each opponent who has three or more poison
    // counters exiles the top card of their library face down. You may look at and play
    // those cards for as long as they remain exiled, ..." (a condition for each opponent).
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Ixhel, Scion of Atraxa");
    t.g.player_mut(P1).counters.insert("poison".into(), 3);
    t.g.player_mut(P2).counters.insert("poison".into(), 3);
    t.library_top(P1, "Gut Shot");
    t.library_top(P2, "Gut Shot");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to_step(Step::End);
    t.resolve_all();
    // (Face down: their names are hidden.)
    let shots: Vec<ObjectId> = t.g.exile.iter().copied().collect();
    assert_eq!(shots.len(), 2);
    assert_eq!(shots.len(), 2, "{}", t.dump_log());
    for (i, s) in shots.into_iter().enumerate() {
        t.g.turn.priority = Some(P0);
        if let Err(e) = t.cast(P0, s).target(Entity::Player(P1)).try_go() {
            panic!("cast {i}: {e:?}\n{}", t.dump_log());
        }
        t.resolve_all();
    }
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ixhel_skips_an_opponent_without_three_poison_counters() {
    cr!("608.2c");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Ixhel, Scion of Atraxa");
    t.g.player_mut(P1).counters.insert("poison".into(), 3);
    t.g.player_mut(P2).counters.insert("poison".into(), 2);
    t.library_top(P1, "Gut Shot");
    t.library_top(P2, "Gut Shot");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1, "{}", t.dump_log());
    // P1's card is playable even though P2 (after P1) exiled nothing.
    let shot = t.g.exile[0];
    t.g.turn.priority = Some(P0);
    if let Err(e) = t.cast(P0, shot).target(Entity::Player(P1)).try_go() {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
}
