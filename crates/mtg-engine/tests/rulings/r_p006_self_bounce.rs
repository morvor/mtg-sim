//! Rulings batch P006 — triggered abilities that return their own source to its owner's
//! hand: they do nothing once the source has left the battlefield (CR 400.7, 608.2b);
//! they function only on the battlefield (CR 113.6); intervening "if" clauses are checked
//! when the ability would trigger and again on resolution (CR 603.4); the source's last
//! known information (CR 608.2h); triggered abilities resolve before the spell that
//! triggered them (CR 405.5, 603.3).

use crate::r_p006_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use mtg_engine::ability::Modification;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Advances to `p`'s end step and puts the end step triggers on the stack.
fn end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

/// Advances to `p`'s next upkeep and puts the upkeep triggers on the stack.
fn upkeep(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

#[test]
fn archwing_dragon_and_mark_of_fury_return_only_from_the_battlefield() {
    cr!("400.7", "608.2b");
    ruling!(
        "Archwing Dragon",
        "Archwing Dragon is only returned to its owner’s hand if it’s on the battlefield."
    );
    ruling!(
        "Mark of Fury",
        "It only returns if it is still on the battlefield at end of turn."
    );
    // On the battlefield: returned.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Archwing Dragon");
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, dragon), 1);
    t.resolve_all();
    assert!(in_hand(&t, dragon));
    // Destroyed with the trigger on the stack: stays in the graveyard.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Archwing Dragon");
    end_step(&mut t, P0);
    destroy(&mut t, dragon);
    t.resolve_all();
    assert!(in_graveyard(&t, dragon));

    // Mark of Fury: its creature dies with the trigger on the stack, so it's put into the
    // graveyard (CR 704.5m) and isn't returned from there.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mark_ = attach_new(&mut t, P0, "Mark of Fury", obj(bears));
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, mark_), 1);
    destroy(&mut t, bears);
    assert!(in_graveyard(&t, mark_));
    t.resolve_all();
    assert!(in_graveyard(&t, mark_));
    // Still attached at resolution: returned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mark_ = attach_new(&mut t, P0, "Mark of Fury", obj(bears));
    end_step(&mut t, P0);
    t.resolve_all();
    assert!(in_hand(&t, mark_));
}

#[test]
fn viashino_sandstalker_returns_at_the_end_of_any_turn() {
    cr!("513.1a", "603.2");
    ruling!(
        "Viashino Sandstalker",
        "It is returned to its owner’s hand at the end of any turn in which it is on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PostcombatMain);
    let stalker = t.battlefield(P0, "Viashino Sandstalker");
    end_step(&mut t, P1);
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(triggers_from(&t, stalker), 1);
    t.resolve_all();
    assert!(in_hand(&t, stalker));
}

#[test]
fn wall_of_junk_returns_at_end_of_combat_only_if_still_there() {
    cr!("603.7", "400.7");
    ruling!(
        "Wall of Junk",
        "It only returns to owner's hand if it is still on the battlefield at end of combat."
    );
    for survives in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Junk");
        crate::r_s01_common::attack_with(&mut t, &[(bears, pl(P1))]);
        crate::r_s01_common::block_and_finish(&mut t, P1, &[(wall, bears)]);
        t.settle();
        assert!(t.on_battlefield(wall), "a 0/7 survives the Bears");
        assert_eq!(triggers_from(&t, wall), 1, "the delayed trigger");
        if !survives {
            destroy(&mut t, wall);
        }
        t.resolve_all();
        if survives {
            assert!(in_hand(&t, wall));
        } else {
            assert!(in_graveyard(&t, wall));
        }
    }
}

#[test]
fn jackalope_herd_returns_only_if_still_on_the_battlefield() {
    cr!("400.7", "608.2b");
    ruling!(
        "Jackalope Herd",
        "It will only return if it is still on the battlefield when the triggered ability resolves."
    );
    supported("Jackalope Herd");
    for kill_first in [false, true] {
        let mut t = TestGame::new(2);
        let herd = t.battlefield(P0, "Jackalope Herd");
        t.lands(P0, "Island", 1);
        let opt = t.hand(P0, "Opt");
        t.cast(P0, opt).go();
        t.settle();
        assert_eq!(triggers_from(&t, herd), 1);
        if kill_first {
            destroy(&mut t, herd);
        }
        t.resolve_all();
        if kill_first {
            assert!(in_graveyard(&t, herd));
        } else {
            assert!(in_hand(&t, herd));
        }
    }
}

