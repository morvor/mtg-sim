//! Rulings batch P186 (review) — counts of what happened to opponents this turn include
//! opponents who have since left the game (CR 800.4i; interpretation
//! `history-counts-players-who-left`): the other wordings the interpretation rests on.

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn moonshae_pixie_counts_an_opponent_dealt_combat_damage_who_then_left() {
    cr!("800.4i", "603.2");
    interp!("history-counts-players-who-left");
    ruling!(
        "Moonshae Pixie // Pixie Dust",
        "If an opponent was dealt combat damage this turn and subsequently lost the game, Moonshae Pixie's second ability still counts that player when determining how many cards its controller should draw."
    );
    supported("Moonshae Pixie // Pixie Dust");
    let run = |leave: bool, combat: bool| {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, true);
        t.g.deal_damage(bears, Entity::Player(P2), 1, combat);
        t.g.flush_events();
        if leave {
            crate::r_p079_common::concede(&mut t, P2);
            assert!(t.has_lost(P2));
        }
        let hand = t.hand_size(P0);
        crate::r_s05_common::enter(&mut t, P0, "Moonshae Pixie // Pixie Dust");
        t.resolve_all();
        t.hand_size(P0) - hand
    };
    assert_eq!(run(false, true), 2);
    assert_eq!(run(true, true), 2, "the opponent who left still counts");
    // Control: noncombat damage to that opponent isn't counted.
    assert_eq!(run(true, false), 1);
}

#[test]
fn chandras_incinerator_counts_damage_to_an_opponent_who_lost() {
    cr!("800.4i", "601.2f");
    ruling!(
        "Chandra's Incinerator",
        "In a multiplayer game, Chandra's Incinerator counts damage dealt to opponents who have lost the game."
    );
    supported("Chandra's Incinerator");
    // {5}{R}, {X} less where X is the noncombat damage dealt to opponents this turn.
    let setup = |lands: usize| {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, false);
        t.g.deal_damage(bears, Entity::Player(P2), 3, false);
        t.g.flush_events();
        crate::r_p079_common::concede(&mut t, P2);
        assert!(t.has_lost(P2));
        t.lands(P0, "Mountain", lands);
        let ci = t.hand(P0, "Chandra's Incinerator");
        (t, ci)
    };
    // 4 damage counted: {1}{R}.
    let (mut t, ci) = setup(2);
    assert!(t.cast(P0, ci).try_go().is_ok());
    let (mut t, ci) = setup(1);
    assert!(t.cast(P0, ci).try_go().is_err());
}

/// A three-player game in P0's precombat main phase: Grizzly Bears of P0's deals `damage`
/// to P2 (combat damage if `combat`), then P2 concedes if `leave`.
fn p2_damaged_then_left(
    card: &str,
    damage: u32,
    combat: bool,
    leave: bool,
) -> (TestGame, ObjectId) {
    supported(card);
    let mut t = TestGame::new(3);
    let source = t.battlefield(P0, card);
    let bears = t.battlefield(P0, "Grizzly Bears");
    if damage > 0 {
        t.g.deal_damage(bears, Entity::Player(P2), damage, combat);
        t.g.flush_events();
    }
    if leave {
        crate::r_p079_common::concede(&mut t, P2);
        assert!(t.has_lost(P2));
    }
    (t, source)
}

#[test]
fn sygg_counts_an_opponent_who_lost_3_life_then_left() {
    cr!("800.4i", "603.4");
    interp!("history-counts-players-who-left");
    ruling!(
        "Sygg, River Cutthroat",
        "or a player who was your opponent at the time they left the game, has lost 3 or more life"
    );
    let drawn = |damage: u32, leave: bool| {
        let (mut t, _) = p2_damaged_then_left("Sygg, River Cutthroat", damage, false, leave);
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.hand_size(P0) - hand
    };
    assert_eq!(drawn(3, false), 1);
    assert_eq!(drawn(3, true), 1, "the opponent who left still counts");
    // Control: 2 life lost isn't enough.
    assert_eq!(drawn(2, true), 0);
}

