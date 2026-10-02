//! Rulings batch P108 — -1/-1 counters as a cost or a drawback, and moving or copying
//! counters (CR 122): "put its counters on target creature" puts the same number of each
//! kind the dead creature had (last known information, CR 603.10a) rather than moving
//! them; "move a counter" removes it from one permanent and puts it on another
//! (CR 122.5); +1/+1 and -1/-1 counters annihilate (CR 704.5q).

use crate::r_p108_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::*;
use mtg_engine::*;

const PLUS1: &str = "+1/+1";
const MINUS1: &str = "-1/-1";

// --- -1/-1 counters as a cost ------------------------------------------------------------

#[test]
fn scarscale_ritual_cant_be_cast_without_a_creature() {
    cr!("118.3", "601.2h");
    ruling!(
        "Scarscale Ritual",
        "If you don’t control any creatures, you can’t cast Scarscale Ritual."
    );
    supported("Scarscale Ritual");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Scarscale Ritual");
    let card = t.hand(P0, "Scarscale Ritual");
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    // With a creature, it's cast and the counter goes on it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    let hand = t.hand_size(P0);
    t.cast(P0, card).go();
    assert_eq!(t.counters(bears, MINUS1), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn lethal_sting_needs_a_creature_to_put_the_counter_on() {
    cr!("118.3", "601.2h");
    supported("Lethal Sting");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Lethal Sting");
    let card = t.hand(P0, "Lethal Sting");
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    // The creature is chosen as the cost is paid.
    t.answer_choose(P0, &[obj(b)]);
    t.cast(P0, card).target(target).go();
    assert_eq!(t.counters(b, MINUS1), 1);
    assert_eq!(t.counters(a, MINUS1), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(target));
}

// --- Exemplar of Strength, Soulstinger -----------------------------------------------------

#[test]
fn exemplar_of_strength_gone_gains_no_life() {
    cr!("608.2c", "400.7");
    ruling!(
        "Exemplar of Strength",
        "If Exemplar of Strength leaves the battlefield in response to its second triggered ability, you won’t be able to remove a counter from it, and you won’t gain 1 life."
    );
    supported("Exemplar of Strength");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let ex = t.battlefield(P0, "Exemplar of Strength");
        put_counters(&mut t, ex, MINUS1, 2);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(ex, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::DeclareAttackers);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if leave {
            destroy(&mut t, ex);
        }
        t.resolve_all();
        if leave {
            assert_eq!(t.life(P0), 20);
        } else {
            assert_eq!(t.counters(ex, MINUS1), 1);
            assert_eq!(t.life(P0), 21);
        }
    }
}

#[test]
fn soulstinger_puts_all_its_counters_on_one_target() {
    cr!("115.1", "603.10a");
    ruling!(
        "Soulstinger",
        "second ability targets one creature to get all the counters. You can’t distribute the counters among multiple creatures."
    );
    supported("Soulstinger");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Soulstinger");
    put_counters(&mut t, s, MINUS1, 3);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(a)]);
    destroy(&mut t, s);
    t.resolve_all();
    assert_eq!(t.counters(a, MINUS1), 3);
    assert_eq!(t.counters(b, MINUS1), 0);
    // A single target was asked for.
    let slots: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(slots, vec![1]);
}

// --- Flourishing Defenses --------------------------------------------------------------------

/// The number of Elf Warrior tokens P0 gets when `act` runs with Flourishing Defenses
/// (always choosing to create one).
fn defenses(act: impl FnOnce(&mut TestGame)) -> usize {
    supported("Flourishing Defenses");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flourishing Defenses");
    for _ in 0..6 {
        t.answer_yes(P0, true);
    }
    act(&mut t);
    t.resolve_all();
    tokens_with(&t, P0, "Elf")
}

#[test]
fn flourishing_defenses_triggers_for_a_creature_entering_with_counters_and_persist() {
    cr!("122.6", "702.79a");
    ruling!(
        "Flourishing Defenses",
        "This ability triggers both when a -1/-1 counter is put on a creature on the battlefield and when a creature enters with a -1/-1 counter on it. This includes when a creature returns to the battlefield as a result of persist."
    );
    // On the battlefield.
    assert_eq!(
        defenses(|t| {
            let b = t.battlefield(P1, "Hill Giant");
            put_counters(t, b, MINUS1, 1);
        }),
        1
    );
    // Returning with persist.
    assert_eq!(
        defenses(|t| {
            let f = t.battlefield(P1, "Kitchen Finks");
            destroy(t, f);
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Kitchen Finks").len(), 1);
        }),
        1
    );
}

