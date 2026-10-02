//! Rulings batch P017 — abilities that check colors, types, and other qualities of spells
//! and permanents ("color break" cards): when targets are chosen and qualities checked
//! (CR 115, 608.2b–c), triggers on casting (CR 601.2i, 603.3), protection from a player
//! (CR 702.16k), and turn-history conditions.

use crate::r_p017_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::target_candidates;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// "if it's [color]": Hydroblast, Pyroblast, Burnout
// ---------------------------------------------------------------------------

/// `blast` (mode 0: counter target spell if it's `color`; mode 1: destroy target
/// permanent if it's `color`) can target a spell or permanent of any color; only one of
/// that color is countered or destroyed.
fn blast_checks_color_on_resolution(blast: &str, land: &str, of_color: &str, other: &str) {
    supported(blast);
    let mut t = TestGame::new(2);
    // Mode 1 on a permanent that isn't that color: legal, nothing happens.
    let not_it = t.battlefield(P1, other);
    t.lands(P0, land, 1);
    let spell = t.hand(P0, blast);
    t.cast(P0, spell).modes(&[1]).target(not_it).go();
    t.resolve_all();
    assert!(t.on_battlefield(not_it), "{blast}");
    assert!(t.in_graveyard(P0, blast));
    // Mode 1 on a permanent of that color: destroyed.
    let it = t.battlefield(P1, of_color);
    t.lands(P0, land, 1);
    let spell = t.hand(P0, blast);
    t.cast(P0, spell).modes(&[1]).target(it).go();
    t.resolve_all();
    assert!(!t.on_battlefield(it), "{blast}");
}

#[test]
fn hydroblast_can_target_anything_and_checks_red_on_resolution() {
    cr!("608.2c", "115.1");
    ruling!(
        "Hydroblast",
        "Hydroblast can target any spell or permanent, not just a red one. It checks the color of the target only on resolution."
    );
    blast_checks_color_on_resolution("Hydroblast", "Island", "Raging Goblin", "Grizzly Bears");
    // Mode 0: a red spell that's turned green by the time Hydroblast resolves isn't
    // countered.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 1);
    let blast = t.hand(P0, "Hydroblast");
    t.cast(P0, blast).modes(&[0]).target(bolt).go();
    modify_no_settle(
        &mut t,
        bolt,
        vec![Modification::SetColors(ColorSet::single(Color::Green))],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn pyroblast_can_target_anything_and_checks_blue_on_resolution() {
    cr!("608.2c", "115.1");
    ruling!(
        "Pyroblast",
        "Pyroblast can target any spell or permanent, not just a blue one. It checks the color of the target only on resolution."
    );
    blast_checks_color_on_resolution("Pyroblast", "Mountain", "Indigo Faerie", "Grizzly Bears");
    // Mode 0: a green spell isn't countered; a blue one is.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let bears = t.cast(P1, bears).go();
    t.lands(P0, "Mountain", 1);
    let blast = t.hand(P0, "Pyroblast");
    t.cast(P0, blast).modes(&[0]).target(bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.lands(P1, "Island", 2);
    let opt = t.hand(P1, "Opt");
    let opt = t.cast(P1, opt).go();
    t.lands(P0, "Mountain", 1);
    let blast = t.hand(P0, "Pyroblast");
    t.cast(P0, blast).modes(&[0]).target(opt).go();
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand, "Opt was countered");
    assert!(t.in_graveyard(P1, "Opt"));
}

#[test]
fn burnout_can_target_any_instant_and_draws_either_way() {
    cr!("608.2c", "603.7a");
    ruling!(
        "Burnout",
        "You may target any instant spell with Burnout. If it’s blue at the time Burnout resolves, that spell will be countered and you’ll draw a card at the beginning of the next turn’s upkeep."
    );
    supported("Burnout");
    let mut t = TestGame::new(2);
    // A red instant: not countered, but P0 still draws at the next upkeep.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Mountain", 2);
    let burnout = t.hand(P0, "Burnout");
    t.cast(P0, burnout).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // A blue instant: countered.
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    let opt = t.cast(P1, opt).go();
    t.lands(P0, "Mountain", 2);
    let burnout = t.hand(P0, "Burnout");
    t.cast(P0, burnout).target(opt).go();
    let p1_hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), p1_hand);
    // Two delayed draws at P1's upkeep (the next turn's).
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Only an instant spell can be targeted, not a creature spell.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.lands(P0, "Mountain", 2);
    let burnout = t.hand(P0, "Burnout");
    let from = t.asked().len();
    let _ = t.cast(P0, burnout).try_go();
    assert!(target_candidates(&t, P0, from).iter().all(|c| c.is_empty()));
}

