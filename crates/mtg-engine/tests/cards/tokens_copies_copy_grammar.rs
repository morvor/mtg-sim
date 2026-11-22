//! Copy grammar: token copies with exceptions ("except it's a 1/1 white Spirit creature
//! with flying in addition to its other types", "except its name is ~'s Warform", "except
//! it's an enchantment and loses all other card types"), "becomes a copy of ..., except
//! ..." (`r707_becomes_copy.rs`, `tokens_copies_copy.rs`) and copies of cards
//! (`copy_exiled_card.rs`).

use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn copy_wordings_compile() {
    assert_compiles(&[
        // Token copies with exceptions.
        "Kaya, Intangible Slayer",
        "Will of the Temur",
        "Mishra, Eminent One",
        "Anikthea, Hand of Erebos",
        "Myrkul, Lord of Bones",
        "Adagia, Windswept Bastion",
        "Inalla, Archmage Ritualist",
        "Cleaver Skaab",
        "Dino DNA",
        "The Cloning of Shredder",
        // Becoming a copy.
        "Deepfathom Echo",
        "Absorbing Man",
        "Taskmaster, Mercenary Mimic",
        "Hulkling, Young Avenger",
        "The Ever-Changing 'Dane",
        "Shuri, Wakandan Inventor",
        "Absorb Identity",
        "Lazav, Wearer of Faces",
        // Copies of cards.
        "Mnemonic Deluge",
        "Spelltwine",
        "Zethi, Arcane Blademaster",
    ]);
}

fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj_now(*o).kind == ObjKind::Token)
        .collect()
}

fn activate_containing(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let s = t.g.current(source);
    let uid = t
        .g
        .obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, s, uid).expect("activation failed");
    t.g.flush_events();
}

