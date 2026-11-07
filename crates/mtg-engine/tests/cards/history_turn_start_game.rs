//! What things were like as the turn began and what happened earlier in the game
//! (`turn_start.rs`, `oracle/patterns/history_conditions_4.rs`): "if you were the monarch
//! as the turn began", "if you had no cards in hand at the beginning of this turn", "the
//! number of untapped lands they controlled at the beginning of this turn", "that
//! player's life total as the turn began", "half their starting life total", "this game",
//! and conditions joined by "and".

use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Ends `from`'s turn and goes to `step` of `to`'s next turn.
fn next_turn(t: &mut TestGame, from: PlayerId, to: PlayerId, step: Step) {
    t.set_step(from, Step::End);
    t.advance_to(to, step);
}

// --- as the turn began ------------------------------------------------------------------

#[test]
fn knights_of_the_black_rose_drains_when_you_were_the_monarch_as_the_turn_began() {
    cr!("725.1", "603.4");
    assert_supported("Knights of the Black Rose");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knights of the Black Rose");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.g.flush_events();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn knights_of_the_black_rose_does_nothing_if_you_became_the_monarch_this_turn() {
    cr!("725.1", "603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knights of the Black Rose");
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.g.flush_events();
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn mindstorm_crown_draws_if_your_hand_was_empty_as_the_turn_began() {
    cr!("603.4");
    ruling!("Mindstorm Crown", "doesn’t have an “intervening ‘if’ clause.”");
    assert_supported("Mindstorm Crown");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindstorm Crown");
    t.g.players[P0.idx()].hand.clear();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "a card drawn in the upkeep");
    assert_eq!(t.life(P0), 20);
    // Next turn it begins with cards in hand: 1 damage, no card.
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    let before = t.hand_size(P0);
    next_turn(&mut t, P1, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn power_surge_counts_the_lands_left_untapped_as_the_turn_began() {
    cr!("502.3", "603.4");
    assert_supported("Power Surge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Power Surge");
    let lands = t.lands(P1, "Mountain", 4);
    t.g.tap(lands[0]);
    t.g.flush_events();
    next_turn(&mut t, P0, P1, Step::Upkeep);
    t.resolve_all();
    // The tapped land untapped in the untap step, after the turn began.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sengir_gains_the_life_the_player_had_as_the_turn_began() {
    cr!("104.3a", "119.1");
    assert_supported("Sengir, the Dark Baron");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sengir, the Dark Baron");
    t.g.players[P2.idx()].life = 7;
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    t.g.lose_life(P2, 7);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.has_lost(P2));
    assert_eq!(t.life(P0), 27);
}

#[test]
fn game_over_costs_less_with_a_player_at_half_their_starting_life_or_less() {
    cr!("119.1", "601.2f");
    assert_supported("Game Over");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let go = t.hand(P0, "Game Over");
    t.g.players[P1.idx()].life = 11;
    assert!(t.cast_with(P0, go, &[]).is_err(), "11 is more than half of 20");
    t.g.players[P1.idx()].life = 10;
    t.cast_with(P0, go, &[]).unwrap();
}

#[test]
fn happily_ever_after_wins_with_all_three_conditions() {
    cr!("104.2b", "603.4");
    assert_supported("Happily Ever After");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Happily Ever After");
    for c in [
        "Savannah Lions",
        "Coral Merfolk",
        "Grizzly Bears",
        "Raging Goblin",
        "Scathe Zombies",
        "Ornithopter",
        "Forest",
    ] {
        t.battlefield(P0, c);
    }
    t.graveyard(P0, "Shock");
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.g.result.is_none(), "five card types only");
    t.graveyard(P0, "Divination");
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    t.g.players[P0.idx()].life = 19;
    next_turn(&mut t, P1, P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.g.result.is_none(), "less life than the starting life total");
    t.g.players[P0.idx()].life = 20;
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn angel_of_destiny_defeats_only_the_players_it_attacked() {
    cr!("104.3a", "508.1");
    assert_supported("Angel of Destiny");
    let mut t = TestGame::new(3);
    let angel = t.battlefield(P0, "Angel of Destiny");
    t.g.players[P0.idx()].life = 40;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(angel, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P2));
}

// --- this game ---------------------------------------------------------------------------

#[test]
fn gollum_drains_opponents_it_dealt_combat_damage_to_earlier_in_the_game() {
    cr!("510.2", "119.3");
    assert_supported("Gollum, Obsessed Stalker");
    let mut t = TestGame::new(3);
    let gollum = t.battlefield(P0, "Gollum, Obsessed Stalker");
    t.g.deal_damage(gollum, Entity::Player(P1), 1, true);
    t.g.deal_damage(gollum, Entity::Player(P2), 1, false);
    t.g.flush_events();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P2, Step::PrecombatMain);
    next_turn(&mut t, P2, P0, Step::PrecombatMain);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 19 - 3, "combat damage two turns ago");
    assert_eq!(t.life(P2), 19, "noncombat damage doesn't count");
}

#[test]
fn diseased_vermin_targets_only_an_opponent_it_dealt_damage_to_before() {
    cr!("115.1", "120.3");
    assert_supported("Diseased Vermin");
    let mut t = TestGame::new(3);
    let dv = t.battlefield(P0, "Diseased Vermin");
    t.g.deal_damage(dv, Entity::Player(P2), 1, true);
    t.g.flush_events();
    t.g.add_counters(Entity::Object(dv), "infection", 2, None);
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P2, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    next_turn(&mut t, P2, P0, Step::Upkeep);
    let infection = t.counters(dv, "infection") as i32;
    t.resolve_all();
    assert_eq!(t.life(P2), 19 - infection);
    let asked = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } if q == P0 => {
                Some(candidates)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(asked, vec![Entity::Player(P2)]);
}