#[test]
fn undiscovered_paradise_returns_at_the_next_untap_step() {
    cr!("502.3", "603.7");
    ruling!(
        "Undiscovered Paradise",
        "It will return even if some effect prevents you from untapping it. The ability is not replacing the untapping of the land in any way."
    );
    ruling!(
        "Undiscovered Paradise",
        "Only returns to owner’s hand if it is still on the battlefield at the beginning of its controller’s next untap."
    );
    supported("Back to Basics");
    // Back to Basics keeps it (a nonbasic land) from untapping: it returns anyway.
    let mut t = TestGame::new(2);
    let paradise = t.battlefield(P0, "Undiscovered Paradise");
    t.battlefield(P1, "Back to Basics");
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, paradise, "Add"
    ));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(paradise), "not during the opponent's untap step");
    t.advance_to(P0, Step::Upkeep);
    assert!(in_hand(&t, paradise));
    // It left the battlefield before then: it stays where it is.
    let mut t = TestGame::new(2);
    let paradise = t.battlefield(P0, "Undiscovered Paradise");
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, paradise, "Add"
    ));
    destroy(&mut t, paradise);
    t.advance_to(P0, Step::Upkeep);
    assert!(in_graveyard(&t, paradise));
}

#[test]
fn mana_bloom_checks_its_counters_as_the_upkeep_begins() {
    cr!("603.4", "502.4");
    ruling!(
        "Mana Bloom",
        "Mana Bloom's last ability will trigger only if it has no charge counters on it at the beginning of your upkeep. If it has one charge counter on it as your turn begins, you won't have priority to remove the counter until after this check."
    );
    supported("Mana Bloom");
    // One counter: no trigger. Removing it during the upkeep doesn't trigger it either.
    let mut t = TestGame::new(2);
    let bloom = t.battlefield(P0, "Mana Bloom");
    t.g.add_counters(obj(bloom), counters::CHARGE, 1, None);
    t.advance_to(P1, Step::Upkeep);
    upkeep(&mut t, P0);
    assert_eq!(t.g.turn.step, Step::Upkeep);
    assert_eq!(triggers_from(&t, bloom), 0);
    // The first chance to remove the counter is now, after the check.
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, bloom, "Add"
    ));
    assert_eq!(t.counters(bloom, counters::CHARGE), 0);
    t.settle();
    assert_eq!(triggers_from(&t, bloom), 0);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.on_battlefield(bloom));
    // No counters as the upkeep begins: it triggers and returns.
    upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, bloom), 1);
    t.resolve_all();
    assert!(in_hand(&t, bloom));
}

#[test]
fn imaginary_pet_checks_the_hand_twice() {
    cr!("603.4");
    ruling!(
        "Imaginary Pet",
        "Imaginary Pet’s ability triggers only if you have cards in your hand as your upkeep begins, and the ability checks again as it resolves. If your hand is empty at both those times, Imaginary Pet stays on the battlefield."
    );
    // Empty hand: no trigger.
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Imaginary Pet");
    t.advance_to(P1, Step::Upkeep);
    t.g.player_mut(P0).hand.clear();
    upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, pet), 0);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.on_battlefield(pet));
    // A card as the upkeep begins, none on resolution: it stays.
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Imaginary Pet");
    t.advance_to(P1, Step::Upkeep);
    t.g.player_mut(P0).hand.clear();
    let card = t.hand(P0, "Grizzly Bears");
    upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, pet), 1);
    crate::r_s05_common::move_to(&mut t, card, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(pet));
    // A card both times: returned.
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Imaginary Pet");
    t.advance_to(P1, Step::Upkeep);
    t.hand(P0, "Grizzly Bears");
    upkeep(&mut t, P0);
    t.resolve_all();
    assert!(in_hand(&t, pet));
}

