//! Rulings batch P052 — sacrificial artifacts and how their abilities resolve: illegal
//! targets (CR 608.2b), coin flips and a sacrifice the controller can't perform
//! (CR 705, 701.21a), revealing (CR 701.20b), searching without revealing (CR 701.23),
//! controlling a player's next turn (CR 723.1b), a counterspell that checks the mana spent
//! (CR 601.2h, 603.4), and Foods that aren't tokens (CR 205.3g).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s35_common::load_dice;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Teferi's Protection (protection from everything; P1's permanents phase out).
fn teferis_protection(t: &mut TestGame) {
    let tp = t.hand(P1, "Teferi's Protection");
    fill_pool(t, P1, 3);
    t.cast(P1, tp).go();
    t.g.settle();
    t.g.resolve_top();
}

#[test]
fn stone_of_erech_draws_nothing_if_its_target_became_illegal() {
    cr!("608.2b");
    ruling!(
        "Stone of Erech",
        "If the target is not legal as Stone of Erech's activated ability tries to resolve, the ability is removed from the stack. You won't draw a card."
    );
    supported("Stone of Erech");
    supported("Teferi's Protection");
    for protected in [false, true] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P0, "Stone of Erech");
        t.graveyard(P1, "Hill Giant");
        fill_pool(&mut t, P0, 2);
        t.activate(P0, stone, 0, &[pl(P1)]).expect("activate");
        if protected {
            teferis_protection(&mut t);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + !protected as usize);
        assert_eq!(t.in_graveyard(P1, "Hill Giant"), protected);
    }
}

#[test]
fn a_skullbomb_draws_nothing_if_its_creature_left() {
    cr!("608.2b");
    ruling!(
        "Basilica Skullbomb",
        "If the target of Basilica Skullbomb's last ability isn't a legal target as the ability tries to resolve (probably because it has left the battlefield), you won't draw a card."
    );
    ruling!(
        "Furnace Skullbomb",
        "If the target of Furnace Skullbomb's last ability isn't a legal target as the ability tries to resolve (probably because it has left the battlefield), you won't draw a card."
    );
    ruling!(
        "Maze Skullbomb",
        "If the target of Maze Skullbomb's last ability isn't a legal target as the ability tries to resolve (probably because it has left the battlefield), you won't draw a card."
    );
    for name in ["Basilica Skullbomb", "Furnace Skullbomb", "Maze Skullbomb"] {
        supported(name);
        for removed in [false, true] {
            let mut t = TestGame::new(2);
            let bomb = t.battlefield(P0, name);
            let bears = t.battlefield(P0, "Grizzly Bears");
            fill_pool(&mut t, P0, 3);
            t.activate(P0, bomb, 1, &[obj(bears)]).expect(name);
            if removed {
                t.g.destroy(bears, None);
            }
            let hand = t.hand_size(P0);
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + !removed as usize, "{name}");
            assert!(t.in_graveyard(P0, name));
        }
    }
}