// ---------------------------------------------------------------------------
// Avoid Fate, Entangling Vines
// ---------------------------------------------------------------------------

#[test]
fn avoid_fate_targets_an_instant_or_aura_spell_and_survives_a_switch_between_them() {
    cr!("115.7", "608.2b");
    ruling!(
        "Avoid Fate",
        "Avoid Fate must target an instant spell or an Aura spell as it’s cast, and it must be targeting an instant or an Aura spell at the time it resolves. It will still resolve if it starts out targeting a spell of one type but the target changes to a spell of the other type"
    );
    supported("Avoid Fate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    // P1 casts Pacifism on P0's Bears, then a Lightning Bolt at them.
    t.lands(P1, "Plains", 2);
    let pacifism = t.hand(P1, "Pacifism");
    let pacifism = t.cast(P1, pacifism).target(bears).go();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(bears).go();
    // P0 counters the Bolt with Avoid Fate; P1 Shunts Avoid Fate onto Pacifism.
    t.lands(P0, "Forest", 1);
    let fate = t.hand(P0, "Avoid Fate");
    let fate = t.cast(P0, fate).target(bolt).go();
    t.lands(P1, "Mountain", 3);
    let shunt = t.hand(P1, "Shunt");
    t.cast(P1, shunt).target(fate).go();
    t.answer_targets(P1, &[Entity::Object(pacifism)]);
    t.resolve();
    // Avoid Fate now targets the Aura spell and counters it; the Bolt resolves.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(!t.on_battlefield(bears));
    let _ = pacifism;
}

#[test]
fn avoid_fate_cant_target_a_sorcery() {
    cr!("115.1", "601.2c");
    ruling!(
        "Avoid Fate",
        "Avoid Fate must target an instant spell or an Aura spell as it’s cast"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let slash = t.hand(P1, "Flame Slash");
    let slash = t.cast(P1, slash).target(bears).go();
    t.lands(P0, "Forest", 1);
    let fate = t.hand(P0, "Avoid Fate");
    assert!(t.cast(P0, fate).target(slash).try_go().is_err());
}

#[test]
fn entangling_vines_needs_a_tapped_creature_and_is_countered_if_it_untaps() {
    cr!("303.4a", "608.2b", "115.1");
    ruling!(
        "Entangling Vines",
        "If Entangling Vines is cast as a spell, it can target only a tapped creature. If the creature has become untapped by the time Entangling Vines would resolve, it’s countered and put into its owner’s graveyard from the stack."
    );
    supported("Entangling Vines");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let vines = t.hand(P0, "Entangling Vines");
    assert!(t.cast(P0, vines).target(bears).try_go().is_err());
    t.g.tap(bears);
    t.cast(P0, vines).target(bears).go();
    t.g.untap(bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Entangling Vines"));
    assert!(t.g.attachments_of(Entity::Object(bears)).is_empty());
}

// ---------------------------------------------------------------------------
// Enchantress triggers: Mesa Enchantress, Kor Spiritdancer, Pearl-Ear
// ---------------------------------------------------------------------------

#[test]
fn mesa_enchantress_triggers_only_on_casting() {
    cr!("601.2i", "603.2");
    ruling!(
        "Mesa Enchantress",
        "Enchantments put onto the battlefield without being cast won't cause Mesa Enchantress's ability to trigger."
    );
    supported("Mesa Enchantress");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mesa Enchantress");
    let hand = t.hand_size(P0);
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn mesa_enchantress_draws_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Mesa Enchantress",
        "Mesa Enchantress's ability will resolve before the spell that caused it to trigger."
    );
    ruling!(
        "Mesa Enchantress",
        "If the enchantment spell is countered, Mesa Enchantress's ability still resolves and causes you to draw a card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mesa Enchantress");
    t.lands(P0, "Plains", 3);
    let anthem = t.hand(P0, "Glorious Anthem");
    let spell = t.cast(P0, anthem).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    // Countered: the card was still drawn.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kor_spiritdancer_triggers_on_any_aura_spell_and_resolves_first() {
    cr!("603.3", "601.2i");
    ruling!(
        "Kor Spiritdancer",
        "If you cast an Aura spell, Kor Spiritdancer's second ability triggers and goes on the stack on top of it. The ability will resolve before the spell does."
    );
    ruling!(
        "Kor Spiritdancer",
        "The second ability triggers when you cast any Aura spell, not just one that targets Kor Spiritdancer."
    );
    supported("Kor Spiritdancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kor Spiritdancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let pacifism = t.hand(P0, "Pacifism");
    let spell = t.cast(P0, pacifism).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(t.named_on_battlefield("Pacifism")[0]));
}

#[test]
fn pearl_ear_draws_only_for_auras_targeting_a_modified_permanent() {
    cr!("700.9", "603.2");
    ruling!(
        "Pearl-Ear, Imperial Advisor",
        "A creature you control that's the target of an Aura spell isn't modified unless it already has a counter on it, an Equipment attached to it, or an Aura you control attached to it."
    );
    supported("Pearl-Ear, Imperial Advisor");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pearl-Ear, Imperial Advisor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let draws = |t: &mut TestGame, target: ObjectId| -> bool {
        t.lands(P0, "Plains", 1);
        let hs = t.hand(P0, "Holy Strength");
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.cast(P0, hs).target(target).go();
        t.resolve_all();
        t.hand_size(P0) == hand
    };
    // Unmodified: the Aura spell targeting it doesn't make it modified.
    assert!(!draws(&mut t, bears));
    // Now it has an Aura P0 controls attached: modified.
    assert!(draws(&mut t, bears));
    // A counter is a modification too.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.add_counters(Entity::Object(elves), counters::PLUS1, 1, None);
    assert!(draws(&mut t, elves));
}

#[test]
fn pearl_ear_affinity_counts_auras_on_opponents_permanents_and_stacks() {
    cr!("702.41a", "702.41b", "601.2f");
    ruling!(
        "Pearl-Ear, Imperial Advisor",
        "If you put an Aura on an opponent's permanent, you still control the Aura, and it still counts for your spells that have affinity for Auras."
    );
    ruling!(
        "Pearl-Ear, Imperial Advisor",
        "If a spell has multiple instances of affinity, each one applies."
    );
    supported("Mirror Gallery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pearl-Ear, Imperial Advisor");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.battlefield(P0, "Pacifism");
    assert!(t.g.attach(pacifism, Entity::Object(theirs)));
    t.g.recompute();
    // Glorious Anthem ({1}{W}{W}) costs {W}{W}.
    t.lands(P0, "Plains", 2);
    let anthem = t.hand(P0, "Glorious Anthem");
    assert!(t.cast(P0, anthem).try_go().is_ok());
    t.resolve_all();
    // Two Pearl-Ears (no legend rule with Mirror Gallery) and two Auras: {4} less.
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Pearl-Ear, Imperial Advisor");
    let mine = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(mine, Entity::Object(theirs)));
    t.g.recompute();
    t.settle();
    assert_eq!(
        t.named_on_battlefield("Pearl-Ear, Imperial Advisor").len(),
        2
    );
    // Two instances of affinity for Auras, two Auras: Sigil of the Empty Throne
    // ({4}{W}{W}) costs {W}{W}.
    t.lands(P0, "Plains", 2);
    let big = t.hand(P0, "Sigil of the Empty Throne");
    assert!(t.cast(P0, big).try_go().is_ok());
}

// ---------------------------------------------------------------------------
// Mentor of the Meek
// ---------------------------------------------------------------------------

#[test]
fn mentor_of_the_meek_sees_effects_that_apply_as_the_creature_enters() {
    cr!("603.6a", "603.2");
    ruling!(
        "Mentor of the Meek",
        "If a creature enters with +1/+1 counters or a continuous effect such as that of Wedding Festivity will apply to the creature on the battlefield, those effects apply when checking to see if Mentor of the Meek's ability will trigger."
    );
    supported("Mentor of the Meek");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1, "a 2/2 triggers it");
    t.g.stack.clear();
    // An anthem like Wedding Festivity's applies as it enters: a 3/3 doesn't.
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0, "a 3/3 with an anthem doesn't");
    // Nor does a 1/1 entering with two +1/+1 counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    let id =
        t.g.create_card_object(card("Llanowar Elves"), P0, Zone::Nowhere);
    let mut etb = mtg_engine::replacement::EtbInfo {
        controller: Some(P0),
        ..Default::default()
    };
    etb.counters.push((counters::PLUS1.into(), 2));
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb,
        source: None,
    });
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0, "a 3/3 with counters doesn't");
}

