//! Rulings batch S27 — activated abilities are written "[Cost]: [Effect]." (CR 602.1),
//! keywords such as equip, level up, unearth and crew stand for activated abilities
//! (CR 702.6a, 702.87a, 702.84a, 702.122a), and a mana ability is one that produces mana,
//! not one that costs mana (CR 605.1a): effects that stop, counter or copy activated
//! abilities see them all.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::{cast_new, change_copy_targets};
use crate::r_s27_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 names `name` for the "choose a card name" of the permanent entering next.
fn name_card(t: &mut TestGame, p: PlayerId, name: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(name.into()));
}

/// The amount of mana of type `ty` in `p`'s mana pool.
fn pool(t: &TestGame, p: PlayerId, ty: ManaType) -> u32 {
    t.g.player(p)
        .mana_pool
        .mana
        .iter()
        .filter(|m| m.ty == ty)
        .count() as u32
}

// ---------------------------------------------------------------------------------------
// Effects that stop activated abilities
// ---------------------------------------------------------------------------------------

#[test]
fn clarion_conqueror_stops_keyword_loyalty_and_mana_abilities() {
    cr!("602.1", "602.5", "702.6a", "702.87a", "606.2", "605.1a");
    ruling!(
        "Clarion Conqueror",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keywords are activated abilities and will have colons in their reminder text."
    );
    supported("Clarion Conqueror");
    // "Activated abilities of artifacts, creatures, and planeswalkers can't be
    // activated." Equip and level up are keywords standing for activated abilities.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let student = t.battlefield(P0, "Student of Warfare");
    let lili = t.battlefield(P0, "Liliana of the Veil");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let all = [blade, student, lili, elves];
    for p in all {
        assert!(!activatable(&mut t, P0, p).is_empty());
    }
    t.battlefield(P1, "Clarion Conqueror");
    for p in all {
        assert!(activatable(&mut t, P0, p).is_empty());
    }
    // A land's abilities are unaffected.
    let plains = t.named_on_battlefield("Plains")[0];
    assert!(!activatable(&mut t, P0, plains).is_empty());
}

#[test]
fn petrify_stops_the_equip_ability_of_the_enchanted_equipment() {
    cr!("602.5", "702.6a");
    ruling!(
        "Petrify",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keywords are activated abilities and will have colons in their reminder text."
    );
    supported("Petrify");
    // "Enchanted permanent can't attack or block, and its activated abilities can't be
    // activated."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let student = t.battlefield(P0, "Student of Warfare");
    assert!(can_activate_containing(&mut t, P0, blade, "Equip"));
    assert!(can_activate_containing(&mut t, P0, student, "Level Up"));
    attach_new(&mut t, P1, "Petrify", blade);
    attach_new(&mut t, P1, "Petrify", student);
    assert!(activatable(&mut t, P0, blade).is_empty());
    assert!(activatable(&mut t, P0, student).is_empty());
}

#[test]
fn stuck_in_summoner_s_sanctum_stops_equip() {
    cr!("602.5", "702.6a");
    ruling!(
        "Stuck in Summoner's Sanctum",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have a colon in their reminder text."
    );
    supported("Stuck in Summoner's Sanctum");
    // "Enchanted permanent doesn't untap during its controller's untap step and its
    // activated abilities can't be activated."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(can_activate_containing(&mut t, P0, blade, "Equip"));
    attach_new(&mut t, P1, "Stuck in Summoner's Sanctum", blade);
    assert!(activatable(&mut t, P0, blade).is_empty());
}

#[test]
fn cursed_totem_stops_creatures_level_up_and_mana_abilities_but_not_equip() {
    cr!("602.5", "702.87a", "605.1a", "702.6a");
    ruling!(
        "Cursed Totem",
        "Activated abilities contain a colon. They're generally written “[Cost]: [Effect].” Some keywords are activated abilities and will have colons in their reminder text."
    );
    supported("Cursed Totem");
    // "Activated abilities of creatures can't be activated."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Cursed Totem");
    let student = t.battlefield(P0, "Student of Warfare");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(activatable(&mut t, P0, student).is_empty());
    assert!(activatable(&mut t, P0, elves).is_empty());
    assert!(!tap_for_mana(&mut t, P0, elves, "Add {G}"));
    // Bonesplitter is an artifact, not a creature: it can still be equipped.
    assert!(can_activate_containing(&mut t, P0, blade, "Equip"));
}

