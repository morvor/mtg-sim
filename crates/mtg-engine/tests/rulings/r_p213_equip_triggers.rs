//! Rulings batch P213 — Equipment (CR 301.5, 702.6): triggered and activated abilities of
//! Equipment and of equipped creatures — which creature "equipped creature" means as the
//! ability resolves (CR 608.2h), last known information (CR 603.10a, 608.2h), combat
//! triggers, and abilities that keep working after the Equipment moves or leaves.

use crate::r_p213_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::*;
use crate::r_s09_common::legal_attack;
use crate::r_s17_common::transform;
use crate::r_s21_common::castable;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves the Equipment (already on the battlefield) onto `to`.
fn move_equipment(t: &mut TestGame, eq: ObjectId, to: ObjectId) {
    let eq = t.g.current(eq);
    assert!(t.g.attach(eq, Entity::Object(t.g.current(to))));
    t.g.recompute();
}

// ---------------------------------------------------------------------------------------
// "Equipped creature" as the ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn heavy_arbalest_affects_the_creature_equipped_as_the_untap_step_begins() {
    cr!("502.3", "613.1f");
    ruling!(
        "Heavy Arbalest",
        "The creature that doesn't untap during its controller's untap step is the one that Heavy Arbalest is equipped to as that untap step begins. If a creature was tapped to activate the ability granted to it by the Arbalest, but the Arbalest is no longer equipping it, it will untap as normal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let arbalest = attach_new(&mut t, P0, "Heavy Arbalest", bears);
    t.activate(P0, bears, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let lion_id = t.g.current(lion);
    t.g.tap(lion_id);
    move_equipment(&mut t, arbalest, lion);
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(lion).tapped);
}

#[test]
fn counters_and_bonuses_go_to_the_creature_equipped_on_resolution() {
    cr!("608.2h", "611.2c", "301.5");
    ruling!(
        "Blade of the Bloodchief",
        "The creature that gets the counters is the one that's equipped by Blade of the Bloodchief at the time the ability resolves. It doesn't matter what creature Blade of the Bloodchief was attached to (if any) when it triggered."
    );
    ruling!(
        "Adventuring Gear",
        "The landfall ability gives +2/+2 to the creature that's equipped by Adventuring Gear at the time that ability resolves. It doesn't matter what creature Adventuring Gear was attached to (if any) when it triggered. That bonus remains with that creature even if Adventuring Gear leaves the battlefield or becomes attached to a different creature later in the turn."
    );
    supported("Blade of the Bloodchief");
    supported("Adventuring Gear");
    // Blade of the Bloodchief: "Whenever a creature dies, put a +1/+1 counter on equipped
    // creature. If equipped creature is a Vampire, put two +1/+1 counters on it instead."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let blade = attach_new(&mut t, P0, "Blade of the Bloodchief", bears);
    let elves = t.battlefield(P1, "Llanowar Elves");
    destroy(&mut t, elves);
    assert_eq!(t.stack_len(), 1);
    move_equipment(&mut t, blade, hawk);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert_eq!(t.counters(hawk, counters::PLUS1), 2);
    // Adventuring Gear: "Landfall — Whenever a land you control enters, equipped creature
    // gets +2/+2 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let gear = attach_new(&mut t, P0, "Adventuring Gear", bears);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_equipment(&mut t, gear, lion);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(lion), (4, 4));
    // The bonus stays with the Lion after the Gear moves back and then leaves.
    move_equipment(&mut t, gear, bears);
    assert_eq!(t.pt(lion), (4, 4));
    destroy(&mut t, gear);
    assert_eq!(t.pt(lion), (4, 4));
}

#[test]
fn mage_slayer_deals_damage_even_if_it_has_left() {
    cr!("608.2h", "603.4", "506.4");
    ruling!(
        "Mage Slayer",
        "When the equipped creature attacks, the first ability will trigger. When that ability resolves, that creature will deal damage equal to its power to player or planeswalker it's attacking, even if Mage Slayer has left the battlefield or is equipping a different creature by that time."
    );
    supported("Mage Slayer");
    // "Whenever equipped creature attacks, it deals damage equal to its power to the
    // player or planeswalker it's attacking."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let slayer = attach_new(&mut t, P0, "Mage Slayer", giant);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, slayer);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------------
// Last known information and values on resolution
// ---------------------------------------------------------------------------------------

#[test]
fn avacyns_collar_checks_whether_it_was_a_human_on_the_battlefield() {
    cr!("603.10a", "603.4", "608.2h");
    ruling!(
        "Avacyn's Collar",
        "The triggered ability checks whether the equipped creature as it last existed on the battlefield was a Human. (It doesn’t matter what its creature types are in the graveyard.)"
    );
    supported("Avacyn's Collar");
    // "Whenever equipped creature dies, if it was a Human, create a 1/1 white Spirit
    // creature token with flying." Grizzly Bears is a Human only on the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Avacyn's Collar", bears);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddSubtypes(vec!["Human".into()])],
    );
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
    // Not a Human: no Spirit.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Avacyn's Collar", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
}

