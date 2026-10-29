//! The game start, outside the game and the draft (cards5-game-rules-dice,
//! `game-start-outside`, `draft`): beginning the game with a card on the battlefield only
//! if you're not the starting player, with a counter, then exiling a card (Gemstone
//! Caverns); a delayed trigger on each opponent's first spell of the game (Chancellor of
//! the Annex); two cards with different names from outside the game and an opponent's
//! choice (Extrapolate the Impossible); adding a booster pack (Lore Seeker) and passing the
//! last cards to a drafter (Canal Dredger).

use crate::combat2_common::supported;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Decision;
use mtg_engine::draft::{Draft, DraftStyle};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

fn begin_prompts(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| {
            *q == p
                && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Begin the game with"))
        })
        .count()
}

// ---------------------------------------------------------------------------
// Gemstone Caverns
// ---------------------------------------------------------------------------

#[test]
fn gemstone_caverns_begins_the_game_with_a_luck_counter_and_exiles_a_card() {
    cr!("103.6", "103.6a", "122.6");
    supported("Gemstone Caverns");
    let mut t = TestGame::new(2);
    assert_eq!(t.g.turn.starting_player, P0);
    let caverns = t.hand(P1, "Gemstone Caverns");
    let opt = t.hand(P1, "Opt");
    let shock = t.hand(P1, "Shock");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(opt)]);
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    let c = t.g.current(caverns);
    assert!(t.on_battlefield(c));
    assert_eq!(t.counters(c, "luck"), 1);
    assert!(t.in_exile("Opt"));
    assert_eq!(t.zone(shock), Zone::Hand(P1));
    // With the luck counter, it taps for any color.
    let colors = mtg_engine::mana_abilities::could_produce(&t.g, c);
    assert!(colors.contains(&ManaType::G), "{colors:?}");
}

#[test]
fn gemstone_caverns_isnt_offered_to_the_starting_player() {
    cr!("103.1", "103.6");
    let mut t = TestGame::new(2);
    let caverns = t.hand(P0, "Gemstone Caverns");
    t.hand(P0, "Opt");
    t.answer_yes(P0, true);
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    assert_eq!(begin_prompts(&t, P0), 0);
    assert_eq!(t.zone(caverns), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn gemstone_caverns_declined_stays_in_hand() {
    cr!("103.6");
    let mut t = TestGame::new(2);
    let caverns = t.hand(P1, "Gemstone Caverns");
    t.hand(P1, "Opt");
    t.answer_yes(P1, false);
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    assert_eq!(begin_prompts(&t, P1), 1);
    assert_eq!(t.zone(caverns), Zone::Hand(P1));
    assert_eq!(t.hand_size(P1), 2);
}

// ---------------------------------------------------------------------------
// Chancellor of the Annex
// ---------------------------------------------------------------------------

/// `n` players; P0 reveals `chancellors` Chancellors of the Annex from their opening hand.
fn chancellor(n: usize, chancellors: usize) -> TestGame {
    let mut t = TestGame::new(n);
    for _ in 0..chancellors {
        t.hand(P0, "Chancellor of the Annex");
        t.answer_yes(P0, true);
    }
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    t
}

/// `p` casts Opt with `extra` generic mana to spare, answering `pay` to paying for it.
fn cast_opt(t: &mut TestGame, p: PlayerId, extra: u32, pay: bool) {
    let opt = t.hand(p, "Opt");
    t.g.players[p.idx()].mana_pool.add_type(ManaType::U, 1 + extra);
    t.answer_yes(p, pay);
    t.cast(p, opt).go();
    t.resolve_all();
}

#[test]
fn chancellor_counters_each_opponents_first_spell_unless_they_pay() {
    cr!("103.6b", "603.7g", "118.12");
    supported("Chancellor of the Annex");
    ruling!(
        "Chancellor of the Annex",
        "In a multiplayer game, if you reveal Chancellor of the Annex from your opening hand, it will trigger each time an opponent casts their first spell of the game."
    );
    let mut t = chancellor(3, 1);
    // P1 can't pay: countered.
    cast_opt(&mut t, P1, 0, true);
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(t.hand_size(P1), 0, "Opt didn't resolve");
    // P2 pays.
    let hand = t.hand_size(P2);
    cast_opt(&mut t, P2, 1, true);
    assert_eq!(t.hand_size(P2), hand + 1, "Opt resolved and drew a card");
    // Their second spells: no trigger.
    let hand = t.hand_size(P1);
    cast_opt(&mut t, P1, 0, true);
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn chancellor_doesnt_trigger_on_your_spells_or_when_not_revealed() {
    cr!("103.6b");
    let mut t = chancellor(2, 1);
    let hand = t.hand_size(P0);
    cast_opt(&mut t, P0, 0, true);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Not revealed: no trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Chancellor of the Annex");
    t.answer_yes(P0, false);
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    assert!(t.g.delayed_triggers.is_empty());
    cast_opt(&mut t, P1, 0, true);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn two_chancellors_each_trigger() {
    cr!("103.6b", "603.7g");
    ruling!(
        "Chancellor of the Annex",
        "If you reveal more than one Chancellor of the Annex from your opening hand, each will trigger"
    );
    // P1 can pay once, not twice.
    let mut t = chancellor(2, 2);
    t.answer_yes(P1, true);
    cast_opt(&mut t, P1, 1, true);
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(t.hand_size(P1), 0);
}

// ---------------------------------------------------------------------------
// Extrapolate the Impossible
// ---------------------------------------------------------------------------

fn outside(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let def: CardDef = (*card(name)).clone();
    t.custom(p, def, Zone::Outside(p))
}

#[test]
fn extrapolate_the_impossible_an_opponent_chooses_one_of_two_cards() {
    cr!("400.11", "400.11b", "608.2d");
    supported("Extrapolate the Impossible");
    let mut t = TestGame::new(2);
    let opt = outside(&mut t, P0, "Opt");
    let shock = outside(&mut t, P0, "Shock");
    let _bolt = outside(&mut t, P0, "Lightning Bolt");
    let spell = t.hand(P0, "Extrapolate the Impossible");
    t.g.players[0].mana_pool.add_type(ManaType::B, 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(opt), Entity::Object(shock)]);
    t.answer_choose(P1, &[Entity::Object(shock)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"));
    assert!(!t.in_hand(P0, "Opt"));
    // P1 chose among the two revealed cards.
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
            _ => None,
        })
        .expect("the opponent chose");
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&Entity::Object(opt)) && offered.contains(&Entity::Object(shock)));
}

#[test]
fn extrapolate_the_impossible_needs_two_different_names() {
    cr!("400.11");
    for (names, yes) in [(&["Opt", "Opt"][..], true), (&["Opt", "Shock"][..], false)] {
        let mut t = TestGame::new(2);
        for n in names {
            outside(&mut t, P0, n);
        }
        let spell = t.hand(P0, "Extrapolate the Impossible");
        t.g.players[0].mana_pool.add_type(ManaType::B, 2);
        t.answer_yes(P0, yes);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 0, "{names:?}, reveal: {yes}");
    }
}

