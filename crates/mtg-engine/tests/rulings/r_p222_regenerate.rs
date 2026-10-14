//! Rulings batch P222 — regeneration (CR 701.19): shields set up at any time, sacrificing
//! the creature or an Aura to pay for it, regeneration shields from spells that do
//! something else too, and regenerating noncreature permanents.

use crate::r_p205_common::is_attacking;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s17_common::{enter_transformed, name_of};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

/// The permanent was regenerated: still on the battlefield (the same object), tapped,
/// with no damage.
fn regenerated(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_live(id) && t.on_battlefield(id) && t.obj(id).tapped && t.obj(id).damage == 0
}

#[test]
fn regenerating_creatures_that_arent_at_risk() {
    cr!("701.19a");
    ruling!("Goblin Turncoat", "Goblin Turncoat can regenerate even if it isn’t in combat, it’s already tapped, or it’s undamaged.");
    ruling!("Nightscape Familiar", "Nightscape Familiar can regenerate even if it isn’t in combat, it’s already tapped, or it’s undamaged.");
    ruling!("Patchwork Gnomes", "Patchwork Gnomes can regenerate even if it isn't in combat, it's already tapped, or it's undamaged.");
    ruling!("Skithiryx, the Blight Dragon", "Skithiryx can regenerate even if it isn't in combat, it's already tapped, or it's undamaged.");
    ruling!("Spectral Lynx", "Spectral Lynx can regenerate even if it isn’t in combat, it’s already tapped, or it’s undamaged.");
    ruling!("Spiritmonger", "Spiritmonger can regenerate even if it isn't in combat, it's already tapped, or it's undamaged.");
    for name in [
        "Goblin Turncoat",
        "Nightscape Familiar",
        "Patchwork Gnomes",
        "Skithiryx, the Blight Dragon",
        "Spectral Lynx",
        "Spiritmonger",
    ] {
        supported(name);
        for shield in [true, false] {
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let c = t.battlefield(P0, name);
            // Tapped, undamaged, and not in combat.
            t.g.tap(c);
            t.lands(P0, "Swamp", 2);
            t.lands(P0, "Wastes", 1);
            let goblin = create_token(&mut t, P0, "Goblin");
            let card = t.hand(P0, "Grizzly Bears");
            if shield {
                assert!(can_activate(&mut t, P0, c), "{name}");
                if name == "Goblin Turncoat" {
                    t.answer_choose(P0, &[goblin.into()]);
                } else if name == "Patchwork Gnomes" {
                    t.answer_choose(P0, &[card.into()]);
                }
                activate_containing(&mut t, P0, c, "Regenerate").unwrap();
                t.resolve_all();
            }
            destroy(&mut t, c);
            assert_eq!(regenerated(&t, c), shield, "{name}");
            assert_eq!(t.in_graveyard(P0, name), !shield, "{name}");
        }
    }
}

#[test]
fn sacrificing_a_creature_to_its_own_regeneration_ability() {
    cr!("602.2b", "601.2h", "701.19a", "608.2b");
    ruling!("Corrupted Harvester", "You may sacrifice Corrupted Harvester to pay for its own regeneration ability. If you do, however, it won’t regenerate. It’ll just end up in its owner’s graveyard as a result of the sacrifice.");
    ruling!("Rusted Slasher", "You may sacrifice Rusted Slasher to pay for its own regeneration ability. If you do, however, it won’t regenerate. It’ll just end up in its owner’s graveyard as a result of the sacrifice.");
    ruling!("Skeletal Kathari", "You may sacrifice Skeletal Kathari to pay for its own regeneration ability. If you do, however, it won’t regenerate. It’ll just end up in its owner’s graveyard as a result of the sacrifice.");
    for name in ["Corrupted Harvester", "Rusted Slasher", "Skeletal Kathari"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = t.battlefield(P0, name);
        t.lands(P0, "Swamp", 1);
        t.answer_choose(P0, &[c.into()]);
        activate_containing(&mut t, P0, c, "Regenerate").unwrap();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert!(t.named_on_battlefield(name).is_empty(), "{name}");
    }
}

