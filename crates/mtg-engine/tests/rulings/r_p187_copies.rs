//! Rulings batch P187 — Phyrexian copy effects and copiable values: Phyrexian Metamorph
//! (CR 707.2, 707.8, 707.9b), Brudiclad's token copies (CR 707.3, 611.2a), Applied
//! Geometry (CR 707.2, 707.9b), Duplicant's power and toughness from a card in exile
//! (CR 604.3, 613.4a), and Metallic Mimic's additional counter (CR 614.1c, 614.12).

use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s10_common::attacking;
use crate::r_s25_common::lands_for_cost;
use crate::r_s28_common::cast_card;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Phyrexian Metamorph
// ---------------------------------------------------------------------------------------

#[test]
fn metamorph_copying_a_creature_with_x_in_its_cost_has_x_zero() {
    cr!("707.2", "202.3e", "107.3m");
    ruling!(
        "Phyrexian Metamorph",
        "If the chosen permanent has {X} in its mana cost (such as Protean Hydra), X is considered to be zero."
    );
    supported("Phyrexian Metamorph");
    supported("Endless One");
    // "Endless One enters with X +1/+1 counters on it." Cast with X = 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Endless One");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let one = t.g.current(card);
    assert_eq!(t.counters(one, "+1/+1"), 3);
    t.answer_choose(P0, &[o(one)]);
    let meta = t.enter(P0, "Phyrexian Metamorph");
    t.settle();
    let m = t.obj_now(meta);
    assert_eq!(m.chars.name, "Endless One");
    assert!(m.is(CardType::Artifact));
    assert_eq!(t.g.mana_value_of(meta), 0);
    // It entered with X = 0 counters: a 0/0 that survives only through the anthem.
    assert_eq!(t.counters(meta, "+1/+1"), 0);
    assert_eq!(t.pt(meta), (1, 1));
}

#[test]
fn metamorph_copying_a_token_copies_its_original_characteristics_and_isnt_a_token() {
    cr!("707.2", "111.4", "707.9b", "111.1");
    ruling!(
        "Phyrexian Metamorph",
        "Phyrexian Metamorph copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, except it's also an artifact. Phyrexian Metamorph is not a token."
    );
    let mut t = TestGame::new(2);
    let goblin = create_token(&mut t, P1, "Goblin");
    t.g.add_counters(o(goblin), "+1/+1", 2, None);
    t.g.recompute();
    t.answer_choose(P0, &[o(goblin)]);
    let meta = t.enter(P0, "Phyrexian Metamorph");
    t.settle();
    let m = t.obj_now(meta);
    assert!(!m.is_token());
    assert!(m.chars.has_subtype("Goblin"));
    assert!(m.is(CardType::Artifact) && m.is(CardType::Creature));
    assert_eq!(m.chars.colors, ColorSet::NONE);
    assert_eq!(t.pt(meta), (1, 1));
    assert_eq!(t.counters(meta, "+1/+1"), 0);
}

#[test]
fn metamorph_copying_nothing_is_a_zero_zero_artifact_creature() {
    cr!("707.2", "704.5f");
    ruling!(
        "Phyrexian Metamorph",
        "You can choose not to copy anything. In that case, Phyrexian Metamorph simply enters as a 0/0 artifact creature and is put into its owner's graveyard as a state-based action"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    let meta = t.enter(P0, "Phyrexian Metamorph");
    assert_eq!(t.obj_now(meta).chars.name, "Phyrexian Metamorph");
    t.settle();
    assert!(t.in_graveyard(P0, "Phyrexian Metamorph"));
    // "(unless something else is raising its toughness)"
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    let meta = t.enter(P0, "Phyrexian Metamorph");
    t.settle();
    assert!(t.on_battlefield(meta));
    assert_eq!(t.pt(meta), (1, 1));
    assert!(t.obj_now(meta).is(CardType::Artifact));
}

// ---------------------------------------------------------------------------------------
// Brudiclad, Telchor Engineer
// ---------------------------------------------------------------------------------------

/// Moves to P0's beginning of combat with Brudiclad's trigger on the stack, choosing
/// `chosen` (or no token) when it resolves, and resolves it.
fn brudiclad_trigger(t: &mut TestGame, chosen: Option<ObjectId>) {
    t.advance_to(P0, Step::BeginningOfCombat);
    match chosen {
        Some(c) => {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[o(c)]);
        }
        None => {
            t.answer_yes(P0, false);
        }
    }
    t.resolve_all();
}

fn myr_tokens(t: &TestGame) -> Vec<ObjectId> {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Myr"))
        .collect()
}

