//! Rulings batch P161 — assorted spells and permanents: who may activate an Aura's
//! "return this Aura" ability (CR 602.2), spells whose targets become illegal (CR 608.2b),
//! "can't cast" effects that don't touch spells already cast (CR 101.2), Combust's
//! unpreventable damage (CR 615.12), values counted on resolution (CR 608.2h), divided
//! damage (CR 601.2d), and combat-damage counters (CR 510.2).

use crate::r_p160_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn aura_on(t: &mut TestGame, caster: PlayerId, aura: &str, target: ObjectId) -> ObjectId {
    cast_resolve(t, caster, aura, &[Entity::Object(target)]);
    t.named_on_battlefield(aura)[0]
}

#[test]
fn only_the_auras_controller_can_return_it() {
    cr!("602.2", "301.5d");
    ruling!(
        "Agoraphobia",
        "Only Agoraphobia's controller can activate its last ability, no matter who controls the creature Agoraphobia's attached to."
    );
    ruling!(
        "Mourning",
        "If attached to an opponent's creature, only you can activate the ability to return it."
    );
    for (aura, mana) in [("Agoraphobia", ["Island", "Island", "Island"]), ("Mourning", ["Swamp", "Swamp", "Swamp"])] {
        supported(aura);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let a = aura_on(&mut t, P0, aura, giant);
        for land in mana {
            t.battlefield(P1, land);
        }
        assert!(t.activate(P1, a, 0, &[]).is_err(), "{aura}");
        for land in mana {
            t.battlefield(P0, land);
        }
        activate_resolve(&mut t, P0, a, 0, &[]);
        assert!(t.in_hand(P0, aura), "{aura}");
    }
}

#[test]
fn agoraphobias_ability_works_only_on_the_battlefield() {
    cr!("113.6", "602.2");
    ruling!(
        "Agoraphobia",
        "Agoraphobia's last ability can be activated only while it's on the battlefield."
    );
    supported("Agoraphobia");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Agoraphobia");
    t.lands(P0, "Island", 3);
    assert!(t.activate(P0, a, 0, &[]).is_err());
    assert!(t.in_hand(P0, "Agoraphobia"));
}

#[test]
fn agoraphobia_must_be_returned_before_combat_damage_is_assigned() {
    cr!("510.1", "510.2", "510.3");
    ruling!(
        "Agoraphobia",
        "Players don't have priority to cast spells and activate abilities between combat damage being assigned and being dealt. This means that if you want to return Agoraphobia to its owner's hand before combat damage is dealt, you must do so before combat damage is assigned (and the creature will no longer get -5/-0)."
    );
    supported("Agoraphobia");
    // P0's Agoraphobia is on P0's Hill Giant (a -2/3). It attacks; P0 returns the Aura in
    // the declare blockers step, so the Giant assigns and deals 3 damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let a = aura_on(&mut t, P0, "Agoraphobia", giant);
    assert_eq!(t.pt(giant), (-2, 3));
    t.lands(P0, "Island", 3);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // No priority is given during the combat damage step until damage has been dealt.
    activate_resolve(&mut t, P0, a, 0, &[]);
    assert_eq!(t.pt(giant), (3, 3));
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 17);
    // Without returning it, the Giant deals no damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    aura_on(&mut t, P0, "Agoraphobia", giant);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rookie_mistake_affects_the_remaining_legal_target() {
    cr!("608.2b");
    ruling!(
        "Rookie Mistake",
        "If one target creature becomes an illegal target before Rookie Mistake resolves, the other is affected as appropriate."
    );
    supported("Rookie Mistake");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Rookie Mistake", &[Entity::Object(a), Entity::Object(b)]);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.pt(b), (1, 3));
    // And the other way around.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Rookie Mistake", &[Entity::Object(a), Entity::Object(b)]);
    cast_new(&mut t, P1, "Unsummon", &[Entity::Object(b)]);
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 4));
}

