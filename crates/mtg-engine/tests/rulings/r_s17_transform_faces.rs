//! Rulings batch S17 — transform (CR 701.27, 712): the faces of nonmodal (transforming)
//! double-faced cards — their characteristics in each zone, mana value, color indicators,
//! entering transformed, token copies, and color identity.

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s17_common::*;
use mtg_engine::card::card;
use mtg_engine::commander_rules::computed_color_identity;
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// Ashling, Rekindled ({1}{R} 1/3 Elemental Sorcerer) // Ashling, Rimebound (blue color
/// indicator, 1/3 Elemental Wizard).
const ASHLING: &str = "Ashling, Rekindled // Ashling, Rimebound";
/// Azusa's Many Journeys ({1}{G} Saga) // Likeness of the Seeker (green color indicator,
/// 3/3 Enchantment Creature — Human Monk).
const AZUSA: &str = "Azusa's Many Journeys // Likeness of the Seeker";
/// Clive, Ifrit's Dominant ({4}{R}{R} 5/5) // Ifrit, Warden of Inferno (9/9 Saga Demon).
const CLIVE: &str = "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno";
/// The Legend of Kyoshi ({4}{G}{G} Saga) // Avatar Kyoshi (colorless 5/4 Avatar).
const KYOSHI: &str = "The Legend of Kyoshi // Avatar Kyoshi";
/// Crystal Fragments ({W} Equipment) // Summon: Alexander (Saga Construct).
const FRAGMENTS: &str = "Crystal Fragments // Summon: Alexander";
const VETERAN: &str = "Lunarch Veteran // Luminous Phantom";

#[test]
fn a_nonmodal_double_faced_cards_mana_value_is_its_front_faces() {
    cr!("202.3b", "712.8e");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "The mana value of a nonmodal double-faced card is the mana value of its front face, no matter which face is up."
    );
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "The mana value of a transforming double-faced card is the mana value of its front face, no matter which face is up."
    );
    supported(ASHLING);
    supported(AZUSA);
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, ASHLING);
    assert_eq!(mana_value(&t, ashling), 2);
    transform(&mut t, ashling);
    assert_eq!(name_of(&t, ashling), "Ashling, Rimebound");
    // The back face has no mana cost, but its mana value is its front face's.
    assert!(t
        .obj_now(ashling)
        .chars
        .mana_cost
        .as_ref()
        .is_none_or(|m| m.to_string().is_empty()));
    assert_eq!(mana_value(&t, ashling), 2);
    // Likeness of the Seeker, put onto the battlefield transformed: 2 ({1}{G}).
    let seeker = enter_transformed(&mut t, P0, AZUSA);
    assert_eq!(name_of(&t, seeker), "Likeness of the Seeker");
    assert_eq!(mana_value(&t, seeker), 2);
    // A spell or ability that cares about mana value sees it: "each creature with mana
    // value 2 or less" includes both.
    let small = t
        .g
        .permanents()
        .filter(|o| o.is_creature() && t.g.mana_value_of(o.id) <= 2)
        .count();
    assert_eq!(small, 2);
}

#[test]
fn a_back_faces_color_indicator_defines_its_color() {
    cr!("202.2e", "204.1", "105.2", "712.8e");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "The back face of a nonmodal double-faced card usually has a color indicator that defines its color."
    );
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "The back face of a transforming double-faced card usually has a color indicator that defines its color."
    );
    let mut t = TestGame::new(2);
    // Ashling, Rekindled is red (its mana cost); Ashling, Rimebound has no mana cost and a
    // blue color indicator: it's blue, not red.
    let ashling = t.battlefield(P0, ASHLING);
    assert_eq!(t.obj_now(ashling).chars.colors, color_set(&[Color::Red]));
    transform(&mut t, ashling);
    assert_eq!(t.obj_now(ashling).chars.colors, color_set(&[Color::Blue]));
    // Likeness of the Seeker: green.
    let seeker = enter_transformed(&mut t, P0, AZUSA);
    assert_eq!(t.obj_now(seeker).chars.colors, color_set(&[Color::Green]));
    // The Legend of Kyoshi's back face, Avatar Kyoshi, has no color indicator: colorless.
    let avatar = enter_transformed(&mut t, P0, KYOSHI);
    assert_eq!(name_of(&t, avatar), "Avatar Kyoshi");
    assert!(t.obj_now(avatar).chars.colors.is_colorless());
}

