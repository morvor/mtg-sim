//! Rulings batch P153 — "scales with mana value": when abilities trigger. Intervening
//! "if" clauses are checked as the trigger event happens and again on resolution (CR
//! 603.4); an ability that triggers on one of several objects doing something triggers
//! once per object (CR 603.2c), one on "one or more" once (CR 603.2d); leaves-the-
//! battlefield abilities look back in time (CR 603.10a); and a kicked permanent that
//! "enters with" a keyword has it as it enters (CR 614.1c, 614.12).

use crate::r_p153_common::*;
use crate::r_s01_common::{custom_card, stack_library, supported, triggers_on_stack, watch};
use crate::r_s02_common::destroy;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Runs `p`'s end step from their precombat main phase: the end step triggers are on the
/// stack when this returns.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

#[test]
fn birthing_ritual_triggers_only_if_you_control_a_creature_at_the_end_step() {
    cr!("603.4");
    ruling!(
        "Birthing Ritual",
        "Birthing Ritual's ability checks if you control a creature at the beginning of your end step. If you don't, the ability won't trigger."
    );
    supported("Birthing Ritual");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Birthing Ritual");
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "look at the top seven"), 0);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Birthing Ritual");
    t.battlefield(P0, "Grizzly Bears");
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "look at the top seven"), 1);
}

#[test]
fn birthing_ritual_does_nothing_if_you_control_no_creature_as_it_resolves() {
    cr!("603.4");
    ruling!(
        "Birthing Ritual",
        "If Birthing Ritual's ability does trigger but you control no creatures when it tries to resolve, the ability will do nothing."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Birthing Ritual");
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant"; 7]);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "look at the top seven"), 1);
    destroy(&mut t, bears);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    // Nothing was looked at or sacrificed: no choices, the library untouched.
    assert_eq!(
        crate::r_s04_common::asked_of_since(&t, P0, from, |d| !matches!(
            d,
            Decision::Priority { .. }
        )),
        0,
        "{}",
        t.dump_log()
    );
    assert_eq!(t.library_size(P0), 37);
}

#[test]
fn brimaz_checks_for_a_dead_phyrexian_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Brimaz, Blight of Oreskos",
        "Brimaz's last ability will check as the end step starts to see if a Phyrexian died under your control this turn. If one did not, the ability won't trigger at all."
    );
    supported("Brimaz, Blight of Oreskos");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brimaz, Blight of Oreskos");
    t.battlefield(P0, "Grizzly Bears");
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "proliferate"), 0);
    // A Phyrexian (Phyrexian Rager) died this turn: proliferate.
    let mut t = TestGame::new(2);
    let brimaz = t.battlefield(P0, "Brimaz, Blight of Oreskos");
    t.g.objects[brimaz.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    let rager = t.battlefield(P0, "Phyrexian Rager");
    destroy(&mut t, rager);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "proliferate"), 1);
    t.answer_choose(P0, &[obj(brimaz)]);
    t.resolve_all();
    assert_eq!(t.counters(brimaz, "+1/+1"), 2);
}

#[test]
fn havi_triggers_for_each_legendary_creature_dying_with_it() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Havi, the All-Father",
        "If Havi and another legendary creature you control die at the same time, Havi's last ability will trigger for each of them."
    );
    supported("Havi, the All-Father");
    let mut t = TestGame::new(2);
    let havi = t.battlefield(P0, "Havi, the All-Father");
    let bosh = t.battlefield(P0, "Bosh, Iron Golem");
    t.graveyard(P0, "Mikaeus, the Lunarch");
    let wrath = t.hand(P0, "Wrath of God");
    mana(&mut t, P0, W, 2);
    mana(&mut t, P0, C, 2);
    t.cast(P0, wrath).go();
    t.resolve();
    assert_eq!(t.zone(havi), Zone::Graveyard(P0));
    assert_eq!(t.zone(bosh), Zone::Graveyard(P0));
    assert_eq!(triggers_on_stack(&t, "return target legendary creature card"), 2);
}

#[test]
fn orah_triggers_for_itself_and_for_a_cleric_dying_with_it() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Orah, Skyclave Hierophant",
        "If another Cleric you control dies at the same time as Orah, Orah's ability triggers for that Cleric in addition to triggering for itself."
    );
    supported("Orah, Skyclave Hierophant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orah, Skyclave Hierophant");
    // Samite Healer (a Cleric, mana value 2): Mikaeus (1) is a legal target for both.
    t.battlefield(P0, "Samite Healer");
    t.graveyard(P0, "Mikaeus, the Lunarch");
    let wrath = t.hand(P0, "Wrath of God");
    mana(&mut t, P0, W, 2);
    mana(&mut t, P0, C, 2);
    t.cast(P0, wrath).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "return target Cleric card"), 2);
}

