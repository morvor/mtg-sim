//! Rulings batch P047 — discarding hands and wheels: choices about empty hands ("if you
//! do" after discarding nothing, CR 118.12, 701.9), counting cards as the spell resolves
//! while it's still on the stack (CR 608.2h, 608.2n), choices made in turn order or
//! secretly (CR 101.4), random discards, and the draw replacements around them.

use crate::r_p047_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn yes(t: &mut TestGame, p: PlayerId, y: bool) {
    t.answer_yes(p, y);
}

fn discards(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Discarded { player, .. } if *player == p))
        .count()
}

fn draws(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Drew { player, .. } if *player == p))
        .count()
}

// ---------------------------------------------------------------------------
// Empty hands
// ---------------------------------------------------------------------------

#[test]
fn snort_discarding_an_empty_hand_still_deals_damage() {
    cr!("118.12", "608.2c");
    ruling!(
        "Snort",
        "A player may choose to discard their hand even if their hand contains zero cards. Snort will still deal 5 damage to that player if they choose to discard their empty hand."
    );
    let mut t = TestGame::new(3);
    t.hand(P2, "Grizzly Bears");
    yes(&mut t, P0, false);
    yes(&mut t, P1, true);
    yes(&mut t, P2, false);
    spell(&mut t, P0, "Snort", &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.hand_size(P1), 5);
    // P2 kept their hand: no damage.
    assert_eq!(t.life(P2), 20);
    assert_eq!(t.hand_size(P2), 1);
}

