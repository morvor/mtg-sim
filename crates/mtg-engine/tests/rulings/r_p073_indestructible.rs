//! Rulings batch P073 — granting indestructible: what indestructible does and doesn't
//! stop (CR 702.12b), targeted grants that last for the turn, base power and toughness
//! setting effects (Taranika, Almost Perfect; CR 613.4b–c), targets checked on activation
//! and resolution (Spearbreaker Behemoth), self-targeting costs (Resolute Watchdog),
//! commanders (Bastion Protector, Anara; CR 903.3d), Wintermoor Commander's
//! characteristic-defining toughness (CR 604.3), That Which Was Taken's divinity
//! counters, and Darksteel Garrison's mandatory trigger.

use crate::r_p073_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::{ability_targets, crew};
use crate::r_s06_common::{activate_containing, attach_new, damage, give_control};
use crate::r_s13_common::add;
use crate::r_s21_common::legal_blocks;
use crate::r_s27_common::can_activate_containing;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// What indestructible stops
// ---------------------------------------------------------------------------

#[test]
fn consecrate_land_stops_destroy_but_not_sacrifice() {
    cr!("702.12b", "701.21a", "608.2b");
    ruling!(
        "Consecrate Land",
        "A permanent with indestructible can't be destroyed, but it can still be sacrificed, exiled, put into a graveyard, and so on."
    );
    ruling!(
        "Consecrate Land",
        "The land can be targeted by land-destroying spells and the spell will resolve, but the land will simply not be destroyed."
    );
    supported("Consecrate Land");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    attach_new(&mut t, P1, "Consecrate Land", obj(forest));
    assert!(crate::r_s04_common::spell_targets(&mut t, P0, "Stone Rain").contains(&obj(forest)));
    let spell = cast_resolve(&mut t, P0, "Stone Rain", &[obj(forest)]);
    assert!(crate::r_s07_common::resolved(&t, spell));
    assert!(t.on_battlefield(forest));
    // Sacrificed or exiled, it goes.
    t.g.sacrifice(forest, P1);
    t.settle();
    assert!(t.in_graveyard(P1, "Forest"));
    let plains = t.battlefield(P1, "Plains");
    attach_new(&mut t, P1, "Consecrate Land", obj(plains));
    t.g.exile_object(plains, None);
    t.settle();
    assert!(t.in_exile("Plains"));
}

#[test]
fn indestructible_permanents_still_go_to_the_graveyard_for_other_reasons() {
    cr!("702.12b", "704.5f", "704.5g", "704.5h", "704.5j", "704.5m");
    supported("Indestructibility");
    // Lethal damage, deathtouch damage and "destroy" don't put it into the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Indestructibility", obj(bears));
    let giant = t.battlefield(P1, "Hill Giant");
    let rats = t.battlefield(P1, "Typhoid Rats");
    damage(&mut t, giant, 3, bears);
    damage(&mut t, rats, 1, bears);
    destroy_now(&mut t, bears);
    assert!(t.on_battlefield(bears));
    // Sacrifice.
    t.g.sacrifice(bears, P0);
    t.settle();
    assert!(!t.on_battlefield(bears));
    // Toughness 0.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Indestructibility", obj(elves));
    cast_resolve(&mut t, P1, "Disfigure", &[obj(elves)]);
    assert!(!t.on_battlefield(elves));
    // The legend rule.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, "Indestructibility", obj(a));
    t.answer_choose(P0, &[]);
    let b = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.dirty = true;
    t.settle();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "one of them is put into the graveyard"
    );
    // An indestructible Aura that's unattached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hs = attach_new(&mut t, P0, "Holy Strength", obj(bears));
    attach_new(&mut t, P0, "Indestructibility", obj(hs));
    assert!(kw(&t, hs, INDESTRUCTIBLE));
    destroy_now(&mut t, bears);
    assert!(!t.on_battlefield(hs));
}

// ---------------------------------------------------------------------------
// Targeted grants for the turn
// ---------------------------------------------------------------------------

