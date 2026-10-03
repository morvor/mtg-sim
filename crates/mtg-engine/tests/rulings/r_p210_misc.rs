//! Rulings batch P210 — enchant (CR 303.4, 702.5): sources of granted abilities,
//! protection Auras, "doesn't untap" Auras, counters an Aura put, damage and life
//! replacement Auras, and other one-off Aura rules.

use crate::r_p209_common::*;
use crate::r_p210_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s09_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// The enchanted creature is the source of the abilities it's granted
// ---------------------------------------------------------------------------------------

#[test]
fn the_enchanted_creature_is_the_source_of_granted_abilities() {
    cr!("113.7a", "702.16b", "702.15b");
    ruling!(
        "Quicksilver Dagger",
        "The enchanted creature is the source of both the damage ability and the resulting damage, not Quicksilver Dagger."
    );
    ruling!(
        "Burning Anger",
        "The enchanted creature is the source of the activated ability, not Burning Anger. For example, if the enchanted creature is green, you could activate the ability choosing a creature with protection from red as the target."
    );
    ruling!(
        "Grasp of the Hieromancer",
        "The enchanted creature is the source of the triggered ability it gains, not Grasp of the Hieromancer. If the enchanted creature isn’t white, that ability can target a creature with protection from white, for example."
    );
    // Quicksilver Dagger on a lifelinking creature: its controller gains the life.
    supported("Quicksilver Dagger");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Quicksilver Dagger", hawk);
    let h = t.hand_size(P0);
    t.activate(P0, hawk, 0, &[P1.into()]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    assert_eq!(t.hand_size(P0), h + 1);
    // Burning Anger (red) on a green creature can target a creature with protection
    // from red.
    supported("Burning Anger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Burning Anger", bears);
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    t.activate(P0, bears, 0, &[paladin.into()]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Paladin en-Vec"));
    // Grasp of the Hieromancer (white) on a green creature taps a protection-from-white
    // creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Grasp of the Hieromancer", bears);
    let knight = t.battlefield(P1, "Black Knight");
    t.answer_targets(P0, &[knight.into()]);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    t.resolve_all();
    assert!(t.obj_now(knight).tapped);
}

// ---------------------------------------------------------------------------------------
// Protection Auras
// ---------------------------------------------------------------------------------------

#[test]
fn holy_mantle_protects_from_creatures_and_creature_cards() {
    cr!("702.16b", "702.16c", "702.16e", "702.97a");
    ruling!(
        "Holy Mantle",
        "The enchanted creature can’t be blocked, it can’t be targeted by abilities of creatures or creature cards (like, notably, the scavenge ability), and all damage dealt to it by creatures or creature cards is prevented."
    );
    supported("Holy Mantle");
    let setup = |mantle: bool| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if mantle {
            attach_new(&mut t, P0, "Holy Mantle", bears);
        }
        (t, bears)
    };
    // Can't be blocked.
    let (mut t, bears) = setup(true);
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    assert!(!t.g.can_block(giant, bears));
    // Damage from a creature is prevented.
    let (mut t, bears) = setup(true);
    let giant = t.battlefield(P1, "Hill Giant");
    deal(&mut t, giant, 3, bears);
    assert_eq!(t.obj_now(bears).damage, 0);
    // Scavenge (an ability of a creature card) can't target it.
    for mantle in [true, false] {
        let (mut t, bears) = setup(mantle);
        let mangler = t.graveyard(P0, "Dreg Mangler");
        lots_of_mana(&mut t, P0);
        let r = t.activate(P0, mangler, 0, &[bears.into()]);
        assert_eq!(r.is_ok(), !mantle, "mantle {mantle}");
    }
}

#[test]
fn guildscorn_ward_protects_from_multicolored() {
    cr!("702.16a", "702.16b", "702.16c", "702.16d", "702.16e");
    ruling!(
        "Guildscorn Ward",
        "The enchanted creature can’t be enchanted or equipped by multicolored Auras and Equipment, it can’t be blocked by multicolored creatures, it can’t be targeted by multicolored spells or abilities from multicolored sources, and all damage dealt to it by multicolored sources is prevented."
    );
    supported("Guildscorn Ward");
    let setup = || {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Guildscorn Ward", bears);
        (t, bears)
    };
    // A multicolored (hybrid) Aura can't enchant it.
    let (mut t, bears) = setup();
    assert!(cast_aura(&mut t, P0, "Scourge of the Nobilis", bears).is_none());
    // A multicolored creature can't block it; a monocolored one can.
    let (mut t, bears) = setup();
    let wolf = t.battlefield(P1, "Watchwolf");
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    assert!(!t.g.can_block(wolf, bears));
    assert!(t.g.can_block(giant, bears));
    // A multicolored spell can't target it.
    let (mut t, bears) = setup();
    let other = t.battlefield(P0, "Hill Giant");
    let targets = spell_targets(&mut t, P1, "Lightning Helix");
    assert!(!targets.contains(&bears.into()));
    assert!(targets.contains(&other.into()));
    // Damage from a multicolored source is prevented.
    let (mut t, bears) = setup();
    let wolf = t.battlefield(P1, "Watchwolf");
    deal(&mut t, wolf, 3, bears);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn gaseous_form_creature_can_still_attack_and_block() {
    cr!("508.1a", "509.1a", "615.1");
    ruling!(
        "Gaseous Form",
        "The enchanted creature can still attack and block."
    );
    supported("Gaseous Form");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Gaseous Form", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    assert!(can_attack(&mut t, bears));
    attack_with(&mut t, &[(bears, P1.into())]);
    assert!(t.g.can_block(giant, bears));
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 0);
    // As a blocker.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Gaseous Form", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, P1.into())]);
    assert!(t.g.can_block(bears, giant));
    block_and_finish(&mut t, P1, &[(bears, giant)]);
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------------------

#[test]
fn solid_footing_changes_only_combat_damage_assignment() {
    cr!("510.1a", "701.14a");
    ruling!(
        "Solid Footing",
        "Solid Footing’s last ability doesn’t change the enchanted creature’s power; it changes only the amount of combat damage that creature assigns. All other rules and effects that check power or toughness use the actual values. For example, if a creature fights while enchanted by Solid Footing, that creature will deal damage equal to its power, not its toughness."
    );
    supported("Solid Footing");
    // Combat: assigns 7 (its toughness).
    let mut t = TestGame::new(2);
    let sentinel = t.battlefield(P0, "Sentinel of the Eternal Watch");
    attach_new(&mut t, P0, "Solid Footing", sentinel);
    assert_eq!(t.pt(sentinel), (5, 7));
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(sentinel, P1.into())]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 13);
    // Fight: deals 5 (its power).
    let mut t = TestGame::new(2);
    let sentinel = t.battlefield(P0, "Sentinel of the Eternal Watch");
    attach_new(&mut t, P0, "Solid Footing", sentinel);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_spell(&mut t, P0, "Prey Upon", &[sentinel.into(), maw.into()]);
    t.resolve_all();
    assert_eq!(t.obj_now(maw).damage, 5);
}

