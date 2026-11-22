//! "Room abilities of dungeons you own trigger an additional time." (Hama Pashar, Ruin
//! Seeker; CR 309.4c, 603.2d).

use mtg_engine::ability::{Effect, KeywordAction, PlayerRef, Sel, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn venture(t: &mut TestGame, choice: Option<usize>) {
    if let Some(i) = choice {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::KeywordAction {
            action: KeywordAction::Venture,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
}

fn treasures(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn hama_pashar_doubles_room_abilities() {
    cr!("309.4c", "603.2d");
    for n in ["Hama Pashar, Ruin Seeker", "Dungeon Delver"] {
        assert!(card(n).unsupported_text().is_empty(), "{n}");
    }
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hama Pashar, Ruin Seeker");
    t.custom(
        P0,
        (*card("Lost Mine of Phandelver")).clone(),
        Zone::Outside(P0),
    );
    // Cave Entrance, then Mine Tunnels: "Create a Treasure token." twice.
    venture(&mut t, None);
    venture(&mut t, Some(1));
    assert_eq!(treasures(&t), 2);
    // Without Hama Pashar: once.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        (*card("Lost Mine of Phandelver")).clone(),
        Zone::Outside(P0),
    );
    venture(&mut t, None);
    venture(&mut t, Some(1));
    assert_eq!(treasures(&t), 1);
}
