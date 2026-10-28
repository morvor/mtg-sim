//! Rulings batch P185 — Human typal cards: sacrificing the source as a cost after
//! targeting it (CR 601.2c, 602.2b), effects that affect only what's there as they resolve
//! (CR 611.2c, 608.2h), simultaneous deaths (CR 603.10a), intervening "if" clauses (CR
//! 603.4), replacement effects that look at what a creature will be as it enters (CR
//! 614.12), prevention by the source's type (CR 609.7, 615), and flashback without a
//! mana cost (CR 702.34, 118.6).

use crate::r_p023_common::enter_copying;
use crate::r_p185_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s24_common::{choose_creature_type, enter_together};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn deranged_outcast_and_skirsdag_flayer_can_target_and_sacrifice_themselves() {
    cr!("601.2c", "601.2h", "602.2b", "608.2b");
    ruling!(
        "Deranged Outcast",
        "Because targets are chosen before costs are paid, you can activate Deranged Outcast’s ability targeting itself and then sacrifice Deranged Outcast to pay the cost."
    );
    ruling!(
        "Skirsdag Flayer",
        "Because targets are chosen before costs are paid, you can activate Skirsdag Flayer’s ability targeting itself and then sacrifice Skirsdag Flayer to pay the cost."
    );
    ruling!(
        "Deranged Outcast",
        "You can sacrifice any Human you control, including Deranged Outcast itself."
    );
    ruling!(
        "Skirsdag Flayer",
        "You can sacrifice any Human you control, including Skirsdag Flayer itself."
    );
    for (name, land, n) in [("Deranged Outcast", "Forest", 2), ("Skirsdag Flayer", "Swamp", 4)] {
        supported(name);
        // Targeting itself and sacrificing itself: the ability is activated, and then does
        // nothing as its only target is gone.
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        let other = t.battlefield(P0, "Vernadi Shieldmate");
        t.lands(P0, land, n);
        t.answer_choose(P0, &[Entity::Object(me)]);
        let r = t.activate(P0, me, 0, &[Entity::Object(me)]);
        assert!(r.is_ok(), "{name}: {r:?}");
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
        t.resolve_all();
        assert!(t.on_battlefield(other), "{name}");
        assert_eq!(t.counters(other, counters::PLUS1), 0, "{name}");
        // Sacrificing another Human instead.
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        let other = t.battlefield(P0, "Vernadi Shieldmate");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, land, n);
        t.answer_choose(P0, &[Entity::Object(other)]);
        t.activate(P0, me, 0, &[Entity::Object(bears)]).unwrap();
        assert!(t.in_graveyard(P0, "Vernadi Shieldmate"), "{name}");
        assert!(t.on_battlefield(me), "{name}");
        t.resolve_all();
        if name == "Deranged Outcast" {
            assert_eq!(t.counters(bears, counters::PLUS1), 2);
        } else {
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        }
    }
}

#[test]
fn erkenbrand_pumps_only_creatures_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Erkenbrand, Lord of Westfold",
        "Erkenbrand, Lord of Westfold's triggered ability affects only creatures you control at the time it resolves. Any creatures that come under your control later in the turn won't be affected."
    );
    supported("Erkenbrand, Lord of Westfold");
    let mut t = TestGame::new(2);
    let erk = t.battlefield(P0, "Erkenbrand, Lord of Westfold");
    enter_with(&mut t, P0, "Vernadi Shieldmate", &[]);
    assert_eq!(t.stack_len(), 1);
    // Arrives before the trigger resolves: affected.
    let early = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(erk), (4, 3));
    assert_eq!(t.pt(early), (3, 2));
    // Comes later: not affected.
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn xathrid_necromancer_dying_with_another_human_triggers_twice() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Xathrid Necromancer",
        "If Xathrid Necromancer dies at the same time as another Human creature you control, its ability will trigger twice."
    );
    supported("Xathrid Necromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xathrid Necromancer");
    t.battlefield(P0, "Vernadi Shieldmate");
    // A non-Human doesn't count.
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    let zombies = with_subtype(&t, P0, "Zombie");
    assert_eq!(zombies.len(), 2);
    assert!(zombies.iter().all(|z| t.obj_now(*z).tapped));
}

