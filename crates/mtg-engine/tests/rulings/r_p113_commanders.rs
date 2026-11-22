//! Rulings batch P113 — partner and companion commanders (Commander Legends, Doctor Who,
//! Baldur's Gate, ...): two commanders, abilities that copy abilities, and miscellaneous
//! abilities of those commanders; also Monitor Monitor and Research.

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, place, supported, tapped_lands};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s13_common::{commander, commander_game, pregame};
use crate::r_s26_common::modify_until_eot;
use crate::r_s35_common::{die_rolls, load_dice};
use crate::r_s05_common::run_from;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::dice::DieRoll;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;
use std::sync::Arc;

/// Puts the real card `name` onto the battlefield as `p`'s commander.
fn commander_on_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    t.g.dirty = true;
    t.settle();
    id
}

#[test]
fn controlling_one_of_two_commanders_is_controlling_your_commander() {
    cr!("903.3", "702.124a");
    ruling!(
        "Akroma, Vision of Ixidor",
        "An effect that checks whether you control your commander is satisfied if you control one or both of your two commanders."
    );
    // Thunderfoot Baloth: "Lieutenant — As long as you control your commander, Thunderfoot
    // Baloth gets +2/+2 and other creatures you control get +2/+2 and have trample."
    supported("Thunderfoot Baloth");
    let mut t = commander_game();
    let baloth = t.battlefield(P0, "Thunderfoot Baloth");
    commander(&mut t, P0, "Akroma, Vision of Ixidor");
    t.g.recompute();
    assert_eq!(t.pt(baloth), (5, 5), "no commander on the battlefield");
    let tevesh = commander_on_battlefield(&mut t, P0, "Tevesh Szat, Doom of Fools");
    assert_eq!(t.pt(baloth), (7, 7), "one of the two");
    let _ = tevesh;
}

#[test]
fn akroma_counts_each_keyword_once() {
    cr!("702.1", "702.16", "613.4c");
    ruling!(
        "Akroma, Vision of Ixidor",
        "A creature with more than one instances of one of the listed keywords gets just +1/+1 for that keyword. Similarly, if a creature has multiple variants of the same keyword (e.g. protection from black and from red), it gets just +1/+1 for that keyword."
    );
    supported("Akroma, Vision of Ixidor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akroma, Vision of Ixidor");
    // Paladin en-Vec: 2/2 first strike, protection from black and from red.
    let paladin = t.battlefield(P0, "Paladin en-Vec");
    modify_until_eot(
        &mut t,
        paladin,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::FirstStrike))],
    );
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(paladin), (4, 4), "first strike and protection: +2/+2");
}

