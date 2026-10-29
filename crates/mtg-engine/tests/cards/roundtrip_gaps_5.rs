//! roundtrip-gaps-5 (docs/agents/items/roundtrip-gaps-5.md): the remaining gap clusters
//! (custom nodes) and the next 40 two-card clusters.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn spikeshell_harrier_slows_the_opponent_who_controlled_the_creature() {
    cr!("702.179b", "608.2h");
    // "return target creature or Vehicle an opponent controls to its owner's hand. If that
    // opponent's speed is greater than each other player's speed, reduce that opponent's
    // speed by 1": the opponent who controlled it, even when its owner is another player.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.g.objects[bears.0 as usize].controller = P1;
    t.g.players[0].speed = Some(1);
    t.g.players[1].speed = Some(3);
    t.g.players[2].speed = Some(2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Spikeshell Harrier");
    t.resolve_all();
    assert!(t.in_hand(P2, "Grizzly Bears"));
    assert_eq!(t.g.players[1].speed, Some(2), "{}", t.dump_log());
    assert_eq!(t.g.players[2].speed, Some(2));
}
