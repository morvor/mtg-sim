//! Rulings batch P029 — who controls a copy of a spell (CR 707.10, 707.10d), the order of
//! several copies (CR 707.10d, 405.3), choosing new targets for a copy (CR 707.10c, 115.7),
//! and costs: a copy's additional costs aren't paid but effects that depend on the
//! original's are copied (CR 707.10, 601.2b, 602.2b).

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::{fling_giant, resolve_until_copies};
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- New targets --------------------------------------------------------------------------------

/// A copier for the target tests: `start` (after `setup` and the original spell) makes the
/// copy, `chooser` choosing its targets with `new` (`None` keeps all of them).
struct Retarget {
    name: &'static str,
    caster: PlayerId,
    chooser: PlayerId,
    setup: fn(&mut TestGame),
    start: fn(&mut TestGame, ObjectId, Option<&[Option<Entity>]>),
}

/// Queues `p`'s new-targets answers: keep them all (`None`) or change them as listed.
fn new_targets(t: &mut TestGame, p: PlayerId, new: Option<&[Option<Entity>]>) {
    match new {
        None => keep_copy_targets(t, p),
        Some(n) => change_copy_targets(t, p, n),
    }
}

const RT_THOUSAND_YEAR_STORM: Retarget = Retarget {
    name: "Thousand-Year Storm",
    caster: P0,
    chooser: P0,
    setup: THOUSAND_YEAR_STORM.setup,
    start: |t, _, new| new_targets(t, P0, new),
};

const RT_DISPLAY_OF_POWER: Retarget = Retarget {
    name: "Display of Power",
    caster: P0,
    chooser: P0,
    setup: nothing,
    start: |t, s, new| {
        cast_new(t, P0, "Display of Power", &[obj(s)]);
        new_targets(t, P0, new);
    },
};

const RT_RETURN_THE_FAVOR: Retarget = Retarget {
    name: "Return the Favor",
    caster: P0,
    chooser: P0,
    setup: nothing,
    start: |t, s, new| {
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 1);
        let card = t.hand(P0, "Return the Favor");
        t.cast(P0, card).modes(&[0]).target(s).go();
        new_targets(t, P0, new);
    },
};

const RT_MELETIS_CHARLATAN: Retarget = Retarget {
    name: "Meletis Charlatan",
    caster: P0,
    chooser: P0,
    setup: MELETIS_CHARLATAN.setup,
    start: |t, s, new| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        let c = t.named_on_battlefield("Meletis Charlatan")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, c, "copies it").unwrap();
        new_targets(t, P0, new);
    },
};

/// P1 casts the spell after P0's Bonus Round: P1 copies it and chooses the targets.
const RT_BONUS_ROUND: Retarget = Retarget {
    name: "Bonus Round",
    caster: P1,
    chooser: P1,
    setup: BONUS_ROUND.setup,
    start: |t, _, new| new_targets(t, P1, new),
};

/// A new target in the Agony Warp scenario: one of its four creatures, or the victim
/// (a player, never a legal target for "target creature").
#[derive(Clone, Copy)]
enum Pick {
    Creature(usize),
    Victim,
}

/// The Agony Warp scenario: the victim (the caster's opponent) controls Hill Giant A,
/// Grizzly Bears B, Craw Wurm C and Hill Giant D; the caster's Agony Warp ("Target
/// creature gets -3/-0 until end of turn. Target creature gets -0/-3 until end of turn.")
/// targets A and B. The copy's targets are changed with `new` (`None`: kept); returns the
/// game after everything resolved, [A, B, C, D] and the copy's targets.
fn warp(r: &Retarget, new: Option<&[Option<Pick>]>) -> (TestGame, [ObjectId; 4], Vec<Entity>) {
    supported(r.name);
    let mut t = TestGame::new(2);
    let victim = if r.caster == P0 { P1 } else { P0 };
    let ids = [
        t.battlefield(victim, "Hill Giant"),
        t.battlefield(victim, "Grizzly Bears"),
        t.battlefield(victim, "Craw Wurm"),
        t.battlefield(victim, "Hill Giant"),
    ];
    (r.setup)(&mut t);
    lands_for_cost(&mut t, r.caster, "Agony Warp");
    let card = t.hand(r.caster, "Agony Warp");
    let spell = t.cast(r.caster, card).target(ids[0]).target(ids[1]).go();
    let new: Option<Vec<Option<Entity>>> = new.map(|n| {
        n.iter()
            .map(|p| {
                p.map(|p| match p {
                    Pick::Creature(i) => obj(ids[i]),
                    Pick::Victim => Entity::Player(victim),
                })
            })
            .collect()
    });
    (r.start)(&mut t, spell, new.as_deref());
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", r.name);
    assert_eq!(t.obj(copies[0]).controller, r.chooser, "{}", r.name);
    let targets = targets_of(&t, copies[0]);
    t.resolve_all();
    (t, ids, targets)
}

