//! Rulings batch S33 — exchanging life totals (CR 119.7, 119.8):
//! each player gains or loses the life needed to end up with the other's former total,
//! so replacement effects can modify those gains and losses and abilities trigger on
//! them (CR 119.3, 119.5); a player who can't gain life can't make an exchange that would
//! raise their total, a player who can't lose life one that would lower it, and a player
//! whose life total can't change can't exchange at all — the exchange doesn't happen.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s04_common::run_with;
use crate::r_s33_common::*;
use mtg_engine::ability::{Duration, Effect, PlayerFilter, Restriction};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 activates `name` ("{T}, Sacrifice this: Exchange life totals with target
/// opponent. Activate only during your upkeep.") targeting P1, in P0's upkeep, and
/// resolves it (with any triggers).
fn mirror(t: &mut TestGame, name: &str) {
    supported(name);
    let m = t.battlefield(P0, name);
    t.set_step(P0, Step::Upkeep);
    t.activate(P0, m, 0, &[Entity::Player(P1)]).unwrap();
    t.g.flush_events();
    t.resolve();
    t.settle();
}

/// P0 applies "you can't lose life this turn" (or "can't gain life") to themself.
fn restrict_p0(t: &mut TestGame, r: Restriction) {
    run_with(
        t,
        P0,
        Effect::AddRestriction {
            restriction: r,
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

#[test]
fn magus_of_the_mirror_each_player_gains_or_loses_the_difference() {
    cr!("119.5", "119.3", "614.1a", "603.2");
    ruling!(
        "Magus of the Mirror",
        "When the life totals are exchanged, each player gains or loses the amount of life necessary to equal the other player’s previous life total. For example, if player A has 5 life and player B has 3 life before the exchange, player A will lose 2 life and player B will gain 2 life. Replacement effects may modify these gains and losses, and triggered abilities may trigger on them."
    );
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    set_life(&mut t, P1, 3);
    let from = t.g.turn_events.len();
    mirror(&mut t, "Magus of the Mirror");
    assert_eq!((t.life(P0), t.life(P1)), (3, 5));
    assert_eq!(life_gains_since(&t, from, P1), vec![2]);
    let lost: Vec<u32> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::LifeLost { player, amount } if *player == P0 => {
                Some(*amount)
            }
            _ => None,
        })
        .collect();
    assert_eq!(lost, vec![2]);
}

#[test]
fn mirror_universe_replacement_effects_and_triggers_apply_to_the_exchange() {
    cr!("119.5", "614.1a", "603.2");
    ruling!(
        "Mirror Universe",
        "When the life totals are exchanged, each player gains or loses the amount of life necessary to equal the other player’s previous life total. For example, if player A has 5 life and player B has 3 life before the exchange, player A will lose 2 life and player B will gain 2 life. Replacement effects may modify these gains and losses, and triggered abilities may trigger on them."
    );
    // P1 controls Cleric Class ("If you would gain life, you gain that much life plus 1
    // instead.") and Bloodbond Vampire ("Whenever you gain life, put a +1/+1 counter on
    // this creature."); P0 controls Vengeful Warchief ("Whenever you lose life for the
    // first time each turn, put a +1/+1 counter on this creature.").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Cleric Class");
    let vampire = t.battlefield(P1, "Bloodbond Vampire");
    let chief = t.battlefield(P0, "Vengeful Warchief");
    set_life(&mut t, P0, 5);
    set_life(&mut t, P1, 3);
    mirror(&mut t, "Mirror Universe");
    // P1 gains 2 + 1.
    assert_eq!((t.life(P0), t.life(P1)), (3, 6));
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 2);
    t.resolve_all();
    assert_eq!(t.counters(vampire, "+1/+1"), 1);
    assert_eq!(t.counters(chief, "+1/+1"), 1);
}

#[test]
fn magus_of_the_mirror_a_player_who_cant_lose_life_cant_exchange_with_a_lower_total() {
    cr!("119.8");
    ruling!(
        "Magus of the Mirror",
        "If an effect says that a player can’t lose life, that player can’t exchange life totals with a player who has a lower life total; in that case, the exchange won’t happen."
    );
    // P0 (10 life) can't lose life this turn; P1 has 3.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    set_life(&mut t, P1, 3);
    restrict_p0(&mut t, Restriction::CantLoseLife(PlayerFilter::You));
    mirror(&mut t, "Magus of the Mirror");
    assert_eq!((t.life(P0), t.life(P1)), (10, 3));
    // The Magus was still sacrificed as a cost.
    assert!(t.in_graveyard(P0, "Magus of the Mirror"));
    // With a higher total than P0's, the exchange happens (P0 gains life).
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 3);
    set_life(&mut t, P1, 10);
    restrict_p0(&mut t, Restriction::CantLoseLife(PlayerFilter::You));
    mirror(&mut t, "Magus of the Mirror");
    assert_eq!((t.life(P0), t.life(P1)), (10, 3));
}

