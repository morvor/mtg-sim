//! CR 702.48 Offering.

use crate::common_k702_011_017::assert_supported;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const OFFERING: CastMethod = CastMethod::Keyword(KeywordKind::Offering);

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}

fn offer(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    t.answer(
        p,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(what)]),
    );
}

#[test]
fn offering_sacrifices_a_permanent_reduces_the_cost_and_gives_flash() {
    cr!("702.48", "702.48a");
    assert_supported("Patron of the Kitsune");
    let mut t = TestGame::new(2);
    // Patron of the Kitsune {4}{W}{W}, Fox offering; Kitsune Blademaster {2}{W} is a Fox.
    let fox = t.battlefield(P0, "Kitsune Blademaster");
    t.lands(P0, "Plains", 3);
    let patron = t.hand(P0, "Patron of the Kitsune");
    // It's P1's turn: offering lets P0 cast it any time they could cast an instant.
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    let actions = t.g.legal_actions(P0);
    assert!(actions.iter().any(
        |a| matches!(a, Action::Cast { card, method } if *card == patron && *method == OFFERING)
    ));
    assert!(!actions.iter().any(
        |a| matches!(a, Action::Cast { card, method } if *card == patron && *method == CastMethod::Normal)
    ));
    offer(&mut t, P0, fox);
    t.cast(P0, patron).method(OFFERING).go();
    assert!(t.in_graveyard(P0, "Kitsune Blademaster"));
    // {4}{W}{W} minus {2}{W}: {2}{W}.
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Patron of the Kitsune").len(), 1);
}

#[test]
fn without_offering_it_is_cast_normally() {
    cr!("702.48a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kitsune Blademaster");
    t.lands(P0, "Plains", 6);
    let patron = t.hand(P0, "Patron of the Kitsune");
    // Not at instant speed without the offering.
    t.set_step(P1, Step::Upkeep);
    assert!(t.cast(P0, patron).try_go().is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, patron).go();
    assert!(!t.in_graveyard(P0, "Kitsune Blademaster"));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn offering_needs_a_permanent_of_the_quality() {
    cr!("702.48a");
    let mut t = TestGame::new(2);
    // Grizzly Bears isn't a Fox, and an opponent's Fox can't be offered.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Kitsune Blademaster");
    t.lands(P0, "Plains", 6);
    let patron = t.hand(P0, "Patron of the Kitsune");
    assert!(t.cast(P0, patron).method(OFFERING).try_go().is_err());
}

#[test]
fn the_offered_permanent_is_chosen_and_sacrificed_while_casting() {
    cr!("702.48b");
    let mut t = TestGame::new(2);
    let fox1 = t.battlefield(P0, "Kitsune Blademaster");
    let fox2 = t.battlefield(P0, "Kitsune Blademaster");
    t.lands(P0, "Plains", 3);
    let patron = t.hand(P0, "Patron of the Kitsune");
    offer(&mut t, P0, fox2);
    let spell = t.cast(P0, patron).method(OFFERING).go();
    // Sacrificed by the time the spell became cast.
    assert!(t.on_battlefield(fox1));
    assert!(!t.on_battlefield(fox2));
    assert_eq!(t.obj_now(spell).zone, object::Zone::Stack);
}

#[test]
fn generic_colored_and_excess_colored_reductions() {
    cr!("702.48c");
    ruling!(
        "Blast-Furnace Hellkite",
        "generic mana in the sacrificed permanent’s mana cost reduces only generic mana in the spell’s total cost. Colored and colorless mana in the sacrificed permanent’s mana cost reduces mana of the same type in spell’s total cost, and any excess reduces the spell’s total cost by that much generic mana."
    );
    let hellkite = card("Blast-Furnace Hellkite");
    assert!(hellkite
        .front()
        .chars
        .abilities
        .iter()
        .any(|a| a.keyword().is_some_and(|k| k.kind == KeywordKind::Offering)));
    // Blast-Furnace Hellkite {7}{R}{R}, artifact offering.
    // Arcbound Whelp {3}{R}: {7}{R}{R} - {3}{R} = {4}{R}.
    let mut t = TestGame::new(2);
    let whelp = t.battlefield(P0, "Arcbound Whelp");
    t.lands(P0, "Mountain", 5);
    let hk = t.hand(P0, "Blast-Furnace Hellkite");
    offer(&mut t, P0, whelp);
    t.cast(P0, hk).method(OFFERING).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Master of Etherium {2}{U}: the {U} has no match and reduces generic mana:
    // {7}{R}{R} - {2} - {1} = {4}{R}{R}.
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Etherium");
    t.lands(P0, "Mountain", 5);
    let hk = t.hand(P0, "Blast-Furnace Hellkite");
    offer(&mut t, P0, master);
    assert!(t.cast(P0, hk).method(OFFERING).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    offer(&mut t, P0, master);
    t.cast(P0, hk).method(OFFERING).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn generic_mana_reduces_only_generic_mana_and_not_below_zero() {
    cr!("702.48c");
    ruling!(
        "Blast-Furnace Hellkite",
        "The total cost of the spell can’t be reduced to less than {0}."
    );
    let mut t = TestGame::new(2);
    // Blightsteel Colossus {12}: {7}{R}{R} becomes {R}{R}, not less.
    let colossus = t.battlefield(P0, "Blightsteel Colossus");
    t.lands(P0, "Mountain", 1);
    let hk = t.hand(P0, "Blast-Furnace Hellkite");
    offer(&mut t, P0, colossus);
    assert!(t.cast(P0, hk).method(OFFERING).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    offer(&mut t, P0, colossus);
    t.cast(P0, hk).method(OFFERING).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_permanent_with_mana_cost_zero_reduces_nothing() {
    cr!("702.48c");
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P0, "Memnite");
    t.lands(P0, "Mountain", 8);
    let hk = t.hand(P0, "Blast-Furnace Hellkite");
    // Memnite {0}: the Hellkite still costs {7}{R}{R}; eight lands aren't enough.
    offer(&mut t, P0, memnite);
    assert!(t.cast(P0, hk).method(OFFERING).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    offer(&mut t, P0, memnite);
    t.cast(P0, hk).method(OFFERING).go();
    assert!(!t.on_battlefield(memnite));
}
