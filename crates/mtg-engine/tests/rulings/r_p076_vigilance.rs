//! Rulings batch P076 — abilities that give vigilance (and other keywords): effects that
//! lock in the affected set as they resolve (CR 611.2c), keywords gained or lost after
//! attackers are declared, equipment that ends up unattached, and Couriers.

use crate::r_p076_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::has_kw;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A creature that enters after an "creatures you control get +1/+1 and gain vigilance
/// until end of turn" trigger resolved isn't affected (CR 611.2c).
fn enters_and_pumps_present_creatures(name: &str, self_pt: (i32, i32)) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let me = t.enter(P0, name);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    // Itself, too.
    assert_eq!(t.pt(me), self_pt);
    assert!(has_kw(&t, me, KeywordKind::Vigilance));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!has_kw(&t, late, KeywordKind::Vigilance));
}

#[test]
fn angel_of_the_dawn_affects_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Angel of the Dawn",
        "Angel of the Dawn's triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+1 or gain vigilance."
    );
    enters_and_pumps_present_creatures("Angel of the Dawn", (4, 4));
}

#[test]
fn dawnfeather_eagle_affects_itself() {
    cr!("611.2c");
    ruling!(
        "Dawnfeather Eagle",
        "Dawnfeather Eagle’s triggered ability affects itself."
    );
    enters_and_pumps_present_creatures("Dawnfeather Eagle", (4, 4));
}

