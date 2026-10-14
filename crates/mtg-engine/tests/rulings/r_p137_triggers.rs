//! Rulings batch P137 — triggered abilities around sacrifices and deaths: when they
//! trigger and when they are put on the stack (CR 603.2, 603.3), leaves-the-battlefield
//! abilities looking back in time (CR 603.10a), intervening "if" clauses (CR 603.4), cast
//! triggers resolving first (CR 603.2), lethal damage and state-based actions before a
//! combat damage trigger resolves (CR 704.3), and layers (CR 613.6).

use crate::r_p137_common::*;
use crate::r_s01_common::triggers_on_stack;
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

#[test]
fn archfiend_of_spite_triggers_even_when_dealt_lethal_damage() {
    cr!("603.10a", "120.3");
    ruling!(
        "Archfiend of Spite",
        "Archfiend of Spite's triggered ability will trigger even if it is dealt lethal damage."
    );
    supported("Archfiend of Spite");
    let mut t = TestGame::new(2);
    let fiend = t.battlefield(P0, "Archfiend of Spite");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.deal_damage(wurm, obj(fiend), 6, false);
    t.settle();
    assert!(t.in_graveyard(P0, "Archfiend of Spite"));
    assert_eq!(t.stack_len(), 1);
    // P1 can't sacrifice six permanents: they lose 6 life.
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(wurm));
}

/// P0's Ashen Reaper (Invasion of Azgol's back face) on the battlefield.
fn ashen_reaper(t: &mut TestGame) -> ObjectId {
    supported("Invasion of Azgol // Ashen Reaper");
    let card = t.graveyard(P0, "Invasion of Azgol // Ashen Reaper");
    let reaper = put_onto_battlefield(t, P0, card, true).expect("Ashen Reaper entered");
    assert_eq!(t.obj(reaper).chars.name, "Ashen Reaper");
    reaper
}

#[test]
fn ashen_reaper_triggers_every_end_step_and_checks_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Invasion of Azgol // Ashen Reaper",
        "Ashen Reaper’s ability triggers at the beginning of each of your end steps, even if nothing has entered any graveyard that turn. The ability checks as it resolves whether to put a +1/+1 counter on Ashen Reaper or not. This means the ability will count any permanent that’s put into a graveyard in response to the ability."
    );
    let mut t = TestGame::new(2);
    let reaper = ashen_reaper(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "it triggers although nothing died");
    // In response, a permanent is put into a graveyard.
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(reaper, counters::PLUS1), 1);
}

#[test]
fn ashen_reaper_counts_a_card_that_already_left_the_graveyard() {
    cr!("608.2c");
    ruling!(
        "Invasion of Azgol // Ashen Reaper",
        "If a nontoken permanent is put into a graveyard, it doesn’t matter what happens to the permanent card afterward. Ashen Reaper’s ability will count it even if the card has left the graveyard by the time the ability resolves."
    );
    let mut t = TestGame::new(2);
    let reaper = ashen_reaper(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    t.g.move_object(
        card,
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(reaper, counters::PLUS1), 1);
}

#[test]
fn barrensteppe_siege_mardu_doesnt_trigger_without_a_death() {
    cr!("603.4");
    ruling!(
        "Barrensteppe Siege",
        "Barrensteppe Siege’s Mardu ability will check as the end step starts to see if a creature died under your control this turn. If none did, the ability won’t trigger at all."
    );
    supported("Barrensteppe Siege");
    for died in [false, true] {
        let mut t = TestGame::new(2);
        // As it enters, P0 chooses Mardu (the second option).
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.enter(P0, "Barrensteppe Siege");
        t.settle();
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let mine = t.battlefield(P0, "Hill Giant");
        if died {
            destroy(&mut t, mine);
        }
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(died), "died: {died}");
        t.resolve_all();
        assert_eq!(t.on_battlefield(theirs), !died);
    }
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered_or_removed() {
    cr!("603.2", "603.3", "608.2");
    ruling!(
        "Breaker of Creation",
        "Breaker of Creation's triggered ability will resolve before Breaker of Creation does. If Breaker of Creation is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Emrakul, the Aeons Torn",
        "Emrakul's second ability triggers as you cast it, and that ability resolves before the spell itself. It resolves even if that spell is somehow removed from the stack."
    );
    supported("Breaker of Creation");
    supported("Emrakul, the Aeons Torn");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 8);
    let breaker = t.hand(P0, "Breaker of Creation");
    let spell = t.cast(P0, breaker).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(t.g.counter(spell, None));
    t.resolve_all();
    // Eight colorless permanents (the Wastes).
    assert_eq!(t.life(P0), 28);
    assert!(t.in_graveyard(P0, "Breaker of Creation"));

    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 15);
    let emrakul = t.hand(P0, "Emrakul, the Aeons Torn");
    let spell = t.cast(P0, emrakul).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The spell leaves the stack (Emrakul can't be countered).
    t.g.move_object(
        spell,
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0]);
}

