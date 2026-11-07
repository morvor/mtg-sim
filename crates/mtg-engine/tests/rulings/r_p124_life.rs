//! Rulings batch P124 — "your life total can't change" (CR 119.10) from Teferi's
//! Protection, Perch Protection and Flare of Fortitude: replacement effects to and from
//! gaining or losing life (CR 614.1a), costs that include gaining or paying life (CR
//! 119.4, 119.8), setting and exchanging life totals (CR 119.5, 701.12b), and commander
//! damage (CR 903.10a).

use crate::r_p124_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The three cards that make P0's life total unable to change.
const PROTECTIONS: [&str; 3] = ["Teferi's Protection", "Perch Protection", "Flare of Fortitude"];

/// P0 casts `name` (Perch Protection with the gift promised) and it resolves.
fn protect(t: &mut TestGame, name: &str) {
    supported(name);
    if name == "Perch Protection" {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    }
    cast_new(t, P0, name, &[]);
    t.resolve_all();
}

/// `p` draws a card (outside of any spell or ability); triggers are put on the stack.
fn draw(t: &mut TestGame, p: PlayerId) {
    t.g.draw_cards(p, 1);
    t.g.flush_events();
    t.settle();
}

/// Whether `p` may begin casting the card with its alternative cost now.
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
fn a_replacement_that_has_you_gain_life_replaces_the_event_with_nothing() {
    cr!("119.10", "614.1a", "614.11");
    ruling!(
        "Teferi's Protection",
        "Effects that replace an event with having you gain life (like Words of Worship's effect does) or having you lose life will apply and end up replacing the event with nothing."
    );
    ruling!(
        "Flare of Fortitude",
        "Effects that replace an event with having you gain life (like Words of Worship's effect does) or having you lose life will end up replacing the event with nothing."
    );
    ruling!(
        "Perch Protection",
        "If your life total can't change, effects that replace an event with having you gain life (like Words of Worship's effect does) or having you lose life will apply and end up replacing the event with nothing."
    );
    supported("Words of Worship");
    for name in PROTECTIONS {
        // Words of Worship: "{1}: The next time you would draw a card this turn, you gain 5
        // life instead." Used before the protection (which may phase it out: the
        // replacement effect stays).
        let mut t = TestGame::new(2);
        let words = t.battlefield(P0, "Words of Worship");
        t.lands(P0, "Wastes", 1);
        t.activate(P0, words, 0, &[]).unwrap();
        t.resolve();
        protect(&mut t, name);
        let hand = t.hand_size(P0);
        draw(&mut t, P0);
        assert_eq!(t.hand_size(P0), hand, "{name}: the draw was replaced");
        assert_eq!(t.life(P0), 20, "{name}");
        // Used up: the next draw is a draw.
        draw(&mut t, P0);
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn a_replacement_of_gaining_life_cant_apply() {
    cr!("119.10", "614.1a", "119.7");
    ruling!(
        "Teferi's Protection",
        "Effects that would replace having you gain life with some other event won't be able to be applied because it's impossible for you to gain life. The same is true for effects that would replace having you lose life with some other event."
    );
    ruling!(
        "Flare of Fortitude",
        "Effects that would replace having you gain life with some other event won't be able to be applied because it's impossible for you to gain life."
    );
    ruling!(
        "Perch Protection",
        "If your life total can't change, effects that would replace having you gain life with some other event won't be able to be applied because it's impossible for you to gain life."
    );
    supported("Lich");
    for name in PROTECTIONS {
        // Lich (put onto the battlefield after the protection, so it's not phased out):
        // "If you would gain life, draw that many cards instead."
        let mut t = TestGame::new(2);
        protect(&mut t, name);
        t.battlefield(P0, "Lich");
        let hand = t.hand_size(P0);
        t.g.gain_life(P0, 3);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.hand_size(P0), hand, "{name}: no cards drawn");
        assert_eq!(t.life(P0), 20, "{name}");
    }
}

#[test]
fn a_cost_that_has_you_gain_life_cant_be_paid() {
    cr!("119.8", "119.7", "118.3", "601.2f");
    ruling!(
        "Teferi's Protection",
        "If a cost includes causing you to gain life (like the alternative cost of an opponent's Invigorate does), that cost can't be paid."
    );
    ruling!(
        "Perch Protection",
        "Similarly, if a cost includes causing you to gain life (like the alternative cost of an opponent's Invigorate does), that cost can't be paid."
    );
    ruling!(
        "Flare of Fortitude",
        "If a cost would include causing you to gain life, that cost can't be paid."
    );
    supported("Invigorate");
    // P1's Invigorate: "If you control a Forest, rather than pay this spell's mana cost,
    // you may have an opponent gain 3 life. Target creature gets +4/+4 until end of turn."
    // P0 is P1's only opponent.
    for name in PROTECTIONS {
        let mut t = TestGame::new(2);
        t.lands(P1, "Forest", 1);
        t.battlefield(P1, "Grizzly Bears");
        let invigorate = t.hand(P1, "Invigorate");
        assert!(alt_castable(&mut t, P1, invigorate), "{name}: before");
        protect(&mut t, name);
        assert!(!alt_castable(&mut t, P1, invigorate), "{name}: after");
    }
}

#[test]
fn a_cost_that_has_you_pay_life_can_only_be_paid_with_zero() {
    cr!("119.4", "119.8", "107.3a");
    ruling!(
        "Perch Protection",
        "If your life total can't change, you can't pay a cost that includes the payment of any amount of life other than 0 life."
    );
    ruling!(
        "Flare of Fortitude",
        "Similarly, you can't pay a cost that includes the payment of any amount of life other than 0 life. However, you can choose to be dealt damage, even though that damage won't result in your life total changing."
    );
    supported("Toxic Deluge");
    // Toxic Deluge: "As an additional cost to cast this spell, pay X life. All creatures
    // get -X/-X until end of turn."
    for name in ["Perch Protection", "Flare of Fortitude"] {
        for x in [1, 0] {
            let mut t = TestGame::new(2);
            protect(&mut t, name);
            lands_for_cost(&mut t, P0, "Toxic Deluge");
            let deluge = t.hand(P0, "Toxic Deluge");
            let cast = t.cast(P0, deluge).x(x).try_go();
            assert_eq!(cast.is_ok(), x == 0, "{name}, X={x}");
            assert_eq!(t.life(P0), 20);
        }
    }
    // Flare of Fortitude: P0 may still tap a painland for colored mana (it's dealt the
    // damage, but its life total doesn't change).
    let mut t = TestGame::new(2);
    protect(&mut t, "Flare of Fortitude");
    let springs = t.battlefield(P0, "Sulfurous Springs");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let _ = springs;
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn setting_your_life_total_does_nothing() {
    cr!("119.5", "119.10");
    ruling!(
        "Teferi's Protection",
        "If an effect would set your life total to a certain number that's different than your current life total, that part of the effect won't do anything."
    );
    ruling!(
        "Flare of Fortitude",
        "If an effect says to set your life total to a certain number, that part of the effect won't do anything."
    );
    ruling!(
        "Perch Protection",
        "If your life total can't change and an effect would set your life total to a specific number that's different from your current life total, that part of the effect won't do anything."
    );
    supported("Repay in Kind");
    // Repay in Kind: "Each player's life total becomes the lowest life total among all
    // players." With P1 at 5, P2 becomes 5; P0 stays at 20.
    for name in PROTECTIONS {
        let mut t = TestGame::new(3);
        set_life(&mut t, P1, 5);
        protect(&mut t, name);
        cast_resolve(&mut t, P0, "Repay in Kind", &[]);
        assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 5, 5), "{name}");
    }
    // Perch Protection: an exchange of life totals doesn't happen either. In P0's upkeep,
    // P0 activates Magus of the Mirror ("{T}, Sacrifice this: Exchange life totals with
    // target opponent. Activate only during your upkeep.").
    ruling!(
        "Perch Protection",
        "Similarly, if an effect would cause you to exchange life totals with another player, the exchange won't happen, and neither player's life total will change."
    );
    supported("Magus of the Mirror");
    for (mine, theirs) in [(5, 12), (12, 5)] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, mine);
        set_life(&mut t, P1, theirs);
        t.set_step(P0, Step::Upkeep);
        protect(&mut t, "Perch Protection");
        let magus = t.battlefield(P0, "Magus of the Mirror");
        t.activate(P0, magus, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Magus of the Mirror"));
        assert_eq!((t.life(P0), t.life(P1)), (mine, theirs));
    }
}

