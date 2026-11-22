//! Rulings batch P120 — statics that depend on +1/+1 counters, in and out of combat: once
//! blocks are declared, gaining evasion doesn't undo them (CR 509.1h, 506.4), losing an
//! attack permission doesn't remove an attacker (CR 506.4), a creature gaining or losing
//! first strike mid-combat deals combat damage once (CR 510.4); renown (CR 702.112),
//! replacement-effect ordering for damage (CR 616.1, 615), cost reductions (CR 601.2f),
//! and self-targeting enters abilities.

use crate::r_p120_common::*;
use crate::r_s29_common::damage_marked;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// From the declare attackers step, declares `dp`'s blocks and stops in the declare
/// blockers step.
fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::DeclareBlockers);
}

fn finish_combat(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::EndOfCombat);
}

fn remove_plus1(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let n = plus1(t, id);
    t.g.remove_counters(Entity::Object(id), PLUS1, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

fn has(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

// --- evasion gained after blocks ------------------------------------------------------------

#[test]
fn dreamdrinker_vampire_gaining_menace_after_being_blocked_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Dreamdrinker Vampire",
        "Once Dreamdrinker Vampire becomes blocked by a creature, giving it menace by putting one or more +1/+1 counters on it won't cause it to become unblocked."
    );
    supported("Dreamdrinker Vampire");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Dreamdrinker Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, v)]);
    give_plus1(&mut t, v, 1);
    t.resolve_all();
    assert!(has(&t, v, KeywordKind::Menace));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn skyknight_squire_gaining_flying_after_being_blocked_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Skyknight Squire",
        "Once Skyknight Squire has been blocked, putting enough +1/+1 counters on it to give it flying won't cause it to become unblocked."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skyknight Squire");
    give_plus1(&mut t, s, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, s)]);
    give_plus1(&mut t, s, 1);
    assert!(has(&t, s, KeywordKind::Flying));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn herald_of_secret_streams_counter_after_blocks_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Herald of Secret Streams",
        "Once a creature you control has become blocked, putting a +1/+1 counter on it won't cause it to become unblocked."
    );
    supported("Herald of Secret Streams");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Secret Streams");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, bears)]);
    give_plus1(&mut t, bears, 1);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

// --- attack permissions and blockers losing reach --------------------------------------------

