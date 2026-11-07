//! Rulings batch P029 — a copy of a spell or ability is created on the stack: it isn't cast
//! or activated (CR 707.10), so abilities that trigger when a player casts a spell or
//! activates an ability (CR 603.2) don't trigger for it, including the copying ability
//! itself; and an ability that makes a triggered ability trigger additional times
//! (CR 603.2d) doesn't apply to a copy of a triggered ability.

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1's Spellshock ("Whenever a player casts a spell, this enchantment deals 2 damage to
/// that player.") and Burning-Tree Shaman ("Whenever a player activates an ability that
/// isn't a mana ability, Burning-Tree Shaman deals 1 damage to that player."): P0 loses 2
/// life for each spell P0 casts and 1 for each ability P0 activates.
fn observers(t: &mut TestGame) {
    supported("Spellshock");
    supported("Burning-Tree Shaman");
    t.battlefield(P1, "Spellshock");
    t.battlefield(P1, "Burning-Tree Shaman");
}

/// One case: the copier, the spells P0 casts in all (the Lightning Bolt included), the
/// abilities P0 activates, and the copies of the Bolt made.
struct Case {
    c: Copier,
    casts: usize,
    activations: i32,
    copies: usize,
}

fn case(c: Copier, casts: usize, activations: i32, copies: usize) -> Case {
    Case {
        c,
        casts,
        activations,
        copies,
    }
}

/// P0 casts Lightning Bolt at P1 and the case's copier copies it: the copies aren't cast
/// (Spellshock doesn't trigger for them, and neither does a copier that triggers on casting
/// spells), and they resolve before the Bolt.
fn bolt_copies_arent_cast(k: &Case) {
    let name = k.c.name;
    supported(name);
    let mut t = TestGame::new(2);
    observers(&mut t);
    (k.c.setup)(&mut t);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    (k.c.start)(&mut t, bolt);
    let copies = resolve_until_copies(&mut t);
    assert!(!copies.is_empty(), "{name}");
    // The copies are above the Bolt (it is still on the stack, beneath them).
    let pos = |t: &TestGame, id: ObjectId| t.g.stack.iter().position(|x| *x == id);
    let b = pos(&t, bolt).unwrap_or_else(|| panic!("{name}: the Bolt left the stack first"));
    assert!(copies.iter().all(|c| pos(&t, *c).unwrap() > b), "{name}");
    let all = resolve_collecting_copies(&mut t);
    assert_eq!(all.len(), k.copies, "{name}: copies");
    assert_eq!(t.life(P1), 20 - 3 * (1 + k.copies as i32), "{name}");
    assert_eq!(
        t.g.history.spells_cast.len(),
        k.casts,
        "{name}: spells cast"
    );
    assert_eq!(
        t.life(P0),
        20 - 2 * k.casts as i32 - k.activations,
        "{name}: Spellshock and Burning-Tree Shaman triggered only for what P0 cast and activated"
    );
}

#[test]
fn copies_of_spells_arent_cast() {
    cr!("707.10", "603.2", "601.2");
    ruling!(
        "Display of Power",
        "The copies are created on the stack, so it's not \"cast.\" Creating the copies won't cause abilities that trigger when a player casts a spell to trigger."
    );
    ruling!(
        "Tempt with Mayhem",
        "The copies are created on the stack, so they're not \"cast.\" Creating a copy won't cause abilities that trigger when a player casts a spell to trigger."
    );
    ruling!(
        "Jin-Gitaxias, Progress Tyrant",
        "The copies that Jin-Gitaxias's first ability creates are created on the stack, so they're not “cast.” Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Kalamax, the Stormsire",
        "The copies that Kalamax's first ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Najal, the Storm Runner",
        "The copies that Najal's ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Twinferno",
        "The copies that Twinferno's ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Lutri, the Spellchaser",
        "The copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger. It will resolve before the original spell does."
    );
    ruling!(
        "Twinning Staff",
        "The copy is created on the stack, so it's not “cast.” Abilities that trigger when a player casts a spell won't trigger. It will resolve before the original spell does."
    );
    ruling!(
        "Swarm Intelligence",
        "The copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell (such as Swarm Intelligence’s own ability) won’t trigger."
    );
    ruling!(
        "Mirari",
        "The copy is not “cast” so it will not trigger anything that triggers on a spell being cast."
    );
    ruling!(
        "Bonus Round",
        "The copy that Bonus Round’s ability creates is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell (like Bonus Round’s ability itself) won’t trigger."
    );
    ruling!(
        "Double Vision",
        "The copy that Double Vision's ability creates is created on the stack, so it's not “cast.” Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Doublecast",
        "The copy that Doublecast’s ability creates is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    ruling!(
        "Galvanic Iteration",
        "The copy that Galvanic Iteration's ability creates is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    ruling!(
        "Fork",
        "The copy that is placed on the stack is not considered to have been \"cast\"."
    );
    // Swarm Intelligence and Mirari would be answered "yes" (and Mirari could pay) again if
    // they triggered for the copy.
    let swarm = Copier {
        name: "Swarm Intelligence",
        setup: SWARM_INTELLIGENCE.setup,
        start: |t, _| {
            for _ in 0..3 {
                t.answer_yes(P0, true);
                keep_copy_targets(t, P0);
            }
        },
    };
    let mirari = Copier {
        name: "Mirari",
        setup: |t| {
            (MIRARI.setup)(t);
            t.lands(P0, "Wastes", 6);
        },
        start: |t, _| {
            for _ in 0..3 {
                t.answer_yes(P0, true);
                keep_copy_targets(t, P0);
            }
        },
    };
    let cases = [
        case(DISPLAY_OF_POWER, 2, 0, 1),
        case(TEMPT_WITH_MAYHEM, 2, 0, 1),
        case(JIN_GITAXIAS, 1, 0, 1),
        case(KALAMAX, 1, 0, 1),
        // Najal's attack trigger is not activated.
        case(NAJAL, 1, 0, 1),
        case(TWINFERNO, 2, 0, 1),
        case(LUTRI, 2, 0, 1),
        // The Staff's ability is activated; its own replacement effect makes two copies.
        case(TWINNING_STAFF, 1, 1, 2),
        case(swarm, 1, 0, 1),
        case(mirari, 1, 0, 1),
        // Bonus Round itself was cast earlier.
        case(BONUS_ROUND, 2, 0, 1),
        case(DOUBLE_VISION, 1, 0, 1),
        case(DOUBLECAST, 2, 0, 1),
        case(GALVANIC_ITERATION, 2, 0, 1),
        case(FORK, 2, 0, 1),
    ];
    for k in &cases {
        bolt_copies_arent_cast(k);
    }
}