/// Any number of the copy's targets may be changed, none, some or all; new targets must
/// be legal.
fn change_any_number_of_targets(r: &Retarget) {
    let name = r.name;
    // None: the copy keeps A and B.
    let (t, [a, b, c, d], targets) = warp(r, None);
    assert_eq!(targets, vec![obj(a), obj(b)], "{name}");
    assert_eq!(t.pt(a).0, -3, "{name}");
    assert!(!t.on_battlefield(b));
    assert_eq!((t.pt(c), t.pt(d)), ((6, 4), (3, 3)), "{name}");
    // One: the copy's second target is the Wurm.
    let (t, [a, _, c, d], targets) = warp(r, Some(&[None, Some(Pick::Creature(2))]));
    assert_eq!(targets, vec![obj(a), obj(c)], "{name}");
    assert_eq!((t.pt(a).0, t.pt(c), t.pt(d)), (-3, (6, 1), (3, 3)), "{name}");
    // All: D and the Wurm.
    let (t, [a, _, c, d], targets) = warp(r, Some(&[Some(Pick::Creature(3)), Some(Pick::Creature(2))]));
    assert_eq!(targets, vec![obj(d), obj(c)], "{name}");
    assert_eq!((t.pt(a).0, t.pt(c), t.pt(d)), (0, (6, 1), (0, 3)), "{name}");
}

/// A new target must be legal: a player for "target creature" is rejected and the target
/// stays unchanged.
fn new_targets_must_be_legal(r: &Retarget) {
    let (_, [a, b, _, _], targets) = warp(r, Some(&[None, Some(Pick::Victim)]));
    assert_eq!(targets, vec![obj(a), obj(b)], "{}", r.name);
}

#[test]
fn copies_may_change_any_number_of_targets_to_legal_ones() {
    cr!("707.10c", "115.7", "115.7d");
    ruling!(
        "Thousand-Year Storm",
        "The copies will have the same targets as the spell they're copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    ruling!(
        "Return the Favor",
        "The copy created by the effect of Return the Favor's first mode will have the same targets as the spell or ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    ruling!(
        "Bonus Round",
        "The copy will have the same targets as the spell it’s copying unless its controller chooses new ones. That player may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    ruling!(
        "Bonus Round",
        "The controller of the spell that caused Bonus Round’s ability to trigger also controls the copy."
    );
    for r in [RT_THOUSAND_YEAR_STORM, RT_RETURN_THE_FAVOR, RT_BONUS_ROUND] {
        change_any_number_of_targets(&r);
        new_targets_must_be_legal(&r);
    }
}

/// P0's Giant Growth at P0's Bears; `r` copies it after the Bears died, with no other
/// creature to choose: the copy keeps the (illegal) Bears and is countered on resolution.
fn keeps_an_illegal_target(r: &Retarget) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    (r.setup)(&mut t);
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    (r.start)(&mut t, growth, Some(&[Some(Entity::Player(P1))]));
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", r.name);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)], "{}", r.name);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

