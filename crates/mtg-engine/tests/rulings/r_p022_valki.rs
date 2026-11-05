//! Rulings batch P022 — permanents that become copies of creatures by their own abilities:
//! Valki, God of Lies, Tilonalli's Skinshifter and Sunfrill Imitator. Summoning sickness
//! (CR 302.6), effects that already applied (CR 613.1, 611.2c), copying something that's
//! copying something else (CR 707.3) and the legend rule with a changed name (CR 704.5j,
//! 707.9b).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const VALKI: &str = "Valki, God of Lies // Tibalt, Cosmic Impostor";

/// P0's Valki enters while P1 holds `card` (exiled with Valki), then Valki's {X} ability
/// (X = `x`) makes it a copy of that card. Valki is summoning sick when `sick`.
fn valki_copying(t: &mut TestGame, card: &str, x: usize, sick: bool) -> ObjectId {
    supported(VALKI);
    let c = t.hand(P1, card);
    t.answer_choose(P0, &[obj(c)]);
    let valki = t.enter(P0, VALKI);
    t.resolve_all();
    assert!(t.in_exile(card));
    if !sick {
        unsick(t, valki);
    }
    t.lands(P0, "Wastes", x);
    let exiled = t.g.current(c);
    t.answer_choose(P0, &[obj(exiled)]);
    let i = activated_index(t, valki, "becomes a copy");
    t.answer(P0, DecisionKind::X, Answer::Number(x as i64));
    t.activate(P0, valki, i, &[]).expect("Valki's ability");
    t.resolve_all();
    assert_eq!(name_of(t, valki), card);
    valki
}

/// P0's Valki (not summoning sick), a copy of P1's Hill Giant card.
pub fn valki_copying_giant() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let v = valki_copying(&mut t, "Hill Giant", 4, false);
    (t, v)
}

#[test]
fn valki_copying_the_turn_it_entered_cant_attack_or_tap() {
    cr!("302.6", "707.2");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "If Valki becomes a copy of a creature during the same turn Valki enters the battlefield, you can't attack with him or use any {T} abilities he gains.");
    supported("Llanowar Elves");
    let mut t = TestGame::new(2);
    let valki = valki_copying(&mut t, "Llanowar Elves", 1, true);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, valki));
    t.set_step(P0, Step::PrecombatMain);
    let i = activated_index(&t, valki, "Add {G}");
    assert!(t.activate(P0, valki, i, &[]).is_err());
}

#[test]
fn valki_keeps_effects_that_applied_before_it_became_a_copy() {
    cr!("613.1", "611.2c", "707.2");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "If an effect begins to apply to Valki before it becomes a copy, that effect will continue to apply.");
    supported(VALKI);
    let mut t = TestGame::new(2);
    let c = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(c)]);
    let valki = t.enter(P0, VALKI);
    t.resolve_all();
    giant_growth(&mut t, P0, valki);
    assert_eq!(t.pt(valki), (5, 4));
    t.lands(P0, "Wastes", 4);
    t.answer_choose(P0, &[obj(t.g.current(c))]);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    let i = activated_index(&t, valki, "becomes a copy");
    t.activate(P0, valki, i, &[]).expect("Valki's ability");
    t.resolve_all();
    assert_eq!(name_of(&t, valki), "Hill Giant");
    assert_eq!(t.pt(valki), (6, 6));
}

// --- Tilonalli's Skinshifter --------------------------------------------------------------

const SKINSHIFTER: &str = "Tilonalli's Skinshifter";

/// P0's Skinshifter attacks with `other`; its trigger targets `other`.
fn skinshifter_attacks_with(t: &mut TestGame, skin: ObjectId, other: ObjectId) {
    t.answer_targets(P0, &[obj(other)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(skin, Entity::Player(P1)), (other, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
}

#[test]
fn tilonallis_skinshifter_copying_a_clone_copies_what_it_copies() {
    cr!("707.3");
    ruling!("Tilonalli's Skinshifter", "If Tilonalli’s Skinshifter copies a creature that’s copying something else, it will become whatever the target is copying.");
    supported(SKINSHIFTER);
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, SKINSHIFTER);
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P0, angel);
    unsick(&mut t, clone);
    skinshifter_attacks_with(&mut t, skin, clone);
    assert_angel(&t, skin, true);
}

#[test]
fn tilonallis_skinshifter_keeps_effects_that_applied_before_it_became_a_copy() {
    cr!("613.1", "611.2c", "707.2");
    ruling!("Tilonalli's Skinshifter", "If an effect from a resolving spell or ability, such as that of Rallying Roar, begins to apply to Tilonalli’s Skinshifter before it becomes a copy of another creature, that effect will continue to apply.");
    supported(SKINSHIFTER);
    supported("Rallying Roar");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, SKINSHIFTER);
    let giant = t.battlefield(P0, "Hill Giant");
    // Rallying Roar: "Creatures you control get +1/+1 until end of turn. Untap them."
    let toks = cast_spell(&mut t, P0, "Rallying Roar", "{2}{W}", &[], None, None);
    assert!(toks.is_empty());
    assert_eq!(t.pt(skin), (1, 2));
    skinshifter_attacks_with(&mut t, skin, giant);
    assert_eq!(name_of(&t, skin), "Hill Giant");
    assert_eq!(t.pt(skin), (4, 4));
}

// --- Sunfrill Imitator ----------------------------------------------------------------------

#[test]
fn sunfrill_imitator_copying_a_legendary_dinosaur_keeps_its_name() {
    cr!("704.5j", "707.9b");
    ruling!("Sunfrill Imitator", "If Sunfrill Imitator copies a legendary creature, Sunfrill Imitator will also be legendary, but because its name remains Sunfrill Imitator, the \"legend rule\" won't cause either of them to be put into the graveyard.");
    supported("Sunfrill Imitator");
    supported("Ghalta, Primal Hunger");
    let mut t = TestGame::new(2);
    let imitator = t.battlefield(P0, "Sunfrill Imitator");
    let ghalta = t.battlefield(P0, "Ghalta, Primal Hunger");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(ghalta)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(imitator, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(name_of(&t, imitator), "Sunfrill Imitator");
    assert!(legendary(&t, imitator));
    assert_eq!(t.pt(imitator), (12, 12));
    assert!(t.on_battlefield(imitator) && t.on_battlefield(ghalta));
}
