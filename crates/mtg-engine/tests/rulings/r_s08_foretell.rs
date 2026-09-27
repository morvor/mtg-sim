//! Rulings batch S08 — foretell (CR 702.143): "During your turn, you may pay {2} and exile
//! this card from your hand face down. Cast it on a later turn for its foretell cost."
//! Foretelling is a special action (CR 116.2h); casting a foretold card follows the card's
//! timing rules; the foretell cost is an alternative cost (CR 118.9).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s08_common::*;
use mtg_engine::ability::Duration;
use mtg_engine::casting::PlayGrant;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);

fn can_foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    actions_of(t, p).contains(&Action::Special(SpecialAction::Foretell { card }))
}

/// Foretells `card` from `p`'s hand (paying {2} from the mana pool); the card in exile.
fn foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    add_mana(t, p, ManaType::C, 2);
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    let new = t.g.current(card);
    assert_eq!(t.obj(new).zone, Zone::Exile);
    assert!(t.obj(new).face_down);
    new
}

#[test]
fn foretelling_is_a_special_action_taken_during_your_turn_even_in_response() {
    cr!("702.143a", "116.2h", "116.3");
    ruling!(
        "Saw It Coming",
        "Because exiling a card with foretell from your hand is a special action, you can do so any time you have priority during your turn, including in response to spells and abilities."
    );
    ruling!(
        "Demon Bolt",
        "Because exiling a card with foretell from your hand is a special action, you can do so any time you have priority during your turn, including in response to spells and abilities. Once you announce you're taking the action, no other player can respond by trying to remove the card from your hand."
    );
    supported("Saw It Coming");
    supported("Demon Bolt");
    let mut t = TestGame::new(2);
    let saw = t.hand(P0, "Saw It Coming");
    let bolt = t.hand(P0, "Demon Bolt");
    // In response to P1's Lightning Bolt in P0's turn: the special action doesn't use the
    // stack, and P1 gets no chance to act while it's taken.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Lightning Bolt");
    t.cast(P1, shock).target(P0).go();
    t.lands(P0, "Wastes", 2);
    assert!(can_foretell(&mut t, P0, saw));
    let from = t.asked().len();
    foretell(&mut t, P0, saw);
    assert_eq!(t.stack_len(), 1);
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    t.resolve_all();
    // In P0's combat: yes. In P1's turn: no.
    t.set_step(P0, Step::DeclareBlockers);
    assert!(can_foretell(&mut t, P0, bolt));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_foretell(&mut t, P0, bolt));
}

#[test]
fn a_foretold_card_is_cast_on_a_later_turn_following_its_timing_rules() {
    cr!("702.143a", "304.1", "307.1");
    ruling!(
        "Crush the Weak",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player’s turn. In most cases, if you foretell a card that isn’t an instant (or doesn’t have flash), you’ll have to wait until your next turn to cast it."
    );
    ruling!(
        "Demon Bolt",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player's turn."
    );
    ruling!(
        "Ultimate Magic: Holy",
        "In most cases, if you foretell a card that isn't an instant, you'll have to wait until your next turn to cast it."
    );
    supported("Crush the Weak");
    supported("Ultimate Magic: Holy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 3);
    let bolt = t.hand(P0, "Demon Bolt");
    let crush = t.hand(P0, "Crush the Weak");
    let holy = t.hand(P0, "Ultimate Magic: Holy");
    let bolt = foretell(&mut t, P0, bolt);
    let crush = foretell(&mut t, P0, crush);
    let holy = foretell(&mut t, P0, holy);
    t.battlefield(P1, "Grizzly Bears");
    // Not the turn they were foretold.
    for c in [bolt, crush, holy] {
        assert!(!can_cast(&mut t, P0, c, FORETELL));
    }
    // P1's turn: the instants, not the sorcery.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, bolt, FORETELL));
    assert!(can_cast(&mut t, P0, holy, FORETELL));
    assert!(!can_cast(&mut t, P0, crush, FORETELL));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, crush, FORETELL));
    // P0's next main phase: the sorcery too.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, crush, FORETELL));
    t.cast(P0, crush).method(FORETELL).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears") || t.in_exile("Grizzly Bears"));
}

#[test]
fn a_foretold_card_cast_for_its_foretell_cost_takes_additional_costs_but_no_other_alternative_cost(
) {
    cr!("702.143a", "118.9a", "118.8", "702.33a");
    ruling!(
        "Saw It Coming",
        "If you’re casting a foretold card from exile for its foretell cost, you can’t choose to cast it for any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    ruling!(
        "Demon Bolt",
        "If you're casting a foretold card from exile for its foretell cost, you can't choose to cast it for any other alternative costs. You can, however, pay additional costs, such as kicker costs."
    );
    // Demon Bolt foretold, with an effect that would let P0 cast it without paying its mana
    // cost: cast for its foretell cost, it's not also free.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Demon Bolt");
    let bolt = foretell(&mut t, P0, bolt);
    t.advance_to(P1, Step::Upkeep);
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: bolt,
        duration: Duration::EndOfTurn,
        free: true,
        source: None,
        turn: t.g.turn.number,
    });
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(!can_cast(&mut t, P0, bolt, FORETELL));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, bolt, FORETELL));
    t.cast(P0, bolt).method(FORETELL).target(giant).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Kicker (an optional additional cost) can be paid on top of the foretell cost.
    let kicked = custom_card(
        "Foretold Kicker Bolt",
        "Instant",
        "{2}{R}",
        None,
        "Kicker {1}\n~ deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.\nForetell {R}",
    );
    let mut t = TestGame::new(2);
    let card = t.custom(P0, kicked, Zone::Hand(P0));
    let card = foretell(&mut t, P0, card);
    t.advance_to(P1, Step::Upkeep);
    t.lands(P0, "Mountain", 2);
    t.cast(P0, card).method(FORETELL).kicked(true).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A mandatory additional cost must be paid.
    let sac = custom_card(
        "Foretold Offering",
        "Instant",
        "{1}{B}",
        None,
        "As an additional cost to cast this spell, sacrifice a creature.\nDraw two cards.\nForetell {B}",
    );
    let mut t = TestGame::new(2);
    let card = t.custom(P0, sac, Zone::Hand(P0));
    let card = foretell(&mut t, P0, card);
    t.advance_to(P1, Step::Upkeep);
    t.lands(P0, "Swamp", 1);
    assert!(!can_cast(&mut t, P0, card, FORETELL));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, card, FORETELL));
    t.cast(P0, card).method(FORETELL).go();
    assert!(!t.g.is_live(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
