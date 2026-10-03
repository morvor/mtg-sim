//! Replacement effects that modify how permanents enter the battlefield, compiled by
//! `src/oracle/patterns/etb_replacement_grammar.rs` and the "as ~ enters" instructions of
//! `etb_choices.rs` (CR 614.1c, 614.12, 614.15).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn etb_grammar_cards_compile() {
    compiles(&[
        "Captive Audience",
        "Callous Oppressor",
        "Heightened Awareness",
        "Monstrous War-Leech",
        "Urborg Lhurgoyf",
        "Phylactery Lich",
        "Overlaid Terrain",
        "Ixidron",
        "What Must Be Done",
        "Necromantic Summons",
        "Recommission",
        "Heroic Return",
        "Vigor Mortis",
        "Winter Soldier, Reborn Avenger",
        "Zameck Guildmage",
        "Combine Guildmage",
        "Turntimber Symbiosis // Turntimber, Serpentine Wood",
        "Nick Fury, Spymaster",
        "Kari Zev, Skyship Raider",
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
    ]);
}

#[test]
fn captive_audience_enters_under_an_opponents_control() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let ca = t.enter(P0, "Captive Audience");
    t.resolve_all();
    let now = t.g.current(ca);
    assert_eq!(t.g.obj(now).controller, P2);
}

#[test]
fn heightened_awareness_discards_your_hand_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let h = t.hand(P0, "Heightened Awareness");
    t.cast(P0, h).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Heightened Awareness").len(), 1);
}

#[test]
fn monstrous_war_leech_mills_as_it_enters_only_if_kicked() {
    cr!("614.1c", "702.33d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Island", 1);
    let leech = t.hand(P0, "Monstrous War-Leech");
    let lib = t.library_size(P0);
    t.cast(P0, leech).kicked(true).go();
    t.resolve();
    assert_eq!(t.library_size(P0), lib - 4);
    // Not kicked: nothing milled.
    t.lands(P0, "Swamp", 4);
    let leech2 = t.hand(P0, "Monstrous War-Leech");
    t.cast(P0, leech2).kicked(false).go();
    t.resolve();
    assert_eq!(t.library_size(P0), lib - 4);
}

#[test]
fn overlaid_terrain_sacrifices_your_lands_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let o = t.hand(P0, "Overlaid Terrain");
    t.cast(P0, o).go();
    t.resolve();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert_eq!(t.named_on_battlefield("Overlaid Terrain").len(), 1);
}

#[test]
fn recommission_gives_a_returned_creature_an_additional_counter() {
    cr!("614.1c", "614.15");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Recommission");
    t.cast(P0, r).target(Entity::Object(bears)).go();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(b, "+1/+1"), 1);
    // An artifact that isn't a creature gets none.
    let rock = t.graveyard(P0, "Mind Stone");
    t.lands(P0, "Plains", 2);
    let r2 = t.hand(P0, "Recommission");
    t.cast(P0, r2).target(Entity::Object(rock)).go();
    t.resolve();
    let s = t.named_on_battlefield("Mind Stone")[0];
    assert_eq!(t.counters(s, "+1/+1"), 0);
}

#[test]
fn zameck_guildmage_creatures_enter_with_an_additional_counter_this_turn() {
    cr!("614.1c", "122.6");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Zameck Guildmage");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve();
    let bears = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // An opponent's creature doesn't.
    let theirs = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn phylactery_lich_puts_a_counter_on_an_artifact_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    let rock = t.battlefield(P0, "Mind Stone");
    t.answer_choose(P0, &[Entity::Object(rock)]);
    t.enter(P0, "Phylactery Lich");
    t.resolve_all();
    assert_eq!(t.counters(rock, "phylactery"), 1);
    assert_eq!(t.named_on_battlefield("Phylactery Lich").len(), 1);
}

#[test]
fn gond_gate_lets_gates_enter_untapped_but_not_ones_put_onto_the_battlefield_tapped() {
    cr!("614.1c", "616.1");
    ruling!(
        "Spelunking",
        "If a land has an ability that says it enters the battlefield tapped, you choose the order"
    );
    compiles(&["Gond Gate", "Uphill Battle", "Archelos, Lagoon Mystic"]);
    // The controller of the entering Gate chooses the order: one order leaves it untapped,
    // the other tapped.
    let mut results = Vec::new();
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gond Gate");
        t.answer(
            P0,
            DecisionKind::Replacement,
            mtg_engine::decision::Answer::Index(pick),
        );
        let gate = t.enter(P0, "Azorius Guildgate");
        t.resolve_all();
        results.push(t.g.obj(t.g.current(gate)).tapped);
    }
    results.sort();
    assert_eq!(results, vec![false, true]);
    // Put onto the battlefield tapped by an instruction: it stays tapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gond Gate");
    let card = t.graveyard(P0, "Plains");
    let mut d = mtg_engine::ability::Destination::battlefield();
    d.tapped = true;
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    let moved = t.g.move_to_destination(vec![card], &d, &mut ctx);
    assert!(t.g.obj(moved[0]).tapped);
}

