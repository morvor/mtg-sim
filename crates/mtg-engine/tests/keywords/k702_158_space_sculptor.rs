//! CR 702.158 Space sculptor (see also `tests/cr/r704_dungeons_and_sectors.rs`).

use crate::common_k702_153_167::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const BELEREN: &str = "Space Beleren";

fn sector(t: &TestGame, id: ObjectId) -> Option<String> {
    t.obj_now(id).sector.as_ref().map(|s| s.to_string())
}

/// Queues `p`'s answer to the next option question (a sector: 0 alpha, 1 beta, 2 gamma).
fn choose(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// The players asked to assign a sector since the `from`th decision, in order.
fn assigners(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { prompt, .. } if prompt.starts_with("Choose a sector for"))
        })
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn space_sculptor_gives_creatures_sector_designations() {
    cr!("702.158", "702.158a", "702.158c");
    ruling!(
        "Space Beleren",
        "Any time Space Beleren is on the battlefield, the battlefield is divided into three sectors: alpha, beta, and gamma. All creatures will be assigned to one of the three sectors."
    );
    ruling!(
        "Space Beleren",
        "First, all players who don’t control a Space Beleren (or another permanent with space sculptor, but come on) in turn order assign their creatures."
    );
    assert_supported(BELEREN);
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let p1s = t.battlefield(P1, "Hill Giant");
    let p2s = t.battlefield(P2, "Savannah Lions");
    t.settle();
    assert_eq!(sector(&t, mine), None);
    let from = t.asked().len();
    choose(&mut t, P1, 1);
    choose(&mut t, P2, 2);
    choose(&mut t, P0, 0);
    let beleren = t.battlefield(P0, BELEREN);
    t.settle();
    // Players who don't control a permanent with space sculptor choose first (in turn
    // order), then the others.
    assert_eq!(assigners(&t, from), vec![P1, P2, P0]);
    assert_eq!(sector(&t, p1s).as_deref(), Some("beta"));
    assert_eq!(sector(&t, p2s).as_deref(), Some("gamma"));
    assert_eq!(sector(&t, mine).as_deref(), Some("alpha"));
    // Space Beleren itself isn't a creature: it gets no sector.
    assert_eq!(sector(&t, beleren), None);
}