#[test]
fn mentor_of_the_meek_checks_power_only_as_it_enters() {
    cr!("603.6a", "603.2");
    ruling!(
        "Mentor of the Meek",
        "Mentor of the Meek's ability checks the power of the other creature only as it enters."
    );
    ruling!(
        "Mentor of the Meek",
        "While resolving the triggered ability of Mentor of the Meek, you can't pay {1} multiple times to draw more than one card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mentor of the Meek");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Pumped above 2 power before the ability resolves: it still resolves.
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    t.lands(P0, "Plains", 5);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(
        t.hand_size(P0),
        hand + 1,
        "one card, however much mana there is"
    );
    // A 3/3 shrunk to 2 power after entering doesn't trigger it.
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(-2), Value::c(0))],
    );
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------
// Lifetap, High Tide
// ---------------------------------------------------------------------------

#[test]
fn lifetap_gains_one_life_for_each_forest_tapped() {
    cr!("603.2");
    ruling!(
        "Lifetap",
        "Gives one life for each and every Forest tapped."
    );
    supported("Lifetap");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifetap");
    let forests = t.lands(P1, "Forest", 3);
    let mine = t.battlefield(P0, "Forest");
    for f in &forests {
        t.g.tap(*f);
    }
    t.g.tap(mine);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn lifetap_affects_every_opponent_in_multiplayer() {
    cr!("603.2", "102.2");
    ruling!("Lifetap", "In multi-player games it affects all opponents.");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Lifetap");
    let a = t.battlefield(P1, "Forest");
    let b = t.battlefield(P2, "Forest");
    t.g.tap(a);
    t.g.tap(b);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn high_tide_applies_to_any_island_even_one_that_enters_later() {
    cr!("305.6", "611.2c", "106.12a");
    ruling!(
        "High Tide",
        "The delayed triggered ability refers to any land with the land type Island, not just those named Island."
    );
    ruling!(
        "High Tide",
        "The effect applies even if the Island in question entered the battlefield after High Tide resolved."
    );
    supported("High Tide");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let tide = t.hand(P0, "High Tide");
    t.cast(P0, tide).go();
    t.resolve_all();
    let vol = t.battlefield(P0, "Volcanic Island");
    assert!(tap_for_mana(&mut t, P0, vol, "{U}"));
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::U), 2);
    // Another player's Island too ("a player taps an Island").
    let theirs = t.battlefield(P1, "Island");
    assert!(tap_for_mana(&mut t, P1, theirs, "{U}"));
    t.resolve_all();
    assert_eq!(pool(&t, P1, ManaType::U), 2);
}

