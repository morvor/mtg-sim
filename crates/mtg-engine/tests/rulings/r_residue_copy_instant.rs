//! Rulings residue — copying spells on the stack (batches P028, P030, P032): what a copy
//! copies and what it doesn't (CR 707.10: targets, modes, X, kicker, splice, divided
//! damage; not text-changing effects, CR 707.2), new targets for copies (CR 707.10c, 115.7),
//! copies aren't cast (CR 707.10), costs paid on resolution (CR 608.2), copies of a spell
//! that targets a spell (CR 115.5), the copying triggers (CR 603.2, 603.4, 115.9c), the
//! controller of copies and their order on the stack (CR 707.10, 405.3), changing a spell's
//! target (CR 115.7), and God-Eternal Kefnet's copies of cards (CR 707.12, 608.2h).

use crate::r_p028_common::{obj, Copier};
use crate::r_p029_common::{resolve_collecting_copies, FORK, TEMPT_WITH_MAYHEM, TWINNING_STAFF};
use crate::r_p030_common::{fling_giant, resolve_until_copies};
use crate::r_s01_common::supported;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The decisions asked of anyone since `from` other than "choose new targets" and new
/// targets themselves: the choices a copy could have let its controller make.
fn other_choices(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .map(|(_, d)| d.clone())
        .filter(|d| {
            !matches!(
                d,
                Decision::Priority { .. } | Decision::YesNo { .. } | Decision::ChooseTargets { .. }
            )
        })
        .map(|d| format!("{d:?}"))
        .collect()
}

// --- Fork --------------------------------------------------------------------------------------

#[test]
fn fork_s_copy_uses_the_same_number_of_targets() {
    cr!("707.10", "707.10c", "601.2c");
    ruling!(
        "Fork",
        "For spells that can have a variable number of targets, the controller of the copy must use the same number of targets the original spell did."
    );
    supported("Fork");
    supported("Electrolyze");
    // Electrolyze ("2 damage divided as you choose among one or two targets") with one
    // target, P1: the copy has one target too, which becomes the Bears. Its controller is
    // asked for one new target (instead of P1), never for a second one.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t.cast(P0, card).targets(&[pl(P1)]).go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Fork", &[obj(e)]);
    change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(new_target_choices(&t, from, copies[0]), vec![(0, 1)]);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
    // With two targets (the Bears and P1), the copy keeps two: one new target is asked
    // for each, and leaving P1 unchanged (no answer) keeps it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t.cast(P0, card).targets(&[obj(bears), pl(P1)]).go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Fork", &[obj(e)]);
    change_copy_targets(&mut t, P0, &[Some(obj(giant)), None]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(
        new_target_choices(&t, from, copies[0]),
        vec![(0, 1), (0, 1)]
    );
    assert_eq!(targets_of(&t, copies[0]), vec![obj(giant), pl(P1)]);
}

/// The (min, max) number of targets of each "new target" choice offered for the copy
/// `copy` since `from`.
fn new_target_choices(t: &TestGame, from: usize, copy: ObjectId) -> Vec<(u32, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                source, min, max, ..
            } if *source == copy => Some((*min, *max)),
            _ => None,
        })
        .collect()
}

#[test]
fn fork_s_copy_keeps_every_choice_but_the_targets() {
    cr!("707.10", "707.10c", "700.2", "601.2d");
    ruling!(
        "Fork",
        "Fork does not let you make non-targeting choices about the spell."
    );
    ruling!(
        "Fork",
        "You need not (and may not) pay any additional mana or other costs (like sacrifices) to use the spell which is Forked. You get control over a complete copy but can change nothing except the targets."
    );
    // Izzet Charm's second mode ("2 damage to target creature") at the Bears: the copy
    // has the same mode, and the only choices offered are its targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    let charm = t.cast(P0, card).modes(&[1]).target(bears).go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Fork", &[obj(charm)]);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(chosen_modes(&t, copies[0]), vec![1]);
    assert_eq!(other_choices(&t, from), Vec::<String>::new());
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 2);
    // Electrolyze divided 1 and 1 between the Bears and P1: the copy's division is kept
    // and no division is asked for.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t.cast(P0, card).targets(&[obj(bears), pl(P1)]).go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Fork", &[obj(e)]);
    change_copy_targets(&mut t, P0, &[Some(obj(giant)), None]);
    resolve_until_copies(&mut t);
    assert_eq!(other_choices(&t, from), Vec::<String>::new());
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(giant).damage, 1);
    // Fling (sacrifice a creature as an additional cost): the copy sacrifices nothing
    // more and deals damage equal to the power of the Hill Giant sacrificed for Fling.
    let mut t = TestGame::new(2);
    let fling = fling_giant(&mut t, P0, pl(P1));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Fork", &[obj(fling)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 14);
    assert_eq!(other_choices(&t, from), Vec::<String>::new());
}