#[test]
fn finale_of_promise_s_copies_arent_cast() {
    cr!("707.10", "603.2", "608.2g");
    ruling!(
        "Finale of Promise",
        "The copies that Finale of Promise creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Finale of Promise");
    // X = 10: P0 casts the Lightning Bolt from the graveyard (a cast) and copies it twice.
    let mut t = TestGame::new(2);
    observers(&mut t);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 10);
    let f = t.hand(P0, "Finale of Promise");
    t.cast(P0, f).x(10).target(bolt).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    for _ in 0..2 {
        keep_copy_targets(&mut t, P0);
    }
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert_eq!(t.life(P1), 20 - 9);
    // Finale and the Bolt were cast; the copies weren't.
    assert_eq!(t.g.history.spells_cast.len(), 2);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn copies_made_for_each_other_creature_arent_cast_and_dont_retrigger() {
    cr!("707.10", "603.2", "707.10d");
    ruling!(
        "Precursor Golem",
        "The copies that Precursor Golem's second ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell (like Precursor Golem's second ability itself) won't trigger."
    );
    ruling!(
        "Mirrorwing Dragon",
        "The copies that the ability creates are created on the stack, so they're not cast. Abilities that trigger when a player casts a spell (like Mirrorwing Dragon's ability itself) won't trigger."
    );
    ruling!(
        "Zada, Hedron Grinder",
        "The copies that the ability creates are created on the stack, so they're not cast. Abilities that trigger when a player casts a spell (like Zada's ability itself) won't trigger."
    );
    ruling!(
        "Beamsplitter Mage",
        "The copy is created on the stack, so it’s not cast. Abilities that trigger when a player casts a spell (like Beamsplitter Mage’s ability itself) won’t trigger."
    );
    supported("Mirror Gallery");
    // Each setup returns the creature the Giant Growth targets and the copies expected. A
    // copy targets another such creature, whose own ability would trigger (and copy the
    // copy back) if the copy were cast.
    type Setup = fn(&mut TestGame) -> (ObjectId, usize);
    let cases: [(&str, Setup); 4] = [
        ("Precursor Golem", |t| {
            // Precursor Golem and its two Golem tokens: a spell at one token is copied for
            // the other two Golems.
            let golem = t.enter(P0, "Precursor Golem");
            t.resolve_all();
            let token =
                t.g.permanents()
                    .find(|o| o.is_token() && o.chars.subtypes.iter().any(|s| s == "Golem"))
                    .map(|o| o.id)
                    .unwrap();
            assert_ne!(token, golem);
            (token, 2)
        }),
        ("Mirrorwing Dragon", |t| {
            let a = t.battlefield(P0, "Mirrorwing Dragon");
            t.battlefield(P0, "Mirrorwing Dragon");
            (a, 1)
        }),
        ("Zada, Hedron Grinder", |t| {
            // Mirror Gallery: "The "legend rule" doesn't apply."
            t.battlefield(P0, "Mirror Gallery");
            let a = t.battlefield(P0, "Zada, Hedron Grinder");
            t.battlefield(P0, "Zada, Hedron Grinder");
            (a, 1)
        }),
        ("Beamsplitter Mage", |t| {
            let a = t.battlefield(P0, "Beamsplitter Mage");
            let b = t.battlefield(P0, "Beamsplitter Mage");
            for _ in 0..3 {
                t.answer_choose(P0, &[obj(b)]);
                t.answer_choose(P0, &[obj(a)]);
            }
            (a, 1)
        }),
    ];
    for (name, setup) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        observers(&mut t);
        let (target, want) = setup(&mut t);
        assert_eq!(
            t.named_on_battlefield(name).len(),
            if name == "Precursor Golem" { 1 } else { 2 },
            "{name}"
        );
        cast_new(&mut t, P0, "Giant Growth", &[obj(target)]);
        let copies = resolve_collecting_copies(&mut t);
        assert_eq!(copies.len(), want, "{name}: copies");
        assert_eq!(t.g.history.spells_cast.len(), 1, "{name}");
        assert_eq!(t.life(P0), 18, "{name}: Spellshock triggered once");
    }
}

