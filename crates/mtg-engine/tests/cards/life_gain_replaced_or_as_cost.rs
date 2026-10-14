//! Life gain replaced with another event ("If an opponent would gain life, that player
//! loses that much life instead."), one-use draw replacements ("The next time you would
//! draw a card this turn, [effect] instead."), and alternative costs that have other
//! players gain life (CR 614.1a, 118.9, 119.7).

use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// P1 gains `n` life from an effect P1 controls.
fn p1_gains(t: &mut TestGame, n: i32) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    t.g.exec(
        &mtg_engine::ability::Effect::GainLife {
            who: mtg_engine::ability::PlayerRef::You,
            n: mtg_engine::ability::Value::Const(n),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn tainted_remedy_opponents_lose_life_instead_of_gaining_it() {
    cr!("614.1a", "119.3");
    compiles("Tainted Remedy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Remedy");
    p1_gains(&mut t, 4);
    assert_eq!(t.life(P1), 16);
    // P0's own life gain isn't replaced.
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P0), 23);
}

/// Activates the Words enchantment `name` ({1}) for P0, then has P0 draw a card.
fn activate_words_then_draw(t: &mut TestGame, name: &str) {
    compiles(name);
    let words = t.battlefield(P0, name);
    t.lands(P0, "Plains", 1);
    t.activate(P0, words, 0, &[]).unwrap();
    t.resolve_all();
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
}

#[test]
fn words_of_wilding_creates_a_bear_instead_of_one_draw() {
    cr!("614.1a", "121.6");
    let mut t = TestGame::new(2);
    let library = t.library_size(P0);
    activate_words_then_draw(&mut t, "Words of Wilding");
    assert_eq!(t.library_size(P0), library);
    assert_eq!(t.hand_size(P0), 0);
    let bears: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Bear"))
        .map(|o| o.id)
        .collect();
    assert_eq!(bears.len(), 1);
    // The effect is used up: the next draw is a normal one.
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn words_of_waste_each_opponent_discards_instead() {
    cr!("614.1a", "121.6");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    activate_words_then_draw(&mut t, "Words of Waste");
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn skyshroud_cutter_each_other_player_gains_life_as_its_cost() {
    cr!("118.9", "601.2h");
    compiles("Skyshroud Cutter");
    let mut t = TestGame::new(2);
    let cutter = t.hand(P0, "Skyshroud Cutter");
    let alt = |t: &mut TestGame| {
        t.g.turn.priority = Some(P0);
        t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, Action::Cast { card, method: CastMethod::Alternative(_) } if *card == cutter)
        })
    };
    // Only with a Forest.
    assert!(!alt(&mut t));
    t.lands(P0, "Forest", 1);
    assert!(alt(&mut t));
    let method = t
        .cast_options(P0, cutter)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .unwrap();
    t.cast(P0, cutter).method(method).go();
    assert_eq!(t.life(P1), 25);
    t.resolve_all();
    assert!(t.on_battlefield(cutter));
}
