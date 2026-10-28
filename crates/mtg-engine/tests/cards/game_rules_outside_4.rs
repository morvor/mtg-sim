//! Cards from outside the game, round 3 (`oracle/patterns/game_rules_4.rs`): a permission
//! to play one card you own from outside the game this turn, casting any number of spells
//! from outside the game, and searches that include outside the game (CR 400.11b).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn wish_lets_you_play_one_card_from_outside_the_game_this_turn() {
    cr!("400.11b", "108.3b");
    interp!("outside-the-game-is-the-sideboard");
    ruling!(
        "Wish",
        "You choose the card to play from outside the game at the time you are playing it, not at the time Wish resolves."
    );
    ruling!(
        "Wish",
        "You still need to pay all costs to play a card this way, and you must follow all normal timing rules."
    );
    assert_supported(&["Wish"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let side =
        t.g.add_to_sideboard(P0, vec![card("Lightning Bolt"), card("Shock")]);
    t.lands(P0, "Mountain", 5);
    let wish = t.hand(P0, "Wish");
    t.answer_yes(P0, true);
    t.cast(P0, wish).go();
    t.resolve_all();
    // One of them, with its costs paid.
    t.cast(P0, side[0]).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.cast(P0, side[1]).target(P1).try_go().is_err());
    // Not on a later turn.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let side = t.g.add_to_sideboard(P0, vec![card("Grizzly Bears")]);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    let wish = t.hand(P0, "Wish");
    t.answer_yes(P0, true);
    t.cast(P0, wish).go();
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.cast(P0, side[0]).try_go().is_err());
}

#[test]
fn spawnsire_of_ulamog_casts_eldrazi_from_outside_the_game() {
    cr!("400.11b", "608.2g");
    interp!("outside-the-game-is-the-sideboard");
    assert_supported(&["Spawnsire of Ulamog"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spawnsire = t.battlefield(P0, "Spawnsire of Ulamog");
    let side = t.g.add_to_sideboard(
        P0,
        vec![
            card("Eldrazi Devastator"),
            card("Ulamog's Crusher"),
            card("Grizzly Bears"),
        ],
    );
    t.lands(P0, "Wastes", 20);
    t.answer_choose(P0, &[Entity::Object(side[0]), Entity::Object(side[1])]);
    t.activate(P0, spawnsire, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Eldrazi Devastator").len(), 1);
    assert_eq!(t.named_on_battlefield("Ulamog's Crusher").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn invasion_of_arcavios_finds_a_card_outside_the_game() {
    cr!("400.11b", "701.23a");
    interp!("outside-the-game-is-the-sideboard");
    assert_supported(&["Invasion of Arcavios // Invocation of the Founders"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let side = t.g.add_to_sideboard(P0, vec![card("Counterspell")]);
    t.library_top(P0, "Lightning Bolt");
    // (Invasion of Arcavios is a battle: its protector is chosen as it enters.)
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.enter(P0, "Invasion of Arcavios // Invocation of the Founders");
    t.resolve_all();
    assert!(t.in_hand(P0, "Counterspell"));
    assert!(!t.in_hand(P0, "Lightning Bolt"));
}