/// Cycles `card` from `p`'s hand (paying with `p`'s lands).
fn cycle(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    t.g.recompute();
    let uid = t
        .g
        .obj(card)
        .chars
        .abilities
        .iter()
        .find(|a| a.text == "Cycling")
        .map(|a| a.uid)
        .expect("a cycling ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, card, uid).expect("cycling");
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn yidaro_comes_back_on_the_fourth_cycle_of_the_game() {
    cr!("702.29a", "603.4");
    assert_supported("Yidaro, Wandering Monster");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    for _ in 0..3 {
        let y = t.hand(P0, "Yidaro, Wandering Monster");
        cycle(&mut t, P0, y);
        for l in t.g.permanents_controlled_by(P0) {
            t.g.objects[l.0 as usize].tapped = false;
        }
    }
    assert!(t.named_on_battlefield("Yidaro, Wandering Monster").is_empty());
    // A later turn: the cycles of earlier turns count.
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::PrecombatMain);
    let y = t.hand(P0, "Yidaro, Wandering Monster");
    cycle(&mut t, P0, y);
    assert_eq!(t.named_on_battlefield("Yidaro, Wandering Monster").len(), 1);
}

#[test]
fn once_upon_a_time_is_free_as_the_first_spell_of_the_game() {
    cr!("118.9", "601.2b");
    assert_supported("Once Upon a Time");
    let mut t = TestGame::new(2);
    let ouat = t.hand(P0, "Once Upon a Time");
    let alts = alternatives(&t, P0, ouat);
    assert_eq!(alts.len(), 1, "free as no spell was cast this game");
    t.cast(P0, ouat).method(alts[0].clone()).go();
}

/// The alternative-cost ways of casting the card offered now.
fn alternatives(t: &TestGame, p: PlayerId, card: ObjectId) -> Vec<mtg_engine::object::CastMethod> {
    t.cast_options(p, card)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, mtg_engine::object::CastMethod::Alternative(_)))
        .collect()
}

#[test]
fn once_upon_a_time_isnt_free_after_a_spell_cast_in_an_earlier_turn() {
    cr!("118.9");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast_with(P0, shock, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::PrecombatMain);
    for l in t.g.permanents_controlled_by(P0) {
        t.g.objects[l.0 as usize].tapped = true;
    }
    let ouat = t.hand(P0, "Once Upon a Time");
    assert!(alternatives(&t, P0, ouat).is_empty(), "a spell was cast this game");
    assert!(matches!(t.zone(ouat), Zone::Hand(_)));
}

#[test]
fn frodo_draws_once_the_ring_has_tempted_you_twice_this_game() {
    cr!("701.54a", "701.54c");
    assert_supported("Frodo, Adventurous Hobbit");
    let mut t = TestGame::new(2);
    let frodo = t.battlefield(P0, "Frodo, Adventurous Hobbit");
    for turn in 0..2 {
        t.g.gain_life(P0, 3);
        t.g.flush_events();
        t.answer_choose(P0, &[Entity::Object(frodo)]);
        let before = t.hand_size(P0);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(frodo, Entity::Player(P1))], &[]);
        t.resolve_all();
        if turn == 0 {
            assert_eq!(t.hand_size(P0), before, "tempted once");
        } else {
            assert_eq!(t.hand_size(P0), before + 1, "tempted twice this game");
        }
        next_turn(&mut t, P0, P1, Step::PrecombatMain);
        next_turn(&mut t, P1, P0, Step::PrecombatMain);
    }
}

/// Compiles `text` as a creature card's rules text, returning the text left unsupported.
fn unsupported_of(text: &str) -> Vec<String> {
    use mtg_engine::oracle::{self, CompileContext};
    let tl = mtg_engine::types::TypeLine::parse("Creature — Test");
    let ctx = CompileContext {
        card_name: "Test Card",
        full_name: "Test Card",
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("1"),
        toughness: Some("1"),
    };
    oracle::compile(text, &ctx)
        .unsupported
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn this_game_queries_compile_only_for_events_kept_for_the_game() {
    // Spell casts, cycles and damage of earlier turns are kept; life gained isn't, so "if
    // you gained life this game" would read only this turn's events and is refused.
    assert!(
        unsupported_of("At the beginning of your end step, if you've cast two or more spells this game, draw a card.").is_empty()
    );
    assert!(
        !unsupported_of("At the beginning of your end step, if you gained life this game, draw a card.").is_empty()
    );
    assert!(
        unsupported_of("At the beginning of your end step, if you gained life this turn, draw a card.").is_empty()
    );
}

#[test]
fn knights_of_the_black_rose_drains_each_new_monarch_this_turn() {
    cr!("725.1", "603.4");
    ruling!("Knights of the Black Rose", "even if the most recent monarch was another player");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Knights of the Black Rose");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.g.flush_events();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    mtg_engine::designations::become_monarch(&mut t.g, P2);
    t.g.flush_events();
    t.resolve_all();
    // P2 was the most recent monarch, not you: you were the monarch as the turn began.
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn mindstorm_crown_counts_the_hand_as_the_turn_began_for_each_crown() {
    cr!("603.4");
    ruling!("Mindstorm Crown", "cares about the number of cards in your hand at the time the turn started");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindstorm Crown");
    t.battlefield(P0, "Mindstorm Crown");
    t.g.players[P0.idx()].hand.clear();
    next_turn(&mut t, P0, P1, Step::PrecombatMain);
    next_turn(&mut t, P1, P0, Step::Upkeep);
    // A card comes to hand before the abilities resolve: each still draws.
    t.hand(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 20);
}
