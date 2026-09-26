//! CR 702.109 Dash.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_027_037::{can_cast, untapped_lands};
use crate::common_k702_052_066::{destroy, run_effect};
use crate::k702_001_010_common::can_attack;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

#[test]
fn a_dashed_creature_has_haste_and_returns_at_the_next_end_step() {
    cr!("702.109", "702.109a");
    assert_supported("Goblin Heelcutter");
    let mut t = TestGame::new(2);
    // Goblin Heelcutter: {3}{R} 3/2, dash {2}{R}.
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Goblin Heelcutter");
    let spell = t.cast(P0, c).method(DASH).go();
    // The dash cost was paid rather than the mana cost; its mana value is still 4.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    let heel = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert!(t.obj(heel).summoning_sick);
    assert!(t.obj(heel).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, heel));
    attack_with(&mut t, &[(heel, Entity::Player(P1))]);
    // At the beginning of the end step it returns to its owner's hand.
    t.advance_to(P0, Step::End);
    assert_eq!(t.life(P1), 17);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Hand(P0));
}

#[test]
fn cast_normally_it_has_no_haste_and_stays() {
    cr!("702.109a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Goblin Heelcutter");
    t.cast(P0, c).go();
    t.resolve_all();
    let heel = t.g.current(c);
    assert!(!t.obj(heel).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, heel));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(c));
}

#[test]
fn dashing_is_casting_the_spell_with_its_normal_timing() {
    cr!("702.109a");
    ruling!(
        "Goblin Heelcutter",
        "If you choose to pay the dash cost rather than the mana cost, you’re still casting the spell. It goes on the stack and can be responded to and countered."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P1, "Island", 2);
    let c = t.hand(P0, "Goblin Heelcutter");
    // Not at instant speed.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, c, DASH));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, c, DASH));
    let spell = t.cast(P0, c).method(DASH).go();
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Heelcutter"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Heelcutter"));
}

#[test]
fn it_returns_only_if_still_on_the_battlefield() {
    cr!("702.109a");
    ruling!(
        "Goblin Heelcutter",
        "If you pay the dash cost to cast a creature spell, that card will be returned to its owner’s hand only if it’s still on the battlefield when its triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Mardu Scout");
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    destroy(&mut t, c);
    assert!(t.in_graveyard(P0, "Mardu Scout"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mardu Scout"));
    assert!(!t.in_hand(P0, "Mardu Scout"));
}

#[test]
fn a_copy_of_a_dashed_creature_neither_has_haste_nor_returns() {
    cr!("702.109a");
    ruling!(
        "Goblin Heelcutter",
        "If a creature enters the battlefield as a copy of or becomes a copy of a creature whose dash cost was paid, the copy won’t have haste and won’t be returned to its owner’s hand."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Mardu Scout");
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    let scout = t.g.current(c);
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.obj(copy).chars.name, "Mardu Scout");
    assert!(!t.obj(copy).chars.has_keyword(KeywordKind::Haste));
    assert!(t.obj(scout).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mardu Scout"));
    assert!(t.on_battlefield(clone));
}

#[test]
fn dash_costs_can_be_reduced() {
    cr!("702.109a");
    ruling!(
        "Warbringer",
        "Warbringer’s first ability can’t affect the colored mana requirement of a dash cost."
    );
    assert_supported("Warbringer");
    let mut t = TestGame::new(2);
    // Warbringer: "Dash costs you pay cost {2} less (as long as this creature is on the
    // battlefield)."
    t.battlefield(P0, "Warbringer");
    t.lands(P0, "Mountain", 1);
    // Goblin Heelcutter's dash {2}{R} costs {R}.
    let c = t.hand(P0, "Goblin Heelcutter");
    assert!(can_cast(&mut t, P0, c, DASH));
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    assert!(t.on_battlefield(c));
    // Mardu Scout's dash {1}{R} still costs {R}.
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Mardu Scout");
    t.cast(P0, s).method(DASH).go();
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn warbringer_doesnt_reduce_its_own_dash_cost() {
    cr!("702.109a");
    ruling!(
        "Warbringer",
        "Warbringer’s first ability doesn’t affect Warbringer itself."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let w = t.hand(P0, "Warbringer");
    assert!(!can_cast(&mut t, P0, w, DASH));
    t.lands(P0, "Mountain", 2);
    assert!(can_cast(&mut t, P0, w, DASH));
}

#[test]
fn a_dashed_creature_that_loses_its_abilities_loses_haste() {
    cr!("702.109a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Mardu Scout");
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    let scout = t.g.current(c);
    assert!(t.obj(scout).chars.has_keyword(KeywordKind::Haste));
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(scout)],
    );
    assert!(!t.obj(scout).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, scout));
}
