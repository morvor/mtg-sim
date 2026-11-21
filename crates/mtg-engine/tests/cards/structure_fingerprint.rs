//! The structural fingerprint of abilities (`mtg_engine::structure`): abilities that differ
//! only in literal parameters share a structure; abilities that differ in a semantic choice
//! don't.

use mtg_engine::ability::{Ability, AbilityKind};
use mtg_engine::card::{abilities_of, card, Layout};
use mtg_engine::oracle::{compile, CompileContext};
use mtg_engine::structure::fingerprint;
use mtg_engine::types::TypeLine;

const NAME: &str = "Structure Probe";

/// The fingerprints of the abilities `text` compiles to on a card named [`NAME`].
fn fps(type_line: &str, text: &str) -> Vec<String> {
    let tl = TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: NAME,
        full_name: NAME,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("2"),
        toughness: Some("2"),
    };
    let c = compile(text, &ctx);
    assert!(c.unsupported.is_empty(), "{text}: {:?}", c.unsupported);
    c.abilities
        .iter()
        .map(|a| fingerprint(a, &[NAME]))
        .collect()
}

fn creature(text: &str) -> Vec<String> {
    fps("Creature — Human Soldier", text)
}

fn instant(text: &str) -> Vec<String> {
    fps("Instant", text)
}

fn real(name: &str) -> Vec<String> {
    let def = card(name);
    let abilities: &[Ability] = abilities_of(&def, 0);
    abilities
        .iter()
        .filter(|a| !matches!(a.kind, AbilityKind::Unsupported(_)))
        .map(|a| fingerprint(a, &[name]))
        .collect()
}

#[test]
fn real_cards_differing_only_in_numbers_and_name_share_a_structure() {
    // "Shock deals 2 damage to any target." / "Lightning Bolt deals 3 damage to any target."
    assert_eq!(real("Shock"), real("Lightning Bolt"));
    assert!(!real("Shock").is_empty());
}

#[test]
fn numbers_are_abstracted() {
    assert_eq!(
        instant("~ deals 2 damage to any target."),
        instant("~ deals 5 damage to any target.")
    );
    assert_eq!(
        creature("When this creature enters, draw a card."),
        creature("When this creature enters, draw three cards.")
    );
    assert_eq!(
        creature("{T}: Target creature gets +1/+1 until end of turn."),
        creature("{T}: Target creature gets +3/+2 until end of turn.")
    );
}

#[test]
fn mana_amounts_colors_and_creature_types_are_abstracted() {
    assert_eq!(
        creature("{2}{R}, {T}: Draw a card."),
        creature("{1}{G}{G}, {T}: Draw a card.")
    );
    assert_eq!(creature("{T}: Add {R}."), creature("{T}: Add {G}."));
    assert_eq!(
        creature("Other Goblins you control get +1/+1."),
        creature("Other Elves you control get +1/+1.")
    );
    // {X} in a cost is a different structure.
    assert_ne!(
        creature("{2}{R}, {T}: Draw a card."),
        creature("{X}{R}, {T}: Draw a card.")
    );
}

#[test]
fn qualifiers_are_kept() {
    // "nontoken"
    assert_ne!(
        creature("Whenever another nontoken creature you control dies, draw a card."),
        creature("Whenever another creature you control dies, draw a card.")
    );
    // "you control"
    assert_ne!(
        creature("Other creatures you control get +1/+1."),
        creature("Other creatures get +1/+1.")
    );
    // "until end of turn"
    assert_ne!(
        instant("Target creature gains flying until end of turn."),
        instant("Target creature gains flying until your next turn.")
    );
    // "up to"
    assert_ne!(
        instant("Destroy up to one target creature."),
        instant("Destroy target creature.")
    );
    // +N/+N and -N/-N
    assert_ne!(
        instant("Target creature gets +2/+2 until end of turn."),
        instant("Target creature gets -2/-2 until end of turn.")
    );
    // Card types and keywords.
    assert_ne!(
        instant("Destroy target artifact."),
        instant("Destroy target enchantment.")
    );
    assert_ne!(creature("Flying"), creature("Trample"));
}