#[test]
fn mask_of_griselbrand_uses_the_power_it_last_had() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Mask of Griselbrand",
        "Use the equipped creature's power as it last existed on the battlefield to determine the value of X."
    );
    supported("Mask of Griselbrand");
    // "Whenever equipped creature dies, you may pay X life, where X is its power. If you
    // do, draw X cards." Hill Giant pumped to 5 power dies.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Mask of Griselbrand", giant);
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.hand_size(P0), hand + 5);
}

#[test]
fn fiendlash_uses_the_power_on_resolution_or_as_it_last_existed() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Fiendlash",
        "Use the equipped creature's power as the triggered ability resolves to determine how much damage is dealt. If the equipped creature is no longer on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    supported("Fiendlash");
    // "Whenever equipped creature is dealt damage, it deals damage equal to its power to
    // target player or planeswalker." Equipped Hill Giant: 5/3.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Fiendlash", giant);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, elves, 1, giant);
    assert_eq!(t.stack_len(), 1);
    // Pumped before the ability resolves: 8 damage.
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(3), Value::c(0))],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Dealt lethal damage: it's gone, its last known power (5) is used.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Fiendlash", giant);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, elves, 3, giant);
    assert!(!t.on_battlefield(giant));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn dragon_throne_of_tarkir_determines_x_on_resolution() {
    cr!("603.10a", "608.2h", "107.3a");
    ruling!(
        "Dragon Throne of Tarkir",
        "The value of X is determined as the activated ability resolves. If the equipped creature isn't on the battlefield at that time, use its power the last time it was on the battlefield to determine the value of X."
    );
    supported("Dragon Throne of Tarkir");
    // Equipped creature has defender and "{2}, {T}: Other creatures you control gain
    // trample and get +X/+X until end of turn, where X is this creature's power."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Dragon Throne of Tarkir", giant);
    assert!(has_kw(&t, giant, KeywordKind::Defender));
    t.lands(P0, "Wastes", 2);
    t.activate(P0, giant, 0, &[]).unwrap();
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    // The creature leaves in response: its last known power.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Dragon Throne of Tarkir", giant);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, giant, 0, &[]).unwrap();
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn shield_of_the_avatar_counts_creatures_as_damage_would_be_dealt() {
    cr!("615.1", "615.7");
    ruling!(
        "Shield of the Avatar",
        "To determine how much damage to prevent, use the number of creatures you control when damage would be dealt, including the equipped creature. Notably, if damage would be dealt to multiple creatures at the same time, you’ll count the number of creatures you control before any of that damage is dealt."
    );
    supported("Shield of the Avatar");
    // "If a source would deal damage to equipped creature, prevent X of that damage,
    // where X is the number of creatures you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Shield of the Avatar", bears);
    t.battlefield(P0, "Llanowar Elves");
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(marked(&t, bears), 1);
    // Pyroclasm deals 2 damage to each creature at once: two creatures are counted,
    // although the Elves die from it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Shield of the Avatar", bears);
    t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 2);
    let clasm = t.hand(P0, "Pyroclasm");
    t.cast(P0, clasm).go();
    t.resolve_all();
    assert_eq!(marked(&t, bears), 0);
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn unscythe_checks_the_creature_equipped_now() {
    cr!("603.10a", "603.2");
    ruling!(
        "Unscythe, Killer of Kings",
        "Whenever a creature is put into a graveyard from the battlefield, check whether that creature had been dealt damage this turn by the creature that Unscythe is currently attached to. If so, Unscythe’s second ability will trigger. This is true even if Unscythe wasn’t attached to the creature at the time it dealt that damage."
    );
    supported("Unscythe, Killer of Kings");
    // "Whenever a creature dealt damage by equipped creature this turn dies, you may
    // exile that card. If you do, create a 2/2 black Zombie creature token."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let unscythe = attach_new(&mut t, P0, "Unscythe, Killer of Kings", lion);
    let wall = t.battlefield(P1, "Wall of Wood");
    // The Bears damages the Wall while Unscythe is on the Lion.
    damage(&mut t, bears, 1, wall);
    move_equipment(&mut t, unscythe, bears);
    t.answer_yes(P0, true);
    destroy(&mut t, wall);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    assert!(t.in_exile("Wall of Wood"));
    // Only the creature equipped now counts: the Lion (no longer equipped) damaged the
    // Elves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let unscythe = attach_new(&mut t, P0, "Unscythe, Killer of Kings", lion);
    let wall = t.battlefield(P1, "Wall of Wood");
    damage(&mut t, lion, 1, wall);
    move_equipment(&mut t, unscythe, bears);
    t.answer_yes(P0, true);
    destroy(&mut t, wall);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
    assert!(t.in_graveyard(P1, "Wall of Wood"));
}

// ---------------------------------------------------------------------------------------
// Combat triggers
// ---------------------------------------------------------------------------------------

#[test]
fn giants_skewer_triggers_even_if_either_creature_survives_or_dies() {
    cr!("510.2", "603.10a", "120.3");
    ruling!(
        "Giant's Skewer",
        "The triggered ability of Giant's Skewer triggers even if the equipped creature died, and even if the creature that was dealt damage didn't die."
    );
    supported("Giant's Skewer");
    // "Whenever equipped creature deals combat damage to a creature, create a Food
    // token." Equipped Grizzly Bears (4/3) is blocked by Gigantosaurus: it dies, the
    // Gigantosaurus doesn't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Giant's Skewer", bears);
    let dino = t.battlefield(P1, "Gigantosaurus");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(dino, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(dino));
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
}

#[test]
fn tormentors_helm_triggers_once_however_many_blockers() {
    cr!("509.3c", "603.2");
    ruling!(
        "Tormentor's Helm",
        "The triggered ability triggers only once if the equipped creature becomes blocked, no matter how many creatures blocked it."
    );
    supported("Tormentor's Helm");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Tormentor's Helm", giant);
    let a = t.battlefield(P1, "Wall of Wood");
    let b = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, giant), (b, giant)]);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn reapers_talisman_attacking_alone_triggers_both_abilities() {
    cr!("506.5", "603.2");
    ruling!(
        "Reaper's Talisman",
        "When the equipped creature attacks alone, it causes both abilities to trigger."
    );
    supported("Reaper's Talisman");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Reaper's Talisman", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Deathtouch));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn shield_of_the_righteous_doesnt_tap_the_blocked_creature() {
    cr!("502.3", "509.1i");
    ruling!(
        "Shield of the Righteous",
        "When the equipped creature blocks a creature, Shield of the Righteous’s second ability doesn’t tap the blocked creature"
    );
    supported("Shield of the Righteous");
    // "Whenever equipped creature blocks a creature, that creature doesn't untap during
    // its controller's next untap step." A creature with vigilance attacks P0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Shield of the Righteous", bears);
    let sentry = custom_card("Sentry", "Creature — Soldier", "{2}", Some((1, 3)), "Vigilance");
    let angel = t.custom(P1, sentry, mtg_engine::object::Zone::Battlefield);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(angel, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[(bears, angel)]);
    assert!(!t.obj_now(angel).tapped);
    // Tapped later, it stays tapped through P1's next untap step.
    let angel_id = t.g.current(angel);
    t.g.tap(angel_id);
    next_upkeep(&mut t, P1);
    assert!(t.obj_now(angel).tapped);
}

