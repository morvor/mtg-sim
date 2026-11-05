//! Rulings batch P002 — "the amount of mana spent to cast": copies (CR 707.10, no mana
//! was spent to cast a copy), spells cast without paying their mana cost but with cost
//! increases (CR 118.9, 601.2f), permanents put onto the battlefield without being cast;
//! triggered abilities of cast spells resolving first (CR 603.3, 608.2); Verazol's copies
//! (CR 707.10); Ambitious Augmenter's counters (CR 603.10a); snow permanent cards
//! (CR 205.4g).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Verazol with `n` +1/+1 counters.
fn verazol(t: &mut TestGame, n: u32) -> ObjectId {
    supported("Verazol, the Split Current");
    let v = t.battlefield(P0, "Verazol, the Split Current");
    put(t, v, counters::PLUS1, n);
    v
}

#[test]
fn verazol_copies_a_spell_countered_meanwhile_and_the_copy_resolves_first() {
    cr!("603.3", "707.10", "608.2");
    ruling!(
        "Verazol, the Split Current",
        "A copy is created even if the spell that caused Verazol's last ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell."
    );
    supported("Burst Lightning");
    // Kicked Burst Lightning (4 damage) at P1; P1 counters it with the trigger waiting.
    let mut t = TestGame::new(2);
    let v = verazol(&mut t, 2);
    let burst = t.hand(P0, "Burst Lightning");
    mana(&mut t, P0, "R4");
    let burst = t.cast(P0, burst).kicked(true).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_resolve_top(&mut t, P1, "Cancel", "1UU", burst);
    assert!(t.in_graveyard(P0, "Burst Lightning"));
    assert_eq!(t.stack_len(), 1, "the trigger");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    assert_eq!(t.life(P1), 16, "the copy dealt 4");
    // Not countered: the copy is above the original and resolves first.
    let mut t = TestGame::new(2);
    verazol(&mut t, 2);
    let burst = t.hand(P0, "Burst Lightning");
    mana(&mut t, P0, "R4");
    let burst = t.cast(P0, burst).kicked(true).target(pl(P1)).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(crate::r_s25_common::spell_copies(&t), vec![top], "the copy is on top");
    assert_eq!(t.g.stack[0], burst);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert!(t.g.stack.contains(&burst));
    t.resolve();
    assert_eq!(t.life(P1), 12);
}

/// `p` casts a new copy of the card `name` targeting the spell `spell`; it resolves.
fn cast_resolve_top(t: &mut TestGame, p: PlayerId, name: &str, cost: &str, spell: ObjectId) {
    cast(t, p, name, cost, &[obj(spell)]);
    t.resolve();
}

#[test]
fn verazol_s_copy_of_an_x_spell_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Verazol, the Split Current",
        "For spells with {X} in their mana costs, the copy has the same value of X."
    );
    supported("Illuminate");
    // Illuminate (X = 3) kicked with {2}{R}: 3 damage to the creature and its controller.
    let mut t = TestGame::new(2);
    verazol(&mut t, 2);
    let a = add(&mut t, P1, "Hill Giant");
    let b = add(&mut t, P1, "Craw Wurm");
    let ill = t.hand(P0, "Illuminate");
    mana(&mut t, P0, "3R2R");
    t.cast(P0, ill)
        .kicked(true)
        .kicked(false)
        .x(3)
        .target(obj(a))
        .go();
    t.answer_yes(P0, true);
    // The copy targets the Craw Wurm instead.
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(dmg(&t, b), 3);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn verazol_put_onto_the_battlefield_has_no_counters_and_dies() {
    cr!("704.5f", "601.2h");
    ruling!(
        "Verazol, the Split Current",
        "If Verazol enters the battlefield without being cast, then no mana was spent to cast it. It enters the battlefield without any +1/+1 counters."
    );
    supported("Verazol, the Split Current");
    let mut t = TestGame::new(2);
    let v = enter(&mut t, P0, "Verazol, the Split Current");
    assert!(!t.on_battlefield(v));
    assert!(t.in_graveyard(P0, "Verazol, the Split Current"));
    // Control: cast for {X}{G}{U} with X = 1, three counters.
    let v = t.hand(P0, "Verazol, the Split Current");
    mana(&mut t, P0, "1GU");
    t.cast(P0, v).x(1).go();
    t.resolve_all();
    let v = t.named_on_battlefield("Verazol, the Split Current");
    assert_eq!(t.counters(v[0], counters::PLUS1), 3);
}