#[test]
fn basri_ultimate_affects_creatures_you_control_as_it_resolves() {
    cr!("611.2c", "606.3");
    ruling!(
        "Basri, Devoted Paladin",
        "Basri's last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +2/+2 or gain flying."
    );
    supported("Basri, Devoted Paladin");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let basri = t.battlefield(P0, "Basri, Devoted Paladin");
    crate::r_s29_common::put_counters(&mut t, basri, "loyalty", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, basri, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    assert!(!has_kw(&t, late, KeywordKind::Flying));
}

#[test]
fn basri_minus_one_counter_arrives_before_blockers() {
    cr!("508.1m", "509.1");
    ruling!(
        "Basri, Devoted Paladin",
        "After activating Basri's second ability, a creature that attacks gets a +1/+1 counter before blockers are chosen."
    );
    supported("Basri, Devoted Paladin");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let basri = t.battlefield(P0, "Basri, Devoted Paladin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, basri, 1, &[]).unwrap();
    t.resolve_all();
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn brambleguard_veteran_affects_raccoons_as_it_resolves() {
    cr!("611.2c", "700.14");
    ruling!(
        "Brambleguard Veteran",
        "Brambleguard Veteran’s ability affects only Raccoons you control at the time it resolves, including Brambleguard Veteran itself (as long as it’s still a Raccoon at that time). Raccoons you begin to control later in the turn or creatures that become Raccoons later in the turn won’t be affected."
    );
    supported("Brambleguard Veteran");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vet = t.battlefield(P0, "Brambleguard Veteran");
    // Spending the fourth total mana on spells this turn: expend 4.
    cast_new(&mut t, P0, "Hill Giant", &[]);
    t.resolve_all();
    assert_eq!(t.pt(vet), (4, 5));
    assert!(has_kw(&t, vet, KeywordKind::Vigilance));
    let late = t.battlefield(P0, "Brambleguard Veteran");
    assert_eq!(t.pt(late), (3, 4));
    assert!(!has_kw(&t, late, KeywordKind::Vigilance));
}

#[test]
fn felidar_retreat_second_mode_affects_creatures_as_it_resolves() {
    cr!("611.2c", "700.2");
    ruling!(
        "Felidar Retreat",
        "Felidar Retreat's second mode affects only creatures you control at the time the ability resolves, including creatures you control but that for some reason didn't get a +1/+1 counter. Creatures you begin to control later in the turn won't gain vigilance or get a +1/+1 counter."
    );
    supported("Felidar Retreat");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felidar Retreat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Melira's Keepers can't have counters put on it."
    let keepers = t.battlefield(P0, "Melira's Keepers");
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    assert_eq!(t.counters(keepers, "+1/+1"), 0);
    assert!(has_kw(&t, keepers, KeywordKind::Vigilance));
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.counters(late, "+1/+1"), 0);
    assert!(!has_kw(&t, late, KeywordKind::Vigilance));
}

#[test]
fn keeper_of_keys_checks_the_monarch_at_upkeep_and_on_resolution() {
    cr!("603.4", "725.1");
    ruling!(
        "Keeper of Keys",
        "The last ability of Keeper of Keys checks to see if you're the monarch as your upkeep begins. If you're not, the ability won't trigger at all. You won't be able to do anything that would make you the monarch during your upkeep in time to have that ability trigger. The ability will also check to see if you're the monarch as it tries to resolve. If you're not the monarch at that time, the ability will have no effect."
    );
    supported("Keeper of Keys");
    use mtg_engine::designations::become_monarch;
    // Not the monarch as the upkeep begins: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keeper of Keys");
    become_monarch(&mut t.g, P1);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Becoming the monarch during the upkeep is too late.
    become_monarch(&mut t.g, P0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // The monarch as it begins, but not as it resolves: no effect.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keeper of Keys");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    become_monarch(&mut t.g, P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    become_monarch(&mut t.g, P1);
    t.resolve_all();
    crate::r_s01_common::attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn keeper_of_keys_affects_creatures_that_arrive_later() {
    cr!("611.2c");
    ruling!(
        "Keeper of Keys",
        "The last ability of Keeper of Keys will affect all creatures you control that turn, even if they weren't on the battlefield or weren't creatures as the ability resolved."
    );
    supported("Keeper of Keys");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keeper of Keys");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn hold_the_gates_gives_vigilance_with_no_gates() {
    cr!("613.1f");
    ruling!(
        "Hold the Gates",
        "Creatures you control will have vigilance even if you control no Gates."
    );
    supported("Hold the Gates");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hold the Gates");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    t.battlefield(P0, "Azorius Guildgate");
    assert_eq!(t.pt(bears), (2, 3));
}

#[test]
fn on_serras_wings_legend_rule_needs_two_legendary_permanents() {
    cr!("704.5j");
    ruling!(
        "On Serra's Wings",
        "If you control two permanents with the same name but only one is legendary, the \"legend rule\" doesn't apply."
    );
    supported("On Serra's Wings");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let wings = t.battlefield(P0, "On Serra's Wings");
    t.g.attach(wings, Entity::Object(a));
    t.g.recompute();
    t.settle();
    assert!(crate::r_s25_common::legendary(&t, a));
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn on_serras_wings_lifelink_twice_is_redundant() {
    cr!("702.15f", "702.20b");
    ruling!(
        "On Serra's Wings",
        "Multiple instances of flying, vigilance, and/or lifelink on the same creature are redundant."
    );
    supported("On Serra's Wings");
    let mut t = TestGame::new(2);
    // Vampire Nighthawk: 2/3 flying, deathtouch, lifelink.
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let wings = t.battlefield(P0, "On Serra's Wings");
    t.g.attach(wings, Entity::Object(hawk));
    t.g.recompute();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hawk, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23, "lifelink twice gains life only once");
    assert!(!t.obj_now(hawk).tapped, "vigilance");
}

#[test]
fn phyrexian_awakening_leaving_doesnt_tap_attackers() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Phyrexian Awakening",
        "Once a Phyrexian you control has legally attacked, causing it to lose vigilance by removing Phyrexian Awakening from the battlefield won't cause that Phyrexian to become tapped."
    );
    supported("Phyrexian Awakening");
    let mut t = TestGame::new(2);
    let awakening = t.battlefield(P0, "Phyrexian Awakening");
    // Vault Skirge: a Phyrexian Bat.
    let skirge = t.battlefield(P0, "Vault Skirge");
    crate::r_s01_common::attack_with(&mut t, &[(skirge, Entity::Player(P1))]);
    assert!(!t.obj_now(skirge).tapped);
    crate::r_s02_common::destroy(&mut t, awakening);
    assert!(!has_kw(&t, skirge, KeywordKind::Vigilance));
    assert!(!t.obj_now(skirge).tapped);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn sentinels_mark_after_attacking_doesnt_untap_or_tap() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Sentinel's Mark",
        "Gaining vigilance any time after the moment you choose to attack with a creature won't cause it to become untapped, and losing vigilance after that time won't cause it to become tapped."
    );
    supported("Sentinel's Mark");
    // Gaining vigilance after attacking: still tapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.obj_now(bears).tapped);
    cast_new(&mut t, P0, "Sentinel's Mark", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    assert!(t.obj_now(bears).tapped);
    // Losing vigilance after attacking: still untapped.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mark = t.battlefield(P0, "Sentinel's Mark");
    t.g.attach(mark, Entity::Object(bears));
    t.g.recompute();
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!t.obj_now(bears).tapped);
    crate::r_s02_common::destroy(&mut t, mark);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn haunted_cloak_moved_away_before_attacking() {
    cr!("302.6", "702.10b");
    ruling!(
        "Haunted Cloak",
        "If a creature enters the battlefield under your control and gains haste, but then loses it before attacking, it won't be able to attack that turn. This means that you can't use one Haunted Cloak to allow two new creatures to attack in the same turn."
    );
    supported("Haunted Cloak");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cloak = t.battlefield(P0, "Haunted Cloak");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Hill Giant");
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, cloak, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, a));
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, cloak, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, a));
    assert!(crate::r_s02_common::can_attack(&mut t, b));
}