// ---------------------------------------------------------------------------
// Protection from a player, discard punishment, extra turns
// ---------------------------------------------------------------------------

#[test]
fn true_name_nemesis_has_protection_from_everything_that_player_controls_or_owns() {
    cr!("702.16k", "702.16e", "702.16b", "108.4a");
    ruling!(
        "True-Name Nemesis",
        "Protection from a player means that True-Name Nemesis has protection from each object controlled by that player. If an object has no controller (such as a card in a graveyard), its owner is considered its controller for this purpose."
    );
    supported("True-Name Nemesis");
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let tnn = t.enter(P0, "True-Name Nemesis");
    t.settle();
    let tnn = t.g.current(tnn);
    // P1's spell can't target it.
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    let _ = t.cast(P1, bolt).target(P0).try_go();
    let offered = target_candidates(&t, P1, from);
    assert!(!offered.is_empty() && !offered[0].contains(&Entity::Object(tnn)));
    assert!(offered[0].contains(&Entity::Player(P0)));
    // Damage from P1's permanent, and from a card in P1's graveyard, is prevented.
    let goblin = t.battlefield(P1, "Raging Goblin");
    let in_gy = t.graveyard(P1, "Raging Goblin");
    t.g.deal_damage(goblin, Entity::Object(tnn), 3, false);
    t.g.deal_damage(in_gy, Entity::Object(tnn), 3, false);
    t.settle();
    assert!(t.on_battlefield(tnn));
    assert_eq!(t.obj_now(tnn).damage, 0);
    // P0's own source isn't stopped.
    let mine = t.battlefield(P0, "Raging Goblin");
    t.g.deal_damage(mine, Entity::Object(tnn), 1, false);
    t.settle();
    assert!(!t.on_battlefield(tnn));
}