#[test]
fn flourishing_defenses_triggers_for_each_counter() {
    cr!("122.6", "603.2c");
    ruling!(
        "Flourishing Defenses",
        "This ability triggers once for each -1/-1 counter. For example, if Leech Bonder (a creature that enters with two -1/-1 counters on it) entered, Flourishing Defenses's ability would trigger twice."
    );
    supported("Leech Bonder");
    assert_eq!(
        defenses(|t| {
            t.enter(P0, "Leech Bonder");
        }),
        2
    );
}

// --- Fate Transfer, Leech Bonder, Nesting Grounds ---------------------------------------------

#[test]
fn fate_transfer_moves_every_kind_of_counter() {
    cr!("122.5", "122.1b");
    ruling!(
        "Fate Transfer",
        "Fate Transfer moves any kind of counters, not just -1/-1 counters."
    );
    ruling!(
        "Fate Transfer",
        "This effect may result in useless counters being placed on a creature."
    );
    supported("Fate Transfer");
    let mut t = TestGame::new(2);
    let from = t.battlefield(P0, "Colossal Dreadmaw");
    put_counters(&mut t, from, PLUS1, 2);
    put_counters(&mut t, from, "flying", 1);
    put_counters(&mut t, from, "age", 3);
    let to = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Fate Transfer", &[obj(from), obj(to)]);
    t.resolve_all();
    assert_eq!(t.counters(to, PLUS1), 2);
    assert_eq!(t.counters(to, "flying"), 1);
    assert_eq!(t.counters(to, "age"), 3);
    assert!(t.obj_now(to).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(to), (5, 5));
    assert_eq!(t.counters(from, PLUS1) + t.counters(from, "age"), 0);
}

#[test]
fn fate_transfer_moves_nothing_if_a_target_is_illegal() {
    cr!("608.2b", "122.5");
    ruling!(
        "Fate Transfer",
        "If either one of the targets becomes illegal (because it leaves the battlefield or for any other reason) by the time Fate Transfer resolves, the counters don’t move. If both targets become illegal, Fate Transfer doesn’t resolve."
    );
    for gone in [0, 1] {
        let mut t = TestGame::new(2);
        let from = t.battlefield(P0, "Colossal Dreadmaw");
        put_counters(&mut t, from, PLUS1, 2);
        let to = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, "Fate Transfer", &[obj(from), obj(to)]);
        destroy(&mut t, if gone == 0 { from } else { to });
        t.resolve_all();
        if gone == 0 {
            assert_eq!(t.counters(to, PLUS1), 0);
        } else {
            assert_eq!(t.counters(from, PLUS1), 2);
        }
    }
    let mut t = TestGame::new(2);
    let from = t.battlefield(P0, "Colossal Dreadmaw");
    let to = t.battlefield(P1, "Hill Giant");
    let spell = cast_new(&mut t, P0, "Fate Transfer", &[obj(from), obj(to)]);
    destroy(&mut t, from);
    destroy(&mut t, to);
    t.resolve_all();
    assert!(!resolved(&t, spell));
}

#[test]
fn leech_bonder_can_move_a_useless_counter() {
    cr!("122.5", "122.1");
    ruling!(
        "Leech Bonder",
        "This effect may result in a useless counter being placed on a creature. For example, if an age counter is moved from a creature with cumulative upkeep to a creature without cumulative upkeep, it will have no effect on the new creature."
    );
    supported("Leech Bonder");
    let mut t = TestGame::new(2);
    let bonder = t.battlefield(P0, "Leech Bonder");
    t.g.tap(bonder);
    let from = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, from, "age", 1);
    let to = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.activate(P0, bonder, 0, &[obj(from), obj(to)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(to, "age"), 1);
    assert_eq!(t.counters(from, "age"), 0);
    assert_eq!(t.pt(to), (2, 2));
}

/// P0's Nesting Grounds, ready to activate (with lands for its {1}).
fn nesting_grounds(t: &mut TestGame) -> ObjectId {
    supported("Nesting Grounds");
    let g = t.battlefield(P0, "Nesting Grounds");
    t.lands(P0, "Wastes", 1);
    g
}

