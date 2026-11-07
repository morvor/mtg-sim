//! Rulings batch P029 — Melek, Izzet Paragon ("You may cast instant and sorcery spells from
//! the top of your library. Whenever you cast an instant or sorcery spell from your
//! library, copy it."): timing (CR 307.1, 117.1a), miracle casts from the hand
//! (CR 702.94a); God-Eternal Kefnet ("You may reveal the first card you draw each turn as
//! you draw it. Whenever you reveal an instant or sorcery card this way, copy that card and
//! you may cast the copy. That copy costs {2} less to cast."): the reveal (CR 701.20a),
//! sequential draws (CR 121.2), casting a copy of a card from the hand (CR 707.12), total
//! cost (CR 601.2f, 202.3), and the Gods leaving with a player who leaves (CR 800.4a).

use crate::r_p028_common::obj;
use crate::r_p029_common::*;
use crate::r_p030_common::{elementals, resolve_until_copies};
use crate::r_s01_common::supported;
use crate::r_s25_common::*;
use mtg_engine::ability::ZoneKind;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Melek, Izzet Paragon -----------------------------------------------------------------

#[test]
fn melek_copies_a_spell_without_targets_cast_from_the_library() {
    cr!("707.10", "115.1", "601.2a");
    ruling!(
        "Melek, Izzet Paragon",
        "Melek’s last ability will copy any instant or sorcery spell you cast from your library, not just one with targets."
    );
    supported("Melek, Izzet Paragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melek, Izzet Paragon");
    let div = t.library_top(P0, "Divination");
    lands_for_cost(&mut t, P0, "Divination");
    t.cast(P0, div).go();
    keep_copy_targets(&mut t, P0);
    let hand = t.hand_size(P0);
    assert_eq!(resolve_collecting_copies(&mut t).len(), 1);
    assert_eq!(t.hand_size(P0), hand + 4);
}

/// Whether P0's queued attempt to cast `card` (normally) from the priority loop leads to
/// it being cast before `until` holds.
fn tries_to_cast(
    t: &mut TestGame,
    card: ObjectId,
    until: impl FnMut(&mtg_engine::game::Game) -> bool,
) -> bool {
    let before = t.g.history.spells_cast.len();
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card,
            method: CastMethod::Normal,
        }),
    );
    t.g.run_until(500, until);
    t.g.history.spells_cast.len() > before
}

/// Whether casting `card` normally is among P0's legal actions now.
fn castable(t: &mut TestGame, card: ObjectId) -> bool {
    t.g.recompute();
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

#[test]
fn melek_doesnt_change_when_a_sorcery_can_be_cast() {
    cr!("307.1", "117.1a", "601.3");
    ruling!(
        "Melek, Izzet Paragon",
        "Melek doesn’t affect the timing rules associated with when you can cast the card. If it’s a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    // P0's main phase, a Lightning Bolt on the stack: the Divination on top of the library
    // can't be cast; once the stack is empty, it can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melek, Izzet Paragon");
    let div = t.library_top(P0, "Divination");
    lands_for_cost(&mut t, P0, "Divination");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert!(!t.g.stack.is_empty());
    t.g.turn.priority = Some(P0);
    assert!(!castable(&mut t, div));
    let from = t.asked().len();
    assert!(!tries_to_cast(&mut t, div, |g| g.stack.is_empty()));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Priority { .. })));
    assert_eq!(t.zone(div), Zone::Library(P0));
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    assert!(tries_to_cast(&mut t, div, |g| !g.stack.is_empty()));
    assert_eq!(t.g.obj(t.g.stack[0]).chars.name, "Divination");
    // On P1's turn, it can't be cast at all.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Melek, Izzet Paragon");
    let div = t.library_top(P0, "Divination");
    lands_for_cost(&mut t, P0, "Divination");
    t.g.turn.priority = Some(P0);
    assert!(!castable(&mut t, div));
    let from = t.asked().len();
    assert!(!tries_to_cast(&mut t, div, |g| g.turn.active == P0));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Priority { .. })));
    // An instant on top of the library can be cast then (through the priority loop).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Melek, Izzet Paragon");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(tries_to_cast(&mut t, bolt, |g| !g.stack.is_empty()));
    assert_eq!((t.g.turn.active, t.g.turn.step), (P1, Step::PrecombatMain));
    assert_eq!(t.g.obj(t.g.stack[0]).chars.name, "Lightning Bolt");
}

