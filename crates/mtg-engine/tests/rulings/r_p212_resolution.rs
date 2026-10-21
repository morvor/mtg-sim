//! Rulings batch P212 — Equipment (CR 301.5, 702.6) abilities that find "equipped
//! creature" as they resolve (CR 608.2h, 301.5a): Umezawa's Jitte, Helm of the Host and
//! Skyclave Pick-Axe; Nim Deathmantle and Scythe of the Wretched returning creatures
//! (CR 603.2, 603.10a); cost reductions locked in while casting (CR 601.2f-h); and Spy
//! Kit's names with a hidden agenda (CR 702.106).

use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s16_common::add_charge;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Umezawa's Jitte: "Remove a charge counter from Umezawa's Jitte: Choose one — Equipped
// creature gets +2/+2 until end of turn; ..."
// ---------------------------------------------------------------------------------------

/// Activates the Jitte's "+2/+2" mode (as P0).
fn jitte_pump(t: &mut TestGame, jitte: ObjectId) {
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    activate_containing(t, P0, jitte, "Remove a charge counter").unwrap();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn jitte_pump_mode_goes_to_the_creature_equipped_on_resolution() {
    cr!("608.2h", "301.5a", "602.5d");
    ruling!(
        "Umezawa's Jitte",
        "If the Jitte is moved after the \"+2/+2\" mode is announced but before it resolves, the bonus is given to the creature that is equipped when the ability resolves."
    );
    ruling!(
        "Umezawa's Jitte",
        "If the Jitte leaves the battlefield after the \"+2/+2\" mode is announced but before it resolves, the bonus is given to the creature that was most recently equipped once the ability resolves."
    );
    ruling!(
        "Umezawa's Jitte",
        "The ability can be used any time Umezawa's Jitte's controller has priority — only the \"target creature\" choice has additional requirements. Choosing the \"Equipped creature gets +2/+2 until end of turn\" mode does nothing if the Jitte isn't equipped to a creature when the ability resolves."
    );
    supported("Umezawa's Jitte");
    // Moved to the Lion in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let jitte = attach_new(&mut t, P0, "Umezawa's Jitte", bears);
    add_charge(&mut t, jitte, 2);
    jitte_pump(&mut t, jitte);
    move_eq(&mut t, jitte, lion);
    t.resolve_all();
    assert_eq!(t.pt(lion), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
    // Destroyed in response: the Bears (most recently equipped) get it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jitte = attach_new(&mut t, P0, "Umezawa's Jitte", bears);
    add_charge(&mut t, jitte, 2);
    jitte_pump(&mut t, jitte);
    destroy(&mut t, jitte);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // Unattached, during the opponent's turn: it can be activated; nothing gets +2/+2.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    add_charge(&mut t, jitte, 2);
    jitte_pump(&mut t, jitte);
    t.resolve_all();
    assert_eq!(t.counters(jitte, counters::CHARGE), 1);
    assert_eq!(t.pt(bears), (2, 2));
}

// ---------------------------------------------------------------------------------------
// Helm of the Host: "At the beginning of combat on your turn, create a token that's a
// copy of equipped creature, except the token isn't legendary. That token gains haste."
// ---------------------------------------------------------------------------------------

/// Helm of the Host's trigger on the stack at the beginning of P0's combat.
fn helm_trigger(t: &mut TestGame) {
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn helm_of_the_host_uses_the_last_equipped_creature() {
    cr!("608.2h", "707.2", "113.7a");
    ruling!(
        "Helm of the Host",
        "If the equipped creature leaves the battlefield before the triggered ability of Helm of the Host resolves, or if there is no equipped creature, no token is created. However, if Helm of the Host leaves the battlefield while its triggered ability is on the stack, a token will be created of the creature it last equipped. If that creature has also left the battlefield, its last known information is used to determine what the token looks like."
    );
    supported("Helm of the Host");
    // 0: the creature leaves; 1: no equipped creature; 2: the Helm leaves; 3: both leave.
    for case in 0..4 {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let helm = if case == 1 {
            t.battlefield(P0, "Helm of the Host")
        } else {
            attach_new(&mut t, P0, "Helm of the Host", giant)
        };
        helm_trigger(&mut t);
        if case == 3 {
            destroy(&mut t, helm);
            destroy(&mut t, giant);
        } else if case == 2 {
            destroy(&mut t, helm);
        } else if case == 0 {
            destroy(&mut t, giant);
        }
        t.resolve_all();
        let toks = tokens(&t, P0);
        if case >= 2 {
            assert_eq!(toks.len(), 1, "case {case}");
            assert_eq!(t.obj_now(toks[0]).chars.name, "Hill Giant", "case {case}");
            assert_eq!(t.pt(toks[0]), (3, 3), "case {case}");
        } else {
            assert!(toks.is_empty(), "case {case}");
        }
    }
}

// ---------------------------------------------------------------------------------------
// Nim Deathmantle
// ---------------------------------------------------------------------------------------

#[test]
fn nim_deathmantle_makes_the_equipped_creature_a_black_zombie() {
    cr!("613.1e", "613.1c", "301.5");
    ruling!(
        "Nim Deathmantle",
        "Nim Deathmantle causes the equipped creature to be a black Zombie even if it didn't return that creature to the battlefield from the graveyard."
    );
    supported("Nim Deathmantle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Nim Deathmantle", bears);
    let o = t.obj_now(bears);
    assert!(o.chars.colors.contains(Color::Black));
    assert!(!o.chars.colors.contains(Color::Green));
    assert!(o.chars.has_subtype("Zombie"));
    assert!(!o.chars.has_subtype("Bear"));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn nim_deathmantle_returns_each_creature_paid_for_and_ends_on_the_last() {
    cr!("603.2c", "603.3b", "701.3b");
    ruling!(
        "Nim Deathmantle",
        "If multiple nontoken creatures are put into your graveyard from the battlefield at the same time, Nim Deathmantle's second ability triggers that many times. You put the triggered abilities on the stack in any order, so you'll determine in which order they resolve. If you pay {4} more than once, each card you paid {4} for will end up on the battlefield under your control, and Nim Deathmantle will end up attached to the last card that returned to the battlefield this way that it could equip."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let mantle = t.battlefield(P0, "Nim Deathmantle");
    t.lands(P0, "Wastes", 8);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_x(&mut t, P1, "Earthquake", 2);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let (b, l) = (t.g.current(bears), t.g.current(lion));
    assert_eq!(t.zone(b), Zone::Battlefield);
    assert_eq!(t.zone(l), Zone::Battlefield);
    // Attached to the one that returned last.
    let last = if t.obj_now(b).timestamp > t.obj_now(l).timestamp {
        b
    } else {
        l
    };
    assert!(on(&t, mantle, last));
}

#[test]
fn nim_deathmantle_returns_a_card_it_cant_equip() {
    cr!("301.5c", "701.3b", "702.16c");
    ruling!(
        "Nim Deathmantle",
        "Nim Deathmantle's second ability may return a card it can't equip to the battlefield. For example, if a nontoken artifact that's become a creature is put into your graveyard from the battlefield, Nim Deathmantle's second ability triggers. If you pay {4} as it resolves, you'll return that card to the battlefield. However, Nim Deathmantle can't equip it, so Nim Deathmantle remains attached to whatever it was already equipping (or, if it was unattached, it remains so). The same is true if a nontoken creature with protection from artifacts is put into your graveyard from the battlefield, for example."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mantle = attach_new(&mut t, P0, "Nim Deathmantle", bears);
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    destroy(&mut t, chosen);
    t.resolve_all();
    let chosen = t.g.current(chosen);
    assert_eq!(t.zone(chosen), Zone::Battlefield);
    assert!(on(&t, mantle, bears));
}

// ---------------------------------------------------------------------------------------
// Scythe of the Wretched: "Whenever a creature dealt damage by equipped creature this
// turn dies, return that card to the battlefield under your control. Attach this
// Equipment to that creature."
// ---------------------------------------------------------------------------------------

#[test]
fn scythe_of_the_wretched_needs_to_equip_only_when_the_creature_dies() {
    cr!("603.2", "603.10a");
    ruling!(
        "Scythe of the Wretched",
        "Scythe of the Wretched needs to equip your creature when the other creature goes to the graveyard for the ability to trigger, but it doesn't matter whether the Scythe equipped your creature when the damage was actually dealt."
    );
    ruling!(
        "Scythe of the Wretched",
        "The Scythe's ability triggers even if something other than the damage dealt by the equipped creature causes the creature to be put into a graveyard that turn."
    );
    supported("Scythe of the Wretched");
    for attached_at_death in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let scythe = t.battlefield(P0, "Scythe of the Wretched");
        let wurm = t.battlefield(P1, "Craw Wurm");
        // The Bears deals 1 damage while the Scythe isn't attached.
        crate::r_p210_common::deal(&mut t, bears, 1, wurm);
        if attached_at_death {
            move_eq(&mut t, scythe, bears);
        }
        // The Wurm is destroyed by something else.
        destroy(&mut t, wurm);
        t.resolve_all();
        let w = t.g.current(wurm);
        if attached_at_death {
            assert_eq!(t.zone(w), Zone::Battlefield);
            assert_eq!(t.obj_now(w).controller, P0);
            assert!(on(&t, scythe, w));
        } else {
            assert_eq!(t.zone(w), Zone::Graveyard(P1));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Glamdring and Spy Kit
// ---------------------------------------------------------------------------------------

#[test]
fn glamdring_reduction_is_locked_in_while_casting() {
    cr!("601.2f", "601.2h", "601.2i");
    ruling!(
        "Glamdring, Foe-hammer // Gleam of Death",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to change how much Glamdring will reduce costs by lowering the power of the equipped creature."
    );
    supported("Glamdring, Foe-hammer // Gleam of Death");
    // Equipped Hill Giant (power 3): Divination ({2}{U}) costs {U}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Glamdring, Foe-hammer // Gleam of Death", giant);
    t.lands(P0, "Island", 1);
    t.lands(P1, "Mountain", 1);
    t.hand(P1, "Lightning Bolt");
    let div = t.hand(P0, "Divination");
    let from = t.asked().len();
    let spell = t.cast(P0, div).go();
    // No player got priority while it was cast and paid for.
    assert_eq!(
        crate::r_s08_common::priority_asks_of(&t, P1, from),
        0,
        "{:?}",
        &t.asked()[from..]
    );
    assert_eq!(t.zone(spell), Zone::Stack);
    // Lowering the power now changes nothing: the spell has been paid for.
    boost(&mut t, giant, -3, 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn spy_kit_names_work_with_a_hidden_agenda() {
    cr!("702.106a", "201.2");
    ruling!(
        "Spy Kit",
        "If you named any nonlegendary creature card for hidden agenda or draft abilities that refer to a creature on the battlefield, the equipped creature will have that name and will qualify for any relevant bonuses."
    );
    supported("Spy Kit");
    // Secrets of Paradise, naming Llanowar Elves: "Creatures you control with the chosen
    // name have '{T}: Add one mana of any color.'"
    let mut t = TestGame::new(2);
    let c = t.custom(P0, (*card::card("Secrets of Paradise")).clone(), Zone::Command);
    t.answer(P0, DecisionKind::Name, Answer::Text("Llanowar Elves".into()));
    mtg_engine::kw::hidden_agenda::as_put_into_command_zone(&mut t.g, P0, c);
    t.g.turn.priority = Some(P0);
    let up = mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
        obj: c,
    });
    t.g.take_action(P0, up);
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let any_color = |t: &TestGame| {
        t.obj_now(bears)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains("one mana of any color"))
    };
    assert!(!any_color(&t));
    attach_new(&mut t, P0, "Spy Kit", bears);
    t.g.recompute();
    assert!(any_color(&t));
}