#[test]
fn dragonspark_reactor_damages_whichever_targets_are_still_legal() {
    cr!("608.2b", "115.1");
    supported("Dragonspark Reactor");
    ruling!(
        "Dragonspark Reactor",
        "If you target both a player and a creature as you activate the ability, and one of the targets is illegal as the ability resolves, Dragonspark Reactor will deal damage to the remaining legal target. If both targets are illegal, the ability won't resolve."
    );
    // (creature leaves, P1 protected, creature's controller)
    for (gone, protected, mine) in [
        (false, false, false),
        (true, false, false),
        (false, true, true),
        (false, true, false),
    ] {
        let mut t = TestGame::new(2);
        let reactor = t.battlefield(P0, "Dragonspark Reactor");
        t.g.objects[reactor.0 as usize]
            .counters
            .insert("charge".into(), 3);
        let wurm = t.battlefield(if mine { P0 } else { P1 }, "Craw Wurm");
        fill_pool(&mut t, P0, 4);
        t.activate(P0, reactor, 0, &[pl(P1), obj(wurm)])
            .expect("activate");
        if gone {
            t.g.destroy(wurm, None);
        }
        if protected {
            teferis_protection(&mut t);
        }
        t.resolve_all();
        let wurm_damage = if t.on_battlefield(wurm) {
            t.obj_now(wurm).damage
        } else {
            0
        };
        match (gone, protected, mine) {
            // Both legal: "that much" is the number of charge counters (it was 0 when
            // read as the amount of an event).
            (false, false, _) => {
                assert_eq!(t.life(P1), 17);
                assert_eq!(wurm_damage, 3);
            }
            (true, _, _) => assert_eq!(t.life(P1), 17),
            (_, true, true) => {
                assert_eq!(t.life(P1), 20);
                assert_eq!(wurm_damage, 3);
            }
            _ => {
                // Both illegal (P1's Wurm phased out with P1's protection).
                assert_eq!(t.life(P1), 20);
                assert_eq!(wurm_damage, 0);
            }
        }
    }
    ruling!(
        "Dragonspark Reactor",
        "Targeting a creature is optional, but you must choose at least one target player to activate Dragonspark Reactor's last ability."
    );
    let mut t = TestGame::new(2);
    let reactor = t.battlefield(P0, "Dragonspark Reactor");
    t.g.objects[reactor.0 as usize]
        .counters
        .insert("charge".into(), 2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_pool(&mut t, P0, 4);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[]);
    t.activate(P0, reactor, 0, &[]).expect("no creature target");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn sorcerers_strongbox_draws_on_a_won_flip_even_without_the_box() {
    cr!("705.2", "701.21a", "608.2c");
    supported("Sorcerer's Strongbox");
    ruling!("Sorcerer's Strongbox", "If you lose the flip, nothing happens.");
    let mut t = TestGame::new(2);
    let bx = t.battlefield(P0, "Sorcerer's Strongbox");
    crate::r_p017_common::load_coins(&mut t, &[false]);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, bx, 0, &[]);
    assert!(t.on_battlefield(bx));
    assert_eq!(t.hand_size(P0), hand);
    ruling!(
        "Sorcerer's Strongbox",
        "If you no longer control Sorcerer’s Strongbox by the time you win the flip, you won’t be able to sacrifice it. You’ll still draw three cards, though."
    );
    let mut t = TestGame::new(2);
    let bx = t.battlefield(P0, "Sorcerer's Strongbox");
    crate::r_p017_common::load_coins(&mut t, &[true]);
    fill_pool(&mut t, P0, 2);
    t.activate(P0, bx, 0, &[]).expect("activate");
    t.g.destroy(bx, None);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.in_graveyard(P0, "Sorcerer's Strongbox"));
    assert!(t.g.history.sacrificed.is_empty());
}

#[test]
fn diviners_lockbox_draws_the_named_top_card_or_leaves_it() {
    cr!("701.20b", "608.2c");
    supported("Diviner's Lockbox");
    ruling!(
        "Diviner's Lockbox",
        "If the top card of your library does have the chosen name, it will be among the three cards you draw."
    );
    ruling!(
        "Diviner's Lockbox",
        "If the top card of your library doesn’t have the chosen name, it remains on top of your library."
    );
    for named in ["Hill Giant", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        let lockbox = t.battlefield(P0, "Diviner's Lockbox");
        let giant = t.library_top(P0, "Hill Giant");
        t.answer(P0, DecisionKind::Name, Answer::Text(named.into()));
        let hand = t.hand_size(P0);
        activate_resolve(&mut t, P0, lockbox, 0, &[]);
        if named == "Hill Giant" {
            assert_eq!(t.zone(giant), Zone::Hand(P0));
            assert_eq!(t.hand_size(P0), hand + 3);
            assert!(!t.on_battlefield(lockbox));
        } else {
            assert_eq!(t.g.player(P0).library.last(), Some(&giant));
            assert_eq!(t.hand_size(P0), hand);
            assert!(t.on_battlefield(lockbox));
        }
    }
}

#[test]
fn cranial_archive_shuffles_an_empty_graveyard_and_draws() {
    cr!("701.24a", "608.2c");
    ruling!(
        "Cranial Archive",
        "If the player’s graveyard is empty, the player will shuffle their library, then you’ll draw a card."
    );
    supported("Cranial Archive");
    let mut t = TestGame::new(2);
    let archive = t.battlefield(P0, "Cranial Archive");
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, archive, 0, &[pl(P1)]);
    assert!(t
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == P1)));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(archive), Zone::Exile);
}

#[test]
fn treasure_chest_puts_the_found_card_into_hand_unrevealed() {
    cr!("701.23a", "701.20a", "706.2");
    ruling!(
        "Treasure Chest",
        "If you choose to put the card into your hand, you don't need to reveal it."
    );
    supported("Treasure Chest");
    for (name, artifact) in [("Ornithopter", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        let chest = t.battlefield(P0, "Treasure Chest");
        let card = t.library_top(P0, name);
        load_dice(&mut t, &[20]);
        t.answer_choose(P0, &[obj(card)]);
        if artifact {
            t.answer_yes(P0, false);
        }
        activate_resolve(&mut t, P0, chest, 0, &[]);
        assert_eq!(t.zone(card), Zone::Hand(P0), "{name}");
        assert!(!t.turn_events.iter().any(|e| matches!(e,
            Event::Custom { name, .. } if name.as_str() == mtg_engine::reveal::REVEALED)));
    }
}

#[test]
fn the_dominion_bracelet_controls_the_next_turn_actually_taken() {
    cr!("723.1b", "500.11");
    ruling!(
        "The Dominion Bracelet",
        "If the targeted player skips their next turn, you’ll control the next turn the affected player actually takes."
    );
    supported("Meditate");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bracelet = t.battlefield(P0, "The Dominion Bracelet");
    t.g.attach(bracelet, obj(wurm));
    fill_pool(&mut t, P0, 8);
    t.activate(P0, wurm, 0, &[pl(P1)]).expect("activate");
    t.resolve_all();
    // P1 skips their next turn.
    let meditate = t.hand(P1, "Meditate");
    fill_pool(&mut t, P1, 3);
    t.g.turn.priority = Some(P1);
    t.cast(P1, meditate).go();
    t.resolve_all();
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::Upkeep);
    // P1's turn was skipped: this is P0's next turn.
    assert_eq!(t.g.turn.number, turn + 1);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P1), P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P1), P0);
}