#[test]
fn fork_s_copy_doesn_t_pay_the_spell_s_additional_costs_again() {
    cr!("707.10", "118.8", "601.2h");
    ruling!(
        "Fork",
        "When a spell with additional costs is copied, you don't have to pay those costs again."
    );
    // Village Rites ("As an additional cost to cast this spell, sacrifice a creature.
    // Draw two cards."): P1 casts it; P0's copy draws P0 two cards and sacrifices nothing.
    supported("Village Rites");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P1, &[obj(theirs)]);
    let rites = cast_new(&mut t, P1, "Village Rites", &[]);
    assert!(!t.on_battlefield(theirs));
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Fork", &[obj(rites)]);
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn fork_doesn_t_copy_a_text_changing_effect_on_the_spell() {
    cr!("707.2", "707.10", "612.1");
    ruling!(
        "Fork",
        "Fork will not copy changes made by modifying effects to the spell prior to the use of Fork, such as text-changing effects."
    );
    supported("Sleight of Mind");
    // P0's Doom Blade ("Destroy target nonblack creature.") at P1's Grizzly Bears; P1's
    // Sleight of Mind makes it "nongreen": its target is illegal. Fork's copy is the
    // printed Doom Blade and destroys the Bears.
    let color_idx = |c: Color, without: Option<Color>| {
        Color::ALL
            .iter()
            .filter(|x| Some(**x) != without)
            .position(|x| *x == c)
            .unwrap()
    };
    for fork in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let blade = cast_new(&mut t, P0, "Doom Blade", &[obj(bears)]);
        t.answer(
            P1,
            DecisionKind::Option,
            Answer::Index(color_idx(Color::Black, None)),
        );
        t.answer(
            P1,
            DecisionKind::Option,
            Answer::Index(color_idx(Color::Green, Some(Color::Black))),
        );
        cast_new(&mut t, P1, "Sleight of Mind", &[obj(blade)]);
        t.resolve();
        if fork {
            cast_new(&mut t, P0, "Fork", &[obj(blade)]);
            keep_copy_targets(&mut t, P0);
            let copies = resolve_until_copies(&mut t);
            assert_eq!(copies.len(), 1);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), !fork, "forked: {fork}");
    }
}

#[test]
fn fork_s_copy_has_the_same_x_and_is_kicked_if_the_spell_was() {
    cr!("707.10", "107.3", "702.33d");
    ruling!(
        "Fork",
        "Forking a spell with an X in the cost requires you to use the same X value."
    );
    ruling!(
        "Fork",
        "If the targeted spell was kicked, then the copy will be kicked as well. If the targeted spell was not kicked, then the copy will not be kicked."
    );
    supported("Blaze");
    // Blaze with X = 3 at P1: the copy deals 3 as well, and isn't asked for X.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, card).x(3).target(P1).go();
    let from = t.asked().len();
    (FORK.start)(&mut t, blaze);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(x_of(&t, copies[0]), Some(3));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(other_choices(&t, from), Vec::<String>::new());
    // Burst Lightning ("2 damage to any target. If this spell was kicked, it deals 4
    // damage instead."), kicked or not: the copy deals what the original does.
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 1);
        if kicked {
            t.lands(P0, "Wastes", 4);
        }
        let card = t.hand(P0, "Burst Lightning");
        let bl = t.cast(P0, card).kicked(kicked).target(P1).go();
        (FORK.start)(&mut t, bl);
        t.resolve_all();
        let each = if kicked { 4 } else { 2 };
        assert_eq!(t.life(P1), 20 - 2 * each, "kicked: {kicked}");
    }
}

#[test]
fn fork_copies_a_spliced_spell_with_the_spliced_text() {
    cr!("702.47a", "702.47c", "707.10");
    ruling!(
        "Fork",
        "If you Fork a Spliced spell, the spliced text is added during the announcement of the original spell, and therefore is fully copied by Fork."
    );
    supported("Lava Spike");
    supported("Glacial Ray");
    // Lava Spike (3 damage to target player) with Glacial Ray (2 damage to any target)
    // spliced onto it: the copy deals 3 + 2 too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let spike = t.hand(P0, "Lava Spike");
    t.hand(P0, "Glacial Ray");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let s = t.cast(P0, spike).targets(&[pl(P1)]).targets(&[pl(P1)]).go();
    (FORK.start)(&mut t, s);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 5 - 5);
}