#[test]
fn disruptor_flute_stops_equip_but_not_a_mana_ability_that_costs_mana() {
    cr!("602.5", "605.1a", "702.6a");
    ruling!(
        "Disruptor Flute",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keywords are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Disruptor Flute");
    // "Activated abilities of sources with the chosen name can't be activated unless
    // they're mana abilities."
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let signet = t.battlefield(P0, "Izzet Signet");
    name_card(&mut t, P1, "Bonesplitter");
    t.enter(P1, "Disruptor Flute");
    name_card(&mut t, P1, "Izzet Signet");
    t.enter(P1, "Disruptor Flute");
    assert!(activatable(&mut t, P0, blade).is_empty());
    // "{1}, {T}: Add {U}{R}." costs mana but produces mana: it's a mana ability.
    assert!(tap_for_mana(&mut t, P0, signet, "Add {U}{R}"));
    assert_eq!(pool(&t, P0, ManaType::U), 1);
    assert_eq!(pool(&t, P0, ManaType::R), 1);
}

#[test]
fn sorcerous_spyglass_stops_an_ability_that_costs_mana_but_not_a_mana_ability() {
    cr!("602.5", "605.1a");
    ruling!(
        "Sorcerous Spyglass",
        "An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Sorcerous Spyglass");
    // Mind Stone: "{T}: Add {C}. {1}, {T}, Sacrifice this artifact: Draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(can_activate_containing(&mut t, P0, stone, "Draw a card"));
    name_card(&mut t, P1, "Mind Stone");
    t.enter(P1, "Sorcerous Spyglass");
    assert!(!can_activate_containing(&mut t, P0, stone, "Draw a card"));
    assert!(tap_for_mana(&mut t, P0, stone, "Add {C}"));
    assert_eq!(pool(&t, P0, ManaType::C), 1);
}

#[test]
fn damping_matrix_stops_abilities_that_cost_mana_but_not_mana_abilities() {
    cr!("602.5", "605.1a");
    ruling!(
        "Damping Matrix",
        "A mana ability is an ability that produces mana, not an ability that costs mana."
    );
    supported("Damping Matrix");
    // "Activated abilities of artifacts and creatures can't be activated unless they're
    // mana abilities."
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let stone = t.battlefield(P0, "Mind Stone");
    let signet = t.battlefield(P0, "Izzet Signet");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.battlefield(P1, "Damping Matrix");
    assert!(!can_activate_containing(&mut t, P0, stone, "Draw a card"));
    assert!(activatable(&mut t, P0, sorcerer).is_empty());
    assert!(tap_for_mana(&mut t, P0, signet, "Add {U}{R}"));
    assert!(tap_for_mana(&mut t, P0, stone, "Add {C}"));
}

// ---------------------------------------------------------------------------------------
// Effects that counter, copy or destroy by activated abilities
// ---------------------------------------------------------------------------------------

#[test]
fn defabricate_counters_unearth_and_equip_abilities() {
    cr!("701.6a", "701.6b", "702.84a", "702.6a", "602.1");
    ruling!(
        "Defabricate",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip and unearth) are activated abilities and will have colons in their reminder text."
    );
    supported("Defabricate");
    supported("Dregscape Zombie");
    // Defabricate: "Choose one — • Counter target artifact or enchantment spell. ... •
    // Counter target activated or triggered ability."
    let mut t = TestGame::new(2);
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    t.lands(P0, "Swamp", 1);
    let unearth = activate_containing(&mut t, P0, zombie, "Unearth")
        .unwrap()
        .expect("on the stack");
    t.lands(P1, "Island", 2);
    let defab = t.hand(P1, "Defabricate");
    t.cast(P1, defab).modes(&[1]).target(unearth).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dregscape Zombie"));
    assert!(t.named_on_battlefield("Dregscape Zombie").is_empty());
    // The {B} paid for it isn't refunded.
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Equip.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let equip = activate_containing(&mut t, P0, blade, "Equip")
        .unwrap()
        .expect("on the stack");
    t.lands(P1, "Island", 2);
    let defab = t.hand(P1, "Defabricate");
    t.cast(P1, defab).modes(&[1]).target(equip).go();
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), None);
}

