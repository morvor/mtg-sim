//! Rulings batch S32 — hands: opening hands (CR 103.6), discarding (CR 701.9) a hand or
//! a card versus milling it, cards revealed from a hand to pay a cost (CR 701.20), the
//! maximum hand size (CR 402.2, 514.1), a foretell cost reduction (CR 702.143), and
//! protection from each color (CR 702.16).

use crate::r_s01_common::*;
use crate::r_s04_common::{ability_targets, add_mana, spell_targets};
use crate::r_s06_common::has_kw;
use crate::r_s05_common::move_to;
use crate::r_s13_common::pregame;
use crate::r_s25_common::cast_new;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn opening_hand_actions_come_after_mulligans_starting_player_first() {
    cr!("103.6", "103.5", "103.8");
    ruling!(
        "Chancellor of the Forge",
        "A player’s “opening hand” is the hand of cards the player has after all players have taken mulligans. If players have any cards in hand that allow actions to be taken with them from a player’s opening hand, the starting player takes all such actions first in any order, followed by each other player in turn order. Then the first turn begins."
    );
    supported("Chancellor of the Forge");
    // Decks of Chancellor of the Forge ("You may reveal this card from your opening hand.
    // If you do, at the beginning of the first upkeep, create a 1/1 red Phyrexian Goblin
    // creature token with haste."). P1 starts; P0 mulligans once (keeping six cards).
    let deck = || vec![card("Chancellor of the Forge"); 40];
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P1),
            ..Default::default()
        },
        vec![deck(), deck()],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    for p in [P0, P1] {
        for _ in 0..7 {
            t.answer_yes(p, true);
        }
    }
    t.g.start();
    let asked = t.asked();
    let last_mulligan = asked
        .iter()
        .rposition(|(_, d)| matches!(d, Decision::Mulligan { .. }))
        .unwrap();
    let reveals: Vec<(usize, PlayerId)> = asked
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| matches!(d, Decision::YesNo { .. }))
        .map(|(i, (p, _))| (i, *p))
        .collect();
    // Every reveal comes after every mulligan decision (the opening hand is the hand
    // after mulligans): the starting player's seven, then P0's six.
    assert!(reveals.iter().all(|(i, _)| *i > last_mulligan));
    let order: Vec<PlayerId> = reveals.iter().map(|(_, p)| *p).collect();
    assert_eq!(order, [vec![P1; 7], vec![P0; 6]].concat());
    // Then the first turn begins: at its upkeep, a Goblin for each revealed card.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let goblins = |p| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.is_token())
            .count()
    };
    assert_eq!((goblins(P1), goblins(P0)), (7, 6));
}

#[test]
fn you_may_discard_your_hand_even_with_no_cards_in_it() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Narset, Jeskai Waymaster",
        "You may choose to discard your hand even if your hand contains zero cards."
    );
    supported("Narset, Jeskai Waymaster");
    // Narset: "At the beginning of your end step, you may discard your hand. If you do,
    // draw cards equal to the number of spells you've cast this turn." P0 casts two
    // spells and has no cards left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Narset, Jeskai Waymaster");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "discarded the empty hand, then drew two");
}