#[test]
fn martyrs_bond_triggers_for_itself_and_each_other_permanent() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Martyr's Bond",
        "If Martyr’s Bond and other nonland permanents are put into the graveyard from the battlefield at the same time, Martyr’s Bond will trigger for itself and each of those other nonland permanents."
    );
    supported("Martyr's Bond");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Martyr's Bond");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Sol Ring");
    // P1 keeps a land so nothing of theirs is destroyed.
    t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 6);
    let wipe = t.hand(P0, "Planar Cleansing");
    t.cast(P0, wipe).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "shares a card type"), 3);
}

#[test]
fn martyrs_bond_one_sacrifice_per_trigger_even_for_multiple_types() {
    cr!("701.21a");
    ruling!(
        "Martyr's Bond",
        "Each opponent must sacrifice only one permanent for each trigger, even if the nonland permanent you controlled that went to the graveyard had multiple types. For example, if an artifact creature you control is put into a graveyard from the battlefield, each opponent sacrifices exactly one artifact or creature, and that permanent doesn’t need to be an artifact creature."
    );
    supported("Martyr's Bond");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Martyr's Bond");
    let thopter = t.battlefield(P0, "Ornithopter");
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(ring)]);
    destroy(&mut t, thopter);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(t.on_battlefield(bears) && t.on_battlefield(anthem));
    // P1 chose among its artifacts and creatures (not the enchantment).
    let offered = offered_to(&t, P1, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 2);
}

#[test]
fn vein_ripper_triggers_for_each_creature_dying_with_it() {
    cr!("603.10a");
    ruling!(
        "Vein Ripper",
        "If Vein Ripper dies at the same time as one or more other creatures, its last ability triggers for each of those creatures."
    );
    supported("Vein Ripper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vein Ripper");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // Vein Ripper itself and the two other creatures.
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn eldrazi_conscription_each_annihilator_instance_triggers() {
    cr!("702.86a", "702.86b");
    ruling!(
        "Eldrazi Conscription",
        "If a creature has multiple instances of annihilator, each triggers separately when that creature attacks."
    );
    supported("Eldrazi Conscription");
    supported("Ulamog's Crusher");
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Ulamog's Crusher");
    let aura = t.battlefield(P0, "Eldrazi Conscription");
    t.g.attach(aura, obj(crusher));
    t.g.recompute();
    for _ in 0..5 {
        t.battlefield(P1, "Forest");
    }
    attack_with(&mut t, &[(crusher, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 1);
}

#[test]
fn combat_damage_edicts_after_lethal_damage_is_dealt() {
    cr!("510.2", "704.3", "704.5g", "603.3");
    ruling!(
        "Wasitora, Nekoru Queen",
        "If a creature is dealt lethal combat damage at the same time that Wasitora deals combat damage to that creature's controller, that creature will be destroyed before that player chooses a creature to sacrifice."
    );
    ruling!(
        "Demon of Loathing",
        "If a creature is dealt lethal damage at the same time that Demon of Loathing deals combat damage to its controller, that player can’t sacrifice that creature."
    );
    supported("Wasitora, Nekoru Queen");
    supported("Demon of Loathing");
    // Wasitora: the blocking Giant Spider dies; P1 has nothing left to sacrifice, so P0
    // gets a Cat Dragon.
    let mut t = TestGame::new(2);
    let wasitora = t.battlefield(P0, "Wasitora, Nekoru Queen");
    let spider = t.battlefield(P1, "Giant Spider");
    attack_with(&mut t, &[(wasitora, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(spider, wasitora)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Giant Spider"));
    assert!(t.life(P1) < 20);
    assert_eq!(t.named_on_battlefield("Cat Dragon Token").len(), 1);
    // Demon of Loathing: P1 must sacrifice its other creature.
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Demon of Loathing");
    let spider = t.battlefield(P1, "Giant Spider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(demon, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(spider, demon)]);
    t.resolve_all();
    assert!(t.life(P1) < 20);
    assert!(!t.on_battlefield(bears));
    for offered in offered_to(&t, P1, from) {
        assert!(!offered.contains(&spider));
    }
}

#[test]
fn demon_of_wailing_agonies_triggers_as_your_commander_dies() {
    cr!("510.2", "603.2", "603.3");
    ruling!(
        "Demon of Wailing Agonies",
        "If Demon of Wailing Agonies deals combat damage to a player at the same time your commander is dealt lethal damage, the triggered ability will trigger and the player will sacrifice a creature."
    );
    supported("Demon of Wailing Agonies");
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Demon of Wailing Agonies");
    let cmdr = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.recompute();
    assert_eq!(t.pt(demon), (6, 6));
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(
        &mut t,
        &[(demon, Entity::Player(P1)), (cmdr, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(giant, cmdr)]);
    t.resolve_all();
    assert!(!t.on_battlefield(cmdr));
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(giant), "P1 sacrificed the Hill Giant");
}

#[test]
fn kuon_ignores_creatures_dying_after_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Kuon, Ogre Ascendant // Kuon's Essence",
        "Creatures put into graveyards after the end-of-turn step begins have no particular effect on Kuon."
    );
    supported("Kuon, Ogre Ascendant // Kuon's Essence");
    let mut t = TestGame::new(2);
    let kuon = t.battlefield(P0, "Kuon, Ogre Ascendant // Kuon's Essence");
    let victims: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    for v in victims {
        destroy(&mut t, v);
    }
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(kuon).chars.name, "Kuon, Ogre Ascendant");
}

#[test]
fn idol_of_false_gods_counts_noncreature_eldrazi() {
    cr!("700.4", "603.6c");
    ruling!(
        "Idol of False Gods",
        "Idol of False Gods's second ability will trigger whenever another Eldrazi you control is put into a graveyard from the battlefield, even if that Eldrazi isn't a creature."
    );
    supported("Idol of False Gods");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Idol of False Gods");
    let other = t.battlefield(P0, "Idol of False Gods");
    destroy(&mut t, other);
    t.resolve_all();
    assert_eq!(t.counters(idol, counters::PLUS1), 1);
}

#[test]
fn idol_of_false_gods_stays_a_creature_after_losing_its_abilities() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Idol of False Gods",
        "If Idol of False Gods loses its abilities, the type-changing effect of its last ability will continue to apply to it. This is because effects that change an object's types are always applied before effects that remove abilities. In that case, if it has eight or more +1/+1 counters on it, it will be an Eldrazi kindred artifact creature."
    );
    supported("Idol of False Gods");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Idol of False Gods");
    t.g.add_counters(obj(idol), counters::PLUS1, 8, None);
    t.g.recompute();
    assert!(t.obj(idol).is(CardType::Creature));
    // Humility: all creatures lose all abilities and are 1/1.
    t.battlefield(P1, "Humility");
    t.g.recompute();
    let o = t.obj_now(idol);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact) && o.is(CardType::Kindred));
    assert!(o.chars.has_subtype("Eldrazi"));
    assert!(o.chars.abilities.is_empty());
    assert_eq!(t.pt(idol), (9, 9));
}

