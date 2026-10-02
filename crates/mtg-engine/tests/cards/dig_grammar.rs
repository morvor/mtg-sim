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

#[test]
fn zimone_s_experiment_puts_the_revealed_lands_and_creatures_where_they_go() {
    cr!("701.20a", "701.20b");
    assert_supported("Zimone's Experiment");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    // Top first: Island, Grizzly Bears, Shock, Forest, Llanowar Elves.
    let ids = stack(
        &mut t,
        P0,
        &["Llanowar Elves", "Forest", "Shock", "Grizzly Bears", "Island"],
    );
    let spell = t.hand(P0, "Zimone's Experiment");
    // Reveal the Island and the Grizzly Bears; the Forest and Llanowar Elves aren't
    // revealed and go to the bottom with the Shock.
    t.answer_choose(P0, &[Entity::Object(ids[4]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    let island = t.named_on_battlefield("Island");
    assert_eq!(island.len(), 1);
    assert!(t.obj_now(island[0]).tapped);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // The cards not revealed stayed in the library (on the bottom), even the land and
    // the creature among them.
    assert!(!t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 4);
    let mut bottom: Vec<String> = t.g.player(P0).library[..3]
        .iter()
        .map(|c| name_of(&t, *c))
        .collect();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Llanowar Elves", "Shock"]);
}

#[test]
fn winding_way_takes_the_cards_of_the_chosen_kind() {
    cr!("701.20a");
    assert_supported("Winding Way");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    stack(&mut t, P0, &["Forest", "Grizzly Bears", "Island", "Llanowar Elves"]);
    let spell = t.hand(P0, "Winding Way");
    // "Choose creature or land": land.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Island") && t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn inscribed_tablet_draws_only_without_a_land() {
    cr!("608.2c");
    ruling!(
        "Inscribed Tablet",
        "If you reveal one or more land cards with Inscribed Tablet’s first ability, you have to put one of them into your hand."
    );
    assert_supported("Inscribed Tablet");
    for land in [true, false] {
        let mut t = TestGame::new(2);
        let tablet = t.battlefield(P0, "Inscribed Tablet");
        t.lands(P0, "Plains", 1);
        let top = if land { "Forest" } else { "Shock" };
        stack(&mut t, P0, &["Grizzly Bears", top, "Lightning Bolt", "Duress", "Ornithopter"]);
        let hand = t.hand_size(P0);
        // No answer: a land must be taken all the same.
        t.activate(P0, tablet, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(t.in_hand(P0, "Forest"), land);
        // A land, or a card drawn instead (a filler: the five revealed cards went to
        // the bottom).
        assert_eq!(t.hand_size(P0), hand + 1);
        let in_hand_names: Vec<String> =
            t.g.player(P0).hand.iter().map(|c| name_of(&t, *c)).collect();
        assert_eq!(in_hand_names.contains(&"Filler".to_string()), !land);
    }
}

#[test]
fn town_greeter_gains_life_only_for_a_town() {
    cr!("608.2c");
    assert_supported("Town Greeter");
    for town in [true, false] {
        let mut t = TestGame::new(2);
        let land = if town { "Capital City" } else { "Forest" };
        stack(&mut t, P0, &["Shock", "Grizzly Bears", land, "Island"]);
        t.enter(P0, "Town Greeter");
        t.settle();
        // The milled cards are new objects, top first: the land is the second.
        let milled = ObjectId(t.g.objects.len() as u32 + 1);
        t.answer_choose(P0, &[Entity::Object(milled)]);
        t.resolve_all();
        assert!(t.in_hand(P0, land), "{land}");
        assert_eq!(t.life(P0), if town { 22 } else { 20 });
    }
}

#[test]
fn kaalia_zenith_seeker_one_card_of_each_type() {
    cr!("608.2c");
    ruling!(
        "Kaalia, Zenith Seeker",
        "If a card has more than one of these types, you choose which type it counts as."
    );
    assert_supported("Kaalia, Zenith Seeker");
    let mut t = TestGame::new(2);
    // Two Angels and a Dragon.
    let ids = stack(
        &mut t,
        P0,
        &["Serra Angel", "Shock", "Shivan Dragon", "Baneslayer Angel", "Forest", "Island"],
    );
    t.answer_choose(
        P0,
        &[
            Entity::Object(ids[0]),
            Entity::Object(ids[3]),
            Entity::Object(ids[2]),
        ],
    );
    t.enter(P0, "Kaalia, Zenith Seeker");
    t.resolve_all();
    // Only one Angel: the first one chosen.
    assert!(t.in_hand(P0, "Serra Angel"));
    assert!(!t.in_hand(P0, "Baneslayer Angel"));
    assert!(t.in_hand(P0, "Shivan Dragon"));
    assert_eq!(library_names(&t, P0)[0], "Filler");
}

#[test]
fn tezzeret_s_gatebreaker_reveals_a_blue_or_artifact_card() {
    cr!("701.20a");
    assert_supported("Tezzeret's Gatebreaker");
    let mut t = TestGame::new(2);
    let ids = stack(
        &mut t,
        P0,
        &["Ornithopter", "Shock", "Counterspell", "Grizzly Bears", "Forest"],
    );
    t.answer_choose(P0, &[Entity::Object(ids[2])]);
    t.enter(P0, "Tezzeret's Gatebreaker");
    t.resolve_all();
    let mut offered = last_choice_candidates(&t);
    offered.sort();
    let mut expected = vec![Entity::Object(ids[0]), Entity::Object(ids[2])];
    expected.sort();
    assert_eq!(offered, expected);
    assert!(t.in_hand(P0, "Counterspell"));
}

#[test]
fn eye_of_yawgmoth_one_into_hand_and_the_rest_exiled() {
    cr!("701.20a");
    assert_supported("Eye of Yawgmoth");
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Yawgmoth");
    t.lands(P0, "Swamp", 3);
    // A 2/2: two cards are revealed.
    let bear = t.battlefield(P0, "Grizzly Bears");
    let ids = stack(&mut t, P0, &["Forest", "Shock", "Island"]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.activate(P0, eye, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Shock"));
    assert!(t.in_exile("Island"));
    assert_eq!(library_names(&t, P0)[0], "Forest");
}

#[test]
fn lurking_predators_creature_onto_the_battlefield_otherwise_maybe_the_bottom() {
    cr!("701.20a");
    ruling!(
        "Lurking Predators",
        "If it's not a creature card and you don't put it on the bottom of your library"
    );
    assert_supported("Lurking Predators");
    for (top, bottom) in [("Grizzly Bears", false), ("Shock", true), ("Shock", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lurking Predators");
        let card = t.library_top(P0, top);
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.answer_yes(P0, bottom);
        t.answer_targets(P1, &[Entity::Player(P0)]);
        t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
        t.resolve();
        if top == "Grizzly Bears" {
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
        } else if bottom {
            assert_eq!(t.g.player(P0).library[0], card);
        } else {
            assert_eq!(t.g.player(P0).library.last(), Some(&card));
        }
    }
}

#[test]
fn epiphany_at_the_drownyard_the_opponent_may_choose_an_empty_pile() {
    cr!("700.3a");
    ruling!(
        "Epiphany at the Drownyard",
        "If X is 0, you’ll reveal one card and one pile will be empty"
    );
    assert_supported("Epiphany at the Drownyard");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.library_top(P0, "Shock");
    let spell = t.hand(P0, "Epiphany at the Drownyard");
    // P0 puts the Shock into the second pile; the opponent chooses the first (empty) one.
    t.answer_choose(P0, &[]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, spell).x(0).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Shock"));
}

#[test]
fn dimir_charm_puts_one_back_and_mills_the_rest() {
    cr!("701.20e");
    assert_supported("Dimir Charm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let ids = stack(&mut t, P1, &["Forest", "Shock", "Island"]);
    let charm = t.hand(P0, "Dimir Charm");
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.cast(P0, charm).modes(&[2]).target(P1).go();
    t.resolve();
    assert_eq!(library_names(&t, P1)[0], "Shock");
    assert!(t.in_graveyard(P1, "Forest") && t.in_graveyard(P1, "Island"));
}

#[test]
fn dihada_makes_a_treasure_for_each_card_put_into_the_graveyard() {
    cr!("608.2c");
    assert_supported("Dihada, Binder of Wills");
    let mut t = TestGame::new(2);
    let dihada = t.battlefield(P0, "Dihada, Binder of Wills");
    let ids = stack(
        &mut t,
        P0,
        &["Forest", "Isamaru, Hound of Konda", "Shock", "Grizzly Bears"],
    );
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.activate(P0, dihada, 1, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Isamaru, Hound of Konda"));
    assert_eq!(t.graveyard_size(P0), 3);
    let treasures = t
        .g
        .permanents()
        .filter(|o| o.chars.subtypes.iter().any(|s| s == "Treasure"))
        .count();
    assert_eq!(treasures, 3);
}

#[test]
fn dream_pillager_may_cast_spells_from_among_the_exiled_cards() {
    cr!("510.2");
    ruling!("Dream Pillager", "Any cards you don't cast will remain exiled.");
    assert_supported("Dream Pillager");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Dream Pillager");
    stack(&mut t, P0, &["Forest", "Shock", "Lightning Bolt", "Island", "Grizzly Bears"]);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    // Four damage: four cards exiled, and spells among them may be cast this turn.
    assert_eq!(t.g.exile.len(), 4);
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t
        .g
        .exile
        .iter()
        .copied()
        .find(|c| name_of(&t, *c) == "Lightning Bolt")
        .unwrap();
    assert!(t.cast_with(P0, bolt, &[Entity::Player(P1)]).is_ok());
    // A land among them can't be played this way.
    let island = t
        .g
        .exile
        .iter()
        .copied()
        .find(|c| name_of(&t, *c) == "Island")
        .unwrap();
    assert!(t.play_land(P0, island).is_err());
}

#[test]
fn guided_passage_the_opponent_chooses_one_card_of_each_kind() {
    cr!("701.20a");
    ruling!(
        "Guided Passage",
        "If you have no cards of any of the specified card types, then ignore those types and the opponent only selects cards of the types you do have."
    );
    assert_supported("Guided Passage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.g.players[0].library.clear();
    // No noncreature, nonland card: only a creature and a land are chosen.
    let ids = stack(&mut t, P0, &["Grizzly Bears", "Forest", "Llanowar Elves", "Island"]);
    let spell = t.hand(P0, "Guided Passage");
    t.answer_choose(P1, &[Entity::Object(ids[2]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    // The opponent chose (among all four).
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert!(t.in_hand(P0, "Llanowar Elves") && t.in_hand(P0, "Island"));
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn heretic_s_punishment_deals_the_greatest_mana_value_among_the_milled_cards() {
    cr!("701.17a");
    ruling!(
        "Heretic's Punishment",
        "If you have two or fewer cards in your library when the ability resolves, all of them will be put into your graveyard."
    );
    assert_supported("Heretic's Punishment");
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Heretic's Punishment");
    t.lands(P0, "Mountain", 4);
    t.g.players[0].library.clear();
    stack(&mut t, P0, &["Shivan Dragon", "Shock"]);
    t.activate(P0, hp, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.life(P1), 14, "Shivan Dragon's mana value");
}

#[test]
fn elixir_gains_life_for_each_card_shuffled_away() {
    cr!("701.24a");
    assert_supported("Elixir");
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P0, "Elixir");
    t.lands(P0, "Plains", 5);
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.activate(P0, elixir, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22, "two nonland cards");
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn god_eternal_bontu_goes_third_from_the_top() {
    cr!("401.7");
    assert_supported("God-Eternal Bontu");
    let mut t = TestGame::new(2);
    let bontu = t.battlefield(P0, "God-Eternal Bontu");
    t.answer_yes(P0, true);
    t.g.destroy(bontu, None);
    t.resolve_all();
    assert_eq!(library_names(&t, P0)[2], "God-Eternal Bontu");
}

#[test]
fn archaic_s_agony_exiles_as_many_cards_as_the_excess_damage() {
    cr!("120.10");
    assert_supported("Archaic's Agony");
    let mut t = TestGame::new(2);
    for l in ["Mountain", "Forest", "Island", "Swamp", "Plains"] {
        t.lands(P0, l, 1);
    }
    let bear = t.battlefield(P1, "Grizzly Bears");
    stack(&mut t, P0, &["Forest", "Shock", "Lightning Bolt", "Island"]);
    let spell = t.hand(P0, "Archaic's Agony");
    // Five colors: five damage to a 2/2, three excess.
    t.cast(P0, spell).target(bear).go();
    t.resolve();
    assert_eq!(t.g.exile.len(), 3);
    assert_eq!(library_names(&t, P0)[0], "Forest");
}

#[test]
fn pact_weapon_reveals_the_card_drawn() {
    cr!("701.20a");
    ruling!(
        "Pact Weapon",
        "you will draw a card, reveal it, and lose life"
    );
    assert_supported("Pact Weapon");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let weapon = t.battlefield(P0, "Pact Weapon");
    t.g.attach(weapon, Entity::Object(bear));
    t.library_top(P0, "Shivan Dragon");
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Shivan Dragon"));
    // +6/+6 for Shivan Dragon's mana value: 8 damage; 6 life lost.
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn getaway_barrel_puts_a_random_creature_onto_the_battlefield() {
    cr!("701.20a");
    assert_supported("Getaway Barrel");
    let mut t = TestGame::new(2);
    let barrel = t.battlefield(P0, "Getaway Barrel");
    stack(&mut t, P0, &["Grizzly Bears", "Shock", "Forest"]);
    t.g.destroy(barrel, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // No choice was offered.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn dakra_mystic_mills_the_revealed_cards_or_everyone_draws() {
    cr!("701.20a");
    assert_supported("Dakra Mystic");
    for mill in [true, false] {
        let mut t = TestGame::new(2);
        let mystic = t.battlefield(P0, "Dakra Mystic");
        t.lands(P0, "Island", 1);
        t.library_top(P0, "Shock");
        t.library_top(P1, "Forest");
        t.answer_yes(P0, mill);
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        t.activate(P0, mystic, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(t.in_graveyard(P0, "Shock"), mill);
        assert_eq!(t.in_graveyard(P1, "Forest"), mill);
        assert_eq!(t.in_hand(P0, "Shock"), !mill);
        assert_eq!(t.hand_size(P1), h1 + usize::from(!mill));
        let _ = h0;
    }
}

#[test]
fn psychic_surgery_exiles_one_of_the_top_two_after_an_opponent_shuffles() {
    cr!("701.24a");
    ruling!(
        "Psychic Surgery",
        "You look at the top two cards of that library as the triggered ability resolves."
    );
    assert_supported("Psychic Surgery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Surgery");
    // P1 shuffles (a search with nothing found), then the trigger resolves.
    t.g.shuffle_library(P1);
    t.settle();
    let top: Vec<ObjectId> = t.g.player(P1).library.iter().rev().take(2).copied().collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(top[1])]);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1);
    assert!(!t.g.is_live(top[1]));
    assert_eq!(t.g.player(P1).library.last(), Some(&top[0]));
}

#[test]
fn break_out_puts_a_cheap_creature_onto_the_battlefield_or_into_hand() {
    cr!("701.20a", "608.2c");
    assert_supported("Break Out");
    for (creature, onto) in [("Grizzly Bears", true), ("Grizzly Bears", false), ("Shivan Dragon", true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Forest", 1);
        let ids = stack(&mut t, P0, &["Shock", creature, "Forest"]);
        let spell = t.hand(P0, "Break Out");
        t.answer_choose(P0, &[Entity::Object(ids[1])]);
        t.answer_yes(P0, onto);
        t.cast(P0, spell).go();
        t.resolve();
        let cheap = creature == "Grizzly Bears";
        let on_battlefield = t.named_on_battlefield(creature).len() == 1;
        assert_eq!(on_battlefield, cheap && onto, "{creature} {onto}");
        assert_eq!(t.in_hand(P0, creature), !(cheap && onto), "{creature} {onto}");
        assert_eq!(library_names(&t, P0)[0], "Filler");
    }
}

#[test]
fn flow_state_takes_two_with_an_instant_and_a_sorcery_in_the_graveyard() {
    cr!("608.2c");
    assert_supported("Flow State");
    for both in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2);
        t.graveyard(P0, "Shock");
        if both {
            t.graveyard(P0, "Divination");
        }
        let ids = stack(&mut t, P0, &["Forest", "Grizzly Bears", "Island"]);
        let spell = t.hand(P0, "Flow State");
        if both {
            t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[2])]);
        } else {
            t.answer_choose(P0, &[Entity::Object(ids[1])]);
        }
        t.cast(P0, spell).go();
        t.resolve();
        assert!(t.in_hand(P0, "Grizzly Bears"));
        assert_eq!(t.in_hand(P0, "Island"), both);
        assert_eq!(library_names(&t, P0)[0], "Filler");
    }
}

#[test]
fn mass_polymorph_reveals_as_many_creatures_as_were_exiled() {
    cr!("701.20a", "701.24a");
    assert_supported("Mass Polymorph");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    stack(
        &mut t,
        P0,
        &["Shivan Dragon", "Serra Angel", "Forest", "Ornithopter", "Shock"],
    );
    let spell = t.hand(P0, "Mass Polymorph");
    t.cast(P0, spell).go();
    t.resolve();
    // Two creatures exiled: the first two creature cards revealed enter.
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
    assert!(t.named_on_battlefield("Shivan Dragon").is_empty());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

/// Cards the dig grammar made fully supported (each with its dig, library position or
/// shuffle text compiled); the families are exercised by the tests above.
#[test]
fn dig_grammar_cards_are_supported() {
    for name in [
        "In the Presence of Ages",
        "Gurmag Nightwatch",
        "Ostrich-Horse",
        "Virtue of Courage // Embereth Blaze",
        "Ugin, the Ineffable",
        "Xenagos, the Reveler",
        "Tezzeret, Master of the Bridge",
        "Oko, Lorwyn Liege // Oko, Shadowmoor Scion",
        "Dihada, Binder of Wills",
        "Kiora, Master of the Depths",
        "Nissa, Nature's Artisan",
        "Conspicuous Snoop",
        "Skill Borrower",
        "Alrund, God of the Cosmos // Hakka, Whispering Raven",
        "Birthing Ritual",
        "Shadow Kin",
        "Delver of Secrets // Insectile Aberration",
        "Quest for Ula's Temple",
        "Vigean Intuition",
        "For the Ancestors",
        "Winding Way",
        "Dimir Charm",
        "Scout the City",
        "Erratic Mutation",
        "Archaic's Agony",
        "Skyserpent Seeker",
        "Venture Forth",
        "Rowan's Grim Search",
        "Follow the Lumarets",
        "Nissa, Resurgent Animist",
        "Sealed Fate",
        "Kamahl's Druidic Vow",
        "Cruel Fate",
        "Ransack",
        "Genesis Ultimatum",
        "Blex, Vexing Pest // Search for Blex",
        "Forging the Anchor",
        "Lead the Stampede",
        "Zimone's Experiment",
        "Gift of the Gargantuan",
        "Uncovered Clues",
        "Dimir Machinations",
        "Through the Forest Gate",
        "Thunderous Debut",
        "Stargaze",
        "Most Decrepit Old Bird // Speak Secrets",
        "Thranduil, Sindarin Liege // Silvan Rally",
        "Midnight Tilling",
        "Picklock Prankster // Free the Fae",
        "Something Worth Saving",
        "Vastlands Scavenger // Bind to Life",
        "Bramble Familiar // Fetch Quest",
        "Beluna Grandsquall // Seek Thrills",
        "Glamdring, Foe-hammer // Gleam of Death",
        "Gutless Plunderer",
        "Munda, Ambush Leader",
        "Genesis Storm",
        "Fathom Trawl",
        "Guided Passage",
        "Epiphany at the Drownyard",
        "Earth's Mightiest Heroes",
        "Truth or Tale",
        "Pieces of the Puzzle",
        "Benefaction of Rhonas",
        "Tracker's Instincts",
        "Kruphix's Insight",
        "Wakanda Forever!",
        "Bounty of Skemfar",
        "Aspiring Champion",
        "Creative Technique",
        "Unexpected Results",
        "Serene Remembrance",
        "The Ring Goes South",
        "Duelist's Flame",
        "Whiskervale Forerunner",
        "Aurora Awakener",
        "God-Eternal Bontu",
        "Definitely Not a Turtle",
        "Treasure Keeper",
        "Gamekeeper",
        "Gyruda, Doom of Depths",
        "Thicket Elemental",
        "Acclaimed Contender",
        "Tezzeret's Gatebreaker",
        "Overgrown Pest",
        "Karumonix, the Rat King",
        "Silhana Wayfinder",
        "Kaslem's Stonetree // Kaslem's Strider",
        "Kaalia, Zenith Seeker",
        "Ancestral Knowledge",
        "Cantankerous Keepers",
        "Lluwen, Imperfect Naturalist",
        "Town Greeter",
        "Cavalier of Thorns",
        "Brass Herald",
        "Marina Vendrell",
        "Torsten, Founder of Benalia",
        "Muxus, Goblin Grandee",
        "Adéwalé, Breaker of Chains",
        "Fertile Thicket",
        "Planar Atlas",
        "Getaway Barrel",
        "Lost in the Woods",
        "Winota, Joiner of Forces",
        "Lurking Predators",
        "Psychic Surgery",
        "Spinner of Souls",
        "Pact Weapon",
        "The Key to the Vault",
        "Choco, Seeker of Paradise",
        "Neera, Wild Mage",
        "Arthur, Marigold Knight",
        "Coral Fighters",
        "Jet, Rebel Leader",
        "Harper Recruiter",
        "Six",
        "Fireflux Squad",
        "Ardent Dustspeaker",
        "Bristlebud Farmer",
        "Doomskar Warrior",
        "Garruk's Harbinger",
        "Dream Pillager",
        "Lord of the Void",
        "Nine-Fingers Keene",
        "Mole Module",
        "Gishath, Sun's Avatar",
        "Ojer Kaslem, Deepest Growth // Temple of Cultivation",
        "Barrowgoyf",
        "Rampant Frogantua",
        "Glint Raker",
        "Eivor, Wolf-Kissed",
        "Nashi, Searcher in the Dark",
        "Avenging Druid",
        "Sludge Titan",
        "The Fifteenth Doctor",
        "Feldon, Ronom Excavator",
        "Auspicious Starrix",
        "Vorinclex // The Grand Evolution",
        "Ballad of the Black Flag",
        "Elixir",
        "Eye of Yawgmoth",
        "Inscribed Tablet",
        "Rick Jones, Destined Sidekick",
        "Elixir of Immortality",
        "Urza, Lord High Artificer",
        "Eerie Gravestone",
        "Bookwurm",
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Heretic's Punishment",
        "Goblin Machinist",
        "Kogla and Yidaro",
        "Void Stalker",
        "Floodpits Drowner",
        "Screaming Swarm",
        "Orcish Librarian",
        "Dakra Mystic",
        "Break Out",
        "Nick Fury, Agent of S.H.I.E.L.D.",
        "Flow State",
        "Mass Polymorph",
    ] {
        assert_supported(name);
    }
}
