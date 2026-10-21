//! The worked examples of CR 727 (restarting the game), played out as tests.

use mtg_engine::card::card;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_card_brought_in_with_living_wish_is_in_the_restarted_games_library() {
    cr!("727.1", "104.6");
    cr_example!("727.2", "Living Wish");
    let d = card("Living Wish");
    assert!(d.is_fully_supported(), "{:?}", d.unsupported_text());
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    // Living Wish: "You may reveal a creature or land card you own from outside the game
    // and put it into your hand. Exile Living Wish."
    let side = t.g.add_to_sideboard(P0, vec![card("Hill Giant")]);
    t.lands(P0, "Forest", 2);
    let wish = t.hand(P0, "Living Wish");
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.cast(P0, wish).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.g.player(P0).sideboard.is_empty());
    // Karn Liberated: "−14: Restart the game, ..."
    let k = t.battlefield(P0, "Karn Liberated");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 30);
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
    assert_eq!(t.g.turn.number, 1);
    // The Hill Giant is part of P0's deck in the new game (it was in the library as the
    // game began; it may since have been drawn into the opening hand), not back outside
    // the game.
    let giants: Vec<Zone> =
        t.g.objects
            .iter()
            .filter(|o| {
                o.next.is_none()
                    && o.zone != Zone::Nowhere
                    && o.card.as_ref().is_some_and(|c| c.name == "Hill Giant")
            })
            .map(|o| {
                assert_eq!(o.owner, P0);
                o.zone
            })
            .collect();
    assert_eq!(giants.len(), 1);
    assert!(
        matches!(giants[0], Zone::Library(P0) | Zone::Hand(P0)),
        "{:?}",
        giants[0]
    );
    assert!(t.g.player(P0).sideboard.is_empty());
    // Living Wish itself (exiled) is in the new game too.
    let wishes = t
        .g
        .objects
        .iter()
        .filter(|o| {
            o.next.is_none()
                && matches!(o.zone, Zone::Library(P0) | Zone::Hand(P0))
                && o.card.as_ref().is_some_and(|c| c.name == "Living Wish")
        })
        .count();
    assert_eq!(wishes, 1);
}