#[test]
fn priest_of_forgotten_gods_uses_the_stack_without_targets() {
    cr!("605.1a", "602.2");
    ruling!(
        "Priest of Forgotten Gods",
        "Because it may have targets, the ability of Priest of Forgotten Gods isn't a mana ability. It uses the stack and can be responded to, even if no targets were chosen."
    );
    supported("Priest of Forgotten Gods");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Priest of Forgotten Gods");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[]);
    let hand = t.hand_size(P0);
    t.activate(P0, priest, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 0);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn the_grim_captain_puts_a_creature_onto_the_battlefield_attacking_without_attacking() {
    cr!("506.3d", "508.4");
    ruling!(
        "Throne of the Grim Captain // The Grim Captain",
        "Although the creature card you put onto the battlefield is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    const THRONE: &str = "Throne of the Grim Captain // The Grim Captain";
    supported(THRONE);
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, THRONE);
    let materials: Vec<ObjectId> = [
        "Frenzied Raptor",
        "Coral Merfolk",
        "Swab Goblin",
        "Falkenrath Perforator",
    ]
    .iter()
    .map(|n| t.graveyard(P0, n))
    .collect();
    t.lands(P0, "Wastes", 4);
    t.answer_choose(P0, &materials.iter().map(|o| obj(*o)).collect::<Vec<_>>());
    t.activate(P0, throne, 1, &[]).expect("craft");
    t.resolve_all();
    let captain = t.named_on_battlefield("The Grim Captain");
    assert_eq!(captain.len(), 1);
    let captain = captain[0];
    t.g.objects[captain.0 as usize].summoning_sick = false;
    t.battlefield(P1, "Grizzly Bears");
    // P0 puts the Falkenrath Perforator ("Whenever this creature attacks, it deals 1
    // damage to defending player") onto the battlefield attacking.
    let perforator = t.g.current(materials[3]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(perforator)]);
    attack_with(&mut t, &[(captain, Entity::Player(P1))]);
    t.resolve_all();
    let p = t.named_on_battlefield("Falkenrath Perforator");
    assert_eq!(p.len(), 1);
    assert!(t.g.is_attacking(p[0]));
    // Its attack trigger didn't trigger.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 0);
}