#[test]
fn psychic_purge_life_loss_isnt_damage() {
    cr!("120.3", "119.3", "702.8a");
    ruling!(
        "Psychic Purge",
        "The loss of life can't be prevented by any means. It is not damage."
    );
    supported("Psychic Purge");
    let mut t = TestGame::new(2);
    t.hand(P0, "Psychic Purge");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Psychic Purge"));
    assert_eq!(t.life(P1), 15);
    assert!(!t
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Damage { target: Entity::Player(p), .. } if *p == P1)));
}

#[test]
fn xantid_swarms_trigger_can_be_responded_to() {
    cr!("603.3", "117.3a");
    ruling!(
        "Xantid Swarm",
        "The defending player may cast spells before Xantid Swarm’s triggered ability resolves."
    );
    supported("Xantid Swarm");
    let mut t = TestGame::new(2);
    let swarm = t.battlefield(P0, "Xantid Swarm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(swarm, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_ok());
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
}

#[test]
fn faithless_looting_draws_and_discards_in_one_resolution() {
    cr!("608.2c", "117.1");
    ruling!(
        "Faithless Looting",
        "You draw two cards and discard two cards all while Faithless Looting is resolving."
    );
    supported("Faithless Looting");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let looting = t.hand(P0, "Faithless Looting");
    let hand = t.hand_size(P0);
    t.cast(P0, looting).go();
    t.settle();
    t.g.resolve_top();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn seedtime_counts_blue_spells_an_opponent_cast_even_if_unresolved() {
    cr!("601.2", "500.7");
    ruling!(
        "Seedtime",
        "You only get an extra turn if an opponent cast a blue spell this turn prior to this spell resolving. Note that \"casting\" a spell only requires it to be announced, so an unresolved spell still counts as having been cast."
    );
    supported("Seedtime");
    let mut t = TestGame::new(2);
    // No blue spell: no extra turn.
    t.lands(P0, "Forest", 2);
    let seed = t.hand(P0, "Seedtime");
    t.cast(P0, seed).go();
    t.resolve_all();
    assert!(t.g.extra_turns.is_empty());
    // P1's blue spell is still on the stack (or countered) when Seedtime resolves.
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    t.lands(P0, "Forest", 2);
    let seed = t.hand(P0, "Seedtime");
    t.cast(P0, seed).go();
    t.resolve();
    assert_eq!(t.g.extra_turns, vec![P0]);
    // P0's own blue spells don't count.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    let seed = t.hand(P0, "Seedtime");
    t.cast(P0, seed).go();
    t.resolve_all();
    assert!(t.g.extra_turns.is_empty());
    // With several opponents, a blue spell cast by any of them counts.
    let mut t = TestGame::new(3);
    t.lands(P2, "Island", 1);
    let opt = t.hand(P2, "Opt");
    t.cast(P2, opt).go();
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    let seed = t.hand(P0, "Seedtime");
    t.cast(P0, seed).go();
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0]);
}

// ---------------------------------------------------------------------------
// Guardian Project
// ---------------------------------------------------------------------------