#[test]
fn cantrips_with_an_illegal_target_dont_draw() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Befuddle",
        "If the target creature is an illegal target by the time Befuddle tries to resolve, the spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Bewilder",
        "If the target creature is an illegal target by the time Bewilder tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    ruling!(
        "Chilling Trap",
        "If the target creature is an illegal target by the time Chilling Trap tries to resolve, the spell doesn't resolve. You don't draw a card if you control a Wizard."
    );
    ruling!(
        "Fleeting Distraction",
        "If the targeted creature is an illegal target by the time Fleeting Distraction resolves, the spell doesn't resolve. You won't draw a card."
    );
    ruling!(
        "Fleeting Distraction",
        "If the targeted creature is an illegal target when Fleeting Distraction tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    supported("Prodigal Sorcerer");
    for name in ["Befuddle", "Bewilder", "Chilling Trap", "Fleeting Distraction"] {
        supported(name);
        // With a legal target: it draws.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Prodigal Sorcerer");
        let giant = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, name, &[Entity::Object(giant)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
        // The target is gone: no draw, and the card goes to the graveyard.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Prodigal Sorcerer");
        let giant = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, name, &[Entity::Object(giant)]);
        cast_new(&mut t, P1, "Unsummon", &[Entity::Object(giant)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert!(t.in_graveyard(P0, name), "{name}");
    }
}

#[test]
fn morgul_knife_wound_creatures_controller_chooses_on_resolution() {
    cr!("118.12", "608.2d");
    ruling!(
        "Morgul-Knife Wound",
        "The creature's controller chooses whether or not to pay 2 life as the triggered ability resolves."
    );
    supported("Morgul-Knife Wound");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        aura_on(&mut t, P0, "Morgul-Knife Wound", giant);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let asked_before = t.asked().len();
        t.answer(P1, DecisionKind::Any, Answer::Bool(pay));
        t.resolve_all();
        let asked: Vec<_> = t.asked()[asked_before..]
            .iter()
            .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
            .cloned()
            .collect();
        assert!(!asked.is_empty());
        assert!(asked.iter().all(|(p, _)| *p == P1), "{asked:?}");
        if pay {
            assert_eq!(t.life(P1), 18);
            assert!(t.on_battlefield(giant));
        } else {
            assert_eq!(t.life(P1), 20);
            assert!(t.in_exile("Hill Giant"));
        }
    }
}

#[test]
fn disturbing_conversion_x_changes_with_the_graveyard() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Disturbing Conversion",
        "The value of X will change as the number of cards in the appropriate graveyard changes."
    );
    supported("Disturbing Conversion");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    aura_on(&mut t, P0, "Disturbing Conversion", giant);
    // Each player milled two: X = 2.
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.pt(giant), (1, 3));
    t.graveyard(P1, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(giant), (0, 3));
    // P0's graveyard doesn't matter.
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(giant), (0, 3));
}

#[test]
fn silence_doesnt_affect_spells_cast_before_it_resolves() {
    cr!("101.2", "608.2");
    ruling!(
        "Silence",
        "Silence won't affect spells that your opponents cast before you cast Silence, including any spells that are still on the stack. Silence also won't stop your opponents from casting spells after you cast Silence but before Silence resolves."
    );
    supported("Silence");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Silence", &[]);
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    // After it resolved: P1 can't cast spells.
    lands_for_cost(&mut t, P1, "Shock");
    let shock = t.hand(P1, "Shock");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    assert!(t.cast(P1, shock).try_go().is_err());
}