#[test]
fn dust_stalker_intervening_if_and_its_own_color() {
    cr!("603.4", "113.6");
    ruling!(
        "Dust Stalker",
        "Dust Stalker’s last ability checks to see if you control another colorless creature as the end step begins. If you do, the ability won’t trigger. If you don’t, the ability will check again as it tries to resolve. If you somehow control another colorless creature at that time, the ability won’t do anything. You won’t return Dust Stalker to its owner’s hand."
    );
    ruling!(
        "Dust Stalker",
        "Dust Stalker’s last ability doesn’t depend on Dust Stalker being colorless. If it gains one or more colors, the ability will still look for another creature you control that’s colorless."
    );
    ruling!(
        "Dust Stalker",
        "Like most abilities, Dust Stalker’s last ability functions only while Dust Stalker is on the battlefield."
    );
    supported("Dust Stalker");
    // Another colorless creature at the beginning of the end step: no trigger.
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Dust Stalker");
    t.battlefield(P0, "Ornithopter");
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, stalker), 0);
    // None then (a colored creature doesn't count), one by resolution: nothing happens.
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Dust Stalker");
    t.battlefield(P0, "Grizzly Bears");
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, stalker), 1);
    t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert!(t.on_battlefield(stalker));
    // Dust Stalker turned red: it still looks only for other colorless creatures.
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Dust Stalker");
    crate::r_p021_common::modify(
        &mut t,
        stalker,
        vec![Modification::SetColors(crate::r_p021_common::red())],
    );
    assert!(t.obj_now(stalker).chars.colors.contains(Color::Red));
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, stalker), 1);
    t.resolve_all();
    assert!(in_hand(&t, stalker));
    // An opponent's colorless creature doesn't stop it; in the graveyard it does nothing.
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Dust Stalker");
    let dead = t.graveyard(P0, "Dust Stalker");
    t.battlefield(P1, "Ornithopter");
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, stalker), 1);
    assert_eq!(triggers_from(&t, dead), 0);
    t.resolve_all();
    assert!(in_hand(&t, stalker));
    assert!(in_graveyard(&t, dead));
}

/// Riders of the Mark attacks P1 alone (unblocked), and the game goes to P0's end step
/// with its trigger on the stack.
fn riders_attacked() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Riders of the Mark");
    t.attack(&[(riders, pl(P1))], &[]);
    end_step(&mut t, P0);
    assert_eq!(triggers_from(&t, riders), 1);
    (t, riders)
}

fn human_soldiers(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Soldier"))
        .count()
}

#[test]
fn riders_of_the_mark_tokens_use_its_last_toughness() {
    cr!("608.2h", "603.4");
    ruling!(
        "Riders of the Mark",
        "Use Riders of the Mark's toughness as it last existed on the battlefield to determine how many 1/1 white Human Soldier creature tokens to create."
    );
    supported("Riders of the Mark");
    let (mut t, riders) = riders_attacked();
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(riders)).go();
    t.resolve();
    // Giant Growth resolved; the trigger is still waiting.
    assert_eq!(triggers_from(&t, riders), 1);
    assert_eq!(t.pt(riders), (10, 7));
    t.resolve_all();
    assert!(in_hand(&t, riders));
    assert_eq!(human_soldiers(&t), 7);
}

#[test]
fn riders_of_the_mark_gone_or_returned_otherwise_makes_no_tokens() {
    cr!("608.2b", "400.7", "603.4");
    ruling!(
        "Riders of the Mark",
        "If Riders of the Mark leaves the battlefield before its triggered ability resolves, you can't return it to its owner's hand from its new zone."
    );
    ruling!(
        "Riders of the Mark",
        "You won't create tokens if Riders of the Mark is returned to your hand by an effect other than its own  triggered ability."
    );
    // Destroyed in response: stays in the graveyard, no tokens.
    let (mut t, riders) = riders_attacked();
    destroy(&mut t, riders);
    t.resolve_all();
    assert!(in_graveyard(&t, riders));
    assert_eq!(human_soldiers(&t), 0);
    // Returned by Unsummon in response: no tokens.
    let (mut t, riders) = riders_attacked();
    t.lands(P0, "Island", 1);
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(obj(riders)).go();
    t.resolve_all();
    assert!(in_hand(&t, riders));
    assert_eq!(human_soldiers(&t), 0);
    // Its own trigger: four tokens.
    let (mut t, riders) = riders_attacked();
    t.resolve_all();
    assert!(in_hand(&t, riders));
    assert_eq!(human_soldiers(&t), 4);
}

