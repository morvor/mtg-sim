//! Durations, grammar V (cards4-triggers-durations): durations ending at a step of a later
//! turn (Orcish Farmer, Halfdane, Xenic Poltergeist) and "for as long as" conditions about
//! the objects an instruction names (Zygon Infiltrator, Old Man of the Sea).

use crate::combat2_common::supported;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn untapped(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

#[test]
fn orcish_farmer_swamp_lasts_until_the_lands_controllers_next_untap_step() {
    cr!("611.2a", "500.4", "305.7");
    ruling!("Orcish Farmer", "This ability changes the targeted land’s land types.");
    interp!("until-its-controllers-next-step");
    supported("Orcish Farmer");
    let mut t = TestGame::new(2);
    let farmer = t.battlefield(P0, "Orcish Farmer");
    untapped(&mut t, farmer);
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, farmer, 0, &[Entity::Object(forest)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    assert!(!t.obj_now(forest).chars.has_subtype("Forest"));
    // Through the rest of P0's turn; it ends as P1's untap step begins.
    t.advance_to(P0, Step::End);
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    assert!(!t.obj_now(forest).chars.has_subtype("Swamp"));
    // A land P0 controls stays a Swamp through P1's turn, until P0's next untap step.
    let farmer = t.g.current(farmer);
    untapped(&mut t, farmer);
    t.activate(P0, farmer, 0, &[Entity::Object(mine)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::End);
    assert!(t.obj_now(mine).chars.has_subtype("Swamp"));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(mine).chars.has_subtype("Forest"));
}

#[test]
fn xenic_poltergeist_animation_wears_off_as_your_next_upkeep_begins() {
    cr!("611.2a", "500.4");
    ruling!("Xenic Poltergeist", "Wears off at the beginning of the upkeep");
    supported("Xenic Poltergeist");
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Xenic Poltergeist");
    untapped(&mut t, ghost);
    let ring = t.battlefield(P1, "Sol Ring");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, ghost, 0, &[Entity::Object(ring)]).unwrap();
    t.resolve_all();
    // An artifact creature with power and toughness equal to its mana value.
    assert!(t.obj_now(ring).chars.card_types.contains(CardType::Creature));
    assert_eq!(t.pt(ring), (1, 1));
    t.advance_to(P1, Step::End);
    assert!(t.obj_now(ring).chars.card_types.contains(CardType::Creature));
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(ring).chars.card_types.contains(CardType::Creature));
}

#[test]
fn halfdane_copies_power_and_toughness_until_the_end_of_your_next_upkeep() {
    cr!("611.2a", "500.5", "613.4b", "608.2h");
    ruling!(
        "Halfdane",
        "the effect from the previous turn's ability will continue to work"
    );
    supported("Halfdane");
    let mut t = TestGame::new(2);
    let halfdane = t.battlefield(P0, "Halfdane");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(halfdane), (2, 2));
    // Fixed as the effect began: a counter on the Bears doesn't change it.
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.g.recompute();
    assert_eq!(t.pt(halfdane), (2, 2));
    // Not over as this upkeep ends: through the opponent's turn.
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.pt(halfdane), (2, 2));
    t.advance_to(P1, Step::End);
    assert_eq!(t.pt(halfdane), (2, 2));
    // P0's next upkeep: the new trigger has no legal target (the Bears are gone), so the
    // old effect lasts until that upkeep ends.
    t.g.destroy(bears, None);
    t.g.settle();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.pt(halfdane), (2, 2));
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.pt(halfdane), (3, 3));
}

#[test]
fn zygon_infiltrator_is_a_copy_for_as_long_as_that_creature_remains_tapped() {
    cr!("611.2b", "707.2");
    ruling!(
        "Zygon Infiltrator",
        "When Zygon Infiltrator becomes a copy of a creature, it's neither entering nor leaving"
    );
    supported("Zygon Infiltrator");
    let mut t = TestGame::new(2);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, zygon, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(zygon), (3, 3));
    // Untapping the creature ends the effect for good, even if it's tapped again.
    t.g.objects[giant.0 as usize].tapped = false;
    t.g.recompute();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
    t.g.objects[giant.0 as usize].tapped = true;
    t.g.recompute();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
}