#[test]
fn tokens_entering_attacking_may_attack_another_opponent() {
    cr!("508.4", "506.2");
    ruling!(
        "Captain's Claws",
        "You choose which opponent or planeswalker controlled by an opponent the token is attacking as it enters the battlefield. It doesn’t have to be the same opponent or planeswalker that the equipped creature is attacking."
    );
    ruling!(
        "Sigiled Sword of Valeron",
        "You choose which player or planeswalker the token is attacking. It doesn't have to be attacking the same player or planeswalker that the equipped creature is attacking."
    );
    for (name, token) in [
        ("Captain's Claws", "Kor"),
        ("Sigiled Sword of Valeron", "Knight"),
    ] {
        supported(name);
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, name, bears);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.resolve_all();
        let toks: Vec<ObjectId> = with_subtype(&t, P0, token)
            .into_iter()
            .filter(|id| *id != t.g.current(bears))
            .collect();
        assert_eq!(toks.len(), 1, "{name}");
        let target = t.g.combat.as_ref().and_then(|c| c.attack_target(toks[0]));
        assert_eq!(target, Some(Entity::Player(P2)), "{name}");
    }
}

#[test]
fn tormentors_trident_you_choose_whom_it_attacks() {
    cr!("508.1d", "506.2");
    ruling!(
        "Tormentor's Trident",
        "You still decide which player or planeswalker the equipped creature attacks."
    );
    supported("Tormentor's Trident");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Tormentor's Trident", bears);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
}

