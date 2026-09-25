//! Impulse draw: "Exile the top card of your library. You may play that card this turn.",
//! "Until the end of your next turn, you may play those cards." The exiled cards can be
//! played (lands too) with the normal timing and costs, only while the permission lasts.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn light_up_the_stage_lasts_until_the_end_of_your_next_turn() {
    cr!("611.2a");
    ruling!(
        "Light Up the Stage",
        "If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    assert_supported("Light Up the Stage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let mountain = t.library_top(P0, "Mountain");
    let spell = t.hand(P0, "Light Up the Stage");
    t.cast(P0, spell).go();
    t.resolve();
    let bolt_x = t.g.current(bolt);
    let mountain_x = t.g.current(mountain);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert_eq!(t.zone(mountain_x), Zone::Exile);
    // The land can be played from exile (it uses the land play).
    t.play_land(P0, mountain_x).expect("play the exiled land");
    assert_eq!(t.named_on_battlefield("Mountain").len(), 4);
    // The Bolt is still playable on P0's next turn...
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let bolt_x = t.g.current(bolt);
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_ok());
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn light_up_the_stage_ends_after_your_next_turn() {
    cr!("611.2a");
    ruling!(
        "Light Up the Stage",
        "If you don't play a card exiled this way, it remains in exile."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Light Up the Stage");
    t.cast(P0, spell).go();
    t.resolve();
    // P0's next turn passes, then on the turn after that the Bolt can't be cast.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_err());
}

#[test]
fn abbot_of_keral_keep_only_until_end_of_turn() {
    cr!("611.2a");
    ruling!(
        "Abbot of Keral Keep",
        "You may play that card that turn even if Abbot of Keral Keep is no longer on the battlefield"
    );
    assert_supported("Abbot of Keral Keep");
    // Cast this turn, with the Abbot gone: allowed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let abbot = t.enter(P0, "Abbot of Keral Keep");
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    t.g.destroy(abbot, None);
    t.settle();
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_ok());
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Not cast this turn: the permission ends.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.enter(P0, "Abbot of Keral Keep");
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_err());
}

#[test]
fn chandra_fire_artisan_exiles_the_top_card_to_play_this_turn() {
    cr!("611.2a");
    ruling!(
        "Chandra, Fire Artisan",
        "Casting an exiled card causes it to leave exile. You can't cast it multiple times."
    );
    assert!(card("Chandra, Fire Artisan")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("Exile the top")));
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let chandra = t.battlefield(P0, "Chandra, Fire Artisan");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert!(t.cast_with(P0, bolt_x, &[Entity::Player(P1)]).is_ok());
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
