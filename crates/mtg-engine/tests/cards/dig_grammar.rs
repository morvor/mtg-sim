//! The dig grammar (`oracle/patterns/dig_grammar.rs`, `dig_steps.rs`): cards looked at,
//! revealed, milled or exiled from the top of a library, then chosen among ("from among
//! them", "and/or" lists, "at random", "revealed this way") and distributed, with "the
//! rest" going somewhere; revealing until several cards of a kind are revealed; piles;
//! library positions.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::turn::Step;
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

/// Stacks named cards on top of `p`'s library; the last one named ends up on top.
fn stack(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.library_top(p, n)).collect()
}

fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.g.obj(id).chars.name.to_string()
}

/// `p`'s library, top first, by name.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| name_of(t, *c))
        .collect()
}

/// Candidates offered by the most recent "choose entities" decision.
fn last_choice_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

fn asked_order(t: &TestGame) -> bool {
    t.asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::Order { .. }))
}

#[test]
fn in_the_presence_of_ages_a_creature_and_a_land_rest_into_the_graveyard() {
    cr!("701.20a", "608.2c");
    ruling!(
        "In the Presence of Ages",
        "you could put no cards, a creature card, a land card, or a creature card and a land card into your hand"
    );
    assert_supported("In the Presence of Ages");
    assert_supported("Kiora, Master of the Depths");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    // Top first: Shock, Llanowar Elves, Forest, Grizzly Bears.
    let ids = stack(&mut t, P0, &["Grizzly Bears", "Forest", "Llanowar Elves", "Shock"]);
    let (bears, forest, elves) = (ids[0], ids[1], ids[2]);
    let spell = t.hand(P0, "In the Presence of Ages");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(forest)]);
    t.cast(P0, spell).go();
    t.resolve();
    // Shock (neither a creature nor a land) wasn't offered.
    let offered = last_choice_candidates(&t);
    assert_eq!(offered.len(), 3);
    assert!(!offered.contains(&Entity::Object(ids[3])));
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Llanowar Elves") && t.in_graveyard(P0, "Shock"));
    let _ = elves;
    assert_eq!(library_names(&t, P0)[0], "Filler");
}

