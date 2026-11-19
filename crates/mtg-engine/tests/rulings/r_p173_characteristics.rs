//! Rulings batch P173 — characteristics and damage: marked damage stays until the cleanup
//! step, so lost bonuses can make it lethal (CR 120.6, 704.5g, 514.2); excess damage takes
//! earlier damage and planeswalker loyalty into account (CR 120.10, 120.4a); type-changing
//! effects (CR 205.1b, 305.6, 305.7) and the replacement effects that apply after them (CR
//! 614.12); what counts once (CR 109.1); color-setting effects (CR 105.3, 613.1e);
//! base-setting effects under modifiers and counters (CR 613.4b, 613.4c); granted mana
//! abilities (CR 113.3b, 605.1a); and Auras that can't legally enchant what they're on
//! (CR 303.4d, 704.5m).

use crate::r_p196_common::pump;
use crate::r_p212_common::back_face;
use crate::r_s01_common::{custom_card, supported, tokens};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, damage, give_control};
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::tapped;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// --- Marked damage and lost bonuses ----------------------------------------------------------

#[test]
fn nonlethal_damage_becomes_lethal_when_a_bonus_is_lost() {
    cr!("120.6", "704.5g", "514.2");
    // Ashaya: 4 Forests and itself (a Forest land too) make it 5/5; 3 damage; two
    // Forests leave: 3/3.
    ruling!(
        "Ashaya, Soul of the Wild",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Ashaya may become lethal if lands you control leave the battlefield during that turn."
    );
    supported("Ashaya, Soul of the Wild");
    let mut t = TestGame::new(2);
    let ashaya = t.battlefield(P0, "Ashaya, Soul of the Wild");
    let forests = t.lands(P0, "Forest", 4);
    assert_eq!(t.pt(ashaya), (5, 5));
    damage(&mut t, forests[0], 3, ashaya);
    assert!(t.on_battlefield(ashaya));
    destroy(&mut t, forests[0]);
    assert!(t.on_battlefield(ashaya));
    destroy(&mut t, forests[1]);
    assert!(!t.on_battlefield(ashaya), "3 damage on a 3/3");

    ruling!(
        "Sanctuary Lockdown",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Human you control may become lethal if Sanctuary Lockdown leaves the battlefield during that turn."
    );
    ruling!(
        "Grand Architect",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a blue creature you control may become lethal if Grand Architect leaves the battlefield during that turn."
    );
    for (lord, creature) in [
        ("Sanctuary Lockdown", "Alaborn Trooper"),
        ("Grand Architect", "Merfolk of the Pearl Trident"),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let c = t.battlefield(P0, creature);
        let base = t.pt(c).1 - 1;
        damage(&mut t, l, base, c);
        assert!(t.on_battlefield(c), "{lord}");
        destroy(&mut t, l);
        assert!(!t.on_battlefield(c), "{lord}");
        // The damage would have worn off in the cleanup step.
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let c = t.battlefield(P0, creature);
        damage(&mut t, l, base, c);
        t.advance_to(P1, Step::Upkeep);
        destroy(&mut t, l);
        assert!(t.on_battlefield(c), "{lord}");
    }
}

// --- Toralf, God of Fury: excess damage ----------------------------------------------------

/// P0's Toralf with a Mountain for a red spell; P1's `victim`.
fn toralf(t: &mut TestGame, victim: &str) -> ObjectId {
    supported("Toralf, God of Fury // Toralf's Hammer");
    t.battlefield(P0, "Toralf, God of Fury // Toralf's Hammer");
    t.battlefield(P0, "Mountain");
    t.battlefield(P1, victim)
}

/// P0 casts `spell` at `target`; Toralf's trigger (if any) targets P1.
fn burn(t: &mut TestGame, spell: &str, target: ObjectId) {
    let s = t.hand(P0, spell);
    t.cast(P0, s).target(target).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
}

