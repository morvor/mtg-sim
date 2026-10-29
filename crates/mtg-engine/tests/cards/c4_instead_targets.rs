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

fn green_suns_twilight(x: i64, battlefield: bool) -> TestGame {
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let bears = t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 6);
    let spell = t.hand(P0, "Green Sun's Twilight");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(forest)]);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(if battlefield { 0 } else { 1 }),
    );
    t.cast(P0, spell).x(x).go();
    t.resolve_all();
    t
}

#[test]
fn green_suns_twilight_puts_both_cards_onto_the_battlefield_with_x_5() {
    cr!("608.2c", "107.3a");
    ruling!(
        "Green Sun's Twilight",
        "You can't choose to put one into your hand and the other onto the battlefield."
    );
    supported("Green Sun's Twilight");
    let t = green_suns_twilight(5, true);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Forest").len(), 7);
    let t = green_suns_twilight(5, false);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Forest"));
    // With X less than 5, they go to your hand.
    let t = green_suns_twilight(4, true);
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Forest"));
}

fn snapdax(with_br: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Plains", 2);
    if with_br {
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Mountain", 1);
    } else {
        t.lands(P0, "Plains", 2);
    }
    let spell = t.hand(P0, "Mythos of Snapdax");
    // Whoever chooses keeps the Llanowar Elves for P1 (P1 by default keeps the first).
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    (t, giant, elves)
}

#[test]
fn mythos_of_snapdax_with_black_and_red_you_choose_for_each_player() {
    cr!("101.4c", "601.2h");
    ruling!(
        "Mythos of Snapdax",
        "If you’re choosing the permanents for each player, your choices must still be legal."
    );
    supported("Mythos of Snapdax");
    let (t, giant, elves) = snapdax(true);
    assert!(t.on_battlefield(elves), "{}", t.dump_log());
    assert!(!t.on_battlefield(giant));
    // Without {B}{R}: P1 chooses.
    let (t, giant, elves) = snapdax(false);
    assert!(t.on_battlefield(giant), "{}", t.dump_log());
    assert!(!t.on_battlefield(elves));
}