#[test]
fn kaya_copy_is_a_1_1_white_spirit_with_flying_in_addition_to_its_types() {
    cr!("707.9b", "707.2", "111.2");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Intangible Slayer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, kaya, "Exile target");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let tok = tokens_named(&t, "Grizzly Bears");
    assert_eq!(tok.len(), 1, "{}", t.dump_log());
    let o = t.obj_now(tok[0]);
    assert_eq!(o.controller, P0);
    assert_eq!(t.pt(tok[0]), (1, 1));
    assert!(o.chars.has_subtype("Spirit") && o.chars.has_subtype("Bear"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn will_of_the_temur_copy_is_a_4_4_flying_dragon_creature() {
    cr!("707.9b", "707.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let ring = t.battlefield(P1, "Sol Ring");
    let w = t.hand(P0, "Will of the Temur");
    t.cast(P0, w).modes(&[0]).target(ring).go();
    t.resolve_all();
    let tok = tokens_named(&t, "Sol Ring");
    assert_eq!(tok.len(), 1, "{}", t.dump_log());
    let c = &t.obj_now(tok[0]).chars;
    assert!(c.is(CardType::Artifact) && c.is(CardType::Creature));
    assert!(c.has_subtype("Dragon"));
    assert_eq!(t.pt(tok[0]), (4, 4));
}

#[test]
fn mishra_warform_is_a_named_4_4_construct_copy_of_the_artifact() {
    cr!("707.9b", "707.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mishra, Eminent One");
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let w = tokens_named(&t, "Mishra's Warform");
    assert_eq!(w.len(), 1, "{}", t.dump_log());
    let c = &t.obj_now(w[0]).chars;
    assert!(c.is(CardType::Artifact) && c.is(CardType::Creature) && c.has_subtype("Construct"));
    assert_eq!(t.pt(w[0]), (4, 4));
}

#[test]
fn myrkul_copy_is_an_enchantment_and_no_other_card_type() {
    cr!("707.9b", "205.1a");
    ruling!(
        "Myrkul, Lord of Bones",
        "The last ability creates a copy of the card as it last existed in the graveyard"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Myrkul, Lord of Bones");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    let tok = tokens_named(&t, "Grizzly Bears");
    assert_eq!(tok.len(), 1);
    let c = &t.obj_now(tok[0]).chars;
    assert!(c.is(CardType::Enchantment) && !c.is(CardType::Creature));
}

#[test]
fn cleaver_skaab_creates_two_copies_of_the_sacrificed_zombie() {
    cr!("707.2", "608.2h");
    ruling!(
        "Cleaver Skaab",
        "You can choose to sacrifice a Zombie token to activate the ability."
    );
    let mut t = TestGame::new(2);
    let skaab = t.battlefield(P0, "Cleaver Skaab");
    // A Zombie token (a token copy of Walking Corpse) pays the sacrifice cost.
    let tc = mtg_engine::replacement::TokenCreate {
        chars: card("Walking Corpse").characteristics(mtg_engine::object::FaceState::Front),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    let corpse = t.g.create_tokens(P0, tc, 1, None)[0];
    assert_eq!(tokens_named(&t, "Walking Corpse"), vec![corpse]);
    t.lands(P0, "Island", 3);
    t.answer_choose(P0, &[Entity::Object(corpse)]);
    activate_containing(&mut t, P0, skaab, "Sacrifice another Zombie");
    t.resolve_all();
    let now = tokens_named(&t, "Walking Corpse");
    assert_eq!(now.len(), 2, "{}", t.dump_log());
    assert!(!now.contains(&corpse));
}

#[test]
fn inalla_copies_the_wizard_that_entered() {
    cr!("707.2", "111.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inalla, Archmage Ritualist");
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    t.enter(P0, "Prodigal Sorcerer");
    t.resolve_all();
    let tok = tokens_named(&t, "Prodigal Sorcerer");
    assert_eq!(tok.len(), 1, "{}", t.dump_log());
    assert!(t
        .obj_now(tok[0])
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn faerie_artisans_keeps_only_its_newest_token() {
    cr!("707.9b", "607.1d");
    ruling!(
        "Faerie Artisans",
        "The token is an artifact in addition to its other types."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Artisans");
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    let bears = tokens_named(&t, "Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t.obj_now(bears[0]).chars.is(CardType::Artifact));
    t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert!(tokens_named(&t, "Grizzly Bears").is_empty(), "{}", t.dump_log());
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
}

#[test]
fn hulkling_becomes_a_copy_keeping_his_name_size_flying_and_ability() {
    cr!("707.9a", "707.9b", "613.2a");
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Hulkling, Young Avenger");
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Island", 1);
    let op = t.hand(P0, "Opt");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.cast(P0, op).go();
    t.resolve_all();
    let c = &t.obj_now(hulk).chars;
    assert_eq!(c.name.as_str(), "Hulkling, Young Avenger");
    assert!(c.has_subtype("Angel"));
    assert!(c.has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    assert!(c.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.pt(hulk), (4, 4));
}

#[test]
fn the_ever_changing_dane_becomes_a_copy_of_the_sacrificed_creature() {
    cr!("707.9a", "613.2a");
    ruling!(
        "The Ever-Changing 'Dane",
        "The copy effect lasts indefinitely."
    );
    let mut t = TestGame::new(2);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, dane, "Sacrifice another creature");
    t.resolve_all();
    let c = &t.obj_now(dane).chars;
    assert_eq!(c.name.as_str(), "Hill Giant", "{}", t.dump_log());
    assert_eq!(t.pt(dane), (3, 3));
    // It keeps the ability.
    assert!(c
        .abilities
        .iter()
        .any(|a| a.text.contains("Sacrifice another creature")));
}

#[test]
fn shuri_makes_an_artifact_a_nonlegendary_copy_of_another_until_end_of_turn() {
    cr!("707.9b", "613.2a");
    let mut t = TestGame::new(2);
    let shuri = t.battlefield(P0, "Shuri, Wakandan Inventor");
    let ring = t.battlefield(P0, "Sol Ring");
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    t.lands(P0, "Island", 1);
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.answer_targets(P0, &[Entity::Object(jitte)]);
    activate_containing(&mut t, P0, shuri, "becomes a copy");
    t.resolve_all();
    let c = &t.obj_now(ring).chars;
    assert_eq!(c.name.as_str(), "Umezawa's Jitte", "{}", t.dump_log());
    assert!(!c.is_legendary());
    assert_eq!(t.named_on_battlefield("Umezawa's Jitte").len(), 2);
}

#[test]
fn absorb_identity_turns_your_shapeshifters_into_copies() {
    cr!("707.2", "608.2h");
    ruling!(
        "Absorb Identity",
        "Either all Shapeshifters you control will become copies of the creature or none of them will."
    );
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Changeling Outcast");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let ai = t.hand(P0, "Absorb Identity");
    t.answer_yes(P0, true);
    t.cast(P0, ai).target(giant).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Hill Giant", "{}", t.dump_log());
}

#[test]
fn mnemonic_deluge_casts_three_copies_of_the_exiled_card() {
    cr!("707.12", "707.12a");
    ruling!(
        "Mnemonic Deluge",
        "You create and cast the copies all during the resolution of Mnemonic Deluge."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let md = t.hand(P0, "Mnemonic Deluge");
    t.cast(P0, md).target(bolt).go();
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 11, "{}", t.dump_log());
    assert!(t.in_exile("Lightning Bolt"));
}