#[test]
fn toralf_excess_damage_counts_damage_already_dealt_this_turn() {
    cr!("120.10", "120.6");
    ruling!(
        "Toralf, God of Fury",
        "A creature has been dealt excess damage if one or more sources deal more damage to it than the minimum amount of damage required to be lethal damage. In most cases, this means damage greater than its toughness, but consider any damage already dealt to it that turn."
    );
    // Shock (2) at Grizzly Bears that already have 1 damage: 1 excess.
    let mut t = TestGame::new(2);
    let bears = toralf(&mut t, "Grizzly Bears");
    let pinger = t.battlefield(P0, "Prodigal Pyromancer");
    damage(&mut t, pinger, 1, bears);
    burn(&mut t, "Shock", bears);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
    // Without the earlier damage, none.
    let mut t = TestGame::new(2);
    let bears = toralf(&mut t, "Grizzly Bears");
    burn(&mut t, "Shock", bears);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn toralf_earlier_combat_damage_doesnt_matter() {
    cr!("120.10", "510.2");
    ruling!(
        "Toralf, God of Fury",
        "It doesn't matter if a creature or planeswalker an opponent controls was dealt combat damage earlier in the turn. The only thing that matters is that the damage dealt to that permanent that caused excess damage to be dealt was noncombat damage."
    );
    // P1's Bears block Llanowar Elves (1 combat damage); after combat, Shock: 1 excess.
    let mut t = TestGame::new(2);
    let bears = toralf(&mut t, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    crate::r_s01_common::attack_with(&mut t, &[(elves, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(bears, elves)]);
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.life(P1), 20);
    burn(&mut t, "Shock", bears);
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
}

#[test]
fn toralf_excess_damage_to_a_creature_planeswalker_uses_the_smaller_lethal_amount() {
    cr!("120.10", "120.4a", "120.3c", "120.3d");
    ruling!(
        "Toralf, God of Fury",
        "If a permanent is both a creature and a planeswalker, the minimum amount of damage to be considered lethal damage is used to determine if excess damage has been dealt. For example, if a 5/5 creature that's also a planeswalker with three loyalty counters on it is dealt 4 noncombat damage, it's been dealt 1 excess damage and Toralf's ability will trigger."
    );
    supported("Flame Slash");
    let mut t = TestGame::new(2);
    toralf(&mut t, "Wastes");
    let walker = t.custom(
        P1,
        custom_card(
            "Test Walker",
            "Legendary Planeswalker Creature — Test",
            "{3}",
            Some((5, 5)),
            "",
        ),
        Zone::Battlefield,
    );
    t.g.add_counters(obj(walker), counters::LOYALTY, 3, None);
    t.g.recompute();
    // Flame Slash: 4 damage to target creature.
    burn(&mut t, "Flame Slash", walker);
    assert!(!t.on_battlefield(walker));
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
}

// --- Type changes ----------------------------------------------------------------------------

#[test]
fn ashaya_creatures_have_the_forest_mana_ability_but_stay_summoning_sick() {
    cr!("305.6", "302.6", "205.1b");
    ruling!(
        "Ashaya, Soul of the Wild",
        "A land that's a Forest has the intrinsic mana ability \"{T}: Add {G}.\""
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashaya, Soul of the Wild");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(tap_for_mana(&mut t, P0, bears, "Add {G}"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(!tap_for_mana(&mut t, P0, sick, "Add {G}"));
}

#[test]
fn ygra_food_creatures_trigger_it_and_enter_as_artifacts() {
    cr!("205.1b", "603.10a", "614.12");
    ruling!(
        "Ygra, Eater of All",
        "Creatures that die while you control Ygra are still Food artifacts as they leave the battlefield, so Ygra's last ability will trigger whenever a creature affected by its second ability dies."
    );
    supported("Ygra, Eater of All");
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Ygra, Eater of All");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, mine);
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.counters(y, counters::PLUS1), 4);

    ruling!(
        "Ygra, Eater of All",
        "Replacement effects that modify artifacts as they enter will apply after you apply Ygra's effect. For example, if you control Ygra and there is an effect that says artifacts enter tapped, creatures will enter tapped."
    );
    supported("Root Maze");
    // Root Maze: "Artifacts and lands enter tapped."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ygra, Eater of All");
    t.battlefield(P1, "Root Maze");
    let bears = t.enter(P0, "Grizzly Bears");
    let other = t.enter(P1, "Grizzly Bears");
    assert!(tapped(&t, bears) && tapped(&t, other));
    // Without Ygra, creatures don't.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Root Maze");
    let bears = t.enter(P0, "Grizzly Bears");
    assert!(!tapped(&t, bears));
}

#[test]
fn krovikan_plague_falls_off_a_creature_that_becomes_a_wall() {
    cr!("303.4d", "704.5m");
    ruling!(
        "Krovikan Plague",
        "The Aura will be put into the graveyard if you lose control of the creature or if it somehow becomes a Wall."
    );
    ruling!(
        "Veteran's Voice",
        "Is put into its owner’s graveyard if you lose control of the creature."
    );
    ruling!(
        "Nature's Chosen",
        "It is put into the graveyard if you lose control of the creature."
    );
    for aura in ["Krovikan Plague", "Veteran's Voice", "Nature's Chosen"] {
        supported(aura);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_s06_common::attach_new(&mut t, P0, aura, bears);
        assert!(!t.in_graveyard(P0, aura));
        give_control(&mut t, bears, P1);
        assert!(t.in_graveyard(P0, aura), "{aura}");
    }
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Krovikan Plague", bears);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddSubtypes(vec!["Wall".into()])],
    );
    assert!(t.in_graveyard(P0, "Krovikan Plague"));
}

#[test]
fn arlinn_isnt_a_werewolf() {
    cr!("205.3j", "712.8d");
    ruling!(
        "Arlinn Kord",
        "Arlinn, Embraced by the Moon is not a Werewolf card; that is, she doesn’t have the creature type Werewolf. Spells and abilities that refer to Werewolf cards or Werewolves don’t apply to Arlinn."
    );
    supported("Arlinn Kord // Arlinn, Embraced by the Moon");
    let mut t = TestGame::new(2);
    let back = back_face(&mut t, P0, "Arlinn Kord // Arlinn, Embraced by the Moon");
    let front = t.battlefield(P0, "Arlinn Kord // Arlinn, Embraced by the Moon");
    for a in [back, front] {
        let o = t.obj_now(a);
        assert!(!o.chars.has_subtype("Werewolf"));
        assert!(o.chars.has_subtype("Arlinn"));
    }
    // Nor is the card in other zones.
    let in_graveyard = t.graveyard(P0, "Arlinn Kord // Arlinn, Embraced by the Moon");
    assert!(!t.obj_now(in_graveyard).chars.has_subtype("Werewolf"));
}

// --- What counts once; colors; base power and toughness ---------------------------------------

#[test]
fn thousand_moons_smithy_counts_an_artifact_creature_once() {
    cr!("109.1", "208.3");
    ruling!(
        "Thousand Moons Smithy",
        "An artifact creature you control counts only once towards the Gnome Soldier's power and toughness."
    );
    supported("Thousand Moons Smithy // Barracks of the Thousand");
    let mut t = TestGame::new(2);
    t.enter(P0, "Thousand Moons Smithy // Barracks of the Thousand");
    t.resolve_all();
    let gnome = tokens(&t, P0)[0];
    // The Smithy and the Gnome (an artifact creature).
    assert_eq!(t.pt(gnome), (2, 2));
    t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(gnome), (3, 3));
}

