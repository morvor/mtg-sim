//! Rulings batch P093 — replacement effects and the draw step: Blood Scrivener applied twice
//! (CR 614.5, 616.1), Necrodominance's graveyard replacement and hand size, and Sylvan
//! Library (draws replaced, several Libraries, other draw-step triggers).

use crate::r_p093_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Blood Scrivener ------------------------------------------------------------------------

#[test]
fn two_blood_scriveners_add_a_card_and_a_life_each() {
    cr!("614.5", "616.1", "121.6", "121.6b");
    ruling!(
        "Blood Scrivener",
        "Each additional Blood Scrivener you control will effectively add one card and 1 life lost."
    );
    supported("Blood Scrivener");
    // "If you would draw a card while you have no cards in hand, instead you draw two cards
    // and you lose 1 life." P0 controls `n` Scriveners and draws a card with an empty hand:
    // the draws and life losses, in order.
    fn draw_with(n: usize) -> (TestGame, Vec<&'static str>) {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Blood Scrivener");
        }
        assert_eq!(t.hand_size(P0), 0);
        let from = t.g.turn_events.len();
        t.g.draw_cards(P0, 1);
        t.g.flush_events();
        let seq = t.g.turn_events[from..]
            .iter()
            .filter_map(|e| match e {
                Event::Drew { player, .. } if *player == P0 => Some("draw"),
                Event::LifeLost { player, .. } if *player == P0 => Some("lose"),
                _ => None,
            })
            .collect();
        (t, seq)
    }
    // Draw two and lose 1 (the second Scrivener on the first of the first one's two
    // draws), then draw another card and lose another 1 life.
    let (mut t, seq) = draw_with(2);
    assert_eq!(seq, ["draw", "draw", "lose", "draw", "lose"]);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 18);
    // With a card in hand, neither applies.
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    assert_eq!((t.hand_size(P0), t.life(P0)), (4, 18));
    // One Scrivener: two cards and 1 life; each additional one adds a card and 1 life.
    let (t, seq) = draw_with(1);
    assert_eq!(seq, ["draw", "draw", "lose"]);
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 19));
    let (t, seq) = draw_with(3);
    assert_eq!(seq, ["draw", "draw", "lose", "draw", "lose", "draw", "lose"]);
    assert_eq!((t.hand_size(P0), t.life(P0)), (4, 17));
}

// --- Necrodominance -------------------------------------------------------------------------

#[test]
fn necrodominance_destroyed_by_your_own_spell() {
    cr!("608.2n", "614.1a");
    ruling!(
        "Necrodominance",
        "If a Necrodominance you control is destroyed by a spell you own, Necrodominance will be exiled and then the spell will be put into your graveyard."
    );
    supported("Necrodominance");
    let mut t = TestGame::new(2);
    let necro = t.battlefield(P0, "Necrodominance");
    cast_spell(&mut t, P0, "Disenchant", &[obj(necro)]);
    assert!(t.in_exile("Necrodominance"));
    assert!(t.in_graveyard(P0, "Disenchant"));
}

#[test]
fn necrodominance_and_spellbook_apply_in_timestamp_order() {
    cr!("613.7", "402.2", "613.11");
    ruling!(
        "Necrodominance",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Necrodominance onto the battlefield and then put Spellbook (an artifact that says you have no maximum hand size) onto the battlefield, you would have no maximum hand size. However, if those permanents entered the battlefield in the opposite order, your maximum hand size would be five."
    );
    supported("Spellbook");
    let mut t = TestGame::new(2);
    t.enter(P0, "Necrodominance");
    t.enter(P0, "Spellbook");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    let mut t = TestGame::new(2);
    t.enter(P0, "Spellbook");
    t.enter(P0, "Necrodominance");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(5));
}

#[test]
fn discarding_with_necrodominance_madness_and_connive_still_work() {
    cr!("702.35a", "701.50a", "614.6");
    ruling!(
        "Necrodominance",
        "If you discard a card while you control Necrodominance, abilities that function when a card is discarded (such as madness) still work, even though that card never reaches your graveyard. In addition, spells or abilities that check the characteristics of a discarded card (such as connive) can find that card in exile."
    );
    // Madness: Fiery Temper discarded to Mind Rot is cast for {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necrodominance");
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Fiery Temper");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_spell(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Fiery Temper"));
    // Connive: Raffine's Informant discards Lightning Bolt (into exile) and gets its
    // counter for the nonland card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necrodominance");
    t.library_top(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    let informant = t.enter(P0, "Raffine's Informant");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.counters(informant, counters::PLUS1), 1);
}

// --- Sylvan Library ---------------------------------------------------------------------------

const LIBRARY: &str = "Sylvan Library";
const PUT_BACK: usize = 0;
const PAY: usize = 1;

/// Queues `p`'s answers for one Sylvan Library trigger: draw (or not), then for each of
/// the chosen cards put it back or pay 4 life.
fn library_answers(t: &mut TestGame, p: PlayerId, draw: bool, choices: &[usize]) {
    t.answer_yes(p, draw);
    for c in choices {
        t.answer(p, DecisionKind::Option, Answer::Index(*c));
    }
}

/// From P1's end step, advances to P0's draw step (the normal draw done) and puts the
/// triggers on the stack.
fn to_draw_step(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    t.settle();
}