#[test]
fn melek_doesnt_copy_a_miracle_cast_from_the_hand() {
    cr!("702.94a", "601.2a");
    ruling!(
        "Melek, Izzet Paragon",
        "Instant and sorcery cards with miracle allow a player to cast a card immediately upon drawing it. If you cast a spell this way, you’re casting it from your hand, not your library. Melek’s ability won’t trigger."
    );
    supported("Thunderous Wrath");
    // Thunderous Wrath: "Thunderous Wrath deals 5 damage to any target. Miracle {R}".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melek, Izzet Paragon");
    t.library_top(P0, "Thunderous Wrath");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast it for its miracle cost
    t.answer_targets(P0, &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve();
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).chars.name, "Thunderous Wrath");
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().cast.from,
        Some(ZoneKind::Hand)
    );
    let copies = resolve_collecting_copies(&mut t);
    assert!(copies.is_empty());
    assert_eq!(t.life(P1), 15);
}

// --- God-Eternal Kefnet ----------------------------------------------------------------------

/// Starts P0's turn with Kefnet on the battlefield and `top` (top first) on top of P0's
/// library, just before the draw step's draw. Returns Kefnet.
fn kefnet_game(top: &[&str]) -> (TestGame, ObjectId) {
    supported("God-Eternal Kefnet");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let k = t.battlefield(P0, "God-Eternal Kefnet");
    for name in top.iter().rev() {
        t.library_top(P0, name);
    }
    (t, k)
}

/// The prompts asking P0 whether to reveal a card being drawn, since `from`.
fn reveal_prompts(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| {
            *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("eveal"))
        })
        .count()
}

#[test]
fn kefnet_s_revealed_card_stays_revealed_until_the_ability_resolves() {
    cr!("701.20a", "701.20b");
    ruling!(
        "God-Eternal Kefnet",
        "If you reveal a card this way, it remains revealed until Kefnet's triggered ability finishes resolving."
    );
    let (mut t, _) = kefnet_game(&["Divination"]);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, false); // don't cast the copy
    t.advance_to_step(Step::Draw);
    t.settle();
    let div = t.g.find_in_zone(Zone::Hand(P0), "Divination")[0];
    assert_eq!(t.stack_len(), 1);
    assert!(is_revealed(&t.g, div));
    // Revealed while the ability is on the stack; no longer once it has resolved.
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(!is_revealed(&t.g, div));
    assert!(t.in_hand(P0, "Divination"));
}

