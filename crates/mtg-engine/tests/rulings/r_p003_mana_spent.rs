//! Rulings batch P003 — "the amount of mana spent to cast": the mana actually paid for the
//! spell's total cost (CR 601.2f-h), including cost increases and additional costs, none
//! for a spell cast without paying its mana cost (CR 118.9) or mana paid later; Verazol's
//! copies (CR 707.10) and its counter removal; illegal targets (CR 608.2b); snow mana
//! (CR 106.3).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The size of the largest "choose cards" decision asked since `from`.
fn looked_at(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn memory_deluge_cast_without_paying_looks_at_nothing_and_mana_leak_mana_doesnt_count() {
    cr!("118.9", "601.2h", "107.3m");
    ruling!(
        "Memory Deluge",
        "If an effect allows you to cast Memory Deluge without paying its mana cost, X will be 0. Memory Deluge will resolve, but you won't do anything."
    );
    ruling!(
        "Memory Deluge",
        "If an effect requires you to pay some amount of mana to prevent Memory Deluge from being countered, as Flip the Switch does, that mana wasn't spent to cast Memory Deluge and will not be taken into account."
    );
    supported("Memory Deluge");
    supported("Omniscience");
    supported("Mana Leak");
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let deluge = t.hand(P0, "Memory Deluge");
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    let from = t.asked().len();
    t.cast(P0, deluge).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Memory Deluge"), "it resolved");
    assert_eq!(looked_at(&t, from), 0);
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.library_size(P0), library);
    // Cast for {2}{U}{U}; P1's Mana Leak, and P0 pays {3}: four cards, not seven.
    let mut t = TestGame::new(2);
    let deluge = cast(&mut t, P0, "Memory Deluge", "2UU", &[]);
    cast(&mut t, P1, "Mana Leak", "1U", &[obj(deluge)]);
    mana(&mut t, P0, "3");
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.g.stack.contains(&deluge), "paid for Mana Leak");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(looked_at(&t, from), 4);
    assert_eq!(t.hand_size(P0), hand + 2);
}

/// P0's Verazol with `n` +1/+1 counters.
fn verazol(t: &mut TestGame, n: u32) -> ObjectId {
    supported("Verazol, the Split Current");
    let v = t.battlefield(P0, "Verazol, the Split Current");
    put(t, v, counters::PLUS1, n);
    v
}

#[test]
fn verazol_counts_cost_increases_and_a_clone_counts_its_own_mana() {
    cr!("601.2f", "601.2h", "614.1c", "707.9a");
    ruling!(
        "Verazol, the Split Current",
        "The amount of mana you spent to cast Verazol is usually equal to its mana value, normally 2 plus the value chosen for X. However, you also include any additional costs you pay"
    );
    ruling!(
        "Verazol, the Split Current",
        "If another creature enters the battlefield as a copy of Verazol, consider how much mana was spent to cast that creature to determine how many +1/+1 counters it enters with."
    );
    supported("Sphere of Resistance");
    supported("Clone");
    // Sphere of Resistance: "Spells cost {1} more to cast." X = 1: {1}{G}{U} plus {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    let v = t.hand(P0, "Verazol, the Split Current");
    mana(&mut t, P0, "2GU");
    t.cast(P0, v).x(1).go();
    t.resolve_all();
    let v = t.named_on_battlefield("Verazol, the Split Current")[0];
    assert_eq!(t.counters(v, counters::PLUS1), 4);
    // P1's Clone cast for {3}{U} (and Sphere's {1}) copying it: five counters. One put
    // onto the battlefield without being cast: none (it's a 0/0 and dies).
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(v)]);
    cast_resolve(&mut t, P1, "Clone", "4U", &[]);
    let copies = t.named_on_battlefield("Verazol, the Split Current");
    assert_eq!(copies.len(), 2);
    let clone = copies.into_iter().find(|c| *c != v).unwrap();
    assert_eq!(t.obj_now(clone).controller, P1);
    assert_eq!(t.counters(clone, counters::PLUS1), 5);
    t.answer_choose(P1, &[obj(v)]);
    let put_in = t.enter(P1, "Clone");
    assert_eq!(t.counters(put_in, counters::PLUS1), 0);
    assert_eq!(t.obj_now(put_in).chars.name, "Verazol, the Split Current");
}

