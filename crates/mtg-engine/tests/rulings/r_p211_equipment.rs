//! Rulings batch P211 — Equipment (CR 301.5, 702.6): what the equipped creature gets, the
//! sources of Equipment abilities and of their damage, abilities that attach Equipment
//! without equipping, "becomes unattached" triggers, and Equipment in combat.

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` activates the equip ability of `equipment` targeting `target` (with whatever mana
/// is available).
fn equip(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.answer_targets(p, &[Entity::Object(target)]);
    activate_containing(t, p, equipment, "Equip")
}

// ---------------------------------------------------------------------------------------
// Sources: the Equipment or the equipped creature
// ---------------------------------------------------------------------------------------

#[test]
fn blazing_sunsteel_deals_all_the_damage_dealt_to_the_creature() {
    cr!("120.3", "120.4a", "603.2");
    ruling!(
        "Blazing Sunsteel",
        "A creature can be dealt an amount of damage greater than its toughness. For example, if the equipped creature has 1 toughness and is dealt 3 damage, it deals 3 damage, not 1, to the chosen target."
    );
    supported("Blazing Sunsteel");
    // "Whenever equipped creature is dealt damage, it deals that much damage to any
    // target." Llanowar Elves (toughness 1) is dealt 3 damage.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Blazing Sunsteel", elves);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, giant, 3, elves);
    assert!(!t.on_battlefield(elves));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn blazing_sunsteel_is_the_abilitys_source_but_the_creature_deals_the_damage() {
    cr!("113.7", "702.16b", "702.16e", "115.4");
    ruling!(
        "Blazing Sunsteel",
        "Blazing Sunsteel is the source of the middle triggered ability, but the equipped creature is the source of the damage. For example, if the equipped creature is green, that ability can't target a permanent with protection from red, but it can target one with protection from green, though the damage would be prevented."
    );
    supported("Vodalian Zombie");
    supported("Oraxid");
    // Green Grizzly Bears equipped with (red) Blazing Sunsteel is dealt 1 damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Blazing Sunsteel", bears);
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    let oraxid = t.battlefield(P1, "Oraxid");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(zombie)]);
    damage(&mut t, elves, 1, bears);
    let offered = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(zombie)));
    assert!(!offered.contains(&Entity::Object(oraxid)));
    t.resolve_all();
    // Protection from green prevents the Bears' damage.
    assert_eq!(t.obj_now(zombie).damage, 0);
    assert!(t.on_battlefield(zombie));
}

#[test]
fn heart_piercer_bow_is_the_source_of_its_ability_and_damage() {
    cr!("113.7", "702.16b", "115.4");
    ruling!(
        "Heart-Piercer Bow",
        "Heart-Piercer Bow (not the equipped creature) is the source of the triggered ability and the source of the damage. This means that a creature with protection from green may be targeted even if Heart-Piercer Bow is equipped to a green creature, but a creature with protection from artifacts may not."
    );
    supported("Heart-Piercer Bow");
    supported("Nacatl Savage");
    // "Whenever equipped creature attacks, this Equipment deals 1 damage to target
    // creature defending player controls."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Heart-Piercer Bow", bears);
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    let cat = t.battlefield(P1, "Nacatl Savage");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(zombie)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let offered = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(zombie)));
    assert!(!offered.contains(&Entity::Object(cat)));
    t.resolve_all();
    // The Bow (colorless) deals the damage: not prevented by protection from green.
    assert_eq!(t.obj_now(zombie).damage, 1);
}

#[test]
fn the_equipped_creature_is_the_source_of_granted_abilities() {
    cr!("113.7", "702.16b", "115.4", "702.6a");
    ruling!(
        "Wolfhunter's Quiver",
        "For each of the activated abilities, the equipped creature is the source of the ability and the damage. For example, if the equipped creature is not an artifact, you could activate either ability targeting a creature with protection from artifacts."
    );
    supported("Wolfhunter's Quiver");
    // Wolfhunter's Quiver: "{T}: This creature deals 1 damage to any target." On green
    // Grizzly Bears it can target Nacatl Savage (protection from artifacts).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Wolfhunter's Quiver", bears);
    let cat = t.battlefield(P1, "Nacatl Savage");
    assert!(ability_targets(&mut t, bears, 0).contains(&Entity::Object(cat)));
    t.activate(P0, bears, 0, &[Entity::Object(cat)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Nacatl Savage"));
    // On an artifact creature (Ornithopter) it can't.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Wolfhunter's Quiver", thopter);
    let cat = t.battlefield(P1, "Nacatl Savage");
    let targets = ability_targets(&mut t, thopter, 0);
    assert!(!targets.contains(&Entity::Object(cat)));
    assert!(targets.contains(&Entity::Player(P1)));
}

// ---------------------------------------------------------------------------------------
// What the equipped creature gets
// ---------------------------------------------------------------------------------------

#[test]
fn holy_frazzle_cannon_gives_one_counter_per_creature() {
    cr!("205.3", "122.1", "712.2");
    ruling!(
        "Invasion of New Capenna // Holy Frazzle-Cannon",
        "A creature “shares a creature type” with the equipped creature if they have at least one creature type in common. Any one creature will get only one +1/+1 counter this way, even if it has multiple creature types in common with the equipped creature."
    );
    supported("Invasion of New Capenna // Holy Frazzle-Cannon");
    // "Whenever equipped creature attacks, put a +1/+1 counter on that creature and each
    // other creature you control that shares a creature type with it."
    let mut t = TestGame::new(2);
    let cannon = t.battlefield(P0, "Invasion of New Capenna // Holy Frazzle-Cannon");
    assert!(mtg_engine::dfc::transform(&mut t.g, cannon));
    t.g.recompute();
    let warrior = t.battlefield(P0, "Elvish Warrior");
    let other_warrior = t.battlefield(P0, "Elvish Warrior");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cannon = t.g.current(cannon);
    assert!(t.g.attach(cannon, Entity::Object(warrior)));
    t.g.recompute();
    attack_with(&mut t, &[(warrior, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(warrior, counters::PLUS1), 1);
    // Elf Warrior: two types in common, one counter.
    assert_eq!(t.counters(other_warrior, counters::PLUS1), 1);
    // Elf Druid: Elf in common.
    assert_eq!(t.counters(elves, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn pennon_blade_counts_the_creatures_its_controller_controls() {
    cr!("109.5", "611.3a", "613.4c");
    ruling!(
        "Pennon Blade",
        "As long as you control Pennon Blade, its ability constantly counts the number of creatures you control. It doesn't matter who controls the creature it's equipping (in case an opponent somehow manages to take control of that creature). If you control the creature it's equipping, the bonus will include that creature too."
    );
    supported("Pennon Blade");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Craw Wurm");
    attach_new(&mut t, P0, "Pennon Blade", bears);
    // Three creatures, the Bears included.
    assert_eq!(t.pt(bears), (5, 5));
    t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(bears), (6, 6));
    // P1 takes the Bears: P0 controls three creatures, P1's Bears gets +3/+3.
    give_control(&mut t, bears, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn belt_of_giant_strength_overwrites_earlier_setting_effects_only() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Belt of Giant Strength",
        "Belt of Giant Strength will overwrite any previous effects that set the creature's power and toughness to specific numbers. Effects that otherwise modify the equipped creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Belt of Giant Strength");
    supported("Turn to Frog");
    // Turn to Frog (base 1/1), then Giant Growth, then a +1/+1 counter, then the Belt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let frog = in_hand_with_mana(&mut t, P0, "Turn to Frog");
    t.cast_with(P0, frog, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast_with(P0, gg, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    attach_new(&mut t, P0, "Belt of Giant Strength", bears);
    assert_eq!(t.pt(bears), (14, 14));
    // A later setting effect overwrites the Belt: Turn to Frog again (1/1 + 3 + 1).
    let frog = in_hand_with_mana(&mut t, P0, "Turn to Frog");
    t.cast_with(P0, frog, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn darksteel_axe_itself_is_indestructible_not_the_creature() {
    cr!("702.12b", "301.5");
    ruling!(
        "Darksteel Axe",
        "Darksteel Axe itself has indestructible, not the creature it's equipping."
    );
    supported("Darksteel Axe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let axe = attach_new(&mut t, P0, "Darksteel Axe", bears);
    assert!(!has_kw(&t, bears, KeywordKind::Indestructible));
    destroy(&mut t, axe);
    assert!(t.on_battlefield(axe));
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(axe));
}

#[test]
fn sigil_of_valor_counts_the_other_creatures_on_resolution_only() {
    cr!("608.2h", "611.2c", "506.5");
    ruling!(
        "Sigil of Valor",
        "Count the number of creatures you control other than the equipped creature as Sigil of Valor's ability resolves to determine the amount of the bonus. Once the ability resolves, the bonus won't change, even if the number of creatures you control does."
    );
    supported("Sigil of Valor");
    // "Whenever equipped creature attacks alone, it gets +1/+1 until end of turn for each
    // other creature you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sigil of Valor", bears);
    let elves = t.battlefield_sick(P0, "Llanowar Elves");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "attacks alone"), 1);
    // Two more creatures enter before it resolves: three others.
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // Then one leaves: the bonus doesn't change.
    destroy(&mut t, elves);
    assert_eq!(t.pt(bears), (5, 5));
}

// ---------------------------------------------------------------------------------------
// Attaching without equipping
// ---------------------------------------------------------------------------------------

#[test]
fn equipment_that_attaches_itself_on_entering_can_be_cast_without_creatures() {
    cr!("301.5c", "603.3d", "702.8a");
    ruling!(
        "Bramble Armor",
        "Bramble Armor doesn't enter the battlefield attached to a creature. Instead, the Equipment enters the battlefield and then a triggered ability attaches it to a creature. You may cast Bramble Armor even if you don't control any creatures."
    );
    ruling!(
        "Galadhrim Bow",
        "Galadhrim Bow doesn't enter the battlefield attached to a creature. Instead, the Equipment enters the battlefield and then a triggered ability attaches it to a creature. You may cast Galadhrim Bow even if you don't control any creatures."
    );
    supported("Bramble Armor");
    supported("Galadhrim Bow");
    for name in ["Bramble Armor", "Galadhrim Bow"] {
        // No creatures: it's cast and enters unattached.
        let mut t = TestGame::new(2);
        let c = in_hand_with_mana(&mut t, P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
        assert!(t.on_battlefield(c), "{name}");
        assert_eq!(attached_to(&t, c), None, "{name}");
        // With a creature: it enters unattached, then its trigger attaches it.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let c = in_hand_with_mana(&mut t, P0, name);
        t.cast(P0, c).go();
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.resolve();
        assert!(t.on_battlefield(c), "{name}");
        assert_eq!(attached_to(&t, c), None, "{name}");
        assert_eq!(on_stack(&t, "attach it"), 1, "{name}");
        t.resolve_all();
        assert_eq!(attached_to(&t, c), Some(Entity::Object(bears)), "{name}");
    }
}

#[test]
fn barbed_spike_creates_and_equips_the_thopter_in_one_resolution() {
    cr!("608.2c", "117.3", "701.3a");
    ruling!(
        "Barbed Spike",
        "Creating the Thopter token and attaching Barbed Spike to it happen as part of the resolution of the same ability. There is no time for an opponent to react after the token is created but before the Equipment is attached."
    );
    supported("Barbed Spike");
    let mut t = TestGame::new(2);
    let spike = t.enter(P0, "Barbed Spike");
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "Thopter"), 1);
    // One resolution, with no priority in between.
    t.g.resolve_top();
    let thopters = with_subtype(&t, P0, "Thopter");
    assert_eq!(thopters.len(), 1);
    assert_eq!(attached_to(&t, spike), Some(Entity::Object(thopters[0])));
    assert_eq!(t.pt(thopters[0]), (2, 1));
}

#[test]
fn cranial_platings_attach_ability_isnt_equip() {
    cr!("602.5d", "702.6a", "117.1a");
    ruling!(
        "Cranial Plating",
        "Cranial Plating's first activated ability is similar to an equip ability, but it's not an equip ability. Most importantly, it can be activated any time you could cast an instant, even during another player's turn."
    );
    supported("Cranial Plating");
    // "{B}{B}: Attach this Equipment to target creature you control." During P1's combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plating = t.battlefield(P0, "Cranial Plating");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::DeclareAttackers);
    // Its equip ability can't be activated now.
    assert!(equip(&mut t, P0, plating, bears).is_err());
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, plating, "{B}{B}").unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, plating), Some(Entity::Object(bears)));
    // One artifact: +1/+0.
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn concealed_weapon_attaches_on_turning_face_up_without_equip_cost_or_timing() {
    cr!("702.168a", "603.2", "702.6a");
    ruling!(
        "Concealed Weapon",
        "Attaching Concealed Weapon with its triggered ability isn't the same as using its equip ability. You don't pay mana for the attachment, and the timing restrictions for equip abilities don't apply."
    );
    supported("Concealed Weapon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let weapon = t.hand(P0, "Concealed Weapon");
    add_mana(&mut t, P0, ManaType::C, 3);
    let spell = t
        .cast(P0, weapon)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj_now(id).face_down);
    // During P1's turn, P0 turns it face up for exactly its disguise cost ({2}{R}).
    t.set_step(P1, Step::BeginningOfCombat);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: id,
        }),
    )
    .unwrap();
    t.g.flush_events();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(on_stack(&t, "attach it"), 1);
    t.resolve_all();
    assert_eq!(attached_to(&t, id), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (5, 2));
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
}

#[test]
fn bladehold_war_whip_reduces_only_generic_mana_in_equip_costs() {
    cr!("601.2f", "602.2b", "702.6a");
    ruling!(
        "Bladehold War-Whip",
        "Bladehold War-Whip's second ability reduces only the amount of generic mana in equip abilities. For example, it will reduce an equip cost of {1} to {0}, but it will have no effect on an equip cost of {G}."
    );
    supported("Bladehold War-Whip");
    supported("Bonesplitter");
    supported("Greatsword of Tyr");
    // Bonesplitter's equip {1} costs {0}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bladehold War-Whip");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    equip(&mut t, P0, splitter, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    // Greatsword of Tyr's equip {W} still costs {W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bladehold War-Whip");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Greatsword of Tyr");
    assert!(equip(&mut t, P0, sword, bears).is_err());
    t.lands(P0, "Plains", 1);
    t.clear_answers();
    equip(&mut t, P0, sword, bears).unwrap();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(bears)));
}

// ---------------------------------------------------------------------------------------
// "Whenever this Equipment becomes unattached from a permanent"
// ---------------------------------------------------------------------------------------

#[test]
fn becoming_unattached_destroys_or_sacrifices_the_permanent() {
    cr!("701.3c", "603.2", "603.10a", "704.5n");
    ruling!(
        "Captain's Hook",
        "Captain's Hook becomes unattached from the creature it's equipping if you equip it to a new creature, if Captain's Hook leaves the battlefield, if the equipped creature ceases to be a creature, or if Captain's Hook ceases to be an Equipment. (It also becomes unattached if the equipped creature leaves the battlefield, but the triggered ability won't do anything in that case.)"
    );
    ruling!(
        "Grafted Wargear",
        "Grafted Wargear becomes unattached from the creature it's equipping if you equip it to a new creature, if Grafted Wargear leaves the battlefield, if the equipped creature ceases to be a creature, or if Grafted Wargear ceases to be an Equipment. (It also becomes unattached if the equipped creature leaves the battlefield, but the triggered ability won't do anything in that case.)"
    );
    supported("Captain's Hook");
    supported("Grafted Wargear");
    supported("One with the Stars");
    for name in ["Captain's Hook", "Grafted Wargear"] {
        // Equipped to a new creature: the old one goes.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let eq = attach_new(&mut t, P0, name, bears);
        t.lands(P0, "Wastes", 1);
        equip(&mut t, P0, eq, giant).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert!(t.on_battlefield(giant), "{name}");
        // The Equipment leaves the battlefield.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let eq = attach_new(&mut t, P0, name, bears);
        destroy(&mut t, eq);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        // The creature stops being a creature (One with the Stars).
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let eq = attach_new(&mut t, P0, name, bears);
        attach_new(&mut t, P0, "One with the Stars", bears);
        t.settle();
        assert_eq!(attached_to(&t, eq), None, "{name}");
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        // The equipped creature leaves the battlefield: nothing else happens.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        attach_new(&mut t, P0, name, bears);
        move_to(&mut t, bears, Zone::Hand(P0));
        t.resolve_all();
        assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
        assert!(t.on_battlefield(giant), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------------------

#[test]
fn echo_circlet_blocks_stand_after_it_leaves_and_its_effect_is_cumulative() {
    cr!("509.1a", "509.1c", "506.4");
    ruling!(
        "Echo Circlet",
        "Destroying or unequipping the Echo Circlet after blockers have been declared will not undo any blocks made by the equipped creature."
    );
    ruling!(
        "Echo Circlet",
        "Echo Circlet’s effect is cumulative. If it equips a creature that can already block an additional creature, now it can block three creatures. The same is true if two Echo Circlets equip the same creature, for example."
    );
    supported("Echo Circlet");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let circlet = attach_new(&mut t, P1, "Echo Circlet", wurm);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(wurm, a), (wurm, b)],
    );
    destroy(&mut t, circlet);
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(wurm).len(), 2);
    t.answer(
        P1,
        DecisionKind::Damage,
        mtg_engine::decision::Answer::Numbers(vec![2, 4]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.life(P1), 20);
    // Two Echo Circlets: it can block three creatures.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    attach_new(&mut t, P1, "Echo Circlet", wurm);
    attach_new(&mut t, P1, "Echo Circlet", wurm);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
        &[(wurm, a), (wurm, b), (wurm, c)],
    );
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(wurm).len(), 3);
}

#[test]
fn dowsing_dagger_stays_untapped_when_the_creature_attacks() {
    cr!("508.1f", "301.5", "712.2");
    ruling!(
        "Dowsing Dagger // Lost Vale",
        "Attacking with an equipped creature doesn't cause Equipment attached to it to become tapped. Dowsing Dagger will normally be untapped when it transforms into Lost Vale."
    );
    supported("Dowsing Dagger // Lost Vale");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dagger = attach_new(&mut t, P0, "Dowsing Dagger // Lost Vale", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(dagger).tapped);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.obj_now(dagger).chars.name, "Lost Vale");
    assert!(!t.obj_now(dagger).tapped);
}

#[test]
fn robe_of_stars_phases_out_with_the_creature_and_back_in_attached() {
    cr!("702.26a", "702.26g", "702.26b");
    ruling!(
        "Robe of Stars",
        "As a creature is phased out, Auras and Equipment attached to it (including Robe of Stars) also phase out at the same time. Those Auras and Equipment will phase in attached to the creature they were attached to when they phased out."
    );
    supported("Robe of Stars");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let robe = attach_new(&mut t, P0, "Robe of Stars", bears);
    t.lands(P0, "Plains", 2);
    activate_containing(&mut t, P0, robe, "phases out").unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    assert!(t.obj_now(robe).phased_out);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out);
    assert!(!t.obj_now(robe).phased_out);
    assert_eq!(attached_to(&t, robe), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (2, 5));
}

#[test]
fn inquisitorial_rosettes_token_may_attack_a_different_player() {
    cr!("508.4", "506.2", "802.2");
    ruling!(
        "Inquisitorial Rosette",
        "For the triggered ability, you declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker the equipped creature is attacking."
    );
    supported("Inquisitorial Rosette");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Inquisitorial Rosette", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    let tok = with_subtype(&t, P0, "Astartes");
    assert_eq!(tok.len(), 1);
    let combat = t.g.combat.as_ref().unwrap();
    assert_eq!(combat.attack_target(tok[0]), Some(Entity::Player(P2)));
    assert_eq!(combat.attack_target(bears), Some(Entity::Player(P1)));
}

#[test]
fn bronze_cudgels_counts_every_resolution_of_its_ability() {
    cr!("603.4", "607.2", "707.10");
    ruling!(
        "Bronze Cudgels",
        "A resolution of the ability counts even if Bronze Cudgels was attached to a different creature or no creature at the time. A copy of the ability (created by Lithoform Engine, for example) will also count toward the total. An ability from another Equipment with the same name doesn't count towards the total, nor does an ability that's been countered."
    );
    supported("Bronze Cudgels");
    supported("Lithoform Engine");
    supported("Squelch");
    // "{2}: Until end of turn, equipped creature gets +X/+0, where X is the number of
    // times this ability has resolved this turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cudgels = t.battlefield(P0, "Bronze Cudgels");
    let other = t.battlefield(P0, "Bronze Cudgels");
    t.lands(P0, "Wastes", 16);
    // Resolved once while unattached.
    activate_containing(&mut t, P0, cudgels, "{2}:").unwrap();
    t.resolve_all();
    // The other Cudgels' resolutions don't count.
    activate_containing(&mut t, P0, other, "{2}:").unwrap();
    t.resolve_all();
    // Countered by Squelch: doesn't count.
    activate_containing(&mut t, P0, cudgels, "{2}:").unwrap();
    let ab = top_of_stack(&t);
    let squelch = in_hand_with_mana(&mut t, P0, "Squelch");
    t.cast_with(P0, squelch, &[Entity::Object(ab)]).unwrap();
    t.resolve_all();
    // Copied by Lithoform Engine: the copy counts.
    let engine = t.battlefield(P0, "Lithoform Engine");
    activate_containing(&mut t, P0, cudgels, "{2}:").unwrap();
    let ab = top_of_stack(&t);
    t.answer_targets(P0, &[Entity::Object(ab)]);
    activate_containing(&mut t, P0, engine, "Copy target activated or triggered").unwrap();
    t.resolve(); // Lithoform Engine's ability: a copy on the stack.
    t.resolve(); // the copy: resolution #2
    t.resolve(); // the original: resolution #3
    assert_eq!(t.stack_len(), 0);
    // Now attached: the next resolution (#4) gives +4/+0.
    assert!(t.g.attach(cudgels, Entity::Object(bears)));
    t.g.recompute();
    activate_containing(&mut t, P0, cudgels, "{2}:").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 2));
}

// ---------------------------------------------------------------------------------------
// "~ can be attached only to a [quality] creature"
// ---------------------------------------------------------------------------------------

#[test]
fn gate_smashers_equip_may_target_any_creature_but_it_attaches_only_to_toughness_4() {
    cr!("301.5b", "301.5c", "701.3b", "704.5n", "702.6a");
    ruling!(
        "Gate Smasher",
        "Gate Smasher's equip ability can target any creature. However, if that creature's toughness is 3 or less as the equip ability resolves, Gate Smasher won't become attached to it."
    );
    supported("Gate Smasher");
    // "This Equipment can be attached only to a creature with toughness 4 or greater."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let smasher = t.battlefield(P0, "Gate Smasher");
    t.lands(P0, "Wastes", 6);
    assert!(ability_targets(&mut t, smasher, 0).contains(&Entity::Object(bears)));
    equip(&mut t, P0, smasher, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, smasher), None);
    assert_eq!(t.pt(bears), (2, 2));
    // Craw Wurm (6/4) can be equipped; targeted while 6/4 but 6/3 as the ability
    // resolves, it isn't.
    equip(&mut t, P0, smasher, wurm).unwrap();
    t.g.add_counters(Entity::Object(wurm), counters::MINUS1, 1, None);
    t.resolve_all();
    assert_eq!(attached_to(&t, smasher), None);
    // With toughness 4 it is; when its toughness drops to 3, it becomes unattached.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let smasher = t.battlefield(P0, "Gate Smasher");
    t.lands(P0, "Wastes", 3);
    equip(&mut t, P0, smasher, wurm).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, smasher), Some(Entity::Object(wurm)));
    assert_eq!(t.pt(wurm), (9, 4));
    t.g.add_counters(Entity::Object(wurm), counters::MINUS1, 1, None);
    t.settle();
    assert_eq!(attached_to(&t, smasher), None);
    assert!(t.on_battlefield(smasher));
    assert_eq!(t.pt(wurm), (5, 3));
}

#[test]
fn kondas_banner_gives_at_most_two_plus_one_bonuses() {
    cr!("105.4", "205.3", "613.4c", "701.3b");
    ruling!(
        "Konda's Banner",
        "A creature can’t get more than +2/+2 from Konda’s Banner. Sharing more than one color or more than one creature type with the equipped creature does nothing."
    );
    supported("Konda's Banner");
    // "Konda's Banner can be attached only to a legendary creature. Creatures that share a
    // color with equipped creature get +1/+1. Creatures that share a creature type with
    // equipped creature get +1/+1." Barktooth Warbeard: a black and red Human Warrior
    // (6/5); the other creature is a black and red Human Warrior too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let barktooth = t.battlefield(P0, "Barktooth Warbeard");
    let mut def = custom_card("Twin", "Creature — Human Warrior", "{B}{R}", Some((1, 1)), "");
    def.faces[0].chars.colors = [Color::Black, Color::Red].into_iter().collect();
    let twin = t.custom(P0, def, Zone::Battlefield);
    let banner = t.battlefield(P0, "Konda's Banner");
    // Not a legendary creature: it can't be attached.
    assert!(!t.g.attach(banner, Entity::Object(bears)));
    assert!(t.g.attach(banner, Entity::Object(barktooth)));
    t.g.recompute();
    assert_eq!(t.pt(twin), (3, 3));
    assert_eq!(t.pt(barktooth), (8, 7));
    assert_eq!(t.pt(bears), (2, 2));
}