#[test]
fn step_between_worlds_empty_zones_and_turn_order() {
    cr!("101.4", "101.4a", "118.12");
    ruling!(
        "Step Between Worlds",
        "A player whose hand and/or graveyard contain no cards can still choose to shuffle their hand and graveyard into their library. If they do, they’ll still draw seven cards."
    );
    ruling!(
        "Step Between Worlds",
        "Players decide and announce in turn order, starting with the active player, whether they want to shuffle their hand and graveyard into their library. Then, all players who chose to do so shuffle their hands and graveyards into their libraries and draw seven cards. Players later in the turn order will know the choices made by players before them."
    );
    supported("Step Between Worlds");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.hand(P2, "Grizzly Bears");
    t.graveyard(P2, "Hill Giant");
    t.hand(P0, "Grizzly Bears");
    // P1 (active) and P0 have empty hands and graveyards; P1 and P2 say yes.
    yes(&mut t, P1, true);
    yes(&mut t, P2, true);
    yes(&mut t, P0, false);
    // What P2 sees when deciding: P1's hand not yet drawn.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.players[1].hand.len(),
    );
    let from = t.asked().len();
    spell(&mut t, P1, "Step Between Worlds", &[]);
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P1, P2, P0]);
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P2), 7);
    assert_eq!(t.graveyard_size(P2), 0);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn hand_dumps_with_an_empty_hand_still_draw() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Dangerous Wager",
        "If you have no cards in hand when Dangerous Wager resolves, you’ll simply draw two cards."
    );
    ruling!(
        "Heartwarming Redemption",
        "If you have no cards in hand as Heartwarming Redemption resolves, you discard nothing then draw one card."
    );
    ruling!(
        "Path of the Pyromancer",
        "If your hand is empty as Path of the Pyromancer resolves, you won’t add any mana, but you will draw a card."
    );
    ruling!("Into the Night", "If you discard zero cards, you will draw one card.");
    ruling!(
        "Valakut Awakening // Valakut Stoneforge",
        "If you put zero cards on the bottom of your library, you draw one card."
    );
    ruling!(
        "Drastic Revelation",
        "You discard your hand as part of Drastic Revelation’s effect, not as an additional cost. If you have no cards in hand at this point, you simply don’t discard any; the rest of the spell will still have its effect."
    );
    // (card, cards in hand afterwards)
    for (name, after) in [
        ("Dangerous Wager", 2),
        ("Heartwarming Redemption", 1),
        ("Path of the Pyromancer", 1),
        ("Into the Night", 1),
        ("Drastic Revelation", 4),
    ] {
        let mut t = TestGame::new(2);
        t.answer_choose(P0, &[]);
        spell(&mut t, P0, name, &[]);
        assert_eq!(t.hand_size(P0), after, "{name}");
        assert_eq!(discards(&t, P0), 3 * usize::from(name == "Drastic Revelation"));
        if name == "Path of the Pyromancer" {
            assert!(t.g.players[0].mana_pool.is_empty(), "no mana");
        }
        if name == "Heartwarming Redemption" {
            assert_eq!(t.life(P0), 21);
        }
    }
    // Valakut Awakening: nothing put on the bottom.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let v = t.hand(P0, "Valakut Awakening // Valakut Stoneforge");
    t.lands(P0, "Mountain", 3);
    t.answer_choose(P0, &[]);
    t.cast(P0, v).method(CastMethod::Normal).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn case_of_the_crimson_pulse_with_no_cards_in_hand() {
    cr!("719.3", "701.9a");
    ruling!(
        "Case of the Crimson Pulse",
        "If you have no cards in hand as Case of the Crimson Pulse's first ability resolves, you won't discard a card, but you will still draw two cards. Similarly, if you have no cards in hand when Case of the Crimson Pulse's last ability resolves, you won't discard any cards, but you'll still draw two cards."
    );
    supported("Case of the Crimson Pulse");
    let mut t = TestGame::new(2);
    let case = t.enter(P0, "Case of the Crimson Pulse");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(discards(&t, P0), 0);
    // Solved: at the beginning of the upkeep with an empty hand, still draw two.
    for c in t.g.players[0].hand.clone() {
        to_graveyard(&mut t, c);
    }
    t.g.objects[case.0 as usize].solved = true;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn moonveil_regent_empty_hands_and_colorless_spells() {
    cr!("118.12", "105.2c", "202.2");
    ruling!(
        "Moonveil Regent",
        "If you have no cards in hand as Moonveil Regent's first ability resolves, you may still choose to discard your hand. You'll just draw a card for each of that spell's colors. Great plan!"
    );
    ruling!(
        "Moonveil Regent",
        "If you cast a colorless spell, you may choose to discard your hand and draw no cards. Less great plan?"
    );
    supported("Moonveil Regent");
    // Empty hand, red spell: one card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moonveil Regent");
    yes(&mut t, P0, true);
    spell(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.hand_size(P0), 1);
    // Two cards in hand, colorless spell: discard them and draw nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moonveil Regent");
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    yes(&mut t, P0, true);
    spell(&mut t, P0, "Ornithopter", &[]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn moonveil_regent_disturb_colors_from_the_color_indicator() {
    cr!("702.146b", "204.1", "105.2");
    ruling!(
        "Moonveil Regent",
        "The colors of a nonmodal double-faced card that was cast with the disturb keyword ability are determined by its color indicator, not the mana cost on the front face of the card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moonveil Regent");
    let vet = t.graveyard(P0, "Lunarch Veteran // Luminous Phantom");
    t.lands(P0, "Plains", 2);
    yes(&mut t, P0, true);
    t.cast(P0, vet)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Disturb))
        .go();
    t.resolve_all();
    // Luminous Phantom's color indicator is white: one card.
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn moonveil_regent_death_trigger_counts_colors_on_resolution() {
    cr!("603.10a", "608.2h", "105.2");
    ruling!(
        "Moonveil Regent",
        "Moonveil Regent's last ability triggers even if you control no permanents that are one or more colors when it dies. The amount of damage is calculated as the ability resolves. Consider only the permanents you control at that time to determine how much damage Moonveil Regent deals."
    );
    // No colored permanents: it triggers, dealing no damage.
    let mut t = TestGame::new(2);
    let mr = t.battlefield(P0, "Moonveil Regent");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    spell(&mut t, P1, "Murder", &[obj(mr)]);
    assert_eq!(t.life(P1), 20);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    // A green permanent as it dies, a red one added before it resolves: two colors.
    let mut t = TestGame::new(2);
    let mr = t.battlefield(P0, "Moonveil Regent");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_x(&mut t, P1, "Murder", None, &[obj(mr)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// Counting as the spell resolves
// ---------------------------------------------------------------------------

#[test]
fn spells_on_the_stack_dont_count_themselves() {
    cr!("608.2h", "608.2n");
    ruling!(
        "Tolarian Winds",
        "It counts how many cards you discard. Since this card will not be in your hand at that time, this card is not counted."
    );
    ruling!(
        "Fateful Showdown",
        "The number of cards in your hand is counted only as Fateful Showdown resolves. Fateful Showdown is on the stack at this time, so it's not counted."
    );
    ruling!(
        "Peer Past the Veil",
        "The value of X is determined only as Peer Past the Veil resolves, after you discard your hand. Since Peer Past the Veil is still on the stack while it's resolving, it won't count itself."
    );
    ruling!(
        "Illuminate History",
        "Illuminate History will stay on the stack until it is finished resolving, so it will not count itself as one of the cards in your graveyard."
    );
    ruling!(
        "Diminishing Returns",
        "Diminishing Returns won’t be put into your graveyard until after it’s finished resolving, which means it won’t be shuffled into your library as part of its own effect."
    );
    // Tolarian Winds with two other cards: two discarded, two drawn.
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    spell(&mut t, P0, "Tolarian Winds", &[]);
    assert_eq!((t.hand_size(P0), draws(&t, P0)), (2, 2));
    // Fateful Showdown with two other cards in hand: 2 damage, draw two.
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    spell(&mut t, P0, "Fateful Showdown", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), 2);
    // Peer Past the Veil: a creature card discarded, a land already there: two types
    // (not instant).
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    spell(&mut t, P0, "Peer Past the Veil", &[]);
    assert_eq!(t.hand_size(P0), 2);
    // Illuminate History with six cards in the graveyard: no Spirit, though it ends up
    // as the seventh.
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer_choose(P0, &[]);
    spell(&mut t, P0, "Illuminate History", &[]);
    assert_eq!(t.graveyard_size(P0), 7);
    assert!(tokens(&t, P0).is_empty());
    // Diminishing Returns ends up in the graveyard, not shuffled.
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        t.answer(p, DecisionKind::Number, Answer::Number(0));
    }
    let dr = spell(&mut t, P0, "Diminishing Returns", &[]);
    assert_eq!(t.zone(dr), Zone::Graveyard(P0));
}

#[test]
fn illuminate_history_discarding_nothing_can_still_make_a_spirit() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Illuminate History",
        "You may choose not to discard any cards as Illuminate History resolves. In that case, you will not draw any cards, but you might still create a Spirit."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer_choose(P0, &[]);
    spell(&mut t, P0, "Illuminate History", &[]);
    assert_eq!(draws(&t, P0), 0);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn heartwarming_redemption_life_counts_the_hand_after_drawing() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Heartwarming Redemption",
        "The amount of life you gain is determined after you draw during Heartwarming Redemption’s resolution. No player may take actions between the time cards are drawn and the time you gain life."
    );
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    spell(&mut t, P0, "Heartwarming Redemption", &[]);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn fateful_showdown_and_borrowed_knowledge_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Fateful Showdown",
        "If Fateful Showdown's target becomes an illegal target, you won't discard your hand or draw any cards."
    );
    ruling!(
        "Borrowed Knowledge",
        "If the first mode of Borrowed Knowledge was chosen but the target is illegal when Borrowed Knowledge tries to resolve, it won't resolve and none of its effects will happen. You won't discard your hand or draw any cards."
    );
    // Fateful Showdown at a creature that leaves.
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_x(&mut t, P0, "Fateful Showdown", None, &[obj(bears)]);
    spell(&mut t, P1, "Unsummon", &[obj(bears)]);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(discards(&t, P0), 0);
    // Borrowed Knowledge's first mode at an opponent who gains hexproof.
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    fill_hand(&mut t, P1, "Grizzly Bears", 4);
    let bk = t.hand(P0, "Borrowed Knowledge");
    give_mana_for(&mut t, P0, "Borrowed Knowledge");
    t.cast(P0, bk).modes(&[0]).target(Entity::Player(P1)).go();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(discards(&t, P0), 0);
}

