//! Life totals and the starting life total (CR 119.1, 119.5), and the Courts' "instead"
//! sentences that refer back to their targets ("each of those players", "that permanent
//! card"), and "Draw another card if [condition]".

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
fn oketras_last_mercy_resets_your_life_to_the_starting_total() {
    cr!("119.1", "119.5");
    assert_supported(&["Oketra's Last Mercy"]);
    let mut t = TestGame::new(2);
    t.g.players[0].life = 3;
    t.lands(P0, "Plains", 3);
    let o = t.hand(P0, "Oketra's Last Mercy");
    t.cast(P0, o).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn the_endstone_halves_the_starting_life_total_rounded_up() {
    cr!("119.1", "119.5", "107.1a");
    assert_supported(&["The Endstone"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Endstone");
    t.g.players[0].life = 35;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
}

#[test]
fn torgaar_sets_a_players_life_to_half_their_starting_total() {
    cr!("119.1", "119.5");
    assert_supported(&["Torgaar, Famine Incarnate"]);
    let mut t = TestGame::new(2);
    t.g.players[1].life = 40;
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Torgaar, Famine Incarnate");
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn resolute_archangel_restores_a_lower_life_total_only() {
    cr!("119.1", "603.4");
    assert_supported(&["Resolute Archangel"]);
    let mut t = TestGame::new(2);
    t.g.players[0].life = 7;
    t.enter(P0, "Resolute Archangel");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Above the starting total: the ability doesn't trigger.
    t.g.players[0].life = 30;
    t.enter(P0, "Resolute Archangel");
    t.resolve_all();
    assert_eq!(t.life(P0), 30);
}

#[test]
fn elenda_grows_with_life_above_the_starting_total() {
    cr!("119.1");
    assert_supported(&["Elenda, Saint of Dusk"]);
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Elenda, Saint of Dusk");
    assert_eq!(t.pt(e), (4, 4));
    t.g.players[0].life = 21;
    t.g.recompute();
    assert_eq!(t.pt(e), (5, 5));
    t.g.players[0].life = 30;
    t.g.recompute();
    assert_eq!(t.pt(e), (10, 10));
}

#[test]
fn exquisite_archangel_replaces_losing_the_game() {
    cr!("104.3b", "614.1a", "119.1");
    assert_supported(&["Exquisite Archangel"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Exquisite Archangel");
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(a));
    assert!(t.in_exile("Exquisite Archangel"));
}

#[test]
fn court_of_cunning_mills_ten_for_the_monarch() {
    cr!("725.1", "608.2c");
    assert_supported(&["Court of Cunning"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Cunning");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.monarch = Some(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 10);
    // Not the monarch: two.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Cunning");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.monarch = Some(P1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn court_of_ardenvale_returns_to_the_battlefield_for_the_monarch() {
    cr!("725.1", "608.2c");
    assert_supported(&["Court of Ardenvale"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Ardenvale");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.monarch = Some(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Not the monarch: to the hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Ardenvale");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.monarch = Some(P1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn imoen_draws_another_card_after_completing_a_dungeon() {
    cr!("726.1");
    assert_supported(&["Imoen, Mystic Trickster"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imoen, Mystic Trickster");
    t.g.initiative = Some(P0);
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
    // After completing a dungeon: another card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imoen, Mystic Trickster");
    t.g.initiative = Some(P0);
    t.g.players[0].dungeons_completed = 1;
    t.g.players[0].completed_dungeons.push("Lost Mine of Phandelver".into());
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 2);
}
