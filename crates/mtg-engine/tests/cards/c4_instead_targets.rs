//! cards4-referents-statics: instead alternatives with their own targets (The Eagles Are
//! Coming!).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn eagles(kicked: bool) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 8);
    let spell = t.hand(P0, "The Eagles Are Coming!");
    let targets: Vec<Entity> = if kicked {
        vec![Entity::Object(a), Entity::Object(b)]
    } else {
        vec![Entity::Object(a)]
    };
    t.cast(P0, spell).kicked(kicked).targets(&targets).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    (t, vec![a, b])
}

fn birds(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.has_subtype("Bird"))
        .count()
}

#[test]
fn the_eagles_are_coming_returns_one_creature_unkicked() {
    cr!("608.2c", "702.33d");
    supported("The Eagles Are Coming!");
    let (t, ids) = eagles(false);
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.on_battlefield(ids[1]));
    assert_eq!(birds(&t), 1);
}

#[test]
fn the_eagles_are_coming_kicked_returns_each_chosen_creature() {
    cr!("608.2c", "702.33d");
    let (t, _) = eagles(true);
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(birds(&t), 2);
}
