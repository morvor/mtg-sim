//! Rulings batch P014 — "catch up" effects that compare a player with the others: life
//! totals that become the lowest or highest one (CR 119.5, 810.9), spells and abilities
//! that check whether an opponent has more life, lands or creatures (on resolution, as
//! targeting requirements, as intervening "if" clauses, in multiplayer games), and
//! Verge Rangers' top card.

use crate::r_p014_common::*;
use crate::r_p038_common::{cast_only, cast_resolve, enter, tokens_of};
use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use rand::SeedableRng;

// ---------------------------------------------------------------------------
// Life totals that become a number
// ---------------------------------------------------------------------------

#[test]
fn repay_in_kind_makes_the_higher_player_lose_life() {
    cr!("119.5", "119.3");
    ruling!(
        "Repay in Kind",
        "For a player’s life total to become a certain number that’s lower than their current life total, what actually happens is that the player loses the appropriate amount of life."
    );
    // Exquisite Blood: "Whenever an opponent loses life, you gain that much life."
    supported("Exquisite Blood");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    set_life(&mut t, P1, 12);
    t.battlefield(P0, "Exquisite Blood");
    let from = t.g.turn_events.len();
    cast_resolve(&mut t, P0, "Repay in Kind", &[]);
    let changes = life_changes(&t, from);
    assert_eq!(changes[0], (P1, -7));
    assert!(changes.contains(&(P0, 7)), "{changes:?}");
    assert_eq!((t.life(P0), t.life(P1)), (12, 5));
}

#[test]
fn arbiter_of_knollridge_life_gain_triggers_lifegain_abilities() {
    cr!("119.5", "603.2");
    ruling!(
        "Arbiter of Knollridge",
        "Abilities that trigger when a player gains life may trigger as a result of this ability."
    );
    supported("Arbiter of Knollridge");
    // Ajani's Pridemate: "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 10);
    let cat = t.battlefield(P1, "Ajani's Pridemate");
    enter(&mut t, P0, "Arbiter of Knollridge");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.counters(cat, counters::PLUS1), 1);
}

#[test]
fn repay_in_kind_in_two_headed_giant_one_member_loses_the_difference() {
    cr!("810.9a", "810.9d", "810.9c");
    ruling!(
        "Repay in Kind",
        "In a Two-Headed Giant game, anything that cares about a player’s life total checks the life total of that player’s team."
    );
    supported("Repay in Kind");
    // One team has 13 life and the other 21.
    let mut t = thg();
    t.g.lose_life(P1, 17);
    t.g.lose_life(P3, 9);
    t.settle();
    assert_eq!((t.life(P0), t.life(P2)), (13, 21));
    t.answer_choose(P2, &[pl(P3)]);
    t.answer_choose(P0, &[pl(P1)]);
    t.set_step(P0, Step::PrecombatMain);
    let from = t.g.turn_events.len();
    cast_resolve(&mut t, P0, "Repay in Kind", &[]);
    assert_eq!(life_changes(&t, from), vec![(P3, -8)]);
    assert_eq!(lives(&t), vec![13, 13, 13, 13]);
}

#[test]
fn arbiter_of_knollridge_in_two_headed_giant_one_member_gains() {
    cr!("810.9a", "810.9d", "810.9c");
    ruling!(
        "Arbiter of Knollridge",
        "In a Two-Headed Giant game, each team chooses one person on that team to gain the appropriate amount of life."
    );
    supported("Arbiter of Knollridge");
    let mut t = thg();
    t.g.lose_life(P3, 12);
    t.settle();
    t.answer_choose(P2, &[pl(P2)]);
    t.answer_choose(P0, &[pl(P0)]);
    let from = t.g.turn_events.len();
    enter(&mut t, P0, "Arbiter of Knollridge");
    t.resolve_all();
    assert_eq!(life_changes(&t, from), vec![(P2, 12)]);
    assert_eq!(lives(&t), vec![30, 30, 30, 30]);
}

