//! Rulings batch S32 — cards returned or cast from graveyards: an escaped permanent is an
//! ordinary new object (CR 400.7), abilities that return a card do nothing once it has
//! left the graveyard (CR 400.7, 608.2b), "dies" means put into a graveyard (CR 700.4),
//! the active player gets priority first after a spell resolves (CR 117.3b), the player
//! picks which permission to cast a card from the graveyard with (CR 601.3), and players
//! who left a multiplayer game aren't counted (CR 800.4a).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s07_common::graveyard_n;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

#[test]
fn an_escaped_permanent_goes_to_the_graveyard_when_it_dies_later() {
    cr!("702.138a", "400.7");
    ruling!(
        "Confession Dial",
        "After an escaped permanent spell resolves, it enters the battlefield and will return to its owner's graveyard if it dies later."
    );
    supported("Confession Dial");
    // "{T}: Target legendary creature card in your graveyard gains escape until end of
    // turn. The escape cost is equal to its mana cost plus exile three other cards from
    // your graveyard."
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Confession Dial");
    let isamaru = t.graveyard(P0, "Isamaru, Hound of Konda");
    for _ in 0..3 {
        t.graveyard(P0, "Shock");
    }
    t.activate(P0, dial, 0, &[Entity::Object(isamaru)])
        .expect("activate Confession Dial");
    t.resolve_all();
    t.lands(P0, "Plains", 1);
    t.cast(P0, isamaru)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .go();
    t.resolve_all();
    let isamaru = t.g.current(isamaru);
    assert!(t.on_battlefield(isamaru));
    assert_eq!(t.graveyard_size(P0), 0, "three cards exiled for escape");
    destroy(&mut t, isamaru);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
}

#[test]
fn a_persist_card_removed_from_the_graveyard_isnt_returned() {
    cr!("702.79a", "400.7", "608.2b");
    ruling!(
        "Obstinate Gargoyle",
        "If a card with persist is removed from the graveyard after it dies but before the triggered ability resolves, it won't be returned to the battlefield."
    );
    supported("Obstinate Gargoyle");
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P0, "Obstinate Gargoyle");
    destroy(&mut t, gargoyle);
    assert_eq!(t.stack_len(), 1, "persist triggered");
    let card = t.g.current(gargoyle);
    cast_new(&mut t, P1, "Cremate", &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Obstinate Gargoyle").is_empty());
    assert!(t.in_exile("Obstinate Gargoyle"));
}

#[test]
fn the_locust_god_returns_at_the_next_end_step_of_the_same_turn() {
    cr!("603.7a", "603.7b", "513.1a");
    ruling!(
        "The Locust God",
        "The “next end step” refers to the next end step that occurs, not the end step of the next turn. If this creature dies before a turn's end step (for example, during combat), it will be returned to its owner's hand at the beginning of that turn's end step."
    );
    supported("The Locust God");
    // "When The Locust God dies, return it to its owner's hand at the beginning of the
    // next end step." It dies in P1's turn (P0 owns it).
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "The Locust God");
    t.set_step(P1, Step::PrecombatMain);
    let turn = t.g.turn.number;
    destroy(&mut t, god);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "The Locust God"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.g.turn.number, turn);
    assert!(t.in_hand(P0, "The Locust God"));
}

#[test]
fn the_locust_god_stays_in_its_new_zone_if_it_left_the_graveyard() {
    cr!("400.7", "603.7c");
    ruling!(
        "The Locust God",
        "If this creature dies but leaves your graveyard before the next end step, it will remain in its new zone."
    );
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "The Locust God");
    destroy(&mut t, god);
    t.resolve_all();
    let card = t.g.current(god);
    cast_new(&mut t, P1, "Cremate", &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.in_exile("The Locust God"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("The Locust God"));
    assert!(!t.in_hand(P0, "The Locust God"));
}

#[test]
fn exiled_instead_of_dying_dies_triggers_dont_trigger() {
    cr!("700.4", "614.1a", "603.6c");
    ruling!(
        "Stone of Erech",
        "Because creatures controlled by opponents aren't being put into a graveyard, any \"when [this creature] dies\" triggered abilities those creatures have won't trigger."
    );
    supported("Stone of Erech");
    supported("Misery's Shadow");
    // "If a creature an opponent controls would die, exile it instead." Doomed Traveler:
    // "When this creature dies, create a 1/1 white Spirit creature token with flying."
    for stone in ["Stone of Erech", "Misery's Shadow"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, stone);
        let traveler = t.battlefield(P1, "Doomed Traveler");
        destroy(&mut t, traveler);
        assert_eq!(t.stack_len(), 0, "{stone}");
        t.resolve_all();
        assert!(t.in_exile("Doomed Traveler"));
        assert!(tokens(&t, P1).is_empty());
        // P0's own Doomed Traveler still dies.
        let mine = t.battlefield(P0, "Doomed Traveler");
        destroy(&mut t, mine);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Doomed Traveler"));
        assert_eq!(tokens(&t, P0).len(), 1, "{stone}");
    }
}

