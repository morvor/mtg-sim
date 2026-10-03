//! Rulings batch P009 — burn whose amount (or condition) is determined as the spell or
//! ability resolves, not as it's cast or put on the stack (CR 608.2h, 608.2c), and
//! triggered abilities that target as they're put on the stack (CR 603.3d).

use crate::r_p009_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn etb_counts_are_taken_as_the_ability_resolves() {
    cr!("608.2h", "603.3d");
    ruling!(
        "Munitions Expert",
        "The number of Goblins you control is counted only as Munitions Expert’s ability resolves. If Munitions Expert is still on the battlefield, it will count itself."
    );
    ruling!(
        "Kessig Malcontents",
        "The number of Humans you control is counted when the \"enters\" ability resolves."
    );
    ruling!(
        "Gruesome Scourger",
        "The number of creatures you control is counted as Gruesome Scourger’s ability resolves. If Gruesome Scourger is still on the battlefield, it will count itself."
    );
    for n in [
        "Munitions Expert",
        "Kessig Malcontents",
        "Gruesome Scourger",
    ] {
        supported(n);
    }

    // Munitions Expert (a Goblin): another Goblin arrives in response -> 2 damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Munitions Expert");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    add(&mut t, P0, "Goblin Piker");
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 2);

    // ... and if it left the battlefield in response, it doesn't count itself.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Goblin Piker");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    let expert = t.enter(P0, "Munitions Expert");
    t.settle();
    kill(&mut t, expert);
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 1);

    // Kessig Malcontents (a Human): a second Human arrives in response -> 2 damage.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Kessig Malcontents");
    t.settle();
    add(&mut t, P0, "Elite Vanguard");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);

    // Gruesome Scourger counts itself and creatures that arrived in response.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Gruesome Scourger");
    t.settle();
    add(&mut t, P0, "Grizzly Bears");
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn spell_counts_are_taken_as_the_spell_resolves() {
    cr!("608.2h");
    ruling!(
        "Stensia Banquet",
        "The number of Vampires you control is counted only as Stensia Banquet resolves."
    );
    ruling!(
        "Scrapyard Salvo",
        "The number of artifact cards in your graveyard is counted when Scrapyard Salvo resolves."
    );
    ruling!(
        "Consuming Corruption",
        "The value of X is calculated only once, as Consuming Corruption resolves."
    );
    for n in ["Stensia Banquet", "Scrapyard Salvo", "Consuming Corruption"] {
        supported(n);
    }

    // Stensia Banquet: one Vampire when cast, two when it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodthrone Vampire");
    cast_new(&mut t, P0, "Stensia Banquet", &[pl(P1)]);
    add(&mut t, P0, "Bloodthrone Vampire");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);

    // Scrapyard Salvo: an artifact card is put into the graveyard in response.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    cast_new(&mut t, P0, "Scrapyard Salvo", &[pl(P1)]);
    let thopter = add(&mut t, P0, "Ornithopter");
    kill(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);

    // Consuming Corruption: X is the number of Swamps as it resolves (its own two plus
    // one more), and that one X is both the damage and the life gained.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Consuming Corruption", &[obj(wurm)]);
    add(&mut t, P0, "Swamp");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn stensia_banquet_with_no_vampires_still_draws() {
    cr!("608.2h", "120.8");
    ruling!(
        "Stensia Banquet",
        "You may cast Stensia Banquet targeting an opponent even if you control no Vampires. You’ll still draw a card."
    );
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Stensia Banquet", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn frost_bite_checks_snow_permanents_as_it_resolves() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Frost Bite",
        "Whether or not you control three or more snow permanents is checked as Frost Bite is resolving."
    );
    supported("Frost Bite");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Snow-Covered Mountain", 2);
    let bite = t.hand(P0, "Frost Bite");
    t.cast(P0, bite).target(wurm).go();
    add(&mut t, P0, "Snow-Covered Forest");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);

    // Three when cast, two as it resolves: 2 damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let lands = t.lands(P0, "Snow-Covered Mountain", 3);
    let bite = t.hand(P0, "Frost Bite");
    t.cast(P0, bite).target(wurm).go();
    kill(&mut t, lands[2]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

#[test]
fn unholy_heat_doesnt_see_itself_in_the_graveyard() {
    cr!("608.2h", "608.2n");
    ruling!(
        "Unholy Heat",
        "Unholy Heat checks your graveyard as it resolves to determine if it deals 2 or 6 damage. At that time, Unholy Heat isn't in the graveyard yet."
    );
    supported("Unholy Heat");
    // Creature, land and sorcery in the graveyard: Unholy Heat (an instant) would be the
    // fourth type, but it isn't there yet.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    for n in ["Grizzly Bears", "Forest", "Divination"] {
        t.graveyard(P0, n);
    }
    cast_new(&mut t, P0, "Unholy Heat", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
    // An artifact card put there in response makes four types: 6 damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    for n in ["Grizzly Bears", "Forest", "Divination"] {
        t.graveyard(P0, n);
    }
    cast_new(&mut t, P0, "Unholy Heat", &[obj(wurm)]);
    let thopter = add(&mut t, P0, "Ornithopter");
    kill(&mut t, thopter);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn voldaren_ambusher_counts_vampires_as_it_resolves() {
    cr!("608.2h", "603.4");
    ruling!(
        "Voldaren Ambusher",
        "Voldaren Ambusher's ability counts the number of Vampires you control as it resolves. If something happens in response such that you don't control any Vampires when it resolves, Voldaren Ambusher won't deal damage to the target creature or planeswalker."
    );
    supported("Voldaren Ambusher");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    lose(&mut t, P1, 1);
    t.answer_targets(P0, &[obj(wurm)]);
    let ambusher = t.enter(P0, "Voldaren Ambusher");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, ambusher);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0);

    // With the Ambusher still there and another Vampire: 2 damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Bloodthrone Vampire");
    lose(&mut t, P1, 1);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Voldaren Ambusher");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