// ---------------------------------------------------------------------------
// Drastic Revelation
// ---------------------------------------------------------------------------

#[test]
fn drastic_revelation_small_library_and_replaced_draws() {
    cr!("121.4", "704.5b", "614.11");
    ruling!(
        "Drastic Revelation",
        "If there are fewer than seven cards left in your library, you’ll draw all the cards in your library, then you’ll discard three cards at random. Then you’ll lose the game as a state-based action."
    );
    ruling!(
        "Drastic Revelation",
        "You’ll discard three cards at random even if some other effect replaces your seven draws."
    );
    // Four cards left: draw them, discard three, lose.
    let mut t = TestGame::new(2);
    let lib: Vec<_> = t.g.players[0].library.clone();
    for c in lib.into_iter().skip(4) {
        to_graveyard(&mut t, c);
    }
    assert_eq!(t.library_size(P0), 4);
    spell(&mut t, P0, "Drastic Revelation", &[]);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(discards(&t, P0), 3);
    assert!(t.has_lost(P0));
    // Alhammarret's Archive doubles the draws: fourteen cards, still three discarded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alhammarret's Archive");
    spell(&mut t, P0, "Drastic Revelation", &[]);
    assert_eq!(t.hand_size(P0), 11);
    assert_eq!(discards(&t, P0), 3);
}