#[test]
fn verazol_copies_kicked_spells_with_or_without_targets_kicked_and_with_the_same_modes() {
    cr!("707.10", "707.10c", "702.33d", "700.2g");
    ruling!(
        "Verazol, the Split Current",
        "Verazol's last ability can copy any kicked spell, not just one with targets."
    );
    ruling!(
        "Verazol, the Split Current",
        "You can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too. Most notably, the copy is also kicked."
    );
    ruling!(
        "Verazol, the Split Current",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    supported("Kavu Titan");
    supported("Inscription of Abundance");
    // Kavu Titan: "Kicker {2}{G}. If this creature was kicked, it enters with three +1/+1
    // counters on it and with trample." No targets; the copy is kicked too.
    let mut t = TestGame::new(2);
    let v = verazol(&mut t, 2);
    let titan = t.hand(P0, "Kavu Titan");
    mana(&mut t, P0, "1G2G");
    t.cast(P0, titan).kicked(true).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    let titans = t.named_on_battlefield("Kavu Titan");
    assert_eq!(titans.len(), 2);
    for k in titans {
        assert_eq!(t.pt(k), (5, 5));
        assert!(t.obj_now(k).has_keyword(KeywordKind::Trample));
    }
    // Inscription of Abundance (kicker {2}{G}; "Choose one. If this spell was kicked,
    // choose any number instead"), kicked, choosing only "Put two +1/+1 counters on target
    // creature": the copy puts two counters too and gains no other mode.
    let mut t = TestGame::new(2);
    let v = verazol(&mut t, 2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let life = t.life(P0);
    let ins = t.hand(P0, "Inscription of Abundance");
    mana(&mut t, P0, "1G2G");
    t.cast(P0, ins).kicked(true).modes(&[0]).target(obj(bear)).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    assert_eq!(t.counters(bear, counters::PLUS1), 4);
    assert_eq!(t.life(P0), life);
}

#[test]
fn verazol_removes_exactly_two_counters_and_none_once_its_gone() {
    cr!("118.3", "118.12");
    ruling!(
        "Verazol, the Split Current",
        "You can't remove more than two counters as Verazol's last ability resolves to copy the spell more than once. You can't remove just one counter from it if it has only one. If Verazol leaves the battlefield while its last ability is on the stack, you can't remove two +1/+1 counters from it at all."
    );
    for (n, leaves, copies, left) in [(5, false, 1, 3), (1, false, 0, 1), (4, true, 0, 0)] {
        let mut t = TestGame::new(2);
        let v = verazol(&mut t, n);
        let titan = t.hand(P0, "Kavu Titan");
        mana(&mut t, P0, "1G2G");
        t.cast(P0, titan).kicked(true).go();
        t.settle();
        if leaves {
            kill(&mut t, v);
        }
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Kavu Titan").len(),
            1 + copies,
            "{n} counters, leaves {leaves}"
        );
        if !leaves {
            assert_eq!(t.counters(v, counters::PLUS1), left);
        }
    }
}

#[test]
fn liberator_counts_the_mana_actually_spent_and_grants_no_sorcery_speed_activation() {
    cr!("601.2h", "118.9", "702.8a", "307.1");
    ruling!(
        "Liberator, Urza's Battlethopter",
        "The last ability cares only about the amount of mana that was actually spent to cast the spell, regardless of its mana cost."
    );
    ruling!(
        "Liberator, Urza's Battlethopter",
        "The third ability applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated \"only as a sorcery.\""
    );
    supported("Liberator, Urza's Battlethopter");
    // "Whenever you cast a spell, if the amount of mana spent to cast that spell is greater
    // than Liberator's power, put a +1/+1 counter on Liberator." It's a 1/2.
    let mut t = TestGame::new(2);
    let lib = t.battlefield(P0, "Liberator, Urza's Battlethopter");
    t.battlefield(P0, "Omniscience");
    // Hill Giant (mana value 4) cast without paying: no counter.
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.counters(lib, counters::PLUS1), 0);
    // Shock ({R}) paid normally: 1 is not greater than 1.
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert_eq!(t.counters(lib, counters::PLUS1), 0);
    // Fireball with X = 2: three mana spent.
    let fb = t.hand(P0, "Fireball");
    mana(&mut t, P0, "2R");
    t.cast(P0, fb).x(2).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.counters(lib, counters::PLUS1), 1);
    // On P1's turn: an artifact spell has flash, an equip ability doesn't.
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    let splitter = t.hand(P0, "Bonesplitter");
    mana(&mut t, P0, "1");
    t.cast(P0, splitter).go();
    t.resolve_all();
    let splitter = t.g.current(splitter);
    assert!(t.on_battlefield(splitter));
    assert!(!can_activate(&mut t, P0, splitter, "Equip", "1"));
}

#[test]
fn molten_note_with_an_illegal_target_doesnt_untap() {
    cr!("608.2b");
    ruling!(
        "Molten Note",
        "If the target creature is an illegal target as Molten Note tries to resolve, it won't resolve and none of its effects will happen. You won't untap any creatures."
    );
    supported("Molten Note");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        t.g.tap(mine);
        let giant = t.battlefield(P1, "Hill Giant");
        let note = t.hand(P0, "Molten Note");
        mana(&mut t, P0, "1RW");
        t.cast(P0, note).x(1).target(obj(giant)).go();
        if gone {
            kill(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.obj_now(mine).tapped, gone, "target gone: {gone}");
        // Three mana spent: 3 damage to the 3/3.
        assert!(!t.on_battlefield(giant));
    }
}