#[test]
fn a_target_with_no_legal_replacement_stays_unchanged_even_if_illegal() {
    cr!("707.10c", "115.7", "608.2b");
    ruling!(
        "Display of Power",
        "The copies will have the same targets as the spells they're copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal. If, for any target, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    ruling!(
        "Meletis Charlatan",
        "The copy will have the same targets as the spell it's copying unless its controller chooses new ones. That player may change any number of the targets, including all of them or none of them. If, for one of the targets, that player can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    for r in [RT_DISPLAY_OF_POWER, RT_MELETIS_CHARLATAN] {
        change_any_number_of_targets(&r);
        new_targets_must_be_legal(&r);
        keeps_an_illegal_target(&r);
    }
    // Meletis Charlatan's copy of P1's spell: P1 controls it and chooses.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    (MELETIS_CHARLATAN.setup)(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let c = t.named_on_battlefield("Meletis Charlatan")[0];
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(&mut t, P0, c, "copies it").unwrap();
    change_copy_targets(&mut t, P1, &[Some(obj(giant))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(t.obj(copies[0]).controller, P1);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn spinerock_tyrant_s_copy_may_get_a_new_legal_target() {
    cr!("707.10c", "115.7");
    ruling!(
        "Spinerock Tyrant",
        "The copy will have the same target unless you choose new ones. If you change the target, the new target must be legal."
    );
    supported("Spinerock Tyrant");
    // "Whenever you cast an instant or sorcery spell with a single target, you may copy
    // it. If you do, those spells gain wither." Giant Growth at the Bears.
    for (new, want) in [(None, 0), (Some(1), 1), (Some(2), 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Spinerock Tyrant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.answer_yes(P0, true);
        let choices = [obj(bears), obj(giant), Entity::Player(P1)];
        match new {
            None => keep_copy_targets(&mut t, P0),
            Some(i) => change_copy_targets(&mut t, P0, &[Some(choices[i])]),
        }
        let copies = resolve_until_copies(&mut t);
        assert_eq!(targets_of(&t, copies[0]), vec![choices[want]], "{new:?}");
    }
}

#[test]
fn mirror_sheen_copies_another_player_s_spell_and_may_change_any_of_its_targets() {
    cr!("707.10", "707.10c", "115.7");
    ruling!("Mirror Sheen", "It doesn’t matter who controls the spell.");
    ruling!(
        "Mirror Sheen",
        "The copy will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them."
    );
    // P1's Lightning Bolt at P0: P0 copies it and aims the copy at P1.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    (MIRROR_SHEEN.setup)(&mut t);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Island", 2);
    let sheen = t.named_on_battlefield("Mirror Sheen")[0];
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(&mut t, P0, sheen, "Copy target").unwrap();
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(t.obj(copies[0]).controller, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 17));
    // P1's Electrolyze at P0 and P0's Bears (1 damage each): none, one or both targets
    // changed (to P1 and P1's Hill Giant).
    for (new, p1_life) in [
        (vec![None, None], 20),
        (vec![Some(1usize), None], 19),
        (vec![Some(1), Some(0)], 19),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        (MIRROR_SHEEN.setup)(&mut t);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
        lands_for_cost(&mut t, P1, "Electrolyze");
        let card = t.hand(P1, "Electrolyze");
        let e = t
            .cast(P1, card)
            .targets(&[Entity::Player(P0), obj(bears)])
            .go();
        let alts = [obj(giant), Entity::Player(P1)];
        let new: Vec<Option<Entity>> = new.iter().map(|i| i.map(|i| alts[i])).collect();
        t.lands(P0, "Wastes", 1);
        t.lands(P0, "Island", 2);
        let sheen = t.named_on_battlefield("Mirror Sheen")[0];
        t.answer_targets(P0, &[obj(e)]);
        activate_containing(&mut t, P0, sheen, "Copy target").unwrap();
        if new.iter().all(|n| n.is_none()) {
            keep_copy_targets(&mut t, P0);
        } else {
            change_copy_targets(&mut t, P0, &new);
        }
        let copies = resolve_until_copies(&mut t);
        let want: Vec<Entity> = [Entity::Player(P0), obj(bears)]
            .iter()
            .zip(&new)
            .map(|(old, n)| n.unwrap_or(*old))
            .collect();
        assert_eq!(targets_of(&t, copies[0]), want);
        t.resolve_all();
        assert_eq!(t.life(P1), p1_life, "{new:?}");
    }
}

// --- Control and order of copies ----------------------------------------------------------------

/// The last order question asked since `from`: who was asked and the items.
fn order_asked(t: &TestGame, from: usize) -> Option<(PlayerId, Vec<String>)> {
    t.asked()[from..].iter().rev().find_map(|(p, d)| match d {
        Decision::Order { items, .. } => Some((*p, items.clone())),
        _ => None,
    })
}

/// After a "copy for each other creature" trigger made `n` copies of `original` for
/// `controller`, who was asked to order them and answered reversed: the copies are
/// `controller`'s, in that order above the original, which resolves last.
fn copies_ordered_above_the_original(
    t: &mut TestGame,
    from: usize,
    original: ObjectId,
    controller: PlayerId,
    n: usize,
) {
    let copies = resolve_until_copies(t);
    assert_eq!(copies.len(), n);
    for c in &copies {
        assert_eq!(t.obj(*c).controller, controller);
    }
    let (asked, items) = order_asked(t, from).expect("the copies were ordered");
    assert_eq!(asked, controller);
    // Answered reversed: the last item is put onto the stack first (the bottom copy).
    let bottom_first: Vec<String> = copies
        .iter()
        .map(|c| format!("{:?}", targets_of(t, *c)[0]))
        .collect();
    assert_eq!(bottom_first.len(), items.len());
    for (got, item) in bottom_first.iter().zip(items.iter().rev()) {
        assert!(item.contains(got.as_str()), "{item} vs {got}");
    }
    let pos = |t: &TestGame, id: ObjectId| t.g.stack.iter().position(|x| *x == id).unwrap();
    assert!(copies.iter().all(|c| pos(t, *c) > pos(t, original)));
    // The original resolves last.
    while t.g.stack.len() > 1 {
        t.resolve();
    }
    assert_eq!(t.g.stack, vec![original]);
}

#[test]
fn the_caster_controls_and_orders_precursor_golem_s_and_mirrorwing_dragon_s_copies() {
    cr!("707.10d", "707.10", "405.5");
    ruling!(
        "Precursor Golem",
        "The controller of the spell that caused Precursor Golem's second ability to trigger also controls all the copies. That player chooses the order the copies are put onto the stack. The original spell will be on the stack beneath those copies and will resolve last."
    );
    ruling!(
        "Mirrorwing Dragon",
        "The player who cast the original spell controls all the copies. That player chooses the order the copies are put onto the stack. The original spell will be on the stack beneath those copies and will resolve last."
    );
    // P1's Lightning Bolt at P0's Precursor Golem: two copies (for the two Golem tokens),
    // P1's.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let golem = t.enter(P0, "Precursor Golem");
    t.resolve_all();
    t.answer(P1, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    let from = t.asked().len();
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(golem)]);
    copies_ordered_above_the_original(&mut t, from, bolt, P1, 2);
    // P1's Lightning Bolt at P0's Mirrorwing Dragon: a copy for each other creature P1
    // controls (two Bears), P1's.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer(P1, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    let from = t.asked().len();
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(dragon)]);
    copies_ordered_above_the_original(&mut t, from, bolt, P1, 2);
}