#[test]
fn guardian_project_checks_the_name_on_entering_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Guardian Project",
        "Whether the entering creature shares a name with a creature you control or a creature card in your graveyard is checked both as that creature enters and as Guardian Project's ability resolves."
    );
    supported("Guardian Project");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Project");
    // The first Grizzly Bears: it triggers and draws.
    let hand = t.hand_size(P0);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A second one: it doesn't trigger at all.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // An opponent's creature with that name doesn't matter.
    t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A Hill Giant card reaches P0's graveyard before it resolves: no card.
    t.graveyard(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn guardian_project_the_entering_creature_itself_in_the_graveyard_shares_its_name() {
    cr!("603.4", "400.7");
    ruling!(
        "Guardian Project",
        "If the entering creature is put into your graveyard while Guardian Project's ability is on the stack, that same card will be a creature card in your graveyard that shares a name with the creature that was on the battlefield, so you won't draw a card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Project");
    let elves = t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(elves, None);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn guardian_project_a_returned_creature_is_a_new_object_sharing_the_name() {
    cr!("603.4", "400.7");
    ruling!(
        "Guardian Project",
        "If the entering creature leaves the battlefield and returns while Guardian Project's ability is on the stack, that same card will be a new creature you control that shares a name with the creature that was on the battlefield, so you won't draw a card. However, Guardian Project's ability may trigger for the new creature and you may draw a card as that ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Project");
    let elves = t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    // Flickered in response: the new Elves triggers it again.
    let exiled =
        t.g.move_object(
            elves,
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            None,
        )
        .expect("exiled");
    t.g.move_object(
        exiled,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .expect("returned");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The new trigger resolves first and draws; the first one doesn't.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Apostle's Blessing
// ---------------------------------------------------------------------------

/// P0 casts Apostle's Blessing on `target`; returns the spell.
fn bless(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Apostle's Blessing");
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Apostle's Blessing");
    t.cast(P0, spell).target(target).go()
}

#[test]
fn apostles_blessing_from_artifacts_unattaches_equipment() {
    cr!("702.16c", "704.5n");
    ruling!(
        "Apostle's Blessing",
        "Any Equipment attached to a creature that gains protection from artifacts will become unattached."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    bless(&mut t, bears);
    // "artifact" is the first option.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn apostles_blessing_targets_on_casting_and_chooses_the_quality_on_resolution() {
    cr!("601.2c", "608.2c", "702.16e");
    ruling!(
        "Apostle's Blessing",
        "You choose the target as part of casting the spell. You choose what attribute the target gains protection from when the spell resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    bless(&mut t, bears);
    t.settle();
    let chose_one = |t: &TestGame| {
        t.asked()[from..].iter().any(|(_, d)| {
            matches!(d, mtg_engine::decision::Decision::ChooseOption { prompt, .. } if prompt == "Choose one")
        })
    };
    assert!(!chose_one(&t));
    // Red is the fifth option (after "artifact", white, blue, black).
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.resolve_all();
    assert!(chose_one(&t));
    // Damage from a red source is prevented; a green one isn't.
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.g.deal_damage(goblin, Entity::Object(bears), 3, false);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.deal_damage(elves, Entity::Object(bears), 1, false);
    t.settle();
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn jeweled_spirit_protection_is_only_from_the_quality_chosen_this_time() {
    cr!("702.16a", "702.16e", "608.2h");
    supported("Jeweled Spirit");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Jeweled Spirit");
    t.lands(P0, "Forest", 4);
    // This turn: protection from red (the fifth option).
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, spirit, 0, &[]).expect("activate");
    t.resolve_all();
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.g.deal_damage(goblin, Entity::Object(spirit), 1, false);
    t.settle();
    assert_eq!(t.obj_now(spirit).damage, 0);
    // Next turn: protection from artifacts. Red damage isn't prevented any more.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, spirit, 0, &[]).expect("activate");
    t.resolve_all();
    let ornithopter = t.battlefield(P1, "Ornithopter");
    t.g.deal_damage(ornithopter, Entity::Object(spirit), 1, false);
    t.settle();
    assert_eq!(t.obj_now(spirit).damage, 0, "artifact damage is prevented");
    t.g.deal_damage(goblin, Entity::Object(spirit), 1, false);
    t.settle();
    assert_eq!(t.obj_now(spirit).damage, 1, "red damage isn't");
}
