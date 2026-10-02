//! Rulings batch P214 — ferocious ("if you control a creature with power 4 or greater"):
//! when the condition is checked, and what an effect that applied keeps doing afterward.
//! Also fabricate (CR 702.123), Fast Healing and fateful hour.

use crate::r_p214_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Ferocious
// ---------------------------------------------------------------------------------------

#[test]
fn barrage_of_boulders_ferocious_affects_all_creatures() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Barrage of Boulders",
        "Barrage of Boulders deals damage only to creatures you don’t control as it resolves. However, the ferocious ability affects all creatures, including ones you control and ones that weren’t on the battlefield as Barrage of Boulders resolved."
    );
    supported("Barrage of Boulders");
    // Barrage of Boulders: "deals 1 damage to each creature you don't control. Ferocious —
    // If you control a creature with power 4 or greater, creatures can't block this turn."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let bb = t.hand(P0, "Barrage of Boulders");
    t.cast(P0, bb).go();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1);
    assert_eq!(damage_on(&t, wurm), 0);
    // P1's creature that enters afterward can't block either, nor can P0's creatures.
    let bears = enter(&mut t, P1, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.g.can_block_at_all(giant));
    assert!(!t.g.can_block_at_all(bears));
    assert!(!t.g.can_block_at_all(wurm));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(bears, wurm)]),
    );
    t.attack(&[(wurm, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn frontier_mastodon_doesnt_count_itself() {
    cr!("614.1c", "614.12");
    ruling!(
        "Frontier Mastodon",
        "Frontier Mastodon's ferocious ability checks if you control a creature with power 4 or greater as Frontier Mastodon enters the battlefield. Because Frontier Mastodon isn't on the battlefield at this time, it won't count itself."
    );
    supported("Frontier Mastodon");
    supported("Glorious Anthem");
    // Frontier Mastodon (3/2): "Ferocious — This creature enters with a +1/+1 counter on it
    // if you control a creature with power 4 or greater." With Glorious Anthem ("Creatures
    // you control get +1/+1."), it's a 4/3 once on the battlefield, but that doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let m = enter(&mut t, P0, "Frontier Mastodon");
    assert_eq!(t.counters(m, counters::PLUS1), 0);
    assert_eq!(t.pt(m), (4, 3));
    // A second one sees the first (power 4): it gets the counter.
    let m2 = enter(&mut t, P0, "Frontier Mastodon");
    assert_eq!(t.counters(m2, counters::PLUS1), 1);
}

#[test]
fn heir_of_the_wilds_checks_on_triggering_and_resolving() {
    cr!("603.4");
    ruling!(
        "Heir of the Wilds",
        "Heir of the Wilds has a triggered ferocious ability with an intervening “if” clause. That ability will check if you control a creature with power 4 or greater whenever Heir of the Wilds attacks."
    );
    supported("Heir of the Wilds");
    // Heir of the Wilds (2/2): "Whenever this creature attacks, if you control a creature
    // with power 4 or greater, this creature gets +1/+1 until end of turn."
    // No such creature: no trigger.
    let mut t = TestGame::new(2);
    let heir = t.battlefield(P0, "Heir of the Wilds");
    attack_with(&mut t, &[(heir, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    // Craw Wurm (6/4) is there as it attacks, but gone as it resolves: nothing.
    let mut t = TestGame::new(2);
    let heir = t.battlefield(P0, "Heir of the Wilds");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(heir, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.pt(heir), (2, 2));
    // A different 4-power creature as it resolves is enough.
    let mut t = TestGame::new(2);
    let heir = t.battlefield(P0, "Heir of the Wilds");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(heir, Entity::Player(P1))]);
    destroy(&mut t, wurm);
    t.battlefield(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.pt(heir), (3, 3));
}

/// P1 puts a Healing Salve shield ("Prevent the next 3 damage that would be dealt to any
/// target this turn.") on its Hill Giant.
fn salve(t: &mut TestGame, target: ObjectId) {
    t.lands(P1, "Plains", 1);
    let hs = t.hand(P1, "Healing Salve");
    t.cast(P1, hs).modes(&[1]).target(target).go();
    t.resolve_all();
}

#[test]
fn wild_slash_damage_cant_be_prevented_including_its_own() {
    cr!("615.12", "609.3");
    ruling!(
        "Wild Slash",
        "Wild Slash's ferocious ability applies to all damage that would be dealt that turn, including the damage Wild Slash deals."
    );
    supported("Wild Slash");
    supported("Healing Salve");
    // Wild Slash: "Ferocious — If you control a creature with power 4 or greater, damage
    // can't be prevented this turn. Wild Slash deals 2 damage to any target."
    for ferocious in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if ferocious {
            t.battlefield(P0, "Craw Wurm");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        salve(&mut t, bears);
        t.lands(P0, "Mountain", 1);
        let ws = t.hand(P0, "Wild Slash");
        t.cast(P0, ws).target(bears).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), !ferocious);
    }
}

#[test]
fn wild_slash_keeps_applying_after_the_creature_leaves() {
    cr!("615.12", "611.2a");
    ruling!(
        "Wild Slash",
        "If you control a creature with power 4 or greater as Wild Slash resolves, the ferocious ability will apply. Damage dealt that turn can't be prevented, even if you no longer control a creature with power 4 or greater as that damage would be dealt."
    );
    supported("Shock");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let ws = t.hand(P0, "Wild Slash");
    t.cast(P0, ws).target(bears).go();
    t.resolve_all();
    destroy(&mut t, wurm);
    // Later that turn: a prevention shield on the Giant, then Shock: its 2 damage isn't
    // prevented.
    salve(&mut t, giant);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
}

#[test]
fn winds_of_qal_sisma_keeps_applying_after_the_creature_leaves() {
    cr!("615.1a", "611.2a", "510.2");
    ruling!(
        "Winds of Qal Sisma",
        "If you control a creature with power 4 or greater as Winds of Qal Sisma resolves, the ferocious ability will apply. Combat damage dealt by creatures your opponents control will be prevented, even if you no longer control a creature with power 4 or greater as that damage would be dealt."
    );
    supported("Winds of Qal Sisma");
    // Winds of Qal Sisma: "Prevent all combat damage that would be dealt this turn.
    // Ferocious — If you control a creature with power 4 or greater, instead prevent all
    // combat damage that would be dealt this turn by creatures your opponents control."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let blocker = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    t.lands(P0, "Forest", 2);
    let w = t.hand(P0, "Winds of Qal Sisma");
    t.cast(P0, w).go();
    t.resolve_all();
    destroy(&mut t, wurm);
    block_and_finish(&mut t, P0, &[(blocker, bears)]);
    // P1's creatures dealt no damage; P0's blocker's damage wasn't prevented.
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(blocker));
    assert_eq!(damage_on(&t, blocker), 0);
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Fabricate
// ---------------------------------------------------------------------------------------

#[test]
fn fabricate_doesnt_make_it_enter_with_counters() {
    cr!("702.123a", "603.3", "117.3b");
    ruling!(
        "Cayth, Famed Mechanist",
        "Fabricate doesn't cause the creature with the ability to enter the battlefield with +1/+1 counters already on it. For example, Cayth, Famed Machinist will enter the battlefield as a 3/3 creature"
    );
    ruling!(
        "Marionette Apprentice",
        "Fabricate doesn't cause the creature with the ability to enter the battlefield with +1/+1 counters already on it. For example, Marionette Apprentice will enter the battlefield as a 1/2 creature"
    );
    supported("Cayth, Famed Mechanist");
    supported("Marionette Apprentice");
    for (name, pt) in [("Cayth, Famed Mechanist", (3, 3)), ("Marionette Apprentice", (1, 2))] {
        let mut t = TestGame::new(2);
        let c = enter(&mut t, P0, name);
        // The fabricate ability waits on the stack; the creature has no counter.
        assert_eq!(triggers_on_stack(&t, "Fabricate"), 1);
        assert_eq!(t.counters(c, counters::PLUS1), 0);
        assert_eq!(t.pt(c), pt);
        // Players may act meanwhile (here P1 gives it -1/-0); the counter arrives as the
        // ability resolves.
        pump(&mut t, c, -1, 0);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.counters(c, counters::PLUS1), 1);
        assert_eq!(t.pt(c), (pt.0, pt.1 + 1));
    }
}

fn fabricate_state(g: &Game) -> (u32, usize) {
    let counters = g
        .permanents()
        .filter(|o| !o.is_token())
        .map(|o| o.counter(counters::PLUS1))
        .sum();
    let servos = g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Servo"))
        .count();
    (counters, servos)
}

/// The fabricate choice of `name` is made as it resolves, and no player gets priority
/// before the counter or token arrives.
fn fabricate_choice_on_resolution(name: &str) {
    for counter in [true, false] {
        let mut t = TestGame::new(2);
        let seen = watch(&mut t, P0, is_yes_no, fabricate_state);
        let from = t.asked().len();
        let c = enter(&mut t, P0, name);
        assert_eq!(t.stack_len(), 1);
        assert!(seen.lock().unwrap().is_empty());
        t.answer_yes(P0, counter);
        t.resolve();
        assert_eq!(*seen.lock().unwrap(), vec![(0, 0)]);
        assert_eq!(count_asked(&t, from, is_priority), 0);
        assert_eq!(t.counters(c, counters::PLUS1), u32::from(counter));
        assert_eq!(
            tokens_with_subtype(&t, P0, "Servo").len(),
            usize::from(!counter)
        );
    }
}

#[test]
fn the_fabricate_choice_is_made_as_it_resolves_marionette_apprentice() {
    cr!("702.123a", "608.2");
    ruling!(
        "Marionette Apprentice",
        "You choose whether to put a +1/+1 counter on the creature or create a Servo token as the fabricate ability is resolving. No player may take actions between the time you choose and the time that a counter is added or a token is created."
    );
    fabricate_choice_on_resolution("Marionette Apprentice");
}

#[test]
fn the_fabricate_choice_is_made_as_it_resolves_cayth() {
    cr!("702.123a", "608.2");
    ruling!(
        "Cayth, Famed Mechanist",
        "You choose whether to put a +1/+1 counter on the creature or create a Servo token as the fabricate ability is resolving. No player may take actions between the time you choose and the time that the counter is added or the token is created."
    );
    fabricate_choice_on_resolution("Cayth, Famed Mechanist");
}

// ---------------------------------------------------------------------------------------
// Fast Healing (Old One Eye)
// ---------------------------------------------------------------------------------------

#[test]
fn fast_healing_triggers_only_from_the_graveyard() {
    cr!("113.6k", "505.1", "603.2");
    ruling!(
        "Old One Eye",
        "Old One-Eye's Fast Healing ability only triggers if it is in your graveyard at the beginning of your precombat main phase."
    );
    supported("Old One Eye");
    // "Fast Healing — At the beginning of your first main phase, you may discard two cards.
    // If you do, return this card from your graveyard to your hand."
    for zone in ["graveyard", "hand", "battlefield"] {
        let mut t = TestGame::new(2);
        match zone {
            "graveyard" => t.graveyard(P0, "Old One Eye"),
            "hand" => t.hand(P0, "Old One Eye"),
            _ => t.battlefield(P0, "Old One Eye"),
        };
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "discard two cards"),
            usize::from(zone == "graveyard"),
            "{zone}"
        );
    }
}