#[test]
fn gatherer_of_graces_dies_to_the_lost_aura_bonus_before_regenerating() {
    cr!("704.5g", "602.2b", "117.5");
    ruling!("Gatherer of Graces", "If Gatherer of Graces already has damage on it equal to its toughness minus 1, then sacrificing an Aura attached to it results in Gatherer of Graces being put into its owner's graveyard before the ability that would regenerate it resolves.");
    supported("Gatherer of Graces");
    for marked in [2, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let gatherer = t.battlefield(P0, "Gatherer of Graces");
        let aura = attach_new(&mut t, P0, "Pacifism", gatherer);
        assert_eq!(t.pt(gatherer), (2, 3));
        let pinger = t.battlefield(P1, "Grizzly Bears");
        damage(&mut t, pinger, marked, gatherer);
        assert!(t.on_battlefield(gatherer));
        t.answer_choose(P0, &[aura.into()]);
        activate_containing(&mut t, P0, gatherer, "Regenerate").unwrap();
        t.settle();
        if marked == 2 {
            // 1/2 with 2 damage: gone before the ability resolves.
            assert!(t.in_graveyard(P0, "Gatherer of Graces"));
            assert_eq!(t.stack_len(), 1);
        } else {
            t.resolve_all();
            destroy(&mut t, gatherer);
            assert!(regenerated(&t, gatherer));
        }
    }
}

#[test]
fn dark_dabbling_draws_as_it_resolves_and_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b", "608.2c", "701.19a");
    ruling!("Dark Dabbling", "You draw a card as Dark Dabbling resolves, not as the creature actually regenerates.");
    ruling!("Dark Dabbling", "If the target creature is an illegal target by the time Dark Dabbling tries to resolve, the ability doesn’t resolve. You don’t draw a card and no creatures are regenerated.");
    supported("Dark Dabbling");
    for spoil in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        // Spell mastery: also regenerate each other creature P0 controls.
        t.graveyard(P0, "Shock");
        t.graveyard(P0, "Shock");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        cast(&mut t, P0, "Dark Dabbling", &[bears.into()]);
        let hand = t.hand_size(P0);
        if spoil {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if spoil {
            assert_eq!(t.hand_size(P0), hand);
            destroy(&mut t, giant);
            assert!(t.in_graveyard(P0, "Hill Giant"));
        } else {
            // Drawn already, before anything regenerates.
            assert_eq!(t.hand_size(P0), hand + 1);
            destroy(&mut t, bears);
            destroy(&mut t, giant);
            assert!(regenerated(&t, bears));
            assert!(regenerated(&t, giant));
            assert_eq!(t.hand_size(P0), hand + 1);
        }
    }
}

