//! The worked examples of CR 205 (the type line), played out as tests. The 205.3c, 205.3e
//! and 205.4b examples are cited by the tests in `r205_type_line.rs`.

use super::r105_util::{card_from_text, matches};
use mtg_engine::ability::Filter;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the permanent `id` has (exactly) these card types.
fn has_types(t: &TestGame, id: ObjectId, types: &[CardType]) -> bool {
    let c = &t.obj_now(id).chars;
    CardType::ALL.iter().all(|ty| c.is(*ty) == types.contains(ty))
}

#[test]
fn lands_that_are_still_lands_keep_their_other_types() {
    cr!("205.1b", "205.4b", "613.1d");
    cr_example!("205.1b", "All lands are 1/1 creatures that are still lands");
    let mut t = TestGame::new(2);
    let anthem = card_from_text(
        "Animate Lands",
        "{3}{G}{G}",
        "Enchantment",
        None,
        "All lands are 1/1 creatures that are still lands.",
    );
    t.custom(P0, anthem, Zone::Battlefield);
    let seat = t.battlefield(P0, "Seat of the Synod"); // Artifact Land
    let forest = t.battlefield(P0, "Snow-Covered Forest"); // Basic Snow Land — Forest
    let pendel = t.battlefield(P0, "Pendelhaven"); // Legendary Land
    t.g.recompute();
    // An artifact land becomes an artifact land creature.
    use CardType::*;
    assert!(has_types(&t, seat, &[Artifact, Land, Creature]));
    assert_eq!(t.pt(seat), (1, 1));
    // Land types and supertypes are retained.
    assert!(has_types(&t, forest, &[Land, Creature]));
    let c = &t.obj_now(forest).chars;
    assert!(c.has_subtype("Forest"));
    assert!(c.has_supertype(Supertype::Basic) && c.has_supertype(Supertype::Snow));
    assert!(matches(&t, forest, &Filter::Subtype("Forest".into()), P0));
    assert!(t.obj_now(pendel).chars.is_legendary());
    assert!(has_types(&t, pendel, &[Land, Creature]));
}

#[test]
fn artifacts_that_become_artifact_creatures_keep_their_other_types() {
    cr!("205.1b");
    cr_example!("205.1b", "All artifacts are 1/1 artifact creatures");
    let mut t = TestGame::new(2);
    let anthem = card_from_text(
        "Animate Artifacts",
        "{3}{U}",
        "Enchantment",
        None,
        "All artifacts are 1/1 artifact creatures.",
    );
    t.custom(P0, anthem, Zone::Battlefield);
    let whip = t.battlefield(P0, "Whip of Erebos"); // Legendary Enchantment Artifact
    let splitter = t.battlefield(P0, "Bonesplitter"); // Artifact — Equipment
    t.g.recompute();
    use CardType::*;
    assert!(has_types(&t, whip, &[Artifact, Enchantment, Creature]));
    assert!(t.obj_now(whip).chars.is_legendary());
    assert_eq!(t.pt(whip), (1, 1));
    assert!(has_types(&t, splitter, &[Artifact, Creature]));
    assert!(t.obj_now(splitter).chars.has_subtype("Equipment"));
}

/// The CR 205.3b, 302.3 and 305.5 examples: what the cards' type lines mean.
pub fn type_line_examples(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let mountain = t.battlefield(P0, "Mountain"); // Basic Land — Mountain
    let goblin = t.battlefield(P0, "Goblin Electromancer"); // Creature — Goblin Wizard
    let splitter = t.battlefield(P0, "Bonesplitter"); // Artifact — Equipment
    t.g.recompute();
    (mountain, goblin, splitter)
}

/// Whether `id` is a land with the subtype Mountain (CR 205.3b, 305.5).
pub fn is_mountain_land(t: &TestGame, id: ObjectId) -> bool {
    let c = &t.obj_now(id).chars;
    has_types(t, id, &[CardType::Land])
        && c.has_supertype(Supertype::Basic)
        && c.subtypes.len() == 1
        && matches(t, id, &Filter::Subtype("Mountain".into()), P0)
}

/// Whether `id` is a creature with the subtypes Goblin and Wizard (CR 205.3b, 302.3).
pub fn is_goblin_wizard_creature(t: &TestGame, id: ObjectId) -> bool {
    let c = &t.obj_now(id).chars;
    has_types(t, id, &[CardType::Creature])
        && c.subtypes.len() == 2
        && matches(t, id, &Filter::Subtype("Goblin".into()), P0)
        && matches(t, id, &Filter::Subtype("Wizard".into()), P0)
}

#[test]
fn the_type_line_gives_card_types_then_subtypes() {
    cr!("205.3b");
    cr_example!("205.3b", "Artifact — Equipment");
    let mut t = TestGame::new(2);
    let (mountain, goblin, splitter) = type_line_examples(&mut t);
    assert!(is_mountain_land(&t, mountain));
    assert!(is_goblin_wizard_creature(&t, goblin));
    assert!(has_types(&t, splitter, &[CardType::Artifact]));
    assert!(matches(&t, splitter, &Filter::Subtype("Equipment".into()), P0));
    // The parsed type lines say the same.
    let tl = TypeLine::parse("Basic Land — Mountain");
    assert_eq!(tl.card_types, CardTypeSet::single(CardType::Land));
    assert_eq!(tl.subtypes, vec!["Mountain"]);
    let tl = TypeLine::parse("Artifact — Equipment");
    assert_eq!(tl.card_types, CardTypeSet::single(CardType::Artifact));
    assert_eq!(tl.subtypes, vec!["Equipment"]);
}
