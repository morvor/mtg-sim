//! The search clause grammar (CR 701.23; `oracle/patterns/search_grammar.rs`): several
//! zones ("your library and/or graveyard", "graveyard, hand, and library"), several card
//! descriptions ("a white card, a blue card, ..."), names with commas, distinct names,
//! dynamic filters ("mana value equal to 1 plus the sacrificed creature's mana value"),
//! split destinations, "third from the top", other players searching, and the sentences
//! that complete a search ("Reveal those cards, put them into your hand, then shuffle.").

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
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

fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

fn searched(t: &TestGame, p: PlayerId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::Searched { player } if *player == p))
}

/// Candidates offered by the `n`th "choose entities" decision (0 = the first).
fn choice_candidates(t: &TestGame, n: usize) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .nth(n)
        .expect("no such choice was asked")
}

fn objs(v: &[ObjectId]) -> Vec<Entity> {
    v.iter().map(|o| Entity::Object(*o)).collect()
}

#[test]
fn a_card_named_with_a_comma_from_the_library_and_or_graveyard() {
    cr!("701.23a", "701.23b");
    assert_supported("Liliana's Influence");
    // Found in the graveyard without searching the library: no shuffle, no search.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let in_yard = t.graveyard(P0, "Liliana, Death Wielder");
    let in_library = t.library_top(P0, "Liliana, Death Wielder");
    let spell = t.hand(P0, "Liliana's Influence");
    t.answer_yes(P0, true); // you may search
    t.answer_yes(P0, false); // not the library
    t.answer_choose(P0, &[Entity::Object(in_yard)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(choice_candidates(&t, 0), objs(&[in_yard]));
    assert!(t.in_hand(P0, "Liliana, Death Wielder"));
    assert_eq!(t.zone(in_library), Zone::Library(P0));
    assert_eq!(shuffles(&t, P0), 0);
    assert!(!searched(&t, P0));

    // Searching the library too: either copy can be found, and the library is shuffled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let in_yard = t.graveyard(P0, "Liliana, Death Wielder");
    let in_library = t.library_top(P0, "Liliana, Death Wielder");
    let spell = t.hand(P0, "Liliana's Influence");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(in_library)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let offered = choice_candidates(&t, 0);
    assert!(offered.contains(&Entity::Object(in_yard)));
    assert!(offered.contains(&Entity::Object(in_library)));
    assert!(t.in_hand(P0, "Liliana, Death Wielder"));
    assert!(t.in_graveyard(P0, "Liliana, Death Wielder"));
    assert_eq!(shuffles(&t, P0), 1);
    assert!(searched(&t, P0));
}

#[test]
fn several_parts_are_found_separately_and_needn_t_all_be_found() {
    cr!("701.23b");
    ruling!("Conflux", "You don't have to find all five cards.");
    ruling!(
        "Conflux",
        "you may find a white-blue card as the white card and another white-blue card as the blue card"
    );
    assert_supported("Conflux");
    let mut t = TestGame::new(2);
    for l in ["Plains", "Plains", "Plains", "Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.battlefield(P0, l);
    }
    let azorius1 = t.library_top(P0, "Azorius Guildmage"); // white and blue
    let azorius2 = t.library_top(P0, "Azorius Guildmage");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spell = t.hand(P0, "Conflux");
    // White: the first Guildmage; blue: the second; black: none; red: Bolt; green: none.
    t.answer_choose(P0, &[Entity::Object(azorius1)]);
    t.answer_choose(P0, &[Entity::Object(azorius2)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    // The blue part is offered the other white-blue card, not the one already found.
    let blue = choice_candidates(&t, 1);
    assert!(blue.contains(&Entity::Object(azorius2)));
    assert!(!blue.contains(&Entity::Object(azorius1)));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(
        t.g.player(P0)
            .hand
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Azorius Guildmage")
            .count(),
        2
    );
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn a_land_card_of_each_basic_land_type() {
    cr!("701.23b", "205.3i");
    ruling!("Gaea's Balance", "You can find a nonbasic land card this way as long as it has a basic land type");
    ruling!("Gaea's Balance", "You can fail to find a land card for any or all basic land types");
    assert_supported("Gaea's Balance");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.battlefield(P0, "Forest");
    }
    t.lands(P0, "Forest", 4);
    let stomping = t.library_top(P0, "Stomping Ground"); // Mountain Forest
    let plains = t.library_top(P0, "Plains");
    let spell = t.hand(P0, "Gaea's Balance");
    // Sacrifice five lands as the additional cost.
    let lands: Vec<ObjectId> = t.named_on_battlefield("Forest")[..5].to_vec();
    t.answer_choose(P0, &objs(&lands));
    // Plains; no Island or Swamp to find (nothing is asked); Mountain. Stomping Ground
    // could also be the Forest, but it's already found.
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.answer_choose(P0, &[Entity::Object(stomping)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Stomping Ground").len(), 1);
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
}

#[test]
fn a_card_named_forest_isn_t_any_forest() {
    cr!("701.23a", "201.2");
    ruling!(
        "Nissa's Encouragement",
        "can't be used to find a card with the land type Forest that isn't also named Forest"
    );
    ruling!("Nissa's Encouragement", "You can find any or all of the cards listed");
    assert_supported("Nissa's Encouragement");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let thicket = t.library_top(P0, "Sheltered Thicket");
    let forest = t.graveyard(P0, "Forest");
    let nissa = t.library_top(P0, "Nissa, Genesis Mage");
    let spell = t.hand(P0, "Nissa's Encouragement");
    // No Brambleweft Behemoth to find: nothing is asked for it.
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_choose(P0, &[Entity::Object(nissa)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!choice_candidates(&t, 0).contains(&Entity::Object(thicket)));
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.in_hand(P0, "Nissa, Genesis Mage"));
    assert_eq!(t.zone(thicket), Zone::Library(P0));
}

#[test]
fn cards_not_named_this_with_different_names() {
    cr!("701.23a", "201.2");
    ruling!("Tiamat", "A Dragon card is a card with the creature type Dragon in its type line.");
    assert_supported("Tiamat");
    for valid in [true, false] {
        let mut t = TestGame::new(2);
        for l in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Forest", "Forest"] {
            t.battlefield(P0, l);
        }
        let other_tiamat = t.library_top(P0, "Tiamat");
        let s1 = t.library_top(P0, "Shivan Dragon");
        let s2 = t.library_top(P0, "Shivan Dragon");
        let hellkite = t.library_top(P0, "Thundermaw Hellkite");
        let tiamat = t.hand(P0, "Tiamat");
        if valid {
            t.answer_choose(P0, &[Entity::Object(s1), Entity::Object(hellkite)]);
        } else {
            // Two cards with the same name: not valid. The default finds as many Dragon
            // cards with different names as it can.
            t.answer_choose(P0, &[Entity::Object(s1), Entity::Object(s2)]);
        }
        t.cast(P0, tiamat).go();
        t.resolve_all();
        assert!(!choice_candidates(&t, 0).contains(&Entity::Object(other_tiamat)));
        let shivans = t
            .g
            .player(P0)
            .hand
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Shivan Dragon")
            .count();
        assert_eq!(shivans, 1);
        assert!(t.in_hand(P0, "Thundermaw Hellkite"));
        assert_eq!(shuffles(&t, P0), 1);
    }
}

#[test]
fn split_destinations_one_tapped_onto_the_battlefield_and_the_other_into_hand() {
    cr!("701.23a", "701.23e");
    ruling!("Cultivate", "If you choose to find only one basic land card, you put it onto the battlefield tapped.");
    assert_supported("Cultivate");
    // Two found: the player chooses which one goes onto the battlefield.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let forest = t.library_top(P0, "Forest");
    let plains = t.library_top(P0, "Plains");
    let spell = t.hand(P0, "Cultivate");
    t.answer_choose(P0, &[Entity::Object(forest), Entity::Object(plains)]);
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let p = t.named_on_battlefield("Plains");
    assert_eq!(p.len(), 1);
    assert!(t.g.obj(p[0]).tapped);
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(shuffles(&t, P0), 1);

    // Only one found: it goes onto the battlefield tapped.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let island = t.library_top(P0, "Island");
    let spell = t.hand(P0, "Cultivate");
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let i = t.named_on_battlefield("Island");
    assert_eq!(i.len(), 1);
    assert!(t.g.obj(i[0]).tapped);
    assert!(!t.in_hand(P0, "Island"));
}

#[test]
fn a_quantity_of_cards_must_be_found_and_split() {
    cr!("701.23d");
    assert_supported("Final Parting");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Final Parting");
    // An answer finding only one card isn't valid: two cards must be found.
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_choose(P0, &[Entity::Object(bears)]); // into the hand
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    let _ = bears;
}

#[test]
fn shuffle_and_put_that_card_third_from_the_top() {
    cr!("701.24b", "701.24g");
    ruling!("Long-Term Plans", "If there are fewer than 3 cards in your library, put the card on the bottom of your library.");
    assert_supported("Long-Term Plans");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spell = t.hand(P0, "Long-Term Plans");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    // The top of the library is its last element.
    assert_eq!(lib[lib.len() - 3], bolt);
    assert_eq!(t.zone(bolt), Zone::Library(P0));
    assert_eq!(shuffles(&t, P0), 1);

    // Two cards in the library: the card goes on the bottom.
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.lands(P0, "Island", 3);
    t.library_top(P0, "Island");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spell = t.hand(P0, "Long-Term Plans");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).library[0], bolt);
}

#[test]
fn shuffle_and_put_those_cards_on_top_in_any_order() {
    cr!("701.24b");
    assert_supported("Congregation at Dawn");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    let bears = t.library_top(P0, "Grizzly Bears");
    let lions = t.library_top(P0, "Savannah Lions");
    let spell = t.hand(P0, "Congregation at Dawn");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(lions)]);
    // Lions on top, then Bears.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.cast(P0, spell).go();
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    assert_eq!(lib[lib.len() - 1], lions);
    assert_eq!(lib[lib.len() - 2], bears);
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn mana_value_equal_to_one_plus_the_sacrificed_creature_s() {
    cr!("701.23a", "202.3");
    assert_supported("Birthing Pod");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let pod = t.battlefield(P0, "Birthing Pod");
    let bears = t.battlefield(P0, "Grizzly Bears"); // mana value 2
    let three = t.library_top(P0, "Centaur Courser"); // mana value 3
    let two = t.library_top(P0, "Grizzly Bears");
    let four = t.library_top(P0, "Ravenous Baloth");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(three)]);
    t.activate(P0, pod, 0, &[]).unwrap();
    t.resolve_all();
    let offered = choice_candidates(&t, 1);
    assert_eq!(offered, vec![Entity::Object(three)]);
    assert!(!offered.contains(&Entity::Object(two)));
    assert!(!offered.contains(&Entity::Object(four)));
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
}

