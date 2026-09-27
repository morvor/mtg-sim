//! Rulings batch S05 — discover (CR 701.57): "Exile cards from the top of your library
//! until you exile a nonland card with mana value N or less. Cast it without paying its
//! mana cost or put it into your hand. Put the rest on the bottom in a random order."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The options offered by each "choose what to cast" decision asked of P0 since `from`.
fn cast_choices(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if *p == P0 && prompt.contains("what to cast") => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// Primordial Gnawer ("When this creature dies, discover 3") dies under P0's control.
fn gnawer_dies(t: &mut TestGame) {
    let gnawer = t.battlefield(P0, "Primordial Gnawer");
    destroy(t, gnawer);
    t.resolve_all();
}

#[test]
fn a_split_card_is_found_by_its_combined_mana_value_and_either_half_can_be_cast() {
    cr!("701.57a", "709.4", "709.3");
    ruling!(
        "Primordial Gnawer",
        "The mana value of a split card is determined by the combined mana cost of its two halves. If discover allows you to cast a split card, you may cast either half (as long as its mana value is less than or equal to the effect's discover value) but not both halves."
    );
    supported("Primordial Gnawer");
    supported("Wear // Tear");
    // Fire // Ice ({1}{R} // {1}{U}) has mana value 4: discover 3 passes over it.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Fire // Ice", "Grizzly Bears"]);
    t.answer_yes(P0, false);
    gnawer_dies(&mut t);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Fire // Ice"));
    // Wear // Tear ({1}{R} // {W}) has mana value 3: it's found, and either half can be
    // cast — one of them, not both.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    stack_library(&mut t, P0, &["Wear // Tear"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    let from = t.asked().len();
    gnawer_dies(&mut t);
    assert_eq!(
        cast_choices(&t, from),
        vec![vec!["Cast Wear".to_string(), "Cast Tear".to_string()]]
    );
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    assert!(t.on_battlefield(relic));
    assert!(t.in_graveyard(P0, "Wear // Tear"));
}

#[test]
fn an_adventurer_card_can_be_cast_only_as_a_spell_with_small_enough_mana_value() {
    cr!("701.57a", "715.3", "715.4");
    ruling!(
        "Walk with the Ancestors",
        "If you discover an adventurer card, split card, or modal double-faced card, you might be able to cast that card with either set of characteristics depending on the effect's discover value."
    );
    supported("Walk with the Ancestors");
    supported("Hit the Mother Lode");
    supported("Galvanic Giant");
    // Galvanic Giant ({3}{U}) // Storm Reading ({5}{U}{U}). Discover 4: only the Giant.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Galvanic Giant"]);
    give_mana_for(&mut t, P0, "Walk with the Ancestors");
    let walk = t.hand(P0, "Walk with the Ancestors");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.cast(P0, walk).targets(&[]).go();
    t.resolve_all();
    assert!(cast_choices(&t, from).is_empty());
    assert_eq!(t.named_on_battlefield("Galvanic Giant").len(), 1);
    // Discover 10: either one. Storm Reading is cast (draw four, discard two), and the
    // card goes on an adventure.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Galvanic Giant"]);
    give_mana_for(&mut t, P0, "Hit the Mother Lode");
    let lode = t.hand(P0, "Hit the Mother Lode");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.cast(P0, lode).go();
    t.resolve_all();
    assert_eq!(
        cast_choices(&t, from),
        vec![vec![
            "Cast Galvanic Giant".to_string(),
            "Cast Storm Reading".to_string()
        ]]
    );
    assert!(t.named_on_battlefield("Galvanic Giant").is_empty());
    assert!(t.in_exile("Galvanic Giant"));
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn a_discovered_card_with_x_is_cast_with_x_0() {
    cr!("701.57a", "107.3b");
    ruling!(
        "Primordial Gnawer",
        "If the discovered card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Walking Ballista");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Walking Ballista"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let from = t.asked().len();
    gnawer_dies(&mut t);
    // No value of X was asked: it's 0, so the Ballista enters with no counters and dies.
    assert_eq!(
        asked_of_since(&t, P0, from, |d| matches!(d, Decision::ChooseX { .. })),
        0
    );
    assert!(t.in_graveyard(P0, "Walking Ballista"));
    assert!(t.named_on_battlefield("Walking Ballista").is_empty());
}

#[test]
fn a_discover_spell_whose_targets_are_all_illegal_doesnt_discover() {
    cr!("608.2b", "701.57a");
    ruling!(
        "Daring Discovery",
        "Some spells and abilities that cause you to discover may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve and you won't discover."
    );
    supported("Daring Discovery");
    // "Up to three target creatures can't block this turn. Discover 4."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant"]);
    give_mana_for(&mut t, P0, "Daring Discovery");
    let spell = t.hand(P0, "Daring Discovery");
    t.cast(P0, spell).targets(&[Entity::Object(bears)]).go();
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Daring Discovery"));
    assert!(t.g.exile.is_empty());
    assert_eq!(t.g.library_top(P0).map(|c| t.obj(c).chars.name.to_string()), Some("Hill Giant".into()));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn a_discovered_card_that_cant_be_cast_goes_to_hand() {
    cr!("701.57a", "601.2c");
    ruling!(
        "Primordial Gnawer",
        "If you can't cast the discovered card (perhaps because there are no legal targets for the spell), you'll put it into your hand."
    );
    supported("Murder");
    // Murder ("Destroy target creature") with no creature on the battlefield.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Murder"]);
    t.answer_yes(P0, true);
    gnawer_dies(&mut t);
    assert!(t.in_hand(P0, "Murder"));
    assert!(t.g.exile.is_empty());
}

#[test]
fn a_discovered_spell_can_have_additional_costs_but_no_alternative_cost() {
    cr!("701.57a", "118.9", "118.8", "601.2b", "601.2f");
    ruling!(
        "Trumpeting Carnosaur",
        "If you cast a spell \"without paying its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast it."
    );
    supported("Village Rites");
    supported("Burst Lightning");
    supported("Mulldrifter");
    // Village Rites: "As an additional cost to cast this spell, sacrifice a creature."
    // Without a creature to sacrifice (the Gnawer died), it can't be cast, and goes to
    // the hand.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Village Rites"]);
    t.answer_yes(P0, true);
    gnawer_dies(&mut t);
    assert!(t.in_hand(P0, "Village Rites"));
    // With a creature, the creature is sacrificed as it's cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Village Rites"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hand = t.hand_size(P0);
    gnawer_dies(&mut t);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Village Rites"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Kicker, an optional additional cost, may be paid: Burst Lightning deals 4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    stack_library(&mut t, P0, &["Burst Lightning"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    enter(&mut t, P0, "Trumpeting Carnosaur");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(tapped_lands(&t, P0), 4);
    // Mulldrifter's evoke cost, an alternative cost, isn't an option: it isn't sacrificed.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Mulldrifter"]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    enter(&mut t, P0, "Trumpeting Carnosaur");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
    assert_eq!(
        asked_of_since(&t, P0, from, |d| matches!(
            d,
            Decision::ChooseCastingMethod { .. }
        )),
        0
    );
}
