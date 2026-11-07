//! The worked examples of CR 701 (keyword actions), played out as tests. The CR 701.12a
//! example and the Guile example of CR 701.24c are cited by tests in
//! `tests/actions/a701_012_exchange.rs` and `tests/actions/a701_024_shuffle.rs`.

use crate::r600_examples_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn count_events(t: &TestGame, f: impl Fn(&Event) -> bool) -> usize {
    t.g.turn_events.iter().filter(|e| f(e)).count()
}

fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    count_events(t, |e| matches!(e, Event::Shuffled { player } if *player == p))
}

fn searches(t: &TestGame, p: PlayerId) -> usize {
    count_events(t, |e| matches!(e, Event::Searched { player, .. } if *player == p))
}

/// The candidates of every "choose entities" decision `p` was asked.
fn entity_choices(t: &TestGame, p: PlayerId) -> Vec<Vec<Entity>> {
    t.asked()
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_merfolk_card_revealed_by_telepathy_can_be_revealed_for_silvergill_adept() {
    cr!("701.20a");
    cr_example!("701.20c", "Telepathy is an enchantment card");
    supported("Telepathy");
    supported("Silvergill Adept");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Telepathy");
    let merfolk = t.hand(P0, "Merrow Reejerey");
    assert!(mtg_engine::reveal::is_revealed(&t.g, merfolk));
    // Only {1}{U}: the alternative {3} can't be paid.
    t.lands(P0, "Island", 2);
    let adept = t.hand(P0, "Silvergill Adept");
    t.cast(P0, adept).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Silvergill Adept").len(), 1);
    assert!(t.in_hand(P0, "Merrow Reejerey"));
    // It was revealed (again) to pay the cost.
    assert_eq!(
        count_events(&t, |e| matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name.as_str() == mtg_engine::reveal::REVEALED && *o == merfolk)),
        1
    );
}

#[test]
fn splinter_must_find_the_graveyard_howling_mine_but_may_leave_library_ones() {
    cr!("701.23a", "701.23d");
    cr_example!("701.23b", "Splinter says");
    supported("Splinter");
    supported("Howling Mine");
    for found in [0usize, 1, 2] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P1, "Howling Mine");
        let in_gy = t.graveyard(P1, "Howling Mine");
        let lib_a = t.library_top(P1, "Howling Mine");
        let lib_b = t.library_top(P1, "Howling Mine");
        t.lands(P0, "Forest", 4);
        let pick: Vec<Entity> = [lib_a, lib_b][..found]
            .iter()
            .map(|o| Entity::Object(*o))
            .collect();
        t.answer_choose(P0, &pick);
        let s = t.hand(P0, "Splinter");
        t.cast(P0, s).target(mine).go();
        t.resolve();
        // The targeted Mine and the one in the graveyard (a public zone) are exiled.
        assert!(!t.g.is_live(mine));
        assert!(!t.g.is_live(in_gy));
        assert_eq!(t.g.find_in_zone(Zone::Exile, "Howling Mine").len(), 2 + found);
        assert_eq!(
            t.g.find_in_zone(Zone::Library(P1), "Howling Mine").len(),
            2 - found
        );
        // The library search is the only choice: the graveyard one had to be found.
        for c in entity_choices(&t, P0) {
            assert!(!c.contains(&Entity::Object(in_gy)));
        }
        assert_eq!(shuffles(&t, P1), 1);
    }
}