#[test]
fn a_token_copy_of_a_double_faced_permanent_or_card_is_a_double_faced_token() {
    cr!("707.8", "707.8a", "712.9");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "A token that is created as a copy of a double-faced permanent or a double-faced card in another zone is a double-faced token."
    );
    let mut t = TestGame::new(2);
    // A copy of Ashling with its back face up (P1 gets it, so the legend rule doesn't
    // apply): the token enters with its back face up...
    let ashling = t.battlefield(P0, ASHLING);
    transform(&mut t, ashling);
    let tok = token_copy(&mut t, P1, ashling);
    assert_eq!(tok.len(), 1);
    let tok = tok[0];
    assert_eq!(face(&t, tok), FaceState::Back);
    assert_eq!(name_of(&t, tok), "Ashling, Rimebound");
    // ... and has both faces: it can transform to its front face.
    transform(&mut t, tok);
    assert_eq!(face(&t, tok), FaceState::Front);
    assert_eq!(name_of(&t, tok), "Ashling, Rekindled");
    assert_eq!(t.obj_now(tok).power(), 1);
    // A copy of the card in a graveyard: a double-faced token with its front face up that
    // can transform too.
    let mut t = TestGame::new(2);
    let in_yard = t.graveyard(P1, ASHLING);
    let tok = token_copy(&mut t, P0, in_yard)[0];
    assert_eq!(face(&t, tok), FaceState::Front);
    assert_eq!(name_of(&t, tok), "Ashling, Rekindled");
    transform(&mut t, tok);
    assert_eq!(name_of(&t, tok), "Ashling, Rimebound");
    assert_eq!(t.obj_now(tok).chars.colors, color_set(&[Color::Blue]));
    // A copy of a single-faced permanent can't transform.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tok = token_copy(&mut t, P0, bears)[0];
    transform(&mut t, tok);
    assert_eq!(face(&t, tok), FaceState::Front);
    assert_eq!(name_of(&t, tok), "Grizzly Bears");
}

