//! Rulings batch P208 — enchant (CR 303.4, 608.2b): an Aura spell whose target became
//! illegal doesn't resolve. It goes to the graveyard from the stack, so neither its
//! enters triggers nor its "put into a graveyard from the battlefield" triggers trigger.

use crate::r_p209_common::cast_spell;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts `aura` targeting a `host` it controls; in response the host returns to its
/// owner's hand. The Aura spell doesn't resolve: it's in P0's graveyard, and nothing else
/// happened (no cards drawn or discarded, no life change, no tokens, nothing on the
/// battlefield or the stack, the library untouched).
fn fizzles(aura: &str, host: &str) {
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, host);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    let spell = cast_spell(&mut t, P0, aura, &[Entity::Object(target)]);
    assert_eq!(t.zone(spell), Zone::Stack, "{aura} is on the stack");
    move_to(&mut t, target, Zone::Hand(P0));
    let hands = (t.hand_size(P0), t.hand_size(P1));
    let libs = (t.library_size(P0), t.library_size(P1));
    let permanents = t.g.battlefield.len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, aura), "{aura} is put into the graveyard");
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), hands, "{aura}: hands");
    assert_eq!((t.library_size(P0), t.library_size(P1)), libs, "{aura}: libraries");
    assert_eq!(t.g.battlefield.len(), permanents, "{aura}: battlefield");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20), "{aura}: life");
    assert_eq!(t.stack_len(), 0, "{aura}: no triggers");
}

#[test]
fn aura_spells_with_an_illegal_target_dont_resolve_or_trigger() {
    cr!("608.2b", "303.4d", "603.6a", "603.6c", "701.6a");
    ruling!(
        "Aspect of Lamprey",
        "If the creature Aspect of Lamprey would enchant is an illegal target by the time the Aura spell resolves, the entire spell doesn't resolve."
    );
    ruling!(
        "Audacity",
        "If the creature Audacity would enchant is an illegal target by the time the Aura spell resolves, the entire spell doesn't resolve. It's put into your graveyard from the stack"
    );
    ruling!(
        "Bitter Chill",
        "If the creature Bitter Chill would enchant is an illegal target by the time the Aura spell resolves"
    );
    ruling!(
        "Demonic Ruckus",
        "If the creature Demonic Ruckus would enchant is an illegal target"
    );
    ruling!(
        "Giant Inheritance",
        "If the creature Giant Inheritance would enchant is an illegal target"
    );
    ruling!(
        "Grim Reaper's Sprint",
        "you won't untap your creatures or get an additional combat phase"
    );
    ruling!(
        "Iroas's Blessing",
        "If the creature Iroas's Blessing would enchant is an illegal target"
    );
    ruling!(
        "Kenrith's Transformation",
        "If the creature Kenrith's Transformation would enchant is an illegal target"
    );
    ruling!(
        "Mantle of the Wolf",
        "If the creature Mantle of the Wolf would enchant is an illegal target"
    );
    ruling!(
        "Reach for the Sky",
        "If the creature Reach for the Sky would enchant is an illegal target"
    );
    ruling!(
        "Rousing Read",
        "If the creature Rousing Read would enchant is an illegal target"
    );
    ruling!(
        "Setessan Training",
        "If the creature Setessan Training would enchant is an illegal target"
    );
    ruling!(
        "Zoetic Glyph",
        "If the creature Zoetic Glyph would enchant is an illegal target as Zoetic Glyph tries to resolve"
    );
    ruling!(
        "Ancestral Vengeance",
        "If the creature targeted by Ancestral Vengeance's enchant creature ability is an illegal target"
    );
    ruling!(
        "Knightly Valor",
        "If the creature this Aura would enchant is an illegal target by the time Knightly Valor tries to resolve"
    );
    ruling!(
        "Pious Interdiction",
        "If the creature this Aura would enchant is an illegal target by the time Pious Interdiction resolves"
    );
    ruling!(
        "Rancor",
        "It won't enter the battlefield, so it won't be put into a graveyard from the battlefield and its ability won't trigger."
    );
    ruling!(
        "Squire's Devotion",
        "If the creature this Aura would enchant is an illegal target by the time Squire's Devotion tries to resolve"
    );
    ruling!(
        "Winter's Rest",
        "If the creature this Aura would enchant is an illegal target by the time Winter's Rest resolves"
    );
    supported("Grizzly Bears");
    for aura in [
        "Aspect of Lamprey",
        "Audacity",
        "Bitter Chill",
        "Demonic Ruckus",
        "Giant Inheritance",
        "Grim Reaper's Sprint",
        "Iroas's Blessing",
        "Kenrith's Transformation",
        "Mantle of the Wolf",
        "Reach for the Sky",
        "Rousing Read",
        "Setessan Training",
        "Ancestral Vengeance",
        "Knightly Valor",
        "Pious Interdiction",
        "Rancor",
        "Squire's Devotion",
        "Winter's Rest",
    ] {
        fizzles(aura, "Grizzly Bears");
    }
    // Zoetic Glyph enchants an artifact.
    supported("Ornithopter");
    fizzles("Zoetic Glyph", "Ornithopter");
}

#[test]
fn the_same_auras_resolve_and_trigger_with_a_legal_target() {
    cr!("608.3a", "303.4f");
    supported("Grizzly Bears");
    // Control: with the target still there, Rousing Read resolves and draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_spell(&mut t, P0, "Rousing Read", &[Entity::Object(bears)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(t.hand_size(P0), hand + 1); // drew two, discarded one
    // Rancor returns to its owner's hand when put into a graveyard from the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = cast_spell(&mut t, P0, "Rancor", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(rancor));
    move_to(&mut t, bears, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Rancor"));
}
