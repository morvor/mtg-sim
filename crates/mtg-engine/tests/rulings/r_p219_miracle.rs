//! Rulings batch P219 — miracle (CR 702.94): "You may reveal this card from your hand as
//! you draw it if it's the first card you've drawn this turn. When you reveal this card
//! this way, you may cast it by paying [cost] rather than its mana cost." Banishing Stroke
//! ("Put target artifact, creature, or enchantment on the bottom of its owner's library.
//! Miracle {W}"), Bonfire of the Damned, Entreat the Angels.

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use crate::r_s08_common::mana_value;
use crate::r_s11_common::triggered_from;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MIRACLE: CastMethod = CastMethod::Keyword(KeywordKind::Miracle);

/// `p` draws `n` cards (as an effect would) and the triggers are put on the stack.
fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// The spell on the stack named `name`.
fn spell_named(t: &TestGame, name: &str) -> Option<ObjectId> {
    t.g.find_in_zone(Zone::Stack, name).first().copied()
}

#[test]
fn a_miracle_spells_mana_value_comes_from_its_mana_cost() {
    cr!("702.94a", "202.3e", "118.9c");
    ruling!(
        "Bonfire of the Damned",
        "Bonfire of the Damned’s mana value is based on its mana cost of {X}{X}{R}, even if you’re casting it for its miracle cost. For example, if you cast Bonfire of the Damned for its miracle cost and choose 4 for X, its mana value is 9."
    );
    ruling!(
        "Entreat the Angels",
        "Entreat the Angels's mana value is based on its mana cost of {X}{X}{W}{W}{W}, even if you're casting it for its miracle cost. For example, if you cast Entreat the Angels for its miracle cost and choose 4 for X, its mana value is 11."
    );
    supported("Bonfire of the Damned");
    supported("Entreat the Angels");
    for (name, land, mv) in [
        ("Bonfire of the Damned", "Mountain", 9),
        ("Entreat the Angels", "Plains", 11),
    ] {
        let mut t = TestGame::new(2);
        t.library_top(P0, name);
        // Miracle {X}{R} / {X}{W}{W} with X = 4.
        t.lands(P0, land, if land == "Plains" { 2 } else { 1 });
        t.lands(P0, "Wastes", 4);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::X, Answer::Number(4));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        draw(&mut t, P0, 1);
        t.resolve();
        let spell = spell_named(&t, name).expect("cast for its miracle cost");
        assert_eq!(mana_value(&t, spell), mv, "{name}");
        let paid = if land == "Plains" { 6 } else { 5 };
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), paid, "{name}");
        t.resolve_all();
        if name == "Entreat the Angels" {
            assert_eq!(with_subtype(&t, P0, "Angel").len(), 4);
        } else {
            assert_eq!(t.life(P1), 16);
        }
    }
}

#[test]
fn a_miracle_card_that_leaves_your_hand_cant_be_cast() {
    cr!("702.94a", "603.11");
    ruling!(
        "Banishing Stroke",
        "If the card with miracle leaves your hand before the triggered ability resolves, you won’t be able to cast it using its miracle ability."
    );
    supported("Banishing Stroke");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stroke = t.library_top(P0, "Banishing Stroke");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    let stroke = t.g.current(stroke);
    assert_eq!(triggered_from(&t, stroke), 1);
    t.g.discard(P0, stroke, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Banishing Stroke"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn miracle_cant_be_combined_with_casting_without_paying_the_mana_cost() {
    cr!("702.94a", "118.9a");
    ruling!(
        "Banishing Stroke",
        "Miracle is an alternative cost to cast the spell with miracle. It can’t be combined with other alternative costs, such as casting a spell “without paying its mana cost.”"
    );
    supported("Omniscience");
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    // With no mana, the miracle cost {W} can't be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stroke = t.library_top(P0, "Banishing Stroke");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    draw(&mut t, P0, 1);
    t.resolve_all();
    let stroke = t.g.current(stroke);
    assert_eq!(t.zone(stroke), Zone::Hand(P0));
    assert!(t.on_battlefield(bears));
    // Omniscience's own permission still works for casting it from hand.
    let methods = cast_methods(&mut t, P0, stroke);
    assert!(!methods.contains(&MIRACLE));
    assert!(!methods.is_empty());
}

#[test]
fn only_the_first_of_several_cards_drawn_can_be_a_miracle() {
    cr!("702.94a", "121.2");
    ruling!(
        "Banishing Stroke",
        "Multiple card draws are always treated as a sequence of individual card draws. For example, if you haven’t drawn any cards yet during a turn and cast a spell that instructs you to draw three cards, you’ll draw them one at a time. Only the first card drawn this way may be revealed and cast using its miracle ability."
    );
    // The miracle card is the second of three.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let cards = stack_library(&mut t, P0, &["Forest", "Banishing Stroke", "Island"]);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 3);
    assert_eq!(triggered_from(&t, t.g.current(cards[1])), 0);
    assert_eq!(t.stack_len(), 0);
    // The first of three.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let cards = stack_library(&mut t, P0, &["Banishing Stroke", "Forest", "Island"]);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 3);
    assert_eq!(triggered_from(&t, t.g.current(cards[0])), 1);
}

#[test]
fn the_total_cost_starts_from_the_miracle_cost() {
    cr!("601.2f", "702.94a", "202.3");
    ruling!(
        "Banishing Stroke",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a miracle cost) you’re paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Thalia, Guardian of Thraben");
    // Thalia: "Noncreature spells cost {1} more to cast." {W} + {1}.
    for (lands, cast) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Thalia, Guardian of Thraben");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.library_top(P0, "Banishing Stroke");
        t.lands(P0, "Plains", lands);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        draw(&mut t, P0, 1);
        t.resolve();
        let spell = spell_named(&t, "Banishing Stroke");
        assert_eq!(spell.is_some(), cast, "{lands} lands");
        if let Some(spell) = spell {
            assert_eq!(mana_value(&t, spell), 6);
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
        }
    }
}

