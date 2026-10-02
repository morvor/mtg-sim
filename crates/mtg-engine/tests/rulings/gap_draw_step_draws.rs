//! "Except the first one they draw in each of their draw steps" (gap-trigger-timing, G30):
//! replacement effects and triggered abilities that leave out the card a player draws in
//! their draw step (CR 504.1), and the "if a player would draw a card, instead ..."
//! replacement effects they are written as (CR 121.6, 614.1a, 614.5, 616.1).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s draw step (its turn-based draw done), triggers on the stack.
fn to_draw_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::Draw);
    t.settle();
}

/// `p` casts Opt ("Scry 1. Draw a card.") and it resolves.
fn opt(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Island", 1);
    let opt = t.hand(p, "Opt");
    t.cast(p, opt).go();
    t.resolve();
}

/// `p` casts Quick Study (an instant: "Draw two cards.") and it resolves.
fn quick_study(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Island", 3);
    let spell = t.hand(p, "Quick Study");
    t.cast(p, spell).go();
    t.resolve();
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Treasure"))
        .count()
}

/// The number of times `p` was asked to order replacement effects.
fn replacement_choices(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::ChooseReplacement { .. }))
        .count()
}

/// Queues `p`'s answer to a replacement-order choice: the option at `index`.
fn choose_replacement(t: &mut TestGame, p: PlayerId, index: usize) {
    t.answer(p, DecisionKind::Replacement, Answer::Index(index));
}

#[test]
fn notion_thief_still_lets_the_opponent_discard_after_a_stolen_draw() {
    cr!("614.1a", "614.6", "504.1");
    ruling!(
        "Notion Thief",
        "If an opponent is instructed to draw a card then discard a card, and Notion Thief causes you to draw a card instead, that opponent still discards a card."
    );
    supported("Notion Thief");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Notion Thief");
    // "{T}: Draw a card, then discard a card."
    let looter = t.battlefield(P1, "Merfolk Looter");
    to_draw_step(&mut t, P1);
    // The draw step's card isn't stolen.
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P0), 0);
    let gy = t.graveyard_size(P1);
    t.activate(P1, looter, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "P0 drew instead");
    assert_eq!(t.hand_size(P1), 0, "P1 still discarded");
    assert_eq!(t.graveyard_size(P1), gy + 1);
}

#[test]
fn two_players_notion_thieves_give_the_card_back_to_the_drawing_player() {
    cr!("614.5", "616.1", "616.1e");
    ruling!(
        "Notion Thief",
        "If two or more players each control a Notion Thief and a player would draw a card other than the first one in their draw step, that player chooses one of the applicable Notion Thief effects to apply."
    );
    ruling!(
        "Notion Thief",
        "The above procedure means that if each player in a two-player game controls a Notion Thief and one would draw a card, it really will be that player who draws a card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Notion Thief");
    t.battlefield(P1, "Notion Thief");
    to_draw_step(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    // P1's second card: P0's Notion Thief makes P0 draw instead; that draw is P1's
    // Notion Thief's to replace, so P1 draws; P0's Notion Thief has already applied.
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn several_hullbreachers_create_only_one_treasure() {
    cr!("614.5", "614.6");
    ruling!(
        "Hullbreacher",
        "If you control multiple Hullbreachers while an opponent would draw a card except their first one in their draw step, you'll create only one Treasure."
    );
    supported("Hullbreacher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hullbreacher");
    t.battlefield(P0, "Hullbreacher");
    to_draw_step(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(treasures(&t, P0), 0);
    opt(&mut t, P1);
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.hand_size(P1), 1, "the drawn card was replaced");
}

#[test]
fn hullbreacher_and_a_draw_doubler_the_drawing_player_chooses_the_order() {
    cr!("616.1", "616.1e", "614.5");
    ruling!(
        "Hullbreacher",
        "If multiple replacement effects apply to the same card draw, the player drawing the card chooses the order in which to apply them."
    );
    ruling!(
        "Alhammarret's Archive",
        "If two or more replacement effects would apply to a card-drawing event, the player drawing the card chooses the order in which to apply them."
    );
    supported("Alhammarret's Archive");
    let mut seen = Vec::new();
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hullbreacher");
        t.battlefield(P1, "Alhammarret's Archive");
        to_draw_step(&mut t, P1);
        assert_eq!(t.hand_size(P1), 1);
        assert_eq!(replacement_choices(&t, P1), 0);
        choose_replacement(&mut t, P1, pick);
        opt(&mut t, P1);
        let options = t
            .asked()
            .iter()
            .find_map(|(q, d)| match d {
                Decision::ChooseReplacement { options } if *q == P1 => Some(options.clone()),
                _ => None,
            })
            .expect("P1 chose the order");
        let archive_first = options[pick].contains("draw two");
        seen.push(archive_first);
        assert_eq!(t.hand_size(P1), 1, "P1 drew nothing");
        if archive_first {
            // Two draws, each replaced by Hullbreacher.
            assert_eq!(treasures(&t, P0), 2);
        } else {
            // The draw is gone: the Archive has nothing left to replace.
            assert_eq!(treasures(&t, P0), 1);
        }
    }
    assert!(seen.contains(&true) && seen.contains(&false));
}

#[test]
fn chains_of_mephistopheles_exempts_the_draw_step_draw_and_replaces_each_other_draw() {
    cr!("121.2", "504.1", "614.1a");
    ruling!(
        "Chains of Mephistopheles",
        "A player’s normal card draw on their turn is exempt from this effect. All other draws will be affected."
    );
    ruling!(
        "Chains of Mephistopheles",
        "If a spell or ability would cause a player to draw multiple cards, this is treated as a number of individual “draw one card” actions. Apply the effect of Chains of Mephistopheles to each one."
    );
    ruling!(
        "Chains of Mephistopheles",
        "If that player has at least one card in their hand, they discard a card and then draws a card."
    );
    supported("Chains of Mephistopheles");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chains of Mephistopheles");
    to_draw_step(&mut t, P1);
    // The normal draw happened.
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 0);
    // Quick Study: two individual draws, each one replaced by "discard, then draw".
    quick_study(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    // Quick Study itself, and a card discarded for each of the two draws.
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn chains_of_mephistopheles_with_an_empty_hand_mills_instead() {
    cr!("614.1a", "701.17a");
    ruling!(
        "Chains of Mephistopheles",
        "If that player’s hand is empty, they put the top card of their library into their graveyard. The player doesn’t draw a card at all."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chains of Mephistopheles");
    t.set_step(P1, Step::PrecombatMain);
    let library = t.library_size(P1);
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P1), 0);
    // Opt and the milled card.
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.library_size(P1), library - 1);
}