#[test]
fn grand_architect_makes_an_artifact_creature_only_blue() {
    cr!("105.3", "613.1e");
    ruling!(
        "Grand Architect",
        "Grand Architect's second ability causes the affected artifact creature to lose any other colors it has, but it doesn't cause it to stop being an artifact."
    );
    let mut t = TestGame::new(2);
    let ga = t.battlefield(P0, "Grand Architect");
    let golem = t.battlefield(P0, "Slash Panther");
    assert!(t.obj_now(golem).chars.colors.contains(Color::Red));
    t.battlefield(P0, "Island");
    t.answer_targets(P0, &[obj(golem)]);
    activate_containing(&mut t, P0, ga, "becomes blue").expect("activates");
    t.resolve_all();
    let o = t.obj_now(golem);
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    // And now Grand Architect's anthem applies.
    assert_eq!(t.pt(golem), (5, 3));
}

#[test]
fn katara_waterbend_sets_base_power_and_toughness_under_counters_and_bonuses() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Katara, Water Tribe's Hope",
        "Katara, Water Tribe's Hope's ability overwrites any effects that previously set a creature's base power and/or toughness. Any existing effects or counters that raise or lower a creature's power and/or toughness continue to apply."
    );
    supported("Katara, Water Tribe's Hope");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Katara, Water Tribe's Hope");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    pump(&mut t, bears, 2, 0);
    // An earlier base-setting effect: 0/1.
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::SetPT(
            Some(mtg_engine::ability::Value::c(0)),
            Some(mtg_engine::ability::Value::c(1)),
        )],
    );
    assert_eq!(t.pt(bears), (3, 2));
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.answer_choose(P0, &[]);
    activate_containing(&mut t, P0, k, "Waterbend").expect("activates");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4 + 1 + 2, 4 + 1), "{}", t.dump_log());
}

// --- Granted mana abilities -------------------------------------------------------------------

#[test]
fn rishkar_any_counter_and_its_own_target() {
    cr!("122.1", "115.3");
    ruling!(
        "Rishkar, Peema Renegade",
        "Each creature you control has Rishkar's mana ability as long as that creature has any kind of counter on it. The effect isn't limited to those with +1/+1 counters."
    );
    ruling!(
        "Rishkar, Peema Renegade",
        "Rishkar can be a target of its own triggered ability."
    );
    supported("Rishkar, Peema Renegade");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rishkar, Peema Renegade");
    let oiled = t.battlefield(P0, "Grizzly Bears");
    let shrunk = t.battlefield(P0, "Grizzly Bears");
    let plain = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(oiled), "oil", 1, None);
    t.g.add_counters(obj(shrunk), counters::MINUS1, 1, None);
    t.g.recompute();
    for c in [oiled, shrunk] {
        assert!(tap_for_mana(&mut t, P0, c, "Add {G}"));
    }
    assert!(!t
        .obj_now(plain)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add {G}")));
    // Its enters trigger targets Rishkar itself and the Bears.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let r = t.enter(P0, "Rishkar, Peema Renegade");
    t.answer_targets(P0, &[obj(r), obj(b)]);
    t.resolve_all();
    assert_eq!(t.counters(r, counters::PLUS1), 1, "{}", t.dump_log());
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

