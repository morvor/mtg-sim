//! Library searches (CR 701.23): tutors to hand, battlefield and top of library, searches
//! by another player (Path to Exile), searches of another player's library (Extract), and
//! card descriptions like "Mercenary permanent card with mana value 3 or less", "Dragon
//! creature cards" and "cards named ~".

use mtg_engine::decision::{Answer, Decision};
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

#[test]
fn rathi_assassin_searches_for_a_small_mercenary_permanent_card() {
    cr!("701.23a", "110.4a");
    assert_supported("Rathi Assassin");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let assassin = t.battlefield(P0, "Rathi Assassin");
    let claw = t.library_top(P0, "Hired Claw"); // Mercenary, mana value 1
    let big = t.library_top(P0, "Rathi Assassin"); // Mercenary, mana value 4
    let lions = t.library_top(P0, "Savannah Lions"); // not a Mercenary
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(claw)]);
    t.activate(P0, assassin, 1, &[]).unwrap();
    t.resolve();
    let offered = last_choice_candidates(&t);
    assert_eq!(offered, vec![Entity::Object(claw)]);
    assert!(!offered.contains(&Entity::Object(big)));
    assert!(!offered.contains(&Entity::Object(lions)));
    let claws = t.named_on_battlefield("Hired Claw");
    assert_eq!(claws.len(), 1);
    assert_eq!(t.g.obj(claws[0]).controller, P0);
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn path_to_exile_lets_the_creatures_controller_search() {
    cr!("701.23a", "110.2a");
    ruling!(
        "Path to Exile",
        "If that player doesn't, the player won't shuffle their library."
    );
    assert_supported("Path to Exile");
    for accept in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 1);
        let bear = t.battlefield(P1, "Grizzly Bears");
        let forest = t.library_top(P1, "Forest");
        // Filler cards on top of the Forest; declining leaves the order untouched.
        for _ in 0..3 {
            t.library_top(P1, "Grizzly Bears");
        }
        let before = t.g.player(P1).library.clone();
        let path = t.hand(P0, "Path to Exile");
        t.answer_yes(P1, accept);
        t.answer_choose(P1, &[Entity::Object(forest)]);
        t.cast(P0, path).target(bear).go();
        t.resolve();
        assert!(t.in_exile("Grizzly Bears"));
        let lands = t.named_on_battlefield("Forest");
        if accept {
            assert_eq!(lands.len(), 1);
            // The land enters tapped under the searching player's control.
            assert_eq!(t.g.obj(lands[0]).controller, P1);
            assert!(t.g.obj(lands[0]).tapped);
        } else {
            assert!(lands.is_empty());
            assert_eq!(t.g.player(P1).library, before);
        }
    }
}

#[test]
fn imperial_seal_shuffles_then_puts_the_card_on_top() {
    cr!("701.23a", "701.24b");
    assert_supported("Imperial Seal");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let wanted = t.library_top(P0, "Lightning Bolt");
    for _ in 0..5 {
        t.library_top(P0, "Grizzly Bears");
    }
    let seal = t.hand(P0, "Imperial Seal");
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(wanted)]);
    t.cast(P0, seal).go();
    t.resolve();
    // The same card (no zone change) is now on top, and the library size is unchanged.
    assert_eq!(t.g.player(P0).library.last(), Some(&wanted));
    assert_eq!(t.zone(wanted), Zone::Library(P0));
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn squadron_hawk_finds_up_to_three_cards_named_squadron_hawk() {
    cr!("701.23a", "201.2a");
    assert_supported("Squadron Hawk");
    let mut t = TestGame::new(2);
    let hawks: Vec<ObjectId> = (0..4).map(|_| t.library_top(P0, "Squadron Hawk")).collect();
    t.library_top(P0, "Screaming Seahawk");
    t.answer_yes(P0, true);
    t.answer_choose(
        P0,
        &[
            Entity::Object(hawks[0]),
            Entity::Object(hawks[1]),
            Entity::Object(hawks[2]),
        ],
    );
    t.enter(P0, "Squadron Hawk");
    t.resolve_all();
    let offered = last_choice_candidates(&t);
    assert_eq!(offered.len(), 4, "only the Squadron Hawks can be found");
    let in_hand = t.g.find_in_zone(Zone::Hand(P0), "Squadron Hawk").len();
    assert_eq!(in_hand, 3);
    assert!(!t.in_hand(P0, "Screaming Seahawk"));
}

#[test]
fn mystical_teachings_finds_an_instant_or_a_card_with_flash() {
    cr!("701.23a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let ambusher = t.library_top(P0, "Ambush Viper");
    let bears = t.library_top(P0, "Grizzly Bears");
    let teach = t.hand(P0, "Mystical Teachings");
    t.answer_choose(P0, &[Entity::Object(ambusher)]);
    t.cast(P0, teach).go();
    t.resolve();
    let offered = last_choice_candidates(&t);
    assert!(offered.contains(&Entity::Object(bolt)));
    assert!(offered.contains(&Entity::Object(ambusher)));
    assert!(!offered.contains(&Entity::Object(bears)));
    assert!(t.in_hand(P0, "Ambush Viper"));
}

#[test]
fn extract_exiles_a_card_from_target_players_library_and_they_shuffle() {
    cr!("701.23a", "701.24a");
    assert_supported("Extract");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let extract = t.hand(P0, "Extract");
    let lib = t.library_size(P1);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, extract).target(P1).go();
    t.resolve();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.library_size(P1), lib - 1);
    // P0 searched P1's library.
    let asked_p0 = t
        .asked()
        .into_iter()
        .any(|(p, d)| p == P0 && matches!(d, Decision::ChooseEntities { .. }));
    assert!(asked_p0);
}

#[test]
fn sarkhan_unbroken_puts_any_number_of_dragons_onto_the_battlefield() {
    cr!("701.23a");
    assert!(card("Sarkhan Unbroken")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("Dragon creature cards")));
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan Unbroken");
    t.g.objects[sarkhan.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    let d1 = t.library_top(P0, "Shivan Dragon");
    let d2 = t.library_top(P0, "Shivan Dragon");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(d1), Entity::Object(d2)]),
    );
    t.activate(P0, sarkhan, 2, &[]).unwrap();
    t.resolve();
    let offered = last_choice_candidates(&t);
    assert!(!offered.contains(&Entity::Object(bears)));
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 2);
}
