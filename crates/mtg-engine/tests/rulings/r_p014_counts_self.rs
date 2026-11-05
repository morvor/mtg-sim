//! Rulings batch P014 — power/toughness-defining abilities (CR 604.3, 208.2a) that count
//! the object itself where it is, count each object once however many of the counted
//! qualities it has, count card types rather than cards, and count land types rather
//! than names. A compact table: each case is a card, a zone, a setup and the expected
//! power and toughness.

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::*;

#[derive(Clone, Copy, Debug)]
enum Z {
    Hand,
    Gy,
    Bf,
}

struct Case {
    name: &'static str,
    zone: Z,
    setup: fn(&mut TestGame),
    pt: (i32, i32),
}

fn none(_: &mut TestGame) {}

/// Puts `c.name` (owned by P0) into its zone after the setup and returns its power and
/// toughness there.
fn pt_of(c: &Case) -> (i32, i32) {
    supported(c.name);
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let id = match c.zone {
        Z::Hand => t.hand(P0, c.name),
        Z::Gy => t.graveyard(P0, c.name),
        Z::Bf => t.battlefield(P0, c.name),
    };
    t.g.recompute();
    let ch = &t.obj_now(id).chars;
    (ch.power.unwrap_or(-99), ch.toughness.unwrap_or(-99))
}

fn check(cases: &[Case]) {
    for c in cases {
        assert_eq!(pt_of(c), c.pt, "{} in {:?}", c.name, c.zone);
    }
}

#[test]
fn defined_pt_counts_the_object_itself() {
    cr!("604.3", "208.2a");
    ruling!(
        "Archpriest of Iona",
        "If Archpriest of Iona is on the battlefield, it will consider itself as a Cleric."
    );
    ruling!(
        "Abominable Treefolk",
        "As long as Abominable Treefolk is on the battlefield, its first ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Abomination of Llanowar",
        "As long as Abomination of Llanowar is on the battlefield (and still an Elf) or in your graveyard, its ability will count itself."
    );
    ruling!(
        "Body of Knowledge",
        "As long as Body of Knowledge is in your hand, its ability will count itself."
    );
    ruling!(
        "Crusader of Odric",
        "As long as Crusader of Odric is on the battlefield and still a creature, its ability will count itself."
    );
    ruling!(
        "Geist-Honored Monk",
        "As long as Geist-Honored Monk is on the battlefield, its second ability will count itself."
    );
    ruling!(
        "Ironroot Warlord",
        "As long as Ironroot Warlord is on the battlefield, its first ability will count itself, so its power will be at least 1."
    );
    ruling!(
        "Master of Etherium",
        "As long as Master of Etherium is on the battlefield, its ability will count itself."
    );
    ruling!(
        "Revenant",
        "As long as Revenant is in your graveyard, its ability will count itself."
    );
    ruling!(
        "Seraph of the Masses",
        "As long as Seraph of the Masses is on the battlefield, its ability will count itself."
    );
    ruling!(
        "Shambling Suit",
        "As long as Shambling Suit is on the battlefield, its ability will count itself, so it'll be at least 1/3."
    );
    ruling!(
        "Silverwing Squadron",
        "As long as Silverwing Squadron is on the battlefield, its ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Wayfaring Temple",
        "As long as Wayfaring Temple is on the battlefield, its first ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Battle Squadron",
        "Battle Squadron’s last ability applies in all zones. While it’s on the battlefield, it counts itself, so it’ll be at least 1/1."
    );
    ruling!(
        "Dauntless Dourbark",
        "Dauntless Dourbark counts itself when determining its power and toughness."
    );
    check(&[
        Case {
            name: "Archpriest of Iona",
            zone: Z::Bf,
            setup: none,
            pt: (1, 2),
        },
        Case {
            name: "Archpriest of Iona",
            zone: Z::Hand,
            setup: none,
            pt: (0, 2),
        },
        Case {
            name: "Abominable Treefolk",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Abominable Treefolk",
            zone: Z::Hand,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Abomination of Llanowar",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Abomination of Llanowar",
            zone: Z::Gy,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Abomination of Llanowar",
            zone: Z::Hand,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Body of Knowledge",
            zone: Z::Hand,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Body of Knowledge",
            zone: Z::Bf,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Crusader of Odric",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Crusader of Odric",
            zone: Z::Hand,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Geist-Honored Monk",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Ironroot Warlord",
            zone: Z::Bf,
            setup: none,
            pt: (1, 5),
        },
        Case {
            name: "Master of Etherium",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Master of Etherium",
            zone: Z::Hand,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Revenant",
            zone: Z::Gy,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Revenant",
            zone: Z::Hand,
            setup: none,
            pt: (0, 0),
        },
        Case {
            name: "Seraph of the Masses",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Shambling Suit",
            zone: Z::Bf,
            setup: none,
            pt: (1, 3),
        },
        Case {
            name: "Silverwing Squadron",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Wayfaring Temple",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Battle Squadron",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Battle Squadron",
            zone: Z::Hand,
            setup: |t| {
                t.battlefield(P0, "Grizzly Bears");
                t.battlefield(P0, "Grizzly Bears");
            },
            pt: (2, 2),
        },
        Case {
            name: "Battle Squadron",
            zone: Z::Gy,
            setup: |t| {
                t.battlefield(P0, "Grizzly Bears");
            },
            pt: (1, 1),
        },
        Case {
            name: "Dauntless Dourbark",
            zone: Z::Bf,
            setup: none,
            pt: (1, 1),
        },
        Case {
            name: "Dauntless Dourbark",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Forest");
            },
            pt: (2, 2),
        },
    ]);
}