#[test]
fn voldaren_ambusher_cares_about_life_lost_not_net_change() {
    cr!("603.4", "119.3");
    ruling!(
        "Voldaren Ambusher",
        "Voldaren Ambusher's ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    gain(&mut t, P1, 2);
    lose(&mut t, P1, 1);
    assert_eq!(t.life(P1), 21);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Voldaren Ambusher");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 1);
    // Without any life loss, no trigger.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    gain(&mut t, P1, 2);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Voldaren Ambusher");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn goblin_razerunners_counts_counters_as_its_trigger_resolves() {
    cr!("608.2h", "603.3d");
    ruling!(
        "Goblin Razerunners",
        "You don't count how many +1/+1 counters are on Goblin Razerunners until its triggered ability resolves."
    );
    supported("Goblin Razerunners");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Goblin Razerunners");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the end step trigger is on the stack");
    t.g.add_counters(obj(goblin), counters::PLUS1, 2, None);
    t.g.recompute();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ghost_spider_counts_attackers_as_its_trigger_resolves() {
    cr!("608.2h", "506.4");
    ruling!(
        "Ghost-Spider, Gwen Stacy",
        "Count the number of attacking creatures as Ghost-Spider's ability resolves. If a player responded to the ability by removing creatures from combat, this may be different than the number of creatures that were attacking as the ability triggered."
    );
    supported("Ghost-Spider, Gwen Stacy");
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Ghost-Spider, Gwen Stacy");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(spider, pl(P1)), (a, pl(P1)), (b, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    mtg_engine::combat::remove_from_combat(&mut t.g, a);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn upkeep_and_end_step_burn_counts_as_it_resolves() {
    cr!("608.2h", "603.3d");
    ruling!(
        "Karma",
        "Amount of damage is determined when the ability resolves and not when it is placed on the stack."
    );
    ruling!(
        "Citadel of Pain",
        "It counts the untapped lands on resolution."
    );
    supported("Karma");
    supported("Citadel of Pain");
    // Karma: P1 has a Swamp at the beginning of the upkeep and a second one by resolution.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karma");
    t.battlefield(P1, "Swamp");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    add(&mut t, P1, "Swamp");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);

    // Citadel of Pain: three untapped lands, one tapped in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Citadel of Pain");
    let lands = t.lands(P0, "Forest", 3);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.tap(lands[0]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn exploding_borders_counts_basic_land_types_after_the_search() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Exploding Borders",
        "You do what the spell says in order, so you'll put a new basic land card onto the battlefield before you determine the value of X."
    );
    supported("Exploding Borders");
    let mut t = TestGame::new(2);
    let island = t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(island)]);
    cast_new(&mut t, P0, "Exploding Borders", &[pl(P1)]);
    t.resolve_all();
    // Mountain + Forest (paying for it) + the Island found.
    assert!(!t.named_on_battlefield("Island").is_empty());
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cragganwick_cremator_targets_before_the_amount_is_known() {
    cr!("603.3d", "608.2h");
    ruling!(
        "Cragganwick Cremator",
        "You choose the target player or planeswalker when the ability is put onto the stack. You won't know how much damage will be dealt until the ability resolves."
    );
    supported("Cragganwick Cremator");
    let mut t = TestGame::new(2);
    t.hand(P0, "Hill Giant");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Cragganwick Cremator");
    t.settle();
    let ab = t.g.stack[0];
    assert_eq!(crate::r_s25_common::targets_of(&t, ab), vec![pl(P1)]);
    assert_eq!(t.hand_size(P0), 1, "nothing is discarded yet");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
}
