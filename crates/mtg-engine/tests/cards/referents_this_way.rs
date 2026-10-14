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

#[test]
fn spiteful_repossession_deals_each_opponent_the_difference() {
    cr!("608.2h", "101.4");
    compiles("Spiteful Repossession");
    let mut t = TestGame::new(3);
    let p2 = PlayerId(2);
    t.lands(P0, "Mountain", 3);
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    t.lands(P1, "Forest", 7);
    t.lands(p2, "Forest", 2);
    let spell = t.hand(P0, "Spiteful Repossession");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "seven lands against three");
    assert_eq!(t.life(p2), 20, "fewer lands than you");
    let treasures = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 4, "one for each damage dealt");
}

#[test]
fn savage_punch_gives_the_bonus_before_the_fight() {
    cr!("701.14a");
    ruling!(
        "Savage Punch",
        "Ferocious abilities of instants and sorceries that don’t use the word “instead” will provide an additional effect"
    );
    compiles("Savage Punch");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Indomitable Ancients");
    let spell = t.hand(P0, "Savage Punch");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(wurm), Entity::Object(giant)])
        .go();
    t.resolve_all();
    // Craw Wurm 6/4 +2/+2 = 8/6 fights a 2/10: it survives the 2 damage and deals 8.
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.g.obj(giant).damage, 8);
}

#[test]
fn beastie_beatdown_counters_then_damage_from_the_creature_you_control() {
    cr!("207.2c", "608.2c");
    compiles("Beastie Beatdown");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    for c in ["Lightning Bolt", "Grizzly Bears", "Forest", "Bonesplitter"] {
        t.graveyard(P0, c);
    }
    let spell = t.hand(P0, "Beastie Beatdown");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bear), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    // A 4/4 deals 4: the 3/3 dies, and the Bears isn't dealt damage (one-sided).
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.g.obj(bear).damage, 0);
}

#[test]
fn animists_awakening_untaps_only_the_lands_it_put_onto_the_battlefield() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Animist's Awakening",
        "If the spell mastery ability applies, you untap only the lands put onto the battlefield with Animist's Awakening."
    );
    compiles("Animist's Awakening");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    for c in ["Grizzly Bears", "Plains", "Island"] {
        t.library_top(P0, c);
    }
    let spell = t.hand(P0, "Animist's Awakening");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    let island = t.named_on_battlefield("Island");
    let plains = t.named_on_battlefield("Plains");
    assert_eq!((island.len(), plains.len()), (1, 1));
    assert!(!t.g.obj(island[0]).tapped && !t.g.obj(plains[0]).tapped);
    // The Forests that paid for it stay tapped.
    assert!(t
        .named_on_battlefield("Forest")
        .iter()
        .all(|f| t.g.obj(*f).tapped));
}

#[test]
fn increasing_confusion_mills_twice_x_from_a_graveyard() {
    cr!("702.34a", "608.2c");
    compiles("Increasing Confusion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    for _ in 0..10 {
        t.library_top(P1, "Forest");
    }
    let spell = t.graveyard(P0, "Increasing Confusion");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .x(2)
        .method(object::CastMethod::Keyword(keywords::KeywordKind::Flashback))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 4);
}