#[test]
fn nesting_grounds_moves_nothing_if_a_target_is_illegal() {
    cr!("608.2b", "122.5");
    ruling!(
        "Nesting Grounds",
        "If either permanent becomes an illegal target, no counter is removed or put."
    );
    for gone_from in [true, false] {
        let mut t = TestGame::new(2);
        let g = nesting_grounds(&mut t);
        let from = t.battlefield(P0, "Hill Giant");
        put_counters(&mut t, from, PLUS1, 1);
        let to = t.battlefield(P0, "Grizzly Bears");
        t.activate(P0, g, 1, &[obj(from), obj(to)]).expect("activate");
        destroy(&mut t, if gone_from { from } else { to });
        t.resolve_all();
        if gone_from {
            assert_eq!(t.counters(to, PLUS1), 0);
        } else {
            assert_eq!(t.counters(from, PLUS1), 1);
        }
    }
}

#[test]
fn nesting_grounds_targets_need_not_share_a_type() {
    cr!("122.1", "122.1b", "122.5");
    ruling!(
        "Nesting Grounds",
        "The two target permanents don't have to share a type, which can result in some counters on permanents that would not occur normally"
    );
    let mut t = TestGame::new(2);
    let g = nesting_grounds(&mut t);
    let from = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, from, "trample", 1);
    let ench = t.battlefield(P0, "Flourishing Defenses");
    t.activate(P0, g, 1, &[obj(from), obj(ench)]).expect("activate");
    t.resolve_all();
    // A keyword counter grants its keyword even to an enchantment.
    assert_eq!(t.counters(ench, "trample"), 1);
    assert!(t.obj_now(ench).has_keyword(KeywordKind::Trample));
    // +1/+1 counters on a land don't make it a creature.
    let mut t = TestGame::new(2);
    let g = nesting_grounds(&mut t);
    let from = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, from, PLUS1, 1);
    let land = t.battlefield(P0, "Forest");
    t.activate(P0, g, 1, &[obj(from), obj(land)]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(land, PLUS1), 1);
    assert!(!t.obj_now(land).is(CardType::Creature));
    assert_eq!(t.pt(from), (3, 3));
}

#[test]
fn nesting_grounds_removes_and_puts_the_counter() {
    cr!("122.5", "614.1a");
    ruling!(
        "Nesting Grounds",
        "To move a counter from one creature to another, the counter is removed from the first permanent and put on the second. Any abilities that care about a counter being removed from or put onto a permanent will apply."
    );
    supported("Hardened Scales");
    let mut t = TestGame::new(2);
    let g = nesting_grounds(&mut t);
    let from = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, from, PLUS1, 1);
    t.battlefield(P0, "Hardened Scales");
    let to = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, g, 1, &[obj(from), obj(to)]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(from, PLUS1), 0);
    assert_eq!(t.counters(to, PLUS1), 2, "Hardened Scales adds one");
}

#[test]
fn nesting_grounds_chooses_the_kind_as_it_resolves() {
    cr!("601.2c", "608.2c");
    ruling!(
        "Nesting Grounds",
        "You choose the two target permanents as Nesting Grounds's second ability is put onto the stack. You choose which kind of counter to move as that ability resolves."
    );
    let mut t = TestGame::new(2);
    let g = nesting_grounds(&mut t);
    let from = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, from, PLUS1, 1);
    let to = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, g, 1, &[obj(from), obj(to)]).expect("activate");
    // A flying counter is put on it in response; it can be the one moved.
    put_counters(&mut t, from, "flying", 1);
    let from_asked = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let offered: Vec<Vec<String>> = t.asked()[from_asked..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1, "the kind is chosen as it resolves");
    assert!(offered[0].iter().any(|o| o.contains("flying")));
    let moved_flying = t.counters(to, "flying") == 1;
    assert_eq!(t.counters(to, "flying") + t.counters(to, PLUS1), 1);
    assert_eq!(
        t.counters(from, "flying") + t.counters(from, PLUS1),
        1,
        "{moved_flying}"
    );
}

// --- "Put its counters on target creature" ----------------------------------------------------

