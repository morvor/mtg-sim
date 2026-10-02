//! Rulings batch P108 — morbid and other "if a creature died this turn" abilities
//! (CR 700.4: "dies" means put into a graveyard from the battlefield; CR 603.4:
//! intervening "if" clauses are checked as the ability triggers and as it resolves).

use crate::r_p108_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Moves an object (followed across zone changes) to exile.
fn exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.settle();
}

#[test]
fn emissary_checks_only_that_a_creature_died() {
    cr!("700.4", "603.4");
    ruling!(
        "Emissary of the Sleepless",
        "checks only if a creature died earlier in the turn. The creature card doesn’t need to still be in the graveyard."
    );
    supported("Emissary of the Sleepless");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    exile(&mut t, bears);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    t.enter(P0, "Emissary of the Sleepless");
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 1);
}

#[test]
fn emissary_makes_one_token_however_many_died() {
    cr!("700.4", "603.4");
    ruling!(
        "Emissary of the Sleepless",
        "doesn’t create additional Spirit tokens if more than one creature died this turn."
    );
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P0);
    a_creature_dies(&mut t, P1);
    a_creature_dies(&mut t, P1);
    t.enter(P0, "Emissary of the Sleepless");
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 1);
}

#[test]
fn emissary_sees_a_dead_token() {
    cr!("700.4", "111.7", "704.5d");
    ruling!(
        "Emissary of the Sleepless",
        "Token creatures that are destroyed or put into a graveyard from the battlefield for other reasons do die"
    );
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P1, "Soldier");
    destroy(&mut t, tok);
    assert!(!t.g.is_live(tok));
    t.enter(P0, "Emissary of the Sleepless");
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 1);
}

#[test]
fn without_a_death_emissary_makes_nothing() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.enter(P0, "Emissary of the Sleepless");
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 0);
}

/// `deaths` creatures died under P0's control earlier this turn, then the
/// card `name` is put onto the battlefield and P0's end step begins. Returns the number
/// Returns the card; its triggered abilities are left on the stack.
fn end_step_triggers(t: &mut TestGame, name: &str, deaths: usize) -> ObjectId {
    for _ in 0..deaths {
        a_creature_dies(t, P0);
    }
    let card = t.battlefield(P0, name);
    t.advance_to(P0, Step::End);
    t.settle();
    card
}

#[test]
fn faramir_needs_not_have_seen_the_death_and_triggers_once() {
    cr!("603.4", "700.4");
    ruling!(
        "Faramir, Field Commander",
        "doesn't need to have been on the battlefield when the creature died."
    );
    ruling!(
        "Faramir, Field Commander",
        "first ability will trigger only once during your end step, no matter how many creatures died under your control this turn."
    );
    supported("Faramir, Field Commander");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let f = end_step_triggers(&mut t, "Faramir, Field Commander", 2);
    assert_eq!(crate::r_s25_common::abilities_from(&t, f).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn faramir_doesnt_trigger_without_a_death_under_your_control() {
    cr!("603.4");
    ruling!(
        "Faramir, Field Commander",
        "if no creatures have died under your control so far this turn as your end step begins, the ability won't trigger at all."
    );
    let mut t = TestGame::new(2);
    // An opponent's creature died: not under your control.
    a_creature_dies(&mut t, P1);
    let f = end_step_triggers(&mut t, "Faramir, Field Commander", 0);
    assert!(crate::r_s25_common::abilities_from(&t, f).is_empty());
    // A creature dying during the end step is too late.
    a_creature_dies(&mut t, P0);
    t.resolve_all();
    assert!(crate::r_s25_common::abilities_from(&t, f).is_empty());
}

#[test]
fn muster_the_departed_needs_not_have_seen_the_death_and_populates_once() {
    cr!("603.4", "701.36a");
    ruling!(
        "Muster the Departed",
        "Muster the Departed doesn't need to have been on the battlefield when the creature died."
    );
    ruling!(
        "Muster the Departed",
        "last ability will trigger only once during your end step, no matter how many creatures died under your control this turn."
    );
    supported("Muster the Departed");
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Spirit");
    let m = end_step_triggers(&mut t, "Muster the Departed", 2);
    assert_eq!(crate::r_s25_common::abilities_from(&t, m).len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 2);
}

#[test]
fn muster_the_departed_doesnt_trigger_without_a_death() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Spirit");
    let m = end_step_triggers(&mut t, "Muster the Departed", 0);
    assert!(crate::r_s25_common::abilities_from(&t, m).is_empty());
    a_creature_dies(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Spirit"), 1);
}

#[test]
fn needletooth_pack_checks_as_the_end_step_starts() {
    cr!("603.4");
    ruling!(
        "Needletooth Pack",
        "last ability will check as the end step starts to see if a creature died this turn. If none did, the ability won't trigger at all."
    );
    supported("Needletooth Pack");
    let mut t = TestGame::new(2);
    let p = end_step_triggers(&mut t, "Needletooth Pack", 0);
    assert!(crate::r_s25_common::abilities_from(&t, p).is_empty());
    a_creature_dies(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.counters(p, "+1/+1"), 0);
    // With a death earlier in the turn, it triggers.
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P1);
    let p = end_step_triggers(&mut t, "Needletooth Pack", 0);
    t.answer_targets(P0, &[obj(p)]);
    t.resolve_all();
    assert_eq!(t.counters(p, "+1/+1"), 2);
}

#[test]
fn zombie_ogre_ventures_once_and_needs_not_have_seen_the_death() {
    cr!("603.4", "701.49a");
    ruling!(
        "Zombie Ogre",
        "doesn't need to have been on the battlefield at the time the creature died for its ability to function."
    );
    ruling!(
        "Zombie Ogre",
        "ability will trigger only once during your end step, no matter how many creatures died this turn."
    );
    supported("Zombie Ogre");
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P1);
    let o = end_step_triggers(&mut t, "Zombie Ogre", 2);
    assert_eq!(crate::r_s25_common::abilities_from(&t, o).len(), 1);
    t.resolve_all();
    let (_, room) = t.g.player(P0).venture.expect("ventured");
    assert_eq!(room, 0, "ventured once: in the first room");
    // Without a death, no trigger.
    let mut t = TestGame::new(2);
    let o = end_step_triggers(&mut t, "Zombie Ogre", 0);
    assert!(crate::r_s25_common::abilities_from(&t, o).is_empty());
    a_creature_dies(&mut t, P0);
    t.resolve_all();
    assert!(t.g.player(P0).venture.is_none());
}

