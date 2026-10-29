//! Rulings batch P010 — "burn player-each": damage dealt to each creature and each player
//! (or each opponent), and the abilities around it.

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn pestilence_survives_the_end_step_while_an_animated_artifact_is_a_creature() {
    cr!("603.4", "514.2");
    ruling!(
        "Pestilence",
        "Note that \"until end of turn\" effects wear off after \"at the beginning of the end step\" triggered abilities"
    );
    for name in ["Pestilence", "Pyrohemia"] {
        supported(name);
        // Mishra's Factory is a creature until end of turn: at the beginning of the end
        // step a creature is on the battlefield, so the sacrifice doesn't trigger, and the
        // effect only wears off in the cleanup step.
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, name);
        let factory = t.battlefield(P0, "Mishra's Factory");
        t.lands(P0, "Wastes", 1);
        activate_containing(&mut t, P0, factory, "becomes").unwrap();
        t.resolve_all();
        assert!(t.obj_now(factory).is(CardType::Creature));
        t.advance_to(P1, Step::Upkeep);
        assert!(!t.obj_now(factory).is(CardType::Creature));
        assert!(t.on_battlefield(e), "{name} was sacrificed");
        // Without the animated Factory, it's sacrificed at the end step.
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, name);
        t.battlefield(P0, "Mishra's Factory");
        t.advance_to(P1, Step::Upkeep);
        assert!(!t.on_battlefield(e), "{name} should be sacrificed");
    }
}

#[test]
fn pestilence_demon_damages_itself() {
    cr!("120.3a", "120.3e");
    ruling!(
        "Pestilence Demon",
        "Pestilence Demon deals 1 damage to itself, as well as to each other creature and each player."
    );
    supported("Pestilence Demon");
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Pestilence Demon");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, demon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(dmg(&t, demon), 1);
    assert_eq!(dmg(&t, bear), 1);
    assert_eq!((t.life(P0), t.life(P1)), (19, 19));
}

/// P0's Blim attacks P1 (who already controls a Forest P0 owns, and P0 controls P1's
/// Grizzly Bears); its trigger gives P1 control of P0's Mountain.
fn blim_hits(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let blim = t.battlefield(P0, "Blim, Comedic Genius");
    let forest = t.battlefield(P0, "Forest");
    crate::r_s06_common::give_control(t, forest, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(t, bears, P0);
    let mountain = t.battlefield(P0, "Mountain");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
        t.hand(P1, "Grizzly Bears");
    }
    t.answer_targets(P0, &[obj(mountain)]);
    t.attack(&[(blim, pl(P1))], &[]);
    (blim, forest, mountain)
}

#[test]
fn blim_counts_every_permanent_a_player_controls_but_doesnt_own() {
    cr!("108.3", "108.4");
    ruling!(
        "Blim, Comedic Genius",
        "Players lose life and discard cards for all permanents they control but don't own, not just the ones they received from you."
    );
    ruling!(
        "Blim, Comedic Genius",
        "You choose which permanent the player gains control of, not the player who was dealt damage."
    );
    ruling!(
        "Blim, Comedic Genius",
        "The player who was dealt damage can't refuse your generous gift."
    );
    supported("Blim, Comedic Genius");
    let mut t = TestGame::new(2);
    let (_, forest, mountain) = blim_hits(&mut t);
    assert_eq!(t.obj_now(mountain).controller, P1);
    assert_eq!(t.obj_now(forest).controller, P1);
    // P1 controls two permanents P0 owns: 4 combat damage, then 2 life and 2 cards.
    assert_eq!(t.life(P1), 20 - 4 - 2);
    assert_eq!(t.hand_size(P1), 1);
    // P0 controls P1's Grizzly Bears: 1 life and 1 card.
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), 2);
    // P0 chose the target; P1 was never asked anything about it.
    let asked = t.asked();
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert!(!asked
        .iter()
        .any(|(p, d)| *p == P1
            && matches!(d, Decision::ChooseTargets { .. } | Decision::YesNo { .. })));
}