/// The Scryfall ruling "In other multiplayer formats, each player's life total is set to
/// the highest life total within their range of influence ... Alice's life total will
/// become 8 and Barry and Doug's life totals will each become 11" isn't cited: under CR
/// 801.10 and 801.11 the ability affects and sees only what is within its controller's
/// range of influence (see 801.11's Coat of Arms example), so with Alice controlling
/// Arbiter, Barry and Doug become 8 and Carrie, out of range, keeps 11. See
/// docs/agents/engine-gaps.md, "Apparent conflicts (Nov 5)".
#[test]
fn arbiter_of_knollridge_with_range_of_influence_uses_its_controllers_range() {
    cr!("801.10", "801.11", "119.5");
    supported("Arbiter of Knollridge");
    // Alice (P0) 4, Barry (P1) 8, Carrie (P2) 11, Doug (P3) 7, each with a range of one.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            range_of_influence: Some(1),
            ..GameConfig::free_for_all()
        },
    );
    for (p, n) in [(P0, 4), (P1, 8), (P2, 11), (P3, 7)] {
        set_life(&mut t, p, n);
    }
    enter(&mut t, P0, "Arbiter of Knollridge");
    t.resolve_all();
    assert_eq!(lives(&t), vec![8, 8, 11, 8]);
}

// ---------------------------------------------------------------------------
// Comparisons on resolution
// ---------------------------------------------------------------------------

#[test]
fn timely_reinforcements_compares_with_each_opponent_on_resolution() {
    cr!("608.2h", "102.3");
    ruling!(
        "Timely Reinforcements",
        "You compare life totals with each of your opponents when Timely Reinforcements resolves."
    );
    // Less life than one opponent (P2), fewer creatures than one opponent (P2).
    let mut t = TestGame::new(3);
    set_life(&mut t, P0, 15);
    set_life(&mut t, P1, 10);
    set_life(&mut t, P2, 18);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Timely Reinforcements", &[]);
    assert_eq!(t.life(P0), 21);
    assert_eq!(tokens_of(&t, P0, "Soldier").len(), 3);
    // It's cast with less life and fewer creatures; before it resolves, P0 gains life and
    // gets creatures: nothing happens.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 15);
    t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Timely Reinforcements", &[]);
    t.g.gain_life(P0, 10);
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert!(tokens_of(&t, P0, "Soldier").is_empty());
}

#[test]
fn timely_reinforcements_in_two_headed_giant() {
    cr!("810.9a", "608.2h");
    ruling!(
        "Timely Reinforcements",
        "In a Two-Headed Giant game, you’ll compare your team’s life total with the opposing team’s life total. You’ll then compare the number of creatures you control (ignoring creatures your teammate controls) to the number of creatures each opponent controls separately."
    );
    supported("Timely Reinforcements");
    // P0's team has less life; P0 controls one creature, its teammate four; P2 controls
    // two and P3 none: P0 controls fewer than P2.
    let mut t = thg();
    t.g.lose_life(P1, 5);
    t.settle();
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Timely Reinforcements", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (31, 31));
    assert_eq!(tokens_of(&t, P0, "Soldier").len(), 3);
    // P0 controls two creatures, P2 and P3 two each (four for their team): no tokens; the
    // teams' life totals are equal: no life.
    let mut t = thg();
    for p in [P0, P0, P2, P2, P3, P3] {
        t.battlefield(p, "Grizzly Bears");
    }
    cast_resolve(&mut t, P0, "Timely Reinforcements", &[]);
    assert_eq!(t.life(P0), 30);
    assert!(tokens_of(&t, P0, "Soldier").is_empty());
}

#[test]
fn even_the_odds_checks_only_as_it_is_cast() {
    cr!("601.3", "608.2");
    ruling!(
        "Even the Odds",
        "It doesn’t matter how many creatures you control as Even the Odds resolves."
    );
    supported("Even the Odds");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Even the Odds", &[]);
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Soldier").len(), 3);
}

