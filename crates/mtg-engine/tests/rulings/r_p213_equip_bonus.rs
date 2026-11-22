//! Rulings batch P213 — Equipment (CR 301.5, 702.6): bonuses that count things and keep
//! updating (CR 611.3a, 613.4c), abilities the equipped creature has regardless of the
//! count, evasion and block requirements, and equip abilities' targets and costs.

use crate::r_p213_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s04_common::ability_targets;
use crate::r_s06_common::*;
use crate::r_s21_common::{blocks_now, legal_blocks};
use crate::r_s26_common::modify_until_eot;
use crate::r_s30_common::set_life;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn creature(t: &mut TestGame, p: PlayerId, name: &str, type_line: &str) -> ObjectId {
    let def = custom_card(name, type_line, "{1}", Some((2, 2)), "");
    t.custom(p, def, mtg_engine::object::Zone::Battlefield)
}

// ---------------------------------------------------------------------------------------
// Bonuses that count
// ---------------------------------------------------------------------------------------

#[test]
fn golem_skin_gauntlets_adds_to_the_other_equipments_bonus() {
    cr!("613.4c", "301.5");
    ruling!(
        "Golem-Skin Gauntlets",
        "The bonus from Golem-Skin Gauntlets is in addition to whatever bonus the other Equipment gives it."
    );
    supported("Golem-Skin Gauntlets");
    // "Equipped creature gets +1/+0 for each Equipment attached to it." With Shuko
    // (+1/+0) also attached: +2/+0 and +1/+0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Golem-Skin Gauntlets", bears);
    assert_eq!(t.pt(bears), (3, 2));
    attach_new(&mut t, P0, "Shuko", bears);
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn bearded_axe_updates_as_the_count_changes() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Bearded Axe",
        "The bonus granted by Bearded Axe is constantly updated as the number of Dwarves, Equipment, and Vehicles you control changes."
    );
    supported("Bearded Axe");
    // "Equipped creature gets +1/+1 for each Dwarf, Equipment, and/or Vehicle you
    // control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bearded Axe", bears);
    assert_eq!(t.pt(bears), (3, 3));
    let shuko = t.battlefield(P0, "Shuko");
    assert_eq!(t.pt(bears), (4, 4));
    t.battlefield(P0, "Staunch Shieldmate");
    assert_eq!(t.pt(bears), (5, 5));
    // An opponent's Dwarf doesn't count.
    t.battlefield(P1, "Staunch Shieldmate");
    assert_eq!(t.pt(bears), (5, 5));
    destroy(&mut t, shuko);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn stoneforge_masterwork_counts_each_creature_once_and_keeps_updating() {
    cr!("611.3a", "613.4c", "704.5g", "205.3");
    ruling!(
        "Stoneforge Masterwork",
        "The bonus is only +1/+1 per applicable creature, even if it shares more than one creature type with the equipped creature. For example, if the equipped creature is a Human Shaman and you control a Human Warrior, a Goblin Shaman, and another Human Shaman, the equipped creature will get +3/+3."
    );
    ruling!(
        "Stoneforge Masterwork",
        "The bonus will change as the creatures you control change. This may cause the equipped creature to lose toughness and die because of damage dealt earlier in the turn."
    );
    supported("Stoneforge Masterwork");
    let mut t = TestGame::new(2);
    let shaman = creature(&mut t, P0, "Human Shaman A", "Creature — Human Shaman");
    let warrior = creature(&mut t, P0, "Human Warrior", "Creature — Human Warrior");
    creature(&mut t, P0, "Goblin Shaman", "Creature — Goblin Shaman");
    creature(&mut t, P0, "Human Shaman B", "Creature — Human Shaman");
    creature(&mut t, P0, "Elf Druid", "Creature — Elf Druid");
    creature(&mut t, P1, "Opposing Human", "Creature — Human Soldier");
    attach_new(&mut t, P0, "Stoneforge Masterwork", shaman);
    assert_eq!(t.pt(shaman), (5, 5));
    // Damage dealt earlier in the turn becomes lethal as the bonus shrinks.
    let bolt_source = t.battlefield(P1, "Hill Giant");
    damage(&mut t, bolt_source, 4, shaman);
    assert!(t.on_battlefield(shaman));
    destroy(&mut t, warrior);
    assert!(!t.on_battlefield(shaman));
}