fn old_one_eye_where(g: &Game) -> bool {
    !g.find_in_zone(Zone::Graveyard(PlayerId(0)), "Old One Eye")
        .is_empty()
}

#[test]
fn fast_healing_discard_and_return_happen_together() {
    cr!("608.2", "117.3b");
    ruling!(
        "Old One Eye",
        "Opponents may respond to the Fast Healing ability after it goes on the stack, but they don't get the chance to cast spells or activate abilities between the time you have chosen to discard two cards and the time Old One-Eye is returned to your hand."
    );
    let mut t = TestGame::new(2);
    let ooe = t.graveyard(P0, "Old One Eye");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The ability is on the stack: P1 may respond (e.g. P1 gets priority now).
    let is_choice = |d: &Decision| {
        matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })
    };
    let seen = watch(&mut t, P0, is_choice, old_one_eye_where);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.resolve();
    // When P0 chose, it was still in the graveyard; right after, it's in hand, with no
    // priority in between.
    assert!(seen.lock().unwrap().iter().all(|s| *s));
    assert!(!seen.lock().unwrap().is_empty());
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.zone(ooe), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

// ---------------------------------------------------------------------------------------
// Fateful hour (Thraben Doomsayer)
// ---------------------------------------------------------------------------------------

#[test]
fn fateful_hour_bonus_ends_when_life_rises_above_5() {
    cr!("611.3a", "704.5g");
    ruling!(
        "Thraben Doomsayer",
        "A creature whose toughness is being increased by Thraben Doomsayer's fateful hour ability loses that bonus as soon as its controller has 6 or more life (or as soon as Thraben Doomsayer leaves the battlefield). If damage marked on that creature is greater than or equal to its toughness after the fateful hour ability stops applying, the creature is destroyed and put into its owner's graveyard."
    );
    supported("Thraben Doomsayer");
    // Thraben Doomsayer: "Fateful hour — As long as you have 5 or less life, other
    // creatures you control get +2/+2."
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let ds = t.battlefield(P0, "Thraben Doomsayer");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.player_mut(P0).life = 5;
        t.g.recompute();
        assert_eq!(t.pt(bears), (4, 4));
        t.g.objects[bears.0 as usize].damage = 3;
        t.settle();
        assert!(t.on_battlefield(bears));
        if leave {
            destroy(&mut t, ds);
        } else {
            t.g.gain_life(P0, 1);
            t.g.flush_events();
            t.settle();
        }
        assert!(!t.on_battlefield(bears));
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
    }
}
