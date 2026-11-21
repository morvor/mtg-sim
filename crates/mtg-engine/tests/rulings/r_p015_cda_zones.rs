//! Rulings batch P015 — characteristic-defining abilities that set power and/or toughness
//! work in every zone (CR 604.3, 208.2a), and the ones that count "you control" include
//! the object itself while it's on the battlefield.

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::*;

#[derive(Clone, Copy, Debug)]
enum Z {
    Hand,
    Gy,
    Exile,
    Bf,
}

struct Case {
    name: &'static str,
    zone: Z,
    setup: fn(&mut TestGame),
    pt: (i32, i32),
}

fn bears(t: &mut TestGame) {
    t.battlefield(P0, "Grizzly Bears");
}
fn thopter(t: &mut TestGame) {
    t.battlefield(P0, "Ornithopter");
}
fn two_forests(t: &mut TestGame) {
    t.lands(P0, "Forest", 2);
}
fn bolt_in_gy(t: &mut TestGame) {
    t.graveyard(P0, "Lightning Bolt");
}

/// Puts `c.name` (owned by P0) into its zone after the setup and returns its power and
/// toughness there.
fn run(c: &Case) -> (i32, i32) {
    supported(c.name);
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let id = match c.zone {
        Z::Hand => t.hand(P0, c.name),
        Z::Gy => t.graveyard(P0, c.name),
        Z::Exile => t.exile(P0, c.name),
        Z::Bf => t.battlefield(P0, c.name),
    };
    t.g.recompute();
    let ch = &t.obj_now(id).chars;
    (ch.power.unwrap_or(-99), ch.toughness.unwrap_or(-99))
}

