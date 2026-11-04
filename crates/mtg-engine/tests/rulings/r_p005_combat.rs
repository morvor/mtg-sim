//! Rulings batch P005 — blocks and combat damage: "blocks or becomes blocked by" triggers
//! checked once (CR 509.2a, 509.3c), blocking restrictions checked only as blockers are
//! declared (CR 509.1b), "becomes blocked" by an effect (CR 509.1h, 702.19d), double strike
//! gained mid-combat (CR 702.4c-d), damage assigned by toughness (CR 510.1a), face-down
//! blockers (CR 708.8), attack requirements and costs (CR 508.1d).

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn old(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].summoning_sick = false;
    t.g.objects[id.0 as usize].entered_turn = 0;
    id
}

/// Runs the game until `step` of the current turn has begun and a player has priority.
fn run_to(t: &mut TestGame, step: Step) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority) || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?}");
    t.settle();
}

/// From the declare attackers step: `dp` declares these blocks; the declare blockers step
/// begins (its triggers on the stack).
fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    run_to(t, Step::DeclareBlockers);
}

/// `ap` attacks: from `ap`'s beginning of combat, declares these attackers.
fn attacks(t: &mut TestGame, ap: PlayerId, attackers: &[(ObjectId, Entity)]) {
    t.set_step(ap, Step::BeginningOfCombat);
    attack_with(t, attackers);
}

fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(obj(id), kind, n, None);
    t.g.recompute();
}

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(id)
}

fn blocking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_blocking(id)
}