#[test]
fn blims_control_change_lasts_indefinitely_until_the_controller_leaves() {
    cr!("611.2a", "800.4a");
    ruling!(
        "Blim, Comedic Genius",
        "The control-change effect of Blim's ability lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Blim leaves the battlefield. In a multiplayer game, it does expire if the current controller leaves the game."
    );
    let mut t = TestGame::new(2);
    let (blim, _, mountain) = blim_hits(&mut t);
    kill(&mut t, blim);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(mountain).controller, P1);
    // Three players: P1 leaves the game and the Mountain comes back to P0.
    let mut t = TestGame::new(3);
    let (_, _, mountain) = blim_hits(&mut t);
    assert_eq!(t.obj_now(mountain).controller, P1);
    t.g.lose_game(P1);
    t.settle();
    assert!(t.on_battlefield(mountain));
    assert_eq!(t.obj_now(mountain).controller, P0);
}

#[test]
fn sunspine_lynx_stops_prevention_but_not_protections_targeting_rules() {
    cr!("615.12", "702.16b", "702.16e");
    ruling!(
        "Sunspine Lynx",
        "Protection prevents damage, so protection will be unable to prevent damage while Sunspine Lynx is on the battlefield. However, this won’t allow spells or abilities to target permanents they wouldn’t ordinarily be able to target because of protection"
    );
    supported("Sunspine Lynx");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunspine Lynx");
    let walker = t.battlefield(P1, "Kor Firewalker");
    // Shock can't target the creature with protection from red.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast_with(P0, shock, &[obj(walker)]).ok();
    t.clear_answers();
    let stack: Vec<ObjectId> = t.stack.clone();
    assert!(stack
        .iter()
        .all(|s| !crate::r_s25_common::targets_of(&t, *s).contains(&obj(walker))));
    // But Volcanic Fallout's red damage isn't prevented by protection.
    t.g.stack.clear();
    cast_new(&mut t, P0, "Volcanic Fallout", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(walker));
    // Without the Lynx, protection prevents it.
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Kor Firewalker");
    cast_new(&mut t, P0, "Volcanic Fallout", &[]);
    t.resolve_all();
    assert_eq!(dmg(&t, walker), 0);
}