#[test]
fn in_the_presence_of_ages_two_creatures_count_as_one_creature_card() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let ids = stack(&mut t, P0, &["Grizzly Bears", "Forest", "Llanowar Elves", "Shock"]);
    let spell = t.hand(P0, "In the Presence of Ages");
    // Two creature cards: only one of them can be the creature card.
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[2])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn ostrich_horse_takes_a_milled_land_or_gets_a_counter() {
    cr!("701.17a", "608.2c");
    assert_supported("Ostrich-Horse");
    for take in [true, false] {
        let mut t = TestGame::new(2);
        stack(&mut t, P0, &["Shock", "Forest", "Grizzly Bears"]);
        let horse = t.enter(P0, "Ostrich-Horse");
        t.settle();
        // The milled cards become new objects (CR 400.7), created top first: the Forest
        // is the second.
        let forest = ObjectId(t.g.objects.len() as u32 + 1);
        if take {
            t.answer_choose(P0, &[Entity::Object(forest)]);
        } else {
            t.answer_choose(P0, &[]);
        }
        t.resolve_all();
        // Only the land among the milled cards (now in the graveyard) was offered.
        let offered = last_choice_candidates(&t);
        assert_eq!(offered.len(), 1);
        assert_eq!(t.in_hand(P0, "Forest"), take);
        assert!(t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Grizzly Bears"));
        assert_eq!(t.counters(horse, "+1/+1"), if take { 0 } else { 1 });
    }
}

#[test]
fn gurmag_nightwatch_one_back_on_top_the_rest_into_the_graveyard() {
    cr!("701.20e");
    assert_supported("Gurmag Nightwatch");
    let mut t = TestGame::new(2);
    // Top first: Shock, Forest, Grizzly Bears.
    let ids = stack(&mut t, P0, &["Grizzly Bears", "Forest", "Shock"]);
    t.answer_choose(P0, &[Entity::Object(ids[0])]);
    t.enter(P0, "Gurmag Nightwatch");
    t.resolve_all();
    assert_eq!(library_names(&t, P0)[0], "Grizzly Bears");
    // The card kept didn't change zones (CR 400.7).
    assert_eq!(t.g.player(P0).library.last(), Some(&ids[0]));
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Shock"));
}

#[test]
fn fathom_trawl_reveals_until_three_nonland_cards() {
    cr!("701.20a", "401.4");
    ruling!(
        "Fathom Trawl",
        "If there are fewer than three nonland cards in your library, you will reveal your entire library"
    );
    assert_supported("Fathom Trawl");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    // Top first: Forest, Shock, Island, Lightning Bolt, Grizzly Bears, Mountain.
    stack(
        &mut t,
        P0,
        &["Mountain", "Grizzly Bears", "Lightning Bolt", "Island", "Shock", "Forest"],
    );
    let spell = t.hand(P0, "Fathom Trawl");
    // The two lands go to the bottom: Island lowest.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.cast(P0, spell).go();
    t.resolve();
    for c in ["Shock", "Lightning Bolt", "Grizzly Bears"] {
        assert!(t.in_hand(P0, c), "{c}");
    }
    // The Mountain wasn't revealed: it's on top now.
    assert_eq!(library_names(&t, P0)[0], "Mountain");
    let lib = &t.g.player(P0).library;
    assert_eq!(name_of(&t, lib[0]), "Island");
    assert_eq!(name_of(&t, lib[1]), "Forest");
    assert!(asked_order(&t));

    // A library with fewer nonland cards: all of it is revealed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &["Forest", "Shock", "Island"]);
    let spell = t.hand(P0, "Fathom Trawl");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn vigean_intuition_takes_every_card_of_the_chosen_type() {
    cr!("701.20a");
    assert_supported("Vigean Intuition");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Forest", 1);
    stack(&mut t, P0, &["Forest", "Shock", "Island", "Lightning Bolt"]);
    let spell = t.hand(P0, "Vigean Intuition");
    let instant = mtg_engine::types::CardType::ALL
        .iter()
        .position(|c| *c == mtg_engine::types::CardType::Instant)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(instant));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Island"));
}

#[test]
fn orcish_librarian_exiles_four_at_random_and_orders_the_rest() {
    cr!("401.4");
    ruling!(
        "Orcish Librarian",
        "You do get to look at the remaining 4 cards before deciding which order to put them back in."
    );
    assert_supported("Orcish Librarian");
    let mut t = TestGame::new(2);
    let lib = t.battlefield(P0, "Orcish Librarian");
    t.lands(P0, "Mountain", 1);
    let names = [
        "Shock",
        "Forest",
        "Island",
        "Lightning Bolt",
        "Grizzly Bears",
        "Mountain",
        "Swamp",
        "Plains",
    ];
    stack(&mut t, P0, &names);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![3, 2, 1, 0]));
    t.activate(P0, lib, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.exile.len(), 4);
    // No choice of which cards to exile was asked.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    // The other four are back on top, in the order chosen, from among the eight.
    let top: Vec<String> = library_names(&t, P0)[..4].to_vec();
    assert!(top.iter().all(|n| names.contains(&n.as_str())));
    assert!(asked_order(&t));
    assert_eq!(library_names(&t, P0)[4], "Filler");
}

#[test]
fn jace_architect_of_thought_piles_into_hand_and_onto_the_bottom() {
    cr!("700.3a", "401.4");
    ruling!(
        "Jace, Architect of Thought",
        "Piles can be empty. If one of the piles is empty, you choose to put all the revealed cards in your hand or on the bottom of your library."
    );
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    let ids = stack(&mut t, P0, &["Shock", "Forest", "Grizzly Bears"]);
    // The opponent puts everything into one pile (the other is empty); P0 picks that
    // pile.
    t.answer_choose(P1, &ids.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>());
    t.activate(P0, jace, 1, &[]).unwrap();
    t.resolve_all();
    let in_hand = ["Shock", "Forest", "Grizzly Bears"]
        .iter()
        .filter(|c| t.in_hand(P0, c))
        .count();
    let on_bottom = t.g.player(P0).library[..3]
        .iter()
        .filter(|c| ids.contains(c))
        .count();
    // Every revealed card went to one place or the other.
    assert_eq!(in_hand + on_bottom, 3);
    assert_eq!(library_names(&t, P0)[0], "Filler");
}

#[test]
fn truth_or_tale_one_card_of_the_chosen_pile_the_rest_on_the_bottom() {
    cr!("700.3a");
    ruling!("Truth or Tale", "Step 2: You separate those cards into two piles");
    assert_supported("Truth or Tale");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let ids = stack(
        &mut t,
        P0,
        &["Shock", "Forest", "Grizzly Bears", "Island", "Lightning Bolt"],
    );
    let spell = t.hand(P0, "Truth or Tale");
    // P0 puts Shock and Forest in the first pile; the opponent chooses it; P0 takes the
    // Forest.
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Forest"));
    // One revealed card went to the hand, the other four to the bottom.
    let in_hand = ids.iter().filter(|c| !t.g.is_live(**c)).count();
    assert_eq!(in_hand, 1);
    let bottom4: Vec<ObjectId> = t.g.player(P0).library[..4].to_vec();
    assert!(bottom4.iter().all(|c| ids.contains(c)));
    assert_eq!(library_names(&t, P0)[0], "Filler");
}