#[test]
fn arrogant_bloodlord_triggers_per_small_creature_and_each_destroys_it() {
    cr!("509.3b", "509.3d", "702.89a");
    ruling!(
        "Arrogant Bloodlord",
        "If Arrogant Bloodlord becomes blocked by multiple creatures with power 1 or less, its ability triggers that many times. Each of those abilities will cause it to be destroyed separately (in case Arrogant Bloodlord isn't destroyed the first time due to an Aura with umbra armor, for example). The same is true if it somehow blocks multiple creatures with power 1 or less."
    );
    supported("Arrogant Bloodlord");
    // Blocked by two 1/1s, wearing Hyena Umbra: the first destruction destroys the Umbra
    // instead, the second destroys the Bloodlord.
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Arrogant Bloodlord");
    let umbra = t.battlefield(P0, "Hyena Umbra");
    assert!(t.g.attach(umbra, obj(b)));
    let e1 = old(&mut t, P1, "Llanowar Elves");
    let e2 = old(&mut t, P1, "Llanowar Elves");
    attacks(&mut t, P0, &[(b, Entity::Player(P1))]);
    block(&mut t, P1, &[(e1, b), (e2, b)]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    assert!(t.in_graveyard(P0, "Arrogant Bloodlord"));
    // Blocking two 1/1s (Brave the Sands lets it block an additional creature).
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Arrogant Bloodlord");
    t.battlefield(P0, "Brave the Sands");
    let e1 = old(&mut t, P1, "Llanowar Elves");
    let e2 = old(&mut t, P1, "Llanowar Elves");
    attacks(
        &mut t,
        P1,
        &[(e1, Entity::Player(P0)), (e2, Entity::Player(P0))],
    );
    block(&mut t, P0, &[(b, e1), (b, e2)]);
    assert!(blocking(&t, b));
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn arrogant_bloodlord_checks_power_only_as_blocks_are_declared() {
    cr!("509.3f", "603.7a");
    ruling!(
        "Arrogant Bloodlord",
        "If Arrogant Bloodlord blocks or becomes blocked by a creature whose power is 2 or greater, Arrogant Bloodlord's ability won't trigger. It still won't trigger if the other creature's power changes to 1 or less later in combat."
    );
    ruling!(
        "Arrogant Bloodlord",
        "The power of the other creature is checked only when it blocks or becomes blocked by Arrogant Bloodlord. Once the Bloodlord's ability triggers, it will be destroyed at end of combat (after combat damage has been dealt) even if the other creature's power changes before then."
    );
    // Blocked by a 2/2 that then shrinks to 1/1: no trigger; it survives combat.
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Arrogant Bloodlord");
    let bears = old(&mut t, P1, "Grizzly Bears");
    attacks(&mut t, P0, &[(b, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, b)]);
    assert_eq!(t.stack_len(), 0);
    put(&mut t, bears, "-1/-1", 1);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    // Blocking a 2/2: no trigger either.
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Arrogant Bloodlord");
    let bears = old(&mut t, P1, "Grizzly Bears");
    attacks(&mut t, P1, &[(bears, Entity::Player(P0))]);
    block(&mut t, P0, &[(b, bears)]);
    assert_eq!(t.stack_len(), 0);
    // Blocked by a 1/1 that then grows to 3/1: still destroyed, at end of combat, after
    // combat damage (the 3/1 dealt its 3 and died).
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Arrogant Bloodlord");
    let elf = old(&mut t, P1, "Llanowar Elves");
    attacks(&mut t, P0, &[(b, Entity::Player(P1))]);
    block(&mut t, P1, &[(elf, b)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    crate::r_p057_common::pump(&mut t, elf, 2, 0);
    run_to(&mut t, Step::CombatDamage);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.on_battlefield(b), "not yet");
    assert_eq!(t.obj_now(b).damage, 3);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Arrogant Bloodlord"));
}

#[test]
fn feasting_hobbit_compares_power_only_as_blockers_are_declared() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Feasting Hobbit",
        "The comparison of power is done only when blockers are declared. Decreasing the power of a creature blocking Feasting Hobbit or increasing the power of Feasting Hobbit after this point won't cause Feasting Hobbit to become unblocked."
    );
    supported("Feasting Hobbit");
    let mut t = TestGame::new(2);
    let h = old(&mut t, P0, "Feasting Hobbit");
    let bears = old(&mut t, P1, "Grizzly Bears");
    let elf = old(&mut t, P1, "Llanowar Elves");
    attacks(&mut t, P0, &[(h, Entity::Player(P1))]);
    // The 1/1 can't block the 2/2 Hobbit; the 2/2 can.
    block(&mut t, P1, &[(elf, h), (bears, h)]);
    assert!(!blocking(&t, elf), "the illegal block was refused");
    let mut t = TestGame::new(2);
    let h = old(&mut t, P0, "Feasting Hobbit");
    let bears = old(&mut t, P1, "Grizzly Bears");
    attacks(&mut t, P0, &[(h, Entity::Player(P1))]);
    block(&mut t, P1, &[(bears, h)]);
    assert!(blocking(&t, bears));
    // Now the Hobbit grows to 4/4 and the Bears shrink to 1/1: still blocked.
    put(&mut t, h, "+1/+1", 2);
    put(&mut t, bears, "-1/-1", 1);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(h));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

/// P0 attacks P1 with `attacker`, no blocks; P1 activates Trap Runner targeting it in the
/// declare blockers step; it resolves.
fn trap(t: &mut TestGame, attacker: ObjectId) -> ObjectId {
    let runner = old(t, P1, "Trap Runner");
    attacks(t, P0, &[(attacker, Entity::Player(P1))]);
    block(t, P1, &[]);
    t.activate(P1, runner, 0, &[obj(attacker)]).unwrap();
    t.resolve();
    runner
}

#[test]
fn trap_runner_trample_damage_still_comes_through() {
    cr!("509.1h", "702.19d");
    ruling!(
        "Trap Runner",
        "If the ability is used on a creature with Trample, the damage still comes through."
    );
    supported("Trap Runner");
    for (name, damage) in [("Colossal Dreadmaw", 6), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        let a = old(&mut t, P0, name);
        trap(&mut t, a);
        assert!(t.g.combat.as_ref().unwrap().is_blocked(a), "{name}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20 - damage, "{name}");
    }
}

#[test]
fn trap_runner_triggers_becomes_blocked_abilities() {
    cr!("509.1h", "509.3c");
    ruling!(
        "Trap Runner",
        "This ability will trigger any creature’s “whenever this becomes blocked” ability."
    );
    supported("Gustcloak Runner");
    let mut t = TestGame::new(2);
    let g = old(&mut t, P0, "Gustcloak Runner");
    trap(&mut t, g);
    assert_eq!(t.stack_len(), 1, "Gustcloak Runner's ability triggered");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!attacking(&t, g), "removed from combat by its trigger");
    assert!(!t.obj_now(g).tapped);
}

#[test]
fn trap_runner_works_after_blockers_in_each_combat_phase() {
    cr!("506.1", "500.8");
    ruling!(
        "Trap Runner",
        "If a turn has multiple combat phases, the ability can be activated during any of them as long as it’s after the beginning of that phase’s Declare Blockers Step."
    );
    use crate::r_s02_common::can_activate;
    let mut t = TestGame::new(2);
    let runner = old(&mut t, P1, "Trap Runner");
    let a = old(&mut t, P0, "Grizzly Bears");
    attacks(&mut t, P0, &[(a, Entity::Player(P1))]);
    assert!(!can_activate(&mut t, P1, runner), "declare attackers step");
    block(&mut t, P1, &[]);
    assert!(
        can_activate(&mut t, P1, runner),
        "first declare blockers step"
    );
    // A second combat phase (Relentless Assault in the postcombat main phase).
    t.advance_to(P0, Step::PostcombatMain);
    crate::r_p044_common::cast_spell(&mut t, P0, "Relentless Assault", &[]);
    assert!(!t.obj_now(a).tapped);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1))]),
    );
    run_to(&mut t, Step::DeclareAttackers);
    assert!(attacking(&t, a));
    assert!(
        !can_activate(&mut t, P1, runner),
        "second combat, before its declare blockers step"
    );
    block(&mut t, P1, &[]);
    assert!(
        can_activate(&mut t, P1, runner),
        "second declare blockers step"
    );
}

