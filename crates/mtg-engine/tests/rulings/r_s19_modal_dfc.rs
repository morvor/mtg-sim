//! Rulings on modal double-faced cards (CR 712.3, 712.8, 712.11b, 712.12, 712.14): playing
//! or casting either face, permissions to play and to cast, the legality of each face,
//! naming a face, mana value, putting one onto the battlefield, and color identity.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land, target_candidates};
use crate::r_s03_common::{in_hand_with_mana, run_effect};
use crate::r_s08_common::{legal_cast_methods, mana_value};
use mtg_engine::ability::{Destination, Effect, Sel};
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::computed_color_identity;
use mtg_engine::decision::Answer;
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

const FELL: &str = "Fell the Profane // Fell Mire";
const BALA_GED: &str = "Bala Ged Recovery // Bala Ged Sanctuary";
const MALAKIR: &str = "Malakir Rebirth // Malakir Mire";
const WITCH: &str = "Witch Enchanter // Witch-Blessed Meadow";
const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";

/// The card in exile whose (front face) name is `name`.
fn exiled(t: &TestGame, name: &str) -> ObjectId {
    t.g.find_in_zone(Zone::Exile, name)[0]
}

/// P0 casts Reckless Impulse ("Exile the top two cards of your library. Until the end of
/// your next turn, you may play those cards.") with the modal double-faced card `mdfc` on
/// top of the library, and gets `lands` to pay for its spell face. Returns the exiled card.
fn impulse(t: &mut TestGame, mdfc: &str, front: &str, lands: &[&str]) -> ObjectId {
    stack_library(t, P0, &[mdfc, "Island"]);
    let ri = in_hand_with_mana(t, P0, "Reckless Impulse");
    t.cast(P0, ri).go();
    t.resolve_all();
    for l in lands {
        t.lands(P0, l, 1);
    }
    exiled(t, front)
}

/// Snapcaster Mage ("When this creature enters, target instant or sorcery card in your
/// graveyard gains flashback until end of turn. The flashback cost is equal to its mana
/// cost.") gives the modal double-faced card `card_id` in P0's graveyard flashback.
fn snapcaster(t: &mut TestGame, card_id: ObjectId) {
    t.answer_targets(P0, &[Entity::Object(card_id)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
}

#[test]
fn permission_to_play_a_specific_mdfc_allows_either_face_but_to_cast_it_only_the_spell() {
    cr!("712.11b", "712.12", "305.1");
    ruling!(
        "Fell the Profane // Fell Mire",
        "If an effect allows you to play a specific modal double-faced card, you may cast it as a spell or play it as a land, as determined by which face you choose to play. If an effect allows you to cast (rather than \"play\") a specific modal double-faced card, you can't play it as a land."
    );
    supported(FELL);
    supported("Reckless Impulse");
    supported("Snapcaster Mage");
    // "You may play those cards": either Fell the Profane (an instant) or Fell Mire.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let fell = impulse(&mut t, FELL, "Fell the Profane", &["Swamp", "Swamp", "Wastes", "Wastes"]);
    assert!(!legal_cast_methods(&mut t, P0, fell).is_empty());
    assert!(can_play_land(&mut t, P0, fell));
    t.play_land(P0, fell).unwrap();
    assert_eq!(t.named_on_battlefield("Fell Mire").len(), 1);
    // Flashback lets a player cast it: not play Fell Mire.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let fell = t.graveyard(P0, FELL);
    snapcaster(&mut t, fell);
    t.lands(P0, "Swamp", 4);
    assert!(legal_cast_methods(&mut t, P0, fell).contains(&CastMethod::Keyword(
        mtg_engine::keywords::KeywordKind::Flashback
    )));
    assert!(!can_play_land(&mut t, P0, fell));
}

#[test]
fn permission_to_play_a_specific_mdfc_allows_either_face_curly_quotes() {
    cr!("712.11b", "712.12");
    ruling!(
        "Bala Ged Recovery // Bala Ged Sanctuary",
        "If an effect allows you to play a specific modal double-faced card, you may cast it as a spell or play it as a land, as determined by which face you choose to play. If an effect allows you to cast (rather than “play”) a specific modal double-faced card, you can’t play it as a land."
    );
    supported(BALA_GED);
    let mut t = TestGame::new(2);
    let bala = impulse(&mut t, BALA_GED, "Bala Ged Recovery", &["Forest", "Wastes", "Wastes"]);
    // (Reckless Impulse is in the graveyard: a target for Bala Ged Recovery.)
    assert!(!legal_cast_methods(&mut t, P0, bala).is_empty());
    assert!(can_play_land(&mut t, P0, bala));
    // Cast with flashback: not played as Bala Ged Sanctuary.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bala = t.graveyard(P0, BALA_GED);
    snapcaster(&mut t, bala);
    t.lands(P0, "Forest", 3);
    assert!(!legal_cast_methods(&mut t, P0, bala).is_empty());
    assert!(!can_play_land(&mut t, P0, bala));
    t.cast(P0, bala)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn playing_lands_from_a_graveyard_allows_only_the_land_face() {
    cr!("712.12", "305.1", "601.3");
    ruling!(
        "Malakir Rebirth // Malakir Mire",
        "If an effect allows you to play a land or cast a spell from among a group of cards, you may play or cast a modal double-faced card with any face that fits the criteria of that effect."
    );
    ruling!(
        "Fell the Profane // Fell Mire",
        "For example, if an effect allows you to play lands from your graveyard, you can play Garden of Freyalise, but you can't cast Disciple of Freyalise."
    );
    supported(MALAKIR);
    supported("Crucible of Worlds");
    // Crucible of Worlds: "You may play lands from your graveyard."
    for (mdfc, land) in [(MALAKIR, "Malakir Mire"), (FELL, "Fell Mire")] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Crucible of Worlds");
        t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let c = t.graveyard(P0, mdfc);
        assert!(can_play_land(&mut t, P0, c));
        assert!(legal_cast_methods(&mut t, P0, c).is_empty());
        t.play_land(P0, c).unwrap();
        assert_eq!(t.named_on_battlefield(land).len(), 1);
    }
}

#[test]
fn only_the_face_being_played_determines_whether_it_can_be_played() {
    cr!("712.11b", "712.12", "305.2", "117.1a");
    ruling!(
        "Malakir Rebirth // Malakir Mire",
        "To determine whether it is legal to play a modal double-faced card, consider only the characteristics of the face you're playing and ignore the other face's characteristics."
    );
    // Malakir Rebirth ({B} instant) can be cast in the opponent's turn; Malakir Mire can't
    // be played then.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let m = t.hand(P0, MALAKIR);
    t.set_step(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, m, CastMethod::Normal));
    assert!(!can_play_land(&mut t, P0, m));
    // In P0's main phase after a land was played: the land face can't be played, the
    // instant can still be cast.
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P0, m));
    let other = t.hand(P0, "Swamp");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(&mut t, P0, m));
    assert!(can_cast(&mut t, P0, m, CastMethod::Normal));
}