#[test]
fn spells_that_gain_or_lose_you_life_still_resolve() {
    cr!("119.10", "608.2c");
    ruling!(
        "Perch Protection",
        "spells and abilities that would normally cause you to gain or lose life still resolve while your life total can't change, but the life-gain or life-loss part simply has no effect."
    );
    // Thoughtseize: "Target player reveals their hand. You choose a nonland card from it.
    // That player discards that card. You lose 2 life." Lightning Helix: "deals 3 damage
    // to any target and you gain 3 life."
    let mut t = TestGame::new(2);
    protect(&mut t, "Perch Protection");
    t.hand(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Thoughtseize", &[Entity::Player(P1)]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    cast_resolve(&mut t, P0, "Lightning Helix", &[obj(bears)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn flare_of_fortitude_commander_damage_is_still_counted() {
    cr!("903.10a", "119.10");
    ruling!(
        "Flare of Fortitude",
        "In a Commander game, the damage dealt to you by commanders will still be counted, even if that damage doesn't cause you to lose life."
    );
    let mut t = crate::r_s13_common::commander_game();
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].is_commander = true;
    t.g.players[P1.idx()].commander_names.push("Hill Giant".into());
    t.set_step(P1, Step::BeginningOfCombat);
    protect(&mut t, "Flare of Fortitude");
    let life_before = t.life(P0);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), life_before);
    assert_eq!(
        t.g.player(P0).commander_damage.get("Hill Giant").copied(),
        Some(3)
    );
}
