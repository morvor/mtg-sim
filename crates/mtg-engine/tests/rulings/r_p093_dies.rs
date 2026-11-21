//! Rulings batch P093 — "whenever this or another creature dies, you draw a card and you lose
//! 1 life" and "you may pay N life. If you do, draw": creatures dying together with the
//! source (CR 603.10a), mandatory and optional payments (CR 603.5, 119.4), last known
//! information (Nikara, Cleopatra, Mask of Griselbrand, Greven), and Necrodominance's
//! graveyard replacement.

use crate::r_p093_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// One case of "the source dies at the same time as other creatures": P0 controls
/// `source` and `others`; Wrath of God destroys them all; P0 draws `draws` cards and ends
/// at `life`.
struct Together {
    source: &'static str,
    others: &'static [&'static str],
    draws: usize,
    life: i32,
}

fn dies_together(c: &Together) {
    supported(c.source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, c.source);
    for o in c.others {
        t.battlefield(P0, o);
    }
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    cast_only(&mut t, P0, "Wrath of God", &[]);
    // Mana for Miara's "{1} and 1 life", three times.
    add_mana(&mut t, P0, ManaType::B, 3);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.named_on_battlefield(c.source).is_empty(), "{}", c.source);
    assert_eq!(t.hand_size(P0), hand + c.draws, "{}: cards drawn", c.source);
    assert_eq!(t.life(P0), c.life, "{}: life", c.source);
}

#[test]
fn the_source_dying_with_other_creatures_triggers_for_each() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Midnight Entourage",
        "If Midnight Entourage and another Aetherborn you control die at the same time, its ability triggers once for each of them."
    );
    dies_together(&Together {
        source: "Midnight Entourage",
        others: &["Gifted Aetherborn"],
        draws: 2,
        life: 18,
    });
    ruling!(
        "Midnight Reaper",
        "If Midnight Reaper dies at the same time as one or more other nontoken creatures you control, Midnight Reaper's ability triggers for each of them."
    );
    // "Whenever a nontoken creature you control dies, this creature deals 1 damage to you
    // and you draw a card." Itself and two others: three triggers.
    dies_together(&Together {
        source: "Midnight Reaper",
        others: &["Grizzly Bears", "Hill Giant"],
        draws: 3,
        life: 17,
    });
    ruling!(
        "Tegwyll, Duke of Splendor",
        "If Tegwyll, Duke of Splendor dies at the same time as one or more other Faeries you control, Tegwyll's ability triggers for each of those other Faeries."
    );
    // "Whenever another Faerie you control dies": not for Tegwyll itself.
    dies_together(&Together {
        source: "Tegwyll, Duke of Splendor",
        others: &["Pestermite", "Pestermite"],
        draws: 2,
        life: 18,
    });
    ruling!(
        "Undead Augur",
        "If Undead Augur dies at the same time as one or more other Zombies you control, Undead Augur's ability triggers for each of them."
    );
    dies_together(&Together {
        source: "Undead Augur",
        others: &["Walking Corpse"],
        draws: 2,
        life: 18,
    });
    ruling!(
        "Miara, Thorn of the Glade",
        "If multiple Elves you control (possibly including Miara, Thorn of the Glade) die at the same time, Miara's ability triggers that many times."
    );
    // Miara and two Llanowar Elves: three triggers, each paid with {1} and 1 life.
    dies_together(&Together {
        source: "Miara, Thorn of the Glade",
        others: &["Llanowar Elves", "Llanowar Elves"],
        draws: 3,
        life: 17,
    });
    ruling!(
        "Agent Venom",
        "If your Agent Venom dies at the same time as one or more other nontoken creatures you control, his ability triggers for each of those creatures."
    );
    // "Whenever another nontoken creature you control dies": two others, two triggers.
    dies_together(&Together {
        source: "Agent Venom",
        others: &["Grizzly Bears", "Hill Giant"],
        draws: 2,
        life: 18,
    });
}