#[test]
fn tymna_counts_an_opponent_dealt_combat_damage_who_then_lost() {
    cr!("800.4i");
    interp!("history-counts-players-who-left");
    ruling!(
        "Tymna the Weaver",
        "If a player was dealt combat damage and subsequently lost the game, Tymna's triggered ability counts that player to determine the value of X."
    );
    // (cards drawn, life paid)
    let run = |combat: bool| {
        let (mut t, _) = p2_damaged_then_left("Tymna the Weaver", 1, combat, true);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, true);
        t.g.flush_events();
        t.answer_yes(P0, true);
        let (hand, life) = (t.hand_size(P0), t.life(P0));
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        (t.hand_size(P0) - hand, life - t.life(P0))
    };
    assert_eq!(run(true), (2, 2), "X counts P2, who has left");
    // Control: P2 was dealt noncombat damage, so only P1 counts.
    assert_eq!(run(false), (1, 1));
}

#[test]
fn kaito_counts_an_opponent_who_lost_life_then_the_game() {
    cr!("800.4i");
    interp!("history-counts-players-who-left");
    ruling!(
        "Kaito, Bane of Nightmares",
        "If an opponent lost life this turn and subsequently lost the game, Kaito's fourth ability still counts that player when determining how many cards to draw."
    );
    let drawn = |leave: bool| {
        let (mut t, kaito) = p2_damaged_then_left("Kaito, Bane of Nightmares", 1, false, leave);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.deal_damage(bears, Entity::Player(P1), 1, false);
        t.g.flush_events();
        t.g.add_counters(Entity::Object(kaito), "loyalty", 3, None);
        t.g.recompute();
        let index =
            t.g.obj(kaito)
                .chars
                .abilities
                .iter()
                .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
                .position(|a| a.text.starts_with("0:"))
                .expect("Kaito's 0 ability");
        let hand = t.hand_size(P0);
        t.activate(P0, kaito, index, &[]).expect("activated");
        t.resolve_all();
        t.hand_size(P0) - hand
    };
    assert_eq!(drawn(false), 2);
    assert_eq!(drawn(true), 2, "the opponent who left still counts");
}

#[test]
fn ludevic_counts_a_player_who_lost_life_then_the_game() {
    cr!("800.4i", "603.4");
    interp!("history-counts-players-who-left");
    ruling!(
        "Ludevic, Necro-Alchemist",
        "If a player lost life and subsequently lost the game, the player whose turn it is may draw a card."
    );
    // "a player other than you lost life this turn" is a count of players
    // (`Value::CountPlayers` with a history filter), checked as the trigger resolves.
    let drawn = |damage: u32, leave: bool| {
        let (mut t, _) = p2_damaged_then_left("Ludevic, Necro-Alchemist", damage, false, leave);
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.hand_size(P0) - hand
    };
    assert_eq!(drawn(1, false), 1);
    assert_eq!(drawn(1, true), 1, "the player who left still counts");
    // Control: no other player lost life this turn.
    assert_eq!(drawn(0, true), 0);
}

#[test]
fn bloodthirst_counts_an_opponent_dealt_damage_who_then_left() {
    // Bloodthirst N: "If an opponent was dealt damage this turn" asks about players by what
    // happened to them this turn, like Spinerock Knoll's "an opponent was dealt 7 or more
    // damage this turn", whose ruling counts an opponent who has since left the game.
    cr!("702.54a", "800.4i");
    interp!("history-counts-players-who-left");
    let counters = |damage: u32, leave: bool| {
        let (mut t, _) = p2_damaged_then_left("Grizzly Bears", damage, false, leave);
        let minotaurs = t.enter(P0, "Gorehorn Minotaurs");
        t.counters(minotaurs, "+1/+1")
    };
    assert_eq!(counters(1, false), 2);
    assert_eq!(counters(1, true), 2, "the opponent who left still counts");
    // Control: no opponent was dealt damage this turn.
    assert_eq!(counters(0, true), 0);
}
