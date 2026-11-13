//! cards5-referents-statics, family `spell-targets`: conditions on a spell's targets and
//! values defined by a target spell (CR 107.3c, 115.7, 115.9).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// P1 casts Lightning Bolt at `at`; P0 activates Reflecting Mirror for `x` at the Bolt,
/// choosing P1 as the new target. Returns the game and the X the ability was activated
/// with, if it was.
fn mirror(at: impl Fn(&mut TestGame) -> Entity, x: i64) -> (TestGame, Option<i32>) {
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Reflecting Mirror");
    t.lands(P0, "Island", 4);
    t.lands(P1, "Mountain", 1);
    let target = at(&mut t);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(target).go();
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ok = t
        .activate(P0, mirror, 0, &[Entity::Object(bolt)])
        .ok()
        .flatten()
        .and_then(|a| t.g.obj(a).stack.as_ref().and_then(|s| s.x));
    t.resolve_all();
    (t, ok)
}

#[test]
fn reflecting_mirror_redirects_a_spell_that_targets_you() {
    cr!("115.7", "115.9b", "107.3c");
    interp!("x-defined-by-target");
    ruling!(
        "Reflecting Mirror",
        "This spell only changes the target of a spell, and not the player."
    );
    supported("Reflecting Mirror");
    // X = 2 (twice the Bolt's mana value): the Bolt now hits P1.
    let (t, x) = mirror(|_| Entity::Player(P0), 2);
    assert_eq!(x, Some(2));
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    // A Bolt at a creature: the target isn't you, nothing changes.
    let (t, x) = mirror(
        |t: &mut TestGame| Entity::Object(t.battlefield(P0, "Craw Wurm")),
        2,
    );
    assert_eq!(x, Some(2));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.g.obj(t.named_on_battlefield("Craw Wurm")[0]).damage, 3);
}

#[test]
fn reflecting_mirror_x_must_be_twice_the_targets_mana_value() {
    cr!("107.3c", "601.2b", "602.2b");
    interp!("x-defined-by-target");
    // An X that isn't twice the Bolt's mana value can't be announced for it: the only
    // activation at the Bolt is for X = 2.
    for x in [1, 4] {
        let (_, used) = mirror(|_| Entity::Player(P0), x);
        assert!(used.is_none() || used == Some(2), "X = {x}: {used:?}");
    }
}