#[test]
fn after_a_spell_resolves_in_your_main_phase_you_get_priority_first() {
    cr!("117.3b", "117.3a", "601.3");
    ruling!(
        "Lurrus of the Dream-Den",
        "If a permanent card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    supported("Lurrus of the Dream-Den");
    // P0 Shocks their own Grizzly Bears; once Shock has resolved, P0 is the first to get
    // priority and casts the Bears from the graveyard with Lurrus's permission.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurrus of the Dream-Den");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(bears).go();
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty() && g.turn.stage == Stage::Priority && g.turn.priority.is_some()
    });
    assert!(ok);
    // State-based actions are checked as P0 would receive priority: the Bears die.
    t.settle();
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    assert_eq!(t.g.turn.priority, Some(P0));
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.lands(P0, "Forest", 2);
    assert_eq!(legal_cast_methods(&mut t, P0, card), vec![CastMethod::Normal]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn you_choose_which_permission_you_cast_a_graveyard_card_with() {
    cr!("601.3");
    ruling!(
        "Karador, Ghost Chieftain",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    supported("Karador, Ghost Chieftain");
    supported("Lurrus of the Dream-Den");
    // Karador (a creature spell) and Lurrus (a permanent spell with mana value 2 or less)
    // both let P0 cast Grizzly Bears from the graveyard. Using Lurrus's leaves Karador's
    // for Hill Giant; using Karador's leaves Lurrus's, which doesn't allow Hill Giant.
    for (use_lurrus, giant_castable) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let karador = t.battlefield(P0, "Karador, Ghost Chieftain");
        let lurrus = t.battlefield(P0, "Lurrus of the Dream-Den");
        let bears = t.graveyard(P0, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        t.lands(P0, "Forest", 5);
        t.lands(P0, "Mountain", 1);
        let permission = if use_lurrus { lurrus } else { karador };
        t.answer_choose(P0, &[Entity::Object(permission)]);
        t.cast(P0, bears).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
        assert_eq!(
            !legal_cast_methods(&mut t, P0, giant).is_empty(),
            giant_castable,
            "used Lurrus's: {use_lurrus}"
        );
    }
}

#[test]
fn an_opponent_who_left_the_game_isnt_considered() {
    cr!("800.4a", "800.4");
    ruling!(
        "Nimana Skitter-Sneak",
        "In a multiplayer game, once an opponent leaves the game, they won’t be considered for such an effect, no matter how many cards they had in their graveyard before leaving the game."
    );
    ruling!(
        "Mind Carver",
        "In a multiplayer game, once an opponent leaves the game, they won't be considered for such an effect, no matter how many cards they had in their graveyard before leaving the game."
    );
    supported("Nimana Skitter-Sneak");
    supported("Mind Carver");
    // Three players: P1 has eight cards in their graveyard, P2 has none.
    let mut t = TestGame::new(3);
    let sneak = t.battlefield(P0, "Nimana Skitter-Sneak");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let carver = t.battlefield(P0, "Mind Carver");
    t.attach(carver, Entity::Object(bears));
    graveyard_n(&mut t, P1, "Island", 8);
    t.settle();
    assert_eq!(t.pt(sneak), (4, 4));
    assert_eq!(t.pt(bears), (5, 3));
    // P1 concedes: the bonuses are gone.
    t.g.take_action(P1, Action::Concede);
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P1));
    assert_eq!(t.pt(sneak), (3, 4));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn several_opponents_with_eight_cards_in_their_graveyards_give_no_extra_bonus() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Nimana Skitter-Sneak",
        "An ability that offers a bonus if an opponent has eight or more cards in their graveyard won’t provide additional benefits if more than one opponent has eight or more cards in their graveyard. There are also no additional benefits no matter how many cards are in an opponent’s graveyard, as long as there are at least eight."
    );
    supported("Soaring Thought-Thief");
    // Nimana Skitter-Sneak: "As long as an opponent has eight or more cards in their
    // graveyard, this creature gets +1/+0 and has menace." Soaring Thought-Thief: "...
    // Rogues you control get +1/+0."
    let mut t = TestGame::new(3);
    let sneak = t.battlefield(P0, "Nimana Skitter-Sneak");
    let thief = t.battlefield(P1, "Soaring Thought-Thief");
    graveyard_n(&mut t, P1, "Island", 7);
    t.settle();
    assert_eq!(t.pt(sneak), (3, 4));
    graveyard_n(&mut t, P1, "Island", 13);
    graveyard_n(&mut t, P2, "Island", 8);
    graveyard_n(&mut t, P0, "Island", 8);
    t.settle();
    // P1 has twenty and P2 eight: +1/+0 once for the Skitter-Sneak (a Rogue) ...
    assert_eq!(t.pt(sneak), (4, 4));
    // ... and once for P1's Thought-Thief, whose opponents P0 and P2 both have eight.
    assert_eq!(t.pt(thief), (2, 3));
    assert!(t.obj(sneak).has_keyword(mtg_engine::keywords::KeywordKind::Menace));
}
