//! Rulings batch S04 — dash (CR 702.109): "You may cast this card by paying [cost] rather
//! than its mana cost," "If this spell's dash cost was paid, return the permanent this
//! spell becomes to its owner's hand at the beginning of the next end step," and "As long
//! as this permanent's dash cost was paid, it has haste."
//!
//! The rulings with curly apostrophes are cited with Goblin Heelcutter by the keyword
//! tests; these are their straight-apostrophe versions.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, can_cast};
use crate::r_s04_common::*;
use mtg_engine::ability::{Duration, Effect, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

/// P0 casts the real card `name` for its dash cost (with the mana for it) and it
/// resolves; returns the permanent.
fn dash(t: &mut TestGame, name: &str) -> ObjectId {
    supported(name);
    let c = t.hand(P0, name);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 1);
    t.cast(P0, c).method(DASH).go();
    t.resolve_all();
    let id = t.g.current(c);
    assert!(t.on_battlefield(id));
    id
}

/// Advances to P0's end step with the dash trigger on the stack.
fn to_end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(on_stack(t, "delayed trigger"), 1, "{:?}", stack_items(t));
}

#[test]
fn a_dashed_creature_returns_only_if_still_on_the_battlefield() {
    cr!("702.109a", "603.7c", "608.2h");
    ruling!(
        "Zurgo Bellstriker",
        "If you pay the dash cost to cast a creature spell, that card will be returned to its owner's hand only if it's still on the battlefield when its triggered ability resolves. If it dies or goes to another zone before then, it will stay where it is."
    );
    // It dies with the trigger on the stack: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let zurgo = dash(&mut t, "Zurgo Bellstriker");
    to_end_step(&mut t);
    t.g.destroy(zurgo, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Zurgo Bellstriker"));
    assert!(!t.in_hand(P0, "Zurgo Bellstriker"));

    // It's exiled with the trigger on the stack: it stays in exile.
    let mut t = TestGame::new(2);
    let zurgo = dash(&mut t, "Zurgo Bellstriker");
    to_end_step(&mut t);
    t.lands(P1, "Plains", 1);
    let swords = t.hand(P1, "Swords to Plowshares");
    t.cast(P1, swords).target(zurgo).go();
    t.resolve_all();
    assert!(t.in_exile("Zurgo Bellstriker"));
    assert!(!t.in_hand(P0, "Zurgo Bellstriker"));

    // Still on the battlefield, it's returned.
    let mut t = TestGame::new(2);
    dash(&mut t, "Zurgo Bellstriker");
    to_end_step(&mut t);
    t.resolve_all();
    assert!(t.in_hand(P0, "Zurgo Bellstriker"));
}

#[test]
fn a_copy_of_a_dashed_creature_has_no_haste_and_isnt_returned() {
    cr!("702.109a", "707.2", "707.5");
    ruling!(
        "Reckless Imp",
        "If a creature enters the battlefield as a copy of or becomes a copy of a creature whose dash cost was paid, the copy won't have haste and won't be returned to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let imp = dash(&mut t, "Reckless Imp");
    assert!(t.obj(imp).chars.has_keyword(KeywordKind::Haste));
    // Clone enters as a copy of it.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(imp)]);
    t.resolve_all();
    let entered = t.g.current(clone);
    assert_eq!(t.obj(entered).chars.name, "Reckless Imp");
    assert!(!t.obj(entered).chars.has_keyword(KeywordKind::Haste));
    // Grizzly Bears (summoning sick) becomes a copy of it.
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    run_with(
        &mut t,
        P0,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears), Entity::Object(imp)],
    );
    assert_eq!(t.obj_now(bears).chars.name, "Reckless Imp");
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, imp));
    assert!(!can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, entered));
    // Only the dashed Imp returns.
    t.set_step(P0, Step::PostcombatMain);
    to_end_step(&mut t);
    t.resolve_all();
    assert!(t.in_hand(P0, "Reckless Imp"));
    assert!(t.on_battlefield(entered));
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_dashed_creature_doesnt_have_to_attack() {
    cr!("702.109a", "508.1a", "508.1d");
    ruling!(
        "Mardu Strike Leader",
        "You don't have to attack with the creature with dash unless another ability says you do."
    );
    // No attack: nothing happens, and it still returns.
    let mut t = TestGame::new(2);
    let leader = dash(&mut t, "Mardu Strike Leader");
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(leader));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mardu Strike Leader"));
    assert_eq!(t.life(P1), 20);

    // Grand Melee: "All creatures attack each combat if able." Then it has to attack.
    supported("Grand Melee");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grand Melee");
    let leader = dash(&mut t, "Mardu Strike Leader");
    attack_with(&mut t, &[]);
    assert!(t.g.is_attacking(leader));
}

#[test]
fn dashing_is_casting_a_spell_with_its_normal_timing() {
    cr!("702.109a", "601.2", "601.2b", "307.1", "702.8a");
    ruling!(
        "Lightning Berserker",
        "If you choose to pay the dash cost rather than the mana cost, you're still casting the spell. It goes on the stack and can be responded to and countered. You can cast a creature spell for its dash cost only when you otherwise could cast that creature spell. Most of the time, this means during your main phase when the stack is empty."
    );
    supported("Lightning Berserker");
    // Lightning Berserker: dash {R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let berserker = t.hand(P0, "Lightning Berserker");
    // Not during combat, nor with a spell on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, berserker, DASH));
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_cast(&mut t, P0, berserker, DASH));
    t.resolve_all();
    // In the main phase with the stack empty, it can: it's a spell on the stack, which
    // Essence Scatter can counter.
    assert!(can_cast(&mut t, P0, berserker, DASH));
    let spell = t.cast(P0, berserker).method(DASH).go();
    assert!(t.g.obj(spell).is_spell());
    t.lands(P1, "Island", 2);
    let scatter = t.hand(P1, "Essence Scatter");
    t.cast(P1, scatter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Berserker"));
    assert!(t.named_on_battlefield("Lightning Berserker").is_empty());

    // With an effect that lets it be cast as though it had flash, it can be dashed at
    // instant speed.
    supported("Leyline of Anticipation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    t.lands(P0, "Mountain", 1);
    let berserker = t.hand(P0, "Lightning Berserker");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_cast(&mut t, P0, berserker, DASH));
    t.cast(P0, berserker).method(DASH).go();
    t.resolve_all();
    let berserker = t.g.current(berserker);
    assert_eq!(t.zone(berserker), Zone::Battlefield);
    assert!(t.obj(berserker).chars.has_keyword(KeywordKind::Haste));
}
