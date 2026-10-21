//! The worked examples of CR 729 (subgames), played out as tests.

use mtg_engine::card::card;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::subgame::{self, Subgame};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn cards_brought_into_a_subgame_go_to_the_main_game_library() {
    cr!("729.4", "729.4a");
    cr_example!("729.5", "brought into the subgame");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    // A main-game card (Grizzly Bears in P0's graveyard) and a card outside the main
    // game (Hill Giant in P0's sideboard).
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.g.add_to_sideboard(P0, vec![card("Hill Giant")])[0];
    let lib0 = t.library_size(P0);
    let Subgame { mut game, outside } = subgame::begin(&mut t.g);
    game.logging = true;
    game.start();
    let mut s = TestGame {
        g: game,
        script: t.script.clone(),
    };
    s.set_step(P0, Step::PrecombatMain);
    // Living Wish twice in the subgame: "You may reveal a creature or land card you own
    // from outside the game and put it into your hand."
    s.lands(P0, "Forest", 4);
    for main in [bears, giant] {
        let proxy = outside.iter().find(|(_, m)| *m == main).unwrap().0;
        let wish = s.hand(P0, "Living Wish");
        s.answer_choose(P0, &[Entity::Object(proxy)]);
        s.cast(P0, wish).go();
        s.resolve_all();
    }
    assert!(s.in_hand(P0, "Grizzly Bears"));
    assert!(s.in_hand(P0, "Hill Giant"));
    subgame::finish(&mut t.g, Subgame { game: s.g, outside });
    // Both are in P0's main-game library now, with the other cards from the subgame.
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.g.player(P0).sideboard.is_empty());
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Grizzly Bears").len(), 1);
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Hill Giant").len(), 1);
    // (The two Living Wishes and four Forests created in the subgame for the test are
    // P0's cards there too.)
    assert_eq!(t.library_size(P0), lib0 + 2 + 2 + 4);
}