#[test]
fn kamahl_animated_land_keeps_its_types_and_abilities() {
    cr!("613.1d", "205.1b");
    ruling!(
        "Kamahl, Heart of Krosa",
        "A land that becomes a creature because of Kamahl's activated ability will retain any other supertypes, card types, subtypes, and abilities it had."
    );
    supported("Kamahl, Heart of Krosa");
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Heart of Krosa");
    let cradle = t.battlefield(P0, "Gaea's Cradle");
    let forest = t.battlefield(P0, "Forest");
    mana(&mut t, P0, ManaType::G, 2);
    t.activate(P0, kamahl, 0, &[Entity::Object(cradle)]).unwrap();
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    t.activate(P0, kamahl, 0, &[Entity::Object(forest)]).unwrap();
    t.resolve_all();
    let c = &t.obj_now(cradle).chars;
    assert!(c.card_types.contains(CardType::Land) && c.card_types.contains(CardType::Creature));
    assert!(c.supertypes.contains(Supertype::Legendary));
    assert!(c.has_subtype("Elemental"));
    assert!(!c.abilities.is_empty());
    assert_eq!(t.pt(cradle), (1, 1));
    assert!(has_kw(&t, cradle, KeywordKind::Vigilance));
    let f = &t.obj_now(forest).chars;
    assert!(f.has_subtype("Forest") && f.supertypes.contains(Supertype::Basic));
}

/// A Courier's "+2/+2 and has [keyword] for as long as this creature remains tapped"
/// checks the creature type as the ability is activated and as it resolves, but once
/// applied it lasts even if the creature's type changes.
fn courier(name: &str, target: &str, color: ManaType, kw: KeywordKind) {
    supported(name);
    let base = mtg_engine::card::card(target).front().chars.power.unwrap();
    // The type changes before the ability resolves: illegal target, no effect.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    let x = t.battlefield(P0, target);
    mana(&mut t, P0, color, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, c, 0, &[Entity::Object(x)]).unwrap();
    crate::r_s26_common::modify_until_eot(&mut t, x, vec![Modification::RemoveAllCreatureTypes]);
    t.resolve_all();
    assert_eq!(t.pt(x).0, base, "{name}: resolved with an illegal target");
    // The type changes after it resolved: the effect continues.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    let x = t.battlefield(P0, target);
    mana(&mut t, P0, color, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, c, 0, &[Entity::Object(x)]).unwrap();
    t.resolve_all();
    crate::r_s26_common::modify_until_eot(&mut t, x, vec![Modification::RemoveAllCreatureTypes]);
    assert_eq!(t.pt(x).0, base + 2, "{name}");
    assert!(has_kw(&t, x, kw), "{name}");
    // Until the Courier untaps.
    t.g.untap(t.g.current(c));
    t.g.recompute();
    assert_eq!(t.pt(x).0, base, "{name}");
}