#[test]
fn a_restriction_on_creature_spells_doesnt_stop_the_land_face() {
    cr!("712.11b", "712.12", "305.1");
    ruling!(
        "Witch Enchanter // Witch-Blessed Meadow",
        "For example, if an effect stops you from casting creature spells, you can't cast Disciple of Freyalise, but you can still play Garden of Freyalise."
    );
    supported(WITCH);
    supported("Steel Golem");
    // Steel Golem: "You can't cast creature spells."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Steel Golem");
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, WITCH);
    assert!(!can_cast(&mut t, P0, w, CastMethod::Normal));
    assert!(can_play_land(&mut t, P0, w));
    // Without Steel Golem it could be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, WITCH);
    assert!(can_cast(&mut t, P0, w, CastMethod::Normal));
}

#[test]
fn a_restriction_on_creature_spells_doesnt_stop_a_noncreature_back_face() {
    cr!("712.11b", "601.3");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "To determine whether it is legal to play a modal double-faced card, consider only the characteristics of the face you’re playing and ignore the other face’s characteristics."
    );
    supported(BIRGI);
    // Steel Golem: "You can't cast creature spells." Birgi is a creature; Harnfel, Horn of
    // Bounty is an artifact.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Steel Golem");
    t.lands(P0, "Mountain", 5);
    let b = t.hand(P0, BIRGI);
    assert!(!can_cast(&mut t, P0, b, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, b, CastMethod::Half(1)));
    let spell = t.cast(P0, b).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj(spell).chars.name, "Harnfel, Horn of Bounty");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Harnfel, Horn of Bounty").len(), 1);
}

