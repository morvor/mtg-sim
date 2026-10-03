//! Rulings batch S31 — a card exiled from an opponent's hand that its owner may play for as
//! long as it remains exiled, for a price (Elite Spellbinder, Invasion of Gobakhan,
//! Lightstall Inquisitor): the owner plays it with the normal timing rules (CR 305.1,
//! 305.2, 307.1), pays its costs plus the increase that comes with the permission (CR
//! 601.2f), and may play a modal double-faced card's land face (CR 712.11b).

use crate::r_s01_common::supported;
use crate::r_s02_common::can_play_land;
use crate::r_s21_common::castable;
use mtg_engine::card::card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BALA_GED: &str = "Bala Ged Recovery // Bala Ged Sanctuary";

/// P0's `name` ("When this creature enters, look at target opponent's hand. You may exile a
/// nonland card from it. For as long as that card remains exiled, its owner may play it. A
/// spell cast this way costs {2} more to cast.") enters and exiles P1's card `exiled`
/// (`extra` is another card in P1's hand). Returns the exiled card.
fn spellbinder_exiles(t: &mut TestGame, name: &str, exiled: &str, extra: &str) -> ObjectId {
    let card = t.hand(P1, exiled);
    t.hand(P1, extra);
    if name.starts_with("Invasion") {
        // The Siege's protector (CR 310.12a).
        t.answer_choose(P0, &[Entity::Player(P1)]);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.enter(P0, name);
    t.resolve_all();
    t.clear_answers();
    assert_eq!(t.zone(card), Zone::Exile, "{name} didn't exile {exiled}");
    card
}

#[test]
fn spellbinders_exiled_card_is_played_with_normal_timing_and_costs_two_more() {
    cr!("307.1", "601.2f", "601.3");
    ruling!(
        "Elite Spellbinder",
        "Playing the exiled card follows all normal timing restrictions."
    );
    ruling!(
        "Invasion of Gobakhan // Lightshield Array",
        "Playing the exiled card follows all normal timing restrictions."
    );
    supported("Elite Spellbinder");
    // (Only Lightshield Array's end step trigger isn't supported.)
    let invasion = card("Invasion of Gobakhan // Lightshield Array");
    assert!(invasion
        .unsupported_text()
        .iter()
        .all(|u| u.starts_with("At the beginning of your end step")));
    for name in ["Elite Spellbinder", "Invasion of Gobakhan // Lightshield Array"] {
        let mut t = TestGame::new(2);
        let bears = spellbinder_exiles(&mut t, name, "Grizzly Bears", "Forest");
        t.lands(P1, "Forest", 4);
        // A creature card: not during P0's turn.
        assert!(!castable(&mut t, P1, bears), "{name}: cast on P0's turn");
        // In P1's main phase with the stack empty: for {1}{G} plus {2}.
        t.set_step(P1, Step::PrecombatMain);
        assert!(castable(&mut t, P1, bears), "{name}: not castable");
        let c = t.g.current(bears);
        t.cast(P1, c).go();
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}: Bears didn't resolve");
        let untapped = t
            .g
            .battlefield
            .iter()
            .filter(|o| t.g.obj(**o).controller == P1 && !t.g.obj(**o).tapped)
            .count();
        // Only Grizzly Bears is untapped: all four Forests paid for it.
        assert_eq!(untapped, 1, "{name}: the spell didn't cost {{2}} more");
    }
}

#[test]
fn spellbinders_cost_increase_makes_the_card_unaffordable_with_its_own_cost() {
    cr!("601.2f", "117.1a");
    ruling!(
        "Elite Spellbinder",
        "Playing the exiled card follows all normal timing restrictions."
    );
    supported("Elite Spellbinder");
    let mut t = TestGame::new(2);
    let bears = spellbinder_exiles(&mut t, "Elite Spellbinder", "Grizzly Bears", "Forest");
    // {1}{G} isn't enough: it costs {2} more.
    t.lands(P1, "Forest", 3);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, bears));
    t.lands(P1, "Forest", 1);
    assert!(castable(&mut t, P1, bears));
    // An instant may be cast at instant speed, still for {2} more.
    let mut t = TestGame::new(2);
    let bolt = spellbinder_exiles(&mut t, "Elite Spellbinder", "Lightning Bolt", "Forest");
    t.lands(P1, "Mountain", 2);
    assert!(!castable(&mut t, P1, bolt));
    t.lands(P1, "Mountain", 1);
    // During P0's turn.
    assert!(castable(&mut t, P1, bolt));
    let c = t.g.current(bolt);
    t.cast(P1, c).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P1));
}

