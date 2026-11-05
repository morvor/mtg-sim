//! Rulings batch P018 — "discard two cards unless you discard a [type] card" (CR 701.9,
//! 118.12), revealing cards until a card is found (CR 701.20), and naming a card as a
//! spell resolves (CR 608.2c, 201.3).

use crate::r_p018_common::*;
use crate::r_s01_common::{asked_since, stack_library, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    plenty_of_mana(t, p);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let s = t.cast(p, card).go();
    empty_pool(t, p);
    s
}

/// The hand cards that `p` ends up discarding when the "discard two cards unless you
/// discard a [type] card" spell `name` resolves, answering the discard choices with
/// `discard`. Returns the cards discarded (those now in the graveyard).
fn discard_unless(
    t: &mut TestGame,
    caster: PlayerId,
    p: PlayerId,
    name: &str,
    hand: &[ObjectId],
    discard: &[ObjectId],
) -> Vec<ObjectId> {
    let e: Vec<Entity> = discard.iter().map(|c| Entity::Object(*c)).collect();
    // One card of the type: "yes, discard it"; otherwise two cards.
    t.answer_yes(p, discard.len() == 1);
    t.answer_choose(p, &e);
    let targets = if caster == p {
        vec![]
    } else {
        vec![Entity::Player(p)]
    };
    cast(t, caster, name, &targets);
    t.resolve_all();
    hand.iter()
        .copied()
        .filter(|c| t.zone(*c) == Zone::Graveyard(p))
        .collect()
}

#[test]
fn discard_one_card_of_the_type_or_any_two_cards() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Practical Research",
        "As Practical Research resolves, you can discard either one instant or sorcery card or discard two cards, each of which may or may not be an instant or sorcery card."
    );
    ruling!(
        "Thirst for Knowledge",
        "You can discard either one artifact card or two cards which may or may not be artifacts. If you really want to, you can discard two artifact cards."
    );
    ruling!(
        "Wrench Mind",
        "You can discard either one artifact card or two cards which may or may not be artifacts. If you really want to, you can discard two artifact cards."
    );
    ruling!(
        "Mystic Meditation",
        "You may choose to discard a creature card and another card if you want to."
    );
    // (spell, a card of the type, another card of the type, a card not of the type,
    // whether the spell targets P1)
    for (spell, typed, typed2, other, targeted) in [
        (
            "Practical Research",
            "Shock",
            "Stone Rain",
            "Hill Giant",
            false,
        ),
        (
            "Thirst for Knowledge",
            "Mind Stone",
            "Ornithopter",
            "Hill Giant",
            false,
        ),
        (
            "Wrench Mind",
            "Mind Stone",
            "Ornithopter",
            "Hill Giant",
            true,
        ),
        (
            "Mystic Meditation",
            "Hill Giant",
            "Grizzly Bears",
            "Shock",
            false,
        ),
    ] {
        supported(spell);
        let p = if targeted { P1 } else { P0 };
        let run = |choice: &[usize]| -> Vec<ObjectId> {
            let mut t = TestGame::new(2);
            let hand = vec![t.hand(p, typed), t.hand(p, typed2), t.hand(p, other)];
            let picked: Vec<ObjectId> = choice.iter().map(|i| hand[*i]).collect();
            discard_unless(&mut t, P0, p, spell, &hand, &picked)
        };
        // One card of the type.
        assert_eq!(run(&[0]).len(), 1, "{spell}: one {typed}");
        // Two cards, one of the type and one not.
        assert_eq!(run(&[0, 2]).len(), 2, "{spell}: {typed} and {other}");
        // Two cards of the type.
        assert_eq!(run(&[0, 1]).len(), 2, "{spell}: two of the type");
    }
}

