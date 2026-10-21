//! Card-flow grammar IV (`cards2-zones-dig`), hands and libraries continued: cards set
//! aside from a hand and put back (Scroll Rack), two cards both on top or both on the
//! bottom (Dream Cache), a hand and the top card of a library revealed together
//! (Psychotic Episode), a card revealed from a hand or the top of a library (Eladamri),
//! cards named like a target exiled from a hand and graveyard (Legion's End, Hour of
//! Glory), a revealed card discarded unless its owner pays life (Wand of Ith), a land
//! type not shared (Hiveheart Shaman), and Gomazoa.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
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

fn exile_names(t: &TestGame) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .exile
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn scroll_rack_exchanges_cards_from_hand_with_the_top_of_the_library() {
    cr!("701.13a", "401.4");
    assert_supported("Scroll Rack");
    let mut t = TestGame::new(2);
    let rack = t.battlefield(P0, "Scroll Rack");
    t.lands(P0, "Island", 1);
    let shock = t.hand(P0, "Shock");
    let bolt = t.hand(P0, "Lightning Bolt");
    library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bolt)]);
    t.activate(P0, rack, 0, &[]).unwrap();
    t.resolve();
    // The top two cards (Swamp, Island) are in hand; Shock and Bolt are on top.
    let mut hand: Vec<String> = t
        .g
        .player(P0)
        .hand
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    hand.sort();
    assert_eq!(hand, ["Island", "Swamp"]);
    let lib = library_names(&t, P0);
    let mut top: Vec<String> = lib[..2].to_vec();
    top.sort();
    assert_eq!(top, ["Lightning Bolt", "Shock"]);
    assert_eq!(lib[2], "Forest");
    assert!(t.g.exile.is_empty());
}

#[test]
fn scroll_rack_with_too_few_cards_in_the_library() {
    cr!("701.13a");
    ruling!("Scroll Rack", "If there are not enough cards in the library, you do not lose.");
    let mut t = TestGame::new(2);
    let rack = t.battlefield(P0, "Scroll Rack");
    t.lands(P0, "Island", 1);
    let shock = t.hand(P0, "Shock");
    let bolt = t.hand(P0, "Lightning Bolt");
    library(&mut t, P0, &["Swamp"]);
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bolt)]);
    t.activate(P0, rack, 0, &[]).unwrap();
    t.resolve();
    assert!(!t.has_lost(P0));
    assert!(t.in_hand(P0, "Swamp"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn dream_cache_puts_two_cards_both_on_the_bottom() {
    cr!("401.4");
    assert_supported("Dream Cache");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let shock = t.hand(P0, "Shock");
    let bolt = t.hand(P0, "Lightning Bolt");
    library(&mut t, P0, &["Forest", "Island", "Swamp", "Mountain", "Plains"]);
    let spell = t.hand(P0, "Dream Cache");
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bolt)]);
    // The bottom.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    let lib = library_names(&t, P0);
    assert_eq!(lib.len(), 4);
    assert_eq!(lib[0], "Island");
    let mut bottom = lib[2..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Lightning Bolt", "Shock"]);
}

#[test]
fn psychotic_episode_puts_the_chosen_revealed_card_on_the_bottom() {
    cr!("701.20a");
    ruling!(
        "Psychotic Episode",
        "You can choose any card that Psychotic Episode instructs the player to reveal"
    );
    assert_supported("Psychotic Episode");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Shock");
    let ids = library(&mut t, P1, &["Forest", "Island", "Serra Angel"]);
    let angel = ids[2];
    let spell = t.hand(P0, "Psychotic Episode");
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    // The top card was among the cards to choose from.
    let offered = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice");
    assert_eq!(offered.len(), 2);
    assert_eq!(library_names(&t, P1), ["Island", "Forest", "Serra Angel"]);
    assert!(t.in_hand(P1, "Shock"));
}

