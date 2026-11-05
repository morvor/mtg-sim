//! Rulings batch P008 — planeswalkers that burn: Chandra, Pyromaster's exile and free
//! copies (CR 107.3, 601.2, 305.2), Sarkhan the Mad's self-damage (CR 608.2), Nicol Bolas's
//! control change with no duration (CR 611.2), loyalty and indestructible (CR 704.5i),
//! replacement effects on the counters a planeswalker enters with (CR 122.6, 616.1), Ral,
//! Storm Conduit's copies (CR 707.10, 707.12), and Goblin Bomb's fuse counters (CR 122.8).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_p113_common::cast_modes;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_cast, can_play_land, destroy};
use crate::r_s06_common::give_control;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::spell_copies;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn chandra_pyromaster_copies_of_an_x_spell_have_x_zero() {
    cr!("107.3b", "118.9c", "707.10");
    ruling!(
        "Chandra, Pyromaster",
        "If the copied card has {X} in its mana cost, you must choose 0 as its value when casting the copies."
    );
    supported("Chandra, Pyromaster");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 7);
    let fireball = t.library_top(P0, "Fireball");
    t.answer_choose_moved(P0, &[obj(fireball)]);
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[obj(wurm)]);
    }
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0, "X = 0 for each copy");
    assert!(t.on_battlefield(wurm));
}

#[test]
fn chandra_pyromaster_chooses_only_among_the_cards_just_exiled() {
    cr!("406.6", "607.1");
    ruling!(
        "Chandra, Pyromaster",
        "In the third ability, \"exiled this way\" means the cards you just exiled because of that ability resolving. Cards exiled with the second ability or with previous activations of the third ability don't count."
    );
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 8);
    // The second ability exiles Lightning Bolt.
    let old = t.library_top(P0, "Lightning Bolt");
    loyalty(&mut t, P0, c, 1, &[]);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    let old = t.g.current(old);
    // The top ten now hold one Shock.
    for _ in 0..9 {
        t.library_top(P0, "Grizzly Bears");
    }
    let shock = t.library_top(P0, "Shock");
    t.g.objects[c.0 as usize].activations_this_turn.clear();
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[obj(old)]);
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.contains(&obj(old)), "the Bolt from the 0 ability");
    assert!(offered.contains(&obj(t.g.current(shock))) || t.in_graveyard(P0, "Shock"));
    assert_eq!(t.life(P1), 20, "the Bolt wasn't copied");
}

#[test]
fn chandra_pyromaster_exiled_card_follows_the_normal_timing_rules() {
    cr!("305.2", "305.3", "307.1", "601.3");
    ruling!(
        "Chandra, Pyromaster",
        "If you exile a land card using the second ability, you may play that land only if you have any available land plays. Normally, this means you can play the land only if you haven't played a land yet that turn."
    );
    ruling!(
        "Chandra, Pyromaster",
        "The card exiled by the second ability is exiled face up. Playing it follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if it's a creature card, you can cast it only during your main phase while the stack is empty."
    );
    // A land: playable only with a land play left.
    for played in [false, true] {
        let mut t = TestGame::new(2);
        let c = pw(&mut t, P0, "Chandra, Pyromaster", 4);
        let forest = t.library_top(P0, "Forest");
        loyalty(&mut t, P0, c, 1, &[]);
        t.resolve_all();
        let forest = t.g.current(forest);
        assert_eq!(t.zone(forest), Zone::Exile);
        assert!(!t.obj_now(forest).face_down, "face up");
        if played {
            t.g.players[0].lands_played_this_turn = 1;
        }
        assert_eq!(can_play_land(&mut t, P0, forest), !played);
    }
    // A creature card: only in a main phase with an empty stack, paying its cost.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 4);
    let bears = t.library_top(P0, "Grizzly Bears");
    loyalty(&mut t, P0, c, 1, &[]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(!can_cast(&mut t, P0, bears, NORMAL), "no mana");
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, bears, NORMAL));
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[pl(P1)]);
    assert!(!can_cast(&mut t, P0, bears, NORMAL), "the stack isn't empty");
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, bears, NORMAL), "not a main phase");
}

#[test]
fn sarkhan_the_mad_damages_himself_as_the_ability_resolves() {
    cr!("608.2", "306.8");
    ruling!(
        "Sarkhan the Mad",
        "Once the first ability starts to resolve and you reveal the top card of your library (and see how much damage Sarkhan the Mad deals to himself), it's too late to respond."
    );
    supported("Sarkhan the Mad");
    let mut t = TestGame::new(2);
    let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
    t.library_top(P0, "Craw Wurm");
    loyalty(&mut t, P0, s, 0, &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0, "nothing in between");
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.counters(s, "loyalty"), 1, "dealt 6 damage (Craw Wurm's mana value)");
}

