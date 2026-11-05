//! Rulings batch P043 — Platinum Emperion: "Your life total can't change. (You can't gain
//! or lose life. You can't pay any amount of life except 0.)" (CR 119.7, 119.8, 119.4,
//! 119.5, 614.1a).

use crate::r_p007_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use crate::r_s25_common::cast_new;
use mtg_engine::events::Event;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn emperion(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Platinum Emperion");
    t.battlefield(p, "Platinum Emperion")
}

fn hit(t: &mut TestGame, src: ObjectId, to: Entity, n: u32, combat: bool) {
    t.g.deal_damage(src, to, n, combat);
    t.g.flush_events();
    t.settle();
}

#[test]
fn platinum_emperion_damage_is_still_dealt() {
    cr!("119.3", "119.8", "120.3a", "603.2");
    ruling!(
        "Platinum Emperion",
        "Abilities that trigger whenever damage is dealt to you will still trigger because that damage is still dealt, even though your life total doesn’t change as a result."
    );
    ruling!(
        "Platinum Emperion",
        "Platinum Emperion’s ability doesn’t prevent damage. Rather, it changes the results of that damage. For example, if a creature with lifelink deals damage to you, you won’t lose any life, but its controller will still gain that much life. Similarly, if a creature you control with lifelink deals damage to another player, that player will lose life but you won’t gain any life."
    );
    // Azor's Elocutors: "Whenever a source deals damage to you, remove a filibuster
    // counter from this creature."
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    let elo = t.battlefield(P0, "Azor's Elocutors");
    t.g.add_counters(obj(elo), "filibuster", 2, None);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    hit(&mut t, hawk, pl(P0), 2, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(elo, "filibuster"), 1, "the trigger");
    assert_eq!(t.life(P1), 22, "lifelink");
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::DamagePrevented { .. })));
    // P0's own Nighthawk: P1 loses 2, P0 gains nothing.
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    hit(&mut t, hawk, pl(P1), 2, true);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn platinum_emperion_replacements_with_life_gain_and_life_gain_replacements() {
    cr!("119.7", "614.1a", "614.6", "614.12");
    ruling!(
        "Platinum Emperion",
        "Effects that replace an event with having you gain life (like Words of Worship’s effect does) or having you lose life will end up replacing the event with nothing."
    );
    ruling!(
        "Platinum Emperion",
        "Effects that would replace having you gain life with some other event won’t be able to be applied because it’s impossible for you to gain life."
    );
    // Words of Worship: "{1}: The next time you would draw a card this turn, you gain 5
    // life instead." The draw is replaced; P0 neither draws nor gains.
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    let words = t.battlefield(P0, "Words of Worship");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, words, 0, &[]).unwrap();
    t.resolve_all();
    t.library_top(P0, "Island");
    let (hand, lib) = (t.hand_size(P0), t.library_size(P0));
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(
        (t.hand_size(P0), t.library_size(P0), t.life(P0)),
        (hand, lib, 20)
    );
    // Lich: "If you would gain life, draw that many cards instead." P0 can't gain life, so
    // Angel's Mercy (gain 7) draws nothing.
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    t.battlefield(P0, "Lich");
    for _ in 0..7 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Angel's Mercy"), "it resolves");
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 20));
}

#[test]
fn platinum_emperion_life_costs() {
    cr!("119.4", "119.8", "601.2h", "107.4f", "118.3");
    ruling!(
        "Platinum Emperion",
        "If a cost would include causing you to gain life (like the alternative cost of an opponent’s Invigorate does), that cost can’t be paid."
    );
    ruling!(
        "Platinum Emperion",
        "You can’t pay a cost that includes the payment of any amount of life other than 0 life. However, you can choose to be dealt damage, even though that damage won’t result in your life total changing."
    );
    // Invigorate: "If you control a Forest, rather than pay this spell's mana cost, you
    // may have an opponent gain 4 life." P1's Emperion: no.
    for with in [false, true] {
        let mut t = TestGame::new(2);
        if with {
            emperion(&mut t, P1);
        }
        t.battlefield(P0, "Forest");
        t.battlefield(P0, "Grizzly Bears");
        let inv = t.hand(P0, "Invigorate");
        let alt = crate::r_s07_common::cast_methods(&mut t, P0, inv)
            .into_iter()
            .any(|m| matches!(m, CastMethod::Alternative(_)));
        assert_eq!(alt, !with, "Invigorate's alternative cost");
    }
    // Gitaxian Probe ({U/P}) with no mana: 2 life can't be paid.
    for with in [false, true] {
        let mut t = TestGame::new(2);
        if with {
            emperion(&mut t, P0);
        }
        let probe = t.hand(P0, "Gitaxian Probe");
        assert_eq!(can_cast(&mut t, P0, probe, CastMethod::Normal), !with);
    }
    // Shivan Reef's colored mana ability deals 1 damage to P0: still usable.
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    let reef = t.battlefield(P0, "Shivan Reef");
    t.activate(P0, reef, 1, &[])
        .expect("Shivan Reef's {R}/{U} ability");
    t.settle();
    assert_eq!(t.life(P0), 20);
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::Damage { source, target, .. } if *source == reef && *target == pl(P0)
    )));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn platinum_emperion_set_and_exchange_life_totals() {
    cr!("119.5", "119.7", "119.8", "701.12a");
    ruling!(
        "Platinum Emperion",
        "If an effect says to set your life total to a certain number, that part of the effect won’t do anything."
    );
    ruling!(
        "Platinum Emperion",
        "If an effect would cause you to exchange life totals with another player, the exchange won’t happen. Neither player’s life total changes."
    );
    ruling!(
        "Platinum Emperion",
        "Spells and abilities that would normally cause you to gain or lose life still resolve, but the life-gain or life-loss part simply has no effect."
    );
    supported("Soul Conduit");
    // Blessed Wind ("Target player's life total becomes 20") at P0, at 12.
    let mut t = TestGame::new(2);
    emperion(&mut t, P0);
    t.g.players[P0.idx()].life = 12;
    cast_new(&mut t, P0, "Blessed Wind", &[pl(P0)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blessed Wind"));
    assert_eq!(t.life(P0), 12);
    // Soul Conduit: P0 (12) and P1 (20) don't exchange.
    let conduit = t.battlefield(P1, "Soul Conduit");
    t.lands(P1, "Wastes", 6);
    t.activate(P1, conduit, 0, &[pl(P0), pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (12, 20));
    // Sovereign's Bite ("Target player loses 3 life and you gain 3 life") at P0: P1 gains.
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Sovereign's Bite", &[pl(P0)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sovereign's Bite"));
    assert_eq!((t.life(P0), t.life(P1)), (12, 23));
}

#[test]
fn platinum_emperion_commander_damage_still_counts() {
    cr!("903.10a", "119.8");
    ruling!(
        "Platinum Emperion",
        "In a Commander game, the damage dealt to you by commanders will still be counted, even if that damage doesn’t cause you to lose life."
    );
    let mut t = TestGame::with_config(
        2,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    emperion(&mut t, P0);
    let cmdr = t.battlefield(P1, "Craw Wurm");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[1].commander_names.push("Craw Wurm".into());
    let life = t.life(P0);
    hit(&mut t, cmdr, pl(P0), 6, true);
    assert_eq!(t.life(P0), life);
    let total: u32 = t.g.player(P0).commander_damage.values().sum();
    assert_eq!(total, 6);
    for _ in 0..3 {
        hit(&mut t, cmdr, pl(P0), 6, true);
    }
    assert!(t.has_lost(P0), "21 or more commander damage");
}
