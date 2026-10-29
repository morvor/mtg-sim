//! cards5-referents-statics, family `values`: values a spell's text defines (CR 107.3c).

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

#[test]
fn spoils_of_war_x_is_the_number_in_an_opponents_graveyard() {
    cr!("107.3c", "601.2b");
    interp!("x-defined-for-a-player");
    ruling!(
        "Spoils of War",
        "choose a higher or lower value for X"
    );
    supported("Spoils of War");
    for (two_opponents, answer, expected) in [(false, 5, 2), (false, 1, 2), (true, 3, 3), (true, 2, 2)] {
        let mut t = TestGame::new(if two_opponents { 3 } else { 2 });
        let bears = t.battlefield(P0, "Grizzly Bears");
        // P1: two creature cards and a land; P2: three artifact or creature cards.
        t.graveyard(P1, "Hill Giant");
        t.graveyard(P1, "Ornithopter");
        t.graveyard(P1, "Island");
        if two_opponents {
            t.graveyard(P2, "Hill Giant");
            t.graveyard(P2, "Ornithopter");
            t.graveyard(P2, "Grizzly Bears");
        }
        t.lands(P0, "Swamp", 6);
        let spell = t.hand(P0, "Spoils of War");
        t.answer(P0, DecisionKind::X, Answer::Number(answer));
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![expected as i64]));
        let s = t.cast(P0, spell).target(Entity::Object(bears)).go();
        assert_eq!(
            t.g.obj(s).stack.as_ref().and_then(|s| s.x),
            Some(expected),
            "answered {answer}"
        );
        t.resolve_all();
        assert_eq!(t.counters(bears, "+1/+1"), expected as u32);
    }
}