// ---------------------------------------------------------------------------
// Choosing how many
// ---------------------------------------------------------------------------

#[test]
fn diminishing_returns_choices_and_draws_in_turn_order() {
    cr!("101.4", "608.2c", "406.3");
    ruling!(
        "Diminishing Returns",
        "Each player chooses to draw any number of cards from zero to seven. First the player whose turn it is chooses how many cards to draw, then draws those cards, then each other player in turn order does the same."
    );
    ruling!(
        "Diminishing Returns",
        "The exiled cards are exiled face up. You’ll see them before you choose how many cards to draw."
    );
    supported("Diminishing Returns");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer(P1, DecisionKind::Number, Answer::Number(7));
    // When P1 chooses, P0 has drawn its three cards; ten face-up cards are in exile.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseNumber { .. }),
        |g| {
            (
                g.players[0].hand.len(),
                g.exile.iter().filter(|e| !g.obj(**e).face_down).count(),
            )
        },
    );
    let seen0 = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseNumber { .. }),
        |g| g.exile.iter().filter(|e| !g.obj(**e).face_down).count(),
    );
    spell(&mut t, P0, "Diminishing Returns", &[]);
    assert_eq!(*seen0.lock().unwrap(), vec![10]);
    assert_eq!(*seen.lock().unwrap(), vec![(3, 10)]);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.hand_size(P1), 7);
    let numbers: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseNumber { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(numbers, vec![P0, P1]);
}

#[test]
fn valakut_awakening_chooses_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Valakut Awakening // Valakut Stoneforge",
        "You choose how many cards to put on the bottom of your library as Valakut Awakening resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    let v = t.hand(P0, "Valakut Awakening // Valakut Stoneforge");
    t.lands(P0, "Mountain", 3);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, v).method(CastMethod::Normal).go();
    // Nothing chosen while casting.
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
    // A card drawn in response is among those that can be put on the bottom.
    let drawn = t.library_top(P0, "Hill Giant");
    t.g.draw_cards(P0, 1);
    let _ = drawn;
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn ill_gotten_gains_fewer_than_three() {
    cr!("608.2c", "609.3");
    ruling!("Ill-Gotten Gains", "Players can get fewer than 3 cards if they want.");
    supported("Ill-Gotten Gains");
    let mut t = TestGame::new(2);
    let g1 = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Ornithopter");
    t.hand(P1, "Shock");
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[obj(g1)]);
    spell(&mut t, P0, "Ill-Gotten Gains", &[]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn recall_discards_on_resolution_then_returns() {
    cr!("608.2c", "107.3a");
    ruling!(
        "Recall",
        "You don’t choose which cards in your graveyard you’ll return to your hand until after you discard cards. You choose a card in your graveyard for each card you discarded, then you put all cards chosen this way into your hand at the same time. You may choose to return some or all of the cards you just discarded."
    );
    ruling!(
        "Recall",
        "You don’t discard cards until Recall resolves. If you don’t have X cards in your hand at that time, you discard all the cards in your hand."
    );
    supported("Recall");
    // X = 2: discard the Bears and the Giant, then return the Bears and a Shock.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    let shock = t.graveyard(P0, "Shock");
    t.answer_choose(P0, &[obj(bears), obj(giant)]);
    t.answer_choose_moved(P0, &[obj(bears), obj(shock)]);
    t.lands(P0, "Wastes", 1);
    spell_x(&mut t, P0, "Recall", 2, &[]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Shock"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_exile("Recall"));
    // X = 3 with one card in hand: that one is discarded, one card returned.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.graveyard(P0, "Shock");
    // {X}{X}{U}: the second X.
    t.lands(P0, "Wastes", 2);
    cast_x(&mut t, P0, "Recall", Some(3), &[]);
    t.resolve_all();
    assert_eq!(discards(&t, P0), 1);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let _ = bears;
}

#[test]
fn flux_discard_is_caused_by_the_spell() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Flux",
        "The discard is forced even though you choose how many to discard."
    );
    // Loxodon Smiter discarded to P0's Flux by its opponent P1 enters the battlefield.
    let mut t = TestGame::new(2);
    let smiter = t.hand(P1, "Loxodon Smiter");
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[obj(smiter)]);
    spell(&mut t, P0, "Flux", &[]);
    assert_eq!(t.named_on_battlefield("Loxodon Smiter").len(), 1);
}

