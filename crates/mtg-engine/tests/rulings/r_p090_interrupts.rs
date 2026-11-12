//! Rulings batch P090 — interrupts: Ertai's Meddling (a spell exiled with delay counters
//! and put back onto the stack as a copy of the original spell, CR 707.2, 707.10a), and
//! changing the target of a spell with a single target (Deflection, Meddle; CR 115.7).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// P0 casts Ertai's Meddling with X = `x` at the spell `spell` and it resolves.
fn meddle(t: &mut TestGame, spell: ObjectId, x: i64) {
    add_mana(t, P0, ManaType::U, 1 + x as u32);
    let em = t.hand(P0, "Ertai's Meddling");
    t.cast(P0, em).x(x).target(spell).go();
    t.resolve();
}

/// Advances to P1's next upkeep (P1's delayed card comes back with one counter) and puts
/// the triggered ability on the stack and resolves it.
fn p1_next_upkeep(t: &mut TestGame) {
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve();
}

#[test]
fn ertais_meddling_cant_be_cast_without_paying_its_mana_cost() {
    cr!("107.3b", "118.9", "601.2b");
    ruling!(
        "Ertai's Meddling",
        "Ertai's Meddling can't be cast through any way that doesn't pay its mana cost. This is because the X in the Meddling's mana cost can't be 0, but effects that allow spells to be cast without paying their mana costs set X to 0."
    );
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    let em = t.hand(P0, "Ertai's Meddling");
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, em, CastMethod::Free));
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, em, CastMethod::Free).is_err());
    assert_eq!(t.zone(em), Zone::Hand(P0));
    // Positive control: paying {X}{U} with X = 1.
    t.lands(P0, "Island", 2);
    assert!(crate::r_s02_common::can_cast(&mut t, P0, em, CastMethod::Normal));
}

#[test]
fn ertais_meddling_exiles_on_resolution_and_the_spell_comes_back_new() {
    cr!("608.2", "400.7", "701.6a");
    ruling!(
        "Ertai's Meddling",
        "This now exiles the spell as part of Ertai's Meddling's resolution, instead of waiting for the targeted spell to start resolving."
    );
    ruling!(
        "Ertai's Meddling",
        "Once it is put back on the stack, it is a \"new\" spell again and can be countered or even targeted by another Ertai's Meddling."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    meddle(&mut t, spell, 1);
    // Exiled at once (the Bolt was still waiting under nothing else).
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    p1_next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    let back = t.g.stack[0];
    assert_ne!(back, spell, "a new object");
    // A second Meddling targets it.
    meddle(&mut t, back, 1);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert_eq!(t.life(P0), 20);
    p1_next_upkeep(&mut t);
    // Countered this time.
    let back = t.g.stack[0];
    t.lands(P0, "Island", 3);
    let cancel = t.hand(P0, "Cancel");
    t.cast(P0, cancel).target(back).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ertais_meddling_on_a_copy_of_a_spell() {
    cr!("707.10a", "704.5e");
    ruling!(
        "Ertai's Meddling",
        "If a copy of a spell (one that has no card representing it) is affected by Ertai's Meddling, the spell ceases to exist when exiled. It will not gain counters and will not be put back on the stack."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Island", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    let twin = t.hand(P1, "Twincast");
    t.cast(P1, twin).target(spell).go();
    t.resolve();
    // The copy is on top of the Bolt.
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert!(crate::r_s04_common::is_spell_copy(&t, copy));
    meddle(&mut t, copy, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    p1_next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "the copy never comes back");
}

#[test]
fn ertais_meddling_remembers_flashback() {
    cr!("702.34a", "707.2");
    ruling!(
        "Ertai's Meddling",
        "If the spell was cast using flashback, Ertai's Meddling will still exile it with delay counters on it. When the card is returned to the stack, it still \"remembers\" the flashback cost was originally paid. It'll be exiled when it resolves or otherwise leaves the stack."
    );
    supported("Think Twice");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 3);
    let tt = t.graveyard(P1, "Think Twice");
    let spell = t
        .cast(P1, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    meddle(&mut t, spell, 1);
    let card = t.g.current(tt);
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(card, "delay"), 1);
    p1_next_upkeep(&mut t);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "it resolved");
    assert!(t.in_exile("Think Twice"), "{}", t.dump_log());
    assert!(!t.in_graveyard(P1, "Think Twice"));
}

#[test]
fn ertais_meddling_delayed_counterspell_finds_no_target() {
    cr!("608.2b", "707.2");
    ruling!(
        "Ertai's Meddling",
        "Note that a delayed spell that targets another spell won't resolve when it resolves since it will find that its target is no longer on the stack."
    );
    // During P1's turn, P0 casts Lightning Bolt at P1, P1 casts Cancel at it, and P0
    // delays the Cancel.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt_spell = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    let cspell = t.cast(P1, cancel).target(bolt_spell).go();
    meddle(&mut t, cspell, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the Bolt resolved");
    p1_next_upkeep(&mut t);
    t.resolve_all();
    // The Cancel didn't resolve (its target is gone), and it's in the graveyard.
    assert!(t.in_graveyard(P1, "Cancel"), "{}", t.dump_log());
    let resolved = t.g.turn_events.iter().any(|e| {
        matches!(e, mtg_engine::events::Event::SpellResolved { spell } if t.g.obj(*spell).chars.name == "Cancel")
    });
    assert!(!resolved);
}

#[test]
fn ertais_meddling_on_a_face_down_spell() {
    cr!("707.2", "708.2", "702.37c");
    ruling!(
        "Ertai's Meddling",
        "If Ertai's Meddling is used to copy a spell being cast face down due to Morph ability, the spell will create a face up, 2/2, colorless, nameless creature with no text."
    );
    supported("Grinning Demon");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 3);
    let demon = t.hand(P1, "Grinning Demon");
    let spell = t
        .cast(P1, demon)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    assert!(t.obj_now(spell).face_down);
    meddle(&mut t, spell, 1);
    p1_next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let perm = t.g.current(demon);
    assert!(t.on_battlefield(perm), "{}", t.dump_log());
    let o = t.obj_now(perm);
    assert!(!o.face_down, "face up");
    assert_eq!(t.pt(perm), (2, 2));
    assert!(o.chars.name.is_empty(), "nameless: {}", o.chars.name);
    assert_eq!(o.chars.colors, mtg_engine::types::ColorSet::NONE);
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.subtypes.is_empty());
    assert!(
        !o.chars
            .abilities
            .iter()
            .any(|a| !matches!(a.kind, AbilityKind::Spell(_))),
        "no text"
    );
}

