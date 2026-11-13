//! Rulings batch P139 — "each opponent sacrifices ..." and other "each player does X"
//! effects in a multiplayer game: the players choose in turn order (starting with the
//! active player, or the next opponent in turn order on your turn), each knowing the
//! earlier choices, and the chosen permanents are sacrificed (or the chosen cards
//! discarded, returned, put onto the battlefield) at the same time (CR 101.4, 701.21a).

use crate::r_p137_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::damage;
use crate::r_s19_common::add_lore;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The number of permanents now, then everything on the stack resolved.
fn go(t: &mut TestGame) -> usize {
    t.settle();
    let n = t.g.permanents().count();
    t.resolve_all();
    n
}

/// P0 casts `card` (paying for it with basic lands put onto the battlefield first).
fn cast_and_go(t: &mut TestGame, card: &str) -> usize {
    give_mana_for(t, P0, card);
    let spell = t.hand(P0, card);
    t.cast(P0, spell).go();
    go(t)
}

/// P0's permanent `card` enters (its enters trigger goes on the stack).
fn enter_and_go(t: &mut TestGame, card: &str) -> usize {
    t.enter(P0, card);
    go(t)
}

/// A creature P0 controls dies (with the dies trigger on the stack, then resolved).
fn p0_creature_dies(t: &mut TestGame) -> usize {
    let bears = t.battlefield(P0, "Wall of Stone");
    destroy(t, bears);
    go(t)
}

fn is_discard_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard"))
}

/// The number of cards in each player's hand (P0, P1, P2).
fn hands(g: &mtg_engine::game::Game) -> [usize; 3] {
    [0, 1, 2].map(|i| g.players[i].hand.len())
}

#[test]
fn edicts_on_your_turn_start_with_the_next_opponent() {
    cr!("101.4", "101.4b", "701.21a", "608.2e");
    ruling!(
        "Sheoldred // The True Scriptures",
        "To resolve Sheoldred’s enters-the-battlefield ability in a multiplayer game, the next opponent in turn order (or, if it’s an opponent’s turn, the opponent whose turn it is) chooses a creature they control to sacrifice. Then each other opponent in turn order does the same. Then those creatures are sacrificed at the same time."
    );
    check_edict(
        "Sheoldred // The True Scriptures",
        P0,
        P1,
        BEARS_GIANT,
        |t| enter_and_go(t, "Sheoldred // The True Scriptures"),
    );
    ruling!(
        "The Eldest Reborn",
        "When the first chapter ability of The Eldest Reborn resolves, the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a creature or planeswalker they control, then each other opponent in turn order (if any) does the same. All chosen permanents are then sacrificed at the same time. Players will know the choices made by earlier players when making their choices."
    );
    check_edict("The Eldest Reborn", P0, P1, BEARS_GIANT, |t| {
        enter_and_go(t, "The Eldest Reborn")
    });
    ruling!(
        "Kefka, Court Mage // Kefka, Ruler of Ruin",
        "While resolving Kefka, Court Mage's last ability, the next opponent in turn order chooses a permanent they control, then each other opponent in turn order does the same. Then each of the chosen permanents are sacrificed simultaneously."
    );
    let t = check_edict(
        "Kefka, Court Mage // Kefka, Ruler of Ruin",
        P0,
        P1,
        BEARS_GIANT,
        |t| {
            let kefka = t.battlefield(P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
            t.lands(P0, "Wastes", 8);
            t.activate(P0, kefka, 0, &[]).unwrap();
            go(t)
        },
    );
    assert_eq!(t.named_on_battlefield("Kefka, Ruler of Ruin").len(), 1);
    ruling!(
        "Dark Intimations",
        "When Dark Intimations resolves, first the player whose turn it is (if that player is an opponent) chooses which creature or planeswalker they will sacrifice, then each other opponent in turn order does the same, then all chosen permanents are sacrificed at the same time. Then each opponent in the same order chooses a card in their hand without revealing it, then all chosen cards are discarded at the same time."
    );
    // P1 and P2 each hold two cards: the discards happen together after the sacrifices.
    let mut seen = None;
    let t = check_edict("Dark Intimations", P0, P1, BEARS_GIANT, |t| {
        for p in [P1, P2] {
            t.hand(p, "Island");
            t.hand(p, "Swamp");
        }
        seen = Some(watch(t, P2, is_discard_choice, hands));
        cast_and_go(t, "Dark Intimations")
    });
    let seen = seen.unwrap();
    // When P2 chose its discard (after P1 chose), P1 hadn't discarded yet.
    assert_eq!(
        seen.lock().unwrap().first().map(|h| (h[1], h[2])),
        Some((2, 2))
    );
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (1, 1));
}

