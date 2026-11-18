//! Rulings batch P093 — spells that trade life for cards: Bad Deal (multiplayer and
//! Two-Headed Giant), Dark Bargain and Stargaze with a short library, Read the Bones,
//! Promise of Power, Abzan Charm's divided counters, Invasion of Fiora's two modes, Parting
//! Thoughts and counters, Spoils of the Vault, Necrologia, and Clue tokens.

use crate::r_p093_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn hand_of(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.hand(p, "Grizzly Bears");
    }
}

// --- Bad Deal ---------------------------------------------------------------------------------

#[test]
fn bad_deal_in_a_multiplayer_game() {
    cr!("101.4", "701.9a", "119.3");
    ruling!(
        "Bad Deal",
        "Each player loses 2 life, including you, regardless of how many cards they draw or discard."
    );
    ruling!(
        "Bad Deal",
        "If an opponent has just one card in hand, that player discards just one card. Other opponents still discard two if able."
    );
    ruling!("Bad Deal", "You draw just two cards no matter how many opponents you have.");
    supported("Bad Deal");
    let mut t = TestGame::new(4);
    hand_of(&mut t, P1, 1);
    hand_of(&mut t, P2, 3);
    let events = t.g.turn_events.len();
    cast_spell(&mut t, P0, "Bad Deal", &[]);
    assert_eq!(drawn_since(&t, events, P0), 2);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(t.graveyard_size(P2), 2);
    // P3 had no cards: it still loses 2 life.
    for p in [P0, P1, P2, P3] {
        assert_eq!(t.life(p), 18, "{p:?}");
    }
}

#[test]
fn bad_deal_in_two_headed_giant() {
    cr!("810.9", "810.9a");
    ruling!(
        "Bad Deal",
        "In a Two-Headed Giant game, Bad Deal causes each team to lose 4 life. Your teammate doesn't draw or discard any cards."
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    hand_of(&mut t, P1, 2);
    hand_of(&mut t, P2, 2);
    hand_of(&mut t, P3, 2);
    let (ours, theirs) = (t.life(P0), t.life(P2));
    cast_spell(&mut t, P0, "Bad Deal", &[]);
    assert_eq!(t.life(P0), ours - 4);
    assert_eq!(t.life(P1), ours - 4);
    assert_eq!(t.life(P2), theirs - 4);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!((t.hand_size(P2), t.hand_size(P3)), (0, 0));
}

// --- Short libraries --------------------------------------------------------------------------

#[test]
fn dark_bargain_with_fewer_than_three_cards() {
    cr!("401.2", "120.3a");
    ruling!(
        "Dark Bargain",
        "If you have fewer than three cards in your library, you put them all into your hand and none into your graveyard."
    );
    supported("Dark Bargain");
    let mut t = TestGame::new(2);
    crate::r_p140_common::library_of(&mut t, P0, 2);
    cast_spell(&mut t, P0, "Dark Bargain", &[]);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.in_graveyard(P0, "Dark Bargain"));
    assert_eq!(t.life(P0), 18);
}

/// P0 casts Stargaze for X with `library` cards in their library; it resolves.
fn stargaze(t: &mut TestGame, x: i64, library: usize) {
    supported("Stargaze");
    crate::r_p140_common::library_of(t, P0, library);
    t.lands(P0, "Swamp", 2 + x as usize);
    let s = t.hand(P0, "Stargaze");
    t.cast(P0, s).x(x).go();
    t.resolve_all();
}

#[test]
fn stargaze_with_a_short_library() {
    cr!("401.2", "119.3", "107.3a");
    ruling!(
        "Stargaze",
        "If your library contains fewer than X cards, you'll put them all into your hand. You can't choose to put any of them into your graveyard. You'll still lose X life."
    );
    let mut t = TestGame::new(2);
    stargaze(&mut t, 3, 2);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.life(P0), 17);

    ruling!(
        "Stargaze",
        "If your library contains fewer than twice X cards, you'll look at your whole library. You'll still lose X life."
    );
    let mut t = TestGame::new(2);
    stargaze(&mut t, 3, 5);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), 0);
    // Two cards and Stargaze itself.
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.life(P0), 17);
}