#[test]
fn heal_the_scars_gains_life_as_it_resolves() {
    cr!("608.2c", "701.19a");
    ruling!("Heal the Scars", "You gain the life when Heal the Scars resolves, not when the creature actually regenerates.");
    supported("Heal the Scars");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    cast(&mut t, P0, "Heal the Scars", &[giant.into()]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    destroy(&mut t, giant);
    assert!(regenerated(&t, giant));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn full_moons_rise_sacrificed_before_damage_loses_its_bonus() {
    cr!("701.19a", "506.4", "510.1");
    ruling!("Full Moon's Rise", "In order to regenerate Werewolves involved in combat, you must sacrifice Full Moon’s Rise before combat damage is assigned. This means they will lose the +1/+0 and trample bonuses before combat damage assignment.");
    supported("Full Moon's Rise");
    let mut t = TestGame::new(2);
    let rise = t.battlefield(P0, "Full Moon's Rise");
    let wolf = t.battlefield(P0, "Ulvenwald Mystics");
    let wall = t.battlefield(P1, "Colossal Dreadmaw");
    assert_eq!(t.pt(wolf), (4, 3));
    assert!(t.obj(wolf).has_keyword(KeywordKind::Trample));
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(wolf, Entity::Player(P1))], &[(wall, wolf)]);
    activate_containing(&mut t, P0, rise, "Regenerate").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Full Moon's Rise"));
    // Before damage: no more bonus.
    t.g.recompute();
    assert_eq!(t.pt(wolf), (3, 3));
    assert!(!t.obj(wolf).has_keyword(KeywordKind::Trample));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(regenerated(&t, wolf));
    assert!(!is_attacking(&t, wolf));
    assert_eq!(t.obj_now(wall).damage, 3);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn boneknitter_can_regenerate_itself() {
    cr!("115.1", "701.19a");
    ruling!("Boneknitter", "It can regenerate itself.");
    supported("Boneknitter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let knitter = t.battlefield(P0, "Boneknitter");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, knitter, 0, &[knitter.into()]).unwrap();
    t.resolve_all();
    destroy(&mut t, knitter);
    assert!(regenerated(&t, knitter));
}

#[test]
fn gore_vassal_regenerates_a_creature_its_counter_would_kill() {
    cr!("701.19a", "704.5g", "608.2c");
    ruling!("Gore Vassal", "Let’s say you activate Gore Vassal’s ability targeting a creature with 3 toughness and 2 damage marked on it. The regeneration shield will be created just before the creature is destroyed for having damage marked on it equal to or greater than its toughness. The creature will then regenerate. That is, all damage will be removed from it and it will become tapped; any counters on the creature, including the -1/-1 counter put on by this ability, remain.");
    supported("Gore Vassal");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vassal = t.battlefield(P0, "Gore Vassal");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, vassal, 2, giant);
    t.activate(P0, vassal, 0, &[giant.into()]).unwrap();
    t.resolve_all();
    assert!(regenerated(&t, giant));
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn gore_vassal_targeting_itself_does_nothing() {
    cr!("602.2b", "608.2b");
    ruling!("Gore Vassal", "You can activate Gore Vassal’s ability targeting itself, but it won’t get a -1/-1 counter or regenerate because it’ll be an illegal target when the ability resolves.");
    supported("Gore Vassal");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vassal = t.battlefield(P0, "Gore Vassal");
    t.activate(P0, vassal, 0, &[vassal.into()]).unwrap();
    assert!(t.in_graveyard(P0, "Gore Vassal"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let card = t.g.current(vassal);
    assert!(t.in_graveyard(P0, "Gore Vassal"));
    assert_eq!(t.counters(card, counters::MINUS1), 0);
}

#[test]
fn reknit_regenerates_a_noncreature_permanent() {
    cr!("701.19a");
    ruling!("Reknit", "Reknit can regenerate an artifact, creature, enchantment, land, or planeswalker. If you regenerate a noncreature permanent, the next time that permanent would be destroyed that turn, instead tap it. If that permanent had any damage on it (because it had been a creature earlier in the turn), that damage is removed.");
    supported("Reknit");
    for name in ["Forest", "Mind Stone", "Glorious Anthem", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let p = t.battlefield(P0, name);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 1);
        let reknit = t.hand(P0, "Reknit");
        // Damage left over from when it was a creature earlier in the turn.
        t.g.obj_mut(p).damage = 1;
        t.cast(P0, reknit).target(p).go();
        t.resolve_all();
        destroy(&mut t, p);
        assert!(regenerated(&t, p), "{name}");
    }
}

#[test]
fn duskworker_is_removed_from_combat_only_when_it_regenerates() {
    cr!("701.19a", "506.4");
    ruling!("Duskworker", "Setting up the regeneration shield doesn't remove Duskworker from combat. However, Duskworker is removed from combat if it would be destroyed and then regenerates.");
    supported("Duskworker");
    let mut t = TestGame::new(2);
    let dusk = t.battlefield(P0, "Duskworker");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(dusk, Entity::Player(P1))], &[(giant, dusk)]);
    assert_eq!(triggers_on_stack(&t, "regenerate"), 1);
    t.resolve_all();
    assert!(is_attacking(&t, dusk));
    destroy(&mut t, dusk);
    assert!(t.on_battlefield(dusk));
    assert!(!is_attacking(&t, dusk));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn ulvenwald_primordials_shield_carries_over_to_the_front_face() {
    cr!("701.19a", "712.18");
    ruling!("Ulvenwald Mystics // Ulvenwald Primordials", "You can regenerate Ulvenwald Primordials in response to the triggered ability that would transform it. If you do, the regeneration shield will apply to Ulvenwald Mystics that turn.");
    supported("Ulvenwald Mystics // Ulvenwald Primordials");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let wolf = enter_transformed(&mut t, P0, "Ulvenwald Mystics // Ulvenwald Primordials");
    assert_eq!(name_of(&t, wolf), "Ulvenwald Primordials");
    // P1 casts two spells this turn.
    for _ in 0..2 {
        cast(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        t.resolve_all();
    }
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "transform"), 1);
    t.lands(P0, "Forest", 1);
    activate_containing(&mut t, P0, wolf, "Regenerate").unwrap();
    t.resolve();
    assert_eq!(name_of(&t, wolf), "Ulvenwald Primordials");
    t.resolve_all();
    assert_eq!(name_of(&t, wolf), "Ulvenwald Mystics");
    destroy(&mut t, wolf);
    assert!(regenerated(&t, wolf));
}