#[test]
fn worldslayer_destroys_the_equipped_creature_too() {
    cr!("701.8a", "510.2");
    ruling!("Worldslayer", "The equipped creature is also destroyed.");
    supported("Worldslayer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let slayer = attach_new(&mut t, P0, "Worldslayer", bears);
    let forest = t.battlefield(P0, "Forest");
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(forest));
    assert!(!t.on_battlefield(wall));
    assert!(t.on_battlefield(slayer));
}

#[test]
fn holy_avenger_puts_an_aura_only_if_it_could_be_attached() {
    cr!("303.4a", "303.4g", "702.4b");
    ruling!(
        "Holy Avenger",
        "You may put an Aura card onto the battlefield attached to the equipped creature if it could legally be attached to that creature."
    );
    supported("Holy Avenger");
    // "Whenever equipped creature deals combat damage, you may put an Aura card from your
    // hand onto the battlefield attached to it." (Double strike: two triggers.)
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Avenger", bears);
    let strength = t.hand(P0, "Holy Strength");
    let growth = t.hand(P0, "Wild Growth");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(strength)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(Entity::Object(bears)));
    // Wild Growth (enchant land) can't be attached to the creature.
    assert!(t.in_hand(P0, "Wild Growth"));
    let _ = growth;
}

// ---------------------------------------------------------------------------------------
// Other rulings
// ---------------------------------------------------------------------------------------

#[test]
fn ronin_warclub_doesnt_attach_to_a_creature_that_cant_be_equipped() {
    cr!("301.5c", "701.3b");
    ruling!(
        "Ronin Warclub",
        "The triggered ability won't attach Ronin Warclub to a creature that can't be equipped (such as Goblin Brawler)."
    );
    supported("Ronin Warclub");
    supported("Goblin Brawler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let club = attach_new(&mut t, P0, "Ronin Warclub", bears);
    enter_creature(&mut t, P0, "Goblin Brawler");
    t.resolve_all();
    assert_eq!(attached_to(&t, club), Some(Entity::Object(bears)));
    let lion = enter_creature(&mut t, P0, "Silvercoat Lion");
    t.resolve_all();
    assert_eq!(attached_to(&t, club), Some(Entity::Object(lion)));
}

fn enter_creature(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

#[test]
fn neglected_heirloom_transforms_with_the_creature() {
    cr!("712.8", "301.5c", "704.5n");
    ruling!(
        "Neglected Heirloom // Ashmouth Blade",
        "When Neglected Heirloom transforms, it remains attached to the creature it's attached to. If the equipped creature transforms into a noncreature permanent, Neglected Heirloom will become unattached before it transforms into Ashmouth Blade."
    );
    supported("Neglected Heirloom // Ashmouth Blade");
    supported("Delver of Secrets // Insectile Aberration");
    // "When equipped creature transforms, transform this Equipment."
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
    let heirloom = attach_new(&mut t, P0, "Neglected Heirloom // Ashmouth Blade", delver);
    transform(&mut t, delver);
    t.resolve_all();
    assert_eq!(t.obj_now(heirloom).chars.name.as_str(), "Ashmouth Blade");
    assert_eq!(attached_to(&t, heirloom), Some(Entity::Object(delver)));
    // Insectile Aberration 3/2, +3/+3 and first strike.
    assert_eq!(t.pt(delver), (6, 5));
    assert!(has_kw(&t, delver, KeywordKind::FirstStrike));
    // Kytheon transforms into a planeswalker: the Heirloom becomes unattached, then still
    // transforms.
    // (Only its transforming matters here, not its own abilities.)
    let name = "Kytheon, Hero of Akros // Gideon, Battle-Forged";
    let mut t = TestGame::new(2);
    let kytheon = t.battlefield(P0, name);
    let heirloom = attach_new(&mut t, P0, "Neglected Heirloom // Ashmouth Blade", kytheon);
    transform(&mut t, kytheon);
    assert_eq!(attached_to(&t, heirloom), None);
    t.resolve_all();
    assert_eq!(t.obj_now(heirloom).chars.name.as_str(), "Ashmouth Blade");
    assert_eq!(attached_to(&t, heirloom), None);
}

#[test]
fn dream_thiefs_bandana_cards_stay_playable_by_you_only() {
    cr!("406.6", "601.3", "609.4b");
    ruling!(
        "Dream-Thief's Bandana",
        "You may look at and play cards exiled face down with Dream-Thief's Bandana's first ability (and spend mana of any type to do so) even if Dream-Thief's Bandana and/or the equipped creature leaves the battlefield. If another player gains control of Dream-Thief's Bandana and/or the equipped creature, that player can't look at or play cards already exiled with Dream-Thief's Bandana, and you still can."
    );
    supported("Dream-Thief's Bandana");
    // "Whenever equipped creature deals combat damage to a player, look at the top card of
    // their library, then exile it face down. For as long as it remains exiled, you may
    // play it, and mana of any type can be spent to cast that spell."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bandana = attach_new(&mut t, P0, "Dream-Thief's Bandana", bears);
    let bolt = t.library_top(P1, "Lightning Bolt");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.zone(bolt), mtg_engine::object::Zone::Exile);
    t.advance_to(P0, Step::PostcombatMain);
    give_control(&mut t, bandana, P1);
    give_control(&mut t, bears, P1);
    destroy(&mut t, bandana);
    t.lands(P0, "Island", 1);
    t.lands(P1, "Mountain", 1);
    assert!(!castable(&mut t, P1, bolt));
    assert!(castable(&mut t, P0, bolt));
    let bolt = t.g.current(bolt);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18 - 3);
}