#[test]
fn milled_cards_arent_discarded() {
    cr!("701.9a", "701.17a");
    ruling!(
        "Ajani's Last Stand",
        "In a Magic game, cards are discarded only from a player’s hand. Effects that put cards from a player’s library into that player’s graveyard do not cause those cards to be discarded."
    );
    supported("Ajani's Last Stand");
    // "When a spell or ability an opponent controls causes you to discard this card, if
    // you control a Plains, create a 4/4 white Avatar creature token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plains");
    t.library_top(P0, "Ajani's Last Stand");
    cast_new(&mut t, P1, "Thought Scour", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ajani's Last Stand"));
    assert!(tokens(&t, P0).is_empty(), "milled: no trigger");
    // Discarded by P1's Mind Rot: the token.
    t.hand(P0, "Ajani's Last Stand");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn x_doesnt_change_when_a_revealed_card_leaves_the_hand() {
    cr!("701.20a", "107.3a", "608.2h");
    ruling!(
        "Martyr of Spores",
        "If one of the cards that’s revealed to pay the cost leaves its owner’s hand before the ability resolves, it stops being revealed, but the value of X is not affected."
    );
    supported("Martyr of Spores");
    // "{1}, Reveal X green cards from your hand, Sacrifice this creature: Target creature
    // gets +X/+X until end of turn." X = 2.
    let mut t = TestGame::new(2);
    let martyr = t.battlefield(P0, "Martyr of Spores");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    let growth = t.hand(P0, "Giant Growth");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(
        P0,
        &[Entity::Object(elves), Entity::Object(growth)],
    );
    t.activate(P0, martyr, 0, &[Entity::Object(bears)])
        .expect("activate Martyr of Spores");
    assert!(t.in_graveyard(P0, "Martyr of Spores"));
    // One of the revealed cards leaves P0's hand before the ability resolves.
    move_to(&mut t, elves, Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn maximum_hand_size_is_checked_only_in_its_players_cleanup_step() {
    cr!("402.2", "514.1");
    ruling!(
        "Jin-Gitaxias, Core Augur",
        "If a player has more cards in their hand than their maximum hand size during the cleanup step of that player's turn, that player discards until they have that many cards. A player's maximum hand size isn't checked at any time other than their own cleanup step."
    );
    supported("Jin-Gitaxias, Core Augur");
    // "Each opponent's maximum hand size is reduced by seven": P1's is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jin-Gitaxias, Core Augur");
    for name in ["Hill Giant", "Shock", "Forest"] {
        t.hand(P1, name);
    }
    // Through P0's turn (and P0's cleanup step), P1 keeps the cards.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.graveyard_size(P1), 0);
    // P0 drew seven at their end step: P0's maximum hand size is still seven.
    assert_eq!(t.hand_size(P0), 7);
    // P1's own cleanup step: P1 discards down to zero (the three, plus the card drawn).
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 4);
}

#[test]
fn the_foretell_cost_reduction_applies_only_to_generic_mana() {
    cr!("702.143a", "118.7", "601.2f");
    ruling!(
        "Ethereal Valkyrie",
        "The reduction of the foretell cost applies only to generic mana in the foretell cost. It can’t reduce requirements of specific colors of mana."
    );
    supported("Ethereal Valkyrie");
    // "Whenever this creature enters or attacks, draw a card, then exile a card from your
    // hand face down. It becomes foretold. Its foretell cost is its mana cost reduced by
    // {2}." Troll Ascetic ({1}{G}{G}) costs {G}{G}; Leatherback Baloth ({G}{G}{G}) still
    // costs {G}{G}{G}.
    const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);
    for (name, cost) in [("Troll Ascetic", 2), ("Leatherback Baloth", 3)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.hand(P0, name);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.enter(P0, "Ethereal Valkyrie");
        t.resolve_all();
        let exiled = t.g.current(c);
        assert_eq!(t.zone(exiled), Zone::Exile, "{name}");
        // On a later turn.
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        let forests = t.lands(P0, "Forest", cost - 1);
        let r = t.cast(P0, exiled).method(FORETELL).try_go();
        assert!(r.is_err(), "{name}: one Forest short");
        for f in forests {
            t.g.objects[f.0 as usize].tapped = false;
        }
        t.lands(P0, "Forest", 1);
        t.cast(P0, exiled).method(FORETELL).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
}

#[test]
fn protection_from_each_color_isnt_protection_from_colorless() {
    cr!("702.16a", "702.16b", "105.2c");
    ruling!(
        "Akroma's Will",
        "\"Protection from each color\" is shorthand for protection from white, from blue, from black, from red, and from green. Colorless is not a color."
    );
    supported("Akroma's Will");
    supported("Rod of Ruin");
    // Mode 2: "Creatures you control gain lifelink, indestructible, and protection from
    // each color until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let will = t.hand(P0, "Akroma's Will");
    t.cast(P0, will).modes(&[1]).go();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Protection));
    // A red spell can't target it; a colorless source's ability can.
    assert!(!spell_targets(&mut t, P1, "Shock").contains(&Entity::Object(bears)));
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert!(ability_targets(&mut t, rod, 0).contains(&Entity::Object(bears)));
    t.lands(P1, "Wastes", 3);
    t.activate(P1, rod, 0, &[Entity::Object(bears)])
        .expect("Rod of Ruin can target it");
    t.resolve_all();
    assert_eq!(t.obj(bears).damage, 1);
}