#[test]
fn cartouche_of_strengths_trample_doesnt_apply_in_a_fight() {
    cr!("701.14b", "702.19a");
    ruling!(
        "Cartouche of Strength",
        "The enchanted creature has +1/+1 and trample while it's fighting. However, trample doesn't apply during a fight."
    );
    supported("Cartouche of Strength");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let aura = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Cartouche of Strength");
    t.cast(P0, aura).target(bears).go();
    t.g.resolve_top();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[elves.into()]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn oppressive_rays_lets_a_must_attack_creature_stay_home() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Oppressive Rays",
        "The enchanted creature's controller can choose to not attack or block with it even if it must attack or block if able. Players can't be forced to pay a cost to attack or block."
    );
    supported("Oppressive Rays");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Guise of Fire", bears);
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    attach_new(&mut t, P1, "Oppressive Rays", bears);
    assert!(legal_attack(&mut t, &[]));
}

// ---------------------------------------------------------------------------------------
// "Doesn't untap during its controller's untap step"
// ---------------------------------------------------------------------------------------

#[test]
fn doesnt_untap_auras_dont_stop_other_untapping() {
    cr!("502.3", "701.26b");
    ruling!(
        "Paralyzing Grasp",
        "The enchanted creature can still be untapped by a spell or ability at other times."
    );
    ruling!(
        "Containment Membrane",
        "The enchanted creature can still be untapped by other spells and abilities."
    );
    ruling!(
        "Runner's Bane",
        "The enchanted creature can still be untapped in other ways, such as by Aurelia, the Warleader."
    );
    ruling!(
        "Claustrophobia",
        "The enchanted creature can still be untapped in other ways. Claustrophobia will remain attached, and the creature will continue to not untap during its controller's untap step."
    );
    ruling!(
        "Locked in the Cemetery",
        "The enchanted creature can still be untapped in other ways. Locked in the Cemetery will remain attached, and the creature will continue to not untap during its controller's untap step."
    );
    for aura in [
        "Paralyzing Grasp",
        "Containment Membrane",
        "Runner's Bane",
        "Claustrophobia",
        "Locked in the Cemetery",
    ] {
        supported(aura);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let a = attach_new(&mut t, P0, aura, bears);
        tap(&mut t, bears);
        untap(&mut t, bears);
        assert!(!t.obj_now(bears).tapped, "{aura}");
        assert_eq!(attached_to(&t, a), Some(bears.into()), "{aura}");
        tap(&mut t, bears);
        t.advance_to(P1, Step::Upkeep);
        assert!(t.obj_now(bears).tapped, "{aura}");
    }
}

