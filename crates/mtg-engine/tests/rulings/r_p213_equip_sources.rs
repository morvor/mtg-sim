//! Rulings batch P213 — Equipment (CR 301.5, 702.6): which object is the source of an
//! ability an Equipment grants and of its damage (CR 113.7, 115.4, 702.16), "unattach"
//! costs, and who may activate granted abilities.

use crate::r_p213_common::*;
use crate::r_s01_common::*;
use crate::r_s04_common::{ability_targets, spell_targets};
use crate::r_s02_common::destroy;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn heavy_arbalest_the_equipped_creature_is_the_source_of_the_damage() {
    cr!("113.7", "115.4", "702.16b", "702.16e");
    ruling!(
        "Heavy Arbalest",
        "The equipped creature is the source of both the damage ability and the resultant damage, not Heavy Arbalest."
    );
    supported("Heavy Arbalest");
    // Green Grizzly Bears: "{T}: This creature deals 2 damage to any target."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Heavy Arbalest", bears);
    let cat = t.battlefield(P1, "Nacatl Savage");
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    let targets = ability_targets(&mut t, bears, 0);
    assert!(targets.contains(&Entity::Object(cat)));
    assert!(!targets.contains(&Entity::Object(zombie)));
    t.activate(P0, bears, 0, &[Entity::Object(cat)]).unwrap();
    t.resolve_all();
    // Protection from artifacts doesn't prevent the (green, nonartifact) Bears' damage.
    assert!(t.in_graveyard(P1, "Nacatl Savage"));
}

#[test]
fn livewire_lash_the_creature_is_the_source_and_its_trigger_resolves_first() {
    cr!("113.7", "115.4", "603.3", "702.16e");
    ruling!(
        "Livewire Lash",
        "The equipped creature is the source of both the triggered ability and the resultant damage, not Livewire Lash. For example, you can target and deal damage to a creature with protection from artifacts"
    );
    ruling!(
        "Livewire Lash",
        "When the equipped creature becomes the target of a spell, the ability granted to it by Livewire Lash will trigger and go on the stack on top of it. It will resolve before the spell does."
    );
    supported("Livewire Lash");
    // "Whenever this creature becomes the target of a spell, this creature deals 2 damage
    // to any target."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Livewire Lash", bears);
    assert_eq!(t.pt(bears), (4, 2));
    let cat = t.battlefield(P1, "Nacatl Savage");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.answer_targets(P0, &[Entity::Object(cat)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger is on top: it resolves while Giant Growth is still on the stack.
    t.resolve();
    assert!(t.in_graveyard(P1, "Nacatl Savage"));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(bears), (4, 2));
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 5));
}

#[test]
fn mask_of_immolation_the_sacrificed_creature_deals_the_damage() {
    cr!("113.7", "115.4", "702.16b", "702.16e", "608.2h");
    ruling!(
        "Mask of Immolation",
        "This means that a green creature equipped by Mask of Immolation may deal damage to a creature with protection from red but may not deal damage to a creature with protection from green."
    );
    supported("Mask of Immolation");
    // "Sacrifice this creature: It deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Mask of Immolation", bears);
    let oraxid = t.battlefield(P1, "Oraxid");
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    let targets = ability_targets(&mut t, bears, 0);
    assert!(targets.contains(&Entity::Object(oraxid)));
    assert!(!targets.contains(&Entity::Object(zombie)));
    t.activate(P0, bears, 0, &[Entity::Object(oraxid)]).unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    // The Bears (green, as it last existed) dealt the damage: not prevented.
    assert_eq!(marked(&t, oraxid), 1);
}