#[test]
fn sunspine_lynx_life_gain_spells_still_resolve() {
    cr!("119.7", "608.2c");
    ruling!(
        "Sunspine Lynx",
        "Spells and abilities that cause players to gain life still resolve while Sunspine Lynx is on the battlefield. No player will gain life, but any other effects of that spell or ability will happen."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunspine Lynx");
    cast_new(&mut t, P0, "Vampire's Kiss", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
}

#[test]
fn sunspine_lynx_only_stops_effects_that_prevent() {
    cr!("615.12", "614.1a");
    ruling!(
        "Sunspine Lynx",
        "Sunspine Lynx only stops damage from being prevented by effects that specifically use the word “prevent.”"
    );
    // Circle of Protection: Red's prevention does nothing (it works without the Lynx).
    for lynx in [true, false] {
        let mut t = TestGame::new(2);
        if lynx {
            t.battlefield(P0, "Sunspine Lynx");
        }
        let cop = t.battlefield(P1, "Circle of Protection: Red");
        t.lands(P1, "Wastes", 1);
        let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
        t.answer_choose(P1, &[obj(bolt)]);
        t.activate(P1, cop, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P1), if lynx { 17 } else { 20 });
    }
    // ... but Benevolent Unicorn's "deals that much damage minus 1 instead" isn't
    // prevention, so it still applies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunspine Lynx");
    t.battlefield(P1, "Benevolent Unicorn");
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tainted_specter_makes_the_player_discard() {
    cr!("701.9a");
    ruling!(
        "Tainted Specter",
        "The current wording returns it to being a discard, so it will trigger abilities appropriately."
    );
    supported("Tainted Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Tainted Specter");
    t.battlefield(P0, "Megrim");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P1, false);
    t.answer_choose(P1, &[obj(card)]);
    t.activate(P0, specter, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Megrim saw the discard (2), and the Specter dealt 1 to each player and creature.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 19);
    assert_eq!(dmg(&t, bear), 1);
    assert_eq!(dmg(&t, specter), 1);
}

#[test]
fn arcbond_creature_is_the_source_of_the_new_damage() {
    cr!("702.15b", "702.2b");
    ruling!("Arcbond", "The creature is the source of the new damage.");
    supported("Arcbond");
    // Vampire Nighthawk (deathtouch, lifelink) is dealt 2 damage: its own 2 damage to
    // each other creature and each player has deathtouch and lifelink.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Arcbond", &[obj(hawk)]);
    t.resolve_all();
    cast_new(&mut t, P1, "Shock", &[obj(hawk)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "deathtouch from the Nighthawk");
    assert_eq!(dmg(&t, hawk), 2, "it doesn't deal damage to itself");
    assert_eq!(t.life(P1), 18);
    // P0 is dealt 2 and gains 6 (2 each to the Wurm, P0 and P1).
    assert_eq!(t.life(P0), 24);
}

#[test]
fn arcbond_triggers_even_if_the_creature_is_dealt_lethal_damage() {
    cr!("603.10a", "603.7c");
    ruling!(
        "Arcbond",
        "The delayed triggered ability Arcbond creates will trigger even if the target creature is dealt lethal damage."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Arcbond", &[obj(bears)]);
    t.resolve_all();
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(dmg(&t, wurm), 3);
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
}

#[test]
fn rivalry_rechecks_the_land_count_on_resolution() {
    cr!("603.4");
    ruling!(
        "Rivalry",
        "This ability will only trigger if the player controls more lands at the beginning of upkeep. This condition is checked again at the start of resolution, and if not true, then nothing happens."
    );
    supported("Rivalry");
    for lose_a_land in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rivalry");
        t.lands(P0, "Mountain", 2);
        let lands = t.lands(P1, "Forest", 3);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if lose_a_land {
            kill(&mut t, lands[0]);
            kill(&mut t, lands[1]);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if lose_a_land { 20 } else { 18 });
    }
    // P0 doesn't control more lands than P1 at P0's upkeep: it doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rivalry");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Forest", 2);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn power_surge_knows_the_untapped_lands_from_the_beginning_of_the_turn() {
    cr!("502.1", "702.26a");
    ruling!(
        "Power Surge",
        "This card knows how many were untapped even if it was not on the battlefield at the beginning of the turn."
    );
    supported("Power Surge");
    // P1's Power Surge is phased out and phases in during P1's untap step, after the turn
    // began. Two of P1's three lands were tapped as the turn began.
    let mut t = TestGame::new(2);
    let surge = t.battlefield(P1, "Power Surge");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![surge]);
    let lands = t.lands(P1, "Mountain", 3);
    t.g.tap(lands[0]);
    t.g.tap(lands[1]);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(surge).phased_out);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn acidic_soil_counts_lands_on_resolution() {
    cr!("608.2h");
    ruling!("Acidic Soil", "This counts the lands on resolution.");
    supported("Acidic Soil");
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    cast_new(&mut t, P0, "Acidic Soil", &[]);
    // P0 has the 3 lands it paid with; P1 gets a third land in response.
    add(&mut t, P1, "Forest");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
}

#[test]
fn volcanic_fallout_can_be_targeted_by_a_counterspell() {
    cr!("101.2", "608.2c");
    ruling!(
        "Volcanic Fallout",
        "Volcanic Fallout can be targeted by spells that try to counter it (such as Countersquall). Those spells will still resolve, but the part of their effect that would counter Volcanic Fallout won't do anything."
    );
    supported("Volcanic Fallout");
    supported("Countersquall");
    let mut t = TestGame::new(2);
    let fallout = cast_new(&mut t, P0, "Volcanic Fallout", &[]);
    cast_new(&mut t, P1, "Countersquall", &[obj(fallout)]);
    t.resolve();
    // Countersquall resolved: Volcanic Fallout's controller lost 2 life, and it's still
    // on the stack.
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 18));
}

fn ashling_activate(t: &mut TestGame, p: PlayerId, ashling: ObjectId) {
    t.lands(p, "Mountain", 2);
    t.activate(p, ashling, 0, &[]).unwrap();
}