// --- Read the Bones ------------------------------------------------------------------------

#[test]
fn read_the_bones_life_loss_is_part_of_the_effect() {
    cr!("118.1", "608.2c", "701.6a");
    ruling!(
        "Read the Bones",
        "The loss of life is part of the spell's effect. It's not an additional cost. If Read the Bones doesn't resolve, you won't lose life."
    );
    let mut t = TestGame::new(2);
    let bones = cast_only(&mut t, P0, "Read the Bones", &[]);
    assert_eq!(t.life(P0), 20);
    cast_new(&mut t, P1, "Counterspell", &[obj(bones)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Read the Bones"));
    assert_eq!((t.life(P0), t.hand_size(P0)), (20, 0));
    let mut t = TestGame::new(2);
    cast_spell(&mut t, P0, "Read the Bones", &[]);
    assert_eq!((t.life(P0), t.hand_size(P0)), (18, 2));
}

// --- Promise of Power ---------------------------------------------------------------------

#[test]
fn promise_of_power_sets_the_tokens_size_on_resolution() {
    cr!("608.2h", "111.1", "702.42a");
    ruling!(
        "Promise of Power",
        "The power and toughness of the Demon token are set when Promise of Power resolves. They’re unaffected if the number of cards in your hand changes later."
    );
    ruling!(
        "Promise of Power",
        "The power/toughness of the token is set when this spell resolves."
    );
    supported("Promise of Power");
    let mut t = TestGame::new(2);
    hand_of(&mut t, P0, 3);
    t.lands(P0, "Swamp", 5);
    let promise = t.hand(P0, "Promise of Power");
    t.cast(P0, promise).modes(&[1]).go();
    t.resolve_all();
    let demon = crate::r_s05_common::tokens_with_subtype(&t, P0, "Demon")[0];
    assert_eq!(t.pt(demon), (3, 3));
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.g.recompute();
    assert_eq!(t.pt(demon), (3, 3));
    // Entwined: the cards are drawn first, then the token counts the hand.
    let mut t = TestGame::new(2);
    hand_of(&mut t, P0, 3);
    t.lands(P0, "Swamp", 9);
    let promise = t.hand(P0, "Promise of Power");
    t.cast(P0, promise).kicked(true).go();
    t.resolve_all();
    let demon = crate::r_s05_common::tokens_with_subtype(&t, P0, "Demon")[0];
    assert_eq!(t.pt(demon), (8, 8));
    assert_eq!(t.life(P0), 15);
}

// --- Abzan Charm ----------------------------------------------------------------------------

#[test]
fn abzan_charm_counters_for_an_illegal_target_are_lost() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Abzan Charm",
        "If you choose the third mode, you choose how the counters will be distributed as you cast the spell. Notably, if you choose to put one +1/+1 counter on each of two target creatures, and one of those creatures becomes an illegal target in response, the +1/+1 counter that would have been put on that creature is lost. It can't be put on the remaining legal target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Abzan Charm");
    let charm = t.hand(P0, "Abzan Charm");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P0, charm).modes(&[2]).targets(&[obj(a), obj(b)]).go();
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { total: 2, .. })));
    crate::r_s05_common::move_to(&mut t, a, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

// --- Invasion of Fiora ----------------------------------------------------------------------

#[test]
fn invasion_of_fiora_destroys_legendary_creatures_first() {
    cr!("700.2", "608.2c");
    ruling!(
        "Invasion of Fiora // Marchesa, Resolute Monarch",
        "If you choose both modes for Invasion of Fiora’s enters-the-battlefield ability, the modes happen in order. First all legendary creatures are destroyed, then all nonlegendary creatures."
    );
    supported("Invasion of Fiora // Marchesa, Resolute Monarch");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    let giant = t.battlefield(P1, "Hill Giant");
    let events = t.g.turn_events.len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    crate::r_s05_common::enter(&mut t, P0, "Invasion of Fiora // Marchesa, Resolute Monarch");
    t.resolve_all();
    let destroyed: Vec<ObjectId> = t.g.turn_events[events..]
        .iter()
        .filter_map(|e| match e {
            Event::Destroyed { obj, .. } => Some(*obj),
            _ => None,
        })
        .collect();
    assert_eq!(destroyed.len(), 3);
    // Grizzly Bears came first on the battlefield, but Isamaru is destroyed first.
    assert_eq!(destroyed[0], isamaru);
    assert!(destroyed[1..].contains(&bears) && destroyed[1..].contains(&giant));
}

// --- Parting Thoughts and counters ------------------------------------------------------------

/// P0 casts Parting Thoughts ("Destroy target creature. You draw X cards and you lose X
/// life, where X is the number of counters on that creature.") at `target`.
fn parting_thoughts(t: &mut TestGame, target: ObjectId) {
    supported("Parting Thoughts");
    cast_spell(t, P0, "Parting Thoughts", &[obj(target)]);
}

#[test]
fn parting_thoughts_counts_counters_only() {
    cr!("122.1", "608.2h");
    ruling!(
        "Parting Thoughts",
        "Parting Thoughts can target a creature without any counters on it. You’ll lose no life and draw no cards."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    parting_thoughts(&mut t, bears);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!((t.hand_size(P0), t.life(P0)), (0, 20));

    ruling!(
        "Parting Thoughts",
        "Parting Thoughts counts every kind of counter on that creature. Effects such as that of Giant Growth, Auras, and Equipment aren’t counters."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    put_counters(&mut t, bears, "stun", 1);
    put_counters(&mut t, bears, "flying", 1);
    crate::r_s06_common::attach_new(&mut t, P1, "Short Sword", bears);
    cast_spell(&mut t, P1, "Giant Growth", &[obj(bears)]);
    parting_thoughts(&mut t, bears);
    assert_eq!((t.hand_size(P0), t.life(P0)), (3, 17));
}

#[test]
fn plus_and_minus_counters_annihilate_before_parting_thoughts_counts() {
    cr!("704.5q", "122.3");
    ruling!(
        "Parting Thoughts",
        "If a creature has +1/+1 counters and -1/-1 counters on it, state-based actions remove the same number of each so that it has only one of those kinds of counters on it. There won’t be any counters of the other kind for Parting Thoughts to count."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 3);
    put_counters(&mut t, bears, counters::MINUS1, 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    parting_thoughts(&mut t, bears);
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 18));
}

#[test]
fn yawgmoth_counters_are_removed_in_pairs() {
    cr!("704.5q");
    ruling!(
        "Yawgmoth, Thran Physician",
        "If a permanent has +1/+1 counters and -1/-1 counters on it, they're removed in pairs as a state-based action until that permanent has only one of those kinds of counters on it."
    );
    // Yawgmoth: "Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one
    // target creature and draw a card."
    let mut t = TestGame::new(2);
    let yawgmoth = t.battlefield(P0, "Yawgmoth, Thran Physician");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::PLUS1, 2);
    t.answer_choose(P0, &[obj(fodder)]);
    crate::r_p087_common::activate(&mut t, P0, yawgmoth, "Sacrifice another", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn yawgmoth_needs_no_target_but_an_illegal_one_stops_the_draw() {
    cr!("115.10a", "608.2b", "601.2c");
    ruling!(
        "Yawgmoth, Thran Physician",
        "You may activate Yawgmoth's first activated ability without choosing a target creature. You'll just draw a card. However, if you choose a target and it becomes illegal before the ability tries to resolve, the ability won't resolve and you won't draw a card."
    );
    let mut t = TestGame::new(2);
    let yawgmoth = t.battlefield(P0, "Yawgmoth, Thran Physician");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(fodder)]);
    crate::r_p087_common::activate(&mut t, P0, yawgmoth, "Sacrifice another", &[]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
    // A target that leaves: no card.
    let mut t = TestGame::new(2);
    let yawgmoth = t.battlefield(P0, "Yawgmoth, Thran Physician");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(fodder)]);
    crate::r_p087_common::activate(&mut t, P0, yawgmoth, "Sacrifice another", &[obj(bears)]);
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (0, 19));
}

