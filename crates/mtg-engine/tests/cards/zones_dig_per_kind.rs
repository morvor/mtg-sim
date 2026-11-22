//! Card-flow grammar IV (`cards2-zones-dig`): dig selections of one card for each kind
//! ("For each card type among noncreature spells you've cast this turn, you may put a
//! card of that type from among the revealed cards into your hand", "For each of those
//! colors, you may exile a card of that color", "a card with flying, a card with first
//! strike, and so on for ..."), reveal-until a chosen kind, and cards exiled from several
//! zones at once.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.0 as usize].library.clear();
    names.iter().map(|n| t.library_top(p, n)).collect()
}

/// `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

fn names(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

fn last_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice")
}

#[test]
fn hurkyl_takes_one_card_of_each_type_among_noncreature_spells_cast() {
    cr!("205.2a", "701.20e");
    assert_supported("Hurkyl, Master Wizard");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hurkyl, Master Wizard");
    t.lands(P0, "Mountain", 1);
    // An instant was cast this turn.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve();
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Divination",
            "Lightning Bolt",
            "Counterspell",
            "Grizzly Bears",
        ],
    );
    let (bolt, counter) = (ids[2], ids[3]);
    // Two instants: only one card of the type can be taken.
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(counter)]);
    t.advance_to_step(Step::End);
    t.resolve_all();
    let offered = last_candidates(&t);
    // Only the instants (the sorcery and the creature are of types not cast).
    assert_eq!(offered.len(), 2);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.library_size(P0), 4);
}

#[test]
fn sanar_exiles_one_card_of_each_color_among_its_permanents() {
    cr!("105.2", "701.20a");
    assert_supported("Sanar, Innovative First-Year");
    let mut t = TestGame::new(2);
    // Sanar is blue and red, the Elves green: X is 3.
    t.battlefield(P0, "Sanar, Innovative First-Year");
    t.battlefield(P0, "Llanowar Elves");
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Grizzly Bears",
            "Lightning Bolt",
            "Shock",
            "Island",
            "Counterspell",
        ],
    );
    let (shock, bolt, counter) = (ids[3], ids[2], ids[5]);
    // Shock and Lightning Bolt are both red: only one of them; Counterspell is blue.
    t.answer_choose(
        P0,
        &[
            Entity::Object(shock),
            Entity::Object(bolt),
            Entity::Object(counter),
        ],
    );
    t.set_step(P0, Step::Draw);
    t.advance_to_step(Step::PrecombatMain);
    t.resolve_all();
    let exiled = names(&t, &t.g.exile.clone());
    assert_eq!(exiled, ["Counterspell", "Shock"], "{}", t.dump_log());
    // The other revealed cards were shuffled back.
    assert_eq!(t.library_size(P0), 4);
    assert!(t
        .g
        .player(P0)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name == "Lightning Bolt"));
}

#[test]
fn selective_adaptation_one_card_for_each_keyword() {
    cr!("701.20e", "702.9a");
    assert_supported("Selective Adaptation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Grizzly Bears",
            "Serra Angel",
            "Giant Spider",
            "Wind Drake",
            "Raging Goblin",
            "Shock",
        ],
    );
    let (angel, spider, drake, goblin) = (ids[2], ids[3], ids[4], ids[5]);
    let spell = t.hand(P0, "Selective Adaptation");
    // Serra Angel (flying, vigilance), Giant Spider (reach), Raging Goblin (haste); the
    // Wind Drake (flying) can't also stand for flying with the Angel standing for it...
    // but the Angel can stand for vigilance, so all four may be chosen.
    t.answer_choose(
        P0,
        &[
            Entity::Object(angel),
            Entity::Object(spider),
            Entity::Object(drake),
            Entity::Object(goblin),
        ],
    );
    // The one put onto the battlefield.
    t.answer_choose(P0, &[Entity::Object(spider)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Giant Spider").len(), 1, "{}", t.dump_log());
    let hand = names(&t, &t.g.player(P0).hand.clone());
    assert_eq!(hand, ["Raging Goblin", "Serra Angel", "Wind Drake"]);
    let gy = names(&t, &t.g.player(P0).graveyard.clone());
    assert_eq!(gy, ["Forest", "Grizzly Bears", "Selective Adaptation", "Shock"]);
}

#[test]
fn abundant_harvest_reveals_until_a_card_of_the_chosen_kind() {
    cr!("701.20a");
    assert_supported("Abundant Harvest");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    library(&mut t, P0, &["Lightning Bolt", "Island", "Shock", "Grizzly Bears"]);
    let spell = t.hand(P0, "Abundant Harvest");
    // Land.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Island"));
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Lightning Bolt");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Grizzly Bears", "Shock"]);

    // Nonland.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    library(&mut t, P0, &["Lightning Bolt", "Island", "Shock", "Forest"]);
    let spell = t.hand(P0, "Abundant Harvest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(library_names(&t, P0)[0], "Island");
}

#[test]
fn crabomination_exiles_from_library_graveyard_and_hand() {
    cr!("701.13a");
    assert_supported("Crabomination");
    let mut t = TestGame::new(2);
    library(&mut t, P1, &["Forest", "Island"]);
    t.graveyard(P1, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Cast nothing.
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.enter(P0, "Crabomination");
    t.resolve_all();
    let exiled = names(&t, &t.g.exile.clone());
    assert_eq!(exiled, ["Grizzly Bears", "Island", "Shock"]);
    assert_eq!(library_names(&t, P1), ["Forest"]);
}

#[test]
fn trial_of_a_time_lord_guilty_puts_the_exiled_cards_on_the_bottom() {
    cr!("714.2b", "701.38a");
    assert_supported("Trial of a Time Lord");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    library(&mut t, P1, &["Forest", "Island"]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let saga = t.enter(P0, "Trial of a Time Lord");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    // Both players vote guilty.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.g.add_counters(Entity::Object(saga), "lore", 3, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(
        library_names(&t, P1),
        ["Island", "Forest", "Grizzly Bears"],
        "{}",
        t.dump_log()
    );
}