#[test]
fn nicol_bolas_control_change_has_no_duration() {
    cr!("611.2b", "613.1b", "613.7");
    ruling!(
        "Nicol Bolas, Planeswalker",
        "The control-change effect has no duration. It will last until the game ends or the affected creature leaves the battlefield. It may be superseded by a later control-change effect."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let b = pw(&mut t, P0, "Nicol Bolas, Planeswalker", 5);
    loyalty(&mut t, P0, b, 1, &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).controller, P0);
    destroy(&mut t, b);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(wurm).controller, P0, "after Bolas left and turns passed");
    give_control(&mut t, wurm, P1);
    assert_eq!(t.obj_now(wurm).controller, P1, "a later control change");
}

#[test]
fn indestructible_planeswalkers_still_lose_loyalty_and_die_at_zero() {
    cr!("120.3c", "704.5i", "702.12b");
    ruling!(
        "Boros Charm",
        "Planeswalkers with indestructible will still have loyalty counters removed from them as they are dealt damage. If a planeswalker with indestructible has no loyalty counters, it will still be put into its owner's graveyard, as the rule that does this doesn't destroy the planeswalker."
    );
    let mut t = TestGame::new(2);
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    t.set_step(P1, Step::PrecombatMain);
    cast_modes(&mut t, P1, "Boros Charm", &[1], &[]);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(t.counters(theirs, "loyalty"), 2);
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[obj(theirs)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Chandra, Torch of Defiance"));
}

#[test]
fn lukka_enters_with_loyalty_modified_by_other_replacement_effects() {
    cr!("122.6", "614.1c", "702.150a");
    ruling!(
        "Lukka, Bound to Ruin",
        "Other replacement effects that would change the number of loyalty counters Lukka enters with will apply as normal."
    );
    supported("Lukka, Bound to Ruin");
    // Vorinclex, Monstrous Raider doubles the 5 loyalty counters Lukka enters with.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let l = cast_paying(
        &mut t,
        P0,
        "Lukka, Bound to Ruin",
        &["Wastes", "Wastes", "Mountain", "Mountain", "Forest"],
        NORMAL,
        &[],
    );
    t.resolve_all();
    let l = t.g.current(l);
    assert!(t.on_battlefield(l));
    assert_eq!(t.counters(l, "loyalty"), 10);
}

#[test]
fn ral_storm_conduit_copies_keep_modes_and_arent_cast() {
    cr!("707.10", "707.10c", "700.2g");
    ruling!(
        "Ral, Storm Conduit",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy created by Ral's last ability will have the same mode or modes. You can't choose different ones."
    );
    ruling!(
        "Ral, Storm Conduit",
        "The copies that Ral's last ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Ral, Storm Conduit");
    let mut t = TestGame::new(2);
    let r = pw(&mut t, P0, "Ral, Storm Conduit", 4);
    let prowess = t.battlefield(P0, "Heartfire Immolator");
    loyalty(&mut t, P0, r, 1, &[]);
    t.resolve_all();
    // Ral's static triggers for the cast and for the copy (1 each, targeting P1).
    for _ in 0..2 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    t.answer_yes(P0, false);
    let s = cast_modes(&mut t, P0, "Boros Charm", &[0], &[pl(P1)]);
    t.settle();
    // Resolve the triggers above the spell until Ral's delayed trigger has copied it.
    while spell_copies(&t).is_empty() && t.stack_len() > 1 {
        t.resolve();
    }
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(chosen_modes(&t, copies[0]), chosen_modes(&t, s));
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 4 - 1 - 1);
    assert_eq!(t.pt(prowess), (3, 3), "prowess triggered once (the cast)");
}

#[test]
fn a_copy_spell_cant_target_itself() {
    cr!("115.5", "601.2c");
    ruling!(
        "Ral, Storm Conduit",
        "Some spells instruct you to \"copy target instant or sorcery spell.\" Because a spell is never a legal target for itself, you can't cast such a spell targeting itself to create a loop that causes Ral's first ability to trigger over and over."
    );
    let mut t = TestGame::new(2);
    let r = pw(&mut t, P0, "Ral, Storm Conduit", 4);
    loyalty(&mut t, P0, r, 1, &[]);
    t.resolve_all();
    lands_for(&mut t, "Twincast");
    let twincast = t.hand(P0, "Twincast");
    t.answer_targets(P0, &[obj(twincast)]);
    assert!(t.cast(P0, twincast).try_go().is_err());
    assert_eq!(t.life(P1), 20);
}

fn lands_for(t: &mut TestGame, name: &str) {
    crate::r_s25_common::lands_for_cost(t, P0, name);
}

#[test]
fn goblin_bomb_losing_a_flip_with_no_fuse_counters_does_nothing() {
    cr!("122.8", "705.2");
    ruling!(
        "Goblin Bomb",
        "Nothing happens if you are instructed to remove a fuse counter from an empty card."
    );
    supported("Goblin Bomb");
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Goblin Bomb");
    t.g.dice.loaded_coins.push_back(false);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(bomb));
    assert_eq!(t.counters(bomb, "fuse"), 0);
    // Then a win puts one on.
    t.advance_to(P1, Step::Upkeep);
    t.g.dice.loaded_coins.push_back(true);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bomb, "fuse"), 1);
}
