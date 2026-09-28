//! Rulings batch S17 — transform (CR 701.27, 712): modal double-faced cards (Eddie Brock
//! // Venom, Lethal Protector): each face's characteristics and mana value, transforming
//! them and putting them onto the battlefield transformed, and effects that put a card
//! with a characteristic onto the battlefield.

use crate::r_s01_common::*;
use crate::r_s08_common::mana_value;
use crate::r_s17_common::*;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Eddie Brock ({2}{B} 3/3; "{3}{B}{R}{G}: Transform Eddie Brock. Activate only as a
/// sorcery.") // Venom, Lethal Protector ({3}{B}{R}{G} 5/5).
const EDDIE: &str = "Eddie Brock // Venom, Lethal Protector";
/// Selfless Glyphweaver (creature) // Deadly Vanity (sorcery).
const GLYPHWEAVER: &str = "Selfless Glyphweaver // Deadly Vanity";

/// Lands for {3}{B}{R}{G}.
fn venom_mana(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Swamp", 1);
    t.lands(p, "Mountain", 1);
    t.lands(p, "Forest", 1);
    t.lands(p, "Wastes", 3);
}

#[test]
fn a_modal_double_faced_cards_mana_value_is_the_face_up_on_the_stack_or_battlefield() {
    cr!("202.3", "712.8a", "712.8f", "712.11b");
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "The mana value of a modal double-faced card is based on the characteristics of the face that's being considered. On the stack or the battlefield, consider whichever face is up. In all other zones, consider only the front face."
    );
    supported(EDDIE);
    let mut t = TestGame::new(2);
    // In hand: Eddie Brock's mana value, 3.
    let card = t.hand(P0, EDDIE);
    assert_eq!(mana_value(&t, card), 3);
    // Cast as Venom: 6 on the stack and on the battlefield.
    venom_mana(&mut t, P0);
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj(spell).chars.name, "Venom, Lethal Protector");
    assert_eq!(mana_value(&t, spell), 6);
    t.resolve_all();
    assert_eq!(face(&t, spell), FaceState::Back);
    assert_eq!(mana_value(&t, spell), 6);
    // In the graveyard: 3 again.
    let yard = t.graveyard(P1, EDDIE);
    assert_eq!(mana_value(&t, yard), 3);
    // Eddie Brock transformed by its own ability into Venom: 6 (unlike a transforming
    // double-faced card, whose back face uses its front face's mana cost).
    let mut t = TestGame::new(2);
    let eddie = t.battlefield(P0, EDDIE);
    assert_eq!(mana_value(&t, eddie), 3);
    venom_mana(&mut t, P0);
    t.activate(P0, eddie, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, eddie), "Venom, Lethal Protector");
    assert_eq!(mana_value(&t, eddie), 6);
}