#[test]
fn a_chosen_name_is_one_faces_name_and_applies_only_to_that_face() {
    cr!("712.19", "201.4d");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "If an effect instructs a player to choose a card name, the name of either face may be chosen. If that effect or a linked ability refers to a spell with the chosen name being cast and/or a land with the chosen name being played, it considers only the chosen name, not the other face’s name."
    );
    supported("Meddling Mage");
    // Meddling Mage: "As this creature enters, choose a nonland card name. Spells with the
    // chosen name can't be cast."
    for (named, blocked, allowed) in [
        ("Harnfel, Horn of Bounty", CastMethod::Half(1), CastMethod::Normal),
        ("Birgi, God of Storytelling", CastMethod::Normal, CastMethod::Half(1)),
    ] {
        let mut t = TestGame::new(2);
        t.answer(P1, DecisionKind::Name, Answer::Text(named.into()));
        let mage = t.enter(P1, "Meddling Mage");
        assert_eq!(t.obj(mage).choices.card_name.as_deref(), Some(named));
        t.lands(P0, "Mountain", 5);
        let b = t.hand(P0, BIRGI);
        assert!(!can_cast(&mut t, P0, b, blocked), "{named}");
        assert!(can_cast(&mut t, P0, b, allowed), "{named}");
    }
}

#[test]
fn an_mdfcs_mana_value_is_that_of_the_face_up_or_its_front_face() {
    cr!("712.8a", "712.8f", "202.3");
    ruling!(
        "Fell the Profane // Fell Mire",
        "The mana value of a modal double-faced card is based on the characteristics of the face that's being considered. On the stack or the battlefield, consider whichever face is up. In all other zones, consider only the front face."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let fell = t.hand(P0, FELL);
    assert_eq!(mana_value(&t, fell), 4);
    let gy = t.graveyard(P0, FELL);
    assert_eq!(mana_value(&t, gy), 4);
    // On the stack as Fell the Profane: 4.
    t.lands(P0, "Swamp", 4);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    let spell = t.cast(P0, fell).target(bears).go();
    assert_eq!(mana_value(&t, spell), 4);
    t.resolve_all();
    // On the battlefield as Fell Mire: 0.
    let mire = t.hand(P0, FELL);
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, mire).unwrap();
    let mire = t.g.current(mire);
    assert_eq!(t.obj(mire).chars.name, "Fell Mire");
    assert_eq!(mana_value(&t, mire), 0);
}

#[test]
fn an_mdfcs_mana_value_is_that_of_the_face_up_or_its_front_face_curly_quotes() {
    cr!("712.8a", "712.8f", "202.3");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "The mana value of a modal double-faced card is based on the characteristics of the face that’s being considered. On the stack and battlefield, consider whichever face is up. In all other zones, consider only the front face."
    );
    // Birgi, God of Storytelling {2}{R} // Harnfel, Horn of Bounty {4}{R}.
    let mut t = TestGame::new(2);
    let b = t.hand(P0, BIRGI);
    assert_eq!(mana_value(&t, b), 3);
    t.lands(P0, "Mountain", 5);
    let spell = t.cast(P0, b).method(CastMethod::Half(1)).go();
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve_all();
    let harnfel = t.g.current(spell);
    assert_eq!(t.obj(harnfel).chars.name, "Harnfel, Horn of Bounty");
    assert_eq!(mana_value(&t, harnfel), 5);
    // In the graveyard: its front face, 3.
    let gy = t.graveyard(P0, BIRGI);
    assert_eq!(mana_value(&t, gy), 3);
    // Bala Ged Recovery {2}{G} in the library is 3; played as Bala Ged Sanctuary it's 0.
    let lib = t.library_top(P0, BALA_GED);
    assert_eq!(mana_value(&t, lib), 3);
    let sanctuary = t.hand(P0, BALA_GED);
    t.play_land(P0, sanctuary).unwrap();
    let sanctuary = t.g.current(sanctuary);
    assert_eq!(t.obj(sanctuary).chars.name, "Bala Ged Sanctuary");
    assert_eq!(mana_value(&t, sanctuary), 0);
}