#[test]
fn amzu_triggers_once_for_several_cards_leaving_the_graveyard_together() {
    cr!("603.2d");
    ruling!(
        "Amzu, Swarm's Hunger",
        "If multiple cards leave your graveyard at the same time, Amzu, Swarm's Hunger's ability will trigger only once."
    );
    supported("Amzu, Swarm's Hunger");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amzu, Swarm's Hunger");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Forest");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.g.turn.priority = Some(P1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    crate::r_s06_common::activate_containing(&mut t, P1, crypt, "Exile").unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(triggers_on_stack(&t, "Insect"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    // The greatest mana value among those cards: Hill Giant's 4.
    assert_eq!(t.counters(toks[0], "+1/+1"), 4);
}

#[test]
fn scythe_specter_triggers_once_per_player_dealt_combat_damage() {
    cr!("603.2c", "120.4");
    ruling!(
        "Scythe Specter",
        "If Scythe Specter deals combat damage to multiple players simultaneously (perhaps because some of the combat damage was redirected to another player), its ability will trigger for each of those players."
    );
    supported("Scythe Specter");
    supported("Harm's Way");
    let mut t = TestGame::new(3);
    let specter = t.battlefield(P0, "Scythe Specter");
    // P1: "The next 2 damage that a source of your choice would deal to you ... this turn
    // is dealt to any target instead": the Specter's damage, redirected to P2.
    let hw = t.hand(P1, "Harm's Way");
    mana(&mut t, P1, W, 1);
    t.g.turn.priority = Some(P1);
    t.answer_choose(P1, &[obj(specter)]);
    t.cast(P1, hw).target(Entity::Player(P2)).go();
    t.resolve_all();
    crate::r_s01_common::attack_with(&mut t, &[(specter, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18), "{}", t.dump_log());
    assert_eq!(triggers_on_stack(&t, "each opponent discards"), 2);
}

#[test]
fn scythe_specter_discards_are_chosen_blind_and_made_together() {
    cr!("701.8", "608.2c");
    ruling!(
        "Scythe Specter",
        "Each opponent in turn order chooses a card to discard without revealing it, then all the chosen cards are discarded simultaneously. No one sees what the other players are discarding before deciding which card to discard."
    );
    let mut t = TestGame::new(3);
    let specter = t.battlefield(P0, "Scythe Specter");
    let giant = t.hand(P1, "Hill Giant");
    let bears = t.hand(P2, "Grizzly Bears");
    // When P2 chooses, P1's card is still in P1's hand (nothing has been discarded).
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.player(P1).hand.len(),
    );
    crate::r_s01_common::attack_with(&mut t, &[(specter, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    assert_eq!(t.zone(bears), Zone::Graveyard(P2));
    // Hill Giant (4) is the greatest mana value: P1 loses 4 more life, P2 none.
    assert_eq!((t.life(P1), t.life(P2)), (20 - 4 - 4, 20));
}

#[test]
fn a_kicked_dralnus_pet_enters_with_flying() {
    cr!("614.12", "614.1c", "603.6d");
    ruling!(
        "Dralnu's Pet",
        "If Dralnu's Pet is kicked, Dralnu's Pet will have flying from the moment it enters. This will trigger abilities that look for a creature with flying entering."
    );
    supported("Dralnu's Pet");
    let watcher = custom_card(
        "Skywatcher",
        "Enchantment",
        "{W}",
        None,
        "Whenever a creature with flying you control enters, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let giant = t.hand(P0, "Hill Giant");
    let pet = t.hand(P0, "Dralnu's Pet");
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, B, 1);
    mana(&mut t, P0, C, 3);
    t.answer_choose(P0, &[obj(giant)]);
    let pet = t.cast(P0, pet).kicked(true).go();
    t.resolve();
    assert!(t.obj_now(pet).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.counters(pet, "+1/+1"), 4);
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Not kicked: no flying, no trigger.
    let mut t = TestGame::new(2);
    let watcher = custom_card(
        "Skywatcher",
        "Enchantment",
        "{W}",
        None,
        "Whenever a creature with flying you control enters, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let pet = t.hand(P0, "Dralnu's Pet");
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, C, 1);
    let pet = t.cast(P0, pet).kicked(false).go();
    t.resolve();
    assert!(!t.obj_now(pet).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 0);
}
