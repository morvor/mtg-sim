//! Token copies and copy effects with exceptions (CR 707.9b, 707.9e, 306.5b, 205.1a):
//! "except it enters with an additional +1/+1 counter on it" (Littjara Mirrorlake),
//! "except it's 1/1 and has toxic 1" (Kinzu), "except it's not legendary and its starting
//! loyalty is 1" (Jace, Mirror Mage), "except it's a legendary Alien named Prisoner Zero"
//! (The Eleventh Hour), "except it's a Vehicle artifact with crew 3 and it loses all other
//! card types" (Imposter Mech), a token copy and a token with the card's power and
//! toughness (Soul Separator), a token copy entering attacking (Echoing Assault), a copy
//! until end of turn with exceptions (Cephalid Facetaker).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn token_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj_now(*o).is_token())
        .collect()
}

#[test]
fn littjara_mirrorlake_token_enters_with_an_additional_counter() {
    cr!("707.9e", "122.6");
    ruling!(
        "Littjara Mirrorlake",
        "The token copies exactly what is printed on the creature and nothing else"
    );
    supported("Littjara Mirrorlake");
    let mut t = TestGame::new(2);
    let lake = t.battlefield(P0, "Littjara Mirrorlake");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(obj(giant), "+1/+1", 2, None);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 3);
    t.activate(P0, lake, 1, &[obj(giant)]).unwrap();
    t.resolve_all();
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.counters(toks[0], "+1/+1"), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn kinzu_token_copy_is_1_1_with_toxic() {
    cr!("707.9b", "702.164a");
    ruling!(
        "Kinzu of the Bleak Coven",
        "If something becomes a copy of the token, the copy also has base power and toughness 1/1 and has toxic 1."
    );
    supported("Kinzu of the Bleak Coven");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kinzu of the Bleak Coven");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(giant).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Toxic));
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn jace_mirror_mage_token_is_not_legendary_with_starting_loyalty_1() {
    cr!("707.9b", "306.5b");
    ruling!(
        "Jace, Mirror Mage",
        "The token created by Jace's triggered ability doesn't have the legendary supertype and has a starting loyalty of 1 rather than 4."
    );
    supported("Jace, Mirror Mage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let j = t.hand(P0, "Jace, Mirror Mage");
    t.cast(P0, j).kicked(true).go();
    t.resolve_all();
    let jaces = t.named_on_battlefield("Jace, Mirror Mage");
    assert_eq!(jaces.len(), 2);
    let tok = jaces
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).is_token())
        .unwrap();
    assert_eq!(t.counters(tok, "loyalty"), 1);
    assert!(!t.obj_now(tok).chars.is_legendary());
    let card_jace = jaces.iter().copied().find(|o| *o != tok).unwrap();
    assert_eq!(t.counters(card_jace, "loyalty"), 4);
}

#[test]
fn the_eleventh_hour_makes_prisoner_zero() {
    cr!("707.9b", "714.2b");
    ruling!(
        "The Eleventh Hour",
        "The token will still have any card types it normally has, such as creature or artifact, but it will lose any non-Alien subtypes it has"
    );
    supported("The Eleventh Hour");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let saga = t.battlefield(P0, "The Eleventh Hour");
    t.g.add_counters(obj(saga), "lore", 2, None);
    t.answer_targets(P0, &[obj(bears)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let pz = token_named(&t, "Prisoner Zero");
    assert_eq!(pz.len(), 1);
    let c = &t.obj_now(pz[0]).chars;
    assert!(c.is_legendary());
    assert!(c.has_subtype("Alien") && !c.has_subtype("Bear"));
    assert!(c.is(CardType::Creature));
    assert_eq!(t.pt(pz[0]), (2, 2));
}

#[test]
fn imposter_mech_is_a_vehicle_with_crew_3() {
    cr!("707.9b", "205.1a", "702.122a");
    ruling!(
        "Imposter Mech",
        "Imposter Mech does not copy any subtypes related to a card type that it lost."
    );
    supported("Imposter Mech");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(theirs)]);
    let mech = t.enter(P0, "Imposter Mech");
    t.resolve_all();
    let c = &t.obj_now(mech).chars;
    assert_eq!(c.name, "Hill Giant");
    assert!(c.is(CardType::Artifact) && !c.is(CardType::Creature));
    assert!(c.has_subtype("Vehicle") && !c.has_subtype("Giant"));
    assert!(t.obj_now(mech).has_keyword(KeywordKind::Crew));
}

#[test]
fn soul_separator_makes_a_1_1_spirit_copy_and_a_zombie_with_its_power() {
    cr!("707.9b", "111.3");
    ruling!(
        "Soul Separator",
        "The power and toughness of the Zombie token are set as it\u{2019}s created based on the card as it last existed in your graveyard."
    );
    supported("Soul Separator");
    let mut t = TestGame::new(2);
    let sep = t.battlefield(P0, "Soul Separator");
    let dead = t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, sep, 0, &[obj(dead)]).unwrap();
    t.resolve_all();
    let spirit = token_named(&t, "Craw Wurm");
    assert_eq!(spirit.len(), 1);
    assert_eq!(t.pt(spirit[0]), (1, 1));
    assert!(t.obj_now(spirit[0]).chars.has_subtype("Spirit"));
    assert!(t.obj_now(spirit[0]).has_keyword(KeywordKind::Flying));
    let z = t.named_on_battlefield("Zombie Token");
    assert_eq!(z.len(), 1);
    assert_eq!(t.pt(z[0]), (6, 4));
}

#[test]
fn echoing_assault_token_copy_enters_attacking_that_player() {
    cr!("508.4", "707.9b");
    supported("Echoing Assault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoing Assault");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(t.obj_now(toks[0]).tapped);
    let combat = t.g.combat.as_ref().unwrap();
    assert_eq!(combat.attack_target(toks[0]), Some(Entity::Player(P1)));
}

#[test]
fn cephalid_facetaker_becomes_a_1_4_unblockable_copy_until_end_of_turn() {
    cr!("707.9b", "611.2a");
    supported("Cephalid Facetaker");
    let mut t = TestGame::new(2);
    let ceph = t.battlefield(P0, "Cephalid Facetaker");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wurm)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(ceph).chars.name, "Craw Wurm");
    assert_eq!(t.pt(ceph), (1, 4));
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(ceph).chars.name, "Cephalid Facetaker");
}