#[test]
fn edicts_on_an_opponents_turn_start_with_that_opponent() {
    cr!("101.4", "101.4b", "701.21a");
    ruling!(
        "Grave Pact",
        "When the triggered ability resolves, first the player whose turn it is (if it isn't your turn) chooses which creature they will sacrifice, then each other player in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    check_edict("Grave Pact", P2, P2, BEARS_GIANT, |t| {
        t.battlefield(P0, "Grave Pact");
        p0_creature_dies(t)
    });
    ruling!(
        "Butcher of Malakir",
        "When the triggered ability resolves, first the player whose turn it is (if that player is an opponent) chooses which creature they will sacrifice, then each other opponent in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    check_edict("Butcher of Malakir", P2, P2, BEARS_GIANT, |t| {
        t.battlefield(P0, "Butcher of Malakir");
        p0_creature_dies(t)
    });
    ruling!(
        "Dictate of Erebos",
        "When the triggered ability resolves, first the player whose turn it is (if that player is an opponent) chooses which creature they will sacrifice, then each other opponent in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    check_edict("Dictate of Erebos", P1, P1, BEARS_GIANT, |t| {
        t.battlefield(P0, "Dictate of Erebos");
        p0_creature_dies(t)
    });
    ruling!(
        "Silverclad Ferocidons",
        "When Silverclad Ferocidons's triggered ability resolves, first the player whose turn it is (if that player is an opponent) chooses which permanent they will sacrifice, then each other opponent in turn order does the same, then all chosen permanents are sacrificed at the same time. Players will know choices made by earlier players when making their choices."
    );
    check_edict("Silverclad Ferocidons", P2, P2, BEARS_GIANT, |t| {
        let dino = t.battlefield(P0, "Silverclad Ferocidons");
        let src = t.battlefield(P0, "Prodigal Pyromancer");
        damage(t, src, 1, dino);
        go(t)
    });
}

#[test]
fn perilous_predicament_each_opponent_chooses_both_creatures_in_turn() {
    cr!("101.4", "701.21a");
    ruling!(
        "Perilous Predicament",
        "When Perilous Predicament resolves, first the player whose turn it is (if that player is an opponent) chooses which creatures they will sacrifice, then each other opponent in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    supported("Perilous Predicament");
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    let mut kept = vec![];
    let mut lost = vec![];
    for p in [P1, P2] {
        let thopter = t.battlefield(p, "Ornithopter");
        t.battlefield(p, "Memnite");
        let a = t.battlefield(p, "Grizzly Bears");
        let b = t.battlefield(p, "Hill Giant");
        t.answer_choose(p, &[obj(thopter)]);
        t.answer_choose(p, &[obj(b)]);
        kept.push(a);
        lost.extend([thopter, b]);
    }
    let seen = [P1, P2].map(|p| watch(&mut t, p, is_choose, |g| g.permanents().count()));
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    let spell = t.hand(P0, "Perilous Predicament");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    let before = t.g.permanents().count();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P2, P2, P1, P1]);
    for s in &seen {
        assert_eq!(
            *s.lock().unwrap(),
            vec![before; 2],
            "nothing sacrificed yet {:?}",
            seen
        );
    }
    assert!(kept.iter().all(|k| t.on_battlefield(*k)));
    assert!(lost.iter().all(|l| !t.on_battlefield(*l)));
}

