//! Rulings batch S27 — other rulings about costs: a spell with Phyrexian mana symbols is
//! their color however they were paid (CR 107.4f, 202.2); cards revealed to pay a cost
//! stay revealed while the ability is on the stack and may be revealed again for another
//! cost (CR 701.20a, 701.20c); a player controlling another player pays that player's
//! costs with only that player's resources (CR 723.5a).

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s23_common::{colors_now, colors_of};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::CastMethod;
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn gitaxian_probe_paid_with_life_is_still_blue() {
    cr!("107.4f", "202.2", "105.2");
    ruling!(
        "Gitaxian Probe",
        "A card with Phyrexian mana symbols in its mana cost is each color that appears in that mana cost, regardless of how that cost may have been paid."
    );
    supported("Gitaxian Probe");
    supported("Red Elemental Blast");
    // Gitaxian Probe ({U/P}: "Look at target player's hand. Draw a card.") cast by paying
    // 2 life is a blue spell: Red Elemental Blast ("Counter target blue spell.") can
    // counter it.
    let mut t = TestGame::new(2);
    let probe = t.hand(P0, "Gitaxian Probe");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let spell = t.cast(P0, probe).target(Entity::Player(P1)).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(colors_now(&mut t, spell), colors_of(&[Color::Blue]));
    let hand = t.hand_size(P0);
    t.lands(P1, "Mountain", 1);
    let reb = t.hand(P1, "Red Elemental Blast");
    t.cast(P1, reb).modes(&[0]).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gitaxian Probe"));
    assert_eq!(t.hand_size(P0), hand);
}

/// Activates a Martyr of Spores ("{1}, Reveal X green cards from your hand, Sacrifice
/// this creature: Target creature gets +X/+X until end of turn.") of P0's, revealing
/// `reveal` (X is their number), targeting `target`. Returns the ability on the stack.
fn martyr(t: &mut TestGame, reveal: &[ObjectId], target: ObjectId) -> ObjectId {
    let m = t.battlefield(P0, "Martyr of Spores");
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(reveal.len() as i64));
    t.answer_choose(
        P0,
        &reveal.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>(),
    );
    t.answer_targets(P0, &[Entity::Object(target)]);
    let ability = activate_containing(t, P0, m, "Reveal")
        .unwrap()
        .expect("on the stack");
    t.clear_answers();
    ability
}

#[test]
fn martyr_of_spores_revealed_cards_stay_revealed_until_the_ability_leaves_the_stack() {
    cr!("701.20a", "118.3");
    ruling!(
        "Martyr of Spores",
        "Each card that’s revealed to pay the cost remains revealed until the ability leaves the stack."
    );
    supported("Martyr of Spores");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    let other = t.hand(P0, "Giant Growth");
    martyr(&mut t, &[bears, elves], giant);
    assert!(is_revealed(&t.g, bears));
    assert!(is_revealed(&t.g, elves));
    assert!(!is_revealed(&t.g, other));
    // Still revealed while P1 responds.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve();
    assert!(is_revealed(&t.g, bears));
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    assert!(!is_revealed(&t.g, bears));
    assert!(!is_revealed(&t.g, elves));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn martyr_of_spores_may_reveal_a_card_that_s_already_revealed() {
    cr!("701.20c", "118.3");
    ruling!(
        "Martyr of Spores",
        "A card that’s already revealed for another cost or because of an effect can be revealed to pay this cost."
    );
    supported("Martyr of Spores");
    // Two Martyrs: the second one's cost reveals the Bears already revealed for the first
    // one's, which is still on the stack.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    martyr(&mut t, &[bears], giant);
    assert!(is_revealed(&t.g, bears));
    martyr(&mut t, &[bears], giant);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn sorin_markov_s_controller_uses_only_the_controlled_player_s_resources() {
    cr!("723.5", "723.5a");
    ruling!(
        "Sorin Markov",
        "You can use only the affected player’s resources (cards, mana, and so on) to pay costs for that player; you can’t use your own. Similarly, you can use the affected player’s resources only to pay that player’s costs; you can’t spend them on your costs."
    );
    supported("Sorin Markov");
    // "−7: You control target player during that player's next turn."
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin Markov");
    t.g.objects[sorin.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.activate(P0, sorin, 2, &[Entity::Player(P1)])
        .expect("activate −7");
    t.resolve_all();
    let theirs = t.battlefield(P1, "Mountain");
    let theirs2 = t.battlefield(P1, "Mountain");
    let mine = t.battlefield(P0, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.advance_to(P1, Step::Upkeep);
    // P0 has P1 cast P1's Lightning Bolt: P1's Mountain pays for it, not P0's.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.advance();
    assert!(t.in_graveyard(P1, "Lightning Bolt") || t.stack_len() == 1);
    assert_eq!(
        [theirs, theirs2]
            .iter()
            .filter(|m| t.obj_now(**m).tapped)
            .count(),
        1
    );
    assert!(!t.obj_now(mine).tapped);
    // P0's own Shock can't be paid for with P1's untapped Mountain; with P0's own, it can.
    t.g.objects[mine.0 as usize].tapped = true;
    assert!(t.cast(P0, shock).target(Entity::Player(P1)).try_go().is_err());
    assert!(t.in_hand(P0, "Shock"));
    t.g.objects[mine.0 as usize].tapped = false;
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert!(t.obj_now(mine).tapped);
    assert_eq!(
        [theirs, theirs2]
            .iter()
            .filter(|m| t.obj_now(**m).tapped)
            .count(),
        1
    );
}