#[test]
fn anje_s_ravager_attack_requirements_and_draws() {
    cr!("508.1d", "508.1h", "506.4");
    ruling!(
        "Anje's Ravager",
        "If Anje's Ravager can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    ruling!(
        "Anje's Ravager",
        "You draw three cards while resolving the triggered ability, even if you discarded zero cards."
    );
    // Untapped, empty hand: it must attack; discard nothing, draw three.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Anje's Ravager");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.g.obj(r).tapped, "attacked");
    assert_eq!(t.hand_size(P0), 3);
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Anje's Ravager");
    t.g.tap(r);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.hand_size(P0), 0);
    // Came under P0's control this turn (summoning sick): it doesn't attack.
    let mut t = TestGame::new(2);
    let r = t.battlefield_sick(P0, "Anje's Ravager");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.g.obj(r).tapped);
    assert_eq!(t.hand_size(P0), 0);
    // Propaganda's cost: not forced to pay, so it needn't attack.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Anje's Ravager");
    t.lands(P0, "Mountain", 2);
    t.battlefield(P1, "Propaganda");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.g.obj(r).tapped);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn vaultguard_trooper_intervening_if() {
    cr!("603.4");
    ruling!(
        "Vaultguard Trooper",
        "Vaultguard Trooper’s ability will check as your end step starts to see if you control two or more tapped creatures. If you don’t, the ability won’t trigger at all. You won’t be able to tap anything during your end step in time to have the ability trigger. If you don’t control two or more tapped creatures when the ability resolves, the ability won’t do anything."
    );
    supported("Vaultguard Trooper");
    // One tapped creature: no trigger.
    let mut t = TestGame::new(2);
    let vt = t.battlefield(P0, "Vaultguard Trooper");
    t.g.tap(vt);
    t.set_step(P0, Step::PostcombatMain);
    t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    assert_eq!(triggers_on_stack(&t, "discard your hand"), 0);
    // Two tapped: it triggers; one untaps before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let vt = t.battlefield(P0, "Vaultguard Trooper");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(vt);
    t.g.tap(b);
    t.hand(P0, "Hill Giant");
    t.set_step(P0, Step::PostcombatMain);
    yes(&mut t, P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "discard your hand"), 1);
    t.g.untap(b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // Still two tapped: discard and draw two.
    let mut t = TestGame::new(2);
    let vt = t.battlefield(P0, "Vaultguard Trooper");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(vt);
    t.g.tap(b);
    t.hand(P0, "Hill Giant");
    t.set_step(P0, Step::PostcombatMain);
    yes(&mut t, P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn forgotten_creation_upkeep_comes_before_the_draw() {
    cr!("503.1", "504.1", "603.2");
    ruling!(
        "Forgotten Creation",
        "The upkeep step is before the draw step, after the untap step. If you have no cards in hand, you can't draw for the turn and then discard that card to draw a new one because Forgotten Creation's last ability will already have triggered and resolved."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forgotten Creation");
    yes(&mut t, P0, true);
    yes(&mut t, P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(discards(&t, P0), 0);
}

#[test]
fn oath_of_scholars_target_requirement() {
    cr!("115.1", "608.2b", "603.3d");
    ruling!(
        "Oath of Scholars",
        "The targeted player having more cards in their hand than the current player is a part of the targeting requirement. A player can’t be targeted by this ability unless it’s true, and the ability doesn’t resolve if it’s no longer true at that time."
    );
    // P0 has fewer cards than P1: the trigger targets P1; P1 discards down before it
    // resolves: P0 doesn't get to wheel.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Scholars");
    t.hand(P0, "Grizzly Bears");
    let p1h = fill_hand(&mut t, P1, "Hill Giant", 2);
    t.set_step(P1, Step::End);
    yes(&mut t, P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw three cards"), 1);
    for c in p1h {
        to_graveyard(&mut t, c);
    }
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 1);
    // Equal hands: no player can be targeted, so it does nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Scholars");
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    yes(&mut t, P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw three cards"), 0);
}

#[test]
fn leave_returns_a_permanent_you_own_but_dont_control() {
    cr!("108.3", "111.2", "709.3");
    ruling!(
        "Leave // Chance",
        "You are a permanent's owner if the card representing it began the game in your deck, or if it's a token that entered the battlefield under your control. Leave can target a permanent you own but don't control."
    );
    supported("Leave // Chance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    spell(&mut t, P0, "Raise the Alarm", &[]);
    let tok = tokens(&t, P0)[0];
    // P1 gains control of both.
    for o in [bears, tok] {
        t.g.objects[o.0 as usize].controller = P1;
        t.g.objects[o.0 as usize].base_controller = P1;
    }
    t.g.recompute();
    assert_eq!(t.g.obj(bears).controller, P1);
    let leave = t.hand(P0, "Leave // Chance");
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[obj(bears), obj(tok)]);
    t.cast(P0, leave).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.g.is_live(tok));
}

#[test]
fn queen_kayla_finds_cards_discarded_into_exile() {
    cr!("614.6", "608.2c");
    ruling!(
        "Queen Kayla bin-Kroog",
        "If a replacement effect causes cards you discard this way to be exiled rather than go to the graveyard, Queen Kayla bin-Kroog's ability can still find those cards in exile. You can still put cards discarded this way onto the battlefield with Queen Kayla's ability."
    );
    let mut t = TestGame::new(2);
    let qk = t.battlefield(P0, "Queen Kayla bin-Kroog");
    t.battlefield(P1, "Rest in Peace");
    let elves = t.hand(P0, "Llanowar Elves");
    t.lands(P0, "Wastes", 4);
    t.answer_choose_moved(P0, &[obj(elves)]);
    t.activate(P0, qk, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn pyretic_charge_affects_only_creatures_there_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Pyretic Charge",
        "Pyretic Charge affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t get +1/+0 for each card discarded this way."
    );
    supported("Pyretic Charge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    fill_hand(&mut t, P0, "Hill Giant", 2);
    spell(&mut t, P0, "Pyretic Charge", &[]);
    assert_eq!(t.pt(bears), (4, 2));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn wheel_of_misfortune_numbers_are_secret_until_all_are_chosen() {
    cr!("608.2c", "101.4");
    ruling!(
        "Wheel of Misfortune",
        "For each player to secretly choose a number, that player should write down their chosen number without showing it to anyone else. Each player then keeps their number secret until all players simultaneously reveal their numbers."
    );
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P1, DecisionKind::Number, Answer::Number(3));
    // As P1 chooses, P0's number is recorded but secret.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseNumber { .. }),
        |g| {
            let top = *g.stack.last().unwrap();
            g.obj(top).choices.secret.clone()
        },
    );
    let w = spell(&mut t, P0, "Wheel of Misfortune", &[]);
    assert_eq!(*seen.lock().unwrap(), vec![vec![P0]]);
    assert!(t.g.obj(w).choices.secret.is_empty(), "revealed together");
    assert_eq!(t.life(P1), 17);
}