#[test]
fn uphill_battle_taps_creatures_opponents_cast_only() {
    cr!("614.1c", "601.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Uphill Battle");
    t.g.turn.active = P1;
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.g.obj(b).tapped);
    // Put onto the battlefield without being played: untapped.
    let other = t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert!(!t.g.obj(t.g.current(other)).tapped);
    // Your own creatures aren't affected.
    let mine = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.g.obj(t.g.current(mine)).tapped);
}

#[test]
fn archelos_makes_other_permanents_enter_tapped_while_tapped() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Archelos, Lagoon Mystic");
    t.g.tap(a);
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.g.obj(t.g.current(bears)).tapped);
    t.g.untap(a);
    let giant = t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert!(!t.g.obj(t.g.current(giant)).tapped);
}

#[test]
fn spark_double_copies_with_an_additional_counter_and_isnt_legendary() {
    cr!("707.9b", "707.9e", "707.9f");
    ruling!(
        "Spark Double",
        "If it copies a creature, Spark Double enters with a +1/+1 counter on it"
    );
    compiles(&["Spark Double", "Moritte of the Frost", "Altered Ego", "Auton Soldier"]);
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(legend)]);
    let sd = t.enter(P0, "Spark Double");
    t.resolve_all();
    let sd = t.g.current(sd);
    let o = t.g.obj(sd);
    assert_eq!(o.chars.name, "Isamaru, Hound of Konda");
    assert!(!o.chars.supertypes.contains(mtg_engine::types::Supertype::Legendary));
    assert_eq!(t.counters(sd, "+1/+1"), 1);
    // Both are still on the battlefield (no legend rule).
    assert!(t.on_battlefield(legend));
}

#[test]
fn altered_ego_enters_with_x_additional_counters() {
    cr!("707.9e", "107.3");
    ruling!(
        "Altered Ego",
        "The value of X in Altered Ego's last ability will be whatever value was chosen for X"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 4);
    let ego = t.hand(P0, "Altered Ego");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, ego).x(3).go();
    t.resolve_all();
    let copy = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .find(|o| *o != bears)
        .expect("copy");
    assert_eq!(t.counters(copy, "+1/+1"), 3);
}

#[test]
fn infinite_reflection_nontoken_creatures_enter_as_copies_of_the_enchanted_creature() {
    cr!("707.9", "614.1c");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = t.battlefield(P0, "Infinite Reflection");
    t.g.obj_mut(aura).attached_to = Some(Entity::Object(giant));
    let bears = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(bears)).chars.name, "Hill Giant");
    // Not an opponent's.
    let theirs = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(theirs)).chars.name, "Grizzly Bears");
}

#[test]
fn thief_of_blood_takes_all_counters_as_it_enters() {
    cr!("614.1c", "614.12a", "122.6");
    compiles(&[
        "Thief of Blood",
        "Shimatsu the Bloodcloaked",
        "Devouring Hellion",
        "Arsenal Thresher",
    ]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.objects[a.0 as usize].counters.insert("+1/+1".into(), 2);
    t.g.objects[b.0 as usize].counters.insert("-1/-1".into(), 1);
    let thief = t.enter(P0, "Thief of Blood");
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(b, "-1/-1"), 0);
    assert_eq!(t.counters(t.g.current(thief), "+1/+1"), 3);
}

#[test]
fn devouring_hellion_enters_with_twice_that_many_counters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let h = t.enter(P0, "Devouring Hellion");
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.counters(t.g.current(h), "+1/+1"), 4);
}

#[test]
fn teferis_time_twist_returns_a_creature_with_an_additional_counter() {
    cr!("614.1c", "614.15", "603.7");
    compiles(&[
        "Teferi's Time Twist",
        "Silver Surfer, Cosmic Voyager",
        "The First Tyrannic War",
    ]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let tw = t.hand(P0, "Teferi's Time Twist");
    t.cast(P0, tw).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    t.advance_to_step(mtg_engine::turn::Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
}
