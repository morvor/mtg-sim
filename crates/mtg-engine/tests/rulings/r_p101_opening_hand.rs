//! Rulings batch P101 — opening hands (CR 103.6): the actions taken with cards from an
//! opening hand after all mulligans, the starting player first; Leyline of Singularity's
//! legend rule as the first upkeep starts (CR 704.5j, 117.5); Providence's "becomes 26"
//! (CR 119.5); Once Upon a Time in the first upkeep.

use crate::r_p101_common::*;
use crate::r_s02_common::can_play_land;
use mtg_engine::card::card;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;

#[test]
fn opening_hand_actions_come_after_mulligans_starting_player_first_for_each_wording() {
    cr!("103.6", "103.6a", "103.6b", "103.5", "103.5c", "103.8");
    ruling!("Chancellor of the Spires", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Devourer of Destiny", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Gemstone Caverns", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline Axe", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Abundance", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Anticipation", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Combustion", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Hope", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Lifeforce", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Lightning", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Mutation", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Punishment", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Resonance", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Sanctity", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Singularity", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Transformation", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of Vitality", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of the Guildpact", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of the Meek", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Leyline of the Void", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Providence", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    ruling!("Sphinx of Foresight", "A player's \"opening hand\" is the hand of cards the player has after all players have taken mulligans.");
    // Three wordings: "If this card is in your opening hand, you may begin the game with it
    // on the battlefield" (the Leylines), "... and you're not the starting player, ..."
    // (Gemstone Caverns), and "You may reveal this card from your opening hand. If you do,
    // at the beginning of the first upkeep / your first upkeep, ..." (the others).
    // Three players, P1 starts, P0 mulligans twice (the first mulligan in a multiplayer
    // game is free, CR 103.5c): P1's actions, then P2's, then P0's.
    let begin = [
        "Leyline Axe",
        "Leyline of Abundance",
        "Leyline of Anticipation",
        "Leyline of Combustion",
        "Leyline of Hope",
        "Leyline of Lifeforce",
        "Leyline of Lightning",
        "Leyline of Mutation",
        "Leyline of Punishment",
        "Leyline of Resonance",
        "Leyline of Sanctity",
        "Leyline of Singularity",
        "Leyline of Transformation",
        "Leyline of Vitality",
        "Leyline of the Guildpact",
        "Leyline of the Meek",
        "Leyline of the Void",
    ];
    for name in begin {
        supported(name);
        let (t, order) = opening_hands(3, name, P1, &[P0, P0]);
        assert_eq!(player_runs(&order), [P1, P2, P0], "{name}");
        // Every copy in each opening hand began the game on the battlefield (six for P0,
        // who mulliganed), before the first turn.
        assert_eq!(
            [P1, P2, P0].map(|p| controls_named(&t, p, name)),
            [7, 7, 6],
            "{name}"
        );
        assert_eq!(t.g.turn.number, 0, "{name}: the first turn hasn't begun");
    }
    supported("Gemstone Caverns");
    let (t, order) = opening_hands(3, "Gemstone Caverns", P1, &[P0, P0]);
    assert_eq!(
        player_runs(&order),
        [P2, P0],
        "not offered to the starting player"
    );
    assert_eq!(
        [P1, P2, P0].map(|p| controls_named(&t, p, "Gemstone Caverns")),
        [0, 4, 3],
        "each copy may begin the game on the battlefield, exiling another card"
    );
    for name in [
        "Chancellor of the Spires",
        "Devourer of Destiny",
        "Providence",
        "Sphinx of Foresight",
    ] {
        supported(name);
        let (t, order) = opening_hands(3, name, P1, &[P0, P0]);
        assert_eq!(player_runs(&order), [P1, P2, P0], "{name}");
        assert_eq!(
            order.len(),
            7 + 7 + 6,
            "{name}: each card in each opening hand"
        );
        assert_eq!(t.g.delayed_triggers.len(), 20, "{name}");
        assert_eq!(t.g.turn.number, 0, "{name}");
    }
    // Then the first turn begins: Chancellor of the Spires' reveals mill each opponent
    // seven cards apiece at the first upkeep (P1's): each player's seven reveals mill all
    // 33 cards of the other's library.
    let (mut t, _) = opening_hands(2, "Chancellor of the Spires", P1, &[]);
    let start = t.g.turn.starting_player;
    assert_eq!(start, P1);
    t.g.begin_turn(start, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    for p in [P0, P1] {
        assert_eq!(t.graveyard_size(p), 33);
        assert_eq!(t.library_size(p), 0);
    }
}

#[test]
fn several_leylines_of_singularity_go_to_the_legend_rule_as_the_first_upkeep_starts() {
    cr!("103.6a", "704.5j", "117.5", "502.4", "205.4a");
    ruling!(
        "Leyline of Singularity",
        "If the game starts with more than one Leyline of Singularity under a single player's control, all but one of them will be put into their owners' graveyards right after the upkeep of the first turn starts. If the game started with multiple copies of any other Leylines under a single player's control, that player will also put all but one of them into their graveyard at that time."
    );
    supported("Leyline of Singularity");
    // P0's opening hand: two Leylines of Singularity, two Leylines of Hope and three Wastes.
    let hand = [
        "Leyline of Singularity",
        "Leyline of Singularity",
        "Leyline of Hope",
        "Leyline of Hope",
        "Wastes",
        "Wastes",
        "Wastes",
    ];
    let mut deck: Vec<_> = hand.iter().map(|n| card(n)).collect();
    deck.extend((0..33).map(|_| card("Wastes")));
    let other: Vec<_> = (0..40).map(|_| card("Wastes")).collect();
    let mut t = crate::r_s12_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            top_of_library: vec![hand.iter().map(|n| SmolStr::new(n)).collect()],
            ..Default::default()
        },
        vec![deck, other],
    );
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    t.g.start();
    // All four began the game on the battlefield; no state-based actions yet (the untap
    // step has no priority).
    assert_eq!(controls_named(&t, P0, "Leyline of Singularity"), 2);
    assert_eq!(controls_named(&t, P0, "Leyline of Hope"), 2);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(controls_named(&t, P0, "Leyline of Singularity"), 2);
    // As P0 would receive priority in the first upkeep: one of each is left.
    t.settle();
    assert_eq!(controls_named(&t, P0, "Leyline of Singularity"), 1);
    assert_eq!(controls_named(&t, P0, "Leyline of Hope"), 1);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn leyline_of_singularity_makes_same_named_tokens_legendary() {
    cr!("704.5j", "205.4a", "111.1");
    ruling!(
        "Leyline of Singularity",
        "The \"legend rule\" says that if a player controls two or more legendary permanents with the same name, that player chooses one of them and the rest are put into their owners' graveyards. Notably, if you control Leyline of Singularity and two or more creature tokens with the same name, only one of the tokens will be allowed to remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Singularity");
    let a = create_token(&mut t, P0, "Thopter");
    let b = create_token(&mut t, P0, "Thopter");
    // P1's Thopter has the same name but another controller: it stays.
    let theirs = create_token(&mut t, P1, "Thopter");
    t.answer_choose(P0, &[obj(b)]);
    t.settle();
    assert!(t.on_battlefield(b), "the one P0 chose stays");
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(theirs));
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn providence_sets_life_to_26_by_gaining_or_losing_once_however_many_are_revealed() {
    cr!("119.5", "103.6b", "603.7g", "614.1a");
    ruling!(
        "Providence",
        "For a player's life total to become 26, the player gains or loses the appropriate amount of life. For example, if your life total is 4 when Providence resolves, it will cause you to gain 22 life; alternatively, if your life total is 40 when it resolves, it will cause you to lose 14 life. Other cards that interact with life gain or life loss will interact with this effect accordingly."
    );
    ruling!(
        "Providence",
        "Revealing multiples of Providence from your opening hand won't cause your life total to become 32 or any number other than 26."
    );
    supported("Providence");
    // Three revealed: the first upkeep sets P0's life to 26, not 32 or more.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Providence");
        t.answer_yes(P0, true);
    }
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.life(P1), 20);
    // Cast at 4 life: P0 gains 22; at 40, P0 loses 14.
    for (from, gained, lost) in [(4, 22, 0), (40, 0, 14)] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, from);
        cast_resolve(&mut t, P0, "Providence", "5WW", &[]);
        assert_eq!(t.life(P0), 26);
        assert_eq!(
            t.g.history.life_gained.get(&P0).copied().unwrap_or(0),
            gained
        );
        assert_eq!(t.g.history.life_lost.get(&P0).copied().unwrap_or(0), lost);
    }
    // Leyline of Hope ("If you would gain life, you gain that much life plus 1 instead.")
    // applies to that gain: 4 + 22 + 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Hope");
    set_life(&mut t, P0, 4);
    cast_resolve(&mut t, P0, "Providence", "5WW", &[]);
    assert_eq!(t.life(P0), 27);
}