#[test]
fn mistbreath_elder_returns_another_or_may_return_itself() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Mistbreath Elder",
        "If you control creatures other than Mistbreath Elder as its triggered ability resolves, you must return one of them to its owner’s hand. If you don’t, you have the option to return Mistbreath Elder to its owner’s hand or leave it on the battlefield and have the ability do nothing."
    );
    supported("Mistbreath Elder");
    // Another creature: it must be returned (no "may"), and the Elder gets a counter.
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Mistbreath Elder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    upkeep(&mut t, P0);
    let from = mark(&t);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(in_hand(&t, bears));
    assert!(t.on_battlefield(elder));
    assert_eq!(t.counters(elder, counters::PLUS1), 1);
    assert_eq!(yes_no_since(&t, P0, from), 0, "not optional");
    // Alone: it may return itself, or stay.
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let elder = t.battlefield(P0, "Mistbreath Elder");
        t.advance_to(P1, Step::Upkeep);
        upkeep(&mut t, P0);
        let from = mark(&t);
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(yes_no_since(&t, P0, from), 1);
        assert_eq!(in_hand(&t, elder), yes);
        assert_eq!(t.counters(elder, counters::PLUS1), 0);
    }
}

#[test]
fn greenbelt_rampager_pays_energy_if_it_can() {
    cr!("107.14", "608.2d");
    ruling!(
        "Greenbelt Rampager",
        "If you have only one {E}, you can't pay it even if you want to."
    );
    ruling!(
        "Greenbelt Rampager",
        "If you have {E}{E}, you have to pay it, even if you don't want to."
    );
    let energy = |t: &TestGame| t.g.player(P0).counter(counters::ENERGY);
    // {E}: can't pay; returned, and gets {E} (now two).
    let mut t = TestGame::new(2);
    t.g.add_counters(pl(P0), counters::ENERGY, 1, None);
    let r = t.enter(P0, "Greenbelt Rampager");
    t.resolve_all();
    assert!(in_hand(&t, r));
    assert_eq!(energy(&t), 2);
    // {E}{E}{E}: it pays two even when answering no to everything.
    let mut t = TestGame::new(2);
    t.g.add_counters(pl(P0), counters::ENERGY, 3, None);
    let r = t.enter(P0, "Greenbelt Rampager");
    t.settle();
    let from = mark(&t);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(yes_no_since(&t, P0, from), 0, "not optional");
    assert!(t.on_battlefield(r));
    assert_eq!(energy(&t), 1);
}

#[test]
fn ephara_s_enlightenment_triggers_and_returns_only_from_the_battlefield() {
    cr!("113.6", "400.7", "603.2");
    ruling!(
        "Ephara's Enlightenment",
        "The last ability of Ephara's Enlightenment will trigger only if it's on the battlefield when the creature enters the battlefield under your control. You may return it to its owner’s hand only if it's still on the battlefield when that ability resolves."
    );
    supported("Ephara's Enlightenment");
    // In the graveyard: no trigger.
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Ephara's Enlightenment");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_from(&t, dead), 0);
    // An opponent's creature doesn't trigger it.
    let mut t = TestGame::new(2);
    let host = t.battlefield(P0, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Ephara's Enlightenment", obj(host));
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_from(&t, aura), 0);
    // Triggered, then the Aura goes to the graveyard (its creature died): not returned.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_from(&t, aura), 1);
    destroy(&mut t, host);
    assert!(in_graveyard(&t, aura));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(in_graveyard(&t, aura));
    // Still there: may return it.
    let mut t = TestGame::new(2);
    let host = t.battlefield(P0, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Ephara's Enlightenment", obj(host));
    t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(in_hand(&t, aura));
}

#[test]
fn squirrel_sanctuary_works_only_on_the_battlefield() {
    cr!("113.6");
    ruling!(
        "Squirrel Sanctuary",
        "The last ability functions only while Squirrel Sanctuary is on the battlefield. It won't return itself from your graveyard to your hand."
    );
    supported("Squirrel Sanctuary");
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Squirrel Sanctuary");
    let live = t.battlefield(P0, "Squirrel Sanctuary");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    destroy(&mut t, bears);
    assert_eq!(triggers_from(&t, dead), 0);
    assert_eq!(triggers_from(&t, live), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(in_graveyard(&t, dead));
    assert!(in_hand(&t, live));
}

#[test]
fn dovin_s_acuity_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "113.6");
    ruling!(
        "Dovin's Acuity",
        "The second ability of Dovin’s Acuity resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Dovin's Acuity",
        "The second ability of Dovin’s Acuity triggers only if it’s on the battlefield. It won’t return from your graveyard."
    );
    supported("Dovin's Acuity");
    let mut t = TestGame::new(2);
    let acuity = t.battlefield(P0, "Dovin's Acuity");
    let dead = t.graveyard(P0, "Dovin's Acuity");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    let opt = t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, acuity), 1);
    assert_eq!(triggers_from(&t, dead), 0);
    // P1 counters Opt on top of the trigger.
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(opt)).go();
    t.resolve(); // Cancel
    assert!(in_graveyard(&t, opt), "Opt countered");
    t.answer_yes(P0, true);
    t.resolve(); // the trigger
    assert!(in_hand(&t, acuity));
    assert!(in_graveyard(&t, dead));
    // Without the counterspell: the trigger resolves while Opt waits.
    let mut t = TestGame::new(2);
    let acuity = t.battlefield(P0, "Dovin's Acuity");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    let opt = t.cast(P0, opt).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(in_hand(&t, acuity));
    assert_eq!(t.zone(opt), Zone::Stack);
}

