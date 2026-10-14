//! Referent grammar III: objects and amounts an earlier instruction or cost affected
//! (CR 608.2c, 608.2h, 400.7).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn devouring_greed_counts_the_spirits_sacrificed_for_its_cost() {
    cr!("601.2b", "608.2c");
    compiles("Devouring Greed");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let a = t.battlefield(P0, "Kami of Ancient Law");
    let b = t.battlefield(P0, "Kami of Ancient Law");
    let c = t.battlefield(P0, "Kami of Ancient Law");
    let spell = t.hand(P0, "Devouring Greed");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Number(2),
    );
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b) && t.on_battlefield(c));
    // 2 + 2 for each of the two Spirits.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn out_of_time_counts_the_creatures_that_phased_out() {
    cr!("702.26a", "702.26g");
    ruling!(
        "Out of Time",
        "If there are no creatures on the battlefield when Out of Time's triggered ability resolves"
    );
    compiles("Out of Time");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.tap(b);
    let time = t.enter(P0, "Out of Time");
    t.resolve_all();
    assert!(t.g.obj(a).phased_out && t.g.obj(b).phased_out);
    assert!(!t.g.obj(b).tapped, "untapped first");
    assert_eq!(t.counters(time, "time"), 2);
    // No creatures: no counters.
    let mut t = TestGame::new(2);
    let time = t.enter(P0, "Out of Time");
    t.resolve_all();
    assert_eq!(t.counters(time, "time"), 0);
}

#[test]
fn last_one_standing_destroys_all_but_one_random_creature() {
    cr!("701.8a");
    ruling!(
        "Last One Standing",
        "The creature chosen at random isn’t targeted, so an opponent’s creature with hexproof can be chosen this way."
    );
    compiles("Last One Standing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let mut all = Vec::new();
    for _ in 0..3 {
        all.push(t.battlefield(P0, "Grizzly Bears"));
        all.push(t.battlefield(P1, "Gladecover Scout"));
    }
    let spell = t.hand(P0, "Last One Standing");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    let left = all.iter().filter(|c| t.on_battlefield(**c)).count();
    assert_eq!(left, 1);
}

#[test]
fn lost_in_the_maze_stuns_only_the_tapped_creatures_you_dont_control() {
    cr!("122.1d");
    compiles("Lost in the Maze");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(theirs)]);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    let maze = t.hand(P0, "Lost in the Maze");
    t.lands(P0, "Island", 4);
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, maze).x(2).go();
    t.resolve_all();
    assert!(t.g.obj(mine).tapped && t.g.obj(theirs).tapped);
    assert_eq!(t.counters(theirs, "stun"), 1);
    assert_eq!(t.counters(mine, "stun"), 0);
}

#[test]
fn unstable_glyphbridge_destroys_all_creatures_except_the_chosen_ones() {
    cr!("608.2c");
    ruling!(
        "Unstable Glyphbridge // Sandswirl Wanderglyph",
        "For each player, including yourself, you must choose a creature with power 2 or less to save if you can."
    );
    let def = card("Unstable Glyphbridge // Sandswirl Wanderglyph");
    // The front face compiles (the back face has other constructs).
    assert!(def
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("chosen this way")));
    let mut t = TestGame::new(2);
    let keep_mine = t.battlefield(P0, "Grizzly Bears");
    let lose_mine = t.battlefield(P0, "Llanowar Elves");
    let keep_theirs = t.battlefield(P1, "Llanowar Elves");
    let big = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 5);
    let glyph = t.hand(P0, "Unstable Glyphbridge // Sandswirl Wanderglyph");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(keep_mine)]);
    t.answer_choose(P0, &[Entity::Object(keep_theirs)]);
    t.cast(P0, glyph).go();
    t.resolve_all();
    assert!(t.on_battlefield(keep_mine) && t.on_battlefield(keep_theirs));
    assert!(!t.on_battlefield(lose_mine) && !t.on_battlefield(big));
}
