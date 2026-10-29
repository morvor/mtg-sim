//! CR 104.6 (and 727): Karn Liberated restarts the game.

use crate::r100_common::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls Karn Liberated with plenty of loyalty.
fn karn(t: &mut TestGame) -> ObjectId {
    let k = t.battlefield(P0, "Karn Liberated");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 30);
    k
}

/// "−3: Exile target permanent."
fn karn_exile(t: &mut TestGame, k: ObjectId, what: ObjectId) {
    t.activate(P0, k, 1, &[Entity::Object(what)]).unwrap();
    t.resolve();
    t.g.objects[k.0 as usize].activations_this_turn.clear();
}

/// "−14: Restart the game, leaving in exile all non-Aura permanent cards exiled with
/// Karn. Then put those cards onto the battlefield under your control."
fn karn_restart(t: &mut TestGame, k: ObjectId) {
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
}

fn cards_owned(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> =
        t.g.objects
            .iter()
            .filter(|o| o.owner == p && o.next.is_none() && o.zone != Zone::Nowhere)
            .filter(|o| !matches!(o.zone, Zone::Outside(_)))
            .map(|o| o.card.as_ref().map_or("?".into(), |c| c.name.to_string()))
            .collect();
    v.sort();
    v
}

#[test]
fn karn_liberated_restarts_the_game() {
    cr!("104.6", "104.1", "727.1", "727.1a", "727.2", "727.4", "727.5");
    ruling!(
        "Karn Liberated",
        "No player wins, loses, or draws the original game as a result of Karn's ability."
    );
    ruling!(
        "Karn Liberated",
        "The player who controlled the ability that restarted the game is the starting player in the new game."
    );
    ruling!(
        "Karn Liberated",
        "Players won't have any counters or emblems they had in the original game."
    );
    ruling!(
        "Karn Liberated",
        "Permanents put onto the battlefield due to Karn's ability will have been under the starting controller's control continuously since the beginning of that player's first turn."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    t.set_step(P1, Step::PrecombatMain);
    t.set_step(P0, Step::PrecombatMain);
    let k = karn(&mut t);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    // An Aura exiled with Karn goes back into its owner's deck.
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, Entity::Object(lions));
    karn_exile(&mut t, k, bear);
    karn_exile(&mut t, k, pacifism);
    t.hand(P1, "Giant Growth");
    t.graveyard(P1, "Lightning Bolt");
    t.g.add_to_sideboard(P1, vec![card("Hill Giant")]);
    t.g.players[1].life = 3;
    t.g.players[1].counters.insert("poison".into(), 4);
    t.g.players[0].life = 11;
    let before_p1 = cards_owned(&t, P1);
    karn_restart(&mut t, k);

    // The old game ended without a result; a new one began with P0 as the starting
    // player, just before its first untap step.
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.g.turn.starting_player, P0);
    assert_eq!((t.g.turn.active, t.g.turn.step), (P0, Step::Untap));
    assert_eq!(t.g.turn.stage, Stage::Begin);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.g.player(P1).poison(), 0);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
    // Every card involved is in the new game — Karn and Pacifism in their owners' decks —
    // except the Bears, which were left in exile and then put onto the battlefield under
    // P0's control.
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    let b = t.g.obj(bears[0]);
    assert_eq!((b.owner, b.controller), (P1, P0));
    assert_eq!(t.g.battlefield.len(), 1);
    assert_eq!(cards_owned(&t, P1), before_p1);
    let p0_cards = cards_owned(&t, P0);
    assert!(p0_cards.contains(&"Karn Liberated".to_string()));
    assert!(p0_cards.contains(&"Savannah Lions".to_string()));
    // The sideboard stays outside the game.
    assert_eq!(t.g.player(P1).sideboard.len(), 1);
    // The Bears can attack on the first turn.
    let ok = t.g.run_until(100, |g| {
        g.turn.step == Step::PrecombatMain && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    assert!(!t.g.obj(bears[0]).summoning_sick);
}

#[test]
fn players_who_left_arent_in_the_new_game() {
    cr!("104.6", "727.1");
    ruling!(
        "Karn Liberated",
        "In a multiplayer game, any player who left the game before it was restarted with Karn's ability won't be involved in the new game."
    );
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    t.take_action(P2, Action::Concede);
    let k = karn(&mut t);
    karn_restart(&mut t, k);
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.g.players_in_game(), vec![P0, P1]);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P2), 0);
    assert!(cards_owned(&t, P2).is_empty());
}

#[test]
fn an_exempted_commander_stays_in_exile_and_remains_the_commander() {
    cr!("727.5a");
    ruling!(
        "Karn Liberated",
        "In that case, the commander remains in exile and will be put onto the battlefield when Karn's ability finishes resolving."
    );
    ruling!(
        "Karn Liberated",
        "The amount of combat damage dealt to players by each commander is reset to 0."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let k = karn(&mut t);
    let cmdr = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[1]
        .commander_names
        .push("Isamaru, Hound of Konda".into());
    t.g.players[0]
        .commander_damage
        .insert("Isamaru, Hound of Konda".into(), 12);
    // Its owner may move it to the command zone (CR 704.6d); they leave it in exile.
    t.answer_yes(P1, false);
    karn_exile(&mut t, k, cmdr);
    let exiled = t.g.find_in_zone(Zone::Exile, "Isamaru, Hound of Konda");
    assert_eq!(exiled.len(), 1);
    karn_restart(&mut t, k);
    let isamaru = t.named_on_battlefield("Isamaru, Hound of Konda");
    assert_eq!(isamaru.len(), 1);
    assert!(t.g.obj(isamaru[0]).is_commander);
    assert_eq!(t.g.obj(isamaru[0]).controller, P0);
    assert!(t
        .g
        .find_in_zone(Zone::Command, "Isamaru, Hound of Konda")
        .is_empty());
    assert!(t.g.player(P0).commander_damage.is_empty());
    assert_eq!(t.life(P0), 40);
}
