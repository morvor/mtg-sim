//! CR 704.5t (completing a dungeon) and CR 704.5u (sector designations from space
//! sculptor).

use crate::r703_common::*;
use mtg_engine::ability::KeywordAction;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::dungeons;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn venture(t: &mut TestGame, p: PlayerId) {
    keyword_action(t, p, KeywordAction::Venture, 1);
}

fn choose(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

fn dungeon_named(t: &TestGame, zone: Zone, name: &str) -> Vec<ObjectId> {
    t.g.find_in_zone(zone, name)
}

#[test]
fn a_dungeon_is_completed_once_the_bottommost_rooms_ability_has_left_the_stack() {
    cr!("704.5t");
    supported("Lost Mine of Phandelver");
    ruling!(
        "Lost Mine of Phandelver",
        "may be put into a player's command zone using the venture into the dungeon"
    );
    let mut t = TestGame::new(2);
    // Venture into Lost Mine of Phandelver (the first of the dungeons offered): Cave
    // Entrance — Scry 1.
    choose(&mut t, P0, 0);
    venture(&mut t, P0);
    let mine = dungeon_named(&t, Zone::Command, "Lost Mine of Phandelver");
    assert_eq!(mine.len(), 1);
    assert_eq!(dungeons::marker(&t.g, P0), Some((mine[0], 0)));
    t.resolve_all();
    // Cave Entrance leads to Goblin Lair or Mine Tunnels: Mine Tunnels — Create a
    // Treasure token.
    choose(&mut t, P0, 1);
    venture(&mut t, P0);
    t.resolve_all();
    assert_eq!(dungeons::marker(&t.g, P0), Some((mine[0], 2)));
    // Mine Tunnels leads to Dark Pool or Fungi Cavern: Dark Pool — Each opponent loses 1
    // life and you gain 1 life.
    choose(&mut t, P0, 0);
    venture(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // Temple of Dumathoin (the bottommost room) — Draw a card.
    let hand = t.hand_size(P0);
    venture(&mut t, P0);
    // The dungeon is the source of a room ability that hasn't left the stack: it stays.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(mine[0]), Zone::Command);
    assert_eq!(t.g.player(P0).dungeons_completed, 0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Then its owner removes it from the game: they completed the dungeon.
    assert!(dungeon_named(&t, Zone::Command, "Lost Mine of Phandelver").is_empty());
    assert_eq!(
        dungeon_named(&t, Zone::Outside(P0), "Lost Mine of Phandelver").len(),
        1
    );
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(dungeons::marker(&t.g, P0), None);
}

#[test]
fn a_dungeon_stays_while_any_of_its_room_abilities_waits() {
    cr!("704.5t");
    let mut t = TestGame::new(2);
    choose(&mut t, P0, 0);
    venture(&mut t, P0);
    let mine = dungeon_named(&t, Zone::Command, "Lost Mine of Phandelver")[0];
    // The marker reaches the bottommost room while an earlier room ability is still on
    // the stack; the dungeon isn't removed while the room abilities wait.
    for pick in [0, 1] {
        choose(&mut t, P0, pick);
        venture(&mut t, P0);
    }
    venture(&mut t, P0);
    assert_eq!(dungeons::marker(&t.g, P0), Some((mine, 6)));
    t.settle();
    assert_eq!(t.stack_len(), 4);
    assert_eq!(t.zone(mine), Zone::Command);
    t.resolve_all();
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
}

fn sector(t: &TestGame, id: ObjectId) -> Option<String> {
    t.obj(id).sector.as_ref().map(|s| s.to_string())
}

fn sector_questions(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("sector"))
        })
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn creatures_without_a_sector_get_one_opponents_of_space_sculptor_first() {
    cr!("704.5u");
    ruling!(
        "Space Beleren",
        "any time there are creatures on the battlefield that aren’t assigned to a sector"
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.settle();
    // No permanent with space sculptor: no sectors.
    assert_eq!(sector(&t, mine), None);
    let asked = t.asked().len();
    // P1 (who doesn't control a permanent with space sculptor) chooses first: gamma.
    // Then P0: alpha.
    choose(&mut t, P1, 2);
    choose(&mut t, P0, 0);
    t.battlefield(P0, "Space Beleren");
    t.settle();
    assert_eq!(sector_questions(&t, asked), vec![P1, P0]);
    assert_eq!(sector(&t, theirs).as_deref(), Some("gamma"));
    assert_eq!(sector(&t, mine).as_deref(), Some("alpha"));
    // Once assigned, a creature keeps its sector: nobody is asked again.
    let asked = t.asked().len();
    t.settle();
    assert!(sector_questions(&t, asked).is_empty());
}

#[test]
fn a_new_creature_gets_its_own_sector_and_sectors_end_with_space_sculptor() {
    cr!("704.5u");
    ruling!("Space Beleren", "Sector assignment isn’t copiable");
    ruling!(
        "Space Beleren",
        "As soon as there are no permanents with space sculptor on the battlefield"
    );
    let mut t = TestGame::new(2);
    let beleren = t.battlefield(P0, "Space Beleren");
    choose(&mut t, P0, 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(sector(&t, bears).as_deref(), Some("alpha"));
    // A Clone copying the Bears doesn't copy its sector: its controller assigns one.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    choose(&mut t, P0, 1);
    let copy = t.enter(P0, "Clone");
    t.settle();
    let copy = t.g.current(copy);
    assert_eq!(t.obj(copy).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(sector(&t, copy).as_deref(), Some("beta"));
    // Without a permanent with space sculptor, all sector designations are lost.
    t.g.move_object(
        beleren,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.settle();
    assert_eq!(sector(&t, bears), None);
    assert_eq!(sector(&t, copy), None);
}