#[test]
fn eladamri_reveals_a_creature_from_the_top_and_puts_it_onto_the_battlefield() {
    cr!("701.20a");
    assert_supported("Eladamri, Korvecdal");
    let mut t = TestGame::new(2);
    let eladamri = t.battlefield(P0, "Eladamri, Korvecdal");
    t.lands(P0, "Forest", 1);
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Shock");
    let ids = library(&mut t, P0, &["Forest", "Serra Angel"]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.activate(P0, eladamri, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1, "{}", t.dump_log());
    assert_eq!(library_names(&t, P0), ["Forest"]);
}

#[test]
fn eladamri_a_noncreature_card_stays_where_it_is() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let eladamri = t.battlefield(P0, "Eladamri, Korvecdal");
    t.lands(P0, "Forest", 1);
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    library(&mut t, P0, &["Forest", "Serra Angel"]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.activate(P0, eladamri, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.zone(shock), Zone::Hand(P0));
    assert!(t.named_on_battlefield("Serra Angel").is_empty());
}

#[test]
fn legions_end_exiles_cards_with_that_name_from_hand_and_graveyard() {
    cr!("201.2");
    assert_supported("Legion's End");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let target = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Shock");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    let spell = t.hand(P0, "Legion's End");
    t.cast(P0, spell).target(Entity::Object(target)).go();
    t.resolve();
    assert_eq!(
        exile_names(&t),
        [
            "Grizzly Bears",
            "Grizzly Bears",
            "Grizzly Bears",
            "Grizzly Bears"
        ]
    );
    assert!(t.in_hand(P1, "Shock"));
    assert!(t.in_graveyard(P1, "Forest"));
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn hour_of_glory_exiles_a_gods_namesakes_from_its_controllers_hand() {
    cr!("201.2");
    assert_supported("Hour of Glory");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let god = t.battlefield(P1, "Rhonas the Indomitable");
    t.hand(P1, "Rhonas the Indomitable");
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "Hour of Glory");
    t.cast(P0, spell).target(Entity::Object(god)).go();
    t.resolve();
    assert_eq!(
        exile_names(&t),
        ["Rhonas the Indomitable", "Rhonas the Indomitable"]
    );
    assert_eq!(t.hand_size(P1), 1);

    // Not a God: the hand isn't touched.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Hour of Glory");
    t.cast(P0, spell).target(Entity::Object(bears)).go();
    t.resolve();
    assert_eq!(exile_names(&t), ["Grizzly Bears"]);
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn wand_of_ith_a_land_is_discarded_unless_its_owner_pays_one_life() {
    cr!("701.9a", "118.12");
    assert_supported("Wand of Ith");
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Wand of Ith");
    t.lands(P0, "Island", 3);
    t.hand(P1, "Forest");
    t.answer_yes(P1, false);
    t.activate(P0, wand, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Forest"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);

    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Wand of Ith");
    t.lands(P0, "Island", 3);
    t.hand(P1, "Serra Angel");
    t.answer_yes(P1, true);
    t.activate(P0, wand, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // A nonland card: paying life equal to its mana value (5) keeps it.
    assert!(t.in_hand(P1, "Serra Angel"));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn hiveheart_shaman_finds_a_basic_land_with_a_new_land_type() {
    cr!("205.3i", "701.23a");
    assert_supported("Hiveheart Shaman");
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Hiveheart Shaman");
    t.battlefield(P0, "Forest");
    let ids = library(&mut t, P0, &["Forest", "Island"]);
    let (forest, island) = (ids[0], ids[1]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.attack(&[(shaman, Entity::Player(P1))], &[]);
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice");
    // The Forest shares a land type with a land P0 controls: it can't be found.
    assert!(!offered.contains(&Entity::Object(forest)));
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
}

#[test]
fn gomazoa_puts_itself_and_the_creature_it_blocks_into_their_libraries() {
    cr!("701.24a");
    ruling!(
        "Gomazoa",
        "If Gomazoa isn’t blocking a creature at the time its ability resolves, only Gomazoa"
    );
    assert_supported("Gomazoa");
    let mut t = TestGame::new(2);
    let goma = t.battlefield(P0, "Gomazoa");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(goma, bears)]));
    t.advance_to(P1, Step::DeclareBlockers);
    t.activate(P0, goma, 0, &[]).unwrap();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{}", t.dump_log());
    assert!(t.g.player(P1).library.iter().any(|c| t.g.obj(*c).chars.name == "Grizzly Bears"));
    assert!(t.g.player(P0).library.iter().any(|c| t.g.obj(*c).chars.name == "Gomazoa"));

    // Not blocking: only Gomazoa.
    let mut t = TestGame::new(2);
    let goma = t.battlefield(P0, "Gomazoa");
    t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, goma, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.g.player(P0).library.iter().any(|c| t.g.obj(*c).chars.name == "Gomazoa"));
}