#[test]
fn only_the_first_of_several_cards_drawn_at_once_may_be_revealed_for_kefnet() {
    cr!("121.2", "701.20a");
    ruling!(
        "God-Eternal Kefnet",
        "Multiple card draws are always treated as a sequence of individual card draws. For example, if you haven't drawn any cards yet during a turn and cast a spell that instructs you to draw three cards, you'll draw them one at a time. Only the first card drawn this way may be revealed and copied with Kefnet's ability."
    );
    // P0's turn, no card drawn yet: P0 draws three cards (as Divination's "draw" would),
    // the second a Lightning Bolt. Only the first (an Island) may be revealed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Eternal Kefnet");
    t.library_top(P0, "Island");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Island");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let from = t.asked().len();
    t.g.draw_cards(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(reveal_prompts(&t, from), 1);
    assert!(t.g.stack.is_empty());
    let bolt = t.g.find_in_zone(Zone::Hand(P0), "Lightning Bolt")[0];
    assert!(!is_revealed(&t.g, bolt));
    // The Bolt first: it is revealed (and stays revealed while Kefnet's ability, which
    // copies it, is on the stack).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Eternal Kefnet");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.g.draw_cards(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(reveal_prompts(&t, from), 1);
    assert_eq!(t.stack_len(), 1);
    let bolt = t.g.find_in_zone(Zone::Hand(P0), "Lightning Bolt")[0];
    assert!(is_revealed(&t.g, bolt));
}

#[test]
fn kefnet_s_copy_is_created_in_and_cast_from_the_hand() {
    cr!("707.12", "601.2a", "603.2");
    ruling!(
        "God-Eternal Kefnet",
        "The copy is created in and cast from your hand."
    );
    supported("Young Pyromancer");
    let (mut t, _) = kefnet_game(&["Lightning Bolt"]);
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast the copy
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to_step(Step::Draw);
    t.settle();
    t.resolve();
    let copy =
        *t.g.stack
            .iter()
            .find(|id| t.g.obj(**id).chars.name == "Lightning Bolt")
            .unwrap();
    assert_eq!(
        t.g.obj(copy).stack.as_ref().unwrap().cast.from,
        Some(ZoneKind::Hand)
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The copy was cast: Young Pyromancer triggered.
    assert_eq!(elementals(&t, P0), 1);
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn kefnet_s_reduction_applies_after_increases_and_the_mana_value_is_unchanged() {
    cr!("601.2f", "202.3", "707.12");
    ruling!(
        "God-Eternal Kefnet",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Kefnet's ability). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Thalia, Guardian of Thraben");
    // Thalia: "Noncreature spells cost {1} more to cast." The increase is added before the
    // reduction: Lightning Bolt ({R}) costs {R} + {1} - {2} = {R}, one Mountain. (Reduced
    // first, {R} - {2} would stay {R} and then cost {1}{R}.)
    let (mut t, _) = kefnet_game(&["Lightning Bolt"]);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let mountain = t.lands(P0, "Mountain", 1)[0];
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast the copy
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to_step(Step::Draw);
    t.settle();
    t.resolve();
    let copy =
        t.g.stack
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).chars.name == "Lightning Bolt")
            .expect("cast for {R}");
    assert_eq!(t.g.mana_value_of(copy), 1);
    assert!(t.obj_now(mountain).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Divination ({2}{U}) costs {2}{U} + {1} - {2} = {1}{U}: two lands, both tapped; its
    // mana value stays 3. One Island isn't enough.
    for lands in [2, 1] {
        let (mut t, _) = kefnet_game(&["Divination"]);
        t.battlefield(P1, "Thalia, Guardian of Thraben");
        let mut ls = t.lands(P0, "Island", 1);
        ls.extend(t.lands(P0, "Wastes", lands - 1));
        t.answer_yes(P0, true); // reveal
        t.answer_yes(P0, true); // cast the copy
        t.advance_to_step(Step::Draw);
        t.settle();
        t.resolve();
        let copy =
            t.g.stack
                .iter()
                .copied()
                .find(|id| t.g.obj(*id).chars.name == "Divination");
        if lands == 2 {
            let copy = copy.expect("cast for {1}{U}");
            assert_eq!(t.g.mana_value_of(copy), 3);
            assert!(ls.iter().all(|l| t.obj_now(*l).tapped));
            let hand = t.hand_size(P0);
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 2);
        } else {
            // {1}{U} can't be paid with one Island.
            assert!(copy.is_none());
        }
    }
}

#[test]
fn two_kefnets_copy_the_card_one_at_a_time_each_reducing_by_two() {
    cr!("603.3b", "707.12", "601.2f", "608.2g");
    ruling!(
        "God-Eternal Kefnet",
        "If you somehow control more than one God-Eternal Kefnet (perhaps because one is a Spark Double), you may reveal a card you draw for any number of their abilities. One at a time, each will copy the card if it's an instant or sorcery, and you may cast each of them. Each copy resolves before you cast the next, and each copy's cost is reduced by only {2}."
    );
    supported("Spark Double");
    // Tidings ({3}{U}{U}: "Draw four cards.") costs {1}{U}{U} each time: six Islands.
    let (mut t, k) = kefnet_game(&["Tidings"]);
    // Spark Double entering as a copy of Kefnet (it isn't legendary).
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(k)]);
    t.enter(P0, "Spark Double");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("God-Eternal Kefnet").len(), 2);
    let islands = t.lands(P0, "Island", 6);
    for _ in 0..6 {
        t.answer_yes(P0, true);
    }
    t.advance_to_step(Step::Draw);
    t.settle();
    // Two triggers, one per Kefnet.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The first copy is on the stack above the second trigger, and resolves first.
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Tidings");
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(islands.iter().filter(|l| t.obj_now(**l).tapped).count(), 3);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 8);
    // {1}{U}{U} twice: all six Islands, not four.
    assert!(islands.iter().all(|l| t.obj_now(*l).tapped));
}