#[test]
fn katilda_gives_no_flashback_cost_to_a_card_without_a_mana_cost() {
    cr!("702.34a", "118.6");
    ruling!(
        "Katilda and Lier",
        "If a card with no mana cost gains flashback, it has no flashback cost. It can’t be cast this way."
    );
    supported("Katilda and Lier");
    for (card, castable) in [("Ancestral Vision", false), ("Shock", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Katilda and Lier");
        let target = t.graveyard(P0, card);
        cast_new(&mut t, P0, "Vernadi Shieldmate", &[Entity::Object(target)]);
        t.resolve_all();
        assert!(t.on_battlefield(t.named_on_battlefield("Vernadi Shieldmate")[0]));
        t.lands(P0, "Mountain", 3);
        t.lands(P0, "Island", 3);
        let methods = legal_cast_methods(&mut t, P0, target);
        assert_eq!(!methods.is_empty(), castable, "{card}: {methods:?}");
    }
}

#[test]
fn repel_the_abominable_prevents_by_the_source_dealing_the_damage() {
    cr!("609.7a", "615.1a", "701.14a");
    ruling!(
        "Repel the Abominable",
        "If a spell or ability causes damage to be dealt, it will indicate what object deals the damage. The object dealing damage might not be the spell itself."
    );
    ruling!(
        "Repel the Abominable",
        "Sources that have no creature type at all are non-Human sources, as are sources with creature types that don’t include Human. A creature that is Human and has other types, such as a Human Werewolf, is still a Human source."
    );
    supported("Repel the Abominable");
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Repel the Abominable", &[]);
    t.resolve_all();
    // Prey Upon (a spell with no creature types) has a Human Werewolf fight a Bear: the
    // Werewolf, a Human source, deals its damage; the Bear's is prevented.
    let wolf = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(
        &mut t,
        P0,
        "Prey Upon",
        &[Entity::Object(wolf), Entity::Object(bears)],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(marked(&t, wolf), 0);
    // Lightning Bolt has no creature type: its damage is prevented.
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn eowyn_needs_another_human_to_have_entered_this_turn() {
    cr!("603.4");
    ruling!(
        "Éowyn, Shieldmaiden",
        "If another Human didn't enter the battlefield before your beginning of combat step, Éowyn, Shieldmaiden's ability won't trigger. You won't draw a card even if you control six or more Humans."
    );
    supported("Éowyn, Shieldmaiden");
    for entered in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Éowyn, Shieldmaiden");
        for _ in 0..5 {
            t.battlefield(P0, "Vernadi Shieldmate");
        }
        if entered {
            enter_with(&mut t, P0, "Vernadi Shieldmate", &[]);
        }
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let knights = tokens(&t, P0).len();
        if entered {
            assert_eq!(knights, 2);
            assert_eq!(t.hand_size(P0), hand + 1);
        } else {
            assert_eq!(knights, 0);
            assert_eq!(t.hand_size(P0), hand);
        }
    }
}

#[test]
fn dearly_departed_looks_at_the_creature_as_it_will_enter_and_is_cumulative() {
    cr!("614.12", "614.1c");
    ruling!(
        "Dearly Departed",
        "In most cases, when determining whether a creature entering under your control should get a +1/+1 counter, you'll simply look at what the creature will looklike on the battlefield. You'll consider any effects affecting a creature entering under your control."
    );
    ruling!(
        "Dearly Departed",
        "The effect is cumulative. Human creatures you control will enter with a +1/+1 counter for each Dearly Departed in your graveyard."
    );
    supported("Dearly Departed");
    supported("Xenograft");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Dearly Departed");
    // Xenograft makes each creature P0 controls a Human: Grizzly Bears enters as one.
    choose_creature_type(&mut t, P0, "Human");
    t.enter(P0, "Xenograft");
    t.settle();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // A second Dearly Departed: two counters.
    t.graveyard(P0, "Dearly Departed");
    let human = t.enter(P0, "Vernadi Shieldmate");
    assert_eq!(t.counters(human, counters::PLUS1), 2);
    // An opponent's Human isn't affected.
    let theirs = t.enter(P1, "Vernadi Shieldmate");
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
}

#[test]
fn rukarumels_type_applies_before_typal_enters_replacements() {
    cr!("614.12", "613.1d");
    ruling!(
        "Rukarumel, Biologist",
        "Replacement effects that modify creatures of a certain type as they enter will apply after you apply this effect. For example, if Warrior is the chosen creature type and you control Bramblewood Paragon, a Runeclaw Bear would enter the battlefield with an additional +1/+1 counter."
    );
    supported("Rukarumel, Biologist");
    supported("Bramblewood Paragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramblewood Paragon");
    choose_creature_type(&mut t, P0, "Warrior");
    t.enter(P0, "Rukarumel, Biologist");
    t.settle();
    let bear = t.enter(P0, "Runeclaw Bear");
    t.settle();
    assert!(t.obj_now(bear).chars.has_subtype("Warrior"));
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
}

#[test]
fn angel_of_glorys_rise_exiles_only_zombie_permanents() {
    cr!("611.2c", "608.2c");
    ruling!(
        "Angel of Glory's Rise",
        "Only Zombie permanents on the battlefield are exiled. Zombie cards in other zones are not."
    );
    supported("Angel of Glory's Rise");
    let mut t = TestGame::new(2);
    let zombie = t.battlefield(P1, "Those Who Serve");
    t.graveyard(P0, "Those Who Serve");
    t.graveyard(P1, "Those Who Serve");
    t.graveyard(P0, "Vernadi Shieldmate");
    enter_with(&mut t, P0, "Angel of Glory's Rise", &[]);
    t.resolve_all();
    assert_eq!(t.zone(zombie), Zone::Exile);
    assert!(t.in_graveyard(P0, "Those Who Serve"));
    assert!(t.in_graveyard(P1, "Those Who Serve"));
    assert_eq!(t.named_on_battlefield("Vernadi Shieldmate").len(), 1);
}

#[test]
fn mass_appeal_counts_humans_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Mass Appeal",
        "The number of Humans you control is counted only when Mass Appeal resolves."
    );
    supported("Mass Appeal");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Vernadi Shieldmate");
    t.battlefield(P0, "Vernadi Shieldmate");
    t.battlefield(P1, "Vernadi Shieldmate");
    cast_new(&mut t, P0, "Mass Appeal", &[]);
    // One leaves and two arrive before it resolves: three Humans.
    destroy(&mut t, h);
    t.battlefield(P0, "Vernadi Shieldmate");
    t.battlefield(P0, "Valeron Outlander");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn moonmist_checks_wolf_or_werewolf_as_combat_damage_is_dealt() {
    cr!("615.1a", "510.2", "611.2c");
    ruling!(
        "Moonmist",
        "Whether or not a creature is a Werewolf or a Wolf is checked only as combat damage is dealt. If the creature isn’t a Werewolf or a Wolf at that time, its combat damage will be prevented."
    );
    supported("Moonmist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let wolf = t.battlefield(P0, "Watchwolf");
    cast_new(&mut t, P0, "Moonmist", &[]);
    t.resolve_all();
    // After Moonmist resolves, the Bears becomes a Wolf (its damage is dealt) and the
    // Watchwolf stops being a Wolf (its damage is prevented).
    add_types(&mut t, bears, &["Wolf"]);
    modify_until_eot(
        &mut t,
        wolf,
        vec![Modification::RemoveSubtypes(vec!["Wolf".into()])],
    );
    t.attack(
        &[
            (bears, Entity::Player(P1)),
            (lion, Entity::Player(P1)),
            (wolf, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(t.life(P1), 18);
}

#[test]
fn resistance_squad_checks_for_another_human_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Resistance Squad",
        "You must control another Human both at the time Resistance Squad enters the battlefield and at the time the ability resolves to draw a card."
    );
    supported("Resistance Squad");
    let mut t = TestGame::new(2);
    enter_with(&mut t, P0, "Resistance Squad", &[]);
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Vernadi Shieldmate");
    enter_with(&mut t, P0, "Resistance Squad", &[]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, h);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vernadi Shieldmate");
    enter_with(&mut t, P0, "Resistance Squad", &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn grumgully_counters_non_humans_as_they_will_enter() {
    cr!("614.12", "614.1c", "707.5");
    ruling!(
        "Grumgully, the Generous",
        "A non-Human creature you control that would enter the battlefield with no +1/+1 counters on it enters with one +1/+1 counter on it."
    );
    ruling!(
        "Grumgully, the Generous",
        "Any other non-Human creatures that enter the battlefield at the same time as Grumgully won't get a +1/+1 counter."
    );
    ruling!(
        "Grumgully, the Generous",
        "If a non-Human creature enters the battlefield as a copy of a Human creature, it won't get a +1/+1 counter. Similarly, if a Human enters as a non-Human creature, it will get a +1/+1 counter."
    );
    supported("Grumgully, the Generous");
    supported("Clone");
    supported("Sakashima of a Thousand Faces");
    // Entering together with Grumgully: no counter.
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Grumgully, the Generous"), (P0, "Grizzly Bears")],
    );
    assert_eq!(t.counters(ids[1], counters::PLUS1), 0);
    assert_eq!(t.counters(ids[0], counters::PLUS1), 0);
    // Entering later: a non-Human gets one, a Human doesn't.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    let human = t.enter(P0, "Vernadi Shieldmate");
    assert_eq!(t.counters(human, counters::PLUS1), 0);
    // Clone (a Shapeshifter) entering as a copy of a Human: no counter.
    let clone = enter_copying(&mut t, P0, "Clone", human);
    assert!(t.obj_now(clone).chars.has_subtype("Human"));
    assert_eq!(t.counters(clone, counters::PLUS1), 0);
    // Sakashima (a Human) entering as a copy of the Bears: a counter.
    t.answer_yes(P0, true);
    let sak = enter_copying(&mut t, P0, "Sakashima of a Thousand Faces", bears);
    assert!(!t.obj_now(sak).chars.has_subtype("Human"));
    assert_eq!(t.counters(sak, counters::PLUS1), 1);
}