#[test]
fn accursed_horde_can_target_itself_and_it_lasts_after_combat() {
    cr!("115.1", "611.2a", "506.4");
    ruling!(
        "Accursed Horde",
        "Accursed Horde's ability can target itself while it's attacking."
    );
    ruling!(
        "Accursed Horde",
        "The target Zombie gains indestructible for the rest of the turn, even after it stops being an attacking creature."
    );
    supported("Accursed Horde");
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P0, "Accursed Horde");
    // Another Zombie that isn't attacking can't be targeted.
    t.battlefield(P0, "Walking Corpse");
    t.lands(P0, "Swamp", 2);
    assert!(ability_targets(&mut t, horde, 0).is_empty(), "not attacking yet");
    attack_with(&mut t, &[(horde, pl(P1))]);
    let cands = ability_targets(&mut t, horde, 0);
    assert_eq!(cands, vec![obj(horde)], "only attacking Zombies");
    t.answer_targets(P0, &[obj(horde)]);
    activate_containing(&mut t, P0, horde, "indestructible").unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    assert!(kw(&t, horde, INDESTRUCTIBLE));
    assert!(!destroy_now(&mut t, horde));
}

#[test]
fn untap_grants_can_target_untapped_creatures() {
    cr!("701.26b", "115.1");
    ruling!(
        "Moment of Valor",
        "Moment of Valor's first mode can target an untapped creature. It still gets +1/+0 and gains indestructible until end of turn."
    );
    ruling!(
        "Taranika, Akroan Veteran",
        "Taranika's triggered ability can target a creature that's already untapped. It still becomes 4/4 and gains indestructible."
    );
    ruling!(
        "Gyome, Master Chef",
        "You can activate the last ability even if the creature is already tapped."
    );
    supported("Moment of Valor");
    supported("Taranika, Akroan Veteran");
    supported("Gyome, Master Chef");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, "Moment of Valor");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Moment of Valor");
    t.cast(P0, c).modes(&[0]).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert!(kw(&t, bears, INDESTRUCTIBLE) && !t.obj_now(bears).tapped);
    // Taranika.
    let mut t = TestGame::new(2);
    let tara = t.battlefield(P0, "Taranika, Akroan Veteran");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(tara, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(kw(&t, bears, INDESTRUCTIBLE) && !t.obj_now(bears).tapped);
    // Gyome: the target is already tapped.
    let mut t = TestGame::new(2);
    let gyome = t.battlefield(P0, "Gyome, Master Chef");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Food");
    t.g.tap(bears);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, gyome, "Sacrifice a Food").unwrap();
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE) && t.obj_now(bears).tapped);
}

#[test]
fn taranika_trigger_is_mandatory_and_other_modifiers_still_apply() {
    cr!("603.3d", "613.4b", "613.4c");
    ruling!(
        "Taranika, Akroan Veteran",
        "Taranika's triggered ability isn't optional. If your only other creatures are bigger than 4/4, you'll have to shrink one for the turn."
    );
    ruling!(
        "Taranika, Akroan Veteran",
        "Any effects that modify the target creature's power and/or toughness without setting them to a specific number or value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    let mut t = TestGame::new(2);
    let tara = t.battlefield(P0, "Taranika, Akroan Veteran");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(tara, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 4), "its only other creature, shrunk");
    // A Giant Growth from before and a +1/+1 counter apply on top of the base 4/4.
    let mut t = TestGame::new(2);
    let tara = t.battlefield(P0, "Taranika, Akroan Veteran");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bears, "+1/+1", 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (6, 6));
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(tara, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn almost_perfect_sets_base_power_and_toughness_in_timestamp_order() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Almost Perfect",
        "Almost Perfect overwrites all previous effects that set the creature’s base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect."
    );
    ruling!(
        "Almost Perfect",
        "Effects that modify a creature’s power and/or toughness, such as the effect of the ability granted by Idolized, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Almost Perfect");
    let mut t = TestGame::new(2);
    let tara = t.battlefield(P0, "Taranika, Akroan Veteran");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An earlier base 4/4 (Taranika), a Giant Growth and a counter from before.
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(tara, pl(P1))]);
    t.resolve_all();
    add(&mut t, bears, "+1/+1", 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (8, 8));
    attach_new(&mut t, P0, "Almost Perfect", obj(bears));
    assert_eq!(t.pt(bears), (13, 14));
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    // A later switch applies on top.
    cast_resolve(&mut t, P0, "Twisted Image", &[obj(bears)]);
    assert_eq!(t.pt(bears), (14, 13));
    // A later base-setting effect overwrites it.
    cast_resolve(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5), "base 1/1 +3/+3 +1/+1, switched");
}

