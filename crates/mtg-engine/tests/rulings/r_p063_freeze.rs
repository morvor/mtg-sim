//! Rulings batch P063 — "freeze" effects: tapping a creature (or a player's lands) so it
//! doesn't untap during its controller's next untap step (CR 502.3, 611.2a), and stun
//! counters (CR 122.1d). They can target creatures that are already tapped (CR 701.26a:
//! tapping them does nothing more), they track the creature rather than its controller,
//! and conditions in them are checked as they resolve (CR 608.2).

use crate::r_p062_common::{
    is_tapped, misses_one_untap, tap, tapped_creature, through_untap_step, untaps_next,
};
use crate::r_p063_common::*;
use crate::r_s01_common::supported;
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::give_control;
use crate::r_s07_common::cast_methods;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::STUN;
use mtg_engine::*;

/// The real creature `name` enters under P0's control and its enters ability targets
/// P1's tapped Bears: they stay tapped through P1's next untap step only.
fn etb_on_tapped_bears(name: &str) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, name);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    t
}

#[test]
fn watertrap_weaver_and_white_dragon_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Watertrap Weaver",
        "Watertrap Weaver’s ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    ruling!(
        "White Dragon",
        "White Dragon's Cold Breath ability can target a tapped creature. That creature won't untap during its controller's next untap step."
    );
    for name in ["Watertrap Weaver", "White Dragon"] {
        let mut t = etb_on_tapped_bears(name);
        let bears = t.named_on_battlefield("Grizzly Bears")[0];
        misses_one_untap(&mut t, bears, P1);
    }
}

/// P0 casts Giant Growth on their Wavecrash Triton; the heroic trigger targets P1's
/// tapped Bears. Returns the Bears.
fn triton_heroic(t: &mut TestGame) -> ObjectId {
    supported("Wavecrash Triton");
    let triton = t.battlefield(P0, "Wavecrash Triton");
    let bears = tapped_creature(t, P1, "Grizzly Bears");
    // The spell's target, then the trigger's.
    cast_and_resolve(t, P0, "Giant Growth", &[obj(triton), obj(bears)]);
    assert_eq!(t.pt(triton), (4, 7), "Giant Growth resolved");
    assert!(is_tapped(t, bears));
    bears
}

#[test]
fn wavecrash_triton_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3", "603.2");
    ruling!(
        "Wavecrash Triton",
        "Wavecrash Triton's ability can target a creature that's already tapped. That creature still won't untap during its controller's next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = triton_heroic(&mut t);
    misses_one_untap(&mut t, bears, P1);
}

/// P0 gains control of `creature` (P1's, which P0's effect froze) before P1's next untap
/// step: it doesn't untap during P1's untap step (P0 controls it) nor during P0's next
/// one, and untaps in P0's untap step after that.
fn p0_steals_it(t: &mut TestGame, creature: ObjectId) {
    give_control(t, creature, P0);
    if t.g.turn.active == P0 {
        through_untap_step(t, P1);
        assert!(is_tapped(t, creature));
    }
    misses_one_untap(t, creature, P0);
}

#[test]
fn wavecrash_triton_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Wavecrash Triton",
        "Wavecrash Triton's ability tracks the creature, but not its controller. If the creature changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = triton_heroic(&mut t);
    p0_steals_it(&mut t, bears);
}

#[test]
fn wall_of_frost_tracks_the_creature_not_its_controller() {
    cr!("502.3", "611.2a", "509.1");
    ruling!(
        "Wall of Frost",
        "Wall of Frost’s ability tracks the creature, not the creature’s controller. That is, if the creature changes controllers before its first controller’s next untap step, then it won’t untap during its new controller’s next untap step."
    );
    supported("Wall of Frost");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Frost");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 attacks P0 with the Bears; the Wall blocks them.
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, bears)]),
    );
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    // P0 gains control of the Bears during P1's turn: P0's next untap step doesn't untap
    // them, P0's one after that does.
    p0_steals_it(&mut t, bears);
    // Control: without the block they'd have untapped in P1's untap step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Frost");
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    untaps_next(&mut t, bears, P1);
}