/// The position of the card `id` (followed across zone changes) in `p`'s library, from the
/// top (0 is the top card).
fn from_top(t: &TestGame, p: PlayerId, id: ObjectId) -> Option<usize> {
    let now = t.g.current(id);
    t.g.player(p).library.iter().rev().position(|c| *c == now)
}

#[test]
fn another_player_s_god_leaves_with_the_player_who_put_it_onto_the_battlefield() {
    cr!("800.4a", "704.5a", "704.5g");
    ruling!(
        "God-Eternal Kefnet",
        "In a multiplayer game, if you put another player's God onto the battlefield under your control, it will be exiled as you leave the game. If you were still the controller of that God, you would control its triggered ability but you have left the game; that ability won't resolve and the card remains in exile. Similarly, if you lose the game at the same time that another player's God that you put onto the battlefield is destroyed, it remains in its owner's graveyard."
    );
    ruling!(
        "God-Eternal Oketra",
        "In a multiplayer game, if you put another player's God onto the battlefield under your control, it will be exiled as you leave the game. If you were still the controller of that God, you would control its triggered ability but you have left the game; that ability won't resolve and the card remains in exile. Similarly, if you lose the game at the same time that another player's God that you put onto the battlefield is destroyed, it remains in its owner's graveyard."
    );
    ruling!(
        "God-Eternal Rhonas",
        "In a multiplayer game, if you put another player's God onto the battlefield under your control, it will be exiled as you leave the game. If you were still the controller of that God, you would control its triggered ability but you have left the game; that ability won't resolve and the card remains in exile. Similarly, if you lose the game at the same time that another player's God that you put onto the battlefield is destroyed, it remains in its owner's graveyard."
    );
    ruling!(
        "Ilharg, the Raze-Boar",
        "In a multiplayer game, if you put another player's God onto the battlefield under your control, it will be exiled as you leave the game. If you were still the controller of that God, you would control its triggered ability but you have left the game; that ability won't resolve and the card remains in exile. Similarly, if you lose the game at the same time that another player's God that you put onto the battlefield is destroyed, it remains in its owner's graveyard."
    );
    supported("Reanimate");
    for god in [
        "God-Eternal Kefnet",
        "God-Eternal Oketra",
        "God-Eternal Rhonas",
        "Ilharg, the Raze-Boar",
    ] {
        supported(god);
        // 0: P0 stays and the God is exiled (the ability resolves: positive control);
        // 1: P0 leaves the game; 2: P0 loses as the God dies.
        for how in 0..3 {
            let mut t = TestGame::new(4);
            // Reanimate: "Put target creature card from a graveyard onto the battlefield
            // under your control. You lose life equal to its mana value."
            let card = t.graveyard(P1, god);
            cast_new(&mut t, P0, "Reanimate", &[obj(card)]);
            t.resolve_all();
            let g = t.g.current(card);
            assert!(t.on_battlefield(g), "{god}");
            assert_eq!((t.obj(g).owner, t.obj(g).controller), (P1, P0));
            for p in [P0, P1, P2, P3] {
                for _ in 0..3 {
                    t.answer_yes(p, true);
                }
            }
            match how {
                0 => {
                    t.g.exile_object(g, None);
                    t.g.flush_events();
                    t.resolve_all();
                    assert_eq!(from_top(&t, P1, card), Some(2), "{god}");
                }
                1 => {
                    t.g.lose_game(P0);
                    t.g.flush_events();
                    t.resolve_all();
                    assert!(t.g.player(P0).left_game);
                    assert!(t.g.result.is_none());
                    assert_eq!(t.zone(t.g.current(card)), Zone::Exile, "{god}");
                    assert_eq!(from_top(&t, P1, card), None, "{god}");
                }
                _ => {
                    // Lethal damage on the God and no life: state-based actions together.
                    t.g.player_mut(P0).life = 0;
                    let tough = t.obj_now(g).chars.toughness.unwrap_or(0);
                    t.g.objects[g.0 as usize].damage = tough as u32;
                    t.resolve_all();
                    assert!(t.g.player(P0).left_game);
                    assert_eq!(t.zone(t.g.current(card)), Zone::Graveyard(P1), "{god}");
                    assert_eq!(from_top(&t, P1, card), None, "{god}");
                }
            }
        }
    }
}
