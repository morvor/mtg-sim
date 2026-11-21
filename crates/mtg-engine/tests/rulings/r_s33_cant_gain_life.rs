//! Rulings batch S33 — "players can't gain life" (CR 119.7): spells and abilities that
//! would cause a player to gain life still resolve, but the life-gain part has no
//! effect; likewise, while a player's life total can't change, the life-gain and
//! life-loss parts do nothing (CR 119.7, 119.8).

use crate::r_s01_common::supported;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// With `name` on P1's battlefield, P0 casts Lightning Helix ("Lightning Helix deals 3
/// damage to any target and you gain 3 life.") at P1: the damage is dealt, the spell
/// resolves, and P0 gains no life.
fn helix_under(name: &str) {
    supported(name);
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    t.battlefield(P1, name);
    set_life(&mut t, P0, 10);
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 10);
    assert!(life_gains_since(&t, from, P0).is_empty());
    // Without it, P0 gains 3.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    cast_and_resolve(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    assert_eq!((t.life(P0), t.life(P1)), (13, 17));
}

#[test]
fn leyline_of_punishment_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Leyline of Punishment",
        "Spells and abilities that would normally cause a player to gain life still resolve, but the life-gain part simply has no effect."
    );
    helix_under("Leyline of Punishment");
}

#[test]
fn everlasting_torment_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Everlasting Torment",
        "Spells and abilities that would normally cause a player to gain life still resolve, but the life-gain part simply has no effect."
    );
    helix_under("Everlasting Torment");
}

#[test]
fn havoc_festival_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Havoc Festival",
        "Spells and abilities that would cause a player to gain life still resolve, but the life-gain part has no effect."
    );
    helix_under("Havoc Festival");
}

#[test]
fn forsaken_wastes_replacing_life_gain_with_another_effect_does_nothing() {
    cr!("119.7", "614.17c", "614.1a");
    ruling!(
        "Forsaken Wastes",
        "Effects that would replace gaining life with some other effect won't be able to do anything because it's impossible for players to gain life."
    );
    supported("Forsaken Wastes");
    supported("Plague Drone");
    // P0's Plague Drone: "If an opponent would gain life, that player loses that much life
    // instead." P1 casts Lightning Helix at P0.
    let helix = |wastes: bool| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Plague Drone");
        if wastes {
            t.battlefield(P0, "Forsaken Wastes");
        }
        set_life(&mut t, P1, 10);
        cast_and_resolve(&mut t, P1, "Lightning Helix", &[Entity::Player(P0)]);
        (t.life(P0), t.life(P1))
    };
    // Players can't gain life: no life-gain event, so nothing to replace with life loss.
    assert_eq!(helix(true), (17, 10));
    // Otherwise P1 loses 3 life instead of gaining it.
    assert_eq!(helix(false), (17, 7));
}

/// P0 controls Words of Worship ("{1}: The next time you would draw a card this turn, you
/// gain 5 life instead.") and activates it, then draws a card. Returns (life gained,
/// cards drawn).
fn words_of_worship_then_draw(t: &mut TestGame) -> (i32, usize) {
    supported("Words of Worship");
    let words = t.battlefield(P0, "Words of Worship");
    t.lands(P0, "Plains", 1);
    t.activate(P0, words, 0, &[]).unwrap();
    t.resolve_all();
    let (life, hand, library) = (t.life(P0), t.hand_size(P0), t.library_size(P0));
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P0) - hand, library - t.library_size(P0));
    (t.life(P0) - life, t.hand_size(P0) - hand)
}

#[test]
fn leyline_of_punishment_replacing_an_event_with_life_gain_replaces_it_with_nothing() {
    cr!("119.7", "614.1a", "121.6");
    ruling!(
        "Leyline of Punishment",
        "Effects that replace an event with gaining life (like Words of Worship's effect does) will end up replacing the event with nothing."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Punishment");
    // The draw is replaced, and P0 gains no life.
    assert_eq!(words_of_worship_then_draw(&mut t), (0, 0));
    // Without Leyline of Punishment: 5 life instead of the card.
    let mut t = TestGame::new(2);
    assert_eq!(words_of_worship_then_draw(&mut t), (5, 0));
    // Only the next draw.
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), 1);
}

/// The alternative-cost ways of casting the card offered now.
fn alternatives(t: &TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.cast_options(p, card)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect()
}

