//! roundtrip-gaps-4 (docs/agents/items/roundtrip-gaps-4.md): the parser bugs behind the
//! renderer gap clusters (named behaviors, same-name searches, delayed triggers with an
//! intervening "if"), and behaviors checked in play because their rendering changed.

use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

#[test]
fn lobotomy_searches_and_shuffles_with_no_card_chosen() {
    cr!("701.23a", "701.24a");
    ruling!(
        "Lobotomy",
        "If the player has no cards in their hand, you still get to look through their deck"
    );
    // "Target player reveals their hand, then you choose a card other than a basic land
    // card from it. Search that player's graveyard, hand, and library for all cards with
    // the same name as the chosen card and exile them. Then that player shuffles."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let l = t.hand(P0, "Lobotomy");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, l).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(shuffles(&t, P1), 1, "{}", t.dump_log());
}

#[test]
fn shimian_specter_shuffles_when_the_hand_has_no_nonland_card() {
    cr!("701.23a", "701.24a");
    ruling!(
        "Shimian Specter",
        "If the player has no nonland cards in their hand, you can still search that player"
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shimian Specter");
    t.hand(P1, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(shuffles(&t, P1), 1, "{}", t.dump_log());
}

#[test]
fn shimian_specter_exiles_the_chosen_cards_name() {
    cr!("701.23a", "201.2");
    // Regression: with a nonland card chosen, every card with its name in that player's
    // graveyard, hand, and library is exiled.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shimian Specter");
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.in_hand(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert!(!t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    let bears_in_library = t
        .g
        .player(P1)
        .library
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == "Grizzly Bears")
        .count();
    assert_eq!(bears_in_library, 0, "{}", t.dump_log());
}

#[test]
fn shirei_returns_nothing_after_its_next_end_step_has_passed() {
    cr!("603.7b", "603.4");
    // "Whenever a creature with power 1 or less is put into your graveyard from the
    // battlefield, you may return that card to the battlefield at the beginning of the
    // next end step if Shirei is still on the battlefield." Shirei phased out at that end
    // step (treated as though it doesn't exist, CR 702.26b): the delayed ability doesn't
    // trigger then, and doesn't wait for a later end step once Shirei phases in.
    let mut t = TestGame::new(2);
    let shirei = t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    assert!(t.g.destroy(elves, None));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![shirei]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"), "{}", t.dump_log());
    mtg_engine::keyword_impls::phase_in(&mut t.g, shirei);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(
        t.in_graveyard(P0, "Llanowar Elves"),
        "returned at a later end step: {}",
        t.dump_log()
    );
}

#[test]
fn shirei_returns_the_card_at_the_next_end_step() {
    cr!("603.7a");
    // Regression: with Shirei on the battlefield, the card comes back.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    assert!(t.g.destroy(elves, None));
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Llanowar Elves").len(),
        1,
        "{}",
        t.dump_log()
    );
}

#[test]
fn plague_reaver_returns_under_the_chosen_opponents_control() {
    cr!("603.7c");
    // "Choose target opponent. Return ~ to the battlefield under that player's control at
    // the beginning of their next upkeep." (the delayed ability refers to the target of
    // the ability that created it).
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Plague Reaver");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, r, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plague Reaver"));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let on = t.named_on_battlefield("Plague Reaver");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).controller, P1);
}

#[test]
fn balancing_act_sacrifices_at_the_same_time() {
    cr!("101.4", "603.10a");
    // "Each player chooses a number of permanents they control equal to the number of
    // permanents controlled by the player who controls the fewest, then sacrifices the
    // rest." The permanents are sacrificed together: Blood Artist sees the other player's
    // creature die with it.
    let mut t = TestGame::new(3);
    let artist = t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Island");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Forest");
    t.battlefield(P2, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(artist)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let b = t.hand(P0, "Balancing Act");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::W, 4);
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blood Artist"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22, "{}", t.dump_log());
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}