#[test]
fn the_copy_s_controller_pays_the_costs_paid_on_resolution() {
    cr!("707.10", "608.2c", "107.14");
    ruling!(
        "Fork",
        "If mana or other costs need to be spent at resolution of the spell, the player of Fork would still be responsible for paying that cost."
    );
    supported("Harnessed Lightning");
    // P1's Harnessed Lightning ("Choose target creature. You get {E}{E}{E}, then you may
    // pay any amount of {E}. Harnessed Lightning deals that much damage to that
    // creature."): P0's copy gives P0 the energy and P0 pays it (two more P0 had).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[0].counters.insert("energy".into(), 2);
    let hl = cast_new(&mut t, P1, "Harnessed Lightning", &[obj(giant)]);
    cast_new(&mut t, P0, "Fork", &[obj(hl)]);
    change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer(P1, DecisionKind::X, Answer::Number(0));
    t.resolve();
    t.resolve();
    // The copy resolved: P0 paid all five.
    assert_eq!(t.g.player(P0).counter("energy"), 0);
    assert_eq!(t.g.player(P1).counter("energy"), 0);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter("energy"), 3);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn a_copy_of_a_spell_targeting_a_spell_may_target_fork_itself() {
    cr!("707.10c", "608.2b");
    ruling!(
        "Fork",
        "If the spell being copied targets a spell on the stack, it is possible to target Fork itself since Fork is still on the stack when you pick the target(s) for the copy. Note that the copy's target will be illegal when it resolves."
    );
    supported("Counterspell");
    // P0's Divination; P1's Counterspell at it; P0 Forks the Counterspell and points the
    // copy at Fork. Fork has left the stack by the time the copy resolves: the copy does
    // nothing, and the original Counterspell counters Divination.
    let mut t = TestGame::new(2);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    let cs = cast_new(&mut t, P1, "Counterspell", &[obj(div)]);
    let fork = cast_new(&mut t, P0, "Fork", &[obj(cs)]);
    change_copy_targets(&mut t, P0, &[Some(obj(fork))]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(fork)]);
    assert!(t.in_graveyard(P0, "Fork"));
    t.resolve();
    // The copy did nothing; Counterspell is still on the stack and counters Divination.
    assert_eq!(t.g.stack, vec![div, cs]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
}

// --- Tempt with Mayhem, Twinning Staff, Finale of Promise -------------------------------------

#[test]
fn tempt_with_mayhem_s_copies_may_each_get_new_targets() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Tempt with Mayhem",
        "Each of the copies will have the same targets as the spell it's copying unless its controller chooses new ones. That player may change any number of the targets, including all of them or none of them. The new targets must be legal. If, for any target, they can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Tempt with Mayhem");
    // P0's Lightning Bolt at P1. P1 accepts and points its copy at P0; P0's two copies:
    // one keeps P1, the other goes to P1's Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    cast_new(&mut t, P0, "Tempt with Mayhem", &[obj(bolt)]);
    t.answer_yes(P1, true);
    change_copy_targets(&mut t, P1, &[Some(pl(P0))]);
    keep_copy_targets(&mut t, P0);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 3);
    let mut by: Vec<(PlayerId, Vec<Entity>)> = copies
        .iter()
        .map(|c| (t.obj(*c).controller, targets_of(&t, *c)))
        .collect();
    by.sort();
    assert_eq!(
        by,
        vec![
            (P0, vec![pl(P1)]),
            (P0, vec![obj(giant)]),
            (P1, vec![pl(P0)]),
        ]
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(giant));
    // Giant Growth at P0's Bears, which then leave. The only other creature, P1's Wall of
    // Denial, has shroud: no copy can get a new legal target, so each keeps the illegal
    // Bears (both players try to choose the Wall instead) and does nothing.
    supported("Wall of Denial");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Denial");
    let gg = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.g.destroy(bears, None);
    t.g.flush_events();
    cast_new(&mut t, P0, "Tempt with Mayhem", &[obj(gg)]);
    t.answer_yes(P1, true);
    change_copy_targets(&mut t, P1, &[Some(obj(wall))]);
    change_copy_targets(&mut t, P0, &[Some(obj(wall))]);
    change_copy_targets(&mut t, P0, &[Some(obj(wall))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 3);
    for c in &copies {
        assert_eq!(targets_of(&t, *c), vec![obj(bears)]);
    }
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert_eq!(t.pt(wall), (0, 8));
}

/// P0 casts Izzet Charm choosing its second mode (2 damage to target creature) at P1's
/// Grizzly Bears; `c` copies it. Each copy has the same mode and no mode is asked for.
fn copies_keep_the_mode(c: &Copier, n: usize) {
    supported(c.name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    (c.setup)(&mut t);
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    let charm = t.cast(P0, card).modes(&[1]).target(bears).go();
    let from = t.asked().len();
    (c.start)(&mut t, charm);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), n, "{}", c.name);
    for cp in &copies {
        assert_eq!(chosen_modes(&t, *cp), vec![1], "{}", c.name);
    }
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })),
        "{}",
        c.name
    );
    assert!(!t.on_battlefield(bears));
}