#[test]
fn a_sector_designation_lasts_until_no_one_controls_space_sculptor() {
    cr!("702.158b");
    ruling!(
        "Space Beleren",
        "A creature that changes controllers maintains its sector assignment: its new controller can’t give it a new one."
    );
    let mut t = TestGame::new(2);
    let beleren = t.battlefield(P0, BELEREN);
    choose(&mut t, P0, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(sector(&t, bears).as_deref(), Some("gamma"));
    // Changing control keeps it (nobody is asked).
    let from = t.asked().len();
    t.g.objects[bears.0 as usize].base_controller = P1;
    t.g.recompute();
    t.settle();
    assert!(assigners(&t, from).is_empty());
    assert_eq!(sector(&t, bears).as_deref(), Some("gamma"));
    // Only permanents have sector designations: a creature that leaves the battlefield
    // and comes back is a new object without one.
    t.g.move_object(
        bears,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    assert_eq!(sector(&t, bears), None);
    // Once no player controls a permanent with space sculptor, designations are lost.
    let giant = t.battlefield(P1, "Hill Giant");
    choose(&mut t, P1, 1);
    t.settle();
    assert_eq!(sector(&t, giant).as_deref(), Some("beta"));
    t.g.move_object(
        beleren,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.settle();
    assert_eq!(sector(&t, giant), None);
}

#[test]
fn an_action_on_each_creature_in_the_sector_of_your_choice() {
    cr!("702.158d");
    ruling!(
        "Space Beleren",
        "For the last two abilities, you choose the sector as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let beleren = t.battlefield(P0, BELEREN);
    choose(&mut t, P1, 0);
    choose(&mut t, P1, 1);
    choose(&mut t, P0, 0);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    // −1: Put a +1/+1 counter on each creature in the sector of your choice: alpha.
    t.activate(P0, beleren, 1, &[]).unwrap();
    choose(&mut t, P0, 0);
    t.resolve_all();
    assert_eq!(plus1(&t, a), 1);
    assert_eq!(plus1(&t, mine), 1);
    assert_eq!(plus1(&t, b), 0);
    // −5: Destroy all creatures in the sector of your choice: beta.
    let bid = t.g.current(beleren).0 as usize;
    t.g.objects[bid]
        .counters
        .insert(counters::LOYALTY.into(), 6);
    t.set_step(P0, Step::PrecombatMain);
    // A loyalty ability of it was activated this turn: pretend it's a new turn.
    t.g.objects[bid].activations_this_turn.clear();
    t.activate(P0, beleren, 2, &[]).unwrap();
    choose(&mut t, P0, 1);
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(a));
    assert!(t.on_battlefield(mine));
}

#[test]
fn creatures_in_the_same_sector_have_the_same_designation() {
    cr!("702.158e");
    ruling!(
        "Space Beleren",
        "unless Space Beleren’s first loyalty ability has been activated, sector assignments have no effect on combat"
    );
    let mut t = TestGame::new(2);
    let beleren = t.battlefield(P0, BELEREN);
    // P1's creatures: a wall in alpha, another in beta; P0's attacker in alpha.
    choose(&mut t, P1, 0);
    choose(&mut t, P1, 1);
    choose(&mut t, P0, 0);
    let wall_a = t.battlefield(P1, "Wall of Stone");
    let wall_b = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    let can_block = |t: &mut TestGame, w: ObjectId| {
        t.set_step(P0, Step::DeclareBlockers);
        t.g.combat
            .as_mut()
            .unwrap()
            .attackers
            .push(mtg_engine::combat::AttackerInfo {
                id: bears,
                target: Some(Entity::Player(P1)),
                original_target: Some(Entity::Player(P1)),
                declared: true,
                blocked: false,
                blockers: vec![],
                band: None,
                defending_player: Some(P1),
            });
        t.g.recompute();
        let ok = mtg_engine::combat::block_options(&t.g, &[P1])
            .iter()
            .any(|(c, a)| *c == w && a.contains(&bears));
        t.g.combat = None;
        ok
    };
    // Without the +1, sectors don't matter.
    assert!(can_block(&mut t, wall_b));
    // +1: creatures in each sector can be blocked this turn only by creatures in the
    // same sector.
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, beleren, 0, &[]).unwrap();
    t.resolve_all();
    assert!(can_block(&mut t, wall_a));
    assert!(!can_block(&mut t, wall_b));
    // Without a permanent with space sculptor, sectors are gone and so is the
    // restriction.
    t.g.move_object(
        beleren,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.settle();
    assert!(can_block(&mut t, wall_b));
}

#[test]
fn the_same_sector_blocking_restriction_lasts_this_turn() {
    cr!("702.158e");
    let mut t = TestGame::new(2);
    let beleren = t.battlefield(P0, BELEREN);
    choose(&mut t, P1, 1);
    choose(&mut t, P0, 0);
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    t.activate(P0, beleren, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!mtg_engine::kw::space_sculptor::same_sector(
        &t.g, wall, bears
    ));
    // Next turn, P0 attacks: the wall in another sector can block again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, bears)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "blocked by the wall");
}

#[test]
fn a_creature_is_never_in_more_than_one_sector() {
    cr!("702.158b");
    ruling!(
        "Space Beleren",
        "A creature can never be in more than one sector. If Space Beleren is on the battlefield, another Space Beleren coming under a player’s control won’t affect any creature’s sector assignment."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, BELEREN);
    choose(&mut t, P1, 1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(sector(&t, theirs).as_deref(), Some("beta"));
    // Another Space Beleren, under the other player's control: nobody is asked again.
    let from = t.asked().len();
    t.battlefield(P1, BELEREN);
    t.settle();
    assert!(assigners(&t, from).is_empty());
    assert_eq!(sector(&t, theirs).as_deref(), Some("beta"));
}
