//! Rulings batch P191 — triggered abilities: what triggers them and what doesn't (CR 603.2,
//! 603.6a), when they resolve relative to the spells that caused them (CR 603.3, 117.3),
//! the order they're put on the stack (CR 603.3b), and what they check as they resolve
//! (CR 608.2b, 608.2h).

use crate::r_p003_common::{kill, put};
use crate::r_p008_common::hit_player;
use crate::r_p015_common::{attacking, move_to};
use crate::r_p076_common::mana;
use crate::r_p191_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, watch, with_subtype};
use crate::r_s05_common::enter;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Tolsimir, Friend to Wolves
// ---------------------------------------------------------------------------------------

#[test]
fn tolsimir_wolf_trigger_comes_as_voja_is_created_and_still_gains_life_without_it() {
    cr!("603.2", "603.6a", "608.2b", "701.14b");
    ruling!(
        "Tolsimir, Friend to Wolves",
        "If Tolsimir is still on the battlefield when its enters-the-battlefield trigger resolves, its last ability will trigger as Voja is created."
    );
    ruling!(
        "Tolsimir, Friend to Wolves",
        "If the Wolf that entered the battlefield is no longer on the battlefield as Tolsimir's last ability resolves, no creature deals or is dealt damage, but you do gain 3 life."
    );
    supported("Tolsimir, Friend to Wolves");
    for voja_leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        enter(&mut t, P0, "Tolsimir, Friend to Wolves");
        assert_eq!(t.stack_len(), 1);
        t.answer_targets(P0, &[obj(bears)]);
        t.resolve();
        // Voja is created and the Wolf trigger is waiting on the stack.
        let voja = t.named_on_battlefield("Voja, Friend to Elves");
        assert_eq!(voja.len(), 1);
        assert_eq!(stacked_triggers(&t, "Wolf").len(), 1);
        if voja_leaves {
            kill(&mut t, voja[0]);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 23, "voja leaves: {voja_leaves}");
        if voja_leaves {
            assert!(t.on_battlefield(bears));
            assert_eq!(t.obj(bears).damage, 0);
        } else {
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert_eq!(t.obj(voja[0]).damage, 2);
        }
    }
    // Tolsimir gone before its enters trigger resolves: Voja is still created, but no
    // Wolf trigger.
    let mut t = TestGame::new(2);
    let tol = enter(&mut t, P0, "Tolsimir, Friend to Wolves");
    kill(&mut t, tol);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Voja, Friend to Elves").len(), 1);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------------
// The Ur-Dragon
// ---------------------------------------------------------------------------------------