#[test]
fn assault_suit_the_creature_cant_be_sacrificed() {
    cr!("701.21a", "118.3");
    ruling!(
        "Assault Suit",
        "The equipped creature can't be sacrificed for any reason. If an effect instructs you to sacrifice it, you can't and it remains on the battlefield. You also can't sacrifice it to pay a cost that requires you to sacrifice a creature."
    );
    supported("Assault Suit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Assault Suit", bears);
    t.lands(P1, "Swamp", 2);
    let edict = t.hand(P1, "Diabolic Edict");
    t.cast(P1, edict).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Bone Splinters ("As an additional cost to cast this spell, sacrifice a creature.")
    // can't be cast with only the equipped creature.
    t.lands(P0, "Swamp", 1);
    t.battlefield(P1, "Hill Giant");
    let splinters = t.hand(P0, "Bone Splinters");
    assert!(!castable(&mut t, P0, splinters));
}

#[test]
fn sacrificing_to_pay_an_equip_cost() {
    cr!("702.6a", "601.2h", "701.3b", "608.2b");
    ruling!(
        "Piston Sledge",
        "You can sacrifice Piston Sledge to pay its own equip cost. It won't become attached as it will be in your graveyard."
    );
    ruling!(
        "Dissection Tools",
        "You can sacrifice any creature you control to pay Dissection Tools's equip cost, including the creature Dissection Tools is currently equipping."
    );
    ruling!(
        "Demonmail Hauberk",
        "You can sacrifice the creature Demonmail Hauberk is equipping in order to equip it to another creature."
    );
    supported("Piston Sledge");
    // Piston Sledge: "Equip—Sacrifice an artifact."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sledge = t.battlefield(P0, "Piston Sledge");
    t.answer_choose(P0, &[Entity::Object(sledge)]);
    equip(&mut t, P0, sledge, bears).unwrap();
    assert!(t.in_graveyard(P0, "Piston Sledge"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Piston Sledge"));
    assert_eq!(t.pt(bears), (2, 2));
    // Dissection Tools and Demonmail Hauberk: "Equip—Sacrifice a creature." Sacrificing
    // the creature it's equipping.
    for name in ["Dissection Tools", "Demonmail Hauberk"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lion = t.battlefield(P0, "Silvercoat Lion");
        let eq = attach_new(&mut t, P0, name, bears);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        equip(&mut t, P0, eq, lion).unwrap();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        t.resolve_all();
        assert_eq!(attached_to(&t, eq), Some(Entity::Object(lion)), "{name}");
    }
}