#[test]
fn twinning_staff_s_copies_have_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Twinning Staff",
        "If the spell that's copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A different mode can't be chosen."
    );
    copies_keep_the_mode(&TWINNING_STAFF, 2);
}

/// P0 casts Finale of Promise with X = 10 (twelve lands), targeting the given cards in
/// P0's graveyard.
fn finale_x10(t: &mut TestGame, targets: &[ObjectId]) -> ObjectId {
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 10);
    let card = t.hand(P0, "Finale of Promise");
    let ts: Vec<Entity> = targets.iter().map(|o| obj(*o)).collect();
    t.cast(P0, card).x(10).targets(&ts).go()
}

#[test]
fn finale_of_promise_s_copies_have_the_same_modes() {
    cr!("707.10", "700.2", "601.2b");
    ruling!(
        "Finale of Promise",
        "If the spell that's copied is modal (that is, it says \"Choose one—\" or the like), the copies will have the same mode or modes. You can't choose different ones."
    );
    supported("Finale of Promise");
    // Izzet Charm (instant) cast with its second mode at the Bears: the two copies have
    // that mode too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let charm = t.graveyard(P0, "Izzet Charm");
    let from = t.asked().len();
    finale_x10(&mut t, &[charm]);
    t.answer_yes(P0, true); // cast it
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(bears)]);
    keep_copy_targets(&mut t, P0);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert_eq!(chosen_modes(&t, *c), vec![1]);
    }
    let modes_asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(modes_asked, 1, "only as the card was cast");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn finale_of_promise_s_free_cast_pays_mandatory_additional_costs() {
    cr!("118.9", "118.9a", "118.9d", "601.2b");
    ruling!(
        "Finale of Promise",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Spark Harvest, you must pay those to cast the card."
    );
    supported("Village Rites");
    // Village Rites (additional cost: sacrifice a creature) cast with Finale (X = 1): the
    // Bears are sacrificed, and two cards drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.graveyard(P0, "Village Rites");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Finale of Promise");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, card).x(1).targets(&[obj(rites)]).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Village Rites"));
    // With no creature to sacrifice, it can't be cast.
    let mut t = TestGame::new(2);
    let rites = t.graveyard(P0, "Village Rites");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Finale of Promise");
    t.answer_yes(P0, true);
    t.cast(P0, card).x(1).targets(&[obj(rites)]).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Village Rites"));
    // Burst Lightning: its optional kicker ({4}) may be paid; the kicked spell deals 4.
    let mut t = TestGame::new(2);
    let bl = t.graveyard(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Finale of Promise");
    t.cast(P0, card).x(1).targets(&[obj(bl)]).go();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Cyclonic Rift ("Return target nonland permanent you don't control to its owner's
    // hand. Overload {6}{U}"): overload, an alternative cost, isn't offered; the Rift is
    // cast normally and returns one of P1's two creatures.
    supported("Cyclonic Rift");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let rift = t.graveyard(P0, "Cyclonic Rift");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Island", 7);
    let card = t.hand(P0, "Finale of Promise");
    let from = t.asked().len();
    t.cast(P0, card).x(2).targets(&[obj(rift)]).go();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[obj(a)]);
    t.resolve_all();
    let methods: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(
        methods
            .iter()
            .flatten()
            .all(|o| !o.to_lowercase().contains("overload")),
        "{methods:?}"
    );
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}

// --- Beamsplitter Mage, Zada --------------------------------------------------------------------

#[test]
fn beamsplitter_mage_s_copy_is_made_even_if_the_mage_left() {
    cr!("603.4", "707.10", "113.7a", "608.2h", "608.2b");
    ruling!(
        "Beamsplitter Mage",
        "If Beamsplitter Mage leaves the battlefield or becomes an illegal target for the spell that caused its ability to trigger, that spell is still copied."
    );
    // Giant Growth at the Mage; in response the Mage dies: the Bears get the copy.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Beamsplitter Mage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.destroy(mage, None);
    t.g.flush_events();
    t.answer_choose(P0, &[obj(bears)]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // The Mage becomes an illegal target (it gains shroud) before the ability resolves:
    // the spell is still copied.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Beamsplitter Mage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        mage,
        vec![mtg_engine::ability::Modification::AddKeyword(
            mtg_engine::keywords::Keyword::new(mtg_engine::keywords::KeywordKind::Shroud),
        )],
    );
    t.answer_choose(P0, &[obj(bears)]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(mage), (2, 2));
}

#[test]
fn beamsplitter_mage_s_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Beamsplitter Mage",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Selective Snare does), the copy has the same value of X."
    );
    supported("Blaze");
    // Blaze with X = 1 at the Mage (a 2/2): the copy deals 1 to the Hill Giant.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Beamsplitter Mage");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(1).target(mage).go();
    t.answer_choose(P0, &[obj(giant)]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(x_of(&t, copies[0]), Some(1));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(t.obj_now(mage).damage, 1);
}