#[test]
fn aetherwing_counts_itself_while_its_an_artifact() {
    cr!("604.3", "712.8e");
    ruling!(
        "Invasion of Kaladesh // Aetherwing, Golden-Scale Flagship",
        "As long as Aetherwing is still an artifact, its ability will count Aetherwing itself."
    );
    let name = "Invasion of Kaladesh // Aetherwing, Golden-Scale Flagship";
    supported(name);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    assert!(mtg_engine::dfc::transform(&mut t.g, c));
    t.g.recompute();
    assert_eq!(t.obj_now(c).chars.name, "Aetherwing, Golden-Scale Flagship");
    // Crewed (by Grizzly Bears, not an artifact) it's an artifact creature counting itself.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, c, &[bears]));
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.power, Some(1));
    t.battlefield(P0, "Sol Ring");
    t.g.recompute();
    assert_eq!(t.obj_now(c).chars.power, Some(2));
}

#[test]
fn pestilence_rats_counts_only_other_rats() {
    cr!("604.3");
    ruling!(
        "Pestilence Rats",
        "Does not count itself for determining power. It only counts other Rats."
    );
    check(&[
        Case {
            name: "Pestilence Rats",
            zone: Z::Bf,
            setup: none,
            pt: (0, 3),
        },
        // Other Rats on either side of the battlefield count.
        Case {
            name: "Pestilence Rats",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Pestilence Rats");
                t.battlefield(P1, "Pestilence Rats");
            },
            pt: (2, 3),
        },
    ]);
}

#[test]
fn an_object_with_several_counted_qualities_counts_once() {
    cr!("604.3", "702.73a");
    ruling!(
        "Shambling Suit",
        "A permanent that's both an artifact and an enchantment is only counted once."
    );
    ruling!(
        "Maraxus of Keld",
        "A permanent that's more than one of these types (such as an artifact creature) counts only once."
    );
    ruling!(
        "Aysen Crusader",
        "Aysen Crusader’s ability counts the number of permanents you control that are Soldiers and/or Warriors. A permanent with both creature types is counted only once."
    );
    ruling!(
        "The Mycotyrant",
        "If you control a creature that is both a Fungus and a Saproling, it counts only once for The Mycotyrant's middle ability."
    );
    ruling!(
        "Haughty Djinn",
        "A split card in your graveyard only counts once for Haughty Djinn's power, even if it's both an instant and a sorcery."
    );
    check(&[
        // Itself and Spear of Heliod (a legendary enchantment artifact, +1/+1 to creatures).
        Case {
            name: "Shambling Suit",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Spear of Heliod");
            },
            pt: (3, 4),
        },
        // Itself and an untapped Ornithopter (artifact creature); a tapped land doesn't count.
        Case {
            name: "Maraxus of Keld",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Ornithopter");
                let f = t.battlefield(P0, "Forest");
                t.g.tap(f);
            },
            pt: (2, 2),
        },
        // Woodland Changeling (every creature type) is one Soldier-and-Warrior.
        Case {
            name: "Aysen Crusader",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Woodland Changeling");
            },
            pt: (3, 3),
        },
        Case {
            name: "Aysen Crusader",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Woodland Changeling");
                t.battlefield(P0, "Goblin Piker");
            },
            pt: (4, 4),
        },
        // Itself (a Fungus) and Woodland Changeling (a Fungus and a Saproling).
        Case {
            name: "The Mycotyrant",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Woodland Changeling");
            },
            pt: (2, 2),
        },
        // Commit // Memory (an instant and a sorcery card) counts once.
        Case {
            name: "Haughty Djinn",
            zone: Z::Bf,
            setup: |t| {
                t.graveyard(P0, "Commit // Memory");
            },
            pt: (1, 4),
        },
        Case {
            name: "Haughty Djinn",
            zone: Z::Bf,
            setup: |t| {
                t.graveyard(P0, "Commit // Memory");
                t.graveyard(P0, "Lightning Bolt");
            },
            pt: (2, 4),
        },
    ]);
}

