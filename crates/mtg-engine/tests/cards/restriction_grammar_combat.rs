//! Combat restrictions and requirements compiled by the restriction grammar (CR 506.5,
//! 508.1c–d, 509.1b–c): "can only attack alone", "No more than N creatures can attack
//! you each combat", "No more than one creature can attack ~ each combat", "must be
//! blocked by two or more creatures / exactly one creature / an Eldrazi if able", "All
//! Walls able to block ~ do so", "can block an additional seven creatures each combat",
//! "can't be blocked except by six or more creatures", "can't block or be blocked by
//! non-Spirit creatures", "[players] can't block with [creatures]", subjects named,
//! goaded or listed, and requirements of resolving effects with players and objects.

use mtg_engine::combat::{
    attack_declaration_legal, attack_options, block_declaration_legal, block_options,
};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Declares `attackers` for the active player and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.advance_to(ap, Step::DeclareAttackers);
}

#[test]
fn can_only_attack_alone() {
    cr!("506.5", "508.1c");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Cruelties");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p1 = Entity::Player(P1);
    assert!(attack_declaration_legal(&t.g, &opts, &[(master, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p1)]));
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(master, p1), (bears, p1)]
    ));
}

#[test]
fn no_more_than_n_creatures_can_attack_you() {
    cr!("508.1c");
    compiles("Crawlspace");
    compiles("Judoon Enforcers");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Crawlspace");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let (p1, p2) = (Entity::Player(P1), Entity::Player(P2));
    assert!(attack_declaration_legal(&t.g, &opts, &[(a, p1), (b, p1)]));
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(a, p1), (b, p1), (c, p1)]
    ));
    // The limit is on attacking the Crawlspace's controller only.
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(a, p1), (b, p1), (c, p2)]
    ));

    // Planeswalkers its controller controls can be attacked by any number of creatures.
    ruling!("Crawlspace", "attack planeswalkers you control with any number");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Crawlspace");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let (p1, w) = (Entity::Player(P1), Entity::Object(walker));
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(a, p1), (b, p1), (c, w)]
    ));
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(a, w), (b, w), (c, w)]
    ));
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(a, p1), (b, p1), (c, p1)]
    ));
}

#[test]
fn each_opponent_cant_block_with_more_than_one_creature_this_combat() {
    cr!("509.1b", "508.1c");
    compiles("Mirri, Weatherlight Duelist");
    let mut t = TestGame::new(2);
    let mirri = t.battlefield(P0, "Mirri, Weatherlight Duelist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(mirri, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(x, bears)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(y, mirri)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, bears), (y, mirri)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, mirri), (y, mirri)]));

    // Without the trigger, both may block.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirri, Weatherlight Duelist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(x, bears), (y, bears)]));

    // "As long as Mirri is tapped, no more than one creature can attack you each combat."
    let mut t = TestGame::new(2);
    let mirri = t.battlefield(P0, "Mirri, Weatherlight Duelist");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let p0 = Entity::Player(P0);
    assert!(attack_declaration_legal(&t.g, &opts, &[(a, p0), (b, p0)]));
    t.g.tap(mirri);
    t.g.recompute();
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[(a, p0)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(a, p0), (b, p0)]));
}

