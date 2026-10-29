//! Rulings batch P180 — permanents that put themselves into their owner's library: Blitz
//! Hellion, Inferno Hellion, Lightning Shrieker, Elixir of Immortality and Sensei's
//! Divining Top.

use crate::r_p180_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, give_control};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The players whose libraries were shuffled this turn, in order.
fn shuffled(t: &TestGame) -> Vec<PlayerId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Shuffled { player } => Some(*player),
            _ => None,
        })
        .collect()
}

#[test]
fn end_step_self_shuffles_leave_a_creature_that_left_where_it_is() {
    cr!("701.24c");
    // The ruling's "nothing happens ... its owner doesn't shuffle their library" yields to
    // CR 701.24c; only its "stays wherever it is" is followed.
    interp!("self-shuffle-gone-library-still-shuffled");
    ruling!("Blitz Hellion", "Blitz Hellion stays wherever it is");
    ruling!(
        "Inferno Hellion",
        "Inferno Hellion’s owner shuffles it into their library only if it’s on the battlefield as its last ability resolves."
    );
    ruling!(
        "Lightning Shrieker",
        "Lightning Shrieker’s owner shuffles it into their library only if it’s on the battlefield as its ability resolves."
    );
    for name in ["Blitz Hellion", "Inferno Hellion", "Lightning Shrieker"] {
        supported(name);
        for leave in [false, true] {
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::BeginningOfCombat);
            let c = t.battlefield(P0, name);
            if name == "Inferno Hellion" {
                t.attack(&[(c, Entity::Player(P1))], &[]);
            }
            t.advance_to(P0, Step::End);
            t.settle();
            assert_eq!(t.stack_len(), 1, "{name}");
            assert!(shuffled(&t).is_empty());
            if leave {
                move_to(&mut t, c, Zone::Hand(P0));
            }
            t.resolve_all();
            let c = t.g.current(c);
            if leave {
                assert_eq!(t.zone(c), Zone::Hand(P0), "{name}");
                assert_eq!(shuffled(&t), vec![P0], "{name}: CR 701.24c");
            } else {
                assert_eq!(t.zone(c), Zone::Library(P0), "{name}");
                assert_eq!(shuffled(&t), vec![P0], "{name}");
            }
        }
    }
}

#[test]
fn a_token_copy_of_blitz_hellion_is_shuffled_away_and_the_library_shuffled() {
    cr!("111.7", "111.8");
    ruling!(
        "Blitz Hellion",
        "A token that’s copying Blitz Hellion will be shuffled into its owner’s library when this ability resolves, then will cease to exist."
    );
    supported("Blitz Hellion");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    let hellion = t.graveyard(P0, "Blitz Hellion");
    let token = crate::r_s17_common::token_copy(&mut t, P0, hellion)[0];
    let lib = t.library_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.g.is_live(token) || !t.on_battlefield(token));
    assert!(t.named_on_battlefield("Blitz Hellion").is_empty());
    assert_eq!(t.library_size(P0), lib, "the token isn't in the library");
    assert_eq!(shuffled(&t), vec![P0], "the library is still shuffled");
}

#[test]
fn elixir_of_immortality_shuffles_itself_only_from_where_it_is_and_into_its_owners_library() {
    cr!("608.2c", "400.7", "701.24a");
    ruling!(
        "Elixir of Immortality",
        "If Elixir of Immortality is in your graveyard at the time the ability resolves, you’ll still wind up shuffling it into your library"
    );
    ruling!(
        "Elixir of Immortality",
        "If you gain control of another player’s Elixir of Immortality and activate it, the Elixir of Immortality will be shuffled into its owner’s library"
    );
    ruling!(
        "Elixir of Immortality",
        "Paying the activation cost of Elixir of Immortality’s ability doesn’t cause it to leave the battlefield."
    );
    supported("Elixir of Immortality");
    // Destroyed in response: it's in your graveyard, so it's shuffled in with it.
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P0, "Elixir of Immortality");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    destroy(&mut t, elixir);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
    // Returned to hand in response: it stays there; the graveyard is still shuffled in.
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P0, "Elixir of Immortality");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    move_to(&mut t, elixir, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Hand(P0));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
    // Another player's Elixir: it goes to its owner's library, your graveyard to yours;
    // both libraries are shuffled.
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P1, "Elixir of Immortality");
    give_control(&mut t, elixir, P0);
    let mine = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Library(P1));
    assert_eq!(t.zone(t.g.current(mine)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(theirs)), Zone::Graveyard(P1));
    let s = shuffled(&t);
    assert!(s.contains(&P0) && s.contains(&P1), "{s:?}");
    // Another player's Elixir in that player's graveyard by resolution: it stays.
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P1, "Elixir of Immortality");
    give_control(&mut t, elixir, P0);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    destroy(&mut t, elixir);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Graveyard(P1));
    // Paying the cost leaves it on the battlefield: untapped, it can be activated again
    // in response to itself.
    let mut t = TestGame::new(2);
    let elixir = t.battlefield(P0, "Elixir of Immortality");
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    assert!(t.on_battlefield(elixir));
    t.g.untap(elixir);
    activate_containing(&mut t, P0, elixir, "gain 5 life").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 30);
    assert_eq!(t.zone(t.g.current(elixir)), Zone::Library(P0));
}

#[test]
fn senseis_divining_top_draw_ability_with_the_top_gone_or_in_response() {
    cr!("113.7a");
    ruling!(
        "Sensei's Divining Top",
        "If Sensei's Divining Top leaves the battlefield while its second ability is on the stack, you'll draw a card and leave Sensei's Divining Top in the zone it's moved to."
    );
    ruling!(
        "Sensei's Divining Top",
        "Sensei's Divining Top's second ability may be activated in response to its first ability."
    );
    supported("Sensei's Divining Top");
    // The Top is destroyed in response to its draw ability.
    let mut t = TestGame::new(2);
    let top = t.battlefield(P0, "Sensei's Divining Top");
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, top, "Draw a card").unwrap();
    destroy(&mut t, top);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(t.g.current(top)), Zone::Graveyard(P0));
    // The draw ability in response to the look ability: draw, the Top goes on top, then
    // the Top is among the three cards looked at.
    let mut t = TestGame::new(2);
    let top = t.battlefield(P0, "Sensei's Divining Top");
    t.lands(P0, "Wastes", 1);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, top, "Look at the top three").unwrap();
    activate_containing(&mut t, P0, top, "Draw a card").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    let card = t.g.current(top);
    assert_eq!(t.g.library_top(P0), Some(card));
    let from = t.asked().len();
    t.resolve_all();
    let seen: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::Order { items, .. } => Some(items.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        seen.iter().any(|s| s.contains("Sensei's Divining Top")),
        "{seen:?}"
    );
    let lib = &t.g.player(P0).library;
    assert!(lib.iter().rev().take(3).any(|c| *c == card) || lib.iter().take(3).any(|c| *c == card));
}