#[test]
fn weakstones_subjugation_stops_untapping_whether_or_not_you_paid() {
    cr!("502.3", "603.5");
    ruling!(
        "Weakstone's Subjugation",
        "The enchanted permanent doesn't untap during its controller's untap step even if you didn't pay {3}."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    cast_aura(&mut t, P0, "Weakstone's Subjugation", bears).unwrap();
    assert!(!t.obj_now(bears).tapped);
    tap(&mut t, bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn urban_burgeoning_land_untaps_in_other_players_untap_steps() {
    cr!("502.3", "502.2");
    ruling!(
        "Urban Burgeoning",
        "The enchanted land untaps at the same time as the active player’s permanents. You can’t choose to not untap it at that time."
    );
    supported("Urban Burgeoning");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    let other = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Urban Burgeoning", land);
    tap(&mut t, land);
    tap(&mut t, other);
    let p1_land = t.battlefield(P1, "Forest");
    tap(&mut t, p1_land);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(land).tapped);
    assert!(!t.obj_now(p1_land).tapped);
    assert!(t.obj_now(other).tapped);
}

#[test]
fn apathy_may_discard_even_if_the_creature_is_untapped() {
    cr!("701.9b");
    ruling!(
        "Apathy",
        "The player may discard a card at random whether the enchanted creature is tapped or not."
    );
    supported("Apathy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Apathy", bears);
    t.advance_to(P1, Step::Upkeep);
    t.hand(P0, "Forest");
    let h = t.hand_size(P0);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h - 1);
    assert!(!t.obj_now(bears).tapped);
}

// ---------------------------------------------------------------------------------------
// Counters an Aura puts stay on the creature
// ---------------------------------------------------------------------------------------

#[test]
fn counters_put_by_an_aura_stay_after_it_leaves_or_moves() {
    cr!("122.1", "122.2");
    ruling!(
        "Daily Regimen",
        "The +1/+1 counters that are put on the enchanted creature are independent of Daily Regimen. If Daily Regimen leaves the battlefield or is moved to another creature, the counters will stay put."
    );
    ruling!(
        "Torture",
        "The -1/-1 counters that are put on the enchanted creature are independent of Torture. If Torture leaves the battlefield or is moved to another creature, the counters will stay put."
    );
    ruling!(
        "Primal Cocoon",
        "The counters are put on the creature, not on Primal Cocoon. They'll stay on the creature even after Primal Cocoon is sacrificed or otherwise stops enchanting that creature."
    );
    for (aura, kind) in [
        ("Daily Regimen", counters::PLUS1),
        ("Torture", counters::MINUS1),
    ] {
        supported(aura);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let other = t.battlefield(P0, "Grizzly Bears");
        let a = attach_new(&mut t, P0, aura, giant);
        lots_of_mana(&mut t, P0);
        t.activate(P0, a, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.counters(giant, kind), 1, "{aura}");
        assert_eq!(t.counters(a, kind), 0, "{aura}");
        assert!(t.g.attach(a, other.into()));
        t.g.recompute();
        assert_eq!(t.counters(giant, kind), 1, "{aura}");
        destroy(&mut t, a);
        assert_eq!(t.counters(giant, kind), 1, "{aura}");
    }
    supported("Primal Cocoon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Primal Cocoon", bears);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Primal Cocoon"));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn blossombind_doesnt_remove_counters_already_there() {
    cr!("122.1", "614.17");
    ruling!(
        "Blossombind",
        "The effect of Blossombind's last ability doesn't remove counters that are already on the enchanted creature."
    );
    supported("Blossombind");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(bears.into(), counters::PLUS1, 2, None);
    attach_new(&mut t, P0, "Blossombind", bears);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.pt(bears), (4, 4));
    t.g.add_counters(bears.into(), counters::PLUS1, 1, None);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------------------
// Player Auras
// ---------------------------------------------------------------------------------------

#[test]
fn grievous_wound_life_gain_spells_still_do_everything_else() {
    cr!("119.7", "608.2b");
    ruling!(
        "Grievous Wound",
        "Spells and abilities that cause the enchanted player to gain life still resolve while Grievous Wound is on the battlefield. The enchanted player won't gain life, but any other effects of that spell or ability will still happen."
    );
    supported("Grievous Wound");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Grievous Wound", P1);
    t.set_step(P1, Step::PrecombatMain);
    let h = t.hand_size(P1);
    cast_spell(&mut t, P1, "Revitalize", &[]);
    assert_eq!(t.hand_size(P1), h);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Revitalize"));
    // P1 still draws the card.
    assert_eq!(t.hand_size(P1), h + 1);
}

#[test]
fn spiteful_shadows_damage_from_the_creature_uses_its_lifelink_and_infect() {
    cr!("702.15b", "702.90b", "120.3b");
    ruling!(
        "Spiteful Shadows",
        "Spiteful Shadows causes the enchanted creature to deal damage to its controller. Abilities like lifelink and infect will apply."
    );
    supported("Spiteful Shadows");
    // Lifelink: P1 loses 1 and gains 1.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Spiteful Shadows", hawk);
    let bears = t.battlefield(P0, "Grizzly Bears");
    deal(&mut t, bears, 1, hawk);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Infect: P1 gets a poison counter instead.
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P1, "Phyrexian Crusader");
    attach_new(&mut t, P0, "Spiteful Shadows", crusader);
    let bears = t.battlefield(P0, "Grizzly Bears");
    deal(&mut t, bears, 1, crusader);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.player(P1).poison(), 1);
}