#[test]
fn x_or_less_where_x_is_defined_and_the_spell_s_x() {
    cr!("701.23a", "107.3c");
    ruling!(
        "Finale of Devastation",
        "If you don't find a creature card with mana value X or less, creatures you control still get +X/+X"
    );
    assert_supported("Eldritch Evolution");
    assert_supported("Finale of Devastation");
    // Eldritch Evolution: X is 2 plus the sacrificed creature's mana value.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let four = t.library_top(P0, "Ravenous Baloth");
    let five = t.library_top(P0, "Thragtusk");
    let spell = t.hand(P0, "Eldritch Evolution");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(four)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let offered = choice_candidates(&t, 1);
    assert!(offered.contains(&Entity::Object(four)));
    assert!(!offered.contains(&Entity::Object(five)));
    assert_eq!(t.named_on_battlefield("Ravenous Baloth").len(), 1);
    assert!(t.in_exile("Eldritch Evolution"));

    // Finale of Devastation with X = 10, finding nothing: creatures still get +10/+10.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 12);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Finale of Devastation");
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).x(10).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (12, 12));
}

#[test]
fn searching_another_player_s_graveyard_hand_and_library() {
    cr!("701.23a", "701.23b");
    ruling!("Memoricide", "you can opt to find all of them, none of them, or any number in between");
    assert_supported("Memoricide");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let in_hand = t.hand(P1, "Grizzly Bears");
    let in_yard = t.graveyard(P1, "Grizzly Bears");
    let in_library = t.library_top(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Memoricide");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    // Leave the one in the graveyard.
    t.answer_choose(P0, &[Entity::Object(in_hand), Entity::Object(in_library)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    let offered = choice_candidates(&t, 0);
    for o in [in_hand, in_yard, in_library] {
        assert!(offered.contains(&Entity::Object(o)));
    }
    assert_eq!(t.zone(in_yard), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(
        t.g.exile
            .iter()
            .filter(|o| t.g.obj(**o).chars.name == "Grizzly Bears")
            .count(),
        2
    );
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(shuffles(&t, P0), 0);
}

#[test]
fn all_cards_with_that_name_in_the_graveyard_are_found() {
    cr!("701.23b");
    assert_supported("Cranial Extraction");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let in_yard = t.graveyard(P1, "Grizzly Bears");
    let in_library = t.library_top(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Cranial Extraction");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    // The one in the library may be left (a hidden zone, CR 701.23b).
    t.answer_choose(P0, &[Entity::Object(in_yard)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(in_library), Zone::Library(P1));
    assert_ne!(t.zone(in_yard), Zone::Graveyard(P1));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(shuffles(&t, P1), 1);

    // An answer leaving the one in the graveyard isn't valid.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let in_yard = t.graveyard(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Cranial Extraction");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_ne!(t.zone(in_yard), Zone::Graveyard(P1));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn that_player_draws_a_card_for_each_card_exiled_from_their_hand() {
    cr!("701.23a");
    assert_supported("Lost Legacy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let h1 = t.hand(P1, "Grizzly Bears");
    let h2 = t.hand(P1, "Grizzly Bears");
    let lib = t.library_top(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Lost Legacy");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_choose(P0, &[Entity::Object(h1), Entity::Object(h2), Entity::Object(lib)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    // Three exiled, two of them from the hand: two cards drawn.
    assert_eq!(t.hand_size(P1), 2);
    assert!(t.g.player(P1).hand.iter().all(|c| t.g.obj(*c).chars.name == "Filler"));
    assert_eq!(shuffles(&t, P1), 1);
}

#[test]
fn target_player_searches_and_must_shuffle() {
    cr!("701.23a", "110.2a");
    ruling!(
        "Fertilid",
        "Although the targeted player doesn't need to find a basic land card if they don't want to, that player must shuffle their library."
    );
    assert_supported("Fertilid");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let fertilid = t.battlefield(P0, "Fertilid");
    t.g.objects[fertilid.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    let forest = t.library_top(P1, "Forest");
    // P1 finds nothing.
    t.answer_choose(P1, &[]);
    t.activate(P0, fertilid, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Library(P1));
    assert_eq!(shuffles(&t, P1), 1);
    // Found: it enters tapped under that player's control.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let fertilid = t.battlefield(P0, "Fertilid");
    t.g.objects[fertilid.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    let forest = t.library_top(P1, "Forest");
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.activate(P0, fertilid, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let f = t.named_on_battlefield("Forest");
    let theirs: Vec<&ObjectId> = f.iter().filter(|o| t.g.obj(**o).controller == P1).collect();
    assert_eq!(theirs.len(), 1);
    assert!(t.g.obj(*theirs[0]).tapped);
}

#[test]
fn any_number_of_target_players_may_each_search() {
    cr!("701.23i", "115.1");
    assert_supported("Turtle Tracks");
    let mut t = TestGame::new(3);
    t.lands(P0, "Forest", 3);
    let mine = t.library_top(P0, "Forest");
    let theirs = t.library_top(P1, "Plains");
    let other = t.library_top(P2, "Island");
    let spell = t.hand(P0, "Turtle Tracks");
    t.answer_targets(P0, &[Entity::Player(P0), Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P1, &[Entity::Object(theirs)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, spell, mtg_engine::object::CastMethod::Normal).unwrap();
    t.resolve_all();
    assert_ne!(t.zone(mine), Zone::Library(P0));
    assert_eq!(t.named_on_battlefield("Forest").len(), 4);
    let p = t.named_on_battlefield("Plains");
    assert_eq!(p.len(), 1);
    assert_eq!(t.g.obj(p[0]).controller, P1);
    assert_eq!(t.zone(other), Zone::Library(P2));
    assert_eq!(shuffles(&t, P0), 1);
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(shuffles(&t, P2), 0);
}

#[test]
fn an_additional_card_if_a_condition_holds_on_resolution() {
    cr!("701.23b", "608.2c");
    ruling!("Tithe", "Counts lands on resolution, not on announcement.");
    assert_supported("Tithe");
    for opponent_lands in [1usize, 3] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 2);
        t.lands(P1, "Swamp", opponent_lands);
        let p1 = t.library_top(P0, "Plains");
        let p2 = t.library_top(P0, "Plains");
        let spell = t.hand(P0, "Tithe");
        t.answer_choose(P0, &[Entity::Object(p1)]);
        t.answer_choose(P0, &[Entity::Object(p2)]);
        t.cast(P0, spell).target(P1).go();
        t.resolve_all();
        let in_hand = t
            .g
            .player(P0)
            .hand
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Plains")
            .count();
        assert_eq!(in_hand, if opponent_lands > 2 { 2 } else { 1 });
    }
}

#[test]
fn instead_search_for_something_else() {
    cr!("701.23a");
    assert_supported("Nissa's Triumph");
    for nissa in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        if nissa {
            t.battlefield(P0, "Nissa, Genesis Mage");
        }
        let swamp = t.library_top(P0, "Swamp");
        let f1 = t.library_top(P0, "Forest");
        let spell = t.hand(P0, "Nissa's Triumph");
        t.answer_choose(P0, &[Entity::Object(f1), Entity::Object(swamp)]);
        t.cast(P0, spell).go();
        t.resolve_all();
        let offered = choice_candidates(&t, 0);
        assert_eq!(offered.contains(&Entity::Object(swamp)), nissa);
        assert!(t.in_hand(P0, "Forest"));
        assert_eq!(t.in_hand(P0, "Swamp"), nissa);
    }
}

#[test]
fn an_instruction_between_putting_the_card_and_shuffling() {
    cr!("701.23a");
    ruling!("Gamble", "You might end up discarding the card you searched for.");
    assert_supported("Gamble");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spell = t.hand(P0, "Gamble");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    // The only card in hand was the found one: it's discarded at random.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn where_the_found_card_goes_depends_on_a_condition() {
    cr!("701.23a", "608.2c");
    assert_supported("Stoic Farmer");
    for opponent_has_more in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 1);
        if opponent_has_more {
            t.lands(P1, "Swamp", 3);
        }
        let plains = t.library_top(P0, "Plains");
        t.answer_choose(P0, &[Entity::Object(plains)]);
        t.enter(P0, "Stoic Farmer");
        t.resolve_all();
        let on_bf = t.named_on_battlefield("Plains").len();
        assert_eq!(on_bf, if opponent_has_more { 2 } else { 1 });
        assert_eq!(t.in_hand(P0, "Plains"), !opponent_has_more);
        assert_eq!(shuffles(&t, P0), 1);
    }
}

#[test]
fn search_target_opponent_s_library_and_cast_the_card() {
    cr!("701.23a");
    ruling!(
        "Knowledge Exploitation",
        "If you can't find an instant or sorcery card that can be legally cast, or choose not to find one, skip that part of the effect. Then the opponent shuffles their library."
    );
    assert_supported("Knowledge Exploitation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let growth = t.library_top(P1, "Giant Growth");
    let spell = t.hand(P0, "Knowledge Exploitation");
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert!(choice_candidates(&t, 0).contains(&Entity::Object(growth)));
    assert_eq!(t.zone(growth), Zone::Library(P1));
    assert_eq!(shuffles(&t, P1), 1);
}

#[test]
fn reveal_then_shuffle_then_put_the_found_card_somewhere() {
    cr!("701.24b", "701.20d");
    assert_supported("Loyal Inventor");
    for assassin in [false, true] {
        let mut t = TestGame::new(2);
        if assassin {
            t.battlefield(P0, "Royal Assassin");
        }
        let ornithopter = t.library_top(P0, "Ornithopter");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(ornithopter)]);
        t.enter(P0, "Loyal Inventor");
        t.resolve_all();
        // The found card wasn't part of the shuffle, so "that card" still finds it.
        assert_eq!(shuffles(&t, P0), 1);
        if assassin {
            assert!(t.in_hand(P0, "Ornithopter"));
        } else {
            let lib = &t.g.player(P0).library;
            let top = *lib.last().unwrap();
            assert_eq!(t.g.obj(top).chars.name, "Ornithopter");
        }
    }
}

#[test]
fn each_player_who_searched_their_library_this_way_shuffles() {
    cr!("701.23a", "701.24a");
    ruling!(
        "Boldwyr Heavyweights",
        "may choose not to search for a creature card"
    );
    assert_supported("Boldwyr Heavyweights");
    for accept in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.library_top(P1, "Grizzly Bears");
        t.answer_yes(P1, accept);
        t.answer_choose(P1, &[Entity::Object(bears)]);
        t.enter(P0, "Boldwyr Heavyweights");
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), usize::from(accept));
        assert_eq!(shuffles(&t, P1), usize::from(accept));
        assert_eq!(shuffles(&t, P0), 0);
    }
}

#[test]
fn the_exiled_found_cards_may_be_cast_this_turn() {
    cr!("701.23a", "400.7");
    // Its first ability is another item's; the -9 ability compiles.
    assert!(card("Chandra, Heart of Fire")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("Search")));
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Heart of Fire");
    t.g.objects[chandra.0 as usize]
        .counters
        .insert("loyalty".into(), 9);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let shock = t.graveyard(P0, "Shock");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(shock)]);
    t.activate(P0, chandra, 1, &[]).unwrap();
    t.resolve_all();
    let offered = choice_candidates(&t, 0);
    assert!(offered.contains(&Entity::Object(bolt)) && offered.contains(&Entity::Object(shock)));
    assert!(!offered.contains(&Entity::Object(bears)));
    assert_eq!(shuffles(&t, P0), 1);
    // Both are exiled, and one can be cast with the red mana the ability added.
    let exiled: Vec<ObjectId> = t
        .g
        .exile
        .iter()
        .copied()
        .filter(|o| matches!(&*t.g.obj(*o).chars.name, "Lightning Bolt" | "Shock"))
        .collect();
    assert_eq!(exiled.len(), 2);
    let bolt_now = *exiled
        .iter()
        .find(|o| t.g.obj(**o).chars.name == "Lightning Bolt")
        .unwrap();
    t.cast(P0, bolt_now).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn doomsday_keeps_five_cards_and_exiles_the_rest() {
    cr!("701.23a", "701.23d", "401.4");
    ruling!("Doomsday", "you must choose five cards from among them");
    assert_supported("Doomsday");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.lands(P0, "Swamp", 3);
    let a = t.library_top(P0, "Lightning Bolt");
    let b = t.library_top(P0, "Grizzly Bears");
    let c = t.library_top(P0, "Shock");
    let d = t.graveyard(P0, "Savannah Lions");
    let e = t.graveyard(P0, "Island");
    let left = t.graveyard(P0, "Forest");
    let spell = t.hand(P0, "Doomsday");
    t.answer_choose(P0, &objs(&[a, b, c, d, e]));
    t.cast(P0, spell).go();
    t.resolve_all();
    // Every card of the library and graveyard was offered.
    let offered = choice_candidates(&t, 0);
    for o in [a, b, c, d, e, left] {
        assert!(offered.contains(&Entity::Object(o)), "{o:?} not offered");
    }
    assert!(t.in_exile("Forest"));
    let lib: Vec<String> = t
        .g
        .player(P0)
        .library
        .iter()
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    assert_eq!(lib.len(), 5, "{lib:?}");
    for n in ["Lightning Bolt", "Grizzly Bears", "Shock", "Savannah Lions", "Island"] {
        assert!(lib.iter().any(|x| x == n), "{n} not in {lib:?}");
    }
    // The cards already in the library stayed the same objects (no zone change).
    for o in [a, b, c] {
        assert_eq!(t.zone(o), Zone::Library(P0));
    }
    assert!(t.asked().iter().any(|(_, d)| matches!(d, Decision::Order { .. })));
    assert_eq!(shuffles(&t, P0), 0);
    assert_eq!(t.life(P0), 10);
    // Spells left the graveyard: the graveyard holds only Doomsday itself.
    assert_eq!(t.g.player(P0).graveyard.len(), 1);
}

#[test]
fn doomsday_with_fewer_than_five_cards_keeps_them_all_in_the_chosen_order() {
    cr!("701.23d", "401.4");
    ruling!("Doomsday", "all of those cards will wind up in your library");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.lands(P0, "Swamp", 3);
    let a = t.library_top(P0, "Lightning Bolt");
    let d = t.graveyard(P0, "Savannah Lions");
    let spell = t.hand(P0, "Doomsday");
    t.answer_choose(P0, &objs(&[a, d]));
    // The second listed card (the Lions, now in the library) goes on top.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.cast(P0, spell).go();
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    assert_eq!(lib.len(), 2);
    assert_eq!(t.g.obj(lib[1]).chars.name, "Savannah Lions");
    assert_eq!(lib[0], a);
}

#[test]
fn x_is_the_number_of_players_with_at_least_two_more_lands() {
    cr!("701.23a", "107.3c");
    ruling!("Surveyor's Scope", "You count the number of players who control at least two more lands than you when the ability resolves.");
    ruling!("Surveyor's Scope", "you'll still search and shuffle your library");
    assert_supported("Surveyor's Scope");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Plains", 3); // two more: counts
    t.lands(P2, "Plains", 2); // one more: doesn't
    let scope = t.battlefield(P0, "Surveyor's Scope");
    let f1 = t.library_top(P0, "Forest");
    let f2 = t.library_top(P0, "Forest");
    t.answer_choose(P0, &objs(&[f1]));
    t.activate(P0, scope, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(f2), Zone::Library(P0)); // shuffled, still in the library
    let forests = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.name == "Forest")
        .count();
    assert_eq!(forests, 1);
    assert_eq!(shuffles(&t, P0), 1);

    // Nobody has two more lands: nothing is found, but the library is still shuffled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Plains", 2);
    let scope = t.battlefield(P0, "Surveyor's Scope");
    t.library_top(P0, "Forest");
    t.activate(P0, scope, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.g.battlefield.iter().any(|o| t.g.obj(*o).chars.name == "Forest"));
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn a_search_if_you_ve_cast_spells_with_both_names_this_turn() {
    cr!("701.23a", "201.2");
    assert_supported("Sift Through Sands");
    // Both named spells cast this turn: The Unspeakable can be found.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let target = t.library_top(P0, "The Unspeakable");
    for _ in 0..8 {
        t.library_top(P0, "Island");
    }
    let reach = t.hand(P0, "Reach Through Mists");
    t.cast(P0, reach).go();
    t.resolve_all();
    let peer = t.hand(P0, "Peer Through Depths");
    t.cast(P0, peer).go();
    t.resolve_all();
    // Peer Through Depths may have put The Unspeakable back on the bottom: it's still in
    // the library either way.
    let target = t.g.current(target);
    assert_eq!(t.zone(target), Zone::Library(P0));
    let sift = t.hand(P0, "Sift Through Sands");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[target]));
    t.cast(P0, sift).go();
    t.resolve_all();
    assert!(t
        .g
        .battlefield
        .iter()
        .any(|o| t.g.obj(*o).chars.name == "The Unspeakable"));

    // Only one of them: no search.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let target = t.library_top(P0, "The Unspeakable");
    let reach = t.hand(P0, "Reach Through Mists");
    t.cast(P0, reach).go();
    t.resolve_all();
    let sift = t.hand(P0, "Sift Through Sands");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[target]));
    t.cast(P0, sift).go();
    t.resolve_all();
    assert!(!t
        .g
        .battlefield
        .iter()
        .any(|o| t.g.obj(*o).chars.name == "The Unspeakable"));
    assert!(!searched(&t, P0));
}