/// Gives the city's blessing to P0 now: ten permanents (CR 702.131b).
fn ascend(t: &mut TestGame) {
    t.lands(P0, "Wastes", 10);
    t.g.recompute();
    t.settle();
}

#[test]
fn storm_fleet_swashbuckler_gaining_double_strike_mid_combat() {
    cr!("702.4c", "702.4d", "510.4");
    ruling!(
        "Storm Fleet Swashbuckler",
        "If Storm Fleet Swashbuckler gains double strike after it has dealt regular combat damage, it won't go back and deal first-strike combat damage. On the other hand, if it gains first strike somehow and then it gains double strike after dealing first-strike combat damage, it will also deal regular combat damage."
    );
    supported("Storm Fleet Swashbuckler");
    // After regular combat damage: nothing more.
    let mut t = TestGame::new(2);
    let s = old(&mut t, P0, "Storm Fleet Swashbuckler");
    attacks(&mut t, P0, &[(s, Entity::Player(P1))]);
    block(&mut t, P1, &[]);
    run_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    ascend(&mut t);
    assert!(t.obj_now(s).has_keyword(KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // With first strike (a first strike counter), double strike after the first-strike
    // damage step: regular damage too.
    let mut t = TestGame::new(2);
    let s = old(&mut t, P0, "Storm Fleet Swashbuckler");
    put(&mut t, s, "first strike", 1);
    attacks(&mut t, P0, &[(s, Entity::Player(P1))]);
    block(&mut t, P1, &[]);
    run_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    ascend(&mut t);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn streetwise_negotiator_only_assigns_combat_damage_by_toughness() {
    cr!("510.1a", "701.14b");
    ruling!(
        "Streetwise Negotiator",
        "Streetwise Negotiator’s ability doesn’t actually change any creature’s power. It changes only the amount of combat damage the creature assigns. All other rules and effects that check power or toughness use the real values, even if they cause damage “equal to a creature’s power” to be dealt. Notably, if Streetwise Negotiator (or the creature that gains its ability) fights, it uses its actual power to determine how much damage it deals."
    );
    supported("Streetwise Negotiator");
    supported("Prey Upon");
    supported("Rabid Bite");
    // Combat: 2 damage (its toughness), power still 0.
    let mut t = TestGame::new(2);
    let n = old(&mut t, P0, "Streetwise Negotiator");
    assert_eq!(t.pt(n), (0, 2));
    attacks(&mut t, P0, &[(n, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    // Fight: 0 damage (its power).
    let mut t = TestGame::new(2);
    let n = old(&mut t, P0, "Streetwise Negotiator");
    let elf = old(&mut t, P1, "Llanowar Elves");
    crate::r_p044_common::cast_spell(&mut t, P0, "Prey Upon", &[obj(n), obj(elf)]);
    assert!(t.on_battlefield(elf));
    assert_eq!(t.obj_now(elf).damage, 0);
    assert_eq!(t.obj_now(n).damage, 1);
    // Damage "equal to its power" (Rabid Bite): 0.
    crate::r_p044_common::cast_spell(&mut t, P0, "Rabid Bite", &[obj(n), obj(elf)]);
    assert_eq!(t.obj_now(elf).damage, 0);
}

#[test]
fn face_down_horde_ambusher_blocks_then_turns_face_up() {
    cr!("708.8", "509.2a", "506.4");
    ruling!(
        "Horde Ambusher",
        "If a face-down Horde Ambusher blocks and is then turned face up, its first ability won't trigger because it didn't have that ability when it blocked."
    );
    ruling!(
        "Horde Ambusher",
        "Turning Horde Ambusher face up after blockers have been declared won't remove any creature from combat or change how any creature is blocking."
    );
    supported("Horde Ambusher");
    let mut t = TestGame::new(2);
    let amb = old(&mut t, P0, "Horde Ambusher");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, amb));
    let giant = old(&mut t, P0, "Hill Giant");
    let att = old(&mut t, P1, "Craw Wurm");
    attacks(&mut t, P1, &[(att, Entity::Player(P0))]);
    block(&mut t, P0, &[(amb, att), (giant, att)]);
    assert_eq!(t.stack_len(), 0, "the face-down blocker has no abilities");
    // Turned face up: "target creature can't block this turn" on the blocking Hill Giant.
    t.answer_targets(P0, &[obj(giant)]);
    assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, amb, P0));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "only the turned-face-up trigger");
    t.resolve_all();
    assert_eq!(t.obj_now(amb).chars.name, "Horde Ambusher");
    assert!(blocking(&t, giant), "still blocking");
    assert!(blocking(&t, amb));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(att));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(
        t.life(P0),
        20,
        "no 1 damage from Horde Ambusher, no combat damage"
    );
    // 2 + 3 damage: neither blocker alone would have killed the 6/4.
    assert!(
        t.in_graveyard(P1, "Craw Wurm"),
        "both blockers dealt damage"
    );
}