#[test]
fn release_each_player_chooses_in_turn_then_all_sacrifice_together() {
    cr!("101.4", "701.21a");
    // "Each player sacrifices an artifact, a creature, an enchantment, a land, and a
    // planeswalker of their choice." compiles like Perilous Predicament: every player's
    // choices come before anyone's sacrifices.
    ruling!(
        "Catch // Release",
        "To resolve Release, the active player chooses a permanent they control for each of the listed types, then each other player in turn order does the same, then all the chosen permanents are sacrificed simultaneously."
    );
    supported("Catch // Release");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Plains", 1);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    for p in [P1, P2] {
        t.battlefield(p, "Grizzly Bears");
        t.battlefield(p, "Hill Giant");
        t.battlefield(p, "Forest");
        t.battlefield(p, "Island");
    }
    let seen = [P1, P2].map(|p| watch(&mut t, p, is_choose, |g| g.permanents().count()));
    let spell = t.hand(P0, "Catch // Release");
    let from = t.asked().len();
    t.cast(P0, spell)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    let before = t.g.permanents().count();
    t.resolve_all();
    let mut order = choosers_since(&t, from);
    order.dedup();
    assert_eq!(order, vec![P0, P1, P2]);
    for s in &seen {
        let s = s.lock().unwrap();
        assert_eq!(s.len(), 2, "a creature and a land");
        assert!(
            s.iter().all(|n| *n == before),
            "nothing sacrificed yet: {before} {s:?}"
        );
    }
    // Each player sacrificed a creature and a land.
    for p in [P0, P1, P2] {
        assert_eq!(count_of(&t, p, CardType::Creature), 1, "{p:?}");
    }
    assert_eq!(count_of(&t, P0, CardType::Land), 5);
    assert_eq!(count_of(&t, P1, CardType::Land), 1);
    assert_eq!(count_of(&t, P2, CardType::Land), 1);
}

#[test]
fn liliana_of_the_veil_discards_start_with_the_active_player_and_happen_together() {
    cr!("101.4", "701.9a");
    ruling!(
        "Liliana of the Veil",
        "When Liliana’s first ability resolves, first the player whose turn it is chooses a card in hand without revealing it, then each other player in turn order does the same. Then all the chosen cards are discarded at the same time."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(3);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    for p in [P0, P1, P2] {
        t.hand(p, "Island");
        t.hand(p, "Swamp");
    }
    let seen = [P0, P1, P2].map(|p| watch(&mut t, p, is_discard_choice, hands));
    let from = t.asked().len();
    t.activate(P0, lili, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P1, P2]);
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![[2, 2, 2]], "nothing discarded yet");
    }
    assert_eq!(hands(&t.g), [1, 1, 1]);
}

#[test]
fn eldest_reborn_chapter_two_discards_in_turn_order_at_the_same_time() {
    cr!("101.4", "714.2b");
    ruling!(
        "The Eldest Reborn",
        "When the second chapter ability of The Eldest Reborn resolves, the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a card in hand without revealing it, then each other opponent in turn order (if any) does the same. All chosen cards are then discarded at the same time."
    );
    supported("The Eldest Reborn");
    let mut t = TestGame::new(3);
    let saga = t.battlefield(P0, "The Eldest Reborn");
    add_lore(&mut t, saga, 1);
    for p in [P0, P1, P2] {
        t.hand(p, "Island");
        t.hand(p, "Swamp");
    }
    let seen = [P1, P2].map(|p| watch(&mut t, p, is_discard_choice, hands));
    let from = t.asked().len();
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1, P2]);
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![[2, 2, 2]]);
    }
    assert_eq!(hands(&t.g), [2, 1, 1]);
}

#[test]
fn vile_mutilator_enchantments_then_creatures_each_in_turn_order() {
    cr!("101.4", "608.2e", "701.21a");
    ruling!(
        "Vile Mutilator",
        "While resolving Vile Mutilator's last ability, the next opponent in turn order chooses a nontoken enchantment they control, then each other opponent in turn order does the same. Then each of the chosen nontoken enchantments are sacrificed simultaneously. Finally, repeat this process for nontoken creatures."
    );
    supported("Vile Mutilator");
    let mut t = TestGame::new(3);
    let mut kept = vec![];
    let mut lost = vec![];
    for p in [P1, P2] {
        let e = t.battlefield(p, "Glorious Anthem");
        t.battlefield(p, "Intangible Virtue");
        let a = t.battlefield(p, "Grizzly Bears");
        let b = t.battlefield(p, "Hill Giant");
        t.answer_choose(p, &[obj(e)]);
        t.answer_choose(p, &[obj(b)]);
        kept.push(a);
        lost.extend([e, b]);
    }
    // When P2 chooses its creature, both enchantments are gone and no creature is.
    let seen = watch(&mut t, P2, is_choose, |g| {
        (
            g.permanents()
                .filter(|o| o.is(CardType::Enchantment))
                .count(),
            g.permanents().filter(|o| o.is(CardType::Creature)).count(),
        )
    });
    let from = t.asked().len();
    enter_and_go(&mut t, "Vile Mutilator");
    let order: Vec<_> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order, vec![P1, P2, P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![(4, 5), (2, 5)]);
    assert!(kept.iter().all(|k| t.on_battlefield(*k)));
    assert!(lost.iter().all(|l| !t.on_battlefield(*l)));
}