#[test]
fn mirrorwing_dragon_s_copies_target_the_caster_s_creatures() {
    cr!("707.10d", "707.10");
    ruling!(
        "Mirrorwing Dragon",
        "The copies are only created targeting creatures that the spell's controller controls. Copies are not created for all creatures on the battlefield, and the affected creatures may be controlled by a different player than the controller of Mirrorwing Dragon. Notably, if you cast Murder targeting your opponent's Mirrorwing Dragon, your creatures will each get a Murder, not your opponent's."
    );
    supported("Murder");
    // P0 casts Murder at P1's Mirrorwing Dragon: P0's Bears and Giant each get a Murder;
    // P1's other creature doesn't.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Mirrorwing Dragon");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Murder", &[obj(dragon)]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| t.obj(*c).controller == P0));
    t.resolve_all();
    assert!(!t.on_battlefield(dragon));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn ink_treader_nephilim_s_controller_controls_and_orders_a_copy_for_each_creature() {
    cr!("707.10d", "707.10", "603.3a");
    ruling!(
        "Ink-Treader Nephilim",
        "Ink-Treader Nephilim's ability is mandatory and will create a copy targeting each possible creature whether you want it to or not. You control all the copies of the spell (even if you didn't control the original spell), so you choose the order to put them on the stack."
    );
    // P1's Lightning Bolt at P0's Ink-Treader Nephilim: a copy for each other creature on
    // the battlefield (P0's Bears and P1's Hill Giant), all P0's, ordered by P0.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let ink = t.battlefield(P0, "Ink-Treader Nephilim");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    // No "may" to decline: a "no" queued for P0 isn't asked for.
    t.answer_yes(P0, false);
    let from = t.asked().len();
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(ink)]);
    copies_ordered_above_the_original(&mut t, from, bolt, P0, 2);
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant) && !t.on_battlefield(ink));
}