#[test]
fn gift_of_estates_needs_an_opponent_with_more_lands_and_finds_nonbasic_plains() {
    cr!("608.2c", "205.3i", "701.23a");
    ruling!(
        "Gift of Estates",
        "Gift of Estates does nothing if each of your opponents has the same number of lands or fewer as you."
    );
    ruling!(
        "Gift of Estates",
        "You can find nonbasic lands if they have the Plains land type, such as dual lands (Tundra, et al)."
    );
    supported("Gift of Estates");
    // An opponent with more lands: Tundra (a nonbasic Plains) is found.
    let mut t = TestGame::new(2);
    let tundra = t.library_top(P0, "Tundra");
    t.library_top(P0, "Plains");
    t.lands(P1, "Forest", 4);
    t.lands(P0, "Plains", 2);
    t.answer_choose(P0, &[obj(tundra)]);
    let gift = t.hand(P0, "Gift of Estates");
    t.cast(P0, gift).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Tundra"));
    // Equal land counts: nothing, not even a search.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Plains");
    t.lands(P1, "Forest", 2);
    t.lands(P0, "Plains", 2);
    let gift = t.hand(P0, "Gift of Estates");
    let from = t.g.turn_events.len();
    t.cast(P0, gift).go();
    t.resolve_all();
    assert!(!events_since(&t, from)
        .iter()
        .any(|e| matches!(e, Event::Searched { .. } | Event::Shuffled { .. })));
    assert!(!t.in_hand(P0, "Plains"));
}

#[test]
fn oreskos_explorer_searches_on_resolution_even_for_zero() {
    cr!("608.2h", "701.23a");
    ruling!(
        "Oreskos Explorer",
        "If no player controls more lands than you do, you'll still search and shuffle your library, although you can't find any cards."
    );
    ruling!(
        "Oreskos Explorer",
        "The value of X is determined as Oreskos Explorer's triggered ability resolves."
    );
    supported("Oreskos Explorer");
    let mut t = TestGame::new(3);
    t.library_top(P0, "Plains");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 2);
    let from = t.g.turn_events.len();
    enter(&mut t, P0, "Oreskos Explorer");
    t.resolve_all();
    let evs = events_since(&t, from);
    assert!(evs
        .iter()
        .any(|e| matches!(e, Event::Searched { player, .. } if *player == P0)));
    assert!(evs
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == P0)));
    assert!(!t.in_hand(P0, "Plains"));
    // X is counted as the ability resolves: P1 and P2 get a third land in response.
    let mut t = TestGame::new(3);
    let a = t.library_top(P0, "Plains");
    let b = t.library_top(P0, "Plains");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 2);
    t.lands(P2, "Forest", 2);
    enter(&mut t, P0, "Oreskos Explorer");
    t.battlefield(P1, "Forest");
    t.battlefield(P2, "Forest");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Hand(P0), "Plains").len(), 2);
}

// ---------------------------------------------------------------------------
// Intervening "if" clauses and targeting requirements
// ---------------------------------------------------------------------------

#[test]
fn linvala_checks_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Linvala, the Preserver",
        "For each triggered ability, the game checks to see if the condition is true at the moment the ability would trigger."
    );
    supported("Linvala, the Preserver");
    // No opponent with more life or creatures: neither ability triggers.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Linvala, the Preserver");
    assert_eq!(t.stack_len(), 0);
    // Both trigger; before they resolve, the opponent's advantage is gone.
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 25);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Linvala, the Preserver");
    assert_eq!(t.stack_len(), 2);
    t.g.lose_life(P1, 10);
    t.g.destroy(b1, None);
    t.g.destroy(b2, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(tokens_of(&t, P0, "Angel").is_empty());
}

#[test]
fn linvala_condition_can_hold_for_different_opponents() {
    cr!("603.4", "102.3");
    ruling!(
        "Linvala, the Preserver",
        "In a multiplayer game, it's possible for the condition to be true at both times but with respect to two different opponents."
    );
    let mut t = TestGame::new(3);
    set_life(&mut t, P1, 25);
    set_life(&mut t, P2, 15);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Linvala, the Preserver");
    assert_eq!(t.stack_len(), 2);
    // Now P2, not P1, has more life and more creatures.
    t.g.lose_life(P1, 10);
    t.g.gain_life(P2, 10);
    t.g.destroy(b1, None);
    t.g.destroy(b2, None);
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(tokens_of(&t, P0, "Angel").len(), 1);
}

