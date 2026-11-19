//! Rulings batch P084 — "target player shuffles up to N target cards from their graveyard
//! into their library" (Gaea's Blessing, Memory's Journey, Stream of Consciousness,
//! Witness the Future) and Turn the Earth: the shuffle happens with no legal card
//! targets, each library is shuffled once, and Gaea's Blessing's milled-with cards.

use crate::r_p084_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_target_player_shuffles_with_no_or_only_illegal_card_targets() {
    cr!("608.2b", "701.24a", "115.1");
    ruling!(
        "Gaea's Blessing",
        "If Gaea's Blessing has no legal target cards in a graveyard, either because its targets became illegal or because you didn't choose any, the target player shuffles their library."
    );
    ruling!(
        "Memory's Journey",
        "If no cards were targeted by Memory's Journey or if all the targeted cards are illegal targets by the time Memory's Journey resolves, the targeted player will still shuffle their library."
    );
    ruling!(
        "Witness the Future",
        "If you choose zero target cards or the cards are illegal targets as Witness the Future resolves, the target player will still shuffle their library."
    );
    ruling!(
        "Stream of Consciousness",
        "If you target any cards in a graveyard and those cards all become illegal targets, the target player simply shuffles their library."
    );
    // (card, cost, also draws (Gaea's Blessing) / looks at four (Witness the Future))
    for (name, cost, draws) in [
        ("Gaea's Blessing", "1G", true),
        ("Memory's Journey", "1U", false),
        ("Witness the Future", "2U", true),
        ("Stream of Consciousness", "1U", false),
    ] {
        for none_chosen in [true, false] {
            let mut t = TestGame::new(2);
            let a = t.graveyard(P1, "Grizzly Bears");
            let b = t.graveyard(P1, "Hill Giant");
            let cards: Vec<Entity> = if none_chosen {
                vec![]
            } else {
                vec![obj(a), obj(b)]
            };
            cast_modes(&mut t, P0, name, cost, &[], &[&[pl(P1)], &cards]);
            let hand = t.hand_size(P0);
            if !none_chosen {
                exile_now(&mut t, a);
                exile_now(&mut t, b);
            }
            let from = t.g.turn_events.len();
            t.resolve_all();
            assert_eq!(shuffles_since(&t, P1, from), 1, "{name} none={none_chosen}");
            // The rest of the spell happens too.
            assert_eq!(
                t.hand_size(P0),
                hand + usize::from(draws),
                "{name} none={none_chosen}"
            );
            assert!(t.in_graveyard(P0, name), "{name}");
        }
    }
}

#[test]
fn turn_the_earth_shuffles_each_library_once() {
    cr!("701.24a", "608.2c");
    ruling!(
        "Turn the Earth",
        "Each affected library is shuffled only once, even if multiple cards are being shuffled into it."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let c = t.graveyard(P0, "Lightning Bolt");
    cast_modes(
        &mut t,
        P0,
        "Turn the Earth",
        "G",
        &[],
        &[&[obj(a), obj(b), obj(c)]],
    );
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(shuffles_since(&t, P1, from), 1);
    assert_eq!(shuffles_since(&t, P0, from), 1);
    assert_eq!(t.library_size(P1), 32);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn gaeas_blessing_milled_with_other_cards_shuffles_them_all_in() {
    // The trigger waits for the whole mill (603.3: it goes on the stack the next time a
    // player would receive priority).
    cr!("603.2", "603.3", "701.17a");
    ruling!(
        "Gaea's Blessing",
        "If an effect puts multiple cards from your library into your graveyard at once, put all of those cards there before Gaea's Blessing shuffles your graveyard into your library."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Gaea's Blessing", "Hill Giant"],
    );
    let lib = t.library_size(P0);
    t.g.mill(P0, 3);
    t.g.flush_events();
    t.settle();
    // All three are in the graveyard while the trigger waits.
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), lib + 1);
}
