//! The worked examples of CR 704 (state-based actions), played out as tests.

use crate::r600_examples_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn maro_survives_a_hand_wheel_and_an_empty_hand_trigger_waits_for_the_spell() {
    cr!("603.8", "704.5f");
    cr_example!("704.4", "Maro");
    supported("Maro");
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    // "Whenever you have no cards in hand, ..." triggers on a state.
    t.custom(
        P0,
        custom_card(
            "Empty Hand Muse",
            "Enchantment",
            None,
            "Whenever you have no cards in hand, you gain 1 life.",
        ),
        Zone::Battlefield,
    );
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    let wheel = t.custom(
        P0,
        custom_costed(
            "Private Wheel",
            "Sorcery",
            "{U}",
            "Discard your hand, then draw seven cards.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, wheel).go();
    t.recompute();
    assert_eq!(t.pt(maro), (2, 2));
    t.resolve();
    // Toughness 0 only in the middle of the resolution: Maro survives as a 7/7.
    assert!(t.on_battlefield(maro));
    assert_eq!(t.pt(maro), (7, 7));
    assert_eq!(t.graveyard_size(P0), 3);
    // The trigger event happened during the resolution: the ability is put on the stack
    // after the spell has resolved.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn lichs_mirror_replaces_a_double_game_loss_once() {
    cr!("704.5a", "704.5b", "614.1a");
    cr_example!("704.7", "Lich’s Mirror");
    supported("Lich's Mirror");
    supported("Night's Whisper");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Lich's Mirror");
    // One card in the library and 1 life.
    t.g.players[0].library.clear();
    t.library_top(P0, "Forest");
    t.g.players[0].life = 1;
    for _ in 0..6 {
        t.hand(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 2);
    let whisper = t.hand(P0, "Night's Whisper");
    t.cast(P0, whisper).go();
    t.resolve();
    // Instead of losing, the hand, graveyard, and permanents are shuffled into the
    // library, P0 draws seven cards, and their life total becomes 20.
    assert!(!t.has_lost(P0));
    assert!(t.g.result.is_none());
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), 7);
    assert!(!t.g.is_live(mirror));
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.g.permanents_controlled_by(P0).is_empty());
    // 7 cards in hand (Forest + 6 Bears) + Night's Whisper + 3 permanents = 11; 7 drawn.
    assert_eq!(t.library_size(P0), 4);
    // A single replacement: the library was shuffled once.
    assert_eq!(
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == P0))
            .count(),
        1
    );
}

#[test]
fn young_wolf_with_a_plus_one_counter_doesnt_return_after_three_minus_one_counters() {
    cr!("704.5f", "704.5r", "702.93a", "603.10a");
    cr_example!("704.8", "Young Wolf");
    supported("Young Wolf");
    let wither = || {
        custom_costed(
            "Triple Wither",
            "Instant",
            "{B}",
            "Put three -1/-1 counters on target creature.",
        )
    };
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Young Wolf");
    t.g.add_counters(Entity::Object(wolf), counters::PLUS1, 1, None);
    t.lands(P1, "Swamp", 1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let s = t.custom(P1, wither(), Zone::Hand(P1));
    t.cast(P1, s).target(wolf).go();
    t.g.resolve_top();
    // Before state-based actions: one +1/+1 counter and three -1/-1 counters.
    assert_eq!(t.counters(wolf, counters::PLUS1), 1);
    assert_eq!(t.counters(wolf, counters::MINUS1), 3);
    t.settle();
    t.resolve_all();
    // It had a +1/+1 counter as it last existed on the battlefield: undying doesn't
    // trigger.
    assert!(t.in_graveyard(P0, "Young Wolf"));
    assert!(t.named_on_battlefield("Young Wolf").is_empty());

    // Without the +1/+1 counter, undying returns it.
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Young Wolf");
    t.lands(P1, "Swamp", 1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let s = t.custom(P1, wither(), Zone::Hand(P1));
    t.cast(P1, s).target(wolf).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Young Wolf");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], counters::PLUS1), 1);
}