#[test]
fn conquerors_flail_shrinks_and_damage_becomes_lethal() {
    cr!("611.3a", "613.4c", "704.5g", "105.2");
    ruling!(
        "Conqueror's Flail",
        "The first ability of Conqueror's Flail constantly updates as the number of colors among permanents you control changes. If the number of colors decreases and the damage that was marked on the equipped creature earlier in the turn is now lethal damage, that creature is destroyed."
    );
    supported("Conqueror's Flail");
    // "Equipped creature gets +1/+1 for each color among permanents you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    attach_new(&mut t, P0, "Conqueror's Flail", bears);
    assert_eq!(t.pt(bears), (4, 4));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 3, bears);
    assert!(t.on_battlefield(bears));
    destroy(&mut t, lion);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn aettir_and_priwen_follows_your_life_total() {
    cr!("613.4b", "611.3a", "704.5g");
    ruling!(
        "Aettir and Priwen",
        "The value of X changes as your life total changes. Since damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to the equipped creature may become lethal if your life total decreases during that turn."
    );
    supported("Aettir and Priwen");
    // "Equipped creature has base power and toughness X/X, where X is your life total."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Aettir and Priwen", bears);
    assert_eq!(t.pt(bears), (20, 20));
    set_life(&mut t, P0, 3);
    t.g.dirty = true;
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 2, bears);
    assert!(t.on_battlefield(bears));
    t.g.lose_life(P0, 1);
    t.settle();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn abilities_granted_regardless_of_the_count() {
    cr!("613.1f", "702.20b", "702.111b");
    ruling!(
        "Glaive of the Guildpact",
        "The equipped creature has vigilance and menace even if you control no Gates."
    );
    ruling!(
        "Mace of the Valiant",
        "The equipped creature has vigilance even if Mace of the Valiant has no charge counters on it."
    );
    supported("Glaive of the Guildpact");
    supported("Mace of the Valiant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Glaive of the Guildpact", bears);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    assert!(has_kw(&t, bears, KeywordKind::Menace));
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let mace = attach_new(&mut t, P0, "Mace of the Valiant", lion);
    assert_eq!(t.counters(mace, "charge"), 0);
    assert_eq!(t.pt(lion), (2, 2));
    assert!(has_kw(&t, lion, KeywordKind::Vigilance));
}