#[test]
fn a_modal_double_faced_card_has_only_the_characteristics_of_the_face_up() {
    cr!("712.8a", "712.8f");
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "Each face of a double-faced card has its own set of characteristics: name, types, subtypes, abilities, and so on. While a double-faced card is on the stack or battlefield, consider only the characteristics of the face that's currently up."
    );
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "While a double-faced card isn't on the stack or battlefield, consider only the characteristics of its front face."
    );
    let mut t = TestGame::new(2);
    // Off the stack and battlefield: a black 3/3 Human Hero Villain named Eddie Brock.
    for id in [t.graveyard(P0, EDDIE), t.exile(P0, EDDIE), t.hand(P1, EDDIE)] {
        let o = t.obj(id);
        assert_eq!(o.chars.name, "Eddie Brock");
        assert_eq!(o.chars.colors, color_set(&[Color::Black]));
        assert_eq!((o.chars.power, o.chars.toughness), (Some(3), Some(3)));
        assert!(o.chars.has_subtype("Human") && !o.chars.has_subtype("Symbiote"));
    }
    // Venom on the stack: a black, red, and green 5/5 Symbiote with menace, trample, and
    // haste — none of Eddie's characteristics.
    let card = t.hand(P0, EDDIE);
    venom_mana(&mut t, P0);
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    let o = t.obj(spell);
    assert_eq!(
        o.chars.colors,
        color_set(&[Color::Black, Color::Red, Color::Green])
    );
    assert!(o.chars.has_subtype("Symbiote") && !o.chars.has_subtype("Human"));
    t.resolve_all();
    let venom = t.g.current(spell);
    assert_eq!(t.pt(venom), (5, 5));
    assert!(t.obj(venom).has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    // Eddie Brock on the battlefield: no haste, no trample.
    let eddie = t.battlefield(P1, EDDIE);
    assert_eq!(t.pt(eddie), (3, 3));
    assert!(!t
        .obj(eddie)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert!(!t
        .obj(eddie)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn a_modal_double_faced_card_transforms_only_into_a_permanent_face() {
    cr!("712.3", "712.10", "701.27d", "712.14a");
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "A modal double-faced card can be transformed or be put onto the battlefield transformed."
    );
    // Eddie Brock transforms into Venom, Lethal Protector.
    let mut t = TestGame::new(2);
    let eddie = t.battlefield(P0, EDDIE);
    transform(&mut t, eddie);
    assert_eq!(name_of(&t, eddie), "Venom, Lethal Protector");
    assert_eq!(face(&t, eddie), FaceState::Back);
    // Selfless Glyphweaver's other face is a sorcery: it doesn't transform.
    let glyph = t.battlefield(P0, GLYPHWEAVER);
    transform(&mut t, glyph);
    assert_eq!(name_of(&t, glyph), "Selfless Glyphweaver");
    assert_eq!(face(&t, glyph), FaceState::Front);
    // Put onto the battlefield transformed: Eddie Brock enters as Venom...
    let card = t.graveyard(P1, EDDIE);
    let venom = put_onto_battlefield(&mut t, P1, card, true).unwrap();
    assert_eq!(name_of(&t, venom), "Venom, Lethal Protector");
    assert_eq!(face(&t, venom), FaceState::Back);
    // ... and Selfless Glyphweaver stays in the graveyard.
    let card = t.graveyard(P1, GLYPHWEAVER);
    assert!(put_onto_battlefield(&mut t, P1, card, true).is_none());
    assert_eq!(t.zone(card), Zone::Graveyard(P1));
}

#[test]
fn a_card_put_onto_the_battlefield_qualifies_by_its_front_face_and_enters_front_face_up() {
    cr!("712.8a", "712.14", "712.14b");
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "If an effect allows you to put a card with particular characteristics onto the battlefield without instructing you to play or cast it, you consider only the characteristics of a modal double-faced card's front face to see if that card qualifies."
    );
    supported("Elvish Piper");
    supported("Growth Spiral");
    // Elvish Piper: "You may put a creature card from your hand onto the battlefield."
    // Eddie Brock qualifies and enters as Eddie Brock.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let piper = t.battlefield(P0, "Elvish Piper");
    let eddie = t.hand(P0, EDDIE);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(eddie)]);
    t.activate(P0, piper, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(eddie), Zone::Battlefield);
    assert_eq!(face(&t, eddie), FaceState::Front);
    assert_eq!(name_of(&t, eddie), "Eddie Brock");
    // Growth Spiral: "You may put a land card from your hand onto the battlefield." Kazandu
    // Mammoth (whose back face is a land) isn't a land card in hand; Brightclimb Pathway
    // is, and enters front face up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let mammoth = t.hand(P0, "Kazandu Mammoth // Kazandu Valley");
    let pathway = t.hand(P0, "Brightclimb Pathway // Grimclimb Pathway");
    let spiral = t.hand(P0, "Growth Spiral");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(pathway)]);
    t.cast(P0, spiral).go();
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&Entity::Object(pathway)));
    assert!(!offered.contains(&Entity::Object(mammoth)));
    assert_eq!(t.zone(pathway), Zone::Battlefield);
    assert_eq!(name_of(&t, pathway), "Brightclimb Pathway");
    assert_eq!(t.zone(mammoth), Zone::Hand(P0));
}