/// P0's upkeep begins (from P1's end step) with the given scripted answers already queued;
/// the upkeep triggers are on the stack.
fn to_p0_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn oath_of_druids_target_must_control_more_creatures() {
    cr!("115.1", "608.2b", "603.3d");
    ruling!(
        "Oath of Druids",
        "The targeted player controlling more creatures than the current player is a part of the targeting requirement."
    );
    supported("Oath of Druids");
    // Equal creatures: no legal target, so the ability is removed from the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Druids");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "reveal cards"), 0);
    // P1 controls more as it triggers but not as it resolves: it doesn't resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Druids");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Hill Giant");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "reveal cards"), 1);
    t.g.destroy(b, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert_eq!(
        t.g.library_top(P0).map(|c| t.g.obj(c).chars.name.clone()),
        Some("Hill Giant".into())
    );
}

#[test]
fn oath_of_mages_target_must_have_more_life() {
    cr!("115.1", "608.2b", "603.3d");
    ruling!(
        "Oath of Mages",
        "The targeted player having more life than the current player is a part of the targeting requirement."
    );
    supported("Oath of Mages");
    // Equal life: no legal target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Mages");
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 0);
    // P1 has more life as it triggers but not as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Mages");
    set_life(&mut t, P1, 22);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    t.g.lose_life(P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Still more life: 1 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Mages");
    set_life(&mut t, P1, 22);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

// ---------------------------------------------------------------------------
// Predatory Advantage
// ---------------------------------------------------------------------------

/// Advances to P1's end step (from its precombat main phase) and resolves everything.
fn through_p1_end(t: &mut TestGame) {
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
}

#[test]
fn predatory_advantage_checks_creature_spells_cast_even_if_countered() {
    cr!("603.4", "601.2i");
    ruling!(
        "Predatory Advantage",
        "Predatory Advantage checks whether a player cast a creature spell, not whether a creature entered the battlefield under that player’s control."
    );
    supported("Predatory Advantage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Predatory Advantage");
    t.set_step(P1, Step::PrecombatMain);
    let bears = cast_only(&mut t, P1, "Grizzly Bears", &[]);
    t.g.turn.priority = Some(P0);
    cast_only(&mut t, P0, "Cancel", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    through_p1_end(&mut t);
    assert!(tokens_of(&t, P0, "Lizard").is_empty());
    // A creature put onto the battlefield without being cast doesn't stop it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Predatory Advantage");
    t.set_step(P1, Step::PrecombatMain);
    enter(&mut t, P1, "Grizzly Bears");
    through_p1_end(&mut t);
    assert_eq!(tokens_of(&t, P0, "Lizard").len(), 1);
}

#[test]
fn predatory_advantage_intervening_if_is_rechecked() {
    cr!("603.4");
    ruling!(
        "Predatory Advantage",
        "Predatory Advantage’s ability has an “intervening ‘if’ clause.”"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Predatory Advantage");
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Lizard"), 1);
    // In response, P1 casts a creature spell with flash.
    cast_only(&mut t, P1, "Ambush Viper", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
    assert!(tokens_of(&t, P0, "Lizard").is_empty());
}

#[test]
fn predatory_advantage_in_two_headed_giant_triggers_for_each_opponent() {
    cr!("805.4d", "603.4");
    ruling!(
        "Predatory Advantage",
        "In a Two-Headed Giant game, Predatory Advantage’s ability triggers twice at the end of the opposing team’s turn: once for each player."
    );
    for one_casts in [false, true] {
        let mut t = thg();
        t.battlefield(P0, "Predatory Advantage");
        t.set_step(P2, Step::PrecombatMain);
        if one_casts {
            t.g.turn.priority = Some(P3);
            cast_only(&mut t, P3, "Grizzly Bears", &[]);
            t.resolve_all();
        }
        t.advance_to(P2, Step::End);
        t.settle();
        t.resolve_all();
        let want = if one_casts { 1 } else { 2 };
        assert_eq!(
            tokens_of(&t, P0, "Lizard").len(),
            want,
            "P3 cast: {one_casts}"
        );
    }
}

// ---------------------------------------------------------------------------
// Can't-play restrictions
// ---------------------------------------------------------------------------

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

#[test]
fn damping_engine_sacrifice_needs_the_most_permanents_and_works_once_a_turn() {
    cr!("116.2d");
    ruling!(
        "Damping Engine",
        "A player can only sacrifice a permanent to end this effect if they have more permanents than any other player, and they haven’t already sacrificed a permanent to end the effect this turn."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Damping Engine");
    t.lands(P0, "Forest", 3);
    t.lands(P2, "Forest", 3);
    // P0 and P2 tie at three (P1 has one): nobody controls more than each other player.
    assert!(specials(&mut t, P0).is_empty());
    assert!(specials(&mut t, P2).is_empty());
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    // Now P0 controls the most (five): it can't cast creature spells, and may sacrifice
    // once to ignore that.
    t.lands(P0, "Forest", 1);
    let more = t.hand(P0, "Grizzly Bears");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 2);
    assert!(t.cast(P0, more).try_go().is_err());
    t.clear_answers();
    let sa = specials(&mut t, P0);
    assert_eq!(sa.len(), 1);
    assert!(specials(&mut t, P2).is_empty());
    t.g.take_action(P0, Action::Special(sa[0].clone()));
    assert!(specials(&mut t, P0).is_empty());
    t.cast(P0, more).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert!(specials(&mut t, P0).is_empty());
}

#[test]
fn ward_of_bones_compares_each_type_separately() {
    cr!("101.2", "601.3");
    ruling!(
        "Ward of Bones",
        "Spelling this out completely: — Each opponent who controls more creatures than you can’t play creature cards. — Each opponent who controls more artifacts than you can’t play artifact cards. — Each opponent who controls more enchantments than you can’t play enchantment cards. — Each opponent who controls more lands than you can’t play land cards."
    );
    supported("Ward of Bones");
    let try_cast = |t: &mut TestGame, name: &str| -> bool {
        mana_all_inline(t, P1);
        let c = t.hand(P1, name);
        let ok = t.cast(P1, c).try_go().is_ok();
        t.clear_answers();
        t.resolve_all();
        ok
    };
    // P1 controls more creatures than P0: no creature spells (an artifact is fine).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ward of Bones");
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Grizzly Bears");
    assert!(!try_cast(&mut t, "Hill Giant"));
    assert!(try_cast(&mut t, "Sol Ring"));
    // More artifacts: no artifact spells (artifact creatures neither).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ward of Bones");
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P1, "Sol Ring");
    assert!(!try_cast(&mut t, "Sol Ring"));
    assert!(!try_cast(&mut t, "Ornithopter"));
    assert!(try_cast(&mut t, "Grizzly Bears"));
    // More enchantments: no enchantment spells.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ward of Bones");
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Glorious Anthem");
    assert!(!try_cast(&mut t, "Glorious Anthem"));
    assert!(try_cast(&mut t, "Grizzly Bears"));
    // More lands: no land plays; with equal lands, a land can be played.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ward of Bones");
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Forest");
    let f = t.hand(P1, "Forest");
    assert!(t.play_land(P1, f).is_err());
    t.battlefield(P0, "Forest");
    assert!(t.play_land(P1, f).is_ok());
}

fn mana_all_inline(t: &mut TestGame, p: PlayerId) {
    crate::r_p038_common::mana_all(t, p, 4);
}

// ---------------------------------------------------------------------------
// Verge Rangers
// ---------------------------------------------------------------------------

fn p0_sees_top(g: &Game) -> bool {
    g.library_top(P0)
        .is_some_and(|top| mtg_engine::zones::can_see_in_library(g, P0, top))
}

#[test]
fn verge_rangers_look_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Verge Rangers",
        "Verge Rangers lets you look at the top card of your library whenever you want (with one restriction; see below), even if you don't have priority."
    );
    supported("Verge Rangers");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let top = t.library_top(P0, "Shock");
    t.battlefield(P0, "Verge Rangers");
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, top));
    // While the opponent has priority, after a draw, the new top card.
    t.set_step(P1, Step::PrecombatMain);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(p0_sees_top(&t.g));
}

fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card: c, .. } if *c == card))
}

#[test]
fn verge_rangers_doesnt_give_extra_land_plays() {
    cr!("305.2", "305.1");
    ruling!(
        "Verge Rangers",
        "Verge Rangers doesn't allow you to play additional lands."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verge Rangers");
    t.lands(P1, "Forest", 3);
    let top = t.library_top(P0, "Plains");
    assert!(can_play_land(&mut t, P0, top));
    let f = t.hand(P0, "Plains");
    t.play_land(P0, f).unwrap();
    assert!(!can_play_land(&mut t, P0, top));
    // Without an opponent controlling more lands, none from the top at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verge Rangers");
    let top = t.library_top(P0, "Plains");
    assert!(!can_play_land(&mut t, P0, top));
}

#[test]
fn verge_rangers_next_card_is_hidden_while_the_land_is_played() {
    cr!("401.5", "305.1");
    ruling!(
        "Verge Rangers",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you can't look at the new top card until you finish doing so. This means that if you play a land from the top of your library and it has a replacement effect that requires you to make a de"
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    let garden = t.library_top(P0, "Temple Garden");
    t.battlefield(P0, "Verge Rangers");
    t.battlefield(P1, "Forest");
    t.g.recompute();
    // The choice for Temple Garden ("you may pay 2 life") is made without seeing the Hill
    // Giant beneath it.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        p0_sees_hill_giant,
    );
    t.answer_yes(P0, true);
    t.play_land(P0, garden).unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert_eq!(t.named_on_battlefield("Temple Garden").len(), 1);
    assert!(p0_sees_hill_giant(&t.g));
    assert!(p0_sees_top(&t.g));
}

/// Whether P0 may look at the Hill Giant in their library.
fn p0_sees_hill_giant(g: &Game) -> bool {
    g.player(P0).library.iter().any(|c| {
        g.obj(*c).chars.name == "Hill Giant" && mtg_engine::zones::can_see_in_library(g, P0, *c)
    })
}

// ---------------------------------------------------------------------------
// Desperate Ravings
// ---------------------------------------------------------------------------

#[test]
fn desperate_ravings_draws_then_discards_at_random_in_one_resolution() {
    cr!("608.2c", "701.9a", "701.9b");
    ruling!(
        "Desperate Ravings",
        "You draw two cards and discard one randomly all while Desperate Ravings is resolving."
    );
    supported("Desperate Ravings");
    // Psychosis Crawler: "Whenever you draw a card, each opponent loses 1 life." Its
    // triggers for the two draws wait until the spell has finished resolving: when they
    // are put on the stack, the card is already discarded, and nobody chose it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychosis Crawler");
    t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_only(&mut t, P0, "Desperate Ravings", &[]);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 2);
    assert_eq!(t.life(P1), 20);
    assert!(!asked_since(&t, from).iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseEntities { prompt, .. } if prompt.contains("discard")
    )));
    // At random: over several games, the Bears (one of the three cards) is sometimes the
    // one discarded and sometimes not.
    let mut seen = [false; 2];
    for seed in 0..16 {
        let mut t = TestGame::new(2);
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        t.hand(P0, "Grizzly Bears");
        cast_resolve(&mut t, P0, "Desperate Ravings", &[]);
        seen[usize::from(t.in_graveyard(P0, "Grizzly Bears"))] = true;
    }
    assert_eq!(seen, [true, true]);
}
