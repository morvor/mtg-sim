//! CR 508.3e: "Whenever [a player] attacks [another player]" triggers only if creatures
//! the first player controls are declared as attackers attacking the second player: an
//! opponent attacking a different player doesn't trigger "whenever an opponent attacks
//! you", and a player attacking you doesn't trigger "whenever a player attacks one or
//! more of your opponents".

use crate::r506_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn three_players() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

/// P0 controls Emberwilde Captain and is the monarch; P1 holds three cards and attacks
/// with two Grizzly Bears as declared by `targets`.
fn attack(targets: &[Entity]) -> TestGame {
    let mut t = three_players();
    // "When this creature enters, you become the monarch. Whenever an opponent attacks you
    // while you're the monarch, this creature deals damage to that player equal to the
    // number of cards in their hand."
    t.enter(P0, "Emberwilde Captain");
    t.resolve_all();
    for name in ["Island", "Island", "Island"] {
        t.hand(P1, name);
    }
    let decl: Vec<(ObjectId, Entity)> = targets
        .iter()
        .map(|e| (t.battlefield(P1, "Grizzly Bears"), *e))
        .collect();
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &decl);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    t
}

#[test]
fn an_opponent_attacking_another_player_doesnt_trigger_whenever_an_opponent_attacks_you() {
    cr!("508.3e");
    let t = attack(&[Entity::Player(P2), Entity::Player(P2)]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_opponent_attacking_you_with_two_creatures_triggers_it_once() {
    cr!("508.3e");
    ruling!(
        "Emberwilde Captain",
        "Emberwilde Captain's last ability triggers once for each player who attacks you"
    );
    let t = attack(&[Entity::Player(P0), Entity::Player(P0)]);
    assert_eq!(t.life(P1), 17);
}

/// P0 controls Jolene, the Plunder Queen; `attacker`'s Grizzly Bears attacks the other
/// player. Returns the Treasures each player controls afterwards.
fn jolene(attacker: PlayerId) -> (usize, usize) {
    // "Whenever a player attacks one or more of your opponents, that attacking player
    // creates a Treasure token. If you would create one or more Treasure tokens, instead
    // create those tokens plus an additional Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jolene, the Plunder Queen");
    let bears = t.battlefield(attacker, "Grizzly Bears");
    let defender = if attacker == P0 { P1 } else { P0 };
    t.set_step(attacker, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(defender))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    let treasures = |p: PlayerId| {
        t.permanents()
            .filter(|o| o.controller == p && o.chars.has_subtype("Treasure"))
            .count()
    };
    (treasures(P0), treasures(P1))
}

#[test]
fn a_player_attacking_you_doesnt_trigger_whenever_a_player_attacks_your_opponents() {
    cr!("508.3e");
    // P0 attacks P1, an opponent of Jolene's controller: P0 creates a Treasure (plus one).
    assert_eq!(jolene(P0), (2, 0));
    // P1 attacks P0, Jolene's controller: no trigger.
    assert_eq!(jolene(P1), (0, 0));
}