// ---------------------------------------------------------------------------
// Lore Seeker, Canal Dredger
// ---------------------------------------------------------------------------

fn pack(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

fn names(cards: &[Arc<CardDef>]) -> Vec<String> {
    cards.iter().map(|c| c.name.to_string()).collect()
}

#[test]
fn lore_seeker_adds_a_booster_pack_drafted_next() {
    cr!("905.1a", "905.2", "905.2b");
    supported("Lore Seeker");
    ruling!(
        "Lore Seeker",
        "To add a booster pack to the draft, first pass the booster pack that contained Lore Seeker to the next player. Then open the new booster pack, draft a card, and pass the new booster pack in the same direction."
    );
    ruling!(
        "Lore Seeker",
        "Adding a booster pack to a draft may cause some players to have more cards in their card pools than others."
    );
    let mut d = Draft::new(
        vec![
            vec![pack(&["Lore Seeker", "Opt", "Shock"])],
            vec![pack(&["Grizzly Bears", "Hill Giant", "Memnite"])],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    // Revealed as it's drafted.
    assert!(d.can_see(P1, "Lore Seeker"));
    assert!(d.can_add_booster(P0));
    assert!(!d.can_add_booster(P1));
    d.add_booster(P0, pack(&["Divination", "Clone", "Wall of Wood"]))
        .unwrap();
    d.pick(P1, &[0]).unwrap();
    // P0 drafts from the new pack; the Lore Seeker pack went on.
    assert_eq!(names(d.pack(P0)), ["Divination", "Clone", "Wall of Wood"]);
    assert_eq!(names(d.pack(P1)), ["Opt", "Shock"]);
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    // Then the pack P1 passed, while the new pack goes on to P1.
    assert_eq!(names(d.pack(P0)), ["Hill Giant", "Memnite"]);
    assert_eq!(names(d.pack(P1)), ["Clone", "Wall of Wood"]);
    while !d.is_complete() {
        for p in [P0, P1] {
            if !d.pack(p).is_empty() {
                // (A player who drafted waits for the others.)
                let _ = d.pick(p, &[0]);
            }
        }
    }
    assert_eq!(d.pool(P0).len(), 5);
    assert_eq!(d.pool(P1).len(), 4);
}

#[test]
fn lore_seeker_adding_is_optional_and_only_right_after_drafting_it() {
    cr!("905.2");
    let mut d = Draft::new(
        vec![
            vec![pack(&["Lore Seeker", "Opt", "Shock"])],
            vec![pack(&["Grizzly Bears", "Hill Giant", "Memnite"])],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    assert!(!d.can_add_booster(P0));
    assert!(d.add_booster(P0, pack(&["Opt"])).is_err());
    while !d.is_complete() {
        for p in [P0, P1] {
            if !d.pack(p).is_empty() {
                // (A player who drafted waits for the others.)
                let _ = d.pick(p, &[0]);
            }
        }
    }
    assert_eq!(d.pool(P0).len(), 3);
    assert_eq!(d.pool(P1).len(), 3);
}

#[test]
fn canal_dredger_gets_the_last_card_of_each_pack() {
    cr!("905.1a", "905.2", "905.2c");
    supported("Canal Dredger");
    ruling!(
        "Canal Dredger",
        "If you draft a card named Canal Dredger, you pass the last card from each booster pack to yourself."
    );
    ruling!(
        "Canal Dredger",
        "You draft all cards passed to you this way one at a time."
    );
    let mut d = Draft::new(
        vec![
            vec![pack(&["Canal Dredger", "Opt", "Shock"])],
            vec![pack(&["Grizzly Bears", "Hill Giant", "Memnite"])],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap(); // Hill Giant
    d.pick(P1, &[0]).unwrap(); // Opt
    // Both packs have one card left: both come to P0, one at a time.
    assert_eq!(names(d.pack(P0)), ["Memnite"]);
    assert!(d.pack(P1).is_empty());
    d.pick(P0, &[0]).unwrap();
    assert_eq!(names(d.pack(P0)), ["Shock"]);
    d.pick(P0, &[0]).unwrap();
    assert!(d.is_complete());
    assert_eq!(
        names(&d.pool(P0)),
        ["Canal Dredger", "Hill Giant", "Memnite", "Shock"]
    );
    assert_eq!(names(&d.pool(P1)), ["Grizzly Bears", "Opt"]);
}

#[test]
fn canal_dredger_each_player_chooses_among_several_drafters() {
    cr!("905.2", "905.2a");
    ruling!(
        "Canal Dredger",
        "If multiple players have drafted a card named Canal Dredger, each player chooses which of those players to pass the last card to."
    );
    for to in [P0, P1] {
        let mut d = Draft::new(
            vec![
                vec![pack(&["Canal Dredger", "Opt"])],
                vec![pack(&["Canal Dredger", "Shock"])],
                vec![pack(&["Grizzly Bears", "Memnite"])],
            ],
            DraftStyle::Conspiracy,
            1,
        );
        d.choose_last_card_recipient(P2, to);
        for p in [P0, P1, P2] {
            d.pick(p, &[0]).unwrap();
        }
        // P2's pack's last card (Memnite) went to the chosen drafter; each drafter's own
        // to themselves.
        while !d.is_complete() {
            for p in [P0, P1, P2] {
                if !d.pack(p).is_empty() {
                    d.pick(p, &[0]).unwrap();
                }
            }
        }
        assert!(names(&d.pool(to)).contains(&"Memnite".to_string()), "to {to:?}");
        assert_eq!(d.pool(P2).len(), 1);
    }
}

#[test]
fn canal_dredger_default_recipient_is_the_next_drafter_in_the_passing_direction() {
    cr!("905.2a");
    interp!("canal-dredger-default-recipient");
    // Four players, passing left in the first round: P2's next drafter to the left is P3,
    // P0's (a drafter) is P0.
    let mut d = Draft::new(
        vec![
            vec![pack(&["Canal Dredger", "Opt"])],
            vec![pack(&["Grizzly Bears", "Shock"])],
            vec![pack(&["Hill Giant", "Memnite"])],
            vec![pack(&["Canal Dredger", "Clone"])],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    for p in [P0, P1, P2, P3] {
        d.pick(p, &[0]).unwrap();
    }
    while !d.is_complete() {
        for p in [P0, P1, P2, P3] {
            if !d.pack(p).is_empty() {
                // (A player who drafted waits for the others.)
                let _ = d.pick(p, &[0]);
            }
        }
    }
    assert!(names(&d.pool(P3)).contains(&"Memnite".to_string()));
    assert!(names(&d.pool(P0)).contains(&"Opt".to_string()));
    // P1's last card: the next drafter to its left is P3.
    assert!(names(&d.pool(P3)).contains(&"Shock".to_string()));
}