#[test]
fn kondas_banner_the_equipped_creature_shares_with_itself() {
    cr!("105.4", "205.3", "613.4c");
    ruling!(
        "Konda's Banner",
        "The equipped creature shares a color with itself, as long as it has one or more colors."
    );
    ruling!(
        "Konda's Banner",
        "The equipped creature shares a creature type with itself, as long as it has one or more creature types. “Artifact” is not a creature type."
    );
    supported("Konda's Banner");
    // "Creatures that share a color with equipped creature get +1/+1. Creatures that
    // share a creature type with equipped creature get +1/+1." Isamaru, Hound of Konda
    // (white Dog) gets both.
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Konda's Banner", isamaru);
    assert_eq!(t.pt(isamaru), (4, 4));
    assert_eq!(t.pt(lion), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    // A colorless legendary artifact creature with no creature types shares neither with
    // itself nor (being an artifact) with other artifact creatures.
    let mut t = TestGame::new(2);
    let golem = custom_card(
        "Typeless Relic",
        "Legendary Artifact Creature",
        "{3}",
        Some((2, 2)),
        "",
    );
    let golem = t.custom(P0, golem, mtg_engine::object::Zone::Battlefield);
    let thopter = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Konda's Banner", golem);
    assert_eq!(t.pt(golem), (2, 2));
    assert_eq!(t.pt(thopter), (0, 2));
}

#[test]
fn silverskin_armor_keeps_the_creatures_colors() {
    cr!("205.1b", "105.2c", "613.1d");
    ruling!(
        "Silverskin Armor",
        "The equipped creature will remain whatever color(s) it was previously. Becoming an artifact doesn’t make the creature colorless."
    );
    supported("Silverskin Armor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Silverskin Armor", bears);
    assert!(t.obj_now(bears).is(CardType::Artifact));
    assert!(t.obj_now(bears).is(CardType::Creature));
    assert_eq!(colors_of(&t, bears), ColorSet::single(Color::Green));
    assert_eq!(t.pt(bears), (3, 3));
}

fn colors_of(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

#[test]
fn bladehold_war_whip_reduces_unattached_equipments_equip_costs() {
    cr!("601.2f", "602.2b", "702.6a");
    ruling!(
        "Bladehold War-Whip",
        "The second ability affects other Equipment even if it isn't currently attached to a creature."
    );
    supported("Bladehold War-Whip");
    // "Equip abilities you activate of other Equipment cost {1} less to activate." The
    // War-Whip is attached to nothing; Golem-Skin Gauntlets (equip {2}) costs {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bladehold War-Whip");
    let gauntlets = t.battlefield(P0, "Golem-Skin Gauntlets");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    equip(&mut t, P0, gauntlets, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, gauntlets), Some(Entity::Object(bears)));
}

#[test]
fn warmongers_chariot_lets_a_defender_attack_without_removing_defender() {
    cr!("702.3b", "508.1c");
    ruling!(
        "Warmonger's Chariot",
        "The second ability doesn’t cause the equipped creature to lose defender. It just lets it attack."
    );
    supported("Warmonger's Chariot");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Wood");
    assert!(!can_attack(&mut t, wall));
    attach_new(&mut t, P0, "Warmonger's Chariot", wall);
    assert!(has_kw(&t, wall, KeywordKind::Defender));
    assert_eq!(t.pt(wall), (2, 5));
    attack_with(&mut t, &[(wall, Entity::Player(P1))]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert!(has_kw(&t, wall, KeywordKind::Defender));
}

// ---------------------------------------------------------------------------------------
// Blocking
// ---------------------------------------------------------------------------------------

#[test]
fn thieves_tools_checks_power_only_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Thieves' Tools",
        "The equipped creature's power is checked only during the declare blockers step. If its power becomes greater than 3 after blockers are declared, it still can't be blocked. Similarly, if its power becomes 3 or less after it's been blocked, this won't make it unblocked."
    );
    supported("Thieves' Tools");
    // "Equipped creature can't be blocked as long as its power is 3 or less."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Thieves' Tools", bears);
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(wall, bears)]));
    t.advance_to(P0, mtg_engine::turn::Step::DeclareBlockers);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(0))],
    );
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // Blocked with power 6, then shrunk to 2: still blocked.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Thieves' Tools", wurm);
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(wall, wurm)]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(wall, wurm)]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::DeclareBlockers);
    modify_until_eot(
        &mut t,
        wurm,
        vec![Modification::ModifyPT(Value::c(-4), Value::c(0))],
    );
    assert_eq!(blocks_now(&t), vec![(wall, wurm)]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn skyblinder_staff_doesnt_stop_reach() {
    cr!("509.1b", "702.17b");
    ruling!(
        "Skyblinder Staff",
        "The equipped creature can still be blocked by a creature with reach."
    );
    supported("Skyblinder Staff");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Skyblinder Staff", bears);
    let spider = t.battlefield(P1, "Giant Spider");
    let drake = t.battlefield(P1, "Wind Drake");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(spider, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(drake, bears)]));
}

#[test]
fn the_defending_player_chooses_which_required_blocker_blocks() {
    cr!("509.1c");
    ruling!(
        "Slayer's Cleaver",
        "The defending player, not the attacking player, chooses which Eldrazi blocks the equipped creature."
    );
    ruling!(
        "Ace's Baseball Bat",
        "The defending player, not the attacking player, chooses which Dalek blocks the equipped creature."
    );
    supported("Slayer's Cleaver");
    supported("Ace's Baseball Bat");
    for (equipment, blocker) in [
        ("Slayer's Cleaver", "Kozilek's Predator"),
        ("Ace's Baseball Bat", "Dalek Squadron"),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, equipment, bears);
        let a = t.battlefield(P1, blocker);
        let b = t.battlefield(P1, blocker);
        let other = t.battlefield(P1, "Wall of Wood");
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert!(legal_blocks(&mut t, P1, &[(a, bears)]), "{equipment}");
        assert!(legal_blocks(&mut t, P1, &[(b, bears)]), "{equipment}");
        assert!(!legal_blocks(&mut t, P1, &[]), "{equipment}");
        assert!(!legal_blocks(&mut t, P1, &[(other, bears)]), "{equipment}");
        block_and_finish(&mut t, P1, &[(b, bears)]);
        assert!(!t.on_battlefield(b) || t.obj_now(b).damage > 0, "{equipment}");
        assert!(t.on_battlefield(a), "{equipment}");
    }
}

