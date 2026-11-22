//! Rulings batch S29 — Donatello, Mutant Mechanic: "{T}: Put three +1/+1 counters on target
//! artifact you control. If it isn't a creature, it becomes a 0/0 Robot creature in
//! addition to its other types. Activate only as a sorcery." The effect sets its base
//! power and toughness (layer 7b, CR 613.4b) with a later timestamp than a station
//! striation's or a Vehicle's printed power and toughness (CR 613.7), adds types without
//! removing any (CR 205.1b), and doesn't remove abilities.

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s16_common::add_charge;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Donatello targets `artifact`, and the ability resolves.
fn donatello(t: &mut TestGame, artifact: ObjectId) {
    supported("Donatello, Mutant Mechanic");
    let d = t.battlefield(P0, "Donatello, Mutant Mechanic");
    t.answer_targets(P0, &[Entity::Object(artifact)]);
    activate_containing(t, P0, d, "Robot").expect("Donatello's ability");
    t.resolve_all();
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.is(CardType::Creature)
}

#[test]
fn a_station_artifact_stays_0_0_as_charge_counters_are_added() {
    cr!("613.4b", "613.7", "721.2a", "721.2b");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact has station, its base power and toughness will be set to 0/0. Adding charge counters to that artifact won't restore the power and toughness printed in any of its striations."
    );
    supported("Atmospheric Greenhouse");
    // Atmospheric Greenhouse: "Station. 8+ | Flying, trample" with a 5/4 box.
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Atmospheric Greenhouse");
    donatello(&mut t, ship);
    assert!(is_creature(&t, ship));
    assert!(t.obj_now(ship).chars.has_subtype("Robot"));
    assert_eq!(t.pt(ship), (3, 3));
    add_charge(&mut t, ship, 8);
    // The striation's abilities apply, but not its 5/4.
    assert!(t.obj_now(ship).chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(ship), (3, 3));
    // An artifact that's already a creature (a charged Greenhouse, 5/4) just gets the
    // counters.
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Atmospheric Greenhouse");
    add_charge(&mut t, ship, 8);
    donatello(&mut t, ship);
    assert!(!t.obj_now(ship).chars.has_subtype("Robot"));
    assert_eq!(t.pt(ship), (8, 7));
}

#[test]
fn a_vehicle_stays_0_0_when_crewed() {
    cr!("613.4b", "205.1b", "702.122a");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact is a Vehicle, its base power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    ruling!(
        "Donatello, Mutant Mechanic",
        "Donatello's first ability doesn't remove any abilities the target artifact has."
    );
    ruling!(
        "Donatello, Mutant Mechanic",
        "The artifact retains any types, subtypes, or supertypes it has."
    );
    supported("Aradara Express");
    // Aradara Express: an 8/6 Vehicle with menace and crew 4.
    let mut t = TestGame::new(2);
    let express = t.battlefield(P0, "Aradara Express");
    donatello(&mut t, express);
    let o = t.obj_now(express);
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.has_subtype("Vehicle") && o.chars.has_subtype("Robot"));
    assert!(o.chars.has_keyword(KeywordKind::Menace));
    assert!(o.chars.has_keyword(KeywordKind::Crew));
    assert_eq!(t.pt(express), (3, 3));
    // Crewing it (Hill Giant and Grizzly Bears, total power 5) doesn't restore its 8/6.
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, express, &[giant, bears]));
    t.resolve_all();
    assert_eq!(t.pt(express), (3, 3));
}