#[test]
fn once_upon_a_time_is_castable_at_the_earliest_in_the_first_upkeep() {
    cr!("103.8", "502.4", "503.1", "117.3a", "305.2");
    ruling!(
        "Once Upon a Time",
        "The earliest opportunity you have to cast Once Upon a Time is during the first player's upkeep, before that player can play a land."
    );
    supported("Once Upon a Time");
    let hand = ["Once Upon a Time", "Forest"];
    let mut deck: Vec<_> = hand.iter().map(|n| card(n)).collect();
    deck.extend((0..38).map(|_| card("Forest")));
    let other: Vec<_> = (0..40).map(|_| card("Wastes")).collect();
    let mut t = crate::r_s12_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            top_of_library: vec![hand.iter().map(|n| SmolStr::new(n)).collect()],
            ..Default::default()
        },
        vec![deck, other],
    );
    t.g.start();
    t.advance_to(P0, Step::Upkeep);
    // No player has had priority before this.
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    let once = t.g.find_in_zone(Zone::Hand(P0), "Once Upon a Time")[0];
    let forest = t.g.find_in_zone(Zone::Hand(P0), "Forest")[0];
    assert!(!can_play_land(&mut t, P0, forest), "not in the upkeep");
    // P0 has no lands: the only way to cast it is its free alternative cost.
    assert!(crate::r_p102_common::alternatives(&mut t, P0, once).len() == 1);
    assert!(!crate::r_p102_common::castable_normally(&mut t, P0, once));
}