#[test]
fn search_for_glory_without_snow_mana_gains_no_life_but_searches() {
    cr!("106.3", "601.2h");
    ruling!(
        "Search for Glory",
        "If you cast Search for Glory without spending snow mana, you won’t gain any life. You’ll still perform the search."
    );
    supported("Search for Glory");
    for snow in [false, true] {
        let mut t = TestGame::new(2);
        let target = t.library_top(P0, "Thalia, Guardian of Thraben");
        let plains = if snow {
            "Snow-Covered Plains"
        } else {
            "Plains"
        };
        t.lands(P0, plains, 3);
        let s = t.hand(P0, "Search for Glory");
        t.answer_choose(P0, &[obj(target)]);
        t.cast(P0, s).go();
        t.resolve_all();
        assert!(t.in_hand(P0, "Thalia, Guardian of Thraben"), "snow: {snow}");
        assert_eq!(t.life(P0), if snow { 23 } else { 20 });
    }
}

/// The Fractal tokens P0 controls now.
fn fractals(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Fractal"))
        .map(|o| o.id)
        .collect()
}

/// The counters of `kind` put on the object this turn.
fn put_this_turn(t: &TestGame, id: ObjectId, kind: &str) -> u32 {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::CountersAdded {
                target, kind: k, n, ..
            } if *target == obj(id) && k == kind => Some(*n),
            _ => None,
        })
        .sum()
}

#[test]
fn ambitious_augmenter_killed_by_minus_counters_moves_both_kinds() {
    cr!("704.3", "704.5f", "704.5q", "603.10a");
    ruling!(
        "Ambitious Augmenter",
        "If enough -1/-1 counters are put on Ambitious Augmenter at the same time to make its toughness 0 or less, its second ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had"
    );
    supported("Ambitious Augmenter");
    let mut t = TestGame::new(2);
    let aug = t.battlefield(P0, "Ambitious Augmenter");
    put(&mut t, aug, counters::PLUS1, 2);
    put(&mut t, aug, counters::MINUS1, 4);
    assert!(!t.on_battlefield(aug));
    t.resolve_all();
    // The Fractal got two +1/+1 and four -1/-1 counters (not none: the annihilation of
    // +1/+1 and -1/-1 counters happened at the same time as the death), and then died as
    // a -2/-2.
    let token = t
        .g
        .turn_events
        .iter()
        .find_map(|e| match e {
            mtg_engine::events::Event::TokenCreated { obj, .. } => Some(*obj),
            _ => None,
        })
        .expect("a Fractal was created");
    assert_eq!(put_this_turn(&t, token, counters::PLUS1), 2);
    assert_eq!(put_this_turn(&t, token, counters::MINUS1), 4);
}

#[test]
fn ambitious_augmenter_counters_go_on_each_token_and_on_the_ozolith() {
    cr!("111.1", "614.1a", "603.10a");
    ruling!(
        "Ambitious Augmenter",
        "If more than one token is created by Ambitious Augmenter’s last ability (because of the effect of a card like Anointed Procession, for example), put Ambitious Augmenter’s counters on each of those tokens."
    );
    ruling!(
        "Ambitious Augmenter",
        "In some unusual cases, you may end up putting the appropriate counters on more than one permanent. For example, if you control The Ozolith (a card from the Ikoria: Lair of Behemoths set) when Ambitious Augmenter dies, you'll put the appropriate number of each kind of counter onto both The Ozolith and the Fractal token."
    );
    supported("The Ozolith");
    for card in ["Anointed Procession", "The Ozolith"] {
        let mut t = TestGame::new(2);
        let other = t.battlefield(P0, card);
        let aug = t.battlefield(P0, "Ambitious Augmenter");
        put(&mut t, aug, counters::PLUS1, 2);
        put(&mut t, aug, "flying", 1);
        kill(&mut t, aug);
        t.resolve_all();
        let toks = fractals(&t);
        let want = if card == "The Ozolith" { 1 } else { 2 };
        assert_eq!(toks.len(), want, "{card}");
        for tok in toks {
            assert_eq!(t.pt(tok), (2, 2), "{card}");
            assert_eq!(t.counters(tok, "flying"), 1, "{card}");
        }
        if card == "The Ozolith" {
            assert_eq!(t.counters(other, counters::PLUS1), 2);
            assert_eq!(t.counters(other, "flying"), 1);
        }
    }
}