#[test]
fn follow_the_lumarets_takes_two_only_after_gaining_life() {
    cr!("608.2c");
    assert_supported("Follow the Lumarets");
    for gained in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        if gained {
            t.g.gain_life(P0, 1);
        }
        let ids = stack(&mut t, P0, &["Shock", "Forest", "Grizzly Bears", "Island"]);
        let spell = t.hand(P0, "Follow the Lumarets");
        if gained {
            t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[2])]);
        } else {
            t.answer_choose(P0, &[Entity::Object(ids[2])]);
        }
        t.cast(P0, spell).go();
        t.resolve();
        let taken = ["Forest", "Grizzly Bears"]
            .iter()
            .filter(|c| t.in_hand(P0, c))
            .count();
        assert_eq!(taken, if gained { 2 } else { 1 });
        // The rest are on the bottom, under the fillers.
        assert_eq!(library_names(&t, P0)[0], "Filler");
    }
}

#[test]
fn search_for_blex_loses_three_life_for_each_card_taken() {
    cr!("608.2c");
    ruling!(
        "Blex, Vexing Pest // Search for Blex",
        "you may put any number of the cards into your hand"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let ids = stack(&mut t, P0, &["Shock", "Forest", "Grizzly Bears", "Island", "Swamp"]);
    let spell = t.hand(P0, "Blex, Vexing Pest // Search for Blex");
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.cast(P0, spell)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve();
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Forest"));
    assert_eq!(t.graveyard_size(P0), 4, "three cards and the spell");
    assert_eq!(t.life(P0), 14);
}

#[test]
fn delver_of_secrets_transforms_only_when_it_reveals_an_instant() {
    cr!("701.20a", "701.20b");
    ruling!(
        "Delver of Secrets // Insectile Aberration",
        "Whether or not you reveal it, the card stays on top of your library."
    );
    assert_supported("Delver of Secrets // Insectile Aberration");
    for (top, reveal, flips) in [
        ("Shock", true, true),
        ("Shock", false, false),
        ("Forest", true, false),
    ] {
        let mut t = TestGame::new(2);
        let delver = t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
        let card = t.library_top(P0, top);
        t.answer_yes(P0, reveal);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(
            t.obj_now(delver).chars.name.as_str() == "Insectile Aberration",
            flips,
            "{top} {reveal}"
        );
        assert_eq!(t.g.player(P0).library.last(), Some(&card));
    }
}

#[test]
fn sealed_fate_exiles_one_and_its_controller_orders_the_rest() {
    cr!("401.4");
    ruling!(
        "Sealed Fate",
        "The controller of this spell decides the order of the cards on the library."
    );
    assert_supported("Sealed Fate");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    let ids = stack(&mut t, P1, &["Shock", "Forest", "Grizzly Bears"]);
    let spell = t.hand(P0, "Sealed Fate");
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    // Top first after: Shock, then Grizzly Bears.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.cast(P0, spell).x(3).target(P1).go();
    t.resolve();
    assert!(t.in_exile("Forest"));
    let top = library_names(&t, P1);
    assert_eq!(&top[..2], ["Shock", "Grizzly Bears"]);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn bounty_of_skemfar_a_land_onto_the_battlefield_and_an_elf_into_hand() {
    cr!("608.2c");
    assert_supported("Bounty of Skemfar");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let ids = stack(
        &mut t,
        P0,
        &["Shock", "Forest", "Llanowar Elves", "Island", "Grizzly Bears", "Swamp"],
    );
    let spell = t.hand(P0, "Bounty of Skemfar");
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.answer_choose(P0, &[Entity::Object(ids[2])]);
    t.cast(P0, spell).go();
    t.resolve();
    let forest = t
        .named_on_battlefield("Forest")
        .into_iter()
        .find(|f| t.obj_now(*f).tapped);
    assert!(forest.is_some(), "the Forest entered tapped");
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(library_names(&t, P0)[0], "Filler");
}

#[test]
fn wakanda_forever_puts_one_permanent_with_an_indestructible_counter() {
    cr!("122.1b");
    assert_supported("Wakanda Forever!");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let ids = stack(
        &mut t,
        P0,
        &["Shock", "Forest", "Llanowar Elves", "Island", "Grizzly Bears", "Swamp"],
    );
    let spell = t.hand(P0, "Wakanda Forever!");
    t.answer_choose(P0, &[Entity::Object(ids[4])]);
    t.answer_choose(P0, &[Entity::Object(ids[2])]);
    t.cast(P0, spell).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.counters(bears[0], "indestructible"), 1);
    assert!(t.in_hand(P0, "Llanowar Elves"));
    for c in ["Shock", "Forest", "Island", "Swamp"] {
        assert!(t.in_graveyard(P0, c), "{c}");
    }
}

#[test]
fn spinner_of_souls_the_creature_found_into_hand_the_rest_on_the_bottom() {
    cr!("701.20a");
    assert_supported("Spinner of Souls");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spinner of Souls");
    let bear = t.battlefield(P0, "Grizzly Bears");
    stack(&mut t, P0, &["Llanowar Elves", "Shock", "Forest"]);
    t.answer_yes(P0, true);
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(library_names(&t, P0)[0], "Filler");
    let lib = &t.g.player(P0).library;
    let mut bottom: Vec<String> = lib[..2].iter().map(|c| name_of(&t, *c)).collect();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Shock"]);
}

#[test]
fn open_the_way_puts_the_lands_found_onto_the_battlefield_tapped() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    stack(&mut t, P0, &["Mountain", "Island", "Shock", "Swamp", "Lightning Bolt"]);
    let spell = t.hand(P0, "Open the Way");
    t.cast(P0, spell).x(2).go();
    t.resolve();
    for c in ["Swamp", "Island"] {
        let on = t.named_on_battlefield(c);
        assert_eq!(on.len(), 1, "{c}");
        assert!(t.obj_now(on[0]).tapped);
    }
    // The Mountain wasn't revealed (two lands were found before it).
    assert_eq!(library_names(&t, P0)[0], "Mountain");
    let lib = &t.g.player(P0).library;
    let mut bottom: Vec<String> = lib[..2].iter().map(|c| name_of(&t, *c)).collect();
    bottom.sort();
    assert_eq!(bottom, ["Lightning Bolt", "Shock"]);
}