#[test]
fn couriers_check_the_creature_type_only_until_the_effect_applies() {
    cr!("608.2b", "611.2b");
    ruling!(
        "Everglove Courier",
        "It checks the creature type when the ability is announced and resolved, but once the effect is placed on the creature, if its creature type changes the effect still continues."
    );
    ruling!(
        "Flamestick Courier",
        "It checks the creature type when the ability is announced and resolved, but once the effect is placed on the creature, if its creature type changes the effect still continues."
    );
    ruling!(
        "Frightshroud Courier",
        "It checks the creature type when the ability is announced and resolved, but once the effect is placed on the creature, if its creature type changes the effect still continues."
    );
    ruling!(
        "Ghosthelm Courier",
        "It checks the creature type when the ability is announced and resolved, but once the effect is placed on the creature, if its creature type changes the effect still continues."
    );
    ruling!(
        "Pearlspear Courier",
        "It checks the creature type when the ability is announced and resolved, but once the effect is placed on the creature, if its creature type changes the effect still continues."
    );
    courier("Everglove Courier", "Llanowar Elves", ManaType::G, KeywordKind::Trample);
    courier("Flamestick Courier", "Raging Goblin", ManaType::R, KeywordKind::Haste);
    courier("Frightshroud Courier", "Scathe Zombies", ManaType::B, KeywordKind::Fear);
    courier("Ghosthelm Courier", "Prodigal Sorcerer", ManaType::U, KeywordKind::Shroud);
    courier("Pearlspear Courier", "Elite Vanguard", ManaType::W, KeywordKind::Vigilance);
}

#[test]
fn jetmir_losing_double_strike_between_damage_steps() {
    cr!("702.4c", "702.4d", "510.4");
    ruling!(
        "Jetmir, Nexus of Revels",
        "If a creature with double strike loses double strike after dealing damage during the first combat damage step but before dealing damage in the second combat damage step, it will not deal damage during that second combat damage step. Notably, this means that if your ninth creature dies in the first combat damage step, the rest of your creatures won't deal combat damage again unless something else is granting them double strike."
    );
    supported("Jetmir, Nexus of Revels");
    let mut t = TestGame::new(2);
    let jetmir = t.battlefield(P0, "Jetmir, Nexus of Revels");
    let elves = t.battlefield(P0, "Llanowar Elves");
    for _ in 0..7 {
        t.battlefield(P0, "Grizzly Bears");
    }
    // White Knight: 2/2 first strike.
    let knight = t.battlefield(P1, "White Knight");
    assert_eq!(t.pt(jetmir), (8, 4));
    assert!(has_kw(&t, jetmir, KeywordKind::DoubleStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(jetmir, Entity::Player(P1)), (elves, Entity::Player(P1))],
        &[(knight, elves)],
    );
    // The Elves (4/1 double strike, trample) died in the first-strike step, trampling
    // over 2: 8 creatures left, no double strike, so Jetmir dealt its 8 damage only once.
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    let from_jetmir = crate::r_s30_common::damage_events(&t)
        .iter()
        .filter(|d| d.0 == jetmir)
        .count();
    assert_eq!(from_jetmir, 1);
    assert_eq!(t.life(P1), 10);
}

#[test]
fn shield_of_the_righteous_has_no_effect_on_an_untapped_creature() {
    cr!("502.3", "611.2a");
    ruling!(
        "Shield of the Righteous",
        "If the blocked creature is already untapped at the time its controller’s next untap step begins, this ability has no effect. It won’t apply at some later time when that creature is tapped."
    );
    supported("Shield of the Righteous");
    let mut t = TestGame::new(2);
    // Serra Angel: vigilance, so it stays untapped while attacking.
    let angel = t.battlefield(P0, "Serra Angel");
    let wall = t.battlefield(P1, "Wall of Stone");
    let shield = t.battlefield(P1, "Shield of the Righteous");
    t.g.attach(shield, Entity::Object(wall));
    t.g.recompute();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(angel, Entity::Player(P1))], &[(wall, angel)]);
    t.resolve_all();
    assert!(!t.obj_now(angel).tapped);
    t.advance_to(P0, Step::Upkeep);
    // Tapped after that untap step: it untaps normally in the next one.
    t.g.tap(t.g.current(angel));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(angel).tapped);
}