#[test]
fn ur_dragon_ignores_dragons_put_onto_the_battlefield_attacking() {
    cr!("508.3a", "508.4", "603.2");
    ruling!(
        "The Ur-Dragon",
        "If a Dragon you control enters the battlefield attacking, it won’t cause The Ur-Dragon’s last ability to trigger."
    );
    supported("The Ur-Dragon");
    let mut t = TestGame::new(2);
    let ur = t.battlefield(P0, "The Ur-Dragon");
    let kaalia = t.battlefield(P0, "Kaalia of the Vast");
    let shivan = t.hand(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(shivan)]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(kaalia, pl(P1))]);
    t.resolve_all();
    let shivan = t.g.current(shivan);
    assert!(t.on_battlefield(shivan));
    assert_eq!(attacking(&t, shivan), Some(pl(P1)));
    assert!(attacking(&t, ur).is_none());
    // No cards drawn: only Kaalia's trigger resolved.
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ur_dragon_puts_the_permanent_after_attackers_are_declared() {
    cr!("508.1", "508.3a", "508.4", "603.2");
    ruling!(
        "The Ur-Dragon",
        "The Ur-Dragon’s triggered ability resolves after all attackers have been chosen. You can’t attack with some Dragons, put a creature card with haste onto the battlefield, and then attack with that creature. Similarly, any “whenever a creature attacks” abilities of the permanent card you put onto the battlefield won’t trigger."
    );
    supported("The Ur-Dragon");
    supported("Anthem of Rakdos");
    for name in ["Raging Goblin", "Anthem of Rakdos"] {
        let mut t = TestGame::new(2);
        let ur = t.battlefield(P0, "The Ur-Dragon");
        let card = t.hand(P0, name);
        t.answer_choose(P0, &[obj(card)]);
        attack_with(&mut t, &[(ur, pl(P1))]);
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        let card = t.g.current(card);
        assert!(t.on_battlefield(card), "{name}");
        assert!(attacking(&t, card).is_none(), "{name}");
        // Anthem of Rakdos didn't see The Ur-Dragon attack.
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(t.pt(ur), (10, 10), "{name}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 10, "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Niv-Mizzet, Parun
// ---------------------------------------------------------------------------------------

#[test]
fn niv_mizzet_triggers_only_on_draws() {
    cr!("121.2", "603.2");
    ruling!(
        "Niv-Mizzet, Parun",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" Niv-Mizzet's first triggered ability won't trigger."
    );
    supported("Niv-Mizzet, Parun");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Parun");
    let bears = t.graveyard(P0, "Grizzly Bears");
    move_to(&mut t, bears, Zone::Hand(P0));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
    // A draw does.
    t.answer_targets(P0, &[pl(P1)]);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn niv_mizzet_triggers_once_per_card_with_targets_chosen_after_the_draws() {
    cr!("121.2", "603.3", "603.3d");
    ruling!(
        "Niv-Mizzet, Parun",
        "If an effect instructs you to draw multiple cards, Niv-Mizzet's first triggered ability triggers that many times. You choose targets for those abilities after you've drawn all of the cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Parun");
    let hand = t.hand_size(P0);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(P0).hand.len(),
    );
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.draw_cards(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    assert_eq!(*seen.lock().unwrap(), vec![hand + 3; 3]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn niv_mizzet_draws_before_the_spell_resolves_and_the_card_can_counter_it() {
    cr!("603.3", "117.3b", "117.3c", "701.6a");
    ruling!(
        "Niv-Mizzet, Parun",
        "Niv-Mizzet's second triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered. This causes its first triggered ability to trigger, and that also resolves before the spell."
    );
    ruling!(
        "Niv-Mizzet, Parun",
        "Players can cast spells and activate abilities after Niv-Mizzet's second triggered ability resolves but before the spell that caused it to trigger does. Notably, the card you draw may be able to counter that spell."
    );
    // P1 casts Shock at P0; P0 draws Cancel off Niv-Mizzet and counters the Shock.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Parun");
    stack_library(&mut t, P0, &["Cancel"]);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    let spell = t.cast(P1, shock).target(pl(P0)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    // The draw trigger resolved; the damage trigger is above the Shock.
    assert!(t.in_hand(P0, "Cancel"));
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(t.g.stack[0], spell);
    assert!(is_trigger(&t, t.g.stack[1]));
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.stack, vec![spell]);
    // P0 casts the Cancel it drew (which triggers Niv-Mizzet again).
    let cancel = t.g.find_in_zone(Zone::Hand(P0), "Cancel")[0];
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, cancel).target(spell).go();
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
    // The spell countered before the draw trigger resolves: P0 still draws (and pings).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, Parun");
    let cancel = t.hand(P0, "Cancel");
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    let spell = t.cast(P1, shock).target(pl(P0)).go();
    t.settle();
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, cancel).target(spell).go();
    for _ in 0..2 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------------
// Rona, Sheoldred's Faithful
// ---------------------------------------------------------------------------------------

#[test]
fn rona_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "117.3b", "701.6a");
    ruling!(
        "Rona, Sheoldred's Faithful",
        "Rona, Sheoldred's Faithful's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Rona, Sheoldred's Faithful");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rona, Sheoldred's Faithful");
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, shock).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let cancel = t.hand(P1, "Cancel");
    mana(&mut t, P1, ManaType::U, 2);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn rona_from_the_graveyard_keeps_creature_timing() {
    cr!("601.3", "307.1", "117.1a");
    ruling!(
        "Rona, Sheoldred's Faithful",
        "The casting permission granted by Rona's last ability doesn't change when you may cast it from your graveyard."
    );
    let mut t = TestGame::new(2);
    let rona = t.graveyard(P0, "Rona, Sheoldred's Faithful");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Island", 1);
    // Main phase, empty stack: castable.
    assert!(!legal_cast_methods(&mut t, P0, rona).is_empty());
    // Not with a spell on the stack.
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, shock).target(pl(P1)).go();
    assert!(legal_cast_methods(&mut t, P0, rona).is_empty());
    t.resolve_all();
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(legal_cast_methods(&mut t, P0, rona).is_empty());
}

// ---------------------------------------------------------------------------------------
// Deathbringer Liege
// ---------------------------------------------------------------------------------------

#[test]
fn deathbringer_liege_white_black_spell_taps_then_destroys() {
    cr!("603.3b", "608.2b", "202.2");
    ruling!(
        "Deathbringer Liege",
        "If you cast a spell that's both white and black, the last two abilities will both trigger. You may use them together to destroy any creature. To do so, put the \"destroy\" ability on the stack first, then put the \"tap\" ability on the stack, with each one targeting the same creature. When the abilities resolve, they'll tap the creature, then destroy it."
    );
    supported("Deathbringer Liege");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deathbringer Liege");
    let giant = t.battlefield(P1, "Hill Giant");
    let drinker = t.hand(P0, "Tithe Drinker");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::B, 1);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(giant)]);
    // First = bottom of the stack: the "destroy" ability (listed second), then "tap".
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.cast(P0, drinker).go();
    t.settle();
    let tap = stacked_triggers(&t, "tap target");
    let destroy = stacked_triggers(&t, "destroy target");
    assert_eq!((tap.len(), destroy.len()), (1, 1));
    // The "destroy" ability was put on the stack first (bottom), "tap" on top.
    let pos = |id: ObjectId| t.g.stack.iter().position(|s| *s == id).unwrap();
    assert!(pos(destroy[0]) < pos(tap[0]));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn deathbringer_liege_destroy_trigger_checks_tapped_on_resolution() {
    cr!("115.1", "608.2b", "608.2c");
    ruling!(
        "Deathbringer Liege",
        "The last ability can target any creature, not just a tapped creature (it doesn't say \"target tapped creature\"). When the ability resolves, check whether the creature is tapped or not. If it's tapped, you may destroy it. If it's not tapped, the ability does nothing."
    );
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Deathbringer Liege");
        let giant = t.battlefield(P1, "Hill Giant");
        let corpse = t.hand(P0, "Walking Corpse");
        mana(&mut t, P0, ManaType::B, 2);
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(giant)]);
        t.cast(P0, corpse).go();
        t.settle();
        assert!(target_choices(&t, from)
            .iter()
            .any(|c| c.contains(&obj(giant))));
        let trig = stacked_triggers(&t, "destroy target");
        assert_eq!(targets_of(&t, trig[0]), vec![obj(giant)]);
        if tapped {
            t.g.tap(giant);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), !tapped);
    }
}