#[test]
fn will_of_the_naga_and_snow_day_can_target_tapped_creatures() {
    cr!("701.26a", "502.3", "115.1");
    ruling!(
        "Will of the Naga",
        "Will of the Naga can target creatures that are already tapped. Those creatures won't untap during their controller's next untap step."
    );
    ruling!(
        "Snow Day",
        "You may choose tapped creatures as targets. Those creatures won’t untap during their controller’s next untap step."
    );
    for name in ["Will of the Naga", "Snow Day"] {
        supported(name);
        let mut t = TestGame::new(2);
        let a = tapped_creature(&mut t, P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        // (Snow Day: draw two, then discard a card.)
        crate::r_p016_common::cast_multi(&mut t, P0, name, &[obj(a), obj(b)]);
        t.resolve_all();
        assert!(is_tapped(&t, a) && is_tapped(&t, b), "{name}");
        through_untap_step(&mut t, P1);
        assert!(is_tapped(&t, a) && is_tapped(&t, b), "{name}");
        through_untap_step(&mut t, P1);
        assert!(!is_tapped(&t, a) && !is_tapped(&t, b), "{name}");
    }
}

#[test]
fn will_of_the_naga_tracks_the_creatures_not_their_controllers() {
    cr!("502.3", "611.2a");
    ruling!(
        "Will of the Naga",
        "Will of the Naga tracks the creatures, but not their controllers. If any of those creatures changes controllers before its first controller's next untap step, then it won't untap during its new controller's next untap step."
    );
    let mut t = TestGame::new(2);
    let a = tapped_creature(&mut t, P1, "Grizzly Bears");
    let b = tapped_creature(&mut t, P1, "Hill Giant");
    crate::r_p016_common::cast_multi(&mut t, P0, "Will of the Naga", &[obj(a), obj(b)]);
    t.resolve_all();
    give_control(&mut t, a, P0);
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, a) && is_tapped(&t, b));
    through_untap_step(&mut t, P0);
    assert!(is_tapped(&t, a), "the stolen one doesn't untap in P0's next untap step");
    through_untap_step(&mut t, P0);
    assert!(!is_tapped(&t, a));
}

#[test]
fn icy_blasts_ferocious_is_checked_only_as_it_resolves() {
    cr!("608.2h", "502.3", "207.2c");
    ruling!(
        "Icy Blast",
        "You check if you control a creature with power 4 or greater as Icy Blast resolves. It doesn’t matter if you control such a creature during any of the relevant untap steps."
    );
    supported("Icy Blast");
    // P0 controls Craw Wurm (6/4) as Icy Blast resolves, then loses it: the Bears still
    // don't untap.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let blast = t.hand(P0, "Icy Blast");
    t.cast(P0, blast).x(1).target(bears).go();
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    move_to(&mut t, wurm, Zone::Graveyard(P0));
    misses_one_untap(&mut t, bears, P1);
    // No such creature as it resolves; one arrives later: the Bears untap as normal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let blast = t.hand(P0, "Icy Blast");
    t.cast(P0, blast).x(1).target(bears).go();
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    t.battlefield(P0, "Craw Wurm");
    untaps_next(&mut t, bears, P1);
}

#[test]
fn protocol_knight_checks_for_another_knight_only_as_it_resolves() {
    cr!("608.2c", "603.4", "122.1d");
    ruling!(
        "Protocol Knight",
        "Whether or not you control another Knight is checked only as Protocol Knight’s ability is resolving. It doesn’t matter if you still control Protocol Knight at that time, or if it’s still a Knight."
    );
    supported("Protocol Knight");
    // Another Knight (White Knight, a Human Knight) arrives while the trigger waits.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Protocol Knight");
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "White Knight");
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    // Protocol Knight leaves before the trigger resolves: the other Knight still counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let pk = enter(&mut t, P0, "Protocol Knight");
    move_to(&mut t, pk, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(bears, STUN), 1);
    // Control: no other Knight as it resolves (the only one left in response).
    let mut t = TestGame::new(2);
    let wk = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Protocol Knight");
    move_to(&mut t, wk, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 0);
}