// --- Spoils of the Vault --------------------------------------------------------------------

#[test]
fn spoils_of_the_vault_naming_a_card_not_in_the_library() {
    cr!("201.3", "701.20a", "119.3");
    ruling!(
        "Spoils of the Vault",
        "You don’t have to choose the name of a card that’s still in your library, or even a card that’s in your deck at all. If no card with the chosen name is in your library, you exile your library and lose 1 life for each of those cards."
    );
    supported("Spoils of the Vault");
    let mut t = TestGame::new(2);
    crate::r_p140_common::library_of(&mut t, P0, 5);
    t.answer(P0, DecisionKind::Name, Answer::Text("Black Lotus".into()));
    cast_spell(&mut t, P0, "Spoils of the Vault", &[]);
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.g.exile.len(), 5);
    assert_eq!((t.hand_size(P0), t.life(P0)), (0, 15));
}

// --- Necrologia -----------------------------------------------------------------------------

#[test]
fn necrologia_is_cast_in_the_end_step_before_the_cleanup_discard() {
    cr!("513.1", "514.1", "601.2f");
    ruling!(
        "Necrologia",
        "This spell is cast during the End step. Since you discard down to 7 cards in the cleanup step, this card is used before the discard."
    );
    supported("Necrologia");
    let mut t = TestGame::new(2);
    hand_of(&mut t, P0, 5);
    t.lands(P0, "Swamp", 5);
    let necrologia = t.hand(P0, "Necrologia");
    assert!(!crate::r_s21_common::castable(&mut t, P0, necrologia));
    t.advance_to(P0, Step::End);
    assert!(crate::r_s21_common::castable(&mut t, P0, necrologia));
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.cast(P0, necrologia).x(4).go();
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (9, 16));
    // In the cleanup step, P0 discards down to seven.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.graveyard_size(P0), 3);
}