#[test]
fn sorcerers_wand_the_creature_is_the_source_and_wizard_is_checked_on_resolution() {
    cr!("113.7", "702.15b", "608.2h");
    ruling!(
        "Sorcerer's Wand",
        "The equipped creature, not Sorcerer’s Wand, is the source of the damage-dealing ability and of the damage dealt."
    );
    ruling!(
        "Sorcerer's Wand",
        "Whether the equipped creature is a Wizard is checked only as the ability resolves. If that creature has left the battlefield, use its last known information to determine if it was a Wizard."
    );
    supported("Sorcerer's Wand");
    supported("Vampire Nighthawk");
    // "{T}: This creature deals 1 damage to target player or planeswalker. If this
    // creature is a Wizard, it deals 2 damage instead." Vampire Nighthawk has lifelink:
    // its controller gains the life because it dealt the damage.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Sorcerer's Wand", hawk);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, hawk, "Wizard").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // Prodigal Sorcerer (a Wizard) leaves the battlefield with the ability on the stack:
    // its last known information says it was a Wizard.
    let mut t = TestGame::new(2);
    let wizard = t.battlefield(P0, "Prodigal Sorcerer");
    attach_new(&mut t, P0, "Sorcerer's Wand", wizard);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, wizard, "Wizard").unwrap();
    destroy(&mut t, wizard);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn razor_boomerang_is_the_damage_source_but_the_creature_targets() {
    cr!("113.7", "115.4", "702.16b", "702.16e", "602.2b", "701.3d");
    ruling!(
        "Razor Boomerang",
        "The source of the damage is Razor Boomerang, not the equipped creature. However, the equipped creature’s ability is what targets the permanent or player."
    );
    ruling!(
        "Razor Boomerang",
        "Unattaching Razor Boomerang is a cost to activate the equipped creature’s ability."
    );
    supported("Razor Boomerang");
    // "{T}, Unattach Razor Boomerang: It deals 1 damage to any target. Return Razor
    // Boomerang to its owner's hand." Equipped to green Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boomerang = attach_new(&mut t, P0, "Razor Boomerang", bears);
    let cat = t.battlefield(P1, "Nacatl Savage");
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    let targets = ability_targets(&mut t, bears, 0);
    assert!(targets.contains(&Entity::Object(cat)));
    assert!(!targets.contains(&Entity::Object(zombie)));
    t.activate(P0, bears, 0, &[Entity::Object(cat)]).unwrap();
    // Unattaching is a cost: it's done as the ability is activated.
    assert_eq!(attached_to(&t, boomerang), None);
    assert!(t.on_battlefield(boomerang));
    t.resolve_all();
    // Razor Boomerang (an artifact) is the source: protection from artifacts prevents it.
    assert_eq!(marked(&t, cat), 0);
    assert!(t.in_hand(P0, "Razor Boomerang"));
}