#[test]
fn adric_copies_any_activated_or_triggered_ability() {
    cr!("707.10", "113.1c");
    ruling!(
        "Adric, Mathematical Genius",
        "Adric, Mathematical Genius can copy any activated or triggered ability on the stack, not just one with targets."
    );
    ruling!(
        "Adric, Mathematical Genius",
        "Adric, Mathematical Genius's first ability targets an activated or triggered ability that is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    supported("Adric, Mathematical Genius");
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let warden = t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Island", 3);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trigger = t.g.stack[0];
    let abilities = t.obj(warden).chars.abilities.len();
    t.activate(P0, adric, 0, &[ent(trigger)]).expect("Adric");
    t.resolve();
    assert_eq!(t.stack_len(), 2, "a copy of the trigger");
    assert_eq!(t.obj(warden).chars.abilities.len(), abilities);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn ich_tekik_sees_artifact_tokens_die() {
    cr!("111.7", "700.4", "603.6c");
    ruling!(
        "Ich-Tekik, Salvage Splicer",
        "Artifact tokens that are destroyed or otherwise die are put into the graveyard before ceasing to exist. They will cause Ich-Tekik's middle ability to trigger."
    );
    supported("Ich-Tekik, Salvage Splicer");
    let mut t = TestGame::new(2);
    let ich = t.battlefield(P0, "Ich-Tekik, Salvage Splicer");
    let treasure = create_token(&mut t, P0, "Treasure");
    destroy(&mut t, treasure);
    t.resolve_all();
    assert_eq!(t.counters(ich, counters::PLUS1), 1);
}

#[test]
fn the_tenth_doctor_leaves_land_cards_in_exile() {
    cr!("406.3", "702.62a");
    ruling!(
        "The Tenth Doctor",
        "Any land cards exiled with The Tenth Doctor's first ability will remain in exile."
    );
    supported("The Tenth Doctor");
    let mut t = TestGame::new(2);
    let doctor = t.battlefield(P0, "The Tenth Doctor");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    attack_with(&mut t, &[(doctor, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Forest"));
    assert!(t.in_exile("Island"));
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears");
    assert_eq!(t.counters(bears[0], counters::TIME), 3);
    // Still in exile at the end of the turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Forest"));
    assert!(t.in_exile("Island"));
}

#[test]
fn tevesh_szat_gains_control_before_the_command_zone_commanders_enter() {
    cr!("903.3", "608.2c");
    ruling!(
        "Tevesh Szat, Doom of Fools",
        "As the third ability is resolving, you'll gain control of all commanders that were already on the battlefield before the commanders from the command zone enter the battlefield under your control."
    );
    supported("Tevesh Szat, Doom of Fools");
    // P1's commander on the battlefield, Impassioned Orator: "Whenever another creature you
    // control enters, you gain 1 life." It's P0's when P1's other commander enters.
    let mut t = commander_game();
    let tevesh = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    t.g.objects[tevesh.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 10);
    let orator = commander_on_battlefield(&mut t, P1, "Impassioned Orator");
    commander(&mut t, P1, "Grizzly Bears");
    t.activate(P0, tevesh, 2, &[]).expect("-10");
    t.resolve_all();
    assert_eq!(t.obj_now(orator).controller, P0);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj(bears[0]).controller, P0);
    // (Commander: 40 starting life.)
    assert_eq!(t.life(P0), 41, "the Orator saw the Bears enter under P0's control");
    assert_eq!(t.life(P1), 40);
}

#[test]
fn rocksteady_both_commanders_start_in_the_command_zone() {
    cr!("903.6", "702.124b");
    ruling!(
        "Rocksteady, Mutant Marauder",
        "Both commanders start in the command zone, and the remaining 98 cards your deck are shuffled to become your library."
    );
    let pair = ["Rocksteady, Mutant Marauder", "Bebop, Skull & Crossbones"];
    let mut lib: Vec<Arc<mtg_engine::card::CardDef>> = pair.iter().map(|c| card(c)).collect();
    lib.extend((0..98).map(|_| card("Forest")));
    lib.rotate_left(1);
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![lib, (0..100).map(|_| card("Island")).collect()],
    );
    for c in pair {
        assert!(t.g.designate_commander(P0, c));
    }
    t.g.start();
    for c in pair {
        let ids = t.g.find_in_zone(Zone::Command, c);
        assert_eq!(ids.len(), 1, "{c}");
        assert!(t.g.obj(ids[0]).is_commander);
    }
    assert_eq!(t.library_size(P0) + t.hand_size(P0), 98);
}

/// P0 rolls a six-sided die (as a resolving effect would).
fn roll_d6(t: &mut TestGame) {
    run_from(t, P0, None, Effect::RollDice(Box::new(DieRoll::new(6))), &[]);
}

#[test]
fn several_monitor_monitors_on_the_same_roll() {
    cr!("706.2a", "706.2b");
    ruling!(
        "Monitor Monitor",
        "If you control multiple Monitor Monitors, you may use each of them once each turn. You may use multiple Monitor Monitors on the same die roll if you wish."
    );
    ruling!(
        "Monitor Monitor",
        "You decide whether to use Monitor Monitor’s ability right after you monitor the result of the die roll. You can’t wait and use it to reroll that die later."
    );
    supported("Monitor Monitor");
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P0, "Monitor Monitor");
    t.battlefield(P0, "Monitor Monitor");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    load_dice(&mut t, &[1, 2, 6]);
    roll_d6(&mut t);
    let r = die_rolls(&t);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].3, 6, "rerolled twice");
    assert_eq!(tapped_lands(&t, P0), 2);
    // Declining: the result stands, and nothing lets P0 reroll it later.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Monitor Monitor");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, false);
    load_dice(&mut t, &[1, 6]);
    roll_d6(&mut t);
    assert_eq!(die_rolls(&t)[0].3, 1);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, m));
    let _ = m1;
}

#[test]
fn research_shuffles_cards_from_outside_the_game_but_not_from_exile() {
    cr!("400.11", "400.11a");
    ruling!(
        "Research // Development",
        "A card \"from outside the game\" may be a card from your collection or a card from your sideboard."
    );
    ruling!(
        "Research // Development",
        "The exile zone is a part of the game, so you can't get exiled cards."
    );
    let mut t = TestGame::new(2);
    let side = place(&mut t, P0, "Grizzly Bears", Zone::Outside(P0));
    let exiled = t.exile(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Research // Development");
    let from = t.asked().len();
    t.answer_choose(P0, &[ent(side)]);
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Half(0))
        .go();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(offered.iter().any(|o| o.contains(&ent(side))));
    assert!(offered.iter().all(|o| !o.contains(&ent(exiled))));
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Grizzly Bears").len(), 1);
    assert!(t.in_exile("Hill Giant"));
}

/// A Planechase game with Goldmeadow ("Whenever chaos ensues, create a 0/1 white Goat
/// creature token.") face up; P0 has Monitor Monitor and a Forest, answers its reroll
/// with `reroll`, and rolls the planar die with the given natural results loaded.
/// Returns the number of Goats P0 has.
fn planar_roll_with_monitor(reroll: bool, naturals: &[u32]) -> usize {
    use mtg_engine::decision::{Action, SpecialAction};
    use mtg_engine::planechase;
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
    );
    for n in ["Goldmeadow", "The Fourth Sphere"] {
        let id = t.command(P0, n);
        t.g.objects[id.0 as usize].face_down = true;
    }
    t.g.recompute();
    planechase::set_starting_plane(&mut t.g);
    t.battlefield(P0, "Monitor Monitor");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, reroll);
    load_dice(&mut t, naturals);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::Other {
            name: planechase::PLANAR_DIE_ACTION.into(),
            obj: None,
        }),
    )
    .unwrap();
    t.resolve_all();
    crate::r_s05_common::tokens_with_subtype(&t, P0, "Goat").len()
}

#[test]
fn monitor_monitor_can_reroll_the_planar_die() {
    cr!("706.2a", "706.7", "901.9");
    ruling!(
        "Monitor Monitor",
        "You can use Monitor Monitor’s reroll ability on any die roll, including a roll of the planar die in a Planechase game."
    );
    // A blank (natural 3) rerolled into the chaos symbol (natural 2).
    assert_eq!(planar_roll_with_monitor(true, &[3, 2]), 1);
    assert_eq!(planar_roll_with_monitor(false, &[3, 2]), 0);
}