// ---------------------------------------------------------------------------------------
// Equip abilities
// ---------------------------------------------------------------------------------------

#[test]
fn steelclaw_lance_checks_knight_only_as_equip_knight_is_activated_and_resolves() {
    cr!("702.6c", "608.2b", "301.5c");
    ruling!(
        "Steelclaw Lance",
        "Whether the target creature is a Knight is checked only as the equip Knight ability is activated and as that ability resolves. If the creature somehow becomes a non-Knight creature later, Steelclaw Lance remains attached to it."
    );
    supported("Steelclaw Lance");
    let mut t = TestGame::new(2);
    let lance = t.battlefield(P0, "Steelclaw Lance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let knight = t.battlefield(P0, "Knight of the White Orchid");
    t.lands(P0, "Wastes", 1);
    let options = ability_targets(&mut t, lance, 0);
    assert!(options.contains(&Entity::Object(knight)));
    assert!(!options.contains(&Entity::Object(bears)));
    equip_nth(&mut t, P0, lance, 0, knight).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, lance), Some(Entity::Object(knight)));
    modify_until_eot(&mut t, knight, vec![Modification::RemoveAllCreatureTypes]);
    assert!(!t.obj_now(knight).chars.has_subtype("Knight"));
    assert_eq!(attached_to(&t, lance), Some(Entity::Object(knight)));
    assert_eq!(t.pt(knight), (4, 4));
    // The Knight stops being one with the ability on the stack: it does nothing.
    let mut t = TestGame::new(2);
    let lance = t.battlefield(P0, "Steelclaw Lance");
    let knight = t.battlefield(P0, "Knight of the White Orchid");
    t.lands(P0, "Wastes", 1);
    equip_nth(&mut t, P0, lance, 0, knight).unwrap();
    modify_until_eot(&mut t, knight, vec![Modification::RemoveAllCreatureTypes]);
    t.resolve_all();
    assert_eq!(attached_to(&t, lance), None);
}

#[test]
fn lightning_greaves_shroud_stops_other_equip_abilities() {
    cr!("702.18a", "702.6a", "115.4");
    ruling!(
        "Lightning Greaves",
        "You can't simply unequip Equipment from a creature. If Lightning Greaves is attached to the only creature you control, you won't be able to attach other equipment to it"
    );
    supported("Lightning Greaves");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let greaves = attach_new(&mut t, P0, "Lightning Greaves", bears);
    let shuko = t.battlefield(P0, "Shuko");
    assert!(equip(&mut t, P0, shuko, bears).is_err());
    // The Greaves can't be moved off it either while it's the only creature (equipping
    // it targets the creature too).
    assert!(equip(&mut t, P0, greaves, bears).is_err());
    // With another creature, the Greaves can move, and then Shuko can be attached.
    let lion = t.battlefield(P0, "Silvercoat Lion");
    equip(&mut t, P0, greaves, lion).unwrap();
    t.resolve_all();
    equip(&mut t, P0, shuko, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, shuko), Some(Entity::Object(bears)));
}

#[test]
fn equipping_the_creature_its_already_attached_to() {
    cr!("701.3b", "702.6a", "702.6c");
    ruling!(
        "Grafted Wargear",
        "You may activate this card's equip ability targeting the creature that it is currently attached to, even if it's the only creature you control. Since the equip ability costs zero, you can do this as many times as you like during your main phases, though it generally won't do anything."
    );
    ruling!(
        "Shuko",
        "You may activate this card’s equip ability targeting the creature that it is currently attached to, even if it’s the only creature you control. Since the equip ability costs zero, you can do this as many times as you like during your main phases, though it generally won’t do anything."
    );
    for name in ["Grafted Wargear", "Shuko"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let eq = attach_new(&mut t, P0, name, bears);
        let before = t.pt(bears);
        for _ in 0..3 {
            equip(&mut t, P0, eq, bears).unwrap();
            t.resolve_all();
        }
        // Nothing happened: still attached, and Grafted Wargear never became unattached
        // (so the creature wasn't sacrificed).
        assert!(t.on_battlefield(bears), "{name}");
        assert_eq!(attached_to(&t, eq), Some(Entity::Object(bears)), "{name}");
        assert_eq!(t.pt(bears), before, "{name}");
    }
}