#[test]
fn cease_fire_doesnt_counter_creature_spells_already_cast() {
    cr!("101.2", "608.2");
    ruling!(
        "Cease-Fire",
        "This spell will not counter any creature spells on the stack. It only prevents new creature spells from being cast after it resolves."
    );
    supported("Cease-Fire");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Grizzly Bears", &[]);
    cast_new(&mut t, P0, "Cease-Fire", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    lands_for_cost(&mut t, P1, "Grizzly Bears");
    let bears = t.hand(P1, "Grizzly Bears");
    assert!(t.cast(P1, bears).try_go().is_err());
}

#[test]
fn evaporate_deals_one_damage_to_a_white_and_blue_creature() {
    cr!("105.4", "120.3");
    ruling!(
        "Evaporate",
        "A creature which is both blue and white only takes one damage."
    );
    supported("Evaporate");
    supported("Azorius Guildmage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P1, "Azorius Guildmage");
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Evaporate", &[]);
    assert!(t.on_battlefield(mage));
    assert_eq!(t.obj(mage).damage, 1);
    assert_eq!(t.obj(angel).damage, 1);
    assert_eq!(t.obj(bears).damage, 0);
}

#[test]
fn combust_can_be_targeted_by_counterspells_whose_other_effects_work() {
    cr!("113.6g", "101.2", "701.6a");
    ruling!(
        "Combust",
        "Combust can be targeted by spells and abilities that try to counter it (such as Cancel). Those spells and abilities will resolve, but the part of their effect that would counter Combust won’t do anything. Any other effects those spells and abilities have will work as normal."
    );
    supported("Combust");
    supported("Contradict");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let combust = cast_new(&mut t, P0, "Combust", &[Entity::Object(angel)]);
    cast_new(&mut t, P1, "Contradict", &[Entity::Object(combust)]);
    let hand = t.hand_size(P1);
    t.resolve();
    // Contradict resolved: P1 drew; Combust is still on the stack.
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.in_graveyard(P1, "Contradict"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
}

#[test]
fn combust_damage_can_be_increased_by_non_prevention_replacements() {
    cr!("615.12", "614.1a");
    ruling!(
        "Combust",
        "Effects that replace or redirect damage without using the word “prevent” aren’t affected by Combust; they’ll work as normal."
    );
    supported("Combust");
    supported("Furnace of Rath");
    supported("Indomitable Ancients");
    // Without the Furnace, the 2/10 survives.
    let mut t = TestGame::new(2);
    let ancients = t.battlefield(P1, "Indomitable Ancients");
    cast_resolve(&mut t, P0, "Combust", &[Entity::Object(ancients)]);
    assert_eq!(t.obj(ancients).damage, 5);
    // Furnace of Rath doubles it to 10.
    let mut t = TestGame::new(2);
    let ancients = t.battlefield(P1, "Indomitable Ancients");
    t.battlefield(P0, "Furnace of Rath");
    cast_resolve(&mut t, P0, "Combust", &[Entity::Object(ancients)]);
    assert!(!t.on_battlefield(ancients));
}

#[test]
fn a_creature_dealt_lethal_damage_by_combust_can_regenerate() {
    cr!("701.19a", "701.19b", "615.12");
    ruling!(
        "Combust",
        "If a creature is dealt lethal damage by Combust, it can still regenerate. If it does, the damage marked on it will be removed from it."
    );
    supported("Combust");
    supported("Regeneration");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P1, Step::PrecombatMain);
    aura_on(&mut t, P1, "Regeneration", angel);
    t.set_step(P0, Step::PrecombatMain);
    let regen = t.named_on_battlefield("Regeneration")[0];
    t.battlefield(P1, "Forest");
    t.activate(P1, regen, 0, &[]).unwrap();
    t.resolve_all();
    cast_resolve(&mut t, P0, "Combust", &[Entity::Object(angel)]);
    assert!(t.on_battlefield(angel));
    assert!(t.obj(angel).tapped);
    assert_eq!(t.obj(angel).damage, 0);
}

#[test]
fn combust_damage_isnt_prevented_by_static_abilities_or_shields() {
    cr!("615.12", "615.1a");
    ruling!(
        "Combust",
        "If a static ability would prevent damage from being dealt to the targeted creature, it fails to prevent the damage dealt by Combust. If that ability has an additional effect that doesn’t depend on the amount of damage prevented, that additional effect will still work. It’s applied just once as Combust resolves."
    );
    ruling!(
        "Combust",
        "Spells that create prevention effects affecting the targeted creature can still be cast, and abilities that create prevention effects affecting the targeted creature can still be activated. However, damage prevention shields (including those created before Combust was cast) don’t have any effect on the damage dealt by Combust. If such a prevention effect has an additional effect, the additional effect will still work (if possible)."
    );
    supported("Combust");
    supported("Inviolability");
    supported("Healing Salve");
    // A static "prevent all damage that would be dealt to enchanted creature".
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P1, Step::PrecombatMain);
    aura_on(&mut t, P1, "Inviolability", angel);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(angel)]);
    assert_eq!(t.obj(angel).damage, 0);
    cast_resolve(&mut t, P0, "Combust", &[Entity::Object(angel)]);
    assert!(!t.on_battlefield(angel));
    // Prevention shields, created before Combust is cast and while it's on the stack.
    let mut t = TestGame::new(2);
    let ancients = t.battlefield(P1, "Indomitable Ancients");
    cast_resolve(&mut t, P1, "Healing Salve", &[Entity::Object(ancients)]);
    cast_new(&mut t, P0, "Combust", &[Entity::Object(ancients)]);
    // Still castable in response.
    cast_new(&mut t, P1, "Healing Salve", &[Entity::Object(ancients)]);
    t.resolve_all();
    assert_eq!(t.obj(ancients).damage, 5);
}