// --- Clue tokens ----------------------------------------------------------------------------

/// The Clue tokens `p` controls.
fn clues(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    crate::r_s05_common::tokens_with_subtype(t, p, "Clue")
}

/// Checks that `clue` is an artifact token whose ability ("{2}, Sacrifice this token:
/// Draw a card.") draws a card for {2}.
fn check_clue(t: &mut TestGame, clue: ObjectId) {
    let o = t.obj_now(clue).clone();
    assert!(o.is_token() && o.is(CardType::Artifact));
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    crate::r_s06_common::activate_containing(t, P0, clue, "Draw a card").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(!t.on_battlefield(clue));
}

#[test]
fn clue_tokens_and_investigating() {
    cr!("111.10f", "701.16a", "603.2");
    ruling!(
        "Forecasting Fortune Teller",
        "Clue tokens are artifact tokens with \"{2}, Sacrifice this token: Draw a card.\""
    );
    ruling!(
        "Forecasting Fortune Teller",
        "Creating a Clue token this way isn't the same thing as investigating. Notably, an ability that triggers whenever you investigate won't trigger because of creating a Clue token this way."
    );
    supported("Forecasting Fortune Teller");
    supported("Erdwal Illuminator");
    // Erdwal Illuminator: "Whenever you investigate for the first time each turn,
    // investigate an additional time." An enters trigger creating a Clue: one Clue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erdwal Illuminator");
    crate::r_s05_common::enter(&mut t, P0, "Forecasting Fortune Teller");
    t.resolve_all();
    assert_eq!(clues(&t, P0).len(), 1);
    let clue = clues(&t, P0)[0];
    check_clue(&mut t, clue);

    ruling!(
        "Sokka's Sword Training",
        "Clue tokens are artifact tokens with \"{2}, Sacrifice this token: Draw a card.\""
    );
    ruling!(
        "Sokka's Sword Training",
        "Creating a Clue token this way isn't the same thing as investigating. Notably, an ability that triggers whenever you investigate won't trigger because of creating a Clue token this way."
    );
    supported("Sokka's Sword Training");
    // A spell creating a Clue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erdwal Illuminator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_spell(&mut t, P0, "Sokka's Sword Training", &[obj(bears)]);
    assert_eq!(clues(&t, P0).len(), 1);
    assert_eq!(t.pt(bears), (4, 4));
    let clue = clues(&t, P0)[0];
    check_clue(&mut t, clue);
}