#[test]
fn aberrant_manawurm_s_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "608.2", "601.2h");
    ruling!(
        "Aberrant Manawurm",
        "Aberrant Manawurm's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Aberrant Manawurm");
    // Divination ({2}{U}): +3/+0 while Divination is still on the stack.
    let mut t = TestGame::new(2);
    let wurm = add(&mut t, P0, "Aberrant Manawurm");
    let div = cast(&mut t, P0, "Divination", "2U", &[]);
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.pt(wurm), (5, 5));
    assert!(t.g.stack.contains(&div));
    assert_eq!(t.hand_size(P0), hand);
    // Countered with the trigger waiting: the trigger still resolves.
    let mut t = TestGame::new(2);
    let wurm = add(&mut t, P0, "Aberrant Manawurm");
    let div = cast(&mut t, P0, "Divination", "2U", &[]);
    cast_resolve_top(&mut t, P1, "Cancel", "1UU", div);
    assert!(t.in_graveyard(P0, "Divination"));
    t.resolve_all();
    assert_eq!(t.pt(wurm), (5, 5));
}

/// P0's Ambitious Augmenter with the given counters dies; returns the Fractal token
/// created (followed even if it died).
fn augmenter_dies(t: &mut TestGame, cs: &[(&str, u32)]) -> ObjectId {
    supported("Ambitious Augmenter");
    let aug = add(t, P0, "Ambitious Augmenter");
    for (k, n) in cs {
        let id = t.g.current(aug);
        t.g.add_counters(obj(id), k, *n, None);
    }
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    if t.on_battlefield(aug) {
        kill(t, aug);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ambitious Augmenter"));
    t.g.turn_events
        .iter()
        .find_map(|e| match e {
            mtg_engine::events::Event::TokenCreated { obj, .. } => Some(*obj),
            _ => None,
        })
        .expect("a Fractal was created")
}

#[test]
fn ambitious_augmenter_puts_each_kind_of_counter_it_had_on_the_fractal() {
    cr!("603.10a", "122.6");
    ruling!(
        "Ambitious Augmenter",
        "Ambitious Augmenter's last ability doesn't cause you to move counters from Ambitious Augmenter onto the Fractal token. Rather, you put the same number of each kind of counter"
    );
    ruling!(
        "Ambitious Augmenter",
        "Ambitious Augmenter's last ability puts all counters that were on it onto the Fractal token, not just its +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let tok = augmenter_dies(&mut t, &[(counters::PLUS1, 2), ("charge", 3)]);
    assert!(t.on_battlefield(tok));
    assert_eq!(t.counters(tok, counters::PLUS1), 2);
    assert_eq!(t.counters(tok, "charge"), 3);
    assert_eq!(t.pt(tok), (2, 2));
}

#[test]
fn ambitious_augmenter_s_minus_counters_can_kill_the_fractal() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Ambitious Augmenter",
        "If Ambitious Augmenter has -1/-1 counters on it when it dies, that ability will include those as well. This may result in the Fractal token dying."
    );
    // One -1/-1 counter: the 1/1 Augmenter dies; the Fractal is a -1/-1 and dies.
    let mut t = TestGame::new(2);
    let tok = augmenter_dies(&mut t, &[(counters::MINUS1, 1)]);
    assert!(!t.on_battlefield(tok));
    let put: u32 =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::CountersAdded { target, kind, n, .. }
                    if *target == obj(tok) && kind == counters::MINUS1 =>
                {
                    Some(*n)
                }
                _ => None,
            })
            .sum();
    assert_eq!(put, 1);
}

#[test]
fn freestrider_commando_cast_free_but_paying_a_cost_increase_gets_no_counters() {
    cr!("601.2f", "118.9", "702.170d");
    ruling!(
        "Freestrider Commando",
        "If Freestrider Commando is cast without paying its mana cost but you paid mana for additional costs or cost increases (such as from Aven Interrupter), Freestrider Commando won’t enter with +1/+1 counters."
    );
    supported("Freestrider Commando");
    supported("Aven Interrupter");
    // Plotted, then cast from exile on a later turn: Aven Interrupter ("Spells your
    // opponents cast from graveyards or from exile cost {2} more to cast") or not.
    for interrupter in [false, true] {
        let mut t = TestGame::new(2);
        if interrupter {
            add(&mut t, P1, "Aven Interrupter");
        }
        let c = t.hand(P0, "Freestrider Commando");
        mana(&mut t, P0, "3G");
        t.g.turn.priority = Some(P0);
        t.g.perform_action(P0, Action::Special(SpecialAction::Plot { card: c }))
            .expect("plot");
        let c = t.g.find_in_zone(Zone::Exile, "Freestrider Commando")[0];
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        if interrupter {
            mana(&mut t, P0, "2");
        }
        t.cast(P0, c)
            .method(CastMethod::Keyword(KeywordKind::Plot))
            .go();
        assert_eq!(t.g.players[0].mana_pool.total(), 0);
        t.resolve_all();
        let cmd = t.named_on_battlefield("Freestrider Commando");
        assert_eq!(cmd.len(), 1);
        let want = if interrupter { 0 } else { 2 };
        assert_eq!(t.counters(cmd[0], counters::PLUS1), want, "{interrupter}");
    }
}