#[test]
fn combust_with_an_illegal_target_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Combust",
        "If the targeted creature is an illegal target by the time Combust would resolve, Combust won’t resolve for having an illegal target."
    );
    supported("Combust");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    cast_new(&mut t, P0, "Combust", &[Entity::Object(angel)]);
    // The Angel stops being white or blue.
    cast_new(&mut t, P1, "Unsummon", &[Entity::Object(angel)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Combust"));
    assert!(t.in_hand(P1, "Serra Angel"));
}

#[test]
fn chaotic_backlash_counts_each_permanent_once() {
    cr!("105.2", "608.2h");
    ruling!(
        "Chaotic Backlash",
        "Each permanent is counted only once. For example, if the targeted player controls a white creature, a blue enchantment, and a white-blue creature, Chaotic Backlash will deal 6 damage to that player."
    );
    supported("Chaotic Backlash");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Serra Angel");
    t.battlefield(P1, "Crystalline Resonance");
    t.battlefield(P1, "Azorius Guildmage");
    t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Chaotic Backlash", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn bereavement_does_nothing_with_an_empty_hand() {
    cr!("701.9a");
    ruling!(
        "Bereavement",
        "If the player has no cards in hand, this has no effect."
    );
    supported("Bereavement");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bereavement");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.hand_size(P1), 0);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(bears)]);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
    // With a card in hand, it's discarded.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(bears)]);
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Forest"));
}

#[test]
fn havoc_affects_every_opponent() {
    cr!("102.2", "102.3");
    ruling!("Havoc", "In a multiplayer game, it affects all opponents.");
    supported("Havoc");
    supported("Raise the Alarm");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Havoc");
    cast_resolve(&mut t, P1, "Raise the Alarm", &[]);
    cast_resolve(&mut t, P2, "Raise the Alarm", &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    // Its controller's own white spells don't count.
    cast_resolve(&mut t, P0, "Raise the Alarm", &[]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ignite_disorder_one_to_three_targets_divided_as_cast() {
    cr!("601.2c", "601.2d", "115.1d");
    ruling!(
        "Ignite Disorder",
        "The number of targets chosen for Ignite Disorder must be at least 1 and at most 3. You divide the damage as you cast the spell, not as it resolves. Each target must be assigned at least 1 damage."
    );
    ruling!(
        "Ignite Disorder",
        "The number of targets must be at least 1 and at most 3."
    );
    ruling!(
        "Ignite Disorder",
        "You divide the damage as you cast Ignite Disorder, not as it resolves. Each target must be assigned at least 1 damage."
    );
    supported("Ignite Disorder");
    supported("Azorius Guildmage");
    // Three targets, 1 each; the division is asked during casting.
    let mut t = TestGame::new(2);
    let mages: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P1, "Azorius Guildmage"))
        .collect();
    lands_for_cost(&mut t, P0, "Ignite Disorder");
    let spell = t.hand(P0, "Ignite Disorder");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1]));
    t.cast(P0, spell)
        .targets(&mages.iter().map(|m| Entity::Object(*m)).collect::<Vec<_>>())
        .go();
    let asked = t.asked();
    let targets = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .unwrap();
    assert_eq!(targets, (1, 3));
    let min_each = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::Divide { min_each, total, .. } => Some((*min_each, *total)),
            _ => None,
        })
        .expect("divided as it's cast");
    assert_eq!(min_each, (1, 3));
    t.resolve_all();
    for m in &mages {
        assert_eq!(t.obj(*m).damage, 1);
    }
    // Two targets, 2 and 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Azorius Guildmage");
    let b = t.battlefield(P1, "Serra Angel");
    lands_for_cost(&mut t, P0, "Ignite Disorder");
    let spell = t.hand(P0, "Ignite Disorder");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.obj(b).damage, 1);
    // One target: all 3.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Serra Angel");
    cast_resolve(&mut t, P0, "Ignite Disorder", &[Entity::Object(b)]);
    assert_eq!(t.obj(b).damage, 3);
}