/// The card's end step ability (it draws a card) triggers once after a creature died
/// before the card entered, however many died, and doesn't trigger without a death.
fn draws_once_at_end_step(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P1);
    a_creature_dies(&mut t, P1);
    let hand = t.hand_size(P0);
    let c = end_step_triggers(&mut t, name, 1);
    assert_eq!(crate::r_s25_common::abilities_from(&t, c).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    let mut t = TestGame::new(2);
    let c = end_step_triggers(&mut t, name, 0);
    assert!(crate::r_s25_common::abilities_from(&t, c).is_empty(), "{name}");
}

#[test]
fn twinblade_assassins_triggers_once_even_for_a_death_before_it_entered() {
    cr!("603.4", "700.4");
    ruling!(
        "Twinblade Assassins",
        "If a creature didn't die before your end step begins, the ability of Twinblade Assassins doesn't trigger at all. The creature may have died before Twinblade Assassins entered the battlefield, however."
    );
    ruling!(
        "Twinblade Assassins",
        "The triggered ability triggers just once, no matter how many creatures died this turn."
    );
    draws_once_at_end_step("Twinblade Assassins");
}

#[test]
fn sabertooth_mauler_triggers_once() {
    cr!("603.4");
    ruling!(
        "Sabertooth Mauler",
        "The triggered ability triggers just once, no matter how many creatures died this turn."
    );
    supported("Sabertooth Mauler");
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P1);
    let m = end_step_triggers(&mut t, "Sabertooth Mauler", 2);
    assert_eq!(crate::r_s25_common::abilities_from(&t, m).len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(m, "+1/+1"), 1);
}