#[test]
fn brudiclad_turns_every_other_token_including_the_new_myr_into_copies() {
    cr!("707.2", "707.3", "611.2a");
    ruling!(
        "Brudiclad, Telchor Engineer",
        "The last effect of Brudiclad's triggered ability affects all tokens you control other than the chosen token, including the token that was just created if that isn't the chosen token."
    );
    ruling!(
        "Brudiclad, Telchor Engineer",
        "The effect of Brudiclad's triggered ability lasts indefinitely. It continues to apply after Brudiclad leaves the game."
    );
    supported("Brudiclad, Telchor Engineer");
    let mut t = TestGame::new(2);
    let brudiclad = t.battlefield(P0, "Brudiclad, Telchor Engineer");
    let goblin = create_token(&mut t, P0, "Goblin");
    let soldier = create_token(&mut t, P0, "Soldier");
    brudiclad_trigger(&mut t, Some(goblin));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 3);
    for tok in &toks {
        let c = &t.obj_now(*tok).chars;
        assert!(c.has_subtype("Goblin"), "{:?}", c.subtypes);
        assert!(!c.has_subtype("Myr") && !c.has_subtype("Soldier"));
        assert_eq!(t.pt(*tok), (1, 1));
    }
    assert!(t.on_battlefield(soldier));
    // It lasts after Brudiclad leaves, and into later turns.
    destroy(&mut t, brudiclad);
    t.advance_to(P1, Step::Upkeep);
    for tok in tokens(&t, P0) {
        assert!(t.obj_now(tok).chars.has_subtype("Goblin"));
    }
}

#[test]
fn brudiclad_copies_what_the_chosen_token_is_copying() {
    cr!("707.3", "707.2");
    ruling!(
        "Brudiclad, Telchor Engineer",
        "If the chosen token is copying something else (for example, if the chosen token was previously affected by Brudiclad's triggered ability), then your tokens become copies of whatever the chosen token copied."
    );
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brudiclad, Telchor Engineer");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    cast_card(&mut t, P0, "Cackling Counterpart");
    t.resolve_all();
    let copy = tokens(&t, P0)[0];
    assert_eq!(t.obj_now(copy).chars.name, "Hill Giant");
    // A Goblin token, plus the Myr: both become Hill Giants.
    create_token(&mut t, P0, "Goblin");
    brudiclad_trigger(&mut t, Some(copy));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 3);
    for tok in toks {
        assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
        assert_eq!(t.pt(tok), (3, 3));
    }
    assert!(myr_tokens(&t).is_empty());
}

#[test]
fn brudiclad_leaving_doesnt_remove_a_hasty_token_from_combat() {
    cr!("506.4", "302.6", "702.10b");
    ruling!(
        "Brudiclad, Telchor Engineer",
        "Once a creature token that came under your control this turn has legally attacked, causing it to lose haste by removing Brudiclad from the battlefield won't cause that token to stop attacking."
    );
    let mut t = TestGame::new(2);
    let brudiclad = t.battlefield(P0, "Brudiclad, Telchor Engineer");
    brudiclad_trigger(&mut t, None);
    let myr = myr_tokens(&t)[0];
    assert!(t.obj_now(myr).has_keyword(KeywordKind::Haste));
    crate::r_s01_common::attack_with(&mut t, &[(myr, Entity::Player(P1))]);
    assert!(attacking(&t, myr));
    destroy(&mut t, brudiclad);
    assert!(!t.obj_now(myr).has_keyword(KeywordKind::Haste));
    assert!(attacking(&t, myr));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Applied Geometry
// ---------------------------------------------------------------------------------------

#[test]
fn applied_geometry_copying_a_token_uses_its_original_characteristics() {
    cr!("707.2", "111.4", "707.9b");
    ruling!(
        "Applied Geometry",
        "If the copied permanent is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token, with the listed exception."
    );
    supported("Applied Geometry");
    // "Create a token that's a copy of target non-Aura permanent you control, except it's a
    // 0/0 Fractal creature in addition to its other types. Put six +1/+1 counters on it."
    let mut t = TestGame::new(2);
    let goblin = create_token(&mut t, P0, "Goblin");
    t.g.add_counters(o(goblin), "+1/+1", 2, None);
    t.g.recompute();
    t.answer_targets(P0, &[o(goblin)]);
    cast_card(&mut t, P0, "Applied Geometry");
    t.resolve_all();
    let new: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| *id != goblin)
        .collect();
    assert_eq!(new.len(), 1);
    let c = &t.obj_now(new[0]).chars;
    assert!(c.has_subtype("Goblin") && c.has_subtype("Fractal"));
    assert_eq!(c.colors, ColorSet::NONE);
    assert_eq!(t.counters(new[0], "+1/+1"), 6);
    assert_eq!(t.pt(new[0]), (6, 6));
}