// --- Costs ---------------------------------------------------------------------------------------

/// The optional cost questions (kicker) asked since `from`.
fn optional_costs_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count()
}

/// The copier copies Fling (the Hill Giant sacrificed: 3 damage) and Burst Lightning
/// (kicked: 4 damage; not kicked: 2): the copy deals the same damage, nothing more is
/// sacrificed and no kicker is offered for the copy.
fn copies_keep_the_paid_additional_costs(c: &Copier) {
    let name = c.name;
    // Fling.
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fling = fling_giant(&mut t, P0, Entity::Player(P1));
    (c.start)(&mut t, fling);
    assert_eq!(resolve_until_copies(&mut t).len(), 1, "{name}");
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "{name}");
    assert!(t.on_battlefield(bears), "{name}: nothing more was sacrificed");
    // Burst Lightning ("Kicker {4}. Burst Lightning deals 2 damage to any target. If this
    // spell was kicked, it deals 4 damage instead."), kicked or not.
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        (c.setup)(&mut t);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 8);
        let card = t.hand(P0, "Burst Lightning");
        let from = t.asked().len();
        let burst = t.cast(P0, card).kicked(kicked).target(P1).go();
        (c.start)(&mut t, burst);
        assert_eq!(resolve_until_copies(&mut t).len(), 1, "{name}");
        t.resolve_all();
        let d = if kicked { 4 } else { 2 };
        assert_eq!(t.life(P1), 20 - 2 * d, "{name}: kicked {kicked}");
        assert_eq!(optional_costs_asked(&t, from), 1, "{name}: kicker offered once");
    }
}

#[test]
fn a_copy_s_additional_costs_arent_paid_but_the_original_s_count() {
    cr!("707.10", "601.2b", "702.33d");
    ruling!(
        "Return the Favor",
        "If you copy a spell with the first mode, you can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    ruling!(
        "Bonus Round",
        "The controller of the copy can’t choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Fling");
    supported("Burst Lightning");
    copies_keep_the_paid_additional_costs(&RETURN_THE_FAVOR);
    copies_keep_the_paid_additional_costs(&BONUS_ROUND);
}

#[test]
fn a_copied_ability_uses_the_original_s_cost_choices() {
    cr!("707.10", "602.2b");
    ruling!(
        "Return the Favor",
        "If you copy an activated ability with the first mode, and the activation cost includes a choice, such as a creature to sacrifice or a number of counters to remove, the copy uses that same information. You can't pay the cost again, even if you want to."
    );
    supported("Brion Stoutarm");
    // Brion Stoutarm: "{R}, {T}, Sacrifice another creature: Brion Stoutarm deals damage
    // equal to the sacrificed creature's power to target player or planeswalker." The Hill Giant (3) is sacrificed; the copy deals 3 too and sacrifices
    // nothing (the Bears stay).
    let mut t = TestGame::new(2);
    let brion = t.battlefield(P0, "Brion Stoutarm");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ability = t.activate(P0, brion, 0, &[]).unwrap().unwrap();
    assert!(!t.on_battlefield(giant));
    // Bears offered for any second sacrifice: never asked.
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    (RETURN_THE_FAVOR.start)(&mut t, ability);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(bears));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
}