#[test]
fn no_legal_target_means_no_miracle_cast_later() {
    cr!("702.94a", "601.2c", "603.11");
    ruling!(
        "Banishing Stroke",
        "You can cast a card for its miracle cost only as the miracle triggered ability resolves. If you don’t want to cast it at that time (or you can’t cast it, perhaps because there are no legal targets available), you won’t be able to cast it later for the miracle cost."
    );
    let mut t = TestGame::new(2);
    let stroke = t.library_top(P0, "Banishing Stroke");
    t.lands(P0, "Plains", 6);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    t.resolve_all();
    let stroke = t.g.current(stroke);
    assert_eq!(t.zone(stroke), Zone::Hand(P0));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    // A target appears: only its mana cost is available.
    t.battlefield(P1, "Grizzly Bears");
    let methods = cast_methods(&mut t, P0, stroke);
    assert!(methods.contains(&CastMethod::Normal));
    assert!(!methods.contains(&MIRACLE));
}

#[test]
fn miracle_works_on_another_players_turn_ignoring_card_type_timing() {
    cr!("702.94a", "608.2g");
    ruling!(
        "Banishing Stroke",
        "You can reveal and cast a card with miracle on any turn, not just your own, if it’s the first card you’ve drawn that turn."
    );
    ruling!(
        "Banishing Stroke",
        "You cast the card with miracle during the resolution of the triggered ability. Ignore any timing rules based on the card’s type."
    );
    // Bonfire of the Damned (a sorcery) during P1's upkeep.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Bonfire of the Damned");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(bears).damage, 1);
    // Banishing Stroke on P1's turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Banishing Stroke");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1));
}

#[test]
fn not_revealing_or_not_casting_keeps_the_drawn_card_in_hand() {
    cr!("702.94a", "121.1", "603.2");
    ruling!(
        "Banishing Stroke",
        "You don’t have to reveal a drawn card with miracle if you don’t wish to cast it at that time."
    );
    ruling!(
        "Banishing Stroke",
        "You still draw the card, whether you use the miracle ability or not. Any ability that triggers whenever you draw a card, for example, will trigger. If you don’t cast the card using its miracle ability, it will remain in your hand."
    );
    supported("Psychosis Crawler");
    // Not revealed: no miracle trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let stroke = t.library_top(P0, "Banishing Stroke");
    t.answer_yes(P0, false);
    draw(&mut t, P0, 1);
    let stroke = t.g.current(stroke);
    assert_eq!(triggered_from(&t, stroke), 0);
    assert_eq!(t.zone(stroke), Zone::Hand(P0));
    // Revealed but not cast: Psychosis Crawler's draw trigger ("Whenever you draw a card,
    // each opponent loses 1 life.") triggers, and the card stays in hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychosis Crawler");
    t.battlefield(P1, "Grizzly Bears");
    let stroke = t.library_top(P0, "Banishing Stroke");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.zone(stroke), Zone::Hand(P0));
    assert_eq!(t.life(P1), 19);
}