#[test]
fn ajani_avenger_opponents_choose_in_turn_order_then_sacrifice_the_rest() {
    cr!("101.4", "701.21a");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "When Ajani, Nacatl Avenger's last ability resolves, starting with the next opponent in turn order, each opponent in turn order chooses permanents they control. Each opponent will know the choices made by players who chose before them. They then sacrifice all of their unchosen nonland permanents simultaneously."
    );
    const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
    supported(AJANI);
    let mut t = TestGame::new(3);
    let card = t.graveyard(P0, AJANI);
    let ajani = crate::r_s17_common::put_onto_battlefield(&mut t, P0, card, true).unwrap();
    let have = t.counters(ajani, "loyalty");
    t.g.add_counters(Entity::Object(ajani), "loyalty", 5 - have, None);
    let mut kept = vec![];
    let mut lost = vec![];
    for p in [P1, P2] {
        let a = t.battlefield(p, "Grizzly Bears");
        let b = t.battlefield(p, "Hill Giant");
        let land = t.battlefield(p, "Forest");
        t.answer_choose(p, &[obj(a)]);
        kept.extend([a, land]);
        lost.push(b);
    }
    let seen = [P1, P2].map(|p| watch(&mut t, p, is_choose, |g| g.permanents().count()));
    let from = t.asked().len();
    t.activate(P0, ajani, 2, &[]).unwrap();
    let before = t.g.permanents().count();
    t.resolve_all();
    let order: Vec<_> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order[0], P1);
    assert_eq!(*order.last().unwrap(), P2);
    for s in &seen {
        assert!(
            s.lock().unwrap().iter().all(|n| *n == before),
            "nothing sacrificed yet: {before} {:?}",
            s.lock().unwrap()
        );
    }
    assert!(kept.iter().all(|k| t.on_battlefield(*k)));
    assert!(lost.iter().all(|l| !t.on_battlefield(*l)));
}