#[test]
fn vexing_bauble_counters_only_spells_no_mana_was_spent_on() {
    cr!("601.2h", "603.4", "118.9");
    supported("Vexing Bauble");
    ruling!(
        "Vexing Bauble",
        "If a spell is cast with an alternative cost that includes mana (such as flashback), Vexing Bauble's first ability won't trigger and won't counter that spell. Similarly, if a spell is cast without paying its mana cost, but the player did pay mana for additional costs such as from kicker or from Thorn of Amethyst, Vexing Bauble's first ability won't trigger and won't counter that spell."
    );
    // Flashback: mana spent.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Vexing Bauble");
    let twice = t.graveyard(P0, "Think Twice");
    fill_pool(&mut t, P0, 3);
    let hand = t.hand_size(P0);
    t.cast(P0, twice)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Free with Omniscience: countered; with Thorn of Amethyst's {1} too: not.
    for thorn in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Vexing Bauble");
        t.battlefield(P0, "Omniscience");
        if thorn {
            t.battlefield(P1, "Thorn of Amethyst");
            fill_pool(&mut t, P0, 1);
        }
        let div = t.hand(P0, "Divination");
        let hand = t.hand_size(P0);
        t.cast(P0, div).method(CastMethod::Free).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand - 1 + if thorn { 2 } else { 0 }, "{thorn}");
    }
}

#[test]
fn a_spell_vexing_bauble_counters_was_still_cast() {
    cr!("601.2i", "603.2", "702.40a");
    ruling!(
        "Vexing Bauble",
        "Players may cast spells that they know Vexing Bauble will counter. Any abilities that trigger when spells are cast will still trigger and resolve if appropriate, and any effects that count how many spells are cast will still count those spells if appropriate."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Vexing Bauble");
    t.battlefield(P0, "Omniscience");
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    t.battlefield(P0, "Young Pyromancer");
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 1);
    // Grapeshot (storm) counts it: two copies' worth of damage plus the original.
    let shot = t.hand(P0, "Grapeshot");
    fill_pool(&mut t, P0, 2);
    t.cast(P0, shot).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 2);
}

#[test]
fn nontoken_foods_are_foods() {
    cr!("205.3g");
    supported("Apothecary White");
    supported("Krovod Haunch");
    ruling!(
        "Apothecary White",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, you can tap Krovod Haunch to pay for Apothecary White's activated ability."
    );
    let mut t = TestGame::new(2);
    let white = t.battlefield(P0, "Apothecary White");
    let haunch = t.battlefield(P0, "Krovod Haunch");
    let block = t.battlefield(P0, "Nutrient Block");
    fill_pool(&mut t, P0, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(haunch), obj(block)]);
    t.activate(P0, white, 0, &[]).expect("activate");
    t.resolve_all();
    assert!(t.obj_now(haunch).tapped && t.obj_now(block).tapped);
    assert_eq!(with_subtype(&t, P0, "Human").len(), 3);
    // Tempting Witch: "{2}, {T}, Sacrifice a Food: Target player loses 3 life."
    supported("Tempting Witch");
    ruling!(
        "Dubious Delicacy",
        "If an effect refers to a Food, it means any Food artifact, not just a Food token."
    );
    ruling!(
        "Nutrient Block",
        "If an effect refers to a Food, it means any Food artifact, not just a Food token."
    );
    for food in ["Dubious Delicacy", "Nutrient Block"] {
        let mut t = TestGame::new(2);
        let witch = t.battlefield(P0, "Tempting Witch");
        let f = t.battlefield(P0, food);
        t.answer_choose(P0, &[obj(f)]);
        t.answer_targets(P0, &[pl(P1)]);
        fill_pool(&mut t, P0, 2);
        activate_containing(&mut t, P0, witch, "Sacrifice a Food").expect(food);
        t.resolve_all();
        assert!(t.in_graveyard(P0, food), "{food}");
        assert_eq!(t.life(P1), 17, "{food}");
    }
}
