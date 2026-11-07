//! Rulings batch P113 — abilities that trigger on a spell or ability being cast or
//! targeting something ("becomes the target of a spell or ability an opponent controls",
//! "whenever you cast ...", "when you spend this mana ..."): they resolve first, even if
//! the spell is countered (CR 603.3, 608.2), and players get priority in between.

use crate::r_p113_common::*;
use crate::r_s01_common::supported;
use crate::r_s07_common::{damage_on, resolved};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P1 casts Shock at P0's `target`, P0's trigger goes on the stack (P0 answers its "may"
/// with `draw`), and P1 counters the Shock with Counterspell on top of the trigger.
/// Returns (game, shock, P0's hand size before).
fn shock_then_counter(t: &mut TestGame, target: ObjectId, draw: bool) -> (ObjectId, usize) {
    let hand = t.hand_size(P0);
    let shock = cast_new(t, P1, "Shock", &[ent(target)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is above the Shock");
    t.answer_yes(P0, draw);
    cast_new(t, P1, "Counterspell", &[ent(shock)]);
    t.resolve_all();
    (shock, hand)
}

#[test]
fn becomes_the_target_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.2", "603.3b", "608.2");
    ruling!(
        "Warden of the Woods",
        "An ability that triggers when something becomes the target of a spell or ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    ruling!(
        "Battle Mammoth",
        "Battle Mammoth's triggered ability resolves before the spell or ability that caused it to trigger. It will resolve even if that spell or ability is countered."
    );
    // (card, cards drawn)
    for (card, n) in [("Warden of the Woods", 2), ("Battle Mammoth", 1)] {
        supported(card);
        let mut t = TestGame::new(2);
        let it = t.battlefield(P0, card);
        let (shock, hand) = shock_then_counter(&mut t, it, true);
        assert!(!resolved(&t, shock), "{card}");
        assert_eq!(t.hand_size(P0), hand + n, "{card}");
        assert_eq!(damage_on(&t, it), 0, "{card}");
    }
}

#[test]
fn becomes_the_target_triggers_resolve_first() {
    cr!("603.3b", "117.3b", "405.5");
    ruling!(
        "Shapers' Sanctuary",
        "The triggered ability of Shapers’ Sanctuary resolves before the spell or ability that caused it to trigger."
    );
    ruling!(
        "Shapers' Sanctuary",
        "Players can cast spells and activate abilities after the triggered ability of Shapers’ Sanctuary resolves but before the spell or ability that caused it to trigger does. Notably, the card you draw may be able to counter that spell or ability."
    );
    supported("Shapers' Sanctuary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shapers' Sanctuary");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Counterspell");
    t.lands(P0, "Island", 2);
    let shock = cast_new(&mut t, P1, "Shock", &[ent(bears)]);
    t.settle();
    t.answer_yes(P0, true);
    // The trigger resolves alone: the Shock is still on the stack, and P0 has drawn the
    // Counterspell.
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_hand(P0, "Counterspell"));
    // P0 casts it at the Shock.
    let cs = t.g.players[0]
        .hand
        .iter()
        .copied()
        .find(|c| t.g.obj(*c).chars.name == "Counterspell")
        .unwrap();
    t.cast_with(P0, cs, &[ent(shock)]).expect("Counterspell");
    t.resolve_all();
    assert!(!resolved(&t, shock));
    assert!(t.on_battlefield(bears));
}

#[test]
fn battle_mammoth_triggers_once_per_permanent_targeted() {
    cr!("603.2c", "115.3");
    ruling!(
        "Battle Mammoth",
        "If a spell or ability an opponent controls targets a single permanent you control more than once, Battle Mammoth's triggered ability will trigger only once. However, if a spell or ability an opponent controls targets multiple permanents you control, Battle Mammoth's triggered ability will trigger once for each of those permanents."
    );
    ruling!(
        "Battle Mammoth",
        "You choose whether to draw a card as Battle Mammoth's triggered ability is resolving."
    );
    supported("Battle Mammoth");
    supported("Seeds of Strength");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn."
    for (distinct, triggers) in [(1usize, 1usize), (2, 2), (3, 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Battle Mammoth");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let c = t.battlefield(P0, "Craw Wurm");
        let targets = match distinct {
            1 => [a, a, a],
            2 => [a, a, b],
            _ => [a, b, c],
        };
        let ts: Vec<Entity> = targets.iter().map(|x| ent(*x)).collect();
        cast_new(&mut t, P1, "Seeds of Strength", &ts);
        t.settle();
        assert_eq!(t.stack_len(), 1 + triggers, "{distinct} permanents");
        // No yes/no was asked yet: the choice is made as each trigger resolves.
        assert!(!t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        let hand = t.hand_size(P0);
        // Draw for the first trigger only.
        t.answer_yes(P0, true);
        for _ in 1..triggers {
            t.answer_yes(P0, false);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{distinct} permanents");
        let _ = c;
    }
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.2", "603.3b", "601.2i");
    ruling!(
        "Balmor, Battlemage Captain",
        "Balmor's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Alphinaud Leveilleur",
        "Alphinaud Leveilleur's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    // Balmor: creatures you control get +1/+0 and gain trample.
    supported("Balmor, Battlemage Captain");
    let mut t = TestGame::new(2);
    let balmor = t.battlefield(P0, "Balmor, Battlemage Captain");
    let opt = cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_new(&mut t, P1, "Counterspell", &[ent(opt)]);
    t.resolve_all();
    assert!(!resolved(&t, opt));
    assert_eq!(t.pt(balmor), (2, 3));
    // Alphinaud: the second spell each turn draws a card even if it's countered.
    supported("Alphinaud Leveilleur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alphinaud Leveilleur");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let hand = t.hand_size(P0);
    let second = cast_new(&mut t, P0, "Hill Giant", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_new(&mut t, P1, "Counterspell", &[ent(second)]);
    t.resolve_all();
    assert!(!resolved(&t, second));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn gilanra_spend_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("106.6", "603.7a", "603.3b");
    ruling!(
        "Gilanra, Caller of Wirewood",
        "An ability that triggers when a player spends mana to cast a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Gilanra, Caller of Wirewood");
    let mut t = TestGame::new(2);
    let gilanra = t.battlefield(P0, "Gilanra, Caller of Wirewood");
    t.lands(P0, "Forest", 5);
    t.activate(P0, gilanra, 0, &[]).unwrap();
    let card = t.hand(P0, "Colossal Dreadmaw");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is above the spell");
    cast_new(&mut t, P1, "Counterspell", &[ent(spell)]);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.hand_size(P0), hand, "drew one card for the one cast");
}
