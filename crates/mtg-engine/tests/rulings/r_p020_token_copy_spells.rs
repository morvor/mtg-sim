//! Rulings batch P020 — a token created as a copy of a creature (or other permanent) by a
//! spell has the copied permanent's "enters" triggered abilities, which trigger as the
//! token enters, and its "as this enters" / "enters with" abilities, which apply as it
//! enters (CR 707.2, 111.4, 614.1c, 603.6a).

use crate::r_p020_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s25_common::lands_for_cost;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// A game with an Aven Riftwatcher controlled by `owner`.
fn with_riftwatcher(owner: PlayerId) -> (TestGame, ObjectId) {
    supported(RIFTWATCHER);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(owner, RIFTWATCHER);
    (t, rift)
}

/// P0 casts `spell` (paid with basic lands) choosing `modes` (if any) and `targets` (one
/// per target slot), then everything resolves.
fn cast_and_resolve(t: &mut TestGame, spell: &str, modes: &[usize], targets: &[Entity]) {
    supported(spell);
    lands_for_cost(t, P0, spell);
    let card = t.hand(P0, spell);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let b = t.cast(P0, card);
    let b = if modes.is_empty() { b } else { b.modes(modes) };
    b.go();
    t.resolve_all();
}

/// `spell` creates one token copy of P0's Aven Riftwatcher: it enters with its time
/// counters and its "enters" trigger gains P0 2 life. Returns the game and the token.
fn one_copy_of_my_riftwatcher(spell: &str, modes: &[usize]) -> (TestGame, ObjectId) {
    let (mut t, rift) = with_riftwatcher(P0);
    let before = tokens(&t, P0);
    let life = t.life(P0);
    cast_and_resolve(&mut t, spell, modes, &[Entity::Object(rift)]);
    let new = riftwatcher_tokens_entered(&t, P0, &before, life, 1);
    (t, new[0])
}

#[test]
fn self_reflection_token_has_the_copied_enters_abilities() {
    cr!("707.2", "111.4", "614.1c", "603.6a");
    ruling!(
        "Self-Reflection",
        "Any \"enters\" abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the target creature will also work."
    );
    one_copy_of_my_riftwatcher("Self-Reflection", &[]);
}

#[test]
fn quasiduplicate_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Quasiduplicate",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    one_copy_of_my_riftwatcher("Quasiduplicate", &[]);
}

#[test]
fn sublime_epiphany_token_has_the_copied_enters_abilities() {
    cr!("707.2", "700.2", "614.1c", "603.6a");
    ruling!(
        "Sublime Epiphany",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    // "• Create a token that's a copy of target creature you control."
    one_copy_of_my_riftwatcher("Sublime Epiphany", &[3]);
}

#[test]
fn quantum_misalignment_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Quantum Misalignment",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    one_copy_of_my_riftwatcher("Quantum Misalignment", &[]);
}

#[test]
fn mirage_mockery_token_has_the_copied_enters_abilities() {
    cr!("707.2", "700.2", "614.1c", "603.6a");
    ruling!(
        "Mirage Mockery",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    // "• Create a token that's a copy of target nonartifact creature you control."
    one_copy_of_my_riftwatcher("Mirage Mockery", &[1]);
}

#[test]
fn croaking_counterpart_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Croaking Counterpart",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    // "...target non-Frog creature, except it's a 1/1 green Frog." An opponent's creature
    // works too.
    let (mut t, rift) = with_riftwatcher(P1);
    let life = t.life(P0);
    cast_and_resolve(&mut t, "Croaking Counterpart", &[], &[Entity::Object(rift)]);
    let new = riftwatcher_tokens_entered(&t, P0, &[], life, 1);
    assert_eq!(t.pt(new[0]), (1, 1));
    assert!(t.obj_now(new[0]).chars.has_subtype("Frog"));
}

#[test]
fn replicate_token_has_the_copied_enters_abilities() {
    cr!("707.2", "709.3", "614.1c", "603.6a");
    ruling!(
        "Repudiate // Replicate",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    supported("Repudiate // Replicate");
    let (mut t, rift) = with_riftwatcher(P0);
    let life = t.life(P0);
    // Replicate {1}{G}{U}: "Create a token that's a copy of target creature you control."
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Repudiate // Replicate");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(Entity::Object(rift))
        .go();
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn rally_the_galadhrim_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Rally the Galadhrim",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    one_copy_of_my_riftwatcher("Rally the Galadhrim", &[]);
}

#[test]
fn cackling_counterpart_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Cackling Counterpart",
        "Any “enters” triggered ability of the copied creature will trigger when the token enters the battlefield. Any “as [this creature] enters” or “[this creature] enters with” abilities of the chosen creature will also work."
    );
    one_copy_of_my_riftwatcher("Cackling Counterpart", &[]);
}