#[test]
fn throwing_knife_is_the_source_of_its_damage() {
    cr!("113.7", "702.16e", "608.2h");
    ruling!(
        "Throwing Knife",
        "Throwing Knife is the source of the damage dealt because of its triggered ability. For example, you could sacrifice Throwing Knife to deal 2 damage to a creature with protection from red, even if Throwing Knife is equipped to a red creature."
    );
    supported("Throwing Knife");
    // "Whenever equipped creature attacks, you may sacrifice this Equipment. If you do,
    // this Equipment deals 2 damage to any target." Equipped to red Gray Ogre.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Gray Ogre");
    attach_new(&mut t, P0, "Throwing Knife", ogre);
    let oraxid = t.battlefield(P1, "Oraxid");
    t.answer_targets(P0, &[Entity::Object(oraxid)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Throwing Knife"));
    assert_eq!(marked(&t, oraxid), 2);
}

#[test]
fn unattach_costs_leave_the_equipment_on_the_battlefield() {
    cr!("701.3d", "602.2b");
    ruling!(
        "Heartseeker",
        "“Unattach Heartseeker” means just that — Heartseeker moves off the creature it was equipping and remains on the battlefield."
    );
    ruling!(
        "Leonin Bola",
        "“Unattach Leonin Bola” means just that — Leonin Bola moves off the creature it was equipping and remains on the battlefield."
    );
    ruling!(
        "Surestrike Trident",
        "“Unattach Surestrike Trident” means just that — Surestrike Trident moves off the creature it was equipping and remains on the battlefield."
    );
    supported("Heartseeker");
    supported("Leonin Bola");
    // Heartseeker: "{T}, Unattach Heartseeker: Destroy target creature."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let seeker = attach_new(&mut t, P0, "Heartseeker", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, bears, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(seeker));
    assert_eq!(attached_to(&t, seeker), None);
    // Leonin Bola: "{T}, Unattach Leonin Bola: Tap target creature."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bola = attach_new(&mut t, P0, "Leonin Bola", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, bears, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert!(t.on_battlefield(bola));
    assert_eq!(attached_to(&t, bola), None);
    // Surestrike Trident: "{T}, Unattach Surestrike Trident: This creature deals damage
    // equal to its power to target player or planeswalker." The power is the creature's
    // once the Trident is unattached (it gives no bonus anyway).
    supported("Surestrike Trident");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let trident = attach_new(&mut t, P0, "Surestrike Trident", giant);
    assert!(has_kw(&t, giant, mtg_engine::keywords::KeywordKind::FirstStrike));
    t.activate(P0, giant, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(attached_to(&t, trident), None);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(trident));
    assert!(!has_kw(&t, giant, mtg_engine::keywords::KeywordKind::FirstStrike));
}

#[test]
fn the_equipped_creatures_controller_activates_the_granted_ability() {
    cr!("113.7", "602.2", "701.3d");
    ruling!(
        "Heartseeker",
        "The equipped creature’s controller chooses whether or not to activate the activated ability. It doesn’t matter who controls Heartseeker."
    );
    ruling!(
        "Leonin Bola",
        "The equipped creature’s controller chooses whether or not to activate the activated ability. It doesn’t matter who controls Leonin Bola."
    );
    ruling!(
        "Surestrike Trident",
        "The equipped creature’s controller chooses whether or not to activate the activated ability. It doesn’t matter who controls Surestrike Trident."
    );
    for name in ["Heartseeker", "Leonin Bola", "Surestrike Trident"] {
        supported(name);
        // P1 controls the Equipment; it's attached to P0's Grizzly Bears.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let eq = attach_new(&mut t, P0, name, bears);
        give_control(&mut t, eq, P1);
        assert_eq!(attached_to(&t, eq), Some(Entity::Object(bears)));
        let giant = t.battlefield(P1, "Hill Giant");
        // Surestrike Trident's ability targets a player or planeswalker.
        let target = if name == "Surestrike Trident" {
            Entity::Player(P1)
        } else {
            Entity::Object(giant)
        };
        assert!(t.activate(P1, bears, 0, &[target]).is_err());
        t.clear_answers();
        t.activate(P0, bears, 0, &[target]).unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(eq), "{name}");
        assert_eq!(attached_to(&t, eq), None, "{name}");
        assert_eq!(t.obj_now(eq).controller, P1, "{name}");
    }
}

#[test]
fn shadowspear_can_be_activated_while_unattached() {
    cr!("301.5", "602.1");
    ruling!(
        "Shadowspear",
        "You can activate Shadowspear's second ability whether or not it's equipped to a creature."
    );
    supported("Shadowspear");
    supported("Gladecover Scout");
    // "{1}: Permanents your opponents control lose hexproof and indestructible until end
    // of turn."
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P0, "Shadowspear");
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, spear, "lose hexproof").unwrap();
    t.resolve_all();
    assert!(spell_targets(&mut t, P0, "Doom Blade").contains(&Entity::Object(scout)));
}

#[test]
fn toralfs_hammer_the_creature_targets_but_the_hammer_deals_the_damage() {
    cr!("113.7", "115.4", "702.16b", "702.16e", "712.11");
    ruling!(
        "Toralf, God of Fury // Toralf's Hammer",
        "The creature equipped by Toralf's Hammer is the source of the activated ability, but Toralf's Hammer is the source of the damage."
    );
    // Toralf's Hammer (the back face, {1}{R}): equipped creature has "{1}{R}, {T},
    // Unattach Toralf's Hammer: It deals 3 damage to any target. Return Toralf's Hammer to
    // its owner's hand." Cast as its back face, attached to green Grizzly Bears.
    let name = "Toralf, God of Fury // Toralf's Hammer";
    assert!(mtg_engine::card::card(name).faces[1].unsupported.is_empty());
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let hammer = t.hand(P0, name);
    t.cast(P0, hammer)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    let hammer = t.g.current(hammer);
    assert!(t.g.attach(hammer, Entity::Object(bears)));
    let oraxid = t.battlefield(P1, "Oraxid");
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    // Protection from green: can't be targeted by the Bears' ability.
    let targets = ability_targets(&mut t, bears, 0);
    assert!(targets.contains(&Entity::Object(oraxid)));
    assert!(!targets.contains(&Entity::Object(zombie)));
    t.activate(P0, bears, 0, &[Entity::Object(oraxid)]).unwrap();
    t.resolve_all();
    // The (red) Hammer deals the damage: protection from red prevents it.
    assert_eq!(marked(&t, oraxid), 0);
    assert_eq!(t.zone(hammer), mtg_engine::object::Zone::Hand(P0));
}