#[test]
fn putting_an_mdfc_onto_the_battlefield_looks_at_its_front_face() {
    cr!("712.8a", "712.14", "712.14b");
    ruling!(
        "Witch Enchanter // Witch-Blessed Meadow",
        "you consider only the characteristics of a modal double-faced card's front face to see if that card qualifies. If it does, it enters the battlefield with its front face up."
    );
    supported("Reanimate");
    supported("Life from the Loam");
    // Reanimate: "Put target creature card from a graveyard onto the battlefield under
    // your control. You lose life equal to its mana value."
    let mut t = TestGame::new(2);
    let w = t.graveyard(P0, WITCH);
    t.lands(P0, "Swamp", 1);
    let r = t.hand(P0, "Reanimate");
    t.cast(P0, r).target(w).go();
    t.resolve_all();
    let enchanter = t.named_on_battlefield("Witch Enchanter");
    assert_eq!(enchanter.len(), 1);
    assert_eq!(t.obj(enchanter[0]).face, FaceState::Front);
    assert_eq!(t.life(P0), 16);
    // Life from the Loam: "Return up to three target land cards from your graveyard to
    // your hand." Witch Enchanter and Fell the Profane aren't land cards there.
    let mut t = TestGame::new(2);
    let w = t.graveyard(P0, WITCH);
    let f = t.graveyard(P0, FELL);
    let forest = t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let loam = t.hand(P0, "Life from the Loam");
    let from = t.asked().len();
    t.cast(P0, loam).targets(&[Entity::Object(forest)]).go();
    let offered = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(forest)));
    assert!(!offered.contains(&Entity::Object(w)));
    assert!(!offered.contains(&Entity::Object(f)));
}

#[test]
fn an_mdfc_put_onto_the_battlefield_enters_front_face_up_or_not_at_all() {
    cr!("712.14", "712.14b");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "If an effect puts a double-faced card onto the battlefield, it enters with its front face up. If that front face can’t be put onto the battlefield, it doesn’t enter the battlefield."
    );
    // Reanimate: Birgi enters as Birgi, God of Storytelling.
    let mut t = TestGame::new(2);
    let b = t.graveyard(P0, BIRGI);
    t.lands(P0, "Swamp", 1);
    let r = t.hand(P0, "Reanimate");
    t.cast(P0, r).target(b).go();
    t.resolve_all();
    let birgi = t.g.current(b);
    assert_eq!(t.zone(birgi), Zone::Battlefield);
    assert_eq!(t.obj(birgi).face, FaceState::Front);
    assert_eq!(t.obj(birgi).chars.name, "Birgi, God of Storytelling");
    // Bala Ged Recovery's front face is a sorcery: an instruction to put it onto the
    // battlefield leaves it in the graveyard.
    let bala = t.graveyard(P0, BALA_GED);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(bala)],
    );
    assert_eq!(t.zone(bala), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Bala Ged Sanctuary").is_empty());
}

/// A 100-card Commander deck: the commander, `card_name`, and Wastes.
fn commander_deck(commander: &str, card_name: &str) -> Vec<Arc<CardDef>> {
    let mut d = vec![card(commander), card(card_name)];
    d.extend((0..98).map(|_| card("Wastes")));
    d
}

/// Whether `card_name` fits the color identity of `commander` in a Commander deck.
fn fits(commander: &str, card_name: &str) -> bool {
    !check_commander(&commander_deck(commander, card_name), &card(commander), &[], false)
        .iter()
        .any(|p| matches!(p, DeckProblem::OutsideColorIdentity { .. }))
}

#[test]
fn a_double_faced_cards_color_identity_combines_both_faces() {
    cr!("903.4", "903.4d", "903.5c");
    ruling!(
        "Needleverge Pathway // Pillarverge Pathway",
        "In the Commander variant, a double-faced card's color identity is determined by the mana costs and mana symbols in the rules text of both faces combined."
    );
    let pathway = "Needleverge Pathway // Pillarverge Pathway";
    supported(pathway);
    // {T}: Add {R}. // {T}: Add {W}.
    assert_eq!(
        computed_color_identity(&card(pathway)),
        ColorSet::single(Color::Red).union(ColorSet::single(Color::White))
    );
    assert!(!fits("Krenko, Mob Boss", pathway));
    assert!(fits("Aurelia, the Warleader", pathway));
}

#[test]
fn a_double_faced_cards_color_identity_combines_both_faces_curly_quotes() {
    cr!("903.4", "903.4d", "903.5c");
    ruling!(
        "Clearwater Pathway // Murkwater Pathway",
        "In the Commander variant, a double-faced card’s color identity is determined by the mana costs and mana symbols in the rules text of both faces combined."
    );
    let pathway = "Clearwater Pathway // Murkwater Pathway";
    supported(pathway);
    assert_eq!(
        computed_color_identity(&card(pathway)),
        ColorSet::single(Color::Blue).union(ColorSet::single(Color::Black))
    );
    assert!(!fits("Talrand, Sky Summoner", pathway));
    assert!(fits("Dragonlord Silumgar", pathway));
}
