//! The worked examples of CR 608 (resolving spells and abilities), played out as tests.
//! The CR 608.2d and 608.2j examples are cited by tests in `r600_resolution.rs`.

use crate::r506_common::{block, change_types, declare, go_to};
use crate::r600_examples_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn sorins_thirst_with_an_illegal_target_doesnt_resolve_and_gains_no_life() {
    cr!("608.2b");
    cr_example!("608.2b", "Sorin’s Thirst");
    supported("Sorin's Thirst");
    // The creature gains protection from black in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let thirst = t.hand(P0, "Sorin's Thirst");
    t.cast(P0, thirst).target(bears).go();
    let ward = custom_costed(
        "Black Ward",
        "Instant",
        "{0}",
        "Target creature gains protection from black until end of turn.",
    );
    let w = t.custom(P1, ward, Zone::Hand(P1));
    t.cast(P1, w).target(bears).go();
    t.resolve();
    t.resolve();
    assert!(t.in_graveyard(P0, "Sorin's Thirst"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).damage, 0);
    assert_eq!(t.life(P0), 20);
    // The creature leaves the battlefield in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let thirst = t.hand(P0, "Sorin's Thirst");
    t.cast(P0, thirst).target(bears).go();
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sorin's Thirst"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn plague_spores_still_destroys_a_creature_land_turned_black_as_its_target_land() {
    cr!("608.2b", "601.2c", "701.19c");
    cr_example!("608.2b", "Plague Spores");
    supported("Plague Spores");
    let mut t = TestGame::new(2);
    // Dryad Arbor: a green land creature, chosen as both targets.
    let arbor = t.battlefield(P1, "Dryad Arbor");
    t.lands(P0, "Swamp", 5);
    t.lands(P0, "Mountain", 1);
    let spores = t.hand(P0, "Plague Spores");
    t.cast(P0, spores).target(arbor).target(arbor).go();
    // In response its controller regenerates it and it becomes black.
    let regen = custom_costed("Regrowth Shield", "Instant", "{0}", "Regenerate target creature.");
    let r = t.custom(P1, regen, Zone::Hand(P1));
    t.cast(P1, r).target(arbor).go();
    t.resolve();
    let paint = custom_costed(
        "Black Paint",
        "Instant",
        "{0}",
        "Target creature becomes black until end of turn.",
    );
    let pnt = t.custom(P1, paint, Zone::Hand(P1));
    t.cast(P1, pnt).target(arbor).go();
    t.resolve();
    assert!(t.obj(arbor).chars.colors.contains(Color::Black));
    // Plague Spores resolves: it's still a legal "target land" and is destroyed, and it
    // can't be regenerated.
    t.resolve_all();
    assert!(!t.on_battlefield(arbor));
    assert!(t.in_graveyard(P1, "Dryad Arbor"));
    assert!(t.in_graveyard(P0, "Plague Spores"));
}

#[test]
fn blatant_thievery_gains_control_of_all_its_targets_at_once() {
    cr!("608.2f", "601.2c");
    cr_example!("608.2f", "Blatant Thievery");
    supported("Blatant Thievery");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    t.lands(P0, "Island", 7);
    let thievery = t.hand(P0, "Blatant Thievery");
    t.cast(P0, thievery).target(a).target(b).go();
    t.resolve();
    assert_eq!(t.obj_now(a).controller, P0);
    assert_eq!(t.obj_now(b).controller, P0);
    // Both control changes are one simultaneous event (nothing happened in between).
    let changes: Vec<usize> = t
        .g
        .turn_events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, mtg_engine::events::Event::ControlChanged { .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[1], changes[0] + 1);
}

#[test]
fn search_party_captain_counts_a_creature_attacked_with_that_is_no_longer_a_creature() {
    cr!("608.2i", "601.2f");
    cr_example!("608.2i", "Search Party Captain");
    supported("Bear Cub");
    supported("Search Party Captain");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Bear Cub");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(cub, Entity::Player(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    // Bear Cub becomes a noncreature permanent.
    change_types(&mut t, cub, &[CardType::Artifact], &[CardType::Creature]);
    assert!(!t.obj(cub).is_creature());
    // Search Party Captain ({3}{W}) costs {1} less: three Plains pay for it.
    let plains = t.lands(P0, "Plains", 3);
    let captain = t.hand(P0, "Search Party Captain");
    let r = t.cast(P0, captain).try_go();
    assert!(r.is_ok(), "{r:?} {}", t.dump_log());
    assert!(plains.iter().all(|p| t.obj(*p).tapped));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Search Party Captain").len(), 1);
}

#[test]
fn wall_of_tears_returns_the_blocked_permanent_even_after_it_stops_being_a_creature() {
    cr!("608.2k", "603.7c");
    cr_example!("608.2k", "Wall of Tears");
    supported("Wall of Tears");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Tears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P0))]);
    block(&mut t, P0, &[(wall, bears)]);
    // Run until the block trigger is on the stack.
    assert!(t.g.run_until(10_000, |g| !g.stack.is_empty()));
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(t.stack_len(), 1);
    // Before the triggered ability resolves, the creature becomes a noncreature permanent.
    change_types(&mut t, bears, &[CardType::Artifact], &[CardType::Creature]);
    assert!(!t.obj(bears).is_creature());
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn clone_copying_dryad_arbor_under_worms_of_the_earth_goes_to_the_graveyard() {
    cr!("608.3e", "614.1c");
    cr_example!("608.3e", "Worms of the Earth");
    supported("Worms of the Earth");
    supported("Clone");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worms of the Earth");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(arbor)]);
    t.cast(P0, clone).go();
    t.resolve();
    // As a copy of Dryad Arbor it would be a land: it can't enter the battlefield from the
    // stack, and is put into its owner's graveyard.
    assert_eq!(t.named_on_battlefield("Dryad Arbor"), vec![arbor]);
    assert!(t.named_on_battlefield("Clone").is_empty());
    assert!(t.in_graveyard(P0, "Clone"));
    assert_eq!(t.stack_len(), 0);
}