/// P0's Prodigal Pyromancer ("{T}: This creature deals 1 damage to any target.") at P1.
/// Returns the ability on the stack.
fn ping_p1(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(t, P0, pyro, "damage").unwrap().unwrap();
    t.settle();
    (pyro, ping)
}

#[test]
fn lithoform_engine_s_copies_arent_cast_or_activated_or_triggered_additional_times() {
    cr!("707.10", "603.2", "602.2", "603.2d");
    ruling!(
        "Lithoform Engine",
        "The copy is created on the stack, so it's not “cast” or “activated.” Creating the copy won't cause abilities that trigger when a player casts a spell or activates an ability to trigger. Abilities that say that a triggered ability triggers additional times won't apply to copying a triggered ability."
    );
    supported("Lithoform Engine");
    supported("Panharmonicon");
    // A spell: one cast (the Bolt), one activation (the Engine's).
    bolt_copies_arent_cast(&case(LITHOFORM_ENGINE, 1, 1, 1));
    // An activated ability: the Pyromancer's and the Engine's were activated; the copy
    // wasn't.
    let mut t = TestGame::new(2);
    observers(&mut t);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let (_, ping) = ping_p1(&mut t);
    t.resolve(); // Burning-Tree Shaman's trigger.
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(ping)]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
    // A triggered ability that triggered twice (Panharmonicon: "If an artifact or creature
    // entering the battlefield causes a triggered ability of a permanent you control to
    // trigger, that ability triggers an additional time."): Elvish Visionary's "When this
    // creature enters, draw a card." is copied once, not twice.
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.battlefield(P0, "Panharmonicon");
    let visionary = t.enter(P0, "Elvish Visionary");
    t.settle();
    let triggers = abilities_from(&t, visionary);
    assert_eq!(triggers.len(), 2);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(triggers[1])]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    t.resolve();
    t.settle();
    assert_eq!(abilities_from(&t, visionary).len(), 3);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn return_the_favor_s_copy_isnt_cast_or_activated() {
    cr!("707.10", "603.2", "602.2");
    ruling!(
        "Return the Favor",
        "The copy made by the effect of Return the Favor's first mode is created on the stack, so it's not \"cast\" or \"activated.\""
    );
    supported("Return the Favor");
    // A spell: the Bolt and Return the Favor were cast.
    bolt_copies_arent_cast(&case(RETURN_THE_FAVOR, 2, 0, 1));
    // An activated ability: Return the Favor was cast, the Pyromancer's ability activated.
    let mut t = TestGame::new(2);
    observers(&mut t);
    let (_, ping) = ping_p1(&mut t);
    (RETURN_THE_FAVOR.start)(&mut t, ping);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20 - 1 - 2);
}

#[test]
fn unbound_flourishing_s_copy_isnt_cast_or_activated() {
    cr!("707.10", "603.2", "602.2", "107.3");
    ruling!(
        "Unbound Flourishing",
        "The copy that Unbound Flourishing's ability creates is created on the stack, so it's not \"cast\" or \"activated.\" Abilities that trigger when a player casts a spell or activates an ability won't trigger."
    );
    supported("Unbound Flourishing");
    supported("Helix Pinnacle");
    // Blaze ({X}{R}: "Blaze deals X damage to any target.") with X = 2 at P1: copied once,
    // the copy not cast.
    let mut t = TestGame::new(2);
    observers(&mut t);
    t.battlefield(P0, "Unbound Flourishing");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(2).target(P1).go();
    keep_copy_targets(&mut t, P0);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 18);
    // Helix Pinnacle ("{X}: Put X tower counters on this enchantment.") with X = 2: the
    // copy isn't activated.
    let mut t = TestGame::new(2);
    observers(&mut t);
    t.battlefield(P0, "Unbound Flourishing");
    let pinnacle = t.battlefield(P0, "Helix Pinnacle");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, pinnacle, "tower").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(pinnacle, "tower"), 4);
    assert_eq!(t.life(P0), 19);
}
