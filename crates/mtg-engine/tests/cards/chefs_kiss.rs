//! Chef's Kiss: "Gain control of target spell that targets only a single permanent or
//! player. Copy it, then reselect the targets at random for the spell and the copy. The
//! new targets can't be you or a permanent you control." (compiled: the random
//! reselection is `Effect::ReselectTargetsAtRandom`, shared with Grip of Chaos).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use rand::SeedableRng;

/// P1 casts `name` with `targets`; P0 answers with Chef's Kiss on it.
fn kiss(t: &mut TestGame, name: &str, mana: &[&str], targets: &[Entity]) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    for m in mana {
        t.lands(P1, m, 1);
    }
    let card = t.hand(P1, name);
    let mut c = t.cast(P1, card);
    for e in targets {
        c = c.target(*e);
    }
    let spell = c.go();
    t.lands(P0, "Mountain", 3);
    let k = t.hand(P0, "Chef's Kiss");
    t.g.turn.priority = Some(P0);
    t.cast(P0, k).target(spell).go();
    t.resolve();
    spell
}

#[test]
fn the_spell_and_its_copy_get_new_targets_that_arent_you_or_yours() {
    cr!("115.7d", "707.10");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let spell = kiss(
        &mut t,
        "Lightning Bolt",
        &["Mountain"],
        &[Entity::Player(P0)],
    );
    // P0 controls the stolen spell and its copy; both now target P1, the only legal
    // target that isn't P0 or a permanent P0 controls.
    assert_eq!(t.obj_now(spell).controller, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(mine));
    // The stolen card goes to its owner's graveyard.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn the_new_targets_are_chosen_at_random() {
    cr!("115.7d");
    ruling!("Chef's Kiss", "including a spell you control");
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..16 {
        let mut t = TestGame::new(2);
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        // P0 steals its own spell (a spell you control is a legal target).
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 4);
        let bolt = t.hand(P0, "Lightning Bolt");
        let spell = t.cast(P0, bolt).target(mine).go();
        let k = t.hand(P0, "Chef's Kiss");
        t.cast(P0, k).target(spell).go();
        t.resolve_all();
        assert!(t.on_battlefield(mine), "never P0's own permanent");
        assert_eq!(t.life(P0), 20, "never P0");
        let dead = [!t.on_battlefield(a), !t.on_battlefield(b)];
        let p1_damage = 20 - t.life(P1);
        // Two bolts among P1 and P1's two creatures (a creature both target dies to the
        // first; the second then has an illegal target).
        let killed = dead.iter().filter(|d| **d).count() as i32;
        assert!(killed * 3 + p1_damage == 6 || (killed == 1 && p1_damage == 0));
        outcomes.insert((dead, p1_damage));
    }
    assert!(outcomes.len() >= 3, "{outcomes:?}");
}

#[test]
fn a_target_with_no_new_legal_target_stays_and_can_be_illegal() {
    cr!("115.7d", "608.2b");
    ruling!(
        "Chef's Kiss",
        "will be unable to find a new legal target, and its target will remain unchanged"
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    // "Target creature you control": P0 now controls the spell and its copy, and every
    // creature P0 controls is excluded, so the target stays: P1's creature, now illegal.
    kiss(
        &mut t,
        "Ranger's Guile",
        &["Forest"],
        &[Entity::Object(theirs)],
    );
    t.resolve_all();
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(t.in_graveyard(P1, "Ranger's Guile"));
}

#[test]
fn no_legal_new_target_keeps_a_target_that_is_still_legal() {
    cr!("115.7d", "702.11c");
    ruling!(
        "Chef's Kiss",
        "If there are no legal targets that could be selected, the targets remain unchanged"
    );
    let mut t = TestGame::new(2);
    // P1 has hexproof, so P0's spells can't target P1: both still target P0.
    t.battlefield(P1, "Leyline of Sanctity");
    kiss(&mut t, "Lava Spike", &["Mountain"], &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn each_instance_of_target_is_reselected() {
    cr!("115.7d", "115.3");
    ruling!(
        "Chef's Kiss",
        "a new target will be reselected at random for each instance of the word target"
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Agony Warp: "Target creature gets -3/-0 until end of turn. Target creature gets
    // -0/-3 until end of turn.", both on P0's Hill Giant; each moves to P1's.
    kiss(
        &mut t,
        "Agony Warp",
        &["Island", "Swamp"],
        &[Entity::Object(mine), Entity::Object(mine)],
    );
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 3));
    assert!(!t.on_battlefield(theirs));
}