#[test]
fn treasure_hunt_takes_the_nonland_card_and_the_lands_above_it() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Treasure Hunt",
        "The cards you put into your hand this way include one nonland card, plus all the land cards on top of it in your library (if any). If the top card of your library is a nonland card, you just get that one."
    );
    ruling!(
        "Treasure Hunt",
        "If all the cards left in your library are lands, you’ll reveal all of them and put them all into your hand."
    );
    supported("Treasure Hunt");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Island", "Hill Giant", "Shock"]);
    cast(&mut t, P0, "Treasure Hunt", &[]);
    t.resolve_all();
    for c in &cards[..3] {
        assert_eq!(t.zone(*c), Zone::Hand(P0));
    }
    assert_eq!(t.zone(cards[3]), Zone::Library(P0));
    // Top card nonland: just that one.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest"]);
    cast(&mut t, P0, "Treasure Hunt", &[]);
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[1]), Zone::Library(P0));
    // Only lands left: all of them.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].library.clear();
    let cards = stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    cast(&mut t, P0, "Treasure Hunt", &[]);
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Hand(P0)));
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.has_lost(P0));
}

// ---------------------------------------------------------------------------
// Demonic Consultation: "Choose a card name. Exile the top six cards of your library,
// then reveal cards from the top of your library until you reveal a card with the chosen
// name. Put that card into your hand and exile all other cards revealed this way."
// ---------------------------------------------------------------------------

const CONSULTATION: &str = "Demonic Consultation";

#[test]
fn demonic_consultation_missing_the_card_exiles_the_library_without_losing_yet() {
    cr!("701.20a", "704.5b", "121.4");
    ruling!(
        "Demonic Consultation",
        "If you don't reveal the named card (perhaps because it was in the top six cards of your library), you'll end up exiling your entire library. You don't lose the game at that point, but will lose the next time you're instructed to draw a card."
    );
    supported(CONSULTATION);
    let mut t = TestGame::new(2);
    // The named card is among the top six.
    stack_library(&mut t, P0, &["Shock", "Hill Giant"]);
    t.answer(P0, DecisionKind::Name, Answer::Text("Hill Giant".into()));
    cast(&mut t, P0, CONSULTATION, &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert!(!t.has_lost(P0));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(!t.has_lost(P0));
    t.g.run_until(10_000, |g| g.players[0].has_lost || g.turn.number > 6);
    assert!(t.has_lost(P0));
}

#[test]
fn demonic_consultation_names_on_resolution_and_affects_only_you() {
    cr!("608.2c", "115.1", "109.5");
    ruling!(
        "Demonic Consultation",
        "There is no way to make this card affect your opponent. It affects \"you\", and \"you\" means the controller of the spell. It has no targets."
    );
    ruling!(
        "Demonic Consultation",
        "You don't name a card until Demonic Consultation resolves."
    );
    let mut t = TestGame::new(2);
    let mut lib = vec!["Shock"; 6];
    lib.push("Hill Giant");
    let mine = stack_library(&mut t, P0, &lib);
    stack_library(&mut t, P1, &["Hill Giant"]);
    let theirs = t.library_size(P1);
    let from = t.asked().len();
    let spell = cast(&mut t, P0, CONSULTATION, &[]);
    assert!(crate::r_s25_common::targets_of(&t, spell).is_empty());
    assert!(asked_since(&t, from).iter().all(|(_, d)| !matches!(
        d,
        Decision::NameCard { .. } | Decision::ChooseTargets { .. }
    )));
    t.answer(P0, DecisionKind::Name, Answer::Text("Hill Giant".into()));
    t.resolve_all();
    assert_eq!(
        asked_since(&t, from)
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::NameCard { .. }))
            .count(),
        1
    );
    assert_eq!(t.zone(mine[6]), Zone::Hand(P0));
    assert!(mine[..6].iter().all(|c| t.zone(*c) == Zone::Exile));
    assert_eq!(t.library_size(P1), theirs);
}

#[test]
fn demonic_consultation_must_name_a_real_card() {
    cr!("201.4");
    ruling!(
        "Demonic Consultation",
        "You must name a card that actually exists in the game of Magic."
    );
    // "Chartreuse" isn't a card: it names nothing. A real card not in the game is fine.
    for (name, valid) in [("Chartreuse", false), ("Black Lotus", true)] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
        let spell = cast(&mut t, P0, CONSULTATION, &[]);
        t.resolve_all();
        let chosen = t.g.obj(spell).choices.card_name.clone();
        assert_eq!(
            chosen.as_deref() == Some(name),
            valid,
            "{name}: chose {chosen:?}"
        );
        assert_eq!(t.library_size(P0), 0);
    }
}