#[test]
fn a_double_faced_card_enters_front_face_up_unless_it_enters_or_is_cast_transformed() {
    cr!("712.13", "712.14", "712.14a", "712.11a");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "A nonmodal double-faced card enters with its front face up by default, unless a spell or ability instructs you to put it onto the battlefield transformed or allows you to cast it transformed"
    );
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "A transforming double-faced card enters the battlefield with its front face up by default, unless a spell or ability instructs you to put it onto the battlefield transformed or you cast it transformed"
    );
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "A nonmodal double-faced card enters with its front face up by default, unless a spell or ability instructs you to put it onto the battlefield transformed or you cast it transformed"
    );
    supported(KYOSHI);
    supported(VETERAN);
    let mut t = TestGame::new(2);
    // Cast: front face up.
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, ASHLING);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).face, FaceState::Front);
    t.resolve_all();
    assert_eq!(face(&t, spell), FaceState::Front);
    assert_eq!(name_of(&t, spell), "Ashling, Rekindled");
    // Put onto the battlefield by an effect: front face up.
    let card = t.graveyard(P0, AZUSA);
    let perm = put_onto_battlefield(&mut t, P0, card, false).unwrap();
    assert_eq!(face(&t, perm), FaceState::Front);
    assert_eq!(name_of(&t, perm), "Azusa's Many Journeys");
    // Put onto the battlefield transformed: back face up.
    let card = t.graveyard(P1, ASHLING);
    let perm = put_onto_battlefield(&mut t, P1, card, true).unwrap();
    assert_eq!(face(&t, perm), FaceState::Back);
    assert_eq!(name_of(&t, perm), "Ashling, Rimebound");
    // Cast transformed (disturb): back face up.
    t.resolve_all();
    t.lands(P0, "Plains", 2);
    let vet = t.graveyard(P0, VETERAN);
    let spell = t
        .cast(P0, vet)
        .method(CastMethod::Keyword(KeywordKind::Disturb))
        .go();
    t.resolve_all();
    assert_eq!(face(&t, spell), FaceState::Back);
    assert_eq!(name_of(&t, spell), "Luminous Phantom");
    // The Legend of Kyoshi is cast as a Saga; its chapter III returns it transformed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let card = t.hand(P0, KYOSHI);
    let spell = t.cast(P0, card).go();
    t.resolve_all();
    let saga = t.g.current(spell);
    assert_eq!(face(&t, saga), FaceState::Front);
    assert_eq!(name_of(&t, saga), "The Legend of Kyoshi");
    t.g.add_counters(Entity::Object(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.resolve_all();
    let avatar = t.named_on_battlefield("Avatar Kyoshi");
    assert_eq!(avatar.len(), 1);
    assert_eq!(t.obj(avatar[0]).face, FaceState::Back);
}

#[test]
fn off_the_battlefield_only_the_front_face_counts_and_on_it_only_the_face_thats_up() {
    cr!("712.8a", "712.8c", "712.8d", "712.8e", "712.11");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "Each nonmodal double-faced card in this release is cast face up. In every zone other than the battlefield, consider only the characteristics of its front face."
    );
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "Each nonmodal double-faced card in this set is cast face up. In every zone other than the battlefield, consider only the characteristics of its front face."
    );
    let mut t = TestGame::new(2);
    // In each zone other than the battlefield: Ashling, Rekindled, a red {1}{R} Sorcerer.
    let in_hand = t.hand(P0, ASHLING);
    let in_yard = t.graveyard(P0, ASHLING);
    let in_exile = t.exile(P0, ASHLING);
    let in_library = t.library_top(P0, ASHLING);
    for id in [in_hand, in_yard, in_exile, in_library] {
        let o = t.obj(id);
        assert_eq!(o.chars.name, "Ashling, Rekindled");
        assert_eq!(o.chars.colors, color_set(&[Color::Red]));
        assert!(o.chars.has_subtype("Sorcerer") && !o.chars.has_subtype("Wizard"));
        assert_eq!(t.g.mana_value_of(id), 2);
    }
    // On the stack: cast face up.
    t.lands(P0, "Mountain", 2);
    let spell = t.cast(P0, in_hand).go();
    assert_eq!(t.obj(spell).face, FaceState::Front);
    assert_eq!(t.obj(spell).chars.name, "Ashling, Rekindled");
    t.resolve_all();
    // On the battlefield back face up: only Ashling, Rimebound's characteristics.
    transform(&mut t, spell);
    let o = t.obj_now(spell);
    assert_eq!(o.chars.name, "Ashling, Rimebound");
    assert!(o.chars.has_subtype("Wizard") && !o.chars.has_subtype("Sorcerer"));
    assert_eq!(o.chars.colors, color_set(&[Color::Blue]));
    // The Legend of Kyoshi in hand is a green Saga, not the colorless Avatar creature.
    let kyoshi = t.hand(P1, KYOSHI);
    let o = t.obj(kyoshi);
    assert_eq!(o.chars.name, "The Legend of Kyoshi");
    assert!(o.is(CardType::Enchantment) && !o.is(CardType::Creature));
    assert_eq!(o.chars.colors, color_set(&[Color::Green]));
    assert_eq!(t.g.mana_value_of(kyoshi), 6);
    // Avatar Kyoshi on the battlefield: a 5/4 legendary creature, no Saga.
    let avatar = enter_transformed(&mut t, P1, KYOSHI);
    let o = t.obj_now(avatar);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Enchantment));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!((o.power(), o.toughness()), (5, 4));
}