/// Whether `p` may begin casting the card with its alternative cost now (its costs can
/// be paid).
fn alt_castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok = t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Cast { card: c, method: CastMethod::Alternative(_) } if *c == card)
    });
    t.g.turn.priority = saved;
    ok
}

#[test]
fn everlasting_torment_a_cost_that_includes_life_gain_cant_be_paid() {
    cr!("119.7", "118.9", "601.2f");
    ruling!(
        "Everlasting Torment",
        "If a cost includes life gain (like Invigorate's alternative cost does), that cost can't be paid."
    );
    supported("Invigorate");
    // Invigorate: "If you control a Forest, rather than pay this spell's mana cost, you
    // may have an opponent gain 3 life. Target creature gets +4/+4 until end of turn."
    let setup = |t: &mut TestGame| {
        t.lands(P0, "Forest", 1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        (t.hand(P0, "Invigorate"), bears)
    };
    // Players can't gain life: only the mana cost (which P0 can't pay with one Forest).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Everlasting Torment");
    let (invigorate, _) = setup(&mut t);
    assert!(!alt_castable(&mut t, P0, invigorate));
    // P0's Erebos, God of the Dead ("Your opponents can't gain life."): neither.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erebos, God of the Dead");
    let (invigorate, _) = setup(&mut t);
    assert!(!alt_castable(&mut t, P0, invigorate));
    // P1's Erebos stops only P0 from gaining life: P1 can gain it, so the cost is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Erebos, God of the Dead");
    let (invigorate, bears) = setup(&mut t);
    assert!(alt_castable(&mut t, P0, invigorate));
    let alts = alternatives(&t, P0, invigorate);
    t.cast(P0, invigorate)
        .method(alts[0].clone())
        .target(bears)
        .go();
    assert_eq!(t.life(P1), 23);
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
    // With three players and P2's Erebos (P0 and P1 can't gain life), P0 may cast it only
    // by choosing P2, who can gain the life.
    for (chosen, ok) in [(P1, false), (P2, true)] {
        let mut t = TestGame::new(3);
        t.battlefield(P2, "Erebos, God of the Dead");
        let (invigorate, bears) = setup(&mut t);
        assert!(alt_castable(&mut t, P0, invigorate));
        let alts = alternatives(&t, P0, invigorate);
        t.answer_choose(P0, &[Entity::Player(chosen)]);
        let cast = t
            .cast(P0, invigorate)
            .method(alts[0].clone())
            .target(bears)
            .try_go();
        assert_eq!(cast.is_ok(), ok, "choosing {chosen}");
        assert_eq!(t.life(P2), if ok { 23 } else { 20 });
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.in_hand(P0, "Invigorate"), !ok);
    }
}

#[test]
fn leyline_of_punishment_a_cost_that_includes_life_gain_cant_be_paid() {
    cr!("119.7", "118.9");
    ruling!(
        "Leyline of Punishment",
        "If a cost includes life gain (like Invigorate's alternative cost does), that cost can't be paid."
    );
    supported("Reverent Silence");
    // Reverent Silence: "If you control a Forest, rather than pay this spell's mana cost,
    // you may have each other player gain 6 life. Destroy all enchantments." With three
    // players, each other player must be able to gain the life.
    let mut t = TestGame::new(3);
    t.lands(P0, "Forest", 1);
    let silence = t.hand(P0, "Reverent Silence");
    let leyline = t.battlefield(P2, "Leyline of Punishment");
    assert!(!alt_castable(&mut t, P0, silence));
    // Once Leyline of Punishment is gone, P1 and P2 each gain 6.
    crate::r_s02_common::destroy(&mut t, leyline);
    assert!(alt_castable(&mut t, P0, silence));
    let alts = alternatives(&t, P0, silence);
    t.cast(P0, silence).method(alts[0].clone()).go();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 26, 26));
    // One of them unable to gain life is enough: P2's Erebos, God of the Dead ("Your
    // opponents can't gain life.") stops P1, though P2 could gain the life.
    let mut t = TestGame::new(3);
    t.lands(P0, "Forest", 1);
    let silence = t.hand(P0, "Reverent Silence");
    t.battlefield(P2, "Erebos, God of the Dead");
    assert!(!alt_castable(&mut t, P0, silence));
}
