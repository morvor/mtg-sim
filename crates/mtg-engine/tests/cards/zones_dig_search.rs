//! Card-flow grammar IV (`cards2-zones-dig`): searches. Same-name searches of a player's
//! zones for an object other than a target ("the chosen card", "that card" for each card
//! exiled), an opponent choosing among the revealed cards found (Gifts Ungiven), group
//! requirements on the cards found ("with different powers", "that share a land type"),
//! and one search for each object for a card with its name (Doubling Chant).

use mtg_engine::ability::{Duration, Sel};
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
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

fn names_in(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    names_in(t, &t.g.player(p).graveyard.clone())
}

fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    names_in(t, &t.g.player(p).hand.clone())
}

fn exile_names(t: &TestGame) -> Vec<String> {
    names_in(t, &t.g.exile.clone())
}

fn library_sorted(t: &TestGame, p: PlayerId) -> Vec<String> {
    names_in(t, &t.g.player(p).library.clone())
}

#[test]
fn lobotomy_exiles_the_chosen_card_and_all_others_with_its_name() {
    cr!("701.23b", "201.2");
    ruling!("Lobotomy", "It exiles the chosen card plus all others in the hand");
    ruling!("Lobotomy", "Does not affect cards on the battlefield.");
    assert_supported("Lobotomy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let shock = t.hand(P1, "Shock");
    t.hand(P1, "Forest");
    t.graveyard(P1, "Shock");
    t.battlefield(P1, "Llanowar Elves");
    library(&mut t, P1, &["Lightning Bolt", "Shock", "Llanowar Elves"]);
    let spell = t.hand(P0, "Lobotomy");
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(exile_names(&t), ["Shock", "Shock", "Shock"]);
    assert_eq!(hand_names(&t, P1), ["Forest"]);
    assert_eq!(library_sorted(&t, P1), ["Lightning Bolt", "Llanowar Elves"]);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn lobotomy_copies_in_the_library_may_be_left() {
    cr!("701.23b");
    ruling!("Lobotomy", "Finding copies while searching private zones is optional.");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let shock = t.hand(P1, "Shock");
    let in_gy = t.graveyard(P1, "Shock");
    library(&mut t, P1, &["Shock", "Lightning Bolt"]);
    let spell = t.hand(P0, "Lobotomy");
    t.answer_choose(P0, &[Entity::Object(shock)]);
    // The copy in the hand (the chosen card) is found; the one in the library isn't.
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    // The graveyard copy must be found (a public zone).
    assert_eq!(t.zone(in_gy), Zone::Exile);
    assert_eq!(exile_names(&t), ["Shock", "Shock"]);
    assert_eq!(library_sorted(&t, P1), ["Lightning Bolt", "Shock"]);
}

#[test]
fn shimian_specter_exiles_cards_named_like_the_chosen_card() {
    cr!("701.23b", "201.2");
    ruling!(
        "Shimian Specter",
        "The cards you’re searching for must be found and exiled if they’re in the graveyard"
    );
    assert_supported("Shimian Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Shimian Specter");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    t.graveyard(P1, "Lightning Bolt");
    library(&mut t, P1, &["Lightning Bolt", "Shock"]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.attack(&[(specter, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(
        exile_names(&t),
        ["Lightning Bolt", "Lightning Bolt", "Lightning Bolt"]
    );
    assert_eq!(hand_names(&t, P1), ["Forest"]);
    assert_eq!(library_sorted(&t, P1), ["Shock"]);
}

#[test]
fn haunting_echoes_exiles_library_cards_named_like_each_exiled_card() {
    cr!("701.23b", "201.2");
    ruling!(
        "Haunting Echoes",
        "you don’t have to find all the cards with the same name as an exiled card"
    );
    assert_supported("Haunting Echoes");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    library(
        &mut t,
        P1,
        &["Shock", "Grizzly Bears", "Forest", "Lightning Bolt", "Shock"],
    );
    let spell = t.hand(P0, "Haunting Echoes");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    // The basic land stays; the Shock and the Bears are exiled with every copy in the
    // library (found by default).
    assert_eq!(graveyard_names(&t, P1), ["Forest"]);
    assert_eq!(
        exile_names(&t),
        [
            "Grizzly Bears",
            "Grizzly Bears",
            "Shock",
            "Shock",
            "Shock"
        ]
    );
    assert_eq!(library_sorted(&t, P1), ["Forest", "Lightning Bolt"]);
}

#[test]
fn gifts_ungiven_opponent_chooses_two_for_the_graveyard() {
    cr!("701.23a", "201.2");
    assert_supported("Gifts Ungiven");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let ids = library(
        &mut t,
        P0,
        &[
            "Shock",
            "Lightning Bolt",
            "Forest",
            "Grizzly Bears",
            "Island",
            "Shock",
        ],
    );
    let (shock, bolt, forest, bears) = (ids[0], ids[1], ids[2], ids[3]);
    let spell = t.hand(P0, "Gifts Ungiven");
    t.answer_choose(
        P0,
        &[
            Entity::Object(shock),
            Entity::Object(bolt),
            Entity::Object(forest),
            Entity::Object(bears),
        ],
    );
    t.answer_choose(P1, &[Entity::Object(bolt), Entity::Object(bears)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(graveyard_names(&t, P0), ["Gifts Ungiven", "Grizzly Bears", "Lightning Bolt"]);
    assert_eq!(hand_names(&t, P0), ["Forest", "Shock"]);
    assert_eq!(library_sorted(&t, P0), ["Island", "Shock"]);
}

#[test]
fn gifts_ungiven_with_two_cards_found_both_go_to_the_graveyard() {
    cr!("701.23a");
    ruling!(
        "Gifts Ungiven",
        "If you find one or two cards, your opponent must choose for them to be put into your graveyard"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let ids = library(&mut t, P0, &["Shock", "Lightning Bolt", "Forest"]);
    let spell = t.hand(P0, "Gifts Ungiven");
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    // The opponent tries to choose none.
    t.answer_choose(P1, &[]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(
        graveyard_names(&t, P0),
        ["Gifts Ungiven", "Lightning Bolt", "Shock"]
    );
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn realms_uncharted_lands_with_different_names() {
    cr!("701.23a", "201.2");
    assert_supported("Realms Uncharted");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let ids = library(&mut t, P0, &["Forest", "Forest", "Island", "Swamp"]);
    let spell = t.hand(P0, "Realms Uncharted");
    // Two Forests can't both be found: only one is kept.
    t.answer_choose(
        P0,
        &[
            Entity::Object(ids[0]),
            Entity::Object(ids[1]),
            Entity::Object(ids[2]),
            Entity::Object(ids[3]),
        ],
    );
    t.answer_choose(P1, &[Entity::Object(ids[2]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(graveyard_names(&t, P0), ["Island", "Realms Uncharted", "Swamp"]);
    assert_eq!(hand_names(&t, P0), ["Forest"]);
    assert_eq!(library_sorted(&t, P0), ["Forest"]);
}

#[test]
fn threats_undetected_creatures_with_different_powers() {
    cr!("701.23a");
    ruling!(
        "Threats Undetected",
        "A creature card has different power from another if their powers are different numbers"
    );
    assert_supported("Threats Undetected");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let ids = library(
        &mut t,
        P0,
        &["Grizzly Bears", "Runeclaw Bear", "Llanowar Elves", "Serra Angel"],
    );
    let (bears, runeclaw, elves, angel) = (ids[0], ids[1], ids[2], ids[3]);
    let spell = t.hand(P0, "Threats Undetected");
    // The two 2-power Bears can't both be found.
    t.answer_choose(
        P0,
        &[
            Entity::Object(bears),
            Entity::Object(runeclaw),
            Entity::Object(elves),
            Entity::Object(angel),
        ],
    );
    t.answer_choose(P1, &[Entity::Object(elves), Entity::Object(angel)]);
    t.cast(P0, spell).go();
    t.resolve();
    // The chosen cards are shuffled into the library; the rest go to hand.
    assert_eq!(hand_names(&t, P0), ["Grizzly Bears"], "{}", t.dump_log());
    assert_eq!(
        library_sorted(&t, P0),
        ["Llanowar Elves", "Runeclaw Bear", "Serra Angel"]
    );
}

#[test]
fn elemental_teachings_rest_onto_the_battlefield_tapped() {
    cr!("701.23a");
    ruling!(
        "Elemental Teachings",
        "If you find one or two cards, your opponent must choose for them to be put into your graveyard."
    );
    assert_supported("Elemental Teachings");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let ids = library(
        &mut t,
        P0,
        &["Lightning Bolt", "Forest", "Island", "Swamp", "Mountain"],
    );
    let spell = t.hand(P0, "Elemental Teachings");
    t.answer_choose(
        P0,
        &[
            Entity::Object(ids[1]),
            Entity::Object(ids[2]),
            Entity::Object(ids[3]),
            Entity::Object(ids[4]),
        ],
    );
    t.answer_choose(P1, &[Entity::Object(ids[2]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(
        graveyard_names(&t, P0),
        ["Elemental Teachings", "Island", "Swamp"]
    );
    for land in ["Mountain"] {
        let on = t.named_on_battlefield(land);
        assert_eq!(on.len(), 1, "{land}");
        assert!(t.obj_now(on[0]).tapped);
    }
    assert_eq!(t.named_on_battlefield("Forest").len(), 6);
    assert_eq!(library_sorted(&t, P0), ["Lightning Bolt"]);
}

#[test]
fn ecological_appreciation_with_two_found_nothing_enters() {
    cr!("701.23a");
    ruling!(
        "Ecological Appreciation",
        "If you find and reveal one or two cards, no creatures will be put onto the battlefield."
    );
    assert_supported("Ecological Appreciation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let ids = library(&mut t, P0, &["Shock", "Grizzly Bears", "Llanowar Elves"]);
    let spell = t.hand(P0, "Ecological Appreciation");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[2])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    assert_eq!(
        library_sorted(&t, P0),
        ["Grizzly Bears", "Llanowar Elves", "Shock"]
    );
}

#[test]
fn ecological_appreciation_with_four_found_two_enter() {
    cr!("701.23a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let ids = library(
        &mut t,
        P0,
        &["Runeclaw Bear", "Grizzly Bears", "Llanowar Elves", "Raging Goblin"],
    );
    let spell = t.hand(P0, "Ecological Appreciation");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let all: Vec<Entity> = ids.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &all);
    t.answer_choose(P1, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Raging Goblin").len(), 1);
    assert_eq!(library_sorted(&t, P0), ["Grizzly Bears", "Runeclaw Bear"]);
    assert!(t.in_exile("Ecological Appreciation"));
}

#[test]
fn burning_rune_demon_opponent_chooses_which_goes_to_hand() {
    cr!("701.23a");
    ruling!(
        "Burning-Rune Demon",
        "If you choose not to search your library, no cards are revealed or put anywhere else"
    );
    assert_supported("Burning-Rune Demon");
    let mut t = TestGame::new(2);
    let ids = library(&mut t, P0, &["Shock", "Lightning Bolt", "Forest"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.answer_choose(P1, &[Entity::Object(ids[0])]);
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    assert_eq!(hand_names(&t, P0), ["Shock"], "{}", t.dump_log());
    assert_eq!(graveyard_names(&t, P0), ["Lightning Bolt"]);
    assert_eq!(library_sorted(&t, P0), ["Forest"]);

    // Declining the search: nothing moves.
    let mut t = TestGame::new(2);
    library(&mut t, P0, &["Shock", "Lightning Bolt", "Forest"]);
    t.answer_yes(P0, false);
    t.enter(P0, "Burning-Rune Demon");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), 3);
}

#[test]
fn myriad_landscape_two_basic_lands_that_share_a_land_type() {
    cr!("701.23a", "205.3i");
    ruling!(
        "Myriad Landscape",
        "You can choose to find one basic land card with Myriad Landscape"
    );
    assert_supported("Myriad Landscape");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Myriad Landscape");
    t.lands(P0, "Plains", 2);
    let ids = library(&mut t, P0, &["Forest", "Island", "Forest"]);
    // A Forest and an Island don't share a land type: only the Forest is kept.
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.activate(P0, land, 1, &[]).unwrap();
    t.resolve_all();
    let forests = t.named_on_battlefield("Forest");
    assert_eq!(forests.len(), 1);
    assert!(t.obj_now(forests[0]).tapped);
    assert!(t.named_on_battlefield("Island").is_empty());
    assert_eq!(library_sorted(&t, P0), ["Forest", "Island"]);
}

#[test]
fn doubling_chant_searches_for_each_creature_by_its_current_name() {
    cr!("701.23a", "201.2");
    ruling!(
        "Doubling Chant",
        "you’ll search for a card with the new name, not one with the original name"
    );
    ruling!("Doubling Chant", "The cards you find in your library must be creature cards");
    assert_supported("Doubling Chant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    // A Clone copying a Grizzly Bears is named Grizzly Bears.
    let clone = t.battlefield(P0, "Clone");
    let bears_src = t.battlefield(P1, "Grizzly Bears");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.set_var(9001, vec![Entity::Object(clone)]);
    ctx.set_var(9002, vec![Entity::Object(bears_src)]);
    mtg_engine::copy::become_copy(
        &mut t.g,
        &Sel::Var(9001),
        &Sel::Var(9002),
        &Duration::EndOfTurn,
        &[],
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    library(
        &mut t,
        P0,
        &["Grizzly Bears", "Grizzly Bears", "Llanowar Elves", "Clone", "Forest"],
    );
    let spell = t.hand(P0, "Doubling Chant");
    t.cast(P0, spell).go();
    t.resolve();
    // Two Grizzly Bears (one for the Bears, one for the Clone) and a Llanowar Elves.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 5);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 2);
    assert_eq!(library_sorted(&t, P0), ["Clone", "Forest"]);
}

#[test]
fn doubling_chant_with_no_creatures_still_shuffles() {
    cr!("701.24a");
    ruling!(
        "Doubling Chant",
        "If you control no creatures when Doubling Chant resolves"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let names = [
        "Lightning Bolt", "Shock", "Forest", "Island", "Swamp", "Mountain", "Plains",
        "Grizzly Bears", "Llanowar Elves", "Divination", "Sol Ring", "Ornithopter",
        "Hill Giant", "Raging Goblin", "Runeclaw Bear", "Giant Growth", "Counterspell",
        "Duress", "Opt", "Disenchant",
    ];
    library(&mut t, P0, &names);
    let before: Vec<ObjectId> = t.g.player(P0).library.clone();
    let spell = t.hand(P0, "Doubling Chant");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.library_size(P0), 20);
    assert_ne!(t.g.player(P0).library, before);
}

#[test]
fn dichotomancy_takes_cards_named_like_the_tapped_permanents() {
    cr!("701.23a", "110.2a");
    assert_supported("Dichotomancy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.battlefield(P1, "Llanowar Elves");
    library(&mut t, P1, &["Grizzly Bears", "Llanowar Elves", "Shock"]);
    let spell = t.hand(P0, "Dichotomancy");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    let mine: Vec<ObjectId> = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .filter(|o| t.obj_now(*o).controller == P0)
        .collect();
    assert_eq!(mine.len(), 1);
    // The untapped Elves' name wasn't searched for.
    assert_eq!(library_sorted(&t, P1), ["Llanowar Elves", "Shock"]);
}

#[test]
fn winds_of_abandon_the_controller_searches_and_puts_the_land_onto_the_battlefield() {
    cr!("701.23a", "110.2a");
    assert_supported("Winds of Abandon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    library(&mut t, P1, &["Shock", "Forest", "Lightning Bolt"]);
    let spell = t.hand(P0, "Winds of Abandon");
    t.cast(P0, spell).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    let forest: Vec<ObjectId> = t
        .named_on_battlefield("Forest")
        .into_iter()
        .filter(|o| t.obj_now(*o).controller == P1)
        .collect();
    assert_eq!(forest.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(forest[0]).tapped);
    assert_eq!(library_sorted(&t, P1), ["Lightning Bolt", "Shock"]);
}

#[test]
fn signal_the_clans_three_different_names_one_at_random_to_hand() {
    cr!("701.23a", "201.2");
    assert_supported("Signal the Clans");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let ids = library(
        &mut t,
        P0,
        &["Grizzly Bears", "Llanowar Elves", "Raging Goblin", "Shock"],
    );
    let all: Vec<Entity> = ids[..3].iter().map(|o| Entity::Object(*o)).collect();
    let spell = t.hand(P0, "Signal the Clans");
    t.answer_choose(P0, &all);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    let in_hand = t.g.obj(t.g.player(P0).hand[0]).chars.name.to_string();
    assert!(["Grizzly Bears", "Llanowar Elves", "Raging Goblin"].contains(&in_hand.as_str()));
    assert_eq!(t.library_size(P0), 3);

    // Two of them share a name: none goes to hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let ids = library(
        &mut t,
        P0,
        &["Grizzly Bears", "Grizzly Bears", "Raging Goblin", "Shock"],
    );
    let all: Vec<Entity> = ids[..3].iter().map(|o| Entity::Object(*o)).collect();
    let spell = t.hand(P0, "Signal the Clans");
    t.answer_choose(P0, &all);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.library_size(P0), 4);
}