#[test]
fn petrahydrox_returns_before_the_spell_which_then_fizzles() {
    cr!("603.3", "608.2b", "113.6");
    ruling!(
        "Petrahydrox",
        "If the spell or ability that targets Petrahydrox has no other targets, it won't resolve (because it no longer has a legal target after Petrahydrox has returned to its owner's hand)."
    );
    ruling!(
        "Petrahydrox",
        "Petrahydrox's ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    ruling!(
        "Petrahydrox",
        "Petrahydrox's ability triggers only if it's on the battlefield when it becomes the target of a spell or ability."
    );
    supported("Petrahydrox");
    // Bolt at it: the trigger returns it first, then Bolt has no legal target.
    let mut t = TestGame::new(2);
    let hydrox = t.battlefield(P0, "Petrahydrox");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(obj(hydrox)).go();
    t.settle();
    assert_eq!(triggers_from(&t, hydrox), 1);
    t.resolve();
    assert!(in_hand(&t, hydrox));
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve();
    assert!(in_graveyard(&t, bolt));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // The Bolt is countered on top of the trigger: the trigger still resolves.
    let mut t = TestGame::new(2);
    let hydrox = t.battlefield(P0, "Petrahydrox");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(obj(hydrox)).go();
    t.settle();
    t.lands(P0, "Island", 3);
    let cancel = t.hand(P0, "Cancel");
    t.cast(P0, cancel).target(obj(bolt)).go();
    t.resolve_all();
    assert!(in_graveyard(&t, bolt));
    assert!(in_hand(&t, hydrox));
    // Targeted in the graveyard (Raise Dead): no trigger.
    let mut t = TestGame::new(2);
    let hydrox = t.graveyard(P0, "Petrahydrox");
    t.lands(P0, "Swamp", 1);
    let raise = t.hand(P0, "Raise Dead");
    t.cast(P0, raise).target(obj(hydrox)).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "only Raise Dead");
}

#[test]
fn mirozel_returns_when_targeted_or_retargeted() {
    cr!("603.2", "115.7");
    ruling!(
        "Mirozel",
        "Triggers on any spell or ability being announced which targets this card. This means it returns to owner’s hand before that spell resolves."
    );
    ruling!(
        "Mirozel",
        "Triggers on a spell or ability being changed to target this card."
    );
    supported("Mirozel");
    // Shock at it: returned before Shock resolves.
    let mut t = TestGame::new(2);
    let miro = t.battlefield(P0, "Mirozel");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(obj(miro)).go();
    t.settle();
    assert_eq!(triggers_from(&t, miro), 1);
    t.resolve();
    assert!(in_hand(&t, miro));
    assert_eq!(t.zone(shock), Zone::Stack);
    // Shock at the Bears, then Deflection changes its target to Mirozel.
    let mut t = TestGame::new(2);
    let miro = t.battlefield(P0, "Mirozel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Island", 4);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(obj(bears)).go();
    t.settle();
    assert_eq!(triggers_from(&t, miro), 0);
    let deflection = t.hand(P1, "Deflection");
    t.cast(P1, deflection).target(obj(shock)).go();
    t.answer_targets(P1, &[obj(miro)]);
    t.resolve(); // Deflection
    assert_eq!(targets_of(&t, shock), vec![obj(miro)]);
    assert_eq!(triggers_from(&t, miro), 1);
    t.resolve_all();
    assert!(in_hand(&t, miro));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}