#[test]
fn shining_armor_with_no_knight_stays_unattached() {
    cr!("603.3d", "301.5");
    ruling!(
        "Shining Armor",
        "If there are no Knights to attach Shining Armor to when it enters the battlefield, it simply remains unattached."
    );
    supported("Shining Armor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let armor = t.enter(P0, "Shining Armor");
    t.resolve_all();
    assert!(t.on_battlefield(armor));
    assert_eq!(t.obj_now(armor).attached_to, None);
}

#[test]
fn forebears_blade_with_no_target_stays_unattached() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Forebear's Blade",
        "If there's no target for the triggered ability of Forebear's Blade, or if the ability's target becomes illegal, Forebear's Blade remains on the battlefield unattached."
    );
    supported("Forebear's Blade");
    // No other creature: no target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Forebear's Blade");
    t.g.attach(blade, Entity::Object(bears));
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None);
    // The target becomes illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let blade = t.battlefield(P0, "Forebear's Blade");
    t.g.attach(blade, Entity::Object(bears));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    crate::r_s02_common::destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None);
}

#[test]
fn commander_mustard_twice_gives_two_instances() {
    cr!("113.2c", "603.2");
    ruling!(
        "Commander Mustard",
        "Activating Commander Mustard's last ability multiple times will cause Soldiers you control to gain multiple instances of the triggered ability. For example, if you activate Commander Mustard's last ability twice and then attack with Commander Mustard, it will have two instances of the triggered ability. Both will trigger, and the defending player will take 2 damage."
    );
    supported("Commander Mustard");
    let mut t = TestGame::new(2);
    let mustard = t.battlefield(P0, "Commander Mustard");
    for _ in 0..2 {
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::W, 1);
        mana(&mut t, P0, ManaType::C, 2);
        t.activate(P0, mustard, 0, &[]).unwrap();
        t.resolve_all();
    }
    crate::r_s01_common::attack_with(&mut t, &[(mustard, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn vihaan_overwrites_set_pt_but_keeps_modifications() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Vihaan, Goldwaker",
        "If a Treasure you control is already a creature when Vihaan’s second ability resolves and you choose to apply its effect to all Treasures you control, that effect will overwrite any previous effects that set that creature’s power and toughness to specific numbers. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Vihaan, Goldwaker");
    use mtg_engine::ability::Value;
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vihaan, Goldwaker");
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        treasure,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    crate::r_s26_common::modify_until_eot(
        &mut t,
        treasure,
        vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
    );
    crate::r_s29_common::put_counters(&mut t, treasure, "+1/+1", 1);
    assert_eq!(t.pt(treasure), (3, 3));
    t.answer_yes(P0, true);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(treasure), (5, 5));
}

#[test]
fn loyal_unicorn_prevents_damage_to_creatures_that_arrive_later() {
    cr!("611.2c", "615.1");
    ruling!(
        "Loyal Unicorn",
        "Loyal Unicorn’s effect will prevent all combat damage that would be dealt to creatures you control, even if those creatures weren’t on the battlefield or weren’t creatures when the effect resolved."
    );
    supported("Loyal Unicorn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loyal Unicorn");
    let cmdr = t.battlefield(P0, "Elite Vanguard");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    let name = t.obj(cmdr).card.as_ref().unwrap().name.clone();
    t.g.players[0].commander_names.push(name);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.attack(&[(late, Entity::Player(P1))], &[(giant, late)]);
    assert!(t.on_battlefield(late));
    assert_eq!(t.obj_now(late).damage, 0);
}