#[test]
fn zygon_infiltrator_copy_ends_when_that_creature_leaves() {
    cr!("611.2b", "400.7");
    let mut t = TestGame::new(2);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.activate(P0, zygon, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Hill Giant");
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(t.obj_now(zygon).chars.name.as_str(), "Zygon Infiltrator");
}

#[test]
fn old_man_of_the_sea_keeps_control_while_tapped_and_bigger() {
    cr!("611.2b");
    ruling!(
        "Old Man of the Sea",
        "The ability doesn’t care whether Old Man of the Sea remains under your control"
    );
    supported("Old Man of the Sea");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Old Man of the Sea");
    untapped(&mut t, old);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // The Bears' power grows past the Old Man's (2): control returns, for good.
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
    t.g.objects[bears.0 as usize].counters.clear();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn old_man_of_the_sea_control_ends_when_it_untaps() {
    cr!("611.2b");
    ruling!(
        "Old Man of the Sea",
        "You lose control of the creature if the Old Man leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Old Man of the Sea");
    untapped(&mut t, old);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P0, "still tapped");
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(old).tapped);
    assert_eq!(t.obj_now(bears).controller, P1);
    // Leaving the battlefield ends it too.
    let old = t.g.current(old);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, old, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.g.destroy(old, None);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn assimilation_aegis_copy_lasts_while_it_remains_attached() {
    cr!("611.2b", "707.2", "607.2a");
    ruling!(
        "Assimilation Aegis",
        "copies exactly what was printed on the original creature card"
    );
    supported("Assimilation Aegis");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let aegis = t.enter(P0, "Assimilation Aegis");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.g.attach(aegis, Entity::Object(bears));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    // Moved to another creature: the first one is itself again.
    t.g.attach(aegis, Entity::Object(wall));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.obj_now(wall).chars.name.as_str(), "Hill Giant");
    // Reattaching starts a new effect.
    t.g.attach(aegis, Entity::Object(bears));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.obj_now(wall).chars.name.as_str(), "Wall of Stone");
}

#[test]
fn blade_of_shared_souls_copy_ends_when_it_becomes_unattached() {
    cr!("611.2b", "707.2");
    ruling!(
        "Blade of Shared Souls",
        "The copiable values granted by Blade of Shared Souls are locked in as its effect begins to apply"
    );
    supported("Blade of Shared Souls");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Blade of Shared Souls");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.g.attach(blade, Entity::Object(bears));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Hill Giant");
    // Locked in: the copied creature leaving doesn't matter.
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Hill Giant");
    t.g.unattach(blade);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Grizzly Bears");
}

#[test]
fn kitesail_larcenist_makes_treasures_while_it_remains_on_the_battlefield() {
    cr!("611.2b", "205.1a", "613.1f");
    ruling!(
        "Kitesail Larcenist",
        "will be only Treasure artifacts as long as Kitesail Larcenist remains on the battlefield"
    );
    supported("Kitesail Larcenist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let kite = t.enter(P0, "Kitesail Larcenist");
    t.resolve_all();
    for o in [bears, giant] {
        let c = &t.obj_now(o).chars;
        assert!(c.has_subtype("Treasure"), "{:?}", c.name);
        assert!(!c.card_types.contains(CardType::Creature));
        assert!(c.card_types.contains(CardType::Artifact));
    }
    t.g.destroy(kite, None);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.card_types.contains(CardType::Creature));
    assert!(!t.obj_now(giant).chars.has_subtype("Treasure"));
}