#[test]
fn spearbreaker_behemoth_checks_power_on_targeting_and_resolution_only() {
    cr!("608.2b", "611.2c", "115.1");
    ruling!(
        "Spearbreaker Behemoth",
        "The ability checks the targeted creature's power twice: when the creature becomes the target, and when the ability resolves. Once the ability resolves, it will continue to apply to the affected creature no matter what its power may become later in the turn."
    );
    supported("Spearbreaker Behemoth");
    let mut t = TestGame::new(2);
    let sb = t.battlefield(P0, "Spearbreaker Behemoth");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let cands = ability_targets(&mut t, sb, 0);
    assert!(cands.contains(&obj(wurm)) && !cands.contains(&obj(bears)));
    // Power drops below 5 in response: the ability does nothing.
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, sb, "indestructible").unwrap();
    cast_resolve(&mut t, P1, "Disfigure", &[obj(wurm)]);
    assert!(!kw(&t, wurm, INDESTRUCTIBLE));
    // Resolved: it stays even after the power drops.
    let wurm2 = t.battlefield(P0, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm2)]);
    activate_containing(&mut t, P0, sb, "indestructible").unwrap();
    t.resolve_all();
    cast_resolve(&mut t, P1, "Disfigure", &[obj(wurm2)]);
    assert_eq!(t.pt(wurm2), (4, 2));
    assert!(kw(&t, wurm2, INDESTRUCTIBLE));
}

#[test]
fn apocalypse_runner_grant_lasts_even_if_power_grows() {
    cr!("611.2c", "608.2b");
    ruling!(
        "Apocalypse Runner",
        "After the first ability has resolved targeting a creature, that creature has lifelink and can’t be blocked for the rest of the turn, even if its power increases to 3 or greater."
    );
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Apocalypse Runner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, runner, "lifelink").unwrap();
    t.resolve_all();
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(wall, bears)]));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!((t.life(P0), t.life(P1)), (25, 15));
}

#[test]
fn sacrifice_cost_grants_target_any_creature_you_control() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Etchings of the Chosen",
        "The last ability of Etchings of the Chosen can target any creature you control, not just one of the chosen type."
    );
    ruling!(
        "Spirit Bonds",
        "You can sacrifice any Spirit to activate the last ability, not just a Spirit token created by Spirit Bonds."
    );
    supported("Etchings of the Chosen");
    supported("Spirit Bonds");
    // Etchings naming Elf: sacrifice the Elves, the Bears (not an Elf) is protected.
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Elf");
    let etch = t.enter(P0, "Etchings of the Chosen");
    t.resolve_all();
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(elves), (2, 2));
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(elves)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, etch, "indestructible").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    // Spirit Bonds: a nontoken Spirit card is sacrificed.
    let mut t = TestGame::new(2);
    let bonds = t.battlefield(P0, "Spirit Bonds");
    let kami = t.battlefield(P0, "Kami of Ancient Law");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.answer_choose(P0, &[obj(kami)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, bonds, "indestructible").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Kami of Ancient Law"));
    assert!(kw(&t, bears, INDESTRUCTIBLE));
}

