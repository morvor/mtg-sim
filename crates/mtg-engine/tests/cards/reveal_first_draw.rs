//! "Reveal the first card you draw each turn. Whenever you reveal a [card] this way, ..."
//! (`oracle/patterns/draw_reveal_first.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) {
    t.g.players[p.0 as usize].library.clear();
    for n in names {
        t.library_top(p, n);
    }
}

#[test]
fn primitive_etchings_draws_when_the_first_card_drawn_is_a_creature() {
    cr!("121.1", "603.2");
    ruling!(
        "Primitive Etchings",
        "If you have more than one of these cards on the battlefield, only one card is revealed"
    );
    assert_supported("Primitive Etchings");
    assert_supported("Rowen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primitive Etchings");
    t.battlefield(P0, "Primitive Etchings");
    library(&mut t, P0, &["Island", "Forest", "Shock", "Grizzly Bears"]);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    // Both trigger: two more cards. They aren't the first card drawn this turn.
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), 1);

    // The first card isn't a creature: nothing; later draws don't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primitive Etchings");
    library(&mut t, P0, &["Island", "Grizzly Bears", "Shock"]);
    t.g.draw_cards(P0, 2);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn keranos_draws_for_a_land_and_deals_damage_for_a_nonland_on_your_turns() {
    cr!("121.1", "603.2");
    ruling!(
        "Keranos, God of Storms",
        "If you reveal a land card this way, Keranos's triggered ability will cause you to draw another card"
    );
    assert_supported("Keranos, God of Storms");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keranos, God of Storms");
    library(&mut t, P0, &["Shock", "Island"]);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keranos, God of Storms");
    library(&mut t, P0, &["Island", "Shock"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 1);

    // Not on an opponent's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keranos, God of Storms");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    library(&mut t, P0, &["Shock", "Island"]);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn inquisitor_eisenhorn_may_reveal_an_instant_or_sorcery() {
    cr!("121.1", "603.2", "111.4");
    ruling!(
        "Inquisitor Eisenhorn",
        "You don't have to reveal a drawn card if you don't wish to create a Cherubael token"
    );
    ruling!(
        "Inquisitor Eisenhorn",
        "You can reveal an instant or sorcery card this way on any turn"
    );
    assert_supported("Inquisitor Eisenhorn");
    for (on_p0_turn, reveal) in [(true, true), (false, true), (true, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Inquisitor Eisenhorn");
        if !on_p0_turn {
            t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        }
        library(&mut t, P0, &["Island", "Shock"]);
        t.answer_yes(P0, reveal);
        t.g.draw_cards(P0, 1);
        t.settle();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Cherubael").len(),
            usize::from(reveal),
            "{on_p0_turn} {reveal}"
        );
    }
}