#[test]
fn star_pupil_puts_all_its_counters_including_minus_ones() {
    cr!("603.10a", "122.8");
    ruling!(
        "Star Pupil",
        "Star Pupil puts all of its counters onto the target creature, not just its +1/+1 counters."
    );
    ruling!(
        "Star Pupil",
        "Star Pupil's ability doesn't cause you to move counters from itself to the target creature. Rather, you put the same number of each kind of counter Star Pupil had when it died onto that creature."
    );
    ruling!(
        "Star Pupil",
        "If Star Pupil has -1/-1 counters on it when it dies, the last ability will include those as well. This may result in the recipient also dying."
    );
    supported("Star Pupil");
    // A flying counter and a +1/+1 counter.
    let mut t = TestGame::new(2);
    let sp = t.enter(P0, "Star Pupil");
    put_counters(&mut t, sp, "flying", 1);
    assert_eq!(t.counters(sp, PLUS1), 1);
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    destroy(&mut t, sp);
    t.resolve_all();
    assert_eq!(t.counters(to, PLUS1), 1);
    assert_eq!(t.counters(to, "flying"), 1);
    // -1/-1 counters: it's a 0/0 with two -1/-1 counters (after annihilation).
    let mut t = TestGame::new(2);
    let sp = t.enter(P0, "Star Pupil");
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    put_counters(&mut t, sp, MINUS1, 3);
    t.resolve_all();
    assert!(!t.on_battlefield(sp));
    assert!(!t.on_battlefield(to), "two -1/-1 counters kill the 2/2");
}

#[test]
fn spiteful_squad_puts_its_counters_including_minus_ones() {
    cr!("603.10a", "122.8", "704.5q");
    ruling!(
        "Spiteful Squad",
        "Spiteful Squad's ability doesn't cause you to move counters from itself to the target creature. Rather, you put the same number of each kind of counter Spiteful Squad had when it died onto that creature."
    );
    ruling!(
        "Spiteful Squad",
        "If Spiteful Squad has -1/-1 counters on it when it dies, the last ability will include those as well. This may result in the recipient also dying. Spiteful indeed."
    );
    supported("Spiteful Squad");
    let mut t = TestGame::new(2);
    let sq = t.enter(P0, "Spiteful Squad");
    assert_eq!(t.counters(sq, PLUS1), 2);
    let to = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(to)]);
    destroy(&mut t, sq);
    t.resolve_all();
    assert_eq!(t.counters(to, PLUS1), 2);
    // Four -1/-1 counters annihilate with its two +1/+1 counters: it dies with two -1/-1
    // counters, which kill a 2/2.
    let mut t = TestGame::new(2);
    let sq = t.enter(P0, "Spiteful Squad");
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    put_counters(&mut t, sq, MINUS1, 4);
    t.resolve_all();
    assert!(!t.on_battlefield(sq));
    assert!(!t.on_battlefield(to));
}

/// The Ozolith and a creature that puts its counters on a target as it dies: both get
/// them.
fn with_ozolith(name: &str) {
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let oz = t.battlefield(P0, "The Ozolith");
    let c = t.enter(P0, name);
    put_counters(&mut t, c, "flying", 1);
    let to = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(to)]);
    destroy(&mut t, c);
    t.resolve_all();
    for id in [oz, to] {
        assert_eq!(t.counters(id, "flying"), 1, "{name}");
        assert!(t.counters(id, PLUS1) >= 1, "{name}");
    }
}

#[test]
fn spiteful_squad_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.8");
    ruling!(
        "Spiteful Squad",
        "if you control The Ozolith when Spiteful Squad dies, you will put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    with_ozolith("Spiteful Squad");
}

#[test]
fn star_pupil_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.8");
    ruling!(
        "Star Pupil",
        "if you control The Ozolith when Star Pupil dies, you will put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    with_ozolith("Star Pupil");
}

#[test]
fn host_of_the_hereafter_puts_the_dead_creatures_counters() {
    cr!("603.10a", "122.8");
    ruling!(
        "Host of the Hereafter",
        "Host of the Hereafter’s last ability doesn’t cause you to move counters from the creature that died onto the target creature. Rather, you put the same number of each kind of counter the creature had when it died onto the target creature."
    );
    ruling!(
        "Host of the Hereafter",
        "Host of the Hereafter’s last ability puts all counters that were on the creature that died onto the target creature, not just its +1/+1 counters."
    );
    supported("Host of the Hereafter");
    let mut t = TestGame::new(2);
    t.enter(P0, "Host of the Hereafter");
    let dying = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, dying, PLUS1, 2);
    put_counters(&mut t, dying, "flying", 1);
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    destroy(&mut t, dying);
    t.resolve_all();
    assert_eq!(t.counters(to, PLUS1), 2);
    assert_eq!(t.counters(to, "flying"), 1);
}