#[test]
fn cruel_reality_must_sacrifice_a_creature_or_planeswalker_if_able() {
    cr!("701.21a", "603.3a");
    ruling!(
        "Cruel Reality",
        "The enchanted player can’t choose to lose 5 life if they have a creature or planeswalker that can be sacrificed."
    );
    ruling!(
        "Cruel Reality",
        "The enchanted player chooses a permanent to sacrifice from among the creatures and planeswalkers that player controls. You don’t choose which type of permanent the player has to sacrifice."
    );
    supported("Cruel Reality");
    // Only a planeswalker: it's sacrificed, no life lost.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Jace Beleren");
    attach_new(&mut t, P0, "Cruel Reality", P1);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert_eq!(t.life(P1), 20);
    // A creature and a planeswalker: P1 chooses among both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    attach_new(&mut t, P0, "Cruel Reality", P1);
    next_upkeep(&mut t, P1);
    let from = t.asked().len();
    t.answer_choose(P1, &[Entity::Object(jace)]);
    t.resolve_all();
    let asked = asked_since(&t, from);
    assert!(asked.iter().any(|(p, d)| *p == P1 && {
        let s = format!("{d:?}");
        s.contains(&format!("{:?}", bears)) && s.contains(&format!("{:?}", jace))
    }));
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Others
// ---------------------------------------------------------------------------------------

#[test]
fn prison_term_moves_from_its_creature_to_the_new_one() {
    cr!("701.3a", "303.4");
    ruling!(
        "Prison Term",
        "The last ability works only if Prison Term is already on the battlefield. You may move it from the creature it's currently enchanting onto the new creature."
    );
    supported("Prison Term");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let term = attach_new(&mut t, P0, "Prison Term", bears);
    t.answer_yes(P0, true);
    let giant = t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(attached_to(&t, term), Some(giant.into()));
    to_combat(&mut t, P1);
    assert!(can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, giant));
}

#[test]
fn in_too_deep_split_second_doesnt_change_when_it_can_be_cast() {
    cr!("702.61a", "307.1");
    ruling!(
        "In Too Deep",
        "Split second doesn't allow players to cast the spell it's on at times when they otherwise wouldn't be able to cast it. An enchantment with split second may still be cast only during its controller's main phase when the stack is empty."
    );
    supported("In Too Deep");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "In Too Deep");
    // Not in the upkeep, nor on the opponent's turn.
    t.set_step(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    // Not with something on the stack in its own main phase.
    t.set_step(P0, Step::PrecombatMain);
    cast_spell(&mut t, P0, "Giant Growth", &[bears.into()]);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    t.resolve_all();
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
}