// ---------------------------------------------------------------------------------------
// Anthem of Rakdos, Nicol Bolas, Dread Cacodemon, Pith Driller
// ---------------------------------------------------------------------------------------

#[test]
fn anthem_of_rakdos_triggers_for_each_attacker() {
    cr!("508.1m", "603.2c");
    ruling!(
        "Anthem of Rakdos",
        "The first ability triggers once for each creature you attack with. If you attack with three creatures, for example, they’ll each get +2/+0 and you’ll be dealt 3 damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anthem of Rakdos");
    // A card in hand: no hellbent doubling.
    t.hand(P0, "Grizzly Bears");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Walking Corpse");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))]);
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(b), (5, 3));
    assert_eq!(t.pt(c), (4, 2));
}

#[test]
fn nicol_bolas_triggers_on_noncombat_damage_too() {
    cr!("120.3", "603.2");
    ruling!(
        "Nicol Bolas",
        "Nicol Bolas's third ability triggers whenever it deals damage to an opponent for any reason. It's not limited to combat damage."
    );
    supported("Nicol Bolas");
    let mut t = TestGame::new(2);
    let bolas = t.battlefield(P0, "Nicol Bolas");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    hit_player(&mut t, bolas, P1, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn dread_cacodemon_put_onto_the_battlefield_from_hand_doesnt_trigger() {
    cr!("603.4", "601.2");
    ruling!(
        "Dread Cacodemon",
        "If you put Dread Cacodemon directly onto the battlefield from your hand, perhaps because of Kaalia of the Vast’s ability, its ability won’t trigger."
    );
    supported("Dread Cacodemon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let demon = t.hand(P0, "Dread Cacodemon");
    move_to(&mut t, demon, Zone::Battlefield);
    assert!(t.on_battlefield(demon));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Cast from hand, it does.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let demon = t.hand(P0, "Dread Cacodemon");
    mana(&mut t, P0, ManaType::B, 3);
    mana(&mut t, P0, ManaType::C, 7);
    t.cast(P0, demon).go();
    t.resolve_all();
    assert!(t.on_battlefield(demon));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn pith_driller_targets_itself_when_alone() {
    cr!("603.3d", "115.1");
    ruling!(
        "Pith Driller",
        "Pith Driller's ability is mandatory. If it's the only creature on the battlefield when it enters, Pith Driller's ability will target itself."
    );
    let mut t = TestGame::new(2);
    let driller = enter(&mut t, P0, "Pith Driller");
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, t.g.stack[0]), vec![obj(driller)]);
    t.resolve_all();
    assert_eq!(t.counters(driller, "-1/-1"), 1);
    assert_eq!(t.pt(driller), (1, 3));
}

// ---------------------------------------------------------------------------------------
// Sphinx Sovereign
// ---------------------------------------------------------------------------------------

#[test]
fn sphinx_sovereign_checks_tapped_as_it_resolves_or_as_it_left() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Sphinx Sovereign",
        "The ability checks whether Sphinx Sovereign is tapped or untapped at the time the ability resolves. If Sphinx Sovereign is no longer on the battlefield by then, the ability checks whether Sphinx Sovereign was tapped or untapped at the time it left the battlefield."
    );
    supported("Sphinx Sovereign");
    // (tap it before resolution, then remove it, P0's life, P1's life)
    for (tap, remove, p0, p1) in [
        (false, false, 23, 20),
        (true, false, 20, 17),
        (false, true, 23, 20),
        (true, true, 20, 17),
    ] {
        let mut t = TestGame::new(2);
        let sphinx = t.battlefield(P0, "Sphinx Sovereign");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if tap {
            t.g.tap(sphinx);
        }
        if remove {
            kill(&mut t, sphinx);
        }
        t.resolve_all();
        assert_eq!(
            (t.life(P0), t.life(P1)),
            (p0, p1),
            "tap {tap}, remove {remove}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Devourer of Destiny
// ---------------------------------------------------------------------------------------

#[test]
fn devourer_of_destiny_revealed_many_times_triggers_many_times() {
    cr!("103.6", "103.6a", "603.7");
    ruling!(
        "Devourer of Destiny",
        "If you reveal more than one Devourer of Destiny from your opening hand, you'll put that many triggered abilities on the stack at the beginning of your first upkeep."
    );
    supported("Devourer of Destiny");
    // Every card in both opening hands is a Devourer: each player reveals seven.
    let (mut t, _) = crate::r_p101_common::opening_hands(2, "Devourer of Destiny", P0, &[]);
    t.g.begin_turn(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let mine: Vec<_> =
        t.g.stack
            .iter()
            .filter(|id| t.g.obj(**id).controller == P0)
            .collect();
    assert_eq!(mine.len(), 7);
    assert_eq!(t.stack_len(), 7);
    let lib = t.library_size(P0);
    t.resolve_all();
    // Each one looks at four cards and keeps at most one on top.
    assert!(t.library_size(P0) <= lib - 7 * 3);
}

// ---------------------------------------------------------------------------------------
// Dawnglow Infusion, Doomgape, Creakwood Liege, Indominus Rex
// ---------------------------------------------------------------------------------------

#[test]
fn dawnglow_infusion_both_colors_is_one_life_gain_event() {
    cr!("119.9", "603.2");
    ruling!(
        "Dawnglow Infusion",
        "If you spent both {G} and {W} to cast Dawnglow Infusion, you’ll gain (X + X) life. An ability that triggers whenever you gain life will trigger just once."
    );
    supported("Dawnglow Infusion");
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let c = t.hand(P0, "Dawnglow Infusion");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, c).x(3).go();
    t.resolve();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn doomgape_alone_sacrifices_itself_and_gains_its_toughness() {
    cr!("701.21a", "608.2h");
    ruling!(
        "Doomgape",
        "If you have no other creatures to sacrifice, you’ll have to sacrifice Doomgape itself. You’ll gain life equal to its toughness."
    );
    let mut t = TestGame::new(2);
    let gape = t.battlefield(P0, "Doomgape");
    to_upkeep(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(gape));
    assert_eq!(t.life(P0), 30);
}

#[test]
fn creakwood_liege_worm_gets_both_bonuses() {
    cr!("613.4c", "105.2");
    ruling!(
        "Creakwood Liege",
        "Since the token is black and green, it will get +2/+2 from Creakwood Liege as long as the Liege is on the battlefield."
    );
    supported("Creakwood Liege");
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Creakwood Liege");
    to_upkeep(&mut t);
    t.answer_yes(P0, true);
    t.resolve_all();
    let worms = with_subtype(&t, P0, "Worm");
    assert_eq!(worms.len(), 1);
    assert_eq!(t.pt(worms[0]), (3, 3));
    kill(&mut t, liege);
    assert_eq!(t.pt(worms[0]), (1, 1));
}

#[test]
fn indominus_rex_counts_counters_put_on_it_before_the_draw_trigger_resolves() {
    cr!("608.2h", "614.1c");
    ruling!(
        "Indominus Rex, Alpha",
        "If you put counters on Indominus Rex before its last ability resolves, those counters will count."
    );
    supported("Indominus Rex, Alpha");
    let mut t = TestGame::new(2);
    let angel = t.hand(P0, "Serra Angel");
    t.answer_choose(P0, &[obj(angel)]);
    let rex = enter(&mut t, P0, "Indominus Rex, Alpha");
    assert_eq!(t.counters(rex, "flying"), 1);
    assert_eq!(t.counters(rex, "vigilance"), 1);
    assert_eq!(t.stack_len(), 1);
    put(&mut t, rex, "+1/+1", 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}