#[test]
fn kaust_needs_the_creature_turned_face_up_before_its_damage() {
    cr!("510.3a", "603.2");
    ruling!(
        "Kaust, Eyes of the Glade",
        "The creature must have been turned face up before dealing combat damage to a player for Kaust, Eyes of the Glade’s first ability to trigger."
    );
    supported("Kaust, Eyes of the Glade");
    for before in [true, false] {
        let mut t = TestGame::new(2);
        old(&mut t, P0, "Kaust, Eyes of the Glade");
        let m = old(&mut t, P0, "Horde Ambusher");
        assert!(mtg_engine::facedown::turn_face_down(&mut t.g, m));
        let victim = old(&mut t, P1, "Grizzly Bears");
        attacks(&mut t, P0, &[(m, Entity::Player(P1))]);
        block(&mut t, P1, &[]);
        let up = |t: &mut TestGame| {
            t.answer_targets(P0, &[obj(victim)]);
            assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, m, P0));
            t.g.flush_events();
            t.resolve_all();
        };
        if before {
            up(&mut t);
        }
        let hand = t.hand_size(P0);
        run_to(&mut t, Step::CombatDamage);
        assert_eq!(t.life(P1), 18);
        if !before {
            up(&mut t);
        }
        t.resolve_all();
        let drew = if before { 1 } else { 0 };
        assert_eq!(
            t.hand_size(P0),
            hand + drew,
            "turned up before damage: {before}"
        );
    }
}

/// Whether `id` attacked in `ap`'s combat this turn when `ap` declares no attackers
/// (the engine fills in the requirements).
fn attacks_by_requirement(t: &mut TestGame, ap: PlayerId, id: ObjectId) -> bool {
    t.set_step(ap, Step::BeginningOfCombat);
    t.answer(ap, DecisionKind::Attackers, Answer::Attackers(vec![]));
    run_to(t, Step::DeclareAttackers);
    attacking(t, id)
}