#[test]
fn an_attached_equipment_becomes_unattached_and_cant_be_attached() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact is an attached Equipment, it becomes unattached. If an Equipment without reconfigure becomes an artifact creature, it can't be attached to another creature."
    );
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    donatello(&mut t, splitter);
    assert!(is_creature(&t, splitter));
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
    // Its equip ability can't attach it.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let _ = activate_containing(&mut t, P0, splitter, "Equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn an_if_its_sentence_checks_the_target_as_it_resolves() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Topple the Statue",
        "If Topple the Statue targets an artifact, that artifact will be tapped if it's untapped, and then it'll be destroyed."
    );
    ruling!(
        "Topple the Statue",
        "If the target is legal but isn't tapped, isn't destroyed, or is neither tapped nor destroyed, you do draw a card."
    );
    supported("Topple the Statue");
    // Sentences like "If it's a land card, you may put it onto the battlefield" are still
    // understood whole by their patterns.
    supported("Explorer's Scope");
    supported("Into the Wilds");
    // "Tap target permanent. If it's an artifact, destroy it. Draw a card."
    let mut t = TestGame::new(2);
    let mind_stone = t.battlefield(P1, "Mind Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    crate::r_s29_common::cast_and_resolve(
        &mut t,
        P0,
        "Topple the Statue",
        &[Entity::Object(mind_stone)],
    );
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert_eq!(t.hand_size(P0), hand + 1);
    crate::r_s29_common::cast_and_resolve(
        &mut t,
        P0,
        "Topple the Statue",
        &[Entity::Object(bears)],
    );
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn an_if_its_sentence_checks_the_returned_permanent_on_the_battlefield() {
    cr!("608.2c", "400.7", "613.1d");
    ruling!(
        "Defy Death",
        "If the creature is an Angel only on the battlefield (perhaps because it’s a non-Angel creature card and Xenograft is on the battlefield), it will get the +1/+1 counters."
    );
    ruling!(
        "Return Upon the Tide",
        "You check if the creature is an Elf once it’s on the battlefield. You’ll create tokens if it is, even if the card in the graveyard wasn’t an Elf card."
    );
    ruling!(
        "Essence Flux",
        "Essence Flux checks whether the creature is a Spirit after it has returned from exile."
    );
    supported("Xenograft");
    // Xenograft (P0): "Each creature you control is the chosen type in addition to its
    // other types."
    let with_xenograft = |kind: &str| {
        let mut t = TestGame::new(2);
        crate::r_s24_common::choose_creature_type(&mut t, P0, kind);
        t.enter(P0, "Xenograft");
        t.resolve_all();
        t
    };
    // Defy Death: "Return target creature card from your graveyard to the battlefield. If
    // it's an Angel, put two +1/+1 counters on it." A Grizzly Bears card: two counters
    // with Xenograft naming Angel, none without.
    supported("Defy Death");
    for (angel, expected) in [(true, 2), (false, 0)] {
        let mut t = if angel {
            with_xenograft("Angel")
        } else {
            TestGame::new(2)
        };
        let card = t.graveyard(P0, "Grizzly Bears");
        crate::r_s29_common::cast_and_resolve(&mut t, P0, "Defy Death", &[Entity::Object(card)]);
        let bears = t.named_on_battlefield("Grizzly Bears")[0];
        assert_eq!(t.counters(bears, counters::PLUS1), expected, "angel: {angel}");
    }
    // Return Upon the Tide: "... If it's an Elf, create two 1/1 green Elf Warrior creature
    // tokens."
    supported("Return Upon the Tide");
    let mut t = with_xenograft("Elf");
    let card = t.graveyard(P0, "Grizzly Bears");
    crate::r_s29_common::cast_and_resolve(
        &mut t,
        P0,
        "Return Upon the Tide",
        &[Entity::Object(card)],
    );
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
    // Essence Flux: "Exile target creature you control, then return that card to the
    // battlefield under its owner's control. If it's a Spirit, put a +1/+1 counter on it."
    supported("Essence Flux");
    let mut t = with_xenograft("Spirit");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Essence Flux", &[Entity::Object(bears)]);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn cemetery_recruitment_draws_for_a_zombie_card_only_if_it_resolves() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Cemetery Recruitment",
        "If the target creature card is an illegal target as Cemetery Recruitment tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card even if the target creature card was a Zombie card."
    );
    supported("Cemetery Recruitment");
    supported("Gravedigger");
    // "Return target creature card from your graveyard to your hand. If it's a Zombie
    // card, draw a card." Gravedigger is a Zombie; Grizzly Bears isn't.
    for (name, drawn) in [("Gravedigger", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P0, name);
        let hand = t.hand_size(P0);
        crate::r_s29_common::cast_and_resolve(
            &mut t,
            P0,
            "Cemetery Recruitment",
            &[Entity::Object(card)],
        );
        assert!(t.in_hand(P0, name));
        assert_eq!(t.hand_size(P0), hand + 1 + drawn, "{name}");
    }
    // The Gravedigger card leaves the graveyard before it resolves: no draw.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Gravedigger");
    crate::r_s25_common::cast_new(&mut t, P0, "Cemetery Recruitment", &[Entity::Object(card)]);
    t.g.exile_object(card, None);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Cemetery Recruitment"));
}
