//! Inferno of the Star Mounts (hand-written, `src/cards/inferno_of_the_star_mounts.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn pump(t: &mut TestGame, inferno: ObjectId, times: usize) {
    for _ in 0..times {
        t.activate(P0, inferno, 0, &[]).unwrap();
        t.resolve_all();
    }
}

#[test]
fn deals_20_when_this_ability_makes_its_power_20() {
    cr!("603.12");
    ruling!("Inferno of the Star Mounts", "If its power becomes 21 or greater this way, the ability won't trigger");
    let mut t = TestGame::new(2);
    let inferno = t.battlefield(P0, "Inferno of the Star Mounts");
    t.lands(P0, "Mountain", 15);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // 6 power: thirteen activations reach 19, the fourteenth makes it 20.
    pump(&mut t, inferno, 13);
    assert_eq!(t.life(P1), 20);
    pump(&mut t, inferno, 1);
    assert_eq!(t.pt(inferno).0, 20);
    assert_eq!(t.life(P1), 0);
}

#[test]
fn not_at_21() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    let inferno = t.battlefield(P0, "Inferno of the Star Mounts");
    t.lands(P0, "Mountain", 1);
    // Already at 20 power by another effect: pumping to 21 doesn't trigger.
    t.g.add_counters(Entity::Object(inferno), "+1/+1", 14, None);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    pump(&mut t, inferno, 1);
    assert_eq!(t.pt(inferno).0, 21);
    assert_eq!(t.life(P1), 20);
}