#[test]
fn disallow_counters_a_crew_ability() {
    cr!("701.6a", "701.6b", "702.122a", "602.1");
    ruling!(
        "Disallow",
        "Activated abilities are written in the form \"Cost: Effect.\" Some keyword abilities, such as equip and crew, are activated abilities and will have colons in their reminder texts."
    );
    supported("Disallow");
    supported("Smuggler's Copter");
    // "Counter target spell, activated ability, or triggered ability."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let crew = activate_containing(&mut t, P0, copter, "Crew")
        .unwrap()
        .expect("on the stack");
    assert!(t.obj_now(bears).tapped);
    t.lands(P1, "Island", 3);
    let disallow = t.hand(P1, "Disallow");
    t.cast(P1, disallow).target(crew).go();
    t.resolve_all();
    assert!(!t.obj_now(copter).is(CardType::Creature));
    // The crew cost isn't refunded: the Bears stay tapped.
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn abstruse_archaic_copies_equip_and_an_ability_that_costs_mana() {
    cr!("707.10", "602.1", "702.6a", "605.1a");
    ruling!(
        "Abstruse Archaic",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Abstruse Archaic");
    // "{1}, {T}: Copy target activated or triggered ability you control from a colorless
    // source. You may choose new targets for the copy."
    // Mind Stone's "{1}, {T}, Sacrifice this artifact: Draw a card." costs mana but isn't
    // a mana ability: it uses the stack and can be copied.
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let stone = t.battlefield(P0, "Mind Stone");
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    let draw = activate_containing(&mut t, P0, stone, "Draw a card")
        .unwrap()
        .expect("on the stack");
    t.answer_targets(P0, &[Entity::Object(draw)]);
    activate_containing(&mut t, P0, archaic, "Copy target").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Equip: the copy moves Bonesplitter to the other creature first.
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(a)]);
    let equip = activate_containing(&mut t, P0, blade, "Equip")
        .unwrap()
        .expect("on the stack");
    t.answer_targets(P0, &[Entity::Object(equip)]);
    activate_containing(&mut t, P0, archaic, "Copy target").unwrap();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    t.resolve();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(b)));
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(a)));
}

#[test]
fn the_peregrine_dynamo_copies_a_legendary_equipment_s_equip() {
    cr!("707.10", "702.6a", "602.1");
    ruling!(
        "The Peregrine Dynamo",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("The Peregrine Dynamo");
    // "{1}, {T}: Copy target activated or triggered ability you control from another
    // legendary source that's not a commander. You may choose new targets for the copy."
    // Glamdring is a legendary Equipment with equip {3}.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let glamdring = t.battlefield(P0, "Glamdring");
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[Entity::Object(a)]);
    let equip = activate_containing(&mut t, P0, glamdring, "Equip")
        .unwrap()
        .expect("on the stack");
    t.answer_targets(P0, &[Entity::Object(equip)]);
    activate_containing(&mut t, P0, dynamo, "Copy target").unwrap();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    t.resolve();
    assert_eq!(attached_to(&t, glamdring), Some(Entity::Object(b)));
    t.resolve_all();
    assert_eq!(attached_to(&t, glamdring), Some(Entity::Object(a)));
}

#[test]
fn ertha_jo_copies_an_equip_ability() {
    cr!("707.10", "702.6a", "603.2");
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text."
    );
    supported("Ertha Jo, Frontier Mentor");
    // "Whenever you activate an ability that targets a creature or player, copy that
    // ability. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ertha Jo, Frontier Mentor");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(a)]);
    activate_containing(&mut t, P0, blade, "Equip").unwrap();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    // Ertha Jo's trigger resolves, then the copy.
    t.resolve();
    t.resolve();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(b)));
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(a)));
}