#[test]
fn lobotomy_against_an_empty_hand_searches_but_exiles_nothing() {
    cr!("701.23a");
    cr_example!("701.23c", "Lobotomy says");
    ruling!(
        "Lobotomy",
        "If the player has no cards in their hand, you still get to look through their deck"
    );
    supported("Lobotomy");
    let mut t = TestGame::new(2);
    assert_eq!(t.hand_size(P1), 0);
    t.graveyard(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let lib = t.library_size(P1);
    let l = t.hand(P0, "Lobotomy");
    t.cast(P0, l).target(P1).go();
    t.resolve();
    assert_eq!(searches(&t, P0), 1);
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(t.library_size(P1), lib);
    assert_eq!(t.graveyard_size(P1), 1);
    assert!(t.g.find_in_zone(Zone::Exile, "Grizzly Bears").is_empty());
}

#[test]
fn shimian_specter_against_a_hand_of_lands_still_searches_and_shuffles() {
    // The same wording as Lobotomy's, from a trigger: "that player" is the damaged one.
    cr!("701.23a", "701.23c");
    ruling!(
        "Shimian Specter",
        "If the player has no nonland cards in their hand, you can still search that player’s library and have them shuffle it."
    );
    supported("Shimian Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Shimian Specter");
    t.hand(P1, "Forest");
    t.graveyard(P1, "Lightning Bolt");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(specter, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(searches(&t, P0), 1);
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(shuffles(&t, P0), 0);
    assert!(t.in_hand(P1, "Forest"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn aven_mindcensor_limits_the_search_but_the_whole_library_is_shuffled() {
    cr!("701.23", "701.24a");
    cr_example!("701.23f", "Veteran Explorer");
    supported("Aven Mindcensor");
    supported("Veteran Explorer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aven Mindcensor");
    let explorer = t.battlefield(P0, "Veteran Explorer");
    // P1's library: a Plains deep down, a Forest among the top four.
    let deep = t.library_top(P1, "Plains");
    for _ in 0..2 {
        t.library_top(P1, "Hill Giant");
    }
    let forest = t.library_top(P1, "Forest");
    t.library_top(P1, "Hill Giant");
    let below: Vec<ObjectId> = {
        let lib = &t.g.player(P1).library;
        lib[..lib.len() - 4].to_vec()
    };
    t.g.destroy(explorer, None);
    t.settle();
    // Each player may search: P0 finds nothing; P1 tries to take both lands.
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.resolve_all();
    // P1 searched only the top four cards: the Forest was found, the Plains couldn't be.
    let p1_choices = entity_choices(&t, P1);
    assert!(p1_choices.iter().all(|c| !c.contains(&Entity::Object(deep))));
    assert!(p1_choices.iter().any(|c| c.contains(&Entity::Object(forest))));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t.g.is_live(deep));
    // P1 shuffles the entire library, not just the top four cards.
    assert_eq!(shuffles(&t, P1), 1);
    let lib = &t.g.player(P1).library;
    assert_eq!(lib.len(), below.len() + 3);
    assert_ne!(lib[..below.len()].to_vec(), below);
}

#[test]
fn black_suns_zenith_cast_with_flashback_is_exiled_and_the_library_shuffled() {
    cr!("702.34a", "614.1a");
    cr_example!("701.24c", "Black Sun’s Zenith says");
    supported("Black Sun's Zenith");
    supported("Recoup");
    let mut t = TestGame::new(2);
    let bsz = t.graveyard(P0, "Black Sun's Zenith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 3);
    let recoup = t.hand(P0, "Recoup");
    t.cast(P0, recoup).target(bsz).go();
    t.resolve();
    let lib = t.library_size(P0);
    t.cast(P0, bsz)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .x(1)
        .go();
    t.resolve();
    assert_eq!(t.counters(bears, "-1/-1"), 1);
    // Exiled by flashback rather than shuffled; the library is shuffled anyway.
    assert!(t.in_exile("Black Sun's Zenith"));
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn loaming_shaman_with_no_cards_targeted_still_shuffles() {
    cr!("701.24a");
    cr_example!("701.24d", "Loaming Shaman says");
    supported("Loaming Shaman");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let shaman = t.hand(P0, "Loaming Shaman");
    t.cast(P0, shaman).go();
    // The trigger targets P1 and no cards.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(shuffles(&t, P0), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn darksteel_colossus_is_shuffled_in_and_gravebane_zombie_goes_on_top() {
    cr!("701.24a", "614.1a");
    cr_example!("701.24g", "Darksteel Colossus and Gravebane Zombie");
    supported("Darksteel Colossus");
    supported("Gravebane Zombie");
    supported("Barter in Blood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darksteel Colossus");
    t.battlefield(P0, "Gravebane Zombie");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 4);
    let lib = t.library_size(P0);
    // Both are put into the graveyard from the battlefield at the same time.
    let barter = t.hand(P1, "Barter in Blood");
    t.cast(P1, barter).go();
    t.resolve();
    assert_eq!(t.library_size(P0), lib + 2);
    let library = &t.g.player(P0).library;
    assert_eq!(
        t.g.obj(*library.last().unwrap()).chars.name,
        "Gravebane Zombie"
    );
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Darksteel Colossus").len(), 1);
    assert_eq!(shuffles(&t, P0), 1);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn midnight_scavengers_and_a_graf_rats_token_are_exiled_but_cant_meld() {
    cr!("701.42b", "111.7");
    cr_example!("701.42c", "Midnight Scavengers and a token");
    supported("Midnight Scavengers");
    supported("Graf Rats");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Graf Rats");
    let scavengers = t.battlefield(P0, "Midnight Scavengers");
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(rats).go();
    t.resolve();
    // Only the token copy remains.
    t.g.exile_object(rats, None);
    let token = t.named_on_battlefield("Graf Rats")[0];
    assert!(t.obj_now(token).is_token());
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.named_on_battlefield("Chittering Host").is_empty());
    assert_eq!(t.zone(scavengers), Zone::Exile);
    assert!(!t.g.is_live(token));
    assert!(t.g.find_in_zone(Zone::Exile, "Graf Rats").len() == 1);
}

#[test]
fn spirit_water_revivals_waterbend_six_can_be_paid_by_tapping_but_not_its_mana_cost() {
    cr!("701.67a", "601.2g");
    cr_example!("701.67b", "Spirit Water Revival");
    supported("Spirit Water Revival");
    // (creatures, Islands, pay the waterbend cost) -> can it be cast?
    let attempt = |creatures: usize, islands: usize, waterbend: bool| {
        let mut t = TestGame::new(2);
        for _ in 0..creatures {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.lands(P0, "Island", islands);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(waterbend));
        let s = t.hand(P0, "Spirit Water Revival");
        let hand = t.hand_size(P0);
        let r = t.cast(P0, s).try_go();
        if r.is_ok() {
            let tapped = t
                .named_on_battlefield("Grizzly Bears")
                .into_iter()
                .filter(|b| t.obj_now(*b).tapped)
                .count();
            t.resolve();
            // With the waterbend cost paid: seven cards; otherwise two.
            let drawn = t.hand_size(P0) + 1 - hand;
            return Some((tapped, drawn));
        }
        None
    };
    // Six creatures pay all of the {6}; three Islands pay {1}{U}{U}.
    assert_eq!(attempt(6, 3, true), Some((6, 7)));
    // Seven creatures and two Islands: no creature can pay for the {1} of the mana cost.
    assert_eq!(attempt(7, 2, true), None);
    assert_eq!(attempt(7, 2, false), None);
    // Without waterbending, the creatures don't help at all.
    assert_eq!(attempt(6, 3, false), Some((0, 2)));
}