// ---------------------------------------------------------------------------------------
// Duplicant
// ---------------------------------------------------------------------------------------

/// P0's Duplicant enters and exiles `target` with its imprint ability.
fn duplicant_exiles(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[o(target)]);
    t.answer_yes(P0, true);
    let dup = t.enter(P0, "Duplicant");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(target)), Zone::Exile);
    dup
}

#[test]
fn duplicant_uses_a_characteristic_defining_pt_of_the_exiled_card_constantly_updated() {
    cr!("604.3", "613.4a", "611.3a");
    ruling!(
        "Duplicant",
        "Abilities that define a * in a creature's power and toughness apply while that card is in exile, but abilities that add or subtract power and toughness don't."
    );
    ruling!(
        "Duplicant",
        "Duplicant's power and toughness are constantly updated if the exiled card's power and/or toughness change."
    );
    supported("Crusader of Odric");
    // Crusader of Odric: "power and toughness are each equal to the number of creatures you
    // control" — in exile, the creatures its owner (P1) controls.
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P1, "Crusader of Odric");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let dup = duplicant_exiles(&mut t, crusader);
    assert_eq!(t.pt(dup), (2, 2));
    t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(dup), (3, 3));
    // Death's Shadow: 13/13 that "gets -X/-X, where X is your life total" only on the
    // battlefield.
    let mut t = TestGame::new(2);
    let shadow = t.battlefield(P1, "Death's Shadow");
    t.g.player_mut(P1).life = 5;
    t.g.recompute();
    assert_eq!(t.pt(shadow), (8, 8));
    let dup = duplicant_exiles(&mut t, shadow);
    assert_eq!(t.pt(dup), (13, 13));
}

// ---------------------------------------------------------------------------------------
// Metallic Mimic
// ---------------------------------------------------------------------------------------

fn choose_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn metallic_mimic_doesnt_add_counters_to_creatures_entering_with_it() {
    cr!("614.1c", "614.12");
    ruling!(
        "Metallic Mimic",
        "Creatures of the chosen type that enter the battlefield at the same time as Metallic Mimic won't enter with an additional +1/+1 counter."
    );
    supported("Metallic Mimic");
    supported("Brilliant Restoration");
    // "Return all artifact and enchantment cards from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    let mimic = t.graveyard(P0, "Metallic Mimic");
    let thopter = t.graveyard(P0, "Ornithopter");
    choose_type(&mut t, P0, "Thopter");
    cast_card(&mut t, P0, "Brilliant Restoration");
    t.resolve_all();
    let (mimic, thopter) = (t.g.current(mimic), t.g.current(thopter));
    assert!(t.on_battlefield(mimic) && t.on_battlefield(thopter));
    assert_eq!(t.counters(thopter, "+1/+1"), 0);
    // One entering later gets it.
    let later = t.enter(P0, "Ornithopter");
    assert_eq!(t.counters(later, "+1/+1"), 1);
}

#[test]
fn metallic_mimic_helps_shapeshifters_only_if_shapeshifter_was_chosen() {
    cr!("614.1c", "614.12", "707.2");
    ruling!(
        "Metallic Mimic",
        "Even though Metallic Mimic is a Shapeshifter, other Shapeshifter creatures you control won't get a +1/+1 counter unless you chose Shapeshifter as Metallic Mimic entered the battlefield."
    );
    supported("Unstable Shapeshifter");
    // Chose Elf: a Shapeshifter doesn't get a counter.
    let mut t = TestGame::new(2);
    choose_type(&mut t, P0, "Elf");
    t.enter(P0, "Metallic Mimic");
    let shifter = t.enter(P0, "Unstable Shapeshifter");
    assert_eq!(t.counters(shifter, "+1/+1"), 0);
    // Chose Shapeshifter: it does, but a Clone entering as a Hill Giant doesn't.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    choose_type(&mut t, P0, "Shapeshifter");
    t.enter(P0, "Metallic Mimic");
    let shifter = t.enter(P0, "Unstable Shapeshifter");
    assert_eq!(t.counters(shifter, "+1/+1"), 1);
    t.answer_choose(P0, &[o(giant)]);
    let clone = t.enter(P0, "Clone");
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    assert_eq!(t.counters(clone, "+1/+1"), 0);
}