#[test]
fn electroduplicate_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Electroduplicate",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied creature will also work."
    );
    let (t, tok) = one_copy_of_my_riftwatcher("Electroduplicate", &[]);
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
}

#[test]
fn kindle_the_inner_flame_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Kindle the Inner Flame",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    let (t, tok) = one_copy_of_my_riftwatcher("Kindle the Inner Flame", &[]);
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
}

#[test]
fn ember_island_production_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Ember Island Production",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the targetted creature will also work."
    );
    // "• Create a token that's a copy of target creature you control, except it's not
    // legendary and it's a 4/4 Hero in addition to its other types."
    let (t, tok) = one_copy_of_my_riftwatcher("Ember Island Production", &[0]);
    assert_eq!(t.pt(tok), (4, 4));
    assert!(t.obj_now(tok).chars.has_subtype("Hero"));
}

#[test]
fn supplant_form_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "608.2h");
    ruling!(
        "Supplant Form",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “As [this creature] enters the battlefield” or “[This creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    // "Return target creature to its owner's hand. You create a token that's a copy of that
    // creature." The returned Riftwatcher's leaves trigger gains its controller 2 life
    // too.
    let (mut t, rift) = with_riftwatcher(P1);
    let (mine, theirs) = (t.life(P0), t.life(P1));
    cast_and_resolve(&mut t, "Supplant Form", &[], &[Entity::Object(rift)]);
    assert!(t.in_hand(P1, RIFTWATCHER));
    assert_eq!(t.life(P1), theirs + 2);
    riftwatcher_tokens_entered(&t, P0, &[], mine, 1);
}

#[test]
fn hate_mirage_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Hate Mirage",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the creature will also work."
    );
    // "Choose up to two target creatures you don't control. For each of those creatures,
    // create a token that's a copy of that creature."
    let (mut t, rift) = with_riftwatcher(P1);
    let rift2 = t.battlefield(P1, RIFTWATCHER);
    let life = t.life(P0);
    supported("Hate Mirage");
    lands_for_cost(&mut t, P0, "Hate Mirage");
    let card = t.hand(P0, "Hate Mirage");
    t.cast(P0, card)
        .targets(&[Entity::Object(rift), Entity::Object(rift2)])
        .go();
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 2);
}

#[test]
fn aggressive_biomancy_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a", "107.3a");
    ruling!(
        "Aggressive Biomancy",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the tokens enter the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the creature will also work."
    );
    // "Create X tokens that are copies of target creature you control, except they have
    // 'When this token enters, it fights up to one target creature you don't control.'"
    supported("Aggressive Biomancy");
    let (mut t, rift) = with_riftwatcher(P0);
    let life = t.life(P0);
    lands_for_cost(&mut t, P0, "Aggressive Biomancy");
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Aggressive Biomancy");
    t.cast(P0, card).x(2).target(Entity::Object(rift)).go();
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 2);
}

#[test]
fn tempt_with_reflections_tokens_use_the_copied_enters_with_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Tempt with Reflections",
        "Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    // "Create a token that's a copy of that creature. Each opponent may create a token
    // that's a copy of that creature. For each opponent who does, create a token that's a
    // copy of that creature." The opponent accepts.
    let (mut t, rift) = with_riftwatcher(P0);
    let (mine, theirs) = (t.life(P0), t.life(P1));
    t.answer_yes(P1, true);
    cast_and_resolve(&mut t, "Tempt with Reflections", &[], &[Entity::Object(rift)]);
    riftwatcher_tokens_entered(&t, P0, &[], mine, 2);
    riftwatcher_tokens_entered(&t, P1, &[], theirs, 1);
}

#[test]
fn echocasting_symposium_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "111.2");
    ruling!(
        "Echocasting Symposium",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied permanent will also work."
    );
    // "Target player creates a token that's a copy of target creature you control." P1
    // creates (and controls) the copy of P0's Riftwatcher: its trigger gains P1 life.
    let (mut t, rift) = with_riftwatcher(P0);
    let before = tokens(&t, P1);
    let life = t.life(P1);
    cast_and_resolve(
        &mut t,
        "Echocasting Symposium",
        &[],
        &[Entity::Player(P1), Entity::Object(rift)],
    );
    riftwatcher_tokens_entered(&t, P1, &before, life, 1);
    assert!(tokens(&t, P0).is_empty());
}