#[test]
fn ashling_counts_resolutions_of_that_ability_from_that_creature() {
    cr!("603.7h", "707.10");
    ruling!(
        "Ashling the Pilgrim",
        "When the ability resolves, it counts the number of times that same ability from that this creature has already resolved that turn. It doesn't matter who controlled the creature or the previous abilities when they resolved."
    );
    ruling!(
        "Ashling the Pilgrim",
        "You get the bonus only the third time the ability resolves. You won't get the bonus the fourth, fifth, sixth, or any subsequent times."
    );
    supported("Ashling the Pilgrim");
    // The third resolution: 3 counters removed, 3 damage to each creature and player;
    // the control change in between doesn't matter.
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    crate::r_s06_common::attach_new(&mut t, P0, "Indestructibility", ashling);
    let wurm = t.battlefield(P1, "Craw Wurm");
    for _ in 0..2 {
        ashling_activate(&mut t, P0, ashling);
        t.resolve_all();
    }
    crate::r_s06_common::give_control(&mut t, ashling, P1);
    ashling_activate(&mut t, P1, ashling);
    t.resolve_all();
    assert_eq!(t.counters(ashling, counters::PLUS1), 0);
    assert_eq!(dmg(&t, wurm), 3);
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
    // The fourth resolution only adds a counter.
    ashling_activate(&mut t, P1, ashling);
    t.resolve_all();
    assert_eq!(t.counters(ashling, counters::PLUS1), 1);
    assert_eq!(dmg(&t, wurm), 3);
}

#[test]
fn ashling_counts_copies_of_the_ability() {
    cr!("603.7h", "707.10");
    ruling!(
        "Ashling the Pilgrim",
        "A copy of this ability (created by Rings of Brighthearth, for example) will count toward the total."
    );
    supported("Rings of Brighthearth");
    // The first activation is copied: two resolutions. The second activation's is the
    // third.
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    t.battlefield(P0, "Rings of Brighthearth");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    ashling_activate(&mut t, P0, ashling);
    t.resolve_all();
    assert_eq!(t.counters(ashling, counters::PLUS1), 2);
    t.answer_yes(P0, false);
    ashling_activate(&mut t, P0, ashling);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ashling_doesnt_count_countered_abilities_or_other_ashlings() {
    cr!("603.7h", "701.6b");
    ruling!(
        "Ashling the Pilgrim",
        "Abilities from other creatures with the same name don't count towards the total. Neither does an ability that's been countered."
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    let other = t.battlefield(P1, "Ashling the Pilgrim");
    // P1's Ashling's ability resolves twice; P0's twice, plus one that's countered.
    for _ in 0..2 {
        ashling_activate(&mut t, P1, other);
        t.resolve_all();
        ashling_activate(&mut t, P0, ashling);
        t.resolve_all();
    }
    ashling_activate(&mut t, P0, ashling);
    let ab = *t.stack.last().unwrap();
    cast_new(&mut t, P1, "Stifle", &[obj(ab)]);
    t.resolve_all();
    assert_eq!(t.counters(ashling, counters::PLUS1), 2);
    assert_eq!(t.life(P0), 20);
    // The next one is the third resolution.
    ashling_activate(&mut t, P0, ashling);
    t.resolve_all();
    assert_eq!(t.counters(ashling, counters::PLUS1), 0);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn hurricane_and_earthquake_check_flying_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Hurricane",
        "Whether or not a creature has flying is only checked on resolution."
    );
    ruling!(
        "Earthquake",
        "Whether or not a creature is without flying is only checked on resolution."
    );
    supported("Hurricane");
    supported("Earthquake");
    for name in ["Hurricane", "Earthquake"] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Wastes", 2);
        crate::r_s25_common::lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        t.cast(P0, card).x(2).go();
        // The Wurm gains flying in response.
        cast_new(&mut t, P1, "Jump", &[obj(wurm)]);
        t.resolve_all();
        let expected = if name == "Hurricane" { 2 } else { 0 };
        assert_eq!(dmg(&t, wurm), expected, "{name}");
        assert_eq!(t.life(P1), 18, "{name}");
    }
}

#[test]
fn searing_rays_color_is_chosen_on_resolution() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Searing Rays",
        "You choose the color on resolution. Your opponent does not get to respond between the color choice and the damage happening."
    );
    supported("Searing Rays");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    cast_new(&mut t, P0, "Searing Rays", &[]);
    let asked = t.asked().len();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(
            // Green.
            4,
        ),
    );
    t.resolve();
    let since = crate::r_s01_common::asked_since(&t, asked);
    // The color was chosen during resolution, with no priority until it was done.
    let first_choice = since
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseOption { .. }));
    assert!(first_choice.is_some(), "{since:?}");
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 19);
}