#[test]
fn mogis_warhound_attacks_only_if_able_and_without_a_cost() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Mogis's Warhound",
        "If, during your declare attackers step, Mogis's Warhound (or the creature it enchants) is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under its controller's control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Mogis's Warhound");
    // `bestowed`: the Warhound enchants a Grizzly Bears (the creature tested).
    let setup = |bestowed: bool| -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        if bestowed {
            let bears = old(&mut t, P0, "Grizzly Bears");
            t.lands(P0, "Mountain", 3);
            let w = t.hand(P0, "Mogis's Warhound");
            t.cast(P0, w)
                .method(CastMethod::Keyword(KeywordKind::Bestow))
                .target(bears)
                .go();
            t.resolve_all();
            (t, bears)
        } else {
            let w = old(&mut t, P0, "Mogis's Warhound");
            (t, w)
        }
    };
    for bestowed in [false, true] {
        let (mut t, c) = setup(bestowed);
        assert!(
            attacks_by_requirement(&mut t, P0, c),
            "able: attacks ({bestowed})"
        );
        let (mut t, c) = setup(bestowed);
        assert!(t.g.tap(c));
        assert!(
            !attacks_by_requirement(&mut t, P0, c),
            "tapped ({bestowed})"
        );
        let (mut t, c) = setup(bestowed);
        crate::r_p044_common::cast_spell(&mut t, P0, "Pacifism", &[obj(c)]);
        assert!(
            !attacks_by_requirement(&mut t, P0, c),
            "can't attack ({bestowed})"
        );
        let (mut t, c) = setup(bestowed);
        t.g.objects[c.0 as usize].summoning_sick = true;
        assert!(
            !attacks_by_requirement(&mut t, P0, c),
            "summoning sick ({bestowed})"
        );
        let (mut t, c) = setup(bestowed);
        t.battlefield(P1, "Ghostly Prison");
        t.lands(P0, "Wastes", 2);
        assert!(
            !attacks_by_requirement(&mut t, P0, c),
            "attack cost ({bestowed})"
        );
    }
}

#[test]
fn mogis_warhound_enchanted_creatures_controller_chooses_what_it_attacks() {
    cr!("508.1a", "508.1d");
    ruling!(
        "Mogis's Warhound",
        "The controller of Mogis's Warhound or the creature it's enchanting chooses which player or planeswalker to attack."
    );
    // P0's Warhound, bestowed on P1's Grizzly Bears; on P1's turn P1 picks P2.
    let mut t = TestGame::new(3);
    let bears = old(&mut t, P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let w = t.hand(P0, "Mogis's Warhound");
    t.cast(P0, w)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    let from = t.asked().len();
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P2))]),
    );
    run_to(&mut t, Step::DeclareAttackers);
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::DeclareAttackers { .. })));
    assert!(!asked
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::DeclareAttackers { .. })));
    let c = t.g.combat.as_ref().unwrap();
    let a = c.attackers.iter().find(|a| a.id == bears).unwrap();
    assert_eq!(a.target, Some(Entity::Player(P2)));
}

#[test]
fn dain_lets_a_must_attack_creature_stay_home_unless_a_free_target_exists() {
    cr!("508.1d", "702.195a");
    ruling!(
        "Dáin, Lord of the Iron Hills",
        "While the effect of Dáin's last ability is applying, your opponents can choose not to attack with a creature that must attack if able as long as there is no other player, planeswalker, or battle for that creature to attack that wouldn't require a cost."
    );
    supported("Dáin, Lord of the Iron Hills");
    for players in [2, 3] {
        let mut t = TestGame::new(players);
        old(&mut t, P0, "Dáin, Lord of the Iron Hills");
        old(&mut t, P0, "Mind Stone");
        old(&mut t, P0, "Mind Stone");
        t.settle();
        let w = old(&mut t, P1, "Mogis's Warhound");
        t.lands(P1, "Wastes", 2);
        let attacked = attacks_by_requirement(&mut t, P1, w);
        if players == 2 {
            assert!(!attacked, "only Dáin's controller to attack: may stay home");
        } else {
            assert!(attacked, "P2 can be attacked for free");
            let c = t.g.combat.as_ref().unwrap();
            let a = c.attackers.iter().find(|a| a.id == w).unwrap();
            assert_eq!(a.target, Some(Entity::Player(P2)));
        }
        assert_eq!(t.life(P0), 20);
    }
}