#[test]
fn sylvan_library_put_back_zero_one_or_two() {
    cr!("119.4", "603.5");
    ruling!("Sylvan Library", "You can put back zero, one, or two cards.");
    for (choices, hand, life) in [
        ([PAY, PAY], 3, 12),
        ([PUT_BACK, PAY], 2, 16),
        ([PUT_BACK, PUT_BACK], 1, 20),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, LIBRARY);
        library_answers(&mut t, P0, true, &choices);
        to_draw_step(&mut t);
        t.resolve_all();
        assert_eq!((t.hand_size(P0), t.life(P0)), (hand, life), "{choices:?}");
        assert_eq!(t.library_size(P0), 30 - hand);
    }
}

#[test]
fn sylvan_library_nothing_can_happen_between_drawing_and_choosing() {
    cr!("608.2c", "117.3b");
    ruling!(
        "Sylvan Library",
        "It’s not possible to take any actions between drawing the cards and choosing two cards. You can’t cast the cards you drew to avoid having two cards to choose."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, LIBRARY);
    t.lands(P0, "Mountain", 1);
    to_draw_step(&mut t);
    // The cards drawn could be cast (Lightning Bolts)...
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Lightning Bolt");
    library_answers(&mut t, P0, true, &[PUT_BACK, PUT_BACK]);
    let from = t.asked().len();
    t.g.resolve_top();
    // ... but the two cards are chosen in the same resolution as they're drawn, so nobody
    // gets priority in between: two of the three cards drawn this turn go back.
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn two_sylvan_libraries_resolve_one_at_a_time() {
    cr!("603.3b", "608.2");
    ruling!(
        "Sylvan Library",
        "If you control more than one Sylvan Library, the triggered ability of each will resolve in sequence. You don’t get to draw all the cards at once and then put them all back at once."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, LIBRARY);
    t.battlefield(P0, LIBRARY);
    library_answers(&mut t, P0, true, &[PUT_BACK, PUT_BACK]);
    library_answers(&mut t, P0, true, &[PUT_BACK, PAY]);
    to_draw_step(&mut t);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The first one is done: two cards drawn and put back.
    assert_eq!((t.hand_size(P0), t.life(P0), t.stack_len()), (1, 20, 1));
    t.resolve();
    // The second one draws those two again and chooses two cards drawn this turn.
    assert_eq!((t.hand_size(P0), t.life(P0), t.stack_len()), (2, 16, 0));
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn sylvan_library_and_other_draw_step_triggers() {
    cr!("101.4", "603.3b", "504.2");
    ruling!(
        "Sylvan Library",
        "If you control other triggered abilities that allow you to draw cards during your draw step, you can choose to order Sylvan Library’s ability before or after those abilities. Triggered abilities controlled by other players will resolve before triggered abilities you control."
    );
    supported("Howling Mine");
    // "At the beginning of each player's draw step, if Howling Mine is untapped, that player
    // draws an additional card." P1's Howling Mine: its trigger resolves first.
    let mut t = TestGame::new(2);
    t.battlefield(P0, LIBRARY);
    let mine = t.battlefield(P1, "Howling Mine");
    to_draw_step(&mut t);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(from_source(&t, mine), 1);
    assert_eq!(*t.g.stack.last().unwrap(), crate::r_s25_common::abilities_from(&t, mine)[0]);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    // Sylvan Library then chooses among three cards drawn this turn... and more.
    library_answers(&mut t, P0, true, &[PUT_BACK, PUT_BACK]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 20));
    // P0's own Howling Mine: P0 orders the two triggers.
    let mut t = TestGame::new(2);
    let library = t.battlefield(P0, LIBRARY);
    t.battlefield(P0, "Howling Mine");
    let from = t.asked().len();
    to_draw_step(&mut t);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    assert_eq!(from_source(&t, library), 1);
}

#[test]
fn sylvan_library_draws_replaced() {
    cr!("121.6", "121.2", "608.2c");
    ruling!(
        "Sylvan Library",
        "If you choose to draw two cards, then replace one or more of those draws with some other effect, the rest of Sylvan Library’s ability still happens. If you’ve actually drawn only one card that turn, you must choose that card and either pay 4 life or put it on top of your library. If you haven’t actually drawn any cards that turn, the rest of the ability has no effect."
    );
    supported("Underrealm Lich");
    // Underrealm Lich: "If you would draw a card, instead look at the top three cards of
    // your library, then put one into your hand and the rest into your graveyard." It
    // arrives after the normal draw: only that card was drawn this turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, LIBRARY);
    to_draw_step(&mut t);
    let drawn = t.g.player(P0).hand[0];
    t.battlefield(P0, "Underrealm Lich");
    library_answers(&mut t, P0, true, &[PUT_BACK]);
    let from = t.asked().len();
    t.resolve_all();
    let choices: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, prompt, .. } if prompt.contains("drawn") => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(choices, vec![vec![obj(drawn)]]);
    // Put back: on top of the library; the Lich's two cards stay in hand.
    assert_eq!(t.zone(drawn), Zone::Library(P0));
    assert_eq!(*t.g.player(P0).library.last().unwrap(), t.g.current(drawn));
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 20));
    // The Lich there all turn: nothing was drawn, the rest does nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, LIBRARY);
    t.battlefield(P0, "Underrealm Lich");
    library_answers(&mut t, P0, true, &[PAY, PAY]);
    to_draw_step(&mut t);
    t.resolve_all();
    t.clear_answers();
    assert_eq!((t.hand_size(P0), t.life(P0)), (3, 20));
}
