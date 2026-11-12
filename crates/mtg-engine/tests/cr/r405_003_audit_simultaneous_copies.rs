//! CR 405.3 audit (zones.rs, `order_simultaneous_stack_objects`): objects an effect puts
//! onto the stack at the same time go on top of what's already there, and a player who
//! controls more than one of them chooses their relative order.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn a_player_orders_their_simultaneous_copies_above_the_rest_of_the_stack() {
    cr!("405.3", "707.10");
    for name in ["Hive Mind", "Twinning Staff", "Prodigal Pyromancer"] {
        supported(name);
    }
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hive Mind");
    t.battlefield(P0, "Twinning Staff");
    // Three abilities on the stack ("{T}: This creature deals 1 damage to any target.").
    for _ in 0..3 {
        let pyro = t.battlefield(P0, "Prodigal Pyromancer");
        t.activate(P0, pyro, 0, &[Entity::Player(P1)])
            .expect("ping");
    }
    // P1 casts Lightning Bolt at P0. Hive Mind ("Whenever a player casts an instant or
    // sorcery spell, each other player copies that spell.") has P0 copy it, and Twinning
    // Staff ("If you would copy a spell one or more times, instead copy it that many times
    // plus an additional time.") makes that two copies put onto the stack at the same time:
    // P0 chooses their order, the second copy lowest.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 5, "three pings, the Bolt, Hive Mind's trigger");
    let below: Vec<ObjectId> = t.g.stack[..3].to_vec();
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.resolve();
    assert_eq!(t.stack_len(), 6);
    assert_eq!(&t.g.stack[..3], below.as_slice());
    assert_eq!(t.g.stack[3], spell);
    let (lower, upper) = (t.g.stack[4], t.g.stack[5]);
    for c in [lower, upper] {
        assert_eq!(t.obj(c).kind, mtg_engine::object::ObjKind::SpellCopy);
        assert_eq!(t.obj(c).controller, P0);
    }
    assert!(
        lower.0 > upper.0,
        "the copy made second is lowest, as P0 ordered them"
    );
    // Everything resolves: the copies and the Bolt hit P0, the pings P1.
    t.resolve_all();
    assert_eq!(t.life(P0), 11);
    assert_eq!(t.life(P1), 17);
}
