//! CR 107.3c: "where X is ..." defines X for the whole ability, including the number
//! of its targets and what may be targeted ("counter target spell with mana value X or
//! less, where X is the number of Faeries you control").

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn spellstutter_sprite_counts_faeries_for_its_target() {
    cr!("107.3c", "115.1");
    assert_supported("Spellstutter Sprite");
    let mut t = TestGame::new(2);
    // Two Faeries once the Sprite is on the battlefield: X is 2.
    t.battlefield(P0, "Faerie Miscreant");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.enter(P0, "Spellstutter Sprite");
    t.resolve_all();
    // The mana value 2 spell was countered.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn spellstutter_sprite_cant_target_a_spell_with_greater_mana_value() {
    cr!("107.3c", "603.3d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Miscreant");
    // Not a Faerie: three creatures, but X is still 2.
    t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    let courser = t.hand(P0, "Centaur Courser");
    let spell = t.cast(P0, courser).go();
    // Mana value 3 with two Faeries: not a legal target, so the ability is removed.
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.enter(P0, "Spellstutter Sprite");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
}

#[test]
fn spellstutter_sprite_rechecks_x_on_resolution() {
    cr!("107.3c", "608.2b");
    ruling!(
        "Spellstutter Sprite",
        "If the number of Faeries you control has decreased enough in that time to make the target illegal"
    );
    let mut t = TestGame::new(2);
    let miscreant = t.battlefield(P0, "Faerie Miscreant");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.enter(P0, "Spellstutter Sprite");
    t.settle();
    // X was 2 as the ability triggered: it targets the spell.
    assert_eq!(t.stack_len(), 2);
    // The other Faerie leaves before the ability resolves: X is 1 and the target is
    // illegal, so the spell resolves.
    t.g.destroy(miscreant, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