#[test]
fn a_forest_treefolk_counts_twice_for_dauntless_dourbark() {
    cr!("604.3", "205.3i");
    ruling!(
        "Dauntless Dourbark",
        "A permanent that's both a Forest and a Treefolk will be counted twice when determining Dauntless Dourbark's power and toughness."
    );
    // Dryad Arbor (a Forest creature) becomes a Treefolk too with Maskwood Nexus.
    supported("Maskwood Nexus");
    check(&[
        Case {
            name: "Dauntless Dourbark",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Dryad Arbor");
            },
            pt: (2, 2),
        },
        Case {
            name: "Dauntless Dourbark",
            zone: Z::Bf,
            setup: |t| {
                t.battlefield(P0, "Dryad Arbor");
                t.battlefield(P0, "Maskwood Nexus");
            },
            pt: (3, 3),
        },
    ]);
}

#[test]
fn card_types_are_counted_not_cards_or_other_types() {
    cr!("604.3", "205.2a", "205.4a", "205.3a");
    ruling!(
        "Consuming Blob",
        "Consuming Blob counts card types, not individual cards. If the only card in your graveyard is an artifact creature card, Consuming Blob will be 2/3."
    );
    ruling!(
        "Nighthawk Scavenger",
        "Card types that can appear on cards in a graveyard include artifact, creature, enchantment, instant, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Vampire and Rogue are subtypes, not card types."
    );
    check(&[
        Case {
            name: "Consuming Blob",
            zone: Z::Bf,
            setup: |t| {
                t.graveyard(P0, "Ornithopter");
            },
            pt: (2, 3),
        },
        Case {
            name: "Consuming Blob",
            zone: Z::Bf,
            setup: |t| {
                for _ in 0..10 {
                    t.graveyard(P0, "Ornithopter");
                }
            },
            pt: (2, 3),
        },
        // A legendary Vampire creature card and a basic snow land: two card types.
        Case {
            name: "Nighthawk Scavenger",
            zone: Z::Bf,
            setup: |t| {
                t.graveyard(P1, "Edgar Markov");
                t.graveyard(P1, "Snow-Covered Forest");
            },
            pt: (3, 3),
        },
        // Its own graveyard doesn't count.
        Case {
            name: "Nighthawk Scavenger",
            zone: Z::Bf,
            setup: |t| {
                t.graveyard(P0, "Edgar Markov");
            },
            pt: (1, 3),
        },
    ]);
}

#[test]
fn dungrove_elder_counts_forests_in_every_zone() {
    cr!("604.3", "205.3i", "613.4a");
    ruling!(
        "Dungrove Elder",
        "Dungrove Elder’s ability sets its power and toughness in all zones, not just the battlefield."
    );
    ruling!(
        "Dungrove Elder",
        "Dungrove Elder’s power and toughness are each equal to the number of lands you control with the land type Forest, not necessarily lands named Forest."
    );
    ruling!(
        "Dungrove Elder",
        "Dungrove Elder’s power and toughness will change as the number of Forests you control changes."
    );
    let forests = |t: &mut TestGame| {
        t.battlefield(P0, "Forest");
        t.battlefield(P0, "Temple Garden");
        t.battlefield(P1, "Forest");
    };
    check(&[
        Case {
            name: "Dungrove Elder",
            zone: Z::Hand,
            setup: forests,
            pt: (2, 2),
        },
        Case {
            name: "Dungrove Elder",
            zone: Z::Gy,
            setup: forests,
            pt: (2, 2),
        },
        Case {
            name: "Dungrove Elder",
            zone: Z::Bf,
            setup: forests,
            pt: (2, 2),
        },
    ]);
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Dungrove Elder");
    let f = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(e), (1, 1));
    t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(e), (2, 2));
    t.g.destroy(f, None);
    t.g.recompute();
    assert_eq!(t.pt(e), (1, 1));
}