#[test]
fn resolute_watchdog_can_target_itself_and_does_nothing() {
    cr!("608.2b", "602.2b", "115.1");
    ruling!(
        "Resolute Watchdog",
        "Resolute Watchdog can be the target of its own ability. The ability won’t resolve since it won’t have a legal target, and no creature will gain indestructible, but this does allow you to sacrifice Resolute Watchdog without another creature to target if you want to."
    );
    supported("Resolute Watchdog");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Resolute Watchdog");
    t.lands(P0, "Wastes", 1);
    assert_eq!(ability_targets(&mut t, dog, 0), vec![obj(dog)]);
    t.answer_targets(P0, &[obj(dog)]);
    activate_containing(&mut t, P0, dog, "indestructible").unwrap();
    assert!(t.in_graveyard(P0, "Resolute Watchdog"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Resolute Watchdog"));
}

#[test]
fn slobad_saves_an_artifact_creature_from_lethal_damage() {
    cr!("514.2", "702.12b");
    ruling!(
        "Slobad, Goblin Tinkerer",
        "Damage wears off simultaneously with this effect ending, so it does save an artifact creature from lethal damage."
    );
    supported("Slobad, Goblin Tinkerer");
    let mut t = TestGame::new(2);
    let slobad = t.battlefield(P0, "Slobad, Goblin Tinkerer");
    let thopter = t.battlefield(P0, "Ornithopter");
    let fodder = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer_targets(P0, &[obj(thopter)]);
    activate_containing(&mut t, P0, slobad, "indestructible").unwrap();
    t.resolve_all();
    damage(&mut t, giant, 3, thopter);
    assert!(t.on_battlefield(thopter));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(thopter));
    assert!(!kw(&t, thopter, INDESTRUCTIBLE));
    assert_eq!(t.obj_now(thopter).damage, 0);
}

#[test]
fn bladed_battle_fan_indestructible_stays_after_it_moves() {
    cr!("611.2a", "702.6a");
    ruling!(
        "Bladed Battle-Fan",
        "The creature that gains indestructible due to Bladed Battle-Fan's triggered ability will keep indestructible for the duration of the turn, even if Bladed Battle-Fan later becomes unattached."
    );
    supported("Bladed Battle-Fan");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_p079_common::cast_card(&mut t, P0, "Bladed Battle-Fan", &[]).unwrap();
    t.resolve_all();
    let fan = t.named_on_battlefield("Bladed Battle-Fan")[0];
    assert_eq!(t.obj_now(fan).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 2));
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, fan, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(fan).attached_to, Some(obj(giant)));
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert!(!kw(&t, giant, INDESTRUCTIBLE));
}

// ---------------------------------------------------------------------------
// Commanders
// ---------------------------------------------------------------------------

#[test]
fn bastion_protector_affects_only_commanders_that_are_creatures() {
    cr!("903.3d", "301.7b");
    ruling!(
        "Bastion Protector",
        "A \"commander creature\" is a permanent that's both someone's commander and a creature."
    );
    ruling!(
        "Bastion Protector",
        "Bastion Protector's ability doesn't apply to commanders that aren't currently creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bastion Protector");
    let copter = bf_commander(&mut t, P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!kw(&t, copter, INDESTRUCTIBLE));
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(kw(&t, copter, INDESTRUCTIBLE));
    assert_eq!(t.pt(copter), (5, 5));
    assert_eq!(t.pt(bears), (2, 2), "not a commander");
}

#[test]
fn anara_affects_commanders_you_control_during_your_turn() {
    cr!("903.3d", "611.3a");
    ruling!(
        "Anara, Wolvid Familiar",
        "Anara's first ability affects commanders you control even if you don't own them. It doesn't affect your commander if another player controls it."
    );
    ruling!(
        "Anara, Wolvid Familiar",
        "If Anara is your commander, its first ability affects itself during your turn."
    );
    supported("Anara, Wolvid Familiar");
    let mut t = TestGame::new(2);
    let anara = bf_commander(&mut t, P0, "Anara, Wolvid Familiar");
    let mine = bf_commander(&mut t, P0, "Grizzly Bears");
    let theirs = bf_commander(&mut t, P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    give_control(&mut t, mine, P1);
    assert!(kw(&t, anara, INDESTRUCTIBLE));
    assert!(kw(&t, theirs, INDESTRUCTIBLE));
    assert!(!kw(&t, mine, INDESTRUCTIBLE));
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!kw(&t, anara, INDESTRUCTIBLE));
    assert!(!kw(&t, theirs, INDESTRUCTIBLE));
}

#[test]
fn damage_marked_on_a_protected_creature_becomes_lethal_when_the_source_leaves() {
    cr!("704.5g", "120.6");
    ruling!(
        "Anara, Wolvid Familiar",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a commander you control may become lethal if Anara leaves the battlefield during that turn."
    );
    ruling!(
        "Wintermoor Commander",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Wintermoor Commander may become lethal if Knights you control leave the battlefield during that turn."
    );
    supported("Wintermoor Commander");
    let mut t = TestGame::new(2);
    let anara = t.battlefield(P0, "Anara, Wolvid Familiar");
    let cmd = bf_commander(&mut t, P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 3, cmd);
    assert!(t.on_battlefield(cmd));
    destroy_now(&mut t, anara);
    assert!(!t.on_battlefield(cmd));
    // Wintermoor Commander with another Knight is 2/2; 1 damage, then the Knight leaves.
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Wintermoor Commander");
    let knight = t.battlefield(P0, "Knight of Malice");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(wc), (2, 2));
    damage(&mut t, giant, 1, wc);
    assert!(t.on_battlefield(wc));
    destroy_now(&mut t, knight);
    assert!(!t.on_battlefield(wc));
}