#[test]
fn fear_of_sleep_paralysis_stops_stunned_opponents_paying_untap_costs() {
    cr!("107.6", "118.3", "122.1d");
    ruling!(
        "Fear of Sleep Paralysis",
        "While you control Fear of Sleep Paralysis, opponents can't use tapped permanents with stun counters on them to pay costs that include untapping a permanent. This includes costs that use the {Q} (untap) symbol."
    );
    supported("Fear of Sleep Paralysis");
    // Order of Whiteclay: "{1}{W}{W}, {Q}: Return target creature card with mana value 3
    // or less from your graveyard to the battlefield."
    let setup = |fear: bool| {
        let mut t = TestGame::new(2);
        if fear {
            t.battlefield(P0, "Fear of Sleep Paralysis");
        }
        let order = t.battlefield(P1, "Order of Whiteclay");
        tap(&mut t, order);
        put(&mut t, order, STUN, 1);
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.lands(P1, "Plains", 3);
        t.set_step(P1, Step::PrecombatMain);
        (t, order, bears)
    };
    let (mut t, order, bears) = setup(true);
    assert!(t.activate(P1, order, 0, &[obj(bears)]).is_err());
    assert_eq!(t.counters(order, STUN), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Control: without Fear of Sleep Paralysis the stun counter pays for {Q}.
    let (mut t, order, bears) = setup(false);
    t.activate(P1, order, 0, &[obj(bears)])
        .expect("{Q} paid by removing the stun counter");
    assert_eq!(t.counters(order, STUN), 0);
    assert!(is_tapped(&t, order));
    t.resolve_all();
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn stun_effects_can_target_tapped_creatures_and_still_stun_them() {
    cr!("701.26a", "122.1d", "608.2b");
    ruling!(
        "Galvanic Giant // Storm Reading",
        "You may choose a creature that's already tapped as the target of Galvanic Giant's ability. If the target creature is already tapped as it resolves, you will still put a stun counter on it."
    );
    ruling!(
        "Sharae of Numbing Depths",
        "You may target a creature that is already tapped with Sharae's first ability. If the target creature is already tapped as it resolves, you will still put a stun counter on it."
    );
    ruling!(
        "Solitary Sanctuary",
        "You may target a creature that is already tapped with Solitary Sanctuary's first ability. If the target creature is already tapped as it resolves, you will still put a stun counter on it."
    );
    ruling!(
        "Tranquilize",
        "You may target a creature that is already tapped with Tranquilize. If the target creature is already tapped as Tranquilize resolves, you will still put three stun counters on it."
    );
    ruling!(
        "Succumb to the Cold",
        "You may target creatures that are already tapped with Succumb to the Cold. If a target creature is already tapped as it resolves, you will still put a stun counter on it."
    );
    ruling!("Wicked Slumber", "Wicked Slumber can target creatures that are already tapped.");
    // Enters abilities.
    for name in ["Sharae of Numbing Depths", "Solitary Sanctuary"] {
        let t = etb_on_tapped_bears(name);
        let bears = t.named_on_battlefield("Grizzly Bears")[0];
        assert_eq!(t.counters(bears, STUN), 1, "{name}");
    }
    // Spells.
    for (name, n) in [("Tranquilize", 3), ("Succumb to the Cold", 1)] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
        cast_and_resolve(&mut t, P0, name, &[obj(bears)]);
        assert!(is_tapped(&t, bears));
        assert_eq!(t.counters(bears, STUN), n, "{name}");
    }
    // Wicked Slumber: two tapped targets, a stun counter on either of them twice.
    supported("Wicked Slumber");
    let mut t = TestGame::new(2);
    let a = tapped_creature(&mut t, P1, "Grizzly Bears");
    let b = tapped_creature(&mut t, P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    let ws = t.hand(P0, "Wicked Slumber");
    t.cast(P0, ws).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    assert!(is_tapped(&t, a) && is_tapped(&t, b));
    assert_eq!(t.counters(a, STUN) + t.counters(b, STUN), 2);
    // Galvanic Giant: P0 casts a spell with mana value 5 or greater (Craw Wurm).
    supported("Galvanic Giant // Storm Reading");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galvanic Giant // Storm Reading");
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.lands(P0, "Forest", 6);
    let wurm = t.hand(P0, "Craw Wurm");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").len() == 1);
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
}

#[test]
fn dreamdew_entrancer_stuns_your_own_tapped_creature_and_you_draw() {
    cr!("701.26a", "122.1d", "608.2c");
    ruling!(
        "Dreamdew Entrancer",
        "You may target a creature that is already tapped with Dreamdew Entrancer’s last ability. If the target creature is already tapped as the ability resolves, you’ll still put three stun counters on it, and if you control it, you’ll still draw two cards."
    );
    supported("Dreamdew Entrancer");
    let mut t = TestGame::new(2);
    let mine = tapped_creature(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[obj(mine)]);
    enter(&mut t, P0, "Dreamdew Entrancer");
    t.resolve_all();
    assert!(is_tapped(&t, mine));
    assert_eq!(t.counters(mine, STUN), 3);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Control: an opponent's creature gets the counters, but P0 draws nothing.
    let mut t = TestGame::new(2);
    let theirs = tapped_creature(&mut t, P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[obj(theirs)]);
    enter(&mut t, P0, "Dreamdew Entrancer");
    t.resolve_all();
    assert_eq!(t.counters(theirs, STUN), 3);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn sensational_spider_man_may_remove_stun_counters_from_your_own_permanents() {
    cr!("122.1d", "608.2c");
    ruling!(
        "Sensational Spider-Man",
        "You may remove stun counters from permanents you control as well as ones your opponents control."
    );
    supported("Sensational Spider-Man");
    let mut t = TestGame::new(2);
    let spidey = t.battlefield(P0, "Sensational Spider-Man");
    let mine = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, mine, STUN, 2);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_yes(P0, true);
    // Both from P0's own Bears, none from the Giant just stunned.
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    let hand = t.hand_size(P0);
    t.attack(&[(spidey, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(mine, STUN), 0);
    assert_eq!(t.counters(theirs, STUN), 1);
    assert!(is_tapped(&t, theirs));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn barls_cage_and_elvish_hunter_can_target_an_untapped_creature() {
    cr!("115.1", "502.3", "611.2a");
    ruling!(
        "Barl's Cage",
        "You may target any creature with this ability. It doesn’t have to be tapped."
    );
    ruling!(
        "Elvish Hunter",
        "You may target any creature with this ability. It doesn’t have to be tapped."
    );
    for (name, ty, n) in [
        ("Barl's Cage", ManaType::C, 3),
        ("Elvish Hunter", ManaType::G, 2),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        pool(&mut t, P0, ty, n);
        t.activate(P0, src, 0, &[obj(bears)])
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        t.resolve_all();
        assert!(!is_tapped(&t, bears), "{name}: it isn't tapped");
        // Tapped later, they still don't untap during P1's next untap step.
        tap(&mut t, bears);
        misses_one_untap(&mut t, bears, P1);
    }
}

#[test]
fn permafrost_trap_targets_up_to_two_tapped_or_untapped_creatures_of_any_color() {
    cr!("118.9", "115.1", "502.3");
    ruling!(
        "Permafrost Trap",
        "You may target any two (or fewer) creatures. It's okay if any of them are already tapped. If you cast Permafrost Trap for {U}, you don't have to target the green creature that entered under an opponent's control."
    );
    supported("Permafrost Trap");
    let mut t = TestGame::new(2);
    // A green creature entered under P1's control this turn.
    let green = t.enter(P1, "Grizzly Bears");
    t.settle();
    let tapped = tapped_creature(&mut t, P1, "Hill Giant");
    let other = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Island", 1);
    let trap = t.hand(P0, "Permafrost Trap");
    let alt = cast_methods(&mut t, P0, trap)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the {U} alternative cost");
    t.cast(P0, trap)
        .method(alt)
        .targets(&[obj(tapped), obj(other)])
        .go();
    t.resolve_all();
    assert!(!is_tapped(&t, green), "the green creature wasn't targeted");
    assert!(is_tapped(&t, tapped) && is_tapped(&t, other));
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, tapped) && is_tapped(&t, other));
}

#[test]
fn mana_vapors_affects_lands_that_entered_after_it_resolved() {
    cr!("502.3", "611.2c");
    ruling!(
        "Mana Vapors",
        "No land controlled by the player will untap during their next untap step. This includes lands which entered the battlefield after Mana Vapors resolved."
    );
    supported("Mana Vapors");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P1, "Forest");
    tap(&mut t, old);
    cast_and_resolve(&mut t, P0, "Mana Vapors", &[Entity::Player(P1)]);
    let new = t.battlefield(P1, "Mountain");
    tap(&mut t, new);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, old) && is_tapped(&t, new));
    assert!(!is_tapped(&t, bears), "only lands are affected");
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, old) && !is_tapped(&t, new));
}