#[test]
fn each_face_has_its_own_characteristics_and_only_the_face_up_counts() {
    cr!("712.8", "712.8d", "712.8e", "712.18");
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "Each face of a nonmodal double-faced card has its own set of characteristics: name, types, subtypes, abilities, and so on. While a nonmodal double-faced permanent is on the battlefield"
    );
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "Each face of a transforming double-faced card has its own set of characteristics: name, types, subtypes, abilities, and so on."
    );
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "Each face of a nonmodal double-faced card has its own set of characteristics: name, types, subtypes, abilities, and so on. While a double-faced permanent is on the battlefield"
    );
    supported(CLIVE);
    let mut t = TestGame::new(2);
    // Clive, Ifrit's Dominant: a 5/5 Human Noble Warrior with an activated ability.
    let clive = t.battlefield(P0, CLIVE);
    let o = t.obj_now(clive);
    assert_eq!((o.power(), o.toughness()), (5, 5));
    assert!(o.chars.has_subtype("Human") && !o.chars.has_subtype("Demon"));
    assert!(!o.is(CardType::Enchantment));
    let activated = |t: &TestGame, id| {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
            .count()
    };
    assert_eq!(activated(&t, clive), 1);
    // Transformed: Ifrit, Warden of Inferno, a 9/9 Enchantment Creature — Saga Demon with
    // chapter abilities and no activated ability. It's the same object.
    transform(&mut t, clive);
    assert!(t.g.is_live(clive));
    let o = t.obj_now(clive);
    assert_eq!(o.chars.name, "Ifrit, Warden of Inferno");
    assert_eq!((o.power(), o.toughness()), (9, 9));
    assert!(o.is(CardType::Enchantment) && o.chars.has_subtype("Saga"));
    assert!(o.chars.has_subtype("Demon") && !o.chars.has_subtype("Human"));
    assert_eq!(activated(&t, clive), 0);
    // Azusa's Many Journeys (a noncreature Saga) and Likeness of the Seeker (a 3/3 Human
    // Monk creature).
    let saga = t.battlefield(P1, AZUSA);
    assert!(!t.obj_now(saga).is_creature());
    assert!(t.obj_now(saga).chars.has_subtype("Saga"));
    let seeker = enter_transformed(&mut t, P1, AZUSA);
    let o = t.obj_now(seeker);
    assert!(o.is_creature() && !o.chars.has_subtype("Saga"));
    assert!(o.chars.has_subtype("Monk"));
    assert_eq!((o.power(), o.toughness()), (3, 3));
    // Ashling: the Sorcerer's "you may discard a card. If you do, draw a card" trigger
    // isn't on the Wizard face; its "enters or transforms into Ashling, Rekindled" works
    // only when it transforms into that face.
    let ashling = t.battlefield(P0, ASHLING);
    let texts = |t: &TestGame, id| -> Vec<String> {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .map(|a| a.text.to_string())
            .collect()
    };
    assert!(texts(&t, ashling).iter().any(|s| s.contains("discard a card")));
    transform(&mut t, ashling);
    assert!(!texts(&t, ashling).iter().any(|s| s.contains("discard a card")));
    assert!(texts(&t, ashling).iter().any(|s| s.contains("add two mana")));
}

#[test]
fn a_card_that_isnt_double_faced_isnt_put_onto_the_battlefield_transformed() {
    cr!("712.14a", "701.27c");
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "For example, if a single-faced card is a copy of Crystal Fragments, it will be exiled during the resolution of its second ability and remain in exile."
    );
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "If you are instructed to put a card that isn't a double-faced card onto the battlefield transformed, it will not enter at all."
    );
    supported(FRAGMENTS);
    // Crystal Fragments: "{5}{W}{W}: Exile this Equipment, then return it to the
    // battlefield transformed under its owner's control."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 7);
    let fragments = t.battlefield(P0, FRAGMENTS);
    activate_containing(&mut t, P0, fragments, "transformed").unwrap();
    t.resolve_all();
    let alexander = t.named_on_battlefield("Summon: Alexander");
    assert_eq!(alexander.len(), 1);
    assert_eq!(t.obj(alexander[0]).face, FaceState::Back);
    // Bonesplitter (a single-faced card) that's a copy of Crystal Fragments is exiled and
    // stays in exile.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 7);
    let original = t.battlefield(P1, FRAGMENTS);
    let splitter = t.battlefield(P0, "Bonesplitter");
    become_copy(&mut t, splitter, original);
    assert_eq!(name_of(&t, splitter), "Crystal Fragments");
    // Told to transform, it doesn't: it isn't represented by a double-faced card.
    transform(&mut t, splitter);
    assert_eq!(name_of(&t, splitter), "Crystal Fragments");
    assert_eq!(face(&t, splitter), FaceState::Front);
    activate_containing(&mut t, P0, splitter, "transformed").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Bonesplitter"));
    assert!(t.named_on_battlefield("Summon: Alexander").is_empty());
    assert!(t.named_on_battlefield("Bonesplitter").is_empty());
    // An effect putting a single-faced card from a graveyard onto the battlefield
    // transformed leaves it in the graveyard.
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(put_onto_battlefield(&mut t, P0, bears, true).is_none());
    assert!(t.g.is_live(bears));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}