const CASES: &[Case] = &[
    Case { name: "Sutured Ghoul", zone: Z::Hand, setup: |_| {}, pt: (0, 0) },
    Case { name: "Matca Rioters", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Forest"); t.battlefield(P0, "Island"); }, pt: (2, 2) },
    Case { name: "Slag Fiend", zone: Z::Hand, setup: |t| { t.graveyard(P0, "Ornithopter"); t.graveyard(P1, "Ornithopter"); }, pt: (2, 2) },
    Case { name: "Zendikar Incarnate", zone: Z::Hand, setup: |t| { t.lands(P0, "Forest", 3); }, pt: (3, 4) },
    Case { name: "Abominable Treefolk", zone: Z::Hand, setup: |t| { t.lands(P0, "Snow-Covered Forest", 2); }, pt: (2, 2) },
    Case { name: "Abomination of Llanowar", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Llanowar Elves"); t.graveyard(P0, "Llanowar Elves"); }, pt: (2, 2) },
    Case { name: "Adeline, Resplendent Cathar", zone: Z::Hand, setup: bears, pt: (1, 4) },
    Case { name: "Adeline, Resplendent Cathar", zone: Z::Bf, setup: bears, pt: (2, 4) },
    Case { name: "Apocalypse Demon", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Grizzly Bears"); t.graveyard(P0, "Grizzly Bears"); }, pt: (3, 3) },
    // Goblin Piker is a Warrior; Archpriest itself is a Cleric.
    Case { name: "Archpriest of Iona", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Goblin Piker"); }, pt: (1, 2) },
    Case { name: "Archpriest of Iona", zone: Z::Bf, setup: |t| { t.battlefield(P0, "Goblin Piker"); }, pt: (2, 2) },
    Case { name: "Avalanche of Sector 7", zone: Z::Hand, setup: |t| { t.battlefield(P1, "Ornithopter"); t.battlefield(P1, "Ornithopter"); }, pt: (2, 3) },
    // Itself (creature) and Lightning Bolt (instant).
    Case { name: "Barrowgoyf", zone: Z::Gy, setup: |t| { t.graveyard(P1, "Lightning Bolt"); }, pt: (2, 3) },
    Case { name: "Body of Knowledge", zone: Z::Hand, setup: |t| { t.hand(P0, "Forest"); t.hand(P0, "Forest"); }, pt: (3, 3) },
    Case { name: "Boneyard Mycodrax", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Grizzly Bears"); t.graveyard(P0, "Grizzly Bears"); }, pt: (2, 2) },
    Case { name: "Boneyard Wurm", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Grizzly Bears"); }, pt: (2, 2) },
    Case { name: "Braulios of Pheres Band", zone: Z::Hand, setup: two_forests, pt: (2, 2) },
    Case { name: "Brotherhood Vertibird", zone: Z::Hand, setup: thopter, pt: (1, 4) },
    Case { name: "Burrowguard Mentor", zone: Z::Hand, setup: bears, pt: (1, 1) },
    // Merfolk Looter ({1}{U}); Callaphe ({1}{U}{U}) adds two once it's a permanent.
    Case { name: "Callaphe, Beloved of the Sea", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Merfolk Looter"); }, pt: (1, 3) },
    Case { name: "Callaphe, Beloved of the Sea", zone: Z::Bf, setup: |t| { t.battlefield(P0, "Merfolk Looter"); }, pt: (3, 3) },
    Case { name: "Cephalopod Sentry", zone: Z::Hand, setup: thopter, pt: (1, 5) },
    Case { name: "Cephalopod Sentry", zone: Z::Bf, setup: thopter, pt: (2, 5) },
    Case { name: "Consuming Aberration", zone: Z::Hand, setup: |t| { t.graveyard(P1, "Grizzly Bears"); t.graveyard(P1, "Forest"); }, pt: (2, 2) },
    Case { name: "Consuming Blob", zone: Z::Gy, setup: bolt_in_gy, pt: (2, 3) },
    Case { name: "Cosmogoyf", zone: Z::Exile, setup: |t| { t.exile(P0, "Lightning Bolt"); t.exile(P1, "Lightning Bolt"); }, pt: (2, 3) },
    Case { name: "Cruel Somnophage // Can't Wake Up", zone: Z::Hand, setup: |t| { t.graveyard(P1, "Grizzly Bears"); }, pt: (1, 1) },
    Case { name: "Crusader of Odric", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Eluge, the Shoreless Sea", zone: Z::Hand, setup: |t| { t.lands(P0, "Island", 2); }, pt: (2, 2) },
    Case { name: "Enigma Drake", zone: Z::Hand, setup: |t| { t.graveyard(P0, "Lightning Bolt"); t.graveyard(P0, "Lightning Bolt"); }, pt: (2, 4) },
    Case { name: "Esgaroth Garrison", zone: Z::Hand, setup: bears, pt: (1, 5) },
    Case { name: "Esgaroth Garrison", zone: Z::Bf, setup: bears, pt: (2, 5) },
    Case { name: "Filigree Attendant", zone: Z::Hand, setup: thopter, pt: (1, 3) },
    Case { name: "Filigree Attendant", zone: Z::Bf, setup: thopter, pt: (2, 3) },
    Case { name: "Geist-Honored Monk", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Greensleeves, Maro-Sorcerer", zone: Z::Hand, setup: two_forests, pt: (2, 2) },
    Case { name: "Haughty Djinn", zone: Z::Gy, setup: bolt_in_gy, pt: (1, 4) },
    Case { name: "Ironroot Warlord", zone: Z::Hand, setup: bears, pt: (1, 5) },
    Case { name: "Kalonian Twingrove", zone: Z::Hand, setup: two_forests, pt: (2, 2) },
    Case { name: "Kinetic Augur", zone: Z::Hand, setup: bolt_in_gy, pt: (1, 4) },
    Case { name: "Lord of Extinction", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Grizzly Bears"); t.graveyard(P1, "Lightning Bolt"); }, pt: (3, 3) },
    Case { name: "Master of Etherium", zone: Z::Hand, setup: thopter, pt: (1, 1) },
    Case { name: "Mirkwood Pathmaker", zone: Z::Hand, setup: two_forests, pt: (2, 2) },
    Case { name: "Nighthawk Scavenger", zone: Z::Hand, setup: |t| { t.graveyard(P1, "Lightning Bolt"); }, pt: (2, 3) },
    Case { name: "Nightmare", zone: Z::Gy, setup: |t| { t.lands(P0, "Swamp", 2); }, pt: (2, 2) },
    Case { name: "Regal Bunnicorn", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Grizzly Bears"); t.battlefield(P0, "Ornithopter"); t.battlefield(P0, "Forest"); }, pt: (2, 2) },
    Case { name: "Renata, Called to the Hunt", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Llanowar Elves"); }, pt: (1, 3) },
    Case { name: "Revenant", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Grizzly Bears"); }, pt: (2, 2) },
    Case { name: "Robobrain War Mind", zone: Z::Hand, setup: |t| { t.hand(P0, "Forest"); }, pt: (2, 5) },
    Case { name: "Scourge of the Skyclaves", zone: Z::Hand, setup: |t| { t.g.players[0].life = 12; t.g.players[1].life = 15; }, pt: (5, 5) },
    Case { name: "Seraph of the Masses", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Serra Avatar", zone: Z::Gy, setup: |t| { t.g.players[0].life = 13; }, pt: (13, 13) },
    Case { name: "Shambling Suit", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Ornithopter"); t.battlefield(P0, "Glorious Anthem"); }, pt: (2, 3) },
    Case { name: "Silverwing Squadron", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Snow Villiers", zone: Z::Hand, setup: bears, pt: (1, 3) },
    Case { name: "Snow Villiers", zone: Z::Bf, setup: bears, pt: (2, 3) },
    // Itself and Forest are permanent cards; Lightning Bolt isn't.
    Case { name: "Souls of the Lost", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Forest"); t.graveyard(P0, "Lightning Bolt"); }, pt: (2, 3) },
    Case { name: "Spellheart Chimera", zone: Z::Hand, setup: bolt_in_gy, pt: (1, 3) },
    Case { name: "Squawkroaster", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Grizzly Bears"); t.battlefield(P0, "Goblin Piker"); }, pt: (2, 4) },
    Case { name: "Squelching Leeches", zone: Z::Hand, setup: |t| { t.lands(P0, "Swamp", 2); }, pt: (2, 2) },
    Case { name: "Suki, Kyoshi Warrior", zone: Z::Hand, setup: bears, pt: (1, 4) },
    Case { name: "Suki, Kyoshi Warrior", zone: Z::Bf, setup: bears, pt: (2, 4) },
    Case { name: "Syr Elenora, the Discerning", zone: Z::Hand, setup: |t| { t.hand(P0, "Forest"); }, pt: (2, 4) },
    Case { name: "Tarmogoyf", zone: Z::Gy, setup: bolt_in_gy, pt: (2, 3) },
    Case { name: "Terravore", zone: Z::Hand, setup: |t| { t.graveyard(P0, "Forest"); t.graveyard(P1, "Forest"); }, pt: (2, 2) },
    Case { name: "The Mycotyrant", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Thallid"); }, pt: (1, 1) },
    Case { name: "The Mycotyrant", zone: Z::Bf, setup: |t| { t.battlefield(P0, "Thallid"); }, pt: (2, 2) },
    Case { name: "Uurg, Spawn of Turg", zone: Z::Hand, setup: |t| { t.graveyard(P0, "Forest"); t.graveyard(P0, "Forest"); }, pt: (2, 5) },
    // Ornithopter is colorless; so is Vile Aggregate (devoid).
    Case { name: "Vile Aggregate", zone: Z::Hand, setup: thopter, pt: (1, 5) },
    Case { name: "Vile Aggregate", zone: Z::Bf, setup: thopter, pt: (2, 5) },
    Case { name: "Wayfaring Temple", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Freedom Fighter Recruit", zone: Z::Hand, setup: bears, pt: (1, 2) },
    Case { name: "Darksteel Juggernaut", zone: Z::Hand, setup: thopter, pt: (1, 1) },
    // Its own {B}{B}{B} plus Dark Ritual's {B}.
    Case { name: "Umbra Stalker", zone: Z::Gy, setup: |t| { t.graveyard(P0, "Dark Ritual"); }, pt: (4, 4) },
    Case { name: "Aven Trailblazer", zone: Z::Hand, setup: |t| { t.battlefield(P0, "Forest"); t.battlefield(P0, "Island"); }, pt: (2, 2) },
    Case { name: "Soulsurge Elemental", zone: Z::Hand, setup: bears, pt: (1, 1) },
    Case { name: "Soulsurge Elemental", zone: Z::Bf, setup: bears, pt: (2, 1) },
];

#[test]
fn power_and_toughness_defining_abilities_work_in_every_zone() {
    cr!("604.3", "208.2a", "613.4a");
    ruling!("Sutured Ghoul", "In zones other than the battlefield, Sutured Ghoul's power and toughness are each 0");
    ruling!("Matca Rioters", "Matca Rioters' ability is a \"characteristic-defining ability.\" It applies at all times in all zones");
    ruling!("Slag Fiend", "Slag Fiend’s power- and toughness-setting ability works in all zones, not just on the battlefield");
    ruling!("Zendikar Incarnate", "The ability defining Zendikar Incarnate’s power works in all zones, not just the battlefield");
    ruling!("Abominable Treefolk", "The ability that defines Abominable Treefolk's power and toughness works in all zones, not just the battlefield");
    ruling!("Abomination of Llanowar", "The ability that defines Abomination of Llanowar's power and toughness applies in all zones, not just the battlefield");
    ruling!("Adeline, Resplendent Cathar", "The ability that defines Adeline's power works in all zones, not just the battlefield");
    ruling!("Apocalypse Demon", "The ability that defines Apocalypse Demon's power and toughness works in all zones, not just the battlefield");
    ruling!("Archpriest of Iona", "The ability that defines Archpriest of Iona's power applies in all zones, not just the battlefield");
    ruling!("Avalanche of Sector 7", "The ability that defines Avalanche of Sector 7's power applies in all zones, not just the battlefield");
    ruling!("Barrowgoyf", "The ability that defines Barrowgoyf's power and toughness works in all zones, not just the battlefield");
    ruling!("Body of Knowledge", "The ability that defines Body of Knowledge's power and toughness applies in all zones, not just the battlefield");
    ruling!("Boneyard Mycodrax", "The ability that defines Boneyard Mycodrax’s power and toughness works in all zones, not just on the battlefield");
    ruling!("Boneyard Wurm", "The ability that defines Boneyard Wurm’s power and toughness works in all zones, not just the battlefield");
    ruling!("Braulios of Pheres Band", "The ability that defines Braulios's power and toughness works in all zones, not just the battlefield");
    ruling!("Brotherhood Vertibird", "The ability that defines Brotherhood Vertibird’s power works in all zones, not just the battlefield");
    ruling!("Burrowguard Mentor", "The ability that defines Burrowguard Mentor’s power and toughness functions in all zones, not just the battlefield");
    ruling!("Callaphe, Beloved of the Sea", "The ability that defines Callaphe’s power works in all zones, not just the battlefield");
    ruling!("Cephalopod Sentry", "The ability that defines Cephalopod Sentry's power works in all zones, not just the battlefield");
    ruling!("Consuming Aberration", "The ability that defines Consuming Aberration's power and toughness works in all zones, not just the battlefield");
    ruling!("Consuming Blob", "The ability that defines Consuming Blob's power and toughness works in all zones, not just the battlefield");
    ruling!("Cosmogoyf", "The ability that defines Cosmogoyf’s power and toughness works in all zones, not just the battlefield");
    ruling!("Cruel Somnophage // Can't Wake Up", "The ability that defines Cruel Somnophage's power and toughness functions in all zones, not just the battlefield");
    ruling!("Crusader of Odric", "The ability that defines Crusader of Odric's power and toughness works in all zones, not just the battlefield");
    ruling!("Eluge, the Shoreless Sea", "The ability that defines Eluge’s power and toughness functions in all zones, not just the battlefield");
    ruling!("Enigma Drake", "The ability that defines Enigma Drake's power works in all zones, not just the battlefield");
    ruling!("Esgaroth Garrison", "The ability that defines Esgaroth Garrison's power works in all zones, not just the battlefield");
    ruling!("Filigree Attendant", "The ability that defines Filigree Attendant's power works in all zones, not just the battlefield");
    ruling!("Geist-Honored Monk", "The ability that defines Geist-Honored Monk's power and toughness works in all zones, not just the battlefield");
    ruling!("Greensleeves, Maro-Sorcerer", "The ability that defines Greensleeves, Maro-Sorcerer's power and toughness works in all zones, not just on the battlefield");
    ruling!("Haughty Djinn", "The ability that defines Haughty Djinn's power functions in all zones");
    ruling!("Ironroot Warlord", "The ability that defines Ironroot Warlord’s power works in all zones, not just the battlefield");
    ruling!("Kalonian Twingrove", "The ability that defines Kalonian Twingrove's power and toughness works in all zones, not just the battlefield");
    ruling!("Kinetic Augur", "The ability that defines Kinetic Augur's power works in all zones, not just on the battlefield");
    ruling!("Lord of Extinction", "The ability that defines Lord of Extinction's power and toughness works in all zones, not just the battlefield");
    ruling!("Master of Etherium", "The ability that defines Master of Etherium's power and toughness applies in all zones, not just the battlefield");
    ruling!("Mirkwood Pathmaker", "The ability that defines Mirkwood Pathmaker's power and toughness works in all zones, not just the battlefield");
    ruling!("Nighthawk Scavenger", "The ability that defines Nighthawk Scavenger's power applies in all zones, not just the battlefield");
    ruling!("Nightmare", "The ability that defines Nightmare’s power and toughness works everywhere, not just on the battlefield");
    ruling!("Regal Bunnicorn", "The ability that defines Regal Bunnicorn's power and toughness functions in all zones, not just the battlefield");
    ruling!("Renata, Called to the Hunt", "The ability that defines Renata's power works in all zones, not just the battlefield");
    ruling!("Revenant", "The ability that defines Revenant's power and toughness applies in all zones, not just the battlefield");
    ruling!("Robobrain War Mind", "The ability that defines Robobrain War Mind's power functions in all zones, not just the battlefield");
    ruling!("Scourge of the Skyclaves", "The ability that defines Scourge of the Skyclaves's power and toughness applies in all zones, not just the battlefield");
    ruling!("Seraph of the Masses", "The ability that defines Seraph of the Masses's power and toughness works in all zones, not just the battlefield");
    ruling!("Serra Avatar", "The ability that defines Serra Avatar’s power and toughness works in all zones, not only the battlefield");
    ruling!("Shambling Suit", "The ability that defines Shambling Suit's power works in all zones, not just the battlefield");
    ruling!("Silverwing Squadron", "The ability that defines Silverwing Squadron's power and toughness works in all zones, not just the battlefield");
    ruling!("Snow Villiers", "The ability that defines Snow Villiers's power applies in all zones, not just the battlefield");
    ruling!("Souls of the Lost", "The ability that defines Souls of the Lost's power and toughness works in all zones");
    ruling!("Spellheart Chimera", "The ability that defines Spellheart Chimera's power functions in all zones, not just the battlefield");
    ruling!("Squawkroaster", "The ability that defines Squawkroaster's power applies in all zones, not just the battlefield");
    ruling!("Squelching Leeches", "The ability that defines Squelching Leeches's power and toughness works in all zones, not just the battlefield");
    ruling!("Suki, Kyoshi Warrior", "The ability that defines Suki, Kyoshi Warrior's power works in all zones, not just the battlefield");
    ruling!("Syr Elenora, the Discerning", "The ability that defines Syr Elenora's power works in all zones, not just the battlefield");
    ruling!("Tarmogoyf", "The ability that defines Tarmogoyf's power and toughness works in all zones, not just the battlefield");
    ruling!("Terravore", "The ability that defines Terravore’s power and toughness works in all zones, not only the battlefield");
    ruling!("The Mycotyrant", "The ability that defines The Mycotyrant's power and toughness works in all zones");
    ruling!("Uurg, Spawn of Turg", "The ability that defines Uurg, Spawn of Turg's power works in all zones, not just on the battlefield");
    ruling!("Vile Aggregate", "The ability that defines Vile Aggregate’s power works in all zones, not just the battlefield");
    ruling!("Wayfaring Temple", "The ability that defines Wayfaring Temple's power and toughness works in all zones, not just the battlefield");
    ruling!("Freedom Fighter Recruit", "The ability that sets Freedom Fighter Recruit's power functions in all zones, not only on the battlefield");
    ruling!("Darksteel Juggernaut", "The first ability works in all zones.");
    ruling!("Umbra Stalker", "This is a characteristic-defining ability, which means it functions in all zones");
    ruling!("Aven Trailblazer", "Aven Trailblazer's second ability is a \"characteristic-defining ability.\" It applies at all times in all zones");
    ruling!("Soulsurge Elemental", "Soulsurge Elemental’s power is continuously calculated, regardless of where it is. If it’s on the battlefield, it counts itself.");
    let mut failures = vec![];
    for c in CASES {
        let got = run(c);
        if got != c.pt {
            failures.push(format!("{} in {:?}: got {got:?}, want {:?}", c.name, c.zone, c.pt));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn a_crewed_vertibird_counts_itself() {
    cr!("604.3", "208.3");
    ruling!(
        "Brotherhood Vertibird",
        "As long as Brotherhood Vertibird is on the battlefield (and still an artifact), that ability will count Brotherhood Vertibird itself."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    let bird = t.battlefield(P0, "Brotherhood Vertibird");
    t.g.recompute();
    // Not a creature: no power or toughness.
    assert_eq!(t.obj_now(bird).chars.power, None);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, bird, 0, &[]).unwrap();
    t.resolve();
    // Ornithopter and itself.
    assert_eq!(t.pt(bird), (2, 4));
}