#[test]
fn through_the_forest_gate_lands_onto_the_battlefield_then_shuffle() {
    cr!("701.24a");
    assert_supported("Through the Forest Gate");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 8);
    let ids = stack(&mut t, P0, &["Island", "Shock", "Swamp"]);
    let spell = t.hand(P0, "Through the Forest Gate");
    t.answer_choose(P0, &[Entity::Object(ids[0]), Entity::Object(ids[2])]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.obj_now(t.named_on_battlefield("Island")[0]).tapped);
    assert!(t.obj_now(t.named_on_battlefield("Swamp")[0]).tapped);
    assert_eq!(t.life(P0), 28);
}

#[test]
fn crown_of_convergence_moves_the_top_card_to_the_bottom() {
    cr!("400.7");
    let mut t = TestGame::new(2);
    let crown = t.battlefield(P0, "Crown of Convergence");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let top = t.library_top(P0, "Shock");
    t.activate(P0, crown, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.player(P0).library[0], top, "the same object, at the bottom");
}

#[test]
fn bookwurm_goes_third_from_the_top() {
    cr!("401.7");
    ruling!(
        "Bookwurm",
        "If you have zero or one card left in your library when Bookwurm's last ability resolves, it is put on the bottom of your library."
    );
    assert_supported("Bookwurm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let wurm = t.graveyard(P0, "Bookwurm");
    t.activate(P0, wurm, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(library_names(&t, P0)[2], "Bookwurm");

    // A one-card library: on the bottom.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.g.players[0].library.truncate(1);
    let wurm = t.graveyard(P0, "Bookwurm");
    t.activate(P0, wurm, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(name_of(&t, t.g.player(P0).library[0]), "Bookwurm");
}

#[test]
fn elixir_of_immortality_shuffles_itself_and_the_graveyard_away() {
    cr!("701.24a");
    ruling!(
        "Elixir of Immortality",
        "you’ll shuffle Elixir of Immortality into its owner’s library directly from the battlefield"
    );
    assert_supported("Elixir of Immortality");
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P0, "Elixir of Immortality");
    t.lands(P0, "Plains", 2);
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Grizzly Bears");
    let before = t.library_size(P0);
    t.activate(P0, elixir, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), before + 3);
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Library(P0));
}

#[test]
fn floodpits_drowner_shuffles_itself_and_the_stunned_creature() {
    cr!("701.24a");
    ruling!(
        "Floodpits Drowner",
        "Floodpits Drowner won't be shuffled into its owner's library"
    );
    assert_supported("Floodpits Drowner");
    let mut t = TestGame::new(2);
    let drowner = t.battlefield(P0, "Floodpits Drowner");
    t.lands(P0, "Island", 2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bear), "stun", 1, None);
    let (mine, theirs) = (t.library_size(P0), t.library_size(P1));
    t.activate(P0, drowner, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve();
    assert_eq!(t.library_size(P0), mine + 1);
    assert_eq!(t.library_size(P1), theirs + 1);
}