#[test]
fn host_of_the_hereafter_dying_with_others_triggers_for_each() {
    cr!("603.10a", "603.3b");
    ruling!(
        "Host of the Hereafter",
        "If Host of the Hereafter dies at the same time as one or more other creatures you control, Host of the Hereafter’s last ability triggers for each of those creatures that had counters on them, including itself if applicable."
    );
    let mut t = TestGame::new(2);
    let host = t.enter(P0, "Host of the Hereafter");
    let a = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, a, PLUS1, 1);
    // Without counters: no trigger for it.
    t.battlefield(P0, "Grizzly Bears");
    let to = t.battlefield(P0, "Colossal Dreadmaw");
    // An indestructible recipient survives the wrath.
    put_counters(&mut t, to, "indestructible", 1);
    t.answer_targets(P0, &[obj(to)]);
    t.answer_targets(P0, &[obj(to)]);
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve();
    assert!(!t.on_battlefield(host));
    assert_eq!(crate::r_s25_common::abilities_from(&t, host).len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(to, PLUS1), 3);
}

#[test]
fn host_of_the_hereafter_sees_counters_of_a_creature_killed_by_minus_ones() {
    cr!("603.10a", "122.8", "704.5f", "704.5q");
    ruling!(
        "Host of the Hereafter",
        "If enough -1/-1 counters are put on a creature you control at the same time to make its toughness 0 or less, Host of the Hereafter’s last ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had"
    );
    ruling!(
        "Host of the Hereafter",
        "If the creature that died had -1/-1 counters on it when it died, Host of the Hereafter’s ability will put those on the target creature as well. This may result in the recipient of the counters also dying."
    );
    // A 2/2 with a +1/+1 counter gets four -1/-1 counters: it dies with both kinds.
    let mut t = TestGame::new(2);
    t.enter(P0, "Host of the Hereafter");
    let dying = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(dying), PLUS1, 1, None);
    let to = t.battlefield(P0, "Colossal Dreadmaw");
    t.answer_targets(P0, &[obj(to)]);
    put_counters(&mut t, dying, MINUS1, 4);
    assert!(!t.on_battlefield(dying));
    t.resolve_all();
    // One +1/+1 and four -1/-1 counters: three -1/-1 counters remain on the 6/6.
    assert_eq!(t.counters(to, MINUS1), 3);
    assert_eq!(t.pt(to), (3, 3));
    // The recipient may die too.
    let mut t = TestGame::new(2);
    t.enter(P0, "Host of the Hereafter");
    let dying = t.battlefield(P0, "Grizzly Bears");
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    put_counters(&mut t, dying, MINUS1, 2);
    t.resolve_all();
    assert!(!t.on_battlefield(to));
}

#[test]
fn host_of_the_hereafter_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.8");
    ruling!(
        "Host of the Hereafter",
        "if you control The Ozolith and Host of the Hereafter when a creature you control with counters on it dies, you’ll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    let mut t = TestGame::new(2);
    let oz = t.battlefield(P0, "The Ozolith");
    t.enter(P0, "Host of the Hereafter");
    let dying = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, dying, PLUS1, 2);
    let to = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(to)]);
    destroy(&mut t, dying);
    t.resolve_all();
    assert_eq!(t.counters(oz, PLUS1), 2);
    assert_eq!(t.counters(to, PLUS1), 2);
}

#[test]
fn a_counter_is_put_on_triggers_for_each_counter() {
    cr!("122.6", "603.2c");
    ruling!(
        "Fathom Mage",
        "If multiple +1/+1 counters are placed on Fathom Mage simultaneously, its last ability will trigger once for each of those counters."
    );
    supported("Fathom Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Fathom Mage");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    put_counters(&mut t, mage, PLUS1, 3);
    assert_eq!(crate::r_s25_common::abilities_from(&t, mage).len(), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

