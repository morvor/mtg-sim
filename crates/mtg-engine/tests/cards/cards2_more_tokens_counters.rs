//! More token and counter phrases (CR 111, 122): "another one of those tokens" for each
//! poisoned opponent (Wurmquake), counters on each of the cards just exiled that has a
//! keyword (Ecstatic Beauty), returning with one fewer counter (Nine-Lives Familiar).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn wurmquake_creates_another_wurm_for_each_corrupted_opponent() {
    cr!("111.1", "122.1");
    ruling!(
        "Wurmquake",
        "The first token that Wurmquake creates as it resolves is created and enters the battlefield before the rest of them"
    );
    supported("Wurmquake");
    let mut t = TestGame::new(3);
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.g.add_counters(Entity::Player(P2), "poison", 2, None);
    t.lands(P0, "Forest", 6);
    let w = t.hand(P0, "Wurmquake");
    t.cast(P0, w).go();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Phyrexian Wurm Token");
    assert_eq!(wurms.len(), 2);
    for w in &wurms {
        assert_eq!(t.pt(*w), (6, 6));
    }
}

#[test]
fn ecstatic_beauty_puts_time_counters_on_exiled_cards_with_suspend() {
    cr!("122.1", "702.62a");
    supported("Ecstatic Beauty");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let sus = t.library_top(P0, "Ecstatic Beauty");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let e = t.hand(P0, "Ecstatic Beauty");
    t.cast(P0, e).go();
    t.resolve_all();
    let now = t.g.current(sus);
    assert_eq!(t.g.obj(now).zone, mtg_engine::object::Zone::Exile);
    assert_eq!(t.g.obj(now).counter("time"), 4);
    let giant =
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.g.obj(giant[0]).counter("time"), 0);
}

#[test]
fn nine_lives_familiar_returns_with_one_fewer_revival_counter() {
    cr!("122.1", "603.7", "603.10a");
    supported("Nine-Lives Familiar");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Nine-Lives Familiar");
    t.g.add_counters(Entity::Object(cat), "revival", 3, None);
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(cat).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Nine-Lives Familiar"));
    t.advance_to_step(Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Nine-Lives Familiar");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "revival"), 2);
}

#[test]
fn ochre_jelly_token_enters_with_half_the_counters_rounded_down() {
    cr!("122.6", "707.2", "603.7");
    ruling!(
        "Ochre Jelly",
        "if Ochre Jelly had five +1/+1 counters on it when it dies, the token you create will enter the battlefield with two +1/+1 counters on it"
    );
    supported("Ochre Jelly");
    let mut t = TestGame::new(2);
    let jelly = t.battlefield(P0, "Ochre Jelly");
    t.g.add_counters(Entity::Object(jelly), "+1/+1", 5, None);
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(jelly).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Ochre Jelly").is_empty());
    t.advance_to_step(Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Ochre Jelly");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).is_token());
    assert_eq!(t.counters(back[0], "+1/+1"), 2);
    assert_eq!(t.pt(back[0]), (2, 2));
}