/// A Commander deck: the commander, `extra`, and basic lands up to 100 cards.
fn commander_deck(
    commander: &Arc<mtg_engine::card::CardDef>,
    extra: Vec<Arc<mtg_engine::card::CardDef>>,
    land: &str,
) -> Vec<Arc<mtg_engine::card::CardDef>> {
    let mut d = vec![commander.clone()];
    let n = 99 - extra.len();
    d.extend(extra);
    d.extend((0..n).map(|_| card(land)));
    d
}

fn outside_identity(problems: &[DeckProblem], name: &str) -> bool {
    problems
        .iter()
        .any(|p| matches!(p, DeckProblem::OutsideColorIdentity { name: n } if n == name))
}

#[test]
fn a_double_faced_cards_color_identity_combines_both_faces() {
    cr!("903.4", "903.4d");
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "For example, Cecil, Dark Knight's color identity is black and white, since its front face is black and its back face has a white color indicator."
    );
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "For example, Oko, Lorwyn Liege's color identity is green and blue, since its front face is blue, its rules text contains a green mana symbol, and its back face has a green color indicator as well as a blue mana symbol in its rules text."
    );
    // Cecil, Dark Knight ({B}) // Cecil, Redeemed Paladin (white color indicator).
    let cecil = card("Cecil, Dark Knight // Cecil, Redeemed Paladin");
    assert_eq!(cecil.front().chars.colors, color_set(&[Color::Black]));
    assert_eq!(
        computed_color_identity(&cecil),
        color_set(&[Color::White, Color::Black])
    );
    // Oko, Lorwyn Liege ({2}{U}, "you may pay {G}") // Oko, Shadowmoor Scion (green
    // indicator, "you may pay {U}").
    let oko = card("Oko, Lorwyn Liege // Oko, Shadowmoor Scion");
    assert_eq!(oko.front().chars.colors, color_set(&[Color::Blue]));
    assert_eq!(
        computed_color_identity(&oko),
        color_set(&[Color::Blue, Color::Green])
    );
    // Ashling ({1}{R}, "you may pay {U}") // Ashling, Rimebound (blue indicator): blue and
    // red. Clive ({4}{R}{R}) // Ifrit (red indicator): red.
    let ashling = card(ASHLING);
    assert_eq!(
        computed_color_identity(&ashling),
        color_set(&[Color::Blue, Color::Red])
    );
    assert_eq!(computed_color_identity(&card(CLIVE)), color_set(&[Color::Red]));
    // So neither Cecil nor Ashling can be in a mono-colored deck of its front face's color.
    let krenko = card("Krenko, Mob Boss");
    let d = commander_deck(&krenko, vec![ashling.clone()], "Mountain");
    assert!(outside_identity(
        &check_commander(&d, &krenko, &[], false),
        &ashling.name
    ));
    let d = commander_deck(&krenko, vec![card(CLIVE)], "Mountain");
    assert!(check_commander(&d, &krenko, &[], false).is_empty());
    let liliana = card("Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    let d = commander_deck(&liliana, vec![cecil.clone()], "Swamp");
    assert!(outside_identity(
        &check_commander(&d, &liliana, &[], false),
        &cecil.name
    ));
}