#[test]
fn wintermoor_commanders_toughness_counts_knights_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Wintermoor Commander",
        "As long as Wintermoor Commander is on the battlefield, its ability will count itself, so it'll be at least 2/1."
    );
    ruling!(
        "Wintermoor Commander",
        "The ability that defines Wintermoor Commander's toughness works in all zones, not just the battlefield."
    );
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Wintermoor Commander");
    assert_eq!(t.pt(wc), (2, 1));
    t.battlefield(P0, "Knight of Malice");
    t.battlefield(P0, "Knight of Malice");
    let in_hand = t.hand(P0, "Wintermoor Commander");
    let in_gy = t.graveyard(P0, "Wintermoor Commander");
    t.g.recompute();
    assert_eq!(t.pt(wc), (2, 3));
    assert_eq!(t.g.obj(in_hand).toughness(), 3);
    assert_eq!(t.g.obj(in_gy).toughness(), 3);
}

// ---------------------------------------------------------------------------
// That Which Was Taken
// ---------------------------------------------------------------------------

#[test]
fn that_which_was_taken_counters_and_indestructible() {
    cr!("122.1", "611.3a", "115.1");
    ruling!(
        "That Which Was Taken",
        "One That Which Was Taken can put divinity counters on another That Which Was Taken."
    );
    ruling!(
        "That Which Was Taken",
        "Permanents with divinity counters on them have indestructible only while That Which Was Taken is on the battlefield."
    );
    ruling!(
        "That Which Was Taken",
        "The divinity counters created by That Which Was Taken are the same type of counter as those used by the Myojin creatures in the Champions of Kamigawa set."
    );
    supported("That Which Was Taken");
    supported("Myojin of Cleansing Fire");
    // It's legendary: the other one is another player's.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "That Which Was Taken");
    let b2 = t.battlefield(P1, "That Which Was Taken");
    let cands = ability_targets(&mut t, a, 0);
    assert!(cands.contains(&obj(b2)) && !cands.contains(&obj(a)));
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(b2)]);
    activate_containing(&mut t, P0, a, "divinity").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(b2, "divinity"), 1);
    assert!(kw(&t, b2, INDESTRUCTIBLE));
    // A Myojin that has lost its counter gets one back and can use it.
    let myojin = t.battlefield(P0, "Myojin of Cleansing Fire");
    assert!(!can_activate_containing(&mut t, P0, myojin, "Remove a divinity counter"));
    t.g.untap(a);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(myojin)]);
    activate_containing(&mut t, P0, a, "divinity").unwrap();
    t.resolve_all();
    assert!(kw(&t, myojin, INDESTRUCTIBLE));
    assert!(can_activate_containing(&mut t, P0, myojin, "Remove a divinity counter"));
    // Only while a That Which Was Taken is on the battlefield.
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bears, "divinity", 1);
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    destroy_now(&mut t, a);
    assert!(!t.on_battlefield(a));
    assert!(kw(&t, bears, INDESTRUCTIBLE), "the other one still applies");
    t.g.exile_object(b2, None);
    t.settle();
    assert_eq!(t.counters(bears, "divinity"), 1);
    assert!(!kw(&t, bears, INDESTRUCTIBLE));
    // The Myojin's own ability still counts the counter.
    assert!(kw(&t, myojin, INDESTRUCTIBLE));
}

#[test]
fn darksteel_garrison_trigger_must_target_an_opponents_creature_if_needed() {
    cr!("603.3d", "115.1");
    ruling!(
        "Darksteel Garrison",
        "The triggered ability is mandatory. If you don’t have a creature to target, you must target another player’s creature, if possible."
    );
    supported("Darksteel Garrison");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Darksteel Garrison", obj(land));
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.g.tap(land);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (3, 3));
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.is_empty()));
}