#[test]
fn spellbinders_exiled_mdfc_may_be_played_as_its_land_face() {
    cr!("712.11b", "305.1", "305.2");
    ruling!(
        "Elite Spellbinder",
        "If the exiled card is a modal double-faced card and its back face is a land, its owner may play it as a land."
    );
    supported(BALA_GED);
    let mut t = TestGame::new(2);
    let bala = spellbinder_exiles(&mut t, "Elite Spellbinder", BALA_GED, "Forest");
    // Not during P0's turn.
    assert!(!can_play_land(&mut t, P1, bala));
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P1, bala));
    let c = t.g.current(bala);
    t.play_land(P1, c).expect("play Bala Ged Sanctuary");
    assert_eq!(t.named_on_battlefield("Bala Ged Sanctuary").len(), 1);
    // It used P1's land play.
    assert_eq!(t.g.players[P1.idx()].lands_played_this_turn, 1);
}

#[test]
fn lightstall_inquisitors_cards_follow_timing_cost_more_and_lands_enter_tapped() {
    cr!("305.1", "305.2", "307.1", "601.2f", "614.1c");
    ruling!(
        "Lightstall Inquisitor",
        "Playing the exiled card follows all normal timing restrictions."
    );
    supported("Lightstall Inquisitor");
    // A spell: on its owner's own main phase, for {1} more.
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.enter(P0, "Lightstall Inquisitor");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.lands(P1, "Forest", 2);
    assert!(!castable(&mut t, P1, bears));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, bears), "it costs {{1}} more");
    t.lands(P1, "Forest", 1);
    assert!(castable(&mut t, P1, bears));
    // A land: with a land play, in its owner's main phase, and it enters tapped.
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.enter(P0, "Lightstall Inquisitor");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!can_play_land(&mut t, P1, forest));
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P1, forest));
    let c = t.g.current(forest);
    t.play_land(P1, c).expect("play the exiled Forest");
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped, "a land played this way enters tapped");
    // A land played from hand isn't affected.
    let other = t.hand(P1, "Forest");
    t.g.players[P1.idx()].lands_played_this_turn = 0;
    t.play_land(P1, other).expect("play a Forest");
    assert!(!t.obj_now(other).tapped);
}

/// P0 casts Soul Partition ("Exile target nonland permanent. For as long as that card
/// remains exiled, its owner may play it. A spell cast by an opponent this way costs {2}
/// more to cast.") on `target`. Returns the exiled card.
fn soul_partition(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.lands(P0, "Plains", 2);
    let sp = t.hand(P0, "Soul Partition");
    t.cast(P0, sp).target(target).go();
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile);
    target
}

#[test]
fn soul_partitions_card_follows_timing_and_only_an_opponent_pays_two_more() {
    cr!("307.1", "601.2f");
    ruling!(
        "Soul Partition",
        "Playing the exiled card follows all normal timing rules for a card of that type, and its owner must pay all costs (including the additional {2} if applicable)"
    );
    supported("Soul Partition");
    // An opponent's Grizzly Bears: in its owner's main phase, for {1}{G} plus {2}.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let bears = soul_partition(&mut t, theirs);
    t.lands(P1, "Forest", 3);
    assert!(!castable(&mut t, P1, bears), "not on P0's turn");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, bears), "it costs {{2}} more");
    t.lands(P1, "Forest", 1);
    assert!(castable(&mut t, P1, bears));
    // P0's own Grizzly Bears: P0 isn't an opponent, it costs only {1}{G}.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = soul_partition(&mut t, mine);
    t.lands(P0, "Forest", 2);
    assert!(castable(&mut t, P0, bears));
    let c = t.g.current(bears);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}