#[test]
fn ravager_wurm_can_destroy_only_a_land_with_a_non_mana_activated_ability() {
    cr!("605.1a", "115.1", "700.2");
    ruling!(
        "Ravager Wurm",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Ravager Wurm");
    supported("Graven Cairns");
    supported("Mishra's Factory");
    // "When this creature enters, choose up to one — ... • Destroy target land with an
    // activated ability that isn't a mana ability." Graven Cairns's "{B/R}, {T}: Add
    // {B}{B}, {B}{R}, or {R}{R}." costs mana but is a mana ability; Mishra's Factory's
    // "{1}: This land becomes a 2/2 ... creature" isn't a mana ability.
    let mut t = TestGame::new(2);
    let cairns = t.battlefield(P1, "Graven Cairns");
    let factory = t.battlefield(P1, "Mishra's Factory");
    let forest = t.battlefield(P1, "Forest");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(factory)]);
    t.enter(P0, "Ravager Wurm");
    t.resolve_all();
    let offered = target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    for c in &offered {
        assert!(c.contains(&Entity::Object(factory)));
        assert!(!c.contains(&Entity::Object(cairns)));
        assert!(!c.contains(&Entity::Object(forest)));
    }
    assert!(t.in_graveyard(P1, "Mishra's Factory"));
    assert!(t.on_battlefield(cairns));
}

#[test]
fn nyxbloom_ancient_multiplies_only_mana_from_abilities_with_the_tap_symbol() {
    cr!("106.12", "106.12b", "605.1a");
    ruling!(
        "Nyxbloom Ancient",
        "You’re “tapping a permanent for mana” only if you’re activating a mana ability of that permanent that includes the {T} symbol in its cost. A mana ability produces mana as part of its effect."
    );
    supported("Nyxbloom Ancient");
    supported("Ashnod's Altar");
    // "If you tap a permanent for mana, it produces three times as much of that mana
    // instead." Ashnod's Altar ("Sacrifice a creature: Add {C}{C}.") isn't tapped for
    // mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nyxbloom Ancient");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let altar = t.battlefield(P0, "Ashnod's Altar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(tap_for_mana(&mut t, P0, elves, "Add {G}"));
    assert_eq!(pool(&t, P0, ManaType::G), 3);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    assert!(tap_for_mana(&mut t, P0, altar, "Add {C}{C}"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(pool(&t, P0, ManaType::C), 2);
    assert_eq!(t.zone(elves), Zone::Battlefield);
}

#[test]
fn granite_shard_s_ability_can_be_activated_by_paying_either_cost() {
    cr!("602.2b", "601.2h", "118.3");
    ruling!(
        "Granite Shard",
        "You can pay either of the two costs (but not both at the same time) to activate the ability."
    );
    supported("Granite Shard");
    // "{3}, {T} or {R}, {T}: This artifact deals 1 damage to any target."
    // With a Mountain, the {R} cost is paid; the {3} one can't be.
    let mut t = TestGame::new(2);
    let shard = t.battlefield(P0, "Granite Shard");
    t.lands(P0, "Mountain", 1);
    assert!(can_activate_containing(&mut t, P0, shard, "{R}, {T}"));
    assert!(!can_activate_containing(&mut t, P0, shard, "{3}, {T}"));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, shard, "{R}, {T}").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // It's tapped: neither cost can be paid again.
    t.lands(P0, "Mountain", 3);
    assert!(activatable(&mut t, P0, shard).is_empty());
    // With three Wastes, the {3} cost is paid.
    let mut t = TestGame::new(2);
    let shard = t.battlefield(P0, "Granite Shard");
    t.lands(P0, "Wastes", 3);
    assert!(!can_activate_containing(&mut t, P0, shard, "{R}, {T}"));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, shard, "{3}, {T}").unwrap();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