#[test]
fn sanctimony_life_gain_is_optional() {
    cr!("603.5");
    ruling!(
        "Sanctimony",
        "Gaining life is optional. If you forget, you can’t go back later even if it is something you usually do."
    );
    supported("Sanctimony");
    for gain in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sanctimony");
        t.answer_yes(P0, gain);
        cast_resolve(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        assert_eq!(t.life(P0), if gain { 19 } else { 18 });
    }
}

#[test]
fn staff_of_the_ages_landwalkers_keep_landwalk_but_can_be_blocked() {
    cr!("702.14a", "702.14c");
    ruling!(
        "Staff of the Ages",
        "It does not remove Landwalk from creatures. It just makes creatures with landwalk blockable as if they did not have the ability."
    );
    supported("Staff of the Ages");
    supported("Bog Wraith");
    for staff in [false, true] {
        let mut t = TestGame::new(2);
        let wraith = t.battlefield(P0, "Bog Wraith");
        t.battlefield(P1, "Swamp");
        let blocker = t.battlefield(P1, "Hill Giant");
        if staff {
            t.battlefield(P1, "Staff of the Ages");
        }
        t.attack(&[(wraith, Entity::Player(P1))], &[(blocker, wraith)]);
        assert_eq!(t.life(P1), if staff { 20 } else { 17 });
        assert!(t
            .obj_now(wraith)
            .has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
    }
}

#[test]
fn typhoon_counts_islands_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Typhoon",
        "Number of Islands is counted on resolution and not on announcement."
    );
    supported("Typhoon");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Island");
    cast_new(&mut t, P0, "Typhoon", &[]);
    t.battlefield(P1, "Island");
    t.battlefield(P1, "Island");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn pale_rider_with_an_empty_hand_discards_nothing() {
    cr!("701.9a", "603.3");
    ruling!(
        "Pale Rider of Trostad",
        "You can cast Pale Rider of Trostad even if you have no other cards in your hand. If you have no cards in hand as the last ability resolves, nothing happens."
    );
    supported("Pale Rider of Trostad");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Pale Rider of Trostad", &[]);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    let rider = t.named_on_battlefield("Pale Rider of Trostad");
    assert_eq!(rider.len(), 1);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn combat_damage_counters_arrive_after_the_damage() {
    cr!("510.2", "702.4b", "603.2");
    ruling!(
        "Markov Blademaster",
        "The +1/+1 counter isn't put on Markov Blademaster in time to increase the amount of combat damage dealt during that combat step. However, the +1/+1 counter put on Markov Blademaster when first-strike damage is dealt will increase the amount of damage dealt during the regular combat damage step."
    );
    ruling!(
        "Erdwal Ripper",
        "The +1/+1 counter isn’t put on the creature in time to increase the amount of combat damage dealt during that combat step."
    );
    supported("Markov Blademaster");
    supported("Erdwal Ripper");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Markov Blademaster");
    t.attack(&[(blade, Entity::Player(P1))], &[]);
    // 1 first-strike damage, then 2 regular damage.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(blade, counters::PLUS1), 2);
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Erdwal Ripper");
    t.attack(&[(ripper, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(ripper), (3, 2));
}