#[test]
fn lilianas_devotee_triggers_once_for_a_death_before_it_entered() {
    cr!("603.4", "118.12");
    ruling!(
        "Liliana's Devotee",
        "If a creature didn't die before your end step begins, the last ability of Liliana's Devotee doesn't trigger at all. The creature may have died before Liliana's Devotee entered the battlefield, however."
    );
    ruling!(
        "Liliana's Devotee",
        "The triggered ability triggers just once, no matter how many creatures died this turn."
    );
    ruling!(
        "Liliana's Devotee",
        "While resolving the triggered ability, you can't pay more than {1}{B} to get more than one Zombie token."
    );
    supported("Liliana's Devotee");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let d = end_step_triggers(&mut t, "Liliana's Devotee", 3);
    assert_eq!(crate::r_s25_common::abilities_from(&t, d).len(), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Zombie"), 1);
    // No death: no trigger.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let d = end_step_triggers(&mut t, "Liliana's Devotee", 0);
    assert!(crate::r_s25_common::abilities_from(&t, d).is_empty());
}

#[test]
fn fungal_rebirth_makes_two_saprolings_besides_returning_the_card() {
    cr!("700.4", "608.2c");
    ruling!(
        "Fungal Rebirth",
        "Fungal Rebirth checks only whether a creature died this turn. It doesn't give you additional Saprolings if more than one creature died."
    );
    ruling!(
        "Fungal Rebirth",
        "If a creature died this turn, you get Saproling tokens in addition to the permanent card being returned to your hand."
    );
    supported("Fungal Rebirth");
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P0);
    a_creature_dies(&mut t, P1);
    a_creature_dies(&mut t, P1);
    let card = t.graveyard(P0, "Hill Giant");
    cast_new(&mut t, P0, "Fungal Rebirth", &[obj(card)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(tokens_with(&t, P0, "Saproling"), 2);
    // Without a death, just the card.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Hill Giant");
    cast_new(&mut t, P0, "Fungal Rebirth", &[obj(card)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(tokens_with(&t, P0, "Saproling"), 0);
}

#[test]
fn morbid_opportunist_triggers_when_it_dies_with_others() {
    cr!("603.10a", "700.4");
    ruling!(
        "Morbid Opportunist",
        "If Morbid Opportunist dies at the same time as one or more other creatures, Morbid Opportunist's ability still triggers."
    );
    supported("Morbid Opportunist");
    let mut t = TestGame::new(2);
    let mo = t.battlefield(P0, "Morbid Opportunist");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(mo));
    assert_eq!(t.hand_size(P0), hand + 1, "one card: it triggers once");
}

#[test]
fn gravelighters_otherwise_refers_to_a_creature_dying() {
    cr!("608.2c", "603.4");
    ruling!(
        "Gravelighter",
        "The word “otherwise” in Gravelighter's triggered ability refers to whether or not a creature died this turn, not whether or not a card was drawn."
    );
    supported("Gravelighter");
    // A creature died, but the library is empty so no card is drawn: no one sacrifices.
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P1);
    let opp = t.battlefield(P1, "Hill Giant");
    t.g.players[0].library.clear();
    let g = t.enter(P0, "Gravelighter");
    t.resolve_all();
    assert!(t.on_battlefield(opp));
    assert!(t.on_battlefield(g));
    // No creature died: each player sacrifices a creature.
    let mut t = TestGame::new(2);
    let opp = t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    t.enter(P0, "Gravelighter");
    t.resolve_all();
    assert!(!t.on_battlefield(opp));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.named_on_battlefield("Gravelighter").is_empty());
}

#[test]
fn vengeful_devil_can_be_activated_for_a_death_before_it_entered() {
    cr!("602.5", "700.4");
    ruling!(
        "Vengeful Devil",
        "Vengeful Devil's ability can be activated even if a creature died only earlier in the turn before Vengeful Devil entered the battlefield."
    );
    supported("Vengeful Devil");
    let mut t = TestGame::new(2);
    let devil = t.battlefield(P0, "Vengeful Devil");
    assert!(t.activate(P0, devil, 0, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    a_creature_dies(&mut t, P0);
    let devil2 = t.battlefield(P0, "Vengeful Devil");
    t.activate(P0, devil2, 0, &[Entity::Player(P1)])
        .expect("a creature died this turn");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn caged_zombie_drains_each_opponent() {
    cr!("602.5", "119.3");
    supported("Caged Zombie");
    let mut t = TestGame::new(3);
    let z = t.battlefield(P0, "Caged Zombie");
    a_creature_dies(&mut t, P1);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, z, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn caged_zombie_costs_the_opposing_two_headed_giant_team_four_life() {
    cr!("602.5", "810.9");
    ruling!(
        "Caged Zombie",
        "In a Two-Headed Giant game, Caged Zombie's ability causes the opposing team to lose 4 life."
    );
    let mut t = two_headed_giant();
    let z = t.battlefield(P0, "Caged Zombie");
    a_creature_dies(&mut t, P2);
    t.lands(P0, "Swamp", 2);
    let (us, them) = (t.life(P0), t.life(P2));
    t.activate(P0, z, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P2), them - 4);
    assert_eq!(t.life(P3), them - 4);
    assert_eq!(t.life(P0), us);
}

// --- Liliana's Scrounger -------------------------------------------------------------------

#[test]
fn lilianas_scrounger_triggers_for_a_death_before_it_entered() {
    cr!("603.4", "700.4");
    ruling!(
        "Liliana's Scrounger",
        "If a creature didn't die before your end step begins, the ability of Liliana's Scrounger doesn't trigger at all. The creature may have died before Liliana's Scrounger entered the battlefield, however."
    );
    ruling!(
        "Liliana's Scrounger",
        "The triggered ability triggers just once, no matter how many creatures died this turn."
    );
    supported("Liliana's Scrounger");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    let s = end_step_triggers(&mut t, "Liliana's Scrounger", 2);
    assert_eq!(crate::r_s25_common::abilities_from(&t, s).len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(lili, "loyalty"), 4);
    // No death: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana of the Veil");
    let s = end_step_triggers(&mut t, "Liliana's Scrounger", 0);
    assert!(crate::r_s25_common::abilities_from(&t, s).is_empty());
}

#[test]
fn lilianas_scrounger_chooses_the_liliana_as_it_resolves() {
    cr!("608.2c");
    ruling!(
        "Liliana's Scrounger",
        "If you control more than one Liliana planeswalker, you choose which one receives a loyalty counter as the ability of Liliana's Scrounger resolves."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Liliana of the Veil");
    let s = end_step_triggers(&mut t, "Liliana's Scrounger", 1);
    assert_eq!(crate::r_s25_common::abilities_from(&t, s).len(), 1);
    // The second Liliana arrives while the ability is on the stack, and gets the counter.
    let b = t.battlefield(P0, "Liliana, the Last Hope");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert_eq!(t.counters(a, "loyalty"), 3);
    assert_eq!(t.counters(b, "loyalty"), 4);
}