#[test]
fn thoughtbound_phantasm_losing_counters_stays_attacking() {
    cr!("506.4");
    ruling!(
        "Thoughtbound Phantasm",
        "Once Thoughtbound Phantasm has attacked, removing +1/+1 counters from it won't remove it from combat."
    );
    supported("Thoughtbound Phantasm");
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Thoughtbound Phantasm");
    give_plus1(&mut t, ph, 3);
    attack_with(&mut t, &[(ph, Entity::Player(P1))]);
    remove_plus1(&mut t, ph);
    block(&mut t, P1, &[]);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sphere_grid_blocker_losing_reach_keeps_blocking() {
    cr!("509.1h", "506.4");
    ruling!(
        "Sphere Grid",
        "Once a creature that has reach because of Sphere Grid's last ability has blocked a creature, removing all +1/+1 counters from that creature or causing Sphere Grid to leave the battlefield won't cause that creature to stop blocking."
    );
    supported("Sphere Grid");
    for grid_leaves in [false, true] {
        let mut t = TestGame::new(2);
        let grid = t.battlefield(P1, "Sphere Grid");
        let blocker = t.battlefield(P1, "Hill Giant");
        give_plus1(&mut t, blocker, 1);
        assert!(has(&t, blocker, KeywordKind::Reach));
        let flyer = t.battlefield(P0, "Wind Drake");
        attack_with(&mut t, &[(flyer, Entity::Player(P1))]);
        block(&mut t, P1, &[(blocker, flyer)]);
        if grid_leaves {
            destroy(&mut t, grid);
        } else {
            remove_plus1(&mut t, blocker);
        }
        assert!(!has(&t, blocker, KeywordKind::Reach));
        finish_combat(&mut t);
        assert_eq!(t.life(P1), 20, "grid_leaves = {grid_leaves}");
        assert!(t.in_graveyard(P0, "Wind Drake"));
    }
}

// --- first strike gained or lost mid-combat ------------------------------------------------

#[test]
fn inspiring_paladin_first_strike_changes_dont_change_damage_steps() {
    cr!("510.4");
    ruling!(
        "Inspiring Paladin",
        "Losing or gaining first strike after first-strike damage has been dealt won't cause a creature you control to deal combat damage twice or not deal combat damage."
    );
    supported("Inspiring Paladin");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Inspiring Paladin");
    let gainer = t.battlefield(P0, "Grizzly Bears");
    let loser = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, loser, 1);
    assert!(has(&t, loser, KeywordKind::FirstStrike));
    assert!(!has(&t, gainer, KeywordKind::FirstStrike));
    attack_with(
        &mut t,
        &[
            (paladin, Entity::Player(P1)),
            (gainer, Entity::Player(P1)),
            (loser, Entity::Player(P1)),
        ],
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    // Paladin 3 and the Giant 4 dealt first-strike damage.
    assert_eq!(t.life(P1), 13);
    // The Bears gains first strike, the Giant loses it.
    give_plus1(&mut t, gainer, 1);
    remove_plus1(&mut t, loser);
    assert!(has(&t, gainer, KeywordKind::FirstStrike));
    assert!(!has(&t, loser, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    // Only the Bears (now 3/3) deals regular combat damage.
    assert_eq!(t.life(P1), 10);
}

// --- renown --------------------------------------------------------------------------------

#[test]
fn aragorn_renown_instances_trigger_separately_and_only_the_first_counts() {
    cr!("702.112a", "702.112c", "603.3b");
    ruling!(
        "Aragorn, Hornburg Hero",
        "If a creature has multiple instances of renown, each triggers separately. The first one that resolves causes the creature to become renowned, which means any additional renown abilities that creature has will have no effect."
    );
    supported("Aragorn, Hornburg Hero");
    supported("Rhox Maulers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aragorn, Hornburg Hero");
    let m = t.battlefield(P0, "Rhox Maulers");
    attack_with(&mut t, &[(m, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "enown"), 2);
    t.resolve();
    let after_first = plus1(&t, m);
    assert!(after_first == 1 || after_first == 2);
    t.resolve();
    assert_eq!(plus1(&t, m), after_first);
    assert!(t.obj_now(m).renowned);
}

// --- Uncivil Unrest and prevention ----------------------------------------------------------

/// P0 controls Uncivil Unrest and a Grizzly Bears with a +1/+1 counter (3/3), returned.
fn unrest_attack(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Uncivil Unrest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(t, bears, 1);
    bears
}

#[test]
fn uncivil_unrest_and_prevention_order_chosen_by_the_damaged_player() {
    cr!("616.1", "615.1a");
    ruling!(
        "Uncivil Unrest",
        "the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Uncivil Unrest’s) apply."
    );
    supported("Uncivil Unrest");
    supported("Healing Salve");
    for (prevent_first, expected) in [(true, 0), (false, 3)] {
        let mut t = TestGame::new(2);
        let bears = unrest_attack(&mut t);
        // P1 prevents the next 3 damage that would be dealt to P1.
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_new(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
        t.resolve_all();
        crate::r_s03_common::respond(&mut t, P1, if prevent_first {
            |g, d| crate::r_s30_common::pick_replacement(g, d, "Healing Salve")
        } else {
            |g, d| crate::r_s30_common::pick_replacement(g, d, "double")
        });
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 20 - expected, "prevent_first = {prevent_first}");
    }
}

#[test]
fn uncivil_unrest_doesnt_apply_when_all_damage_is_prevented() {
    cr!("615.1a", "616.1");
    ruling!(
        "Uncivil Unrest",
        "If all of the damage is prevented, Uncivil Unrest’s effect no longer applies."
    );
    supported("Holy Day");
    let mut t = TestGame::new(2);
    let bears = unrest_attack(&mut t);
    cast_new(&mut t, P1, "Holy Day", &[]);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

// --- costs and the rest ---------------------------------------------------------------------

#[test]
fn hamza_reduces_creature_spells_by_creatures_with_counters_as_you_cast() {
    cr!("601.2f");
    ruling!(
        "Hamza, Guardian of Arashin",
        "Once you announce you're casting a creature spell, no player may take actions until the spell has been paid for."
    );
    supported("Hamza, Guardian of Arashin");
    let mut t = TestGame::new(2);
    let hamza = t.battlefield(P0, "Hamza, Guardian of Arashin");
    give_plus1(&mut t, hamza, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    // Hill Giant ({3}{R}) costs {1}{R} with two creatures with counters.
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Hill Giant");
    t.cast_with(P0, giant, &[]).expect("cast for {1}{R}");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn oran_rief_ooze_and_pridemalkin_can_target_themselves() {
    cr!("603.3d");
    ruling!(
        "Oran-Rief Ooze",
        "Oran-Rief Ooze can be the target of its own first ability."
    );
    ruling!(
        "Pridemalkin",
        "Pridemalkin can be the target of its own first ability."
    );
    supported("Oran-Rief Ooze");
    supported("Pridemalkin");
    for name in ["Oran-Rief Ooze", "Pridemalkin"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        // It enters; its enters ability goes on the stack (targeting it) as P0 would get
        // priority.
        let id = t.enter(P0, name);
        t.answer_targets(P0, &[obj(id)]);
        t.settle();
        let cands = crate::r_s02_common::target_candidates(&t, P0, from);
        assert!(cands.iter().any(|c| c.contains(&obj(id))), "{name}");
        t.resolve_all();
        assert_eq!(plus1(&t, id), 1, "{name}");
    }
}

#[test]
fn pridemalkin_gives_itself_trample() {
    cr!("611.3a");
    ruling!(
        "Pridemalkin",
        "Pridemalkin's second ability applies to Pridemalkin as long as it has a +1/+1 counter on it."
    );
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pridemalkin");
    assert!(!has(&t, p, KeywordKind::Trample));
    give_plus1(&mut t, p, 1);
    assert!(has(&t, p, KeywordKind::Trample));
}

#[test]
fn power_depot_can_only_be_played_as_a_land() {
    cr!("305.1", "305.9");
    ruling!(
        "Power Depot",
        "Power Depot is a land, so it can only be played as a land. It cannot be cast as a spell."
    );
    supported("Power Depot");
    let mut t = TestGame::new(2);
    let d = t.hand(P0, "Power Depot");
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, d, CastMethod::Normal));
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, d));
}