#[test]
fn no_more_than_one_creature_can_attack_this_planeswalker() {
    cr!("508.1c");
    let mut t = TestGame::new(2);
    // Tomik, Orzhov Lawmage: "Planeswalkers you control have "No more than one creature
    // can attack ~ each combat."" compiles the same predicate; test it directly.
    let walker = t.battlefield(P1, "The Eternal Wanderer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let w = Entity::Object(walker);
    let p1 = Entity::Player(P1);
    assert!(attack_declaration_legal(&t.g, &opts, &[(a, w), (b, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(a, w), (b, w)]));
}

#[test]
fn must_be_blocked_by_two_or_more_and_by_exactly_one() {
    cr!("509.1c");
    compiles("Gorm the Great");
    let mut t = TestGame::new(2);
    let gorm = t.battlefield(P0, "Gorm the Great");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(gorm, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(x, gorm), (y, gorm)]));
    // One blocker obeys "must be blocked" but not "by two or more": fewer requirements.
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, gorm)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));

    // Nacatl War-Pride: its attack trigger creates a copy for each creature the
    // defending player controls; each must be blocked by exactly one creature if able.
    let mut t = TestGame::new(2);
    let pride = t.battlefield(P0, "Nacatl War-Pride");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pride, Entity::Player(P1))]);
    t.resolve_all();
    let copies: Vec<ObjectId> = t
        .g
        .attackers()
        .into_iter()
        .filter(|a| *a != pride)
        .collect();
    assert_eq!(copies.len(), 2);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(
        &t.g,
        &opts,
        &[(x, pride), (y, copies[0])]
    ));
    // Two blockers on one of them obeys fewer requirements.
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, pride), (y, pride)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(x, pride)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn must_be_blocked_by_an_eldrazi_and_all_walls_able_to_block_do_so() {
    cr!("509.1c");
    compiles("Slayer's Cleaver");
    compiles("Marble Priest");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Marble Priest");
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(priest, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(wall, priest)]));
    assert!(block_declaration_legal(
        &t.g,
        &opts,
        &[(wall, priest), (bears, priest)]
    ));
    // The Wall must block it; the Bears needn't.
    assert!(!block_declaration_legal(&t.g, &opts, &[(bears, priest)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));

    let mut t = TestGame::new(2);
    let cleaver = t.battlefield(P0, "Slayer's Cleaver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(cleaver, Entity::Object(bears));
    let eldrazi = t.battlefield(P1, "Eldrazi Devastator");
    let other = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(eldrazi, bears)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(other, bears)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn blocks_additional_creatures_and_min_blockers() {
    cr!("509.1b", "509.1a");
    compiles("Watcher in the Web");
    compiles("Hexmark Destroyer");
    let mut t = TestGame::new(2);
    let attackers: Vec<ObjectId> = (0..9)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    let watcher = t.battlefield(P1, "Watcher in the Web");
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    attack_with(&mut t, &decl);
    let opts = block_options(&t.g, &[P1]);
    let all: Vec<(ObjectId, ObjectId)> = attackers.iter().map(|a| (watcher, *a)).collect();
    // One creature plus an additional seven: eight, not nine.
    assert!(!block_declaration_legal(&t.g, &opts, &all));
    assert!(block_declaration_legal(&t.g, &opts, &all[..8]));
    assert!(block_declaration_legal(&t.g, &opts, &all[..7]));

    let mut t = TestGame::new(2);
    let destroyer = t.battlefield(P0, "Hexmark Destroyer");
    let blockers: Vec<ObjectId> = (0..6)
        .map(|_| t.battlefield(P1, "Grizzly Bears"))
        .collect();
    attack_with(&mut t, &[(destroyer, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    let six: Vec<(ObjectId, ObjectId)> = blockers.iter().map(|b| (*b, destroyer)).collect();
    assert!(block_declaration_legal(&t.g, &opts, &six));
    assert!(!block_declaration_legal(&t.g, &opts, &six[..5]));
}

#[test]
fn cant_block_or_be_blocked_by_non_spirit_creatures() {
    cr!("509.1b");
    compiles("Lost in the Spirit World");
    compiles("Sneaky Homunculus");
    let mut t = TestGame::new(2);
    let lost = t.hand(P0, "Lost in the Spirit World");
    t.lands(P0, "Island", 3);
    t.cast(P0, lost).go();
    t.resolve();
    let spirit = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).is_token())
        .unwrap();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ghost = t.battlefield(P1, "Spectral Sailor");
    t.g.objects[spirit.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(spirit, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(bears, spirit)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(ghost, spirit)]));

    // Sneaky Homunculus can't block creatures with power 2 or greater.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let homunculus = t.battlefield(P1, "Sneaky Homunculus");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (elf, Entity::Player(P1))],
    );
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(homunculus, bears)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(homunculus, elf)]));
}

#[test]
fn listed_named_and_goaded_subjects() {
    cr!("509.1b", "508.1c", "701.15a");
    compiles("Magistrate's Veto");
    compiles("Rite of the Raging Storm");
    compiles("Bothersome Quasit");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magistrate's Veto");
    let attacker = t.battlefield(P0, "Hill Giant");
    let white = t.battlefield(P1, "Savannah Lions");
    let blue = t.battlefield(P1, "Coral Merfolk");
    let green = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(white, attacker)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(blue, attacker)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(green, attacker)]));

    // "Creatures named Lightning Rager can't attack you or planeswalkers you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rite of the Raging Storm");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let rager = t.named_on_battlefield("Lightning Rager");
    assert_eq!(rager.len(), 1);
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(rager[0], Entity::Player(P0))]
    ));

    // "Goaded creatures your opponents control can't block."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bothersome Quasit");
    let attacker = t.battlefield(P0, "Hill Giant");
    let goaded = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[goaded.0 as usize].goaded_by.push(P0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(goaded, attacker)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(other, attacker)]));
}

#[test]
fn requirements_with_a_player_or_an_object() {
    cr!("508.1d", "509.1c");
    compiles("Dulcet Sirens");
    compiles("Alluring Siren");
    compiles("Hunt Down");
    // "Target creature attacks target opponent this turn if able."
    let mut t = TestGame::new(3);
    let sirens = t.battlefield(P0, "Dulcet Sirens");
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    t.activate(
        P0,
        sirens,
        0,
        &[Entity::Object(bears), Entity::Player(P2)],
    )
    .unwrap();
    t.resolve();
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(bears, Entity::Player(P2))]
    ));
    assert!(!attack_declaration_legal(
        &t.g,
        &opts,
        &[(bears, Entity::Player(P0))]
    ));
    assert!(!attack_declaration_legal(&t.g, &opts, &[]));

    // "Target creature blocks target creature this turn if able."
    let mut t = TestGame::new(2);
    let hunt = t.hand(P0, "Hunt Down");
    t.lands(P0, "Forest", 1);
    let attacker = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.cast(P0, hunt).target(blocker).target(attacker).go();
    t.resolve();
    attack_with(
        &mut t,
        &[
            (attacker, Entity::Player(P1)),
            (other, Entity::Player(P1)),
        ],
    );
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(blocker, attacker)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(blocker, other)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));
}

#[test]
fn opponents_cant_block_with_creatures_with_even_mana_values() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Void Winnower");
    let attacker = t.battlefield(P0, "Hill Giant");
    // Grizzly Bears: mana value 2 (even); Llanowar Elves: 1 (odd).
    let even = t.battlefield(P1, "Grizzly Bears");
    let odd = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(even, attacker)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(odd, attacker)]));
}