#[test]
fn zada_s_copies_are_yours_in_the_order_you_choose_above_the_original() {
    cr!("707.10", "707.10d", "405.3");
    ruling!(
        "Zada, Hedron Grinder",
        "You control all the copies. You choose the order the copies are put onto the stack. The original spell will be on the stack beneath those copies and will resolve last."
    );
    // P0's Giant Growth at P0's Zada, with P0's Bears and Hill Giant: two copies, P0's,
    // above the original. Both orders can be chosen.
    let mut tops = vec![];
    for order in [[0usize, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        let zada = t.battlefield(P0, "Zada, Hedron Grinder");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        let gg = cast_new(&mut t, P0, "Giant Growth", &[obj(zada)]);
        let copies = resolve_until_copies(&mut t);
        assert_eq!(copies.len(), 2);
        assert_eq!(t.g.stack[0], gg);
        assert!(copies.iter().all(|c| t.obj(*c).controller == P0));
        let top = targets_of(&t, *t.g.stack.last().unwrap());
        tops.push(top);
        // The copies resolve first; the original last.
        t.resolve();
        t.resolve();
        assert_eq!(t.g.stack, vec![gg]);
        assert_eq!(t.pt(bears), (5, 5));
        assert_eq!(t.pt(giant), (6, 6));
        assert_eq!(t.pt(zada), (3, 3));
        t.resolve();
        assert_eq!(t.pt(zada), (6, 6));
    }
    assert_ne!(tops[0], tops[1], "both orders can be chosen");
}

// --- Return the Favor ---------------------------------------------------------------------------

#[test]
fn return_the_favor_changes_the_target_on_resolution_if_it_can() {
    cr!("115.7", "115.7a", "608.2c");
    ruling!(
        "Return the Favor",
        "You don't choose the new target for the spell or ability targeted with Return the Favor's second mode until Return the Favor resolves. You must change the target if possible. However, you can't change the target to an illegal target. If there are no legal targets to choose from, the target isn't changed. It doesn't matter if the original target has somehow become illegal itself."
    );
    supported("Return the Favor");
    supported("Disfigure");
    let cast_rtf = |t: &mut TestGame, s: ObjectId| {
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 1);
        let card = t.hand(P0, "Return the Favor");
        t.cast(P0, card).modes(&[1]).target(s).go()
    };
    // P1's Disfigure (-2/-2 to target creature) at P0's Bears. A Hill Giant enters after
    // Return the Favor is cast: it's the new target, chosen on resolution.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dis = cast_new(&mut t, P1, "Disfigure", &[obj(bears)]);
    cast_rtf(&mut t, dis);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(targets_of(&t, dis), vec![obj(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(giant), (1, 1));
    // Declining to choose doesn't keep the target: it must change if possible.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let dis = cast_new(&mut t, P1, "Disfigure", &[obj(bears)]);
    cast_rtf(&mut t, dis);
    t.answer_targets(P0, &[]);
    t.resolve();
    assert_eq!(targets_of(&t, dis), vec![obj(giant)]);
    // The original target is gone (illegal): it still changes to the Giant.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let dis = cast_new(&mut t, P1, "Disfigure", &[obj(bears)]);
    cast_rtf(&mut t, dis);
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    assert_eq!(targets_of(&t, dis), vec![obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    // No other legal target (the only other creature, P1's Wall of Denial, has shroud):
    // the target isn't changed, and the Wall can't be chosen.
    supported("Wall of Denial");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Denial");
    let dis = cast_new(&mut t, P1, "Disfigure", &[obj(bears)]);
    cast_rtf(&mut t, dis);
    t.answer_targets(P0, &[obj(wall)]);
    t.resolve();
    assert_eq!(targets_of(&t, dis), vec![obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(wall), (0, 8));
}

// --- Ulalek, Fused Atrocity ---------------------------------------------------------------------

/// P0 casts Dimensional Infiltrator (an Eldrazi creature with flash and no targets) with
/// Ulalek on the battlefield and pays {C}{C} for Ulalek's ability (P0's Wastes).
pub fn eldrazi_spell(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    cast_new(t, P0, "Dimensional Infiltrator", &[])
}

#[test]
fn ulalek_s_copies_may_get_new_targets_even_an_aura_spell() {
    cr!("707.10c", "115.7d", "707.10f", "608.3c", "608.3f");
    ruling!(
        "Ulalek, Fused Atrocity",
        "Each copy will have the same targets as the spell or ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If the spell is a permanent spell with targets, such as an Aura, you may also choose a new target for that spell. The new targets must be legal. If, for any target, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Ulalek, Fused Atrocity");
    supported("Dimensional Infiltrator");
    // P0 casts Pacifism at P1's Craw Wurm and Lightning Bolt at P1, then an Eldrazi spell:
    // the copy of Pacifism gets P1's other Craw Wurm (all its targets changed), the Bolt's
    // copy keeps P1 (none changed), and the Infiltrator's copy has no targets.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    let pac = cast_new(&mut t, P0, "Pacifism", &[obj(a)]);
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    eldrazi_spell(&mut t);
    // The copies are made bottom first: Pacifism's, then the Bolt's.
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    keep_copy_targets(&mut t, P0);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 3);
    let copy_of = |t: &TestGame, name: &str| {
        *copies
            .iter()
            .find(|c| t.obj(**c).chars.name == name)
            .unwrap()
    };
    assert_eq!(targets_of(&t, copy_of(&t, "Pacifism")), vec![obj(b)]);
    assert_eq!(targets_of(&t, pac), vec![obj(a)]);
    assert_eq!(targets_of(&t, copy_of(&t, "Lightning Bolt")), vec![pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let pacifisms = |t: &TestGame, w: ObjectId| -> Vec<ObjectId> {
        t.g.attachments_of(obj(w))
            .iter()
            .copied()
            .filter(|x| t.obj(*x).chars.name == "Pacifism")
            .collect()
    };
    // The card is attached to the first Wurm; the copy resolved as a token Pacifism
    // attached to the second.
    let on_a = pacifisms(&t, a);
    let on_b = pacifisms(&t, b);
    assert_eq!((on_a.len(), on_b.len()), (1, 1));
    assert!(!t.obj(on_a[0]).is_token());
    assert!(t.obj(on_b[0]).is_token());
    // The new targets must be legal, and a target with no new legal one stays, even an
    // illegal one: P0's Shatter ("Destroy target artifact.") at P1's Ornithopter, which is
    // then destroyed (no artifact is left). P0 tries to give the copy P1's Craw Wurm
    // instead: the copy keeps the Ornithopter.
    supported("Shatter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let thopter = t.battlefield(P1, "Ornithopter");
    cast_new(&mut t, P0, "Shatter", &[obj(thopter)]);
    t.g.destroy(thopter, None);
    t.g.flush_events();
    eldrazi_spell(&mut t);
    change_copy_targets(&mut t, P0, &[Some(obj(wurm))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    let shatter = *copies
        .iter()
        .find(|c| t.obj(**c).chars.name == "Shatter")
        .unwrap();
    assert_eq!(targets_of(&t, shatter), vec![obj(thopter)]);
}

/// P0 casts `spell`, dividing it as `division` among `targets`, then an Eldrazi spell with
/// Ulalek on the battlefield; the copy of `spell` changes its first target to
/// `new_first`, and its controller would divide it as `other_division` if asked again.
/// Returns the spell, its copy and the number of divisions asked since the spell was
/// cast.
fn ulalek_copies_divided(
    t: &mut TestGame,
    spell: &str,
    targets: &[Entity],
    division: Vec<i64>,
    new_first: Entity,
    other_division: Vec<i64>,
) -> (ObjectId, ObjectId, usize) {
    supported(spell);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(division));
    lands_for_cost(t, P0, spell);
    let card = t.hand(P0, spell);
    let s = t.cast(P0, card).targets(targets).go();
    let from = t.asked().len();
    eldrazi_spell(t);
    let mut new: Vec<Option<Entity>> = vec![Some(new_first)];
    new.extend(targets[1..].iter().map(|_| None));
    change_copy_targets(t, P0, &new);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(other_division));
    let copies = resolve_until_copies(t);
    let copy = *copies
        .iter()
        .find(|c| t.obj(**c).chars.name == spell)
        .unwrap();
    let asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Divide { .. }))
        .count();
    (s, copy, asked)
}

/// The division of the spell or ability `id` on the stack, over its targets in order.
fn division_of(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.divided.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn ulalek_s_copies_keep_the_division_of_damage_and_counters() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Ulalek, Fused Atrocity",
        "If the spell or ability that's copied has damage divided as it was cast or activated, the division can't be changed (although the targets receiving that damage still can). The same is true of spells and abilities that distribute counters."
    );
    // Arc Lightning ("3 damage divided as you choose among one, two, or three targets"):
    // 1 to P1's Bears and 2 to P1. The copy gets the Craw Wurm instead of the Bears and
    // keeps the division (1 to the Wurm, 2 to P1); no division is asked for again.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let (arc, copy, asked) = ulalek_copies_divided(
        &mut t,
        "Arc Lightning",
        &[obj(bears), pl(P1)],
        vec![1, 2],
        obj(wurm),
        vec![2, 1],
    );
    assert_eq!(asked, 0);
    assert_eq!(division_of(&t, copy), vec![1, 2]);
    assert_eq!(division_of(&t, arc), vec![1, 2]);
    assert_eq!(targets_of(&t, copy), vec![obj(wurm), pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.obj_now(wurm).damage, 1);
    assert_eq!(t.obj_now(bears).damage, 1);
    // Defend the Celestus ("Distribute three +1/+1 counters among one, two, or three
    // target creatures you control."): 2 on P0's first Bears and 1 on the Hill Giant. The
    // copy puts its 2 on P0's second Bears instead, and 1 more on the Giant.
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Grizzly Bears");
    let second = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let (dtc, copy, asked) = ulalek_copies_divided(
        &mut t,
        "Defend the Celestus",
        &[obj(first), obj(giant)],
        vec![2, 1],
        obj(second),
        vec![1, 2],
    );
    assert_eq!(asked, 0);
    assert_eq!(division_of(&t, copy), vec![2, 1]);
    assert_eq!(division_of(&t, dtc), vec![2, 1]);
    assert_eq!(targets_of(&t, copy), vec![obj(second), obj(giant)]);
    t.resolve_all();
    let plus = |t: &TestGame, id: ObjectId| t.counters(id, "+1/+1");
    assert_eq!(
        (plus(&t, first), plus(&t, second), plus(&t, giant)),
        (2, 2, 2)
    );
}

#[test]
fn ulalek_s_copies_aren_t_cast() {
    cr!("707.10", "603.2", "601.2i", "602.2b");
    ruling!(
        "Ulalek, Fused Atrocity",
        "The copies are created on the stack, so they're not \"cast\" or \"activated.\" Creating the copies won't cause abilities that trigger when a player casts a spell or activates an ability to trigger. Abilities of copied spells that say \"When you cast [this spell]\" won't trigger."
    );
    supported("Desolation Twin");
    supported("Kambal, Consul of Allocation");
    // P1's Kambal ("Whenever an opponent casts a noncreature spell, that player loses 2
    // life and you gain 2 life."): P0's Electrolyze at P1, copied by Ulalek: P0 loses 2
    // once (the Infiltrator is a creature spell).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    t.battlefield(P1, "Kambal, Consul of Allocation");
    cast_new(&mut t, P0, "Electrolyze", &[pl(P1)]);
    eldrazi_spell(&mut t);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20 + 2 - 4);
    // Desolation Twin ("When you cast this spell, create a 10/10 colorless Eldrazi
    // creature token."), its cast trigger resolved (and Ulalek's ability for it not paid
    // for), then an Eldrazi spell: the Twin's copy doesn't make another 10/10.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    // (No mana is left to pay for Ulalek's ability as the Twin is cast.)
    let twin = cast_new(&mut t, P0, "Desolation Twin", &[]);
    t.settle();
    while *t.g.stack.last().unwrap() != twin {
        t.resolve();
    }
    assert_eq!(creature_tokens(&t, P0), 1);
    eldrazi_spell(&mut t);
    t.settle();
    t.resolve();
    // The Twin, the Infiltrator and their copies; nothing else.
    assert_eq!(t.stack_len(), 4, "{}", t.dump_log());
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Desolation Twin").len(), 2);
    assert_eq!(t.named_on_battlefield("Dimensional Infiltrator").len(), 2);
    // Only the 10/10 (besides the copies that became tokens).
    let other_tokens =
        t.g.permanents()
            .filter(|o| {
                o.is_token()
                    && o.chars.name != "Desolation Twin"
                    && o.chars.name != "Dimensional Infiltrator"
            })
            .count();
    assert_eq!(other_tokens, 1);
    // A copy of an activated ability isn't activated: P1's Harsh Mentor ("Whenever an
    // opponent activates an ability of an artifact, creature, or land on the battlefield,
    // if it isn't a mana ability, this creature deals 2 damage to that player.") sees P0
    // activate Prodigal Pyromancer's "{T}: This creature deals 1 damage to any target." at
    // P1 once; Ulalek copies the ability, and the copy deals its 1 without another trigger.
    supported("Harsh Mentor");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    t.battlefield(P1, "Harsh Mentor");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.g.objects[pyro.0 as usize].summoning_sick = false;
    t.answer_targets(P0, &[pl(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, pyro, "damage").unwrap();
    t.settle();
    eldrazi_spell(&mut t);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "the ability and its copy");
    assert_eq!(t.life(P0), 18, "one Harsh Mentor trigger");
}

// --- God-Eternal Kefnet ---------------------------------------------------------------------------

/// P0 controls Kefnet; `name` is on top of P0's library. P0 draws it as the first card
/// drawn this turn, answering whether to reveal it with `reveal`. (The turn is P1's, in
/// its upkeep, unless `own_turn`.)
pub fn kefnet_draw(name: &str, reveal: bool, own_turn: bool) -> (TestGame, ObjectId) {
    supported("God-Eternal Kefnet");
    let mut t = TestGame::new(2);
    if own_turn {
        t.set_step(P0, Step::PrecombatMain);
    } else {
        t.set_step(P1, Step::Upkeep);
    }
    t.battlefield(P0, "God-Eternal Kefnet");
    let card = t.library_top(P0, name);
    t.answer_yes(P0, reveal);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    (t, card)
}

#[test]
fn kefnet_reveals_and_copies_on_any_turn_and_casts_a_sorcery_copy_then() {
    cr!("707.12", "608.2g");
    ruling!(
        "God-Eternal Kefnet",
        "You can reveal and copy an instant or sorcery card this way on any turn, not just your own, if it's the first card you've drawn that turn."
    );
    ruling!(
        "God-Eternal Kefnet",
        "You can cast the copy during the resolution of the triggered ability if it's a sorcery, no matter whose turn it is or which phase it is."
    );
    // P1's upkeep: P0 draws Divination (a sorcery, {2}{U}: {U} with Kefnet's reduction).
    let (mut t, _) = kefnet_draw("Divination", true, false);
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true); // cast the copy
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).chars.name, "Divination");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A second card drawn that turn can't be revealed.
    let from = t.asked().len();
    t.library_top(P0, "Divination");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(t.g.stack.is_empty());
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
}

#[test]
fn kefnet_s_copy_can_be_cast_only_as_the_ability_resolves() {
    cr!("707.12", "608.2g");
    ruling!(
        "God-Eternal Kefnet",
        "You can cast the copy only as Kefnet's triggered ability resolves. If you don't want to cast it at that time (or you can't cast it, perhaps because there are no legal targets available), the copy ceases to exist. You can't cast it later."
    );
    // Declined: no copy is left anywhere (the hand holds just the card).
    let (mut t, card) = kefnet_draw("Lightning Bolt", true, true);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert_eq!(t.hand_size(P0), hand);
    let bolts: Vec<&mtg_engine::object::GameObject> =
        t.g.objects
            .iter()
            .filter(|o| {
                o.chars.name == "Lightning Bolt" && o.zone != Zone::Nowhere && t.g.is_live(o.id)
            })
            .collect();
    assert_eq!(bolts.len(), 1);
    assert_eq!(bolts[0].id, t.g.current(card));
    // Shatter ("Destroy target artifact.") with no artifact to target: it can't be cast,
    // and the copy is gone.
    let (mut t, card) = kefnet_draw("Shatter", true, true);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert_eq!(t.hand_size(P0), hand);
    assert!(t
        .g
        .objects
        .iter()
        .filter(|o| o.chars.name == "Shatter" && t.g.is_live(o.id) && o.zone != Zone::Nowhere)
        .all(|o| o.id == t.g.current(card)));
}

#[test]
fn kefnet_copies_a_card_that_left_the_hand_using_its_last_known_information() {
    cr!("707.12", "608.2h");
    interp!("copy-of-a-card-that-left-is-made-where-it-was");
    ruling!(
        "God-Eternal Kefnet",
        "If the card leaves your hand before Kefnet's triggered ability resolves, you'll copy it using its last known information."
    );
    // With the first ruling, the second says where that copy is made (decided by the two
    // rulings: the user, Nov 20).
    ruling!(
        "God-Eternal Kefnet",
        "The copy is created in and cast from your hand."
    );
    // Lightning Bolt revealed; in response it's discarded. The copy is still made and cast.
    let (mut t, card) = kefnet_draw("Lightning Bolt", true, true);
    assert_eq!(t.stack_len(), 1);
    let now = t.g.current(card);
    t.g.discard(P0, now, None);
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    // The copy was created where the card was, and cast from there.
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).kind, ObjKind::CardCopy);
    assert_eq!(
        t.obj(copy).stack.as_ref().unwrap().cast.from,
        Some(mtg_engine::ability::ZoneKind::Hand)
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