#[test]
fn a_copy_of_memory_deluge_or_search_for_glory_had_no_mana_spent() {
    cr!("707.10", "601.2h", "107.4h");
    ruling!(
        "Memory Deluge",
        "If Memory Deluge is copied, no mana was spent to cast the copy, so X will be 0."
    );
    ruling!(
        "Search for Glory",
        "If Search for Glory is copied, no snow mana was spent to cast the copy, so you won’t gain any life."
    );
    supported("Twincast");
    // Memory Deluge for {2}{U}{U} copied by Twincast: the copy looks at nothing (no choice
    // offered), the original at four.
    let mut t = TestGame::new(2);
    let deluge = cast(&mut t, P0, "Memory Deluge", "2UU", &[]);
    cast(&mut t, P0, "Twincast", "UU", &[obj(deluge)]);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "the copy and the original");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve();
    let looked: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert!(looked.is_empty(), "{looked:?}");
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Search for Glory cast with three snow mana and copied: the original gains 3, the
    // copy nothing, and both search.
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Thalia, Guardian of Thraben");
    let b = t.library_top(P0, "Snow-Covered Forest");
    t.lands(P0, "Snow-Covered Plains", 3);
    let s = t.hand(P0, "Search for Glory");
    t.cast(P0, s).go();
    cast(&mut t, P0, "Twincast", "UU", &[obj(s)]);
    t.answer_choose(P0, &[obj(b)]);
    t.answer_choose(P0, &[obj(a)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Thalia, Guardian of Thraben"));
    assert!(t.in_hand(P0, "Snow-Covered Forest"));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn snow_permanent_cards_are_permanent_cards_with_snow() {
    cr!("205.4g", "110.4b");
    ruling!(
        "Search for Glory",
        "A snow permanent card is any artifact, creature, enchantment, land, or planeswalker card with the supertype snow."
    );
    ruling!(
        "Jorn, God of Winter // Kaldring, the Rimestaff",
        "A snow permanent card is any artifact, creature, enchantment, land, or planeswalker card with the supertype snow."
    );
    supported("Blizzard Brawl");
    // Search for Glory: Snow-Covered Forest is offered, the snow sorcery Blizzard Brawl
    // (not legendary) isn't.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Snow-Covered Forest");
    let brawl = t.library_top(P0, "Blizzard Brawl");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(forest)]);
    cast_resolve(&mut t, P0, "Search for Glory", "2W", &[]);
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("a search");
    assert!(offered.contains(&obj(forest)));
    assert!(!offered.contains(&obj(brawl)));
    assert!(t.in_hand(P0, "Snow-Covered Forest"));
    // Kaldring, the Rimestaff: "{T}: You may play target snow permanent card from your
    // graveyard this turn." It can target the Forest but not Blizzard Brawl.
    let mut t = TestGame::new(2);
    let kaldring = add(&mut t, P0, "Jorn, God of Winter // Kaldring, the Rimestaff");
    assert!(mtg_engine::dfc::transform(&mut t.g, kaldring));
    let kaldring = t.g.current(kaldring);
    t.g.recompute();
    assert_eq!(t.obj_now(kaldring).chars.name, "Kaldring, the Rimestaff");
    let forest = t.graveyard(P0, "Snow-Covered Forest");
    let brawl = t.graveyard(P0, "Blizzard Brawl");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(forest)]);
    crate::r_s06_common::activate_containing(&mut t, P0, kaldring, "snow permanent card")
        .expect("activate");
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("a target choice");
    assert!(offered.contains(&obj(forest)));
    assert!(!offered.contains(&obj(brawl)));
}