#[test]
fn ertais_meddling_keeps_spliced_effects() {
    cr!("702.47c", "707.2");
    ruling!(
        "Ertai's Meddling",
        "If Ertai's Meddling is used to copy an arcane spell that had effects spliced onto it, it will create a spell with all of those effects. That spell's controller will not be able to splice additional effects onto the spell, since they did not re-cast the spell and instead simply put it back onto the stack."
    );
    supported("Glacial Ray");
    supported("Kodama's Might");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 1);
    let might = t.hand(P1, "Kodama's Might");
    t.hand(P1, "Glacial Ray");
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P1, might).target(bears).target(P0).go();
    meddle(&mut t, spell, 1);
    // A second Glacial Ray and mana to splice it, if P1 were offered.
    t.lands(P1, "Mountain", 2);
    t.hand(P1, "Glacial Ray");
    let from = t.asked().len();
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    p1_next_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "the spliced Glacial Ray's 2 damage {:?}\n{}", t.asked(), t.dump_log());
    let offered = asked_since(&t, from).iter().any(|(_, d)| {
        matches!(d, mtg_engine::decision::Decision::OptionalCost { name, .. } if name == "splice")
    });
    assert!(!offered, "no splicing onto a spell put back on the stack");
    assert_eq!(tapped_lands(&t, P1), 0);
}

#[test]
fn deflection_needs_a_spell_with_a_single_target() {
    cr!("115.7a", "115.7");
    ruling!(
        "Deflection",
        "If a spell targets multiple things, you can’t target it with Deflection, even if all but one of those targets has become illegal."
    );
    supported("Deflection");
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P1, "Seeds of Strength");
    let seeds = t.hand(P1, "Seeds of Strength");
    let s = t
        .cast(P1, seeds)
        .target(a)
        .target(a)
        .target(b)
        .go();
    assert_eq!(
        mtg_engine::copy::chosen_targets(&t.g, s),
        vec![Entity::Object(a), Entity::Object(a), Entity::Object(b)]
    );
    // Hill Giant leaves: only one legal target remains.
    crate::r_s05_common::move_to(&mut t, b, Zone::Hand(P1));
    let defl = t.hand(P0, "Deflection");
    t.lands(P0, "Island", 4);
    let cands = crate::r_s04_common::spell_targets(&mut t, P0, "Deflection");
    assert!(!cands.contains(&Entity::Object(s)), "{cands:?}");
    // Positive control: a Shock (one target) can be.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let sh = t.cast(P1, shock).target(P0).go();
    let cands = crate::r_s04_common::spell_targets(&mut t, P0, "Deflection");
    assert!(cands.contains(&Entity::Object(sh)), "{cands:?}");
    let _ = defl;
}

#[test]
fn meddle_needs_a_spell_whose_only_target_is_a_creature() {
    cr!("115.7", "608.2c");
    ruling!(
        "Meddle",
        "Only works on spells with a single target, where that target is a creature. Will not work on spells that target a single creature and a player, for example."
    );
    supported("Meddle");
    supported("Arc Trail");
    // Arc Trail: "Arc Trail deals 2 damage to any target and 1 damage to another target."
    // At Grizzly Bears and P0: Meddle does nothing.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Mountain", 2);
    let arc = t.hand(P1, "Arc Trail");
    let s = t
        .cast(P1, arc)
        .target(bears)
        .target(P0)
        .go();
    assert_eq!(
        mtg_engine::copy::chosen_targets(&t.g, s),
        vec![Entity::Object(bears), Entity::Player(P0)]
    );
    t.lands(P0, "Island", 2);
    let meddle = t.hand(P0, "Meddle");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, meddle).target(s).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
    assert_eq!(t.life(P0), 19);
    // A Shock at the Bears alone: redirected to the Hill Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let s = t.cast(P1, shock).target(bears).go();
    t.lands(P0, "Island", 2);
    let meddle = t.hand(P0, "Meddle");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, meddle).target(s).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 2);
    // A Shock at a player: no change.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let s = t.cast(P1, shock).target(P0).go();
    t.lands(P0, "Island", 2);
    let meddle = t.hand(P0, "Meddle");
    t.cast(P0, meddle).target(s).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}