#[test]
fn erebos_triggers_for_creatures_dying_with_it() {
    cr!("603.10a", "700.5");
    ruling!(
        "Erebos, Bleak-Hearted",
        "If one or more creatures you control die at the same time as Erebos, its triggered ability triggers for each of those creatures."
    );
    supported("Erebos, Bleak-Hearted");
    supported("Barter in Blood");
    // Phyrexian Obliterator ({B}{B}{B}{B}) makes Erebos a creature (devotion five); Barter
    // in Blood ("Each player sacrifices two creatures.") makes P0 sacrifice both.
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, Bleak-Hearted");
    t.battlefield(P0, "Phyrexian Obliterator");
    assert!(t.obj(erebos).is(CardType::Creature));
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    cast_spell(&mut t, P0, "Barter in Blood", &[]);
    assert!(t.in_graveyard(P0, "Erebos, Bleak-Hearted"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn erebos_can_target_the_creature_it_sacrifices() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Erebos, Bleak-Hearted",
        "Erebos’s last ability can target the creature you’ll sacrifice to pay its cost. In this case, the ability won’t resolve, but Erebos’s triggered ability will trigger."
    );
    // "{1}{B}, Sacrifice another creature: Target creature gets -2/-1 until end of turn."
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, Bleak-Hearted");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(bears)]);
    activate(&mut t, P0, erebos, &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The ability and Erebos's trigger are on the stack.
    assert_eq!(from_source(&t, erebos), 2);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(giant), (3, 3));
}

fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, targets: &[Entity]) {
    crate::r_p087_common::activate(t, p, source, "Sacrifice another creature", targets);
}

#[test]
fn midnight_reaper_triggers_on_its_own_death_unless_its_a_token() {
    cr!("603.10a", "111.1");
    ruling!(
        "Midnight Reaper",
        "Midnight Reaper's ability triggers when it dies if it's not a token."
    );
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P0, "Midnight Reaper");
    let hand = t.hand_size(P0);
    kill(&mut t, reaper);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
    // A token copy of it: no trigger.
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P1, "Midnight Reaper");
    let copy = crate::r_s17_common::token_copy(&mut t, P0, reaper)[0];
    let hand = t.hand_size(P0);
    kill(&mut t, copy);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
}

/// One mandatory "you draw a card and you lose 1 life" trigger: no question is asked.
#[test]
fn draw_and_lose_triggers_are_mandatory() {
    cr!("608.2c", "119.3");
    ruling!(
        "Midnight Entourage",
        "Midnight Entourage’s triggered ability is mandatory. You draw a card and lose 1 life even if you don’t want to."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Midnight Entourage");
    let aetherborn = t.battlefield(P0, "Gifted Aetherborn");
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    kill(&mut t, aetherborn);
    t.resolve_all();
    assert_eq!(yes_nos(&t, P0, from), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
    ruling!(
        "Kothophed, Soul Hoarder",
        "The triggered ability is mandatory. You can't decline to draw the card and lose life, even if you want to."
    );
    supported("Kothophed, Soul Hoarder");
    // "Whenever a permanent owned by another player is put into a graveyard from the
    // battlefield, you draw a card and you lose 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kothophed, Soul Hoarder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(yes_nos(&t, P0, from), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

#[test]
fn may_pay_triggers_are_paid_at_most_once_each() {
    cr!("603.5", "119.4", "608.2c");
    ruling!(
        "Miara, Thorn of the Glade",
        "Each time Miara's triggered ability resolves, you may pay {1} and 1 life only once to draw only one card."
    );
    // Plenty of mana and life: one question, one payment, one card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Miara, Thorn of the Glade");
    let elf = t.battlefield(P0, "Llanowar Elves");
    add_mana(&mut t, P0, ManaType::B, 5);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    kill(&mut t, elf);
    t.resolve_all();
    t.clear_answers();
    assert_eq!(yes_nos(&t, P0, from), 1);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);

    ruling!(
        "Crossway Troublemakers",
        "You may pay 2 life only once for each Vampire that dies. You can't pay multiple times for the same triggered ability and draw multiple cards."
    );
    supported("Crossway Troublemakers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crossway Troublemakers");
    let vampire = t.battlefield(P0, "Child of Night");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    kill(&mut t, vampire);
    t.resolve_all();
    t.clear_answers();
    assert_eq!(yes_nos(&t, P0, from), 1);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 18));

    ruling!(
        "Raiders' Spoils",
        "You decide whether to pay 1 life as the triggered ability resolves. You may do this only once per ability (in other words, once per Warrior that deals combat damage to a player)."
    );
    supported("Raiders' Spoils");
    // Two Elvish Warriors connect: two triggers, one payment each, decided as each
    // resolves.
    let mut t = TestGame::new(2);
    let spoils = t.battlefield(P0, "Raiders' Spoils");
    let a = t.battlefield(P0, "Elvish Warrior");
    let b = t.battlefield(P0, "Elvish Warrior");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 14);
    assert_eq!(from_source(&t, spoils), 2);
    assert_eq!(yes_nos(&t, P0, from), 0);
    t.resolve();
    assert_eq!(yes_nos(&t, P0, from), 1);
    t.resolve_all();
    t.clear_answers();
    assert_eq!(yes_nos(&t, P0, from), 2);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 2, 18));
}

