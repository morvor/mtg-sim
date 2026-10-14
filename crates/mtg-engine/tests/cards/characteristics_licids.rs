//! The Licids: "{W}, {T}: ~ loses this ability and becomes an Aura enchantment with enchant
//! creature. Attach it to target creature. You may pay {W} to end this effect." The
//! creature becomes an Aura (CR 205.1a, 303.4) without the ability (CR 613.1f); its
//! controller may end the effect any time they have priority (a special action, CR
//! 116.2c), and then it's a creature again, which becomes unattached (CR 704.5p).

use crate::basic_effects_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn end_effect_actions(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s @ SpecialAction::Offer { .. }) => Some(s),
            _ => None,
        })
        .collect()
}

const LICIDS: [&str; 12] = [
    "Calming Licid",
    "Convulsing Licid",
    "Corrupting Licid",
    "Dominating Licid",
    "Enraging Licid",
    "Gliding Licid",
    "Leeching Licid",
    "Nurturing Licid",
    "Quickening Licid",
    "Stinging Licid",
    "Tempting Licid",
    "Transmogrifying Licid",
];

#[test]
fn licids_compile() {
    for name in LICIDS {
        assert_supported(name);
    }
}

#[test]
fn gliding_licid_becomes_an_aura_and_can_end_the_effect() {
    cr!("116.2c", "205.1a", "303.4a", "613.1f", "704.5p");
    ruling!(
        "Gliding Licid",
        "Paying the cost to end the effect is a special action"
    );
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Gliding Licid");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.activate(P0, licid, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve();
    let c = &t.obj_now(licid).chars;
    assert!(!c.is_creature());
    assert!(c.card_types.contains(CardType::Enchantment));
    assert!(c.has_subtype("Aura") && !c.has_subtype("Licid"));
    assert!(c.has_keyword(KeywordKind::Enchant));
    // It lost the activated ability, but keeps its other ability.
    assert!(!c
        .abilities
        .iter()
        .any(|a| matches!(a.kind, ability::AbilityKind::Activated(_))));
    assert_eq!(t.obj_now(licid).attached_to, Some(Entity::Object(bear)));
    assert!(t.obj_now(bear).chars.has_keyword(KeywordKind::Flying));
    // The opponent can't end the effect; its controller can, paying {U}.
    t.g.players[P1.idx()].mana_pool.add_type(ManaType::U, 1);
    assert!(end_effect_actions(&mut t, P1).is_empty());
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::U, 1);
    let sa = end_effect_actions(&mut t, P0)
        .pop()
        .expect("the action to end the effect");
    t.g.take_action(P0, Action::Special(sa));
    t.settle();
    let c = &t.obj_now(licid).chars;
    assert!(c.is_creature() && c.has_subtype("Licid") && !c.has_subtype("Aura"));
    assert!(c
        .abilities
        .iter()
        .any(|a| matches!(a.kind, ability::AbilityKind::Activated(_))));
    assert!(t.on_battlefield(licid));
    assert_eq!(t.obj_now(licid).attached_to, None);
    assert!(!t.obj_now(bear).chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(licid), (2, 2));
}

#[test]
fn transmogrifying_licid_stops_being_an_artifact_creature() {
    cr!("205.1a");
    ruling!(
        "Transmogrifying Licid",
        "ceases to be either a creature or an artifact"
    );
    ruling!(
        "Transmogrifying Licid",
        "it goes back to being an artifact creature with the creature type Licid"
    );
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Transmogrifying Licid");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, licid, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve();
    let c = &t.obj_now(licid).chars;
    assert!(!c.card_types.contains(CardType::Artifact) && !c.is_creature());
    // Enchanted creature gets +1/+1 and is an artifact in addition to its other types.
    assert_eq!(t.pt(bear), (3, 3));
    assert!(t.obj_now(bear).chars.card_types.contains(CardType::Artifact));
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::C, 1);
    let sa = end_effect_actions(&mut t, P0).pop().expect("end the effect");
    t.g.take_action(P0, Action::Special(sa));
    t.settle();
    let c = &t.obj_now(licid).chars;
    assert!(c.card_types.contains(CardType::Artifact) && c.is_creature());
    assert!(c.has_subtype("Licid"));
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn a_licid_on_a_creature_that_stops_being_one_goes_to_the_graveyard() {
    cr!("704.5m");
    ruling!(
        "Transmogrifying Licid",
        "the Licid will find itself enchanting something illegal"
    );
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Transmogrifying Licid");
    let land = t.battlefield(P1, "Forest");
    let genju = t.battlefield(P1, "Genju of the Cedars");
    t.g.attach(genju, Entity::Object(land));
    t.lands(P1, "Forest", 2);
    t.activate(P1, genju, 0, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(land).chars.is_creature());
    t.lands(P0, "Wastes", 1);
    t.activate(P0, licid, 0, &[Entity::Object(land)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(licid).attached_to, Some(Entity::Object(land)));
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(t.in_graveyard(P0, "Transmogrifying Licid"));
}