#[test]
fn flare_of_fortitude_your_life_total_cant_change_so_no_exchange() {
    cr!("119.7", "119.8");
    ruling!(
        "Flare of Fortitude",
        "If an effect would cause you to exchange life totals with another player, the exchange won't happen. Neither player's life total changes."
    );
    supported("Flare of Fortitude");
    // "Until end of turn, your life total can't change, and permanents you control gain
    // hexproof and indestructible." In P0's upkeep, P0 casts it, then activates Magus of
    // the Mirror.
    for (mine, theirs) in [(5, 12), (12, 5)] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, mine);
        set_life(&mut t, P1, theirs);
        t.set_step(P0, Step::Upkeep);
        crate::r_s29_common::cast_and_resolve(&mut t, P0, "Flare of Fortitude", &[]);
        mirror(&mut t, "Magus of the Mirror");
        assert_eq!((t.life(P0), t.life(P1)), (mine, theirs));
    }
}

#[test]
fn teferis_protection_your_life_total_cant_change_so_no_exchange() {
    cr!("119.7", "119.8");
    ruling!(
        "Teferi's Protection",
        "If an effect would cause you to exchange life totals with another player, the exchange won't happen. Neither player's life total changes."
    );
    supported("Teferi's Protection");
    // "Until your next turn, your life total can't change and you gain protection from
    // everything." In P0's upkeep, P0 casts it, then activates Mirror Universe targeting
    // P1 (the target is P1, so P0's protection doesn't matter).
    for (mine, theirs) in [(5, 12), (12, 5)] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, mine);
        set_life(&mut t, P1, theirs);
        t.set_step(P0, Step::Upkeep);
        crate::r_s29_common::cast_and_resolve(&mut t, P0, "Teferi's Protection", &[]);
        mirror(&mut t, "Mirror Universe");
        assert!(t.in_graveyard(P0, "Mirror Universe"));
        assert_eq!((t.life(P0), t.life(P1)), (mine, theirs));
    }
}

#[test]
fn soul_conduit_a_player_who_cant_gain_or_lose_life_cant_exchange() {
    cr!("119.7", "119.8");
    ruling!(
        "Soul Conduit",
        "If a player can’t gain life, that player can’t exchange life totals with a player with a higher life total. If a player can’t lose life, that player can’t exchange life totals with a player with a lower life total. In either of these cases, neither player’s life total will change."
    );
    supported("Soul Conduit");
    // "{6}, {T}: Two target players exchange life totals."
    let conduit = |t: &mut TestGame| {
        let c = t.battlefield(P0, "Soul Conduit");
        t.lands(P0, "Wastes", 6);
        t.activate(P0, c, 0, &[Entity::Player(P0), Entity::Player(P1)])
            .unwrap();
        t.g.flush_events();
        t.resolve_all();
    };
    // Leyline of Punishment: "Players can't gain life." P0 (7) can't take P1's 15.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Punishment");
    set_life(&mut t, P0, 7);
    set_life(&mut t, P1, 15);
    conduit(&mut t);
    assert_eq!((t.life(P0), t.life(P1)), (7, 15));
    // P0 (15) can't lose life: no exchange with P1 (7) either.
    let mut t = TestGame::new(2);
    restrict_p0(&mut t, Restriction::CantLoseLife(PlayerFilter::You));
    set_life(&mut t, P0, 15);
    set_life(&mut t, P1, 7);
    conduit(&mut t);
    assert_eq!((t.life(P0), t.life(P1)), (15, 7));
    // Without such effects, the exchange happens.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 15);
    set_life(&mut t, P1, 7);
    conduit(&mut t);
    assert_eq!((t.life(P0), t.life(P1)), (7, 15));
}