#[test]
fn pay_exactly_x_life_or_none() {
    cr!("119.4", "603.5");
    ruling!(
        "Mask of Griselbrand",
        "You must pay exactly X life or nothing. You can't pay less life to draw fewer cards."
    );
    // "Whenever equipped creature dies, you may pay X life, where X is its power. If you
    // do, draw X cards." Hill Giant (3 power): one yes/no, no number.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P0, "Mask of Griselbrand", giant);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(yes_nos(&t, P0, from), 1);
    assert_eq!(numbers_asked(&t, from), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 3, 17));
    // With less life than X, it can't be paid at all (CR 119.4).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P0, "Mask of Griselbrand", giant);
    set_life(&mut t, P0, 2);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    kill(&mut t, giant);
    t.resolve_all();
    t.clear_answers();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 2));

    ruling!(
        "Tymna the Weaver",
        "You must pay exactly X life or none. You can't pay less life to draw fewer cards."
    );
    // Three players: two Grizzly Bears deal combat damage to both opponents (X = 2).
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Tymna the Weaver");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(yes_nos(&t, P0, from), 1);
    assert_eq!(numbers_asked(&t, from), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 2, 18));
}

#[test]
fn midnight_entourage_leaving_can_make_damage_lethal() {
    cr!("120.6", "704.5g", "704.5f", "613.4c");
    ruling!(
        "Midnight Entourage",
        "If a creature has been dealt damage, that damage remains marked on it until the cleanup step. If Midnight Entourage leaves the battlefield and another Aetherborn creature you control has been dealt damage, that creature will be destroyed if the damage is now lethal. The same is true if an Aetherborn’s toughness becomes 0 this way."
    );
    // Gifted Aetherborn (2/3) is 3/4 with Midnight Entourage; dealt 3 damage, it survives
    // until the Entourage leaves.
    let mut t = TestGame::new(2);
    let entourage = t.battlefield(P0, "Midnight Entourage");
    let aetherborn = t.battlefield(P0, "Gifted Aetherborn");
    assert_eq!(t.pt(aetherborn), (3, 4));
    crate::r_s06_common::damage(&mut t, entourage, 3, aetherborn);
    assert!(t.on_battlefield(aetherborn));
    crate::r_s05_common::move_to(&mut t, entourage, Zone::Hand(P0));
    assert_eq!(t.zone(entourage), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Gifted Aetherborn"));
    // An Aetherborn whose toughness is 1 only thanks to the Entourage: 0 once it leaves.
    let mut t = TestGame::new(2);
    let entourage = t.battlefield(P0, "Midnight Entourage");
    let aetherborn = t.battlefield(P0, "Gifted Aetherborn");
    modify_until_eot(
        &mut t,
        aetherborn,
        vec![Modification::ModifyPT(Value::c(0), Value::c(-3))],
    );
    assert_eq!(t.pt(aetherborn), (3, 1));
    kill(&mut t, entourage);
    assert!(t.in_graveyard(P0, "Gifted Aetherborn"));
}

#[test]
fn necrodominance_stops_dies_triggers() {
    cr!("614.6", "700.4", "603.6c");
    ruling!(
        "Necrodominance",
        "While Necrodominance is on the battlefield under your control, abilities that trigger whenever a permanent you own is put into your graveyard from the battlefield (for example, \"When this creature dies…\") won't trigger because cards and tokens are never put into your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necrodominance");
    t.battlefield(P0, "Midnight Reaper");
    t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    wrath(&mut t, P0);
    assert!(t.in_exile("Midnight Reaper"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
}

// --- Nikara and Cleopatra: counters on the creature as it last existed -----------------------

#[test]
fn nikara_triggers_for_a_creature_leaving_with_it() {
    cr!("603.10a", "603.4");
    ruling!(
        "Nikara, Lair Scavenger",
        "If a creature with counters on it leaves the battlefield at the same time as Nikara, Nikara's ability triggers for that creature."
    );
    supported("Nikara, Lair Scavenger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nikara, Lair Scavenger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    let hand = t.hand_size(P0);
    wrath(&mut t, P0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

#[test]
fn nikara_sees_counters_removed_in_pairs_as_the_creature_dies() {
    cr!("704.5q", "704.3", "603.10a", "704.5g", "704.5f");
    ruling!(
        "Nikara, Lair Scavenger",
        "If a creature with +1/+1 counters on it receives an equal number of -1/-1 counters and this causes it to be destroyed by lethal damage or put into its owner's graveyard for having 0 or less toughness, Nikara's last ability triggers. That's because the ability checks the creature as it last existed on the battlefield, and it still had those counters on it at that point."
    );
    // Lethal damage: Grizzly Bears with two +1/+1 counters (4/4) dealt 3 damage, then two
    // -1/-1 counters.
    let mut t = TestGame::new(2);
    let nikara = t.battlefield(P0, "Nikara, Lair Scavenger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    crate::r_s06_common::damage(&mut t, nikara, 3, bears);
    assert!(t.on_battlefield(bears));
    let hand = t.hand_size(P0);
    put_counters(&mut t, bears, counters::MINUS1, 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
    // Zero toughness: Grizzly Bears with a +1/+1 counter and -2/-2 until end of turn, then
    // a -1/-1 counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nikara, Lair Scavenger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(-2), Value::c(-2))],
    );
    assert!(t.on_battlefield(bears));
    let hand = t.hand_size(P0);
    put_counters(&mut t, bears, counters::MINUS1, 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

#[test]
fn one_card_however_many_counters() {
    cr!("603.4", "122.1");
    ruling!(
        "Nikara, Lair Scavenger",
        "Nikara's last ability causes you to draw only one card and lose only 1 life, no matter how many counters or kinds of counters that creature had on it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nikara, Lair Scavenger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 3);
    put_counters(&mut t, bears, counters::SHIELD, 1);
    put_counters(&mut t, bears, "stun", 2);
    let hand = t.hand_size(P0);
    // Returned to its owner's hand (the shield counter would stop a destroy).
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 2, 19));

    ruling!(
        "Cleopatra, Exiled Pharaoh",
        "You lose only 2 life when Cleopatra’s last ability resolves regardless of how many counters the legendary creature that died had on it."
    );
    supported("Cleopatra, Exiled Pharaoh");
    // "Whenever a legendary creature with counters on it dies, draw a card for each
    // counter on it. You lose 2 life." Four counters: four cards, 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cleopatra, Exiled Pharaoh");
    let tymna = t.battlefield(P0, "Tymna the Weaver");
    put_counters(&mut t, tymna, counters::PLUS1, 3);
    put_counters(&mut t, tymna, counters::SHIELD, 1);
    let hand = t.hand_size(P0);
    t.g.sacrifice(tymna, P0);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tymna the Weaver"));
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 4, 18));
}

// --- Corpse Augur -------------------------------------------------------------------------

#[test]
fn corpse_augur_counts_on_resolution_including_itself() {
    cr!("608.2h", "603.3d", "603.10a");
    ruling!(
        "Corpse Augur",
        "You choose the target player as the ability goes on the stack, but you determine the value of X as that ability resolves. If you target yourself and Corpse Augur is still in the graveyard, its ability will count itself."
    );
    supported("Corpse Augur");
    // "When this creature dies, you draw X cards and you lose X life, where X is the number
    // of creature cards in target player's graveyard."
    let mut t = TestGame::new(2);
    let augur = t.battlefield(P0, "Corpse Augur");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    kill(&mut t, augur);
    assert_eq!(from_source(&t, augur), 1);
    // A creature card added while the ability waits counts.
    t.graveyard(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 3, 17));
    // Exiled from the graveyard before it resolves: it doesn't count itself.
    let mut t = TestGame::new(2);
    let augur = t.battlefield(P0, "Corpse Augur");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    kill(&mut t, augur);
    let in_gy = t.g.current(augur);
    crate::r_s05_common::move_to(&mut t, in_gy, Zone::Exile);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

// --- Greven, Predator Captain --------------------------------------------------------------

#[test]
fn greven_counts_life_lost_not_net_change() {
    cr!("119.3", "611.3a");
    ruling!(
        "Greven, Predator Captain",
        "Greven checks for how much life you lost, not how much your life total has changed. If you lost 4 life but also gained 6 life, you still lost 4 life and Greven gets +4/+0."
    );
    supported("Greven, Predator Captain");
    let mut t = TestGame::new(2);
    let greven = t.battlefield(P0, "Greven, Predator Captain");
    t.g.lose_life(P0, 4);
    t.g.gain_life(P0, 6);
    t.g.flush_events();
    t.g.recompute();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(greven), (9, 5));
}

/// P0 attacks with Greven and sacrifices `fodder` to its trigger.
fn greven_attacks(t: &mut TestGame, greven: ObjectId, fodder: ObjectId) {
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(fodder)]);
    attack_with(t, &[(greven, Entity::Player(P1))]);
}

#[test]
fn greven_uses_the_sacrificed_creatures_last_known_power_and_toughness() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Greven, Predator Captain",
        "The sacrificed creature's power and toughness are determined by its last known information before it left the battlefield."
    );
    ruling!(
        "Greven, Predator Captain",
        "You sacrifice a creature (or choose not to), draw cards, and lose life all while Greven's triggered ability is resolving. Nothing can happen in between, and no player may choose to take actions."
    );
    // Hill Giant (3/3) with +2/+0 until end of turn: draw 5, lose 3.
    let mut t = TestGame::new(2);
    let greven = t.battlefield(P0, "Greven, Predator Captain");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    greven_attacks(&mut t, greven, giant);
    assert_eq!(from_source(&t, greven), 1);
    // Nothing is sacrificed until the ability resolves.
    assert!(t.on_battlefield(giant));
    let hand = t.hand_size(P0);
    let events = t.g.turn_events.len();
    // One resolution does it all (no reflexive ability or other stack object between the
    // sacrifice and the draws, so no player gets priority in between).
    t.g.resolve_top();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(drawn_since(&t, events, P0), 5);
    assert_eq!(losses_since(&t, events, P0), vec![3]);
    assert_eq!(t.hand_size(P0), hand + 5);
    assert_eq!(t.life(P0), 17);
    // Greven gets +3/+0 for the life lost.
    t.g.recompute();
    assert_eq!(t.pt(greven), (8, 5));
    // Choosing not to: nothing happens.
    let mut t = TestGame::new(2);
    let greven = t.battlefield(P0, "Greven, Predator Captain");
    t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(greven, Entity::Player(P1))]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 20));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