#[test]
fn two_chains_of_mephistopheles_discard_then_mill() {
    cr!("614.5", "616.1");
    ruling!(
        "Chains of Mephistopheles",
        "The effect is cumulative. If there are two of these on the battlefield, each of them will modify each draw (after the first one if during the draw step), and will cause the player to discard or to “mill” a card from their library."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chains of Mephistopheles");
    t.battlefield(P0, "Chains of Mephistopheles");
    t.set_step(P1, Step::PrecombatMain);
    t.hand(P1, "Grizzly Bears");
    let library = t.library_size(P1);
    opt(&mut t, P1);
    // One Chains: discard the Bears, then draw — that draw is replaced by the other Chains:
    // the hand is empty now, so a card is milled instead.
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.library_size(P1), library - 1);
}

#[test]
fn magus_of_the_chains_works_like_chains_of_mephistopheles() {
    cr!("121.2", "504.1", "614.5");
    ruling!(
        "Magus of the Chains",
        "If an effect instructs you to draw multiple cards, you draw those cards one at a time. Apply the effect of Magus of the Chains to each one."
    );
    ruling!(
        "Magus of the Chains",
        "If that player has at least one card in their hand, they discard a card and then draw a card."
    );
    ruling!(
        "Magus of the Chains",
        "If that player’s hand is empty, they mill a card. The player doesn’t draw a card at all."
    );
    ruling!(
        "Magus of the Chains",
        "The effects of more than one Magus of the Chains are cumulative. If there are two of these on the battlefield, each of them will modify each draw (after the first one if during the draw step) and will cause the player to discard or to mill a card."
    );
    supported("Magus of the Chains");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Chains");
    to_draw_step(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    quick_study(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 3);
    // Two of them: discard, then the draw becomes a mill.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Chains");
    t.battlefield(P0, "Magus of the Chains");
    t.set_step(P1, Step::PrecombatMain);
    t.hand(P1, "Grizzly Bears");
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 3);
    // Empty hand: a card is milled and none drawn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Chains");
    t.set_step(P1, Step::PrecombatMain);
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn two_teferis_ageless_insights_make_each_later_draw_four() {
    cr!("614.5", "504.1");
    ruling!(
        "Teferi's Ageless Insight",
        "However, if that happens, each card that player would draw after the first will result in four cards being drawn."
    );
    supported("Teferi's Ageless Insight");
    let mut t = TestGame::new(2);
    // "The "legend rule" doesn't apply."
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Teferi's Ageless Insight");
    t.battlefield(P0, "Teferi's Ageless Insight");
    to_draw_step(&mut t, P0);
    // The draw step's card is drawn normally.
    let hand = t.hand_size(P0);
    opt(&mut t, P0);
    // Opt left the hand; its one card became four.
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn putting_a_card_into_your_hand_isnt_drawing_it() {
    cr!("121.1");
    ruling!(
        "Teferi's Ageless Insight",
        "If a spell or ability causes you to put a card into your hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    ruling!(
        "Orcish Bowmasters",
        "If a spell or ability causes an opponent to put cards into their hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    supported("Orcish Bowmasters");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi's Ageless Insight");
    t.battlefield(P1, "Orcish Bowmasters");
    t.battlefield(P1, "Xyris, the Writhing Storm");
    t.lands(P0, "Island", 2);
    let impulse = t.hand(P0, "Impulse");
    t.cast(P0, impulse).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "one card, not doubled");
    t.settle();
    assert_eq!(t.stack_len(), 0, "neither Bowmasters nor Xyris triggered");
}

#[test]
fn bowmasters_and_xyris_trigger_on_each_card_but_the_draw_step_one() {
    cr!("504.1", "603.2", "121.2");
    ruling!(
        "Xyris, the Writhing Storm",
        "If a spell or ability causes an opponent to put cards into their hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    let mut t = TestGame::new(2);
    // "When this creature enters and whenever an opponent draws a card except the first
    // one they draw in each of their draw steps, this creature deals 1 damage to any
    // target. Then amass Orcs 1."
    t.battlefield(P0, "Orcish Bowmasters");
    // "Whenever an opponent draws a card except the first one they draw in each of their
    // draw steps, create a 1/1 green Snake creature token."
    t.battlefield(P0, "Xyris, the Writhing Storm");
    to_draw_step(&mut t, P1);
    assert_eq!(t.stack_len(), 0);
    // Bowmasters' targets.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    quick_study(&mut t, P1);
    t.settle();
    // Two cards, two triggers each.
    assert_eq!(t.stack_len(), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Two Snake tokens (Xyris is a Snake too).
    assert_eq!(with_subtype(&t, P0, "Snake").len(), 3);
    assert_eq!(with_subtype(&t, P0, "Army").len(), 1);
}

#[test]
fn two_bards_or_archives_multiply_draws_by_four() {
    cr!("614.5", "504.1");
    ruling!(
        "Bard, King of Dale",
        "However, if that happens, cards drawn by that player will be multiplied by four."
    );
    ruling!(
        "Alhammarret's Archive",
        "Similarly, the effects of the last abilities of multiple Archives are cumulative. If you control two, you’ll draw four times the number of cards, and so on."
    );
    supported("Bard, King of Dale");
    for name in ["Bard, King of Dale", "Alhammarret's Archive"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        t.battlefield(P0, name);
        t.battlefield(P0, name);
        to_draw_step(&mut t, P0);
        let hand = t.hand_size(P0);
        opt(&mut t, P0);
        assert_eq!(t.hand_size(P0), hand + 4, "{name}");
    }
}

#[test]
fn teferis_ageless_insight_and_bard_the_drawing_player_orders_them() {
    cr!("616.1", "616.1e");
    ruling!(
        "Teferi's Ageless Insight",
        "If two or more replacement effects would apply to a card-drawing event, the player drawing the card chooses the order in which to apply them."
    );
    ruling!(
        "Bard, King of Dale",
        "If two or more replacement effects would apply to a card-drawing event, the player drawing the card chooses the order in which to apply them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi's Ageless Insight");
    t.battlefield(P0, "Bard, King of Dale");
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    opt(&mut t, P0);
    assert!(replacement_choices(&t, P0) >= 1);
    assert_eq!(t.hand_size(P0), hand + 4);
}

/// Leaves `n` cards in `p`'s library.
fn library_of(t: &mut TestGame, p: PlayerId, n: usize) {
    t.g.players[p.idx()].library.clear();
    for _ in 0..n {
        t.library_top(p, "Island");
    }
}

/// Xyris (3/5 flying) attacks P1 unblocked; runs until the end of combat or the end of
/// the game.
fn xyris_hits(t: &mut TestGame) {
    use mtg_engine::turn::Stage;
    let xyris = t.battlefield(P0, "Xyris, the Writhing Storm");
    attack_with(t, &[(xyris, Entity::Player(P1))]);
    let ok = t.g.run_until(10_000, |g| {
        g.result.is_some()
            || (g.turn.step == Step::EndOfCombat && g.turn.stage == Stage::Priority)
    });
    assert!(ok);
}

#[test]
fn xyris_you_and_that_player_each_draw_and_lose_together() {
    cr!("121.2c", "704.5b", "104.4a");
    ruling!(
        "Xyris, the Writhing Storm",
        "If the amount of damage Xyris deals is greater than the number of cards in your library and the number of cards in that player's library, you both lose the game at the same time. It doesn't matter if one of you had more cards in library than the other. If there are no other players left in the game, the game is a draw."
    );
    supported("Xyris, the Writhing Storm");
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 2);
    library_of(&mut t, P1, 1);
    xyris_hits(&mut t);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.result, Some(mtg_engine::game::GameResult::Draw));
}

#[test]
fn xyris_each_draw_that_many_cards() {
    cr!("121.2c", "510.2");
    let mut t = TestGame::new(2);
    xyris_hits(&mut t);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.hand_size(P1), 3);
}

#[test]
fn xyris_a_player_dealt_lethal_damage_loses_before_you_draw() {
    cr!("704.5a", "704.3", "104.2a");
    ruling!(
        "Xyris, the Writhing Storm",
        "If the amount of damage Xyris deals is greater than the number of cards in your library but causes the defending player's life total to become 0 or less, that player loses the game before Xyris's ability causes you to draw cards. If there are no other players left in the game, you win the game."
    );
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 1);
    t.g.players[P1.idx()].life = 3;
    xyris_hits(&mut t);
    assert_eq!(t.g.result, Some(mtg_engine::game::GameResult::Win(vec![P0])));
    assert_eq!(t.hand_size(P0), 0);
}