/// P0 casts a friend-or-foe sorcery calling every player in `friends` a friend and the
/// others foes.
fn friend_or_foe(t: &mut TestGame, card: &str, friends: &[PlayerId]) {
    for p in [P0, P1, P2] {
        let i = if friends.contains(&p) { 0 } else { 1 };
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    give_mana_for(t, P0, card);
    let spell = t.hand(P0, card);
    t.cast(P0, spell).go();
}

#[test]
fn pirs_whim_and_virtuss_maneuver_foes_choose_in_turn_order() {
    cr!("101.4", "101.4b", "701.21a");
    ruling!(
        "Pir's Whim",
        "When Pir’s Whim resolves and your foes perform their actions, first the next foe in turn order (or, if it’s a foe’s turn, that foe) chooses an artifact or enchantment they control, then each other foe in turn order does the same, then all chosen permanents are sacrificed at the same time. Foes will know choices made by earlier players when making their choices."
    );
    check_edict("Pir's Whim", P0, P1, ["Sol Ring", "Glorious Anthem"], |t| {
        friend_or_foe(t, "Pir's Whim", &[P0]);
        go(t)
    });
    ruling!(
        "Virtus's Maneuver",
        "When Virtus’s Maneuver resolves and you choose friend or foe, first the next friend in turn order (or, if it’s a friend’s turn, that friend) chooses a creature card in their graveyard, then each other friend in turn order does the same, then each of those players return those cards to their hands at the same time. Friends will know choices made by earlier players when making their choices. Repeat this process for foes, who choose a creature one at a time and then sacrifice them all at the same time."
    );
    check_edict("Virtus's Maneuver", P0, P1, BEARS_GIANT, |t| {
        friend_or_foe(t, "Virtus's Maneuver", &[P0]);
        go(t)
    });
}

#[test]
fn virtuss_maneuver_friends_choose_in_turn_order_then_return_together() {
    cr!("101.4");
    ruling!(
        "Virtus's Maneuver",
        "When Virtus’s Maneuver resolves and you choose friend or foe, first the next friend in turn order (or, if it’s a friend’s turn, that friend) chooses a creature card in their graveyard, then each other friend in turn order does the same, then each of those players return those cards to their hands at the same time. Friends will know choices made by earlier players when making their choices. Repeat this process for foes, who choose a creature one at a time and then sacrifice them all at the same time."
    );
    supported("Virtus's Maneuver");
    let mut t = TestGame::new(3);
    let mut chosen = vec![];
    for p in [P1, P2] {
        let a = t.graveyard(p, "Grizzly Bears");
        t.graveyard(p, "Hill Giant");
        t.answer_choose(p, &[obj(a)]);
        chosen.push(a);
    }
    let seen = [P1, P2].map(|p| watch(&mut t, p, is_choose, hands));
    let from = t.asked().len();
    // P0 is a foe with nothing to sacrifice; P1 and P2 are friends.
    friend_or_foe(&mut t, "Virtus's Maneuver", &[P1, P2]);
    t.resolve_all();
    let order: Vec<_> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order, vec![P1, P2]);
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![[0, 0, 0]], "nothing returned yet");
    }
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P2, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant") && t.in_graveyard(P2, "Hill Giant"));
}

#[test]
fn pirs_whim_friends_search_in_turn_order_then_lands_enter_together() {
    cr!("101.4", "701.23a", "701.23i");
    ruling!(
        "Pir's Whim",
        "When Pir’s Whim resolves and your friends perform their actions, first the next friend in turn order (or, if it’s a friend’s turn, that friend) searches their library for a land without revealing the card they choose, then each other friend in turn order does the same. Then all the lands are put onto the battlefield at the same time."
    );
    supported("Pir's Whim");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.library_top(p, "Plains");
    }
    let lands = |g: &mtg_engine::game::Game| {
        g.permanents()
            .filter(|o| o.chars.name == "Plains" && o.tapped)
            .count()
    };
    let seen = [P0, P1, P2].map(|p| watch(&mut t, p, is_choose, lands));
    let from = t.asked().len();
    // Everyone is a friend; it's P0's (a friend's) turn: P0 searches first.
    friend_or_foe(&mut t, "Pir's Whim", &[P0, P1, P2]);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P1, P2]);
    for s in &seen {
        assert_eq!(
            *s.lock().unwrap(),
            vec![0],
            "no land entered while searching"
        );
    }
    assert_eq!(lands(&t.g), 3);
}

#[test]
fn fandaniel_opponents_choose_up_to_one_then_those_who_didnt_lose_life() {
    cr!("101.4", "701.21a");
    ruling!(
        "Fandaniel, Telophoroi Ascian",
        "While resolving Fandaniel's second ability, the next opponent in turn order chooses up to one nontoken creature to be sacrificed, then each other opponent in turn order does the same. Then each of the chosen creatures are sacrificed simultaneously. Finally, each opponent who didn't sacrifice a creature this way loses 2 life for each instant and sorcery card in your graveyard."
    );
    supported("Fandaniel, Telophoroi Ascian");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Fandaniel, Telophoroi Ascian");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    // P1 sacrifices; P2 declines.
    t.answer_choose(P1, &[obj(b1)]);
    t.answer_yes(P1, true);
    t.answer_choose(P2, &[]);
    t.answer_yes(P2, false);
    let seen = watch(&mut t, P2, is_choose, |g| g.permanents().count());
    let from = t.asked().len();
    let before = t.g.permanents().count();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let order: Vec<_> = choosers_since(&t, from)
        .into_iter()
        .filter(|p| *p != P0)
        .collect();
    assert_eq!(order[0], P1);
    assert!(seen.lock().unwrap().iter().all(|n| *n == before));
    assert!(!t.on_battlefield(b1) && t.on_battlefield(b2));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 16);
}