#[test]
fn kitesail_larcenist_gone_before_its_ability_resolves_does_nothing() {
    cr!("611.2b");
    ruling!(
        "Kitesail Larcenist",
        "If Kitesail Larcenist leaves the battlefield before its enters-the-battlefield ability resolves"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let kite = t.enter(P0, "Kitesail Larcenist");
    t.g.settle();
    t.g.destroy(kite, None);
    t.resolve_all();
    assert!(!t.obj_now(giant).chars.has_subtype("Treasure"));
    assert!(t.obj_now(giant).chars.card_types.contains(CardType::Creature));
}

#[test]
fn sen_triplets_locks_the_opponent_and_lets_you_play_from_their_hand() {
    cr!("611.2a", "305.1", "601.2a");
    ruling!(
        "Sen Triplets",
        "Sen Triplets doesn't allow you to play more than one land during your turn"
    );
    supported("Sen Triplets");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sen Triplets");
    let bolt = t.hand(P1, "Lightning Bolt");
    let forest = t.hand(P1, "Forest");
    let mountain = t.hand(P1, "Mountain");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // The opponent can't cast spells this turn.
    mana(&mut t, P1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_err());
    t.g.players[1].mana_pool = Default::default();
    // P0 casts the opponent's Bolt and plays one of their lands.
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "its owner's graveyard");
    t.play_land(P0, forest).unwrap();
    let f = t.g.current(forest);
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).controller, P0);
    assert!(t.play_land(P0, mountain).is_err(), "one land a turn");
    // Only this turn.
    let shock = t.hand(P1, "Shock");
    t.advance_to(P1, Step::PrecombatMain);
    mana(&mut t, P0);
    assert!(t.cast_with(P0, shock, &[Entity::Player(P1)]).is_err());
}

#[test]
fn oracle_en_vec_chosen_creatures_must_attack_and_the_others_cant() {
    cr!("508.1c", "508.1d", "603.7a", "611.2a");
    ruling!(
        "Oracle en-Vec",
        "Creatures which are chosen to attack, but do not attack, are destroyed at the end of that turn"
    );
    interp!("next-turn-effects-start-with-the-turn");
    supported("Oracle en-Vec");
    let mut t = TestGame::new(2);
    let oracle = t.battlefield(P0, "Oracle en-Vec");
    untapped(&mut t, oracle);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(giant)]);
    t.activate(P0, oracle, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // Nothing yet during P0's turn; during P1's next turn the Bears and the Giant must
    // attack, and the Wurm can't.
    t.advance_to(P1, Step::BeginningOfCombat);
    t.g.objects[giant.0 as usize].tapped = true;
    assert!(!crate::combat2_common::legal_attack(&mut t, &[]));
    assert!(!crate::combat2_common::legal_attack(
        &mut t,
        &[(bears, Entity::Player(P0)), (wurm, Entity::Player(P0))]
    ));
    assert!(crate::combat2_common::legal_attack(
        &mut t,
        &[(bears, Entity::Player(P0))]
    ));
    crate::combat2_common::declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    // The Giant (chosen, tapped, didn't attack) is destroyed; the Bears attacked; the Wurm
    // wasn't chosen.
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears) || t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn giant_oyster_shrinks_the_creature_while_it_remains_tapped() {
    cr!("603.7b", "603.7c", "611.2b", "502.3");
    ruling!(
        "Giant Oyster",
        "a repeating delayed triggered ability that’s in effect only as long as Giant Oyster remains tapped"
    );
    supported("Giant Oyster");
    let mut t = TestGame::new(2);
    let oyster = t.battlefield(P0, "Giant Oyster");
    untapped(&mut t, oyster);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.objects[wurm.0 as usize].tapped = true;
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, oyster, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    // The Wurm doesn't untap during P1's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(wurm).tapped);
    // P0 keeps the Oyster tapped: a -1/-1 counter at P0's draw step.
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "-1/-1"), 1);
    assert_eq!(t.pt(wurm), (5, 3));
    // P0 untaps the Oyster: the counters are removed, and no more come.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(wurm).tapped);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.obj_now(oyster).tapped);
    assert_eq!(t.counters(wurm, "-1/-1"), 0);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "-1/-1"), 0);
    assert!(t.g.delayed_triggers.is_empty());
    // And the Wurm untaps as usual.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(wurm).tapped);
}
