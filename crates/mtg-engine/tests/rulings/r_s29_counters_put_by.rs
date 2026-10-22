//! Rulings batch S29 — replacement effects that care about who puts counters on a
//! permanent or player (CR 122.6, 122.6a) or who creates tokens: Halving Season ("If an
//! opponent would put one or more counters on a permanent or player, they put half that
//! many ... instead, rounded down.") and Vorinclex, Monstrous Raider ("If you would put one
//! or more counters on a permanent or player, put twice that many ... instead." and the
//! same halving as Halving Season), applied one at a time (CR 614.1a, 616.1).

use crate::r_s01_common::supported;
use crate::r_s29_common::*;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` creates `n` Treasure tokens as a resolving effect would. Returns how many Treasures
/// `p` controls afterwards.
fn create_treasures(t: &mut TestGame, p: PlayerId, n: i32) -> usize {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure");
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(n),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    crate::r_s01_common::with_subtype(t, p, "Treasure").len()
}

/// `by` puts `n` counters of `kind` on `on` as a resolving ability of `source` (a
/// permanent `by` controls) would.
fn put_by(t: &mut TestGame, source: ObjectId, on: Entity, kind: &str, n: u32) -> u32 {
    let k = t.g.add_counters(on, kind, n, Some(source));
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    k
}

#[test]
fn halving_season_doesnt_change_your_own_tokens_or_counters() {
    cr!("614.1a", "122.6");
    ruling!(
        "Halving Season",
        "Halving Season’s abilities won’t change the number of tokens you create or counters you place, even if you’re placing counters on permanents your opponents control."
    );
    supported("Halving Season");
    let mut t = TestGame::new(2);
    let season = t.battlefield(P0, "Halving Season");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P0 creates four Treasures and puts four +1/+1 counters on P1's Bears: unchanged.
    assert_eq!(create_treasures(&mut t, P0, 4), 4);
    assert_eq!(
        put_by(&mut t, season, Entity::Object(theirs), counters::PLUS1, 4),
        4
    );
    assert_eq!(t.counters(theirs, counters::PLUS1), 4);
    // P1 creates five Treasures: two; P1 puts three counters on P0's creature and on
    // itself: one each.
    assert_eq!(create_treasures(&mut t, P1, 5), 2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(
        put_by(&mut t, theirs, Entity::Object(mine), counters::PLUS1, 3),
        1
    );
    assert_eq!(
        put_by(&mut t, theirs, Entity::Player(P1), counters::POISON, 3),
        1
    );
    assert_eq!(t.g.player(P1).counter(counters::POISON), 1);
}

#[test]
fn several_halving_seasons_halve_one_at_a_time_rounding_down_each_time() {
    cr!("614.1a", "616.1", "616.1f");
    ruling!(
        "Halving Season",
        "If a player would create tokens or put counters on a permanent or player while their opponents control two or more Halving Seasons, apply the replacement effects one at a time, rounding down each time. For example, if a player would create 30 Treasure tokens while their opponents control three Halving Seasons, they would halve 30 to 15, halve 15 to 7, and then halve 7 to 3, and end up creating three Treasure tokens."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Halving Season");
    }
    assert_eq!(create_treasures(&mut t, P1, 30), 3);
    // Counters too: 30 → 15 → 7 → 3.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(
        put_by(&mut t, bears, Entity::Object(bears), counters::PLUS1, 30),
        3
    );
}

#[test]
fn vorinclex_cares_about_who_puts_the_counters() {
    cr!("122.6", "122.6a", "614.1a");
    ruling!(
        "Vorinclex, Monstrous Raider",
        "Unlike many similar effects, Vorinclex cares deeply about who is putting the counters on the permanent or player to determine which of its two last abilities applies."
    );
    supported("Vorinclex, Monstrous Raider");
    supported("Star Pupil");
    let mut t = TestGame::new(2);
    let vorinclex = t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // P0 puts counters on P1's creature and on P1: doubled.
    assert_eq!(
        put_by(&mut t, vorinclex, Entity::Object(theirs), counters::PLUS1, 2),
        4
    );
    assert_eq!(
        put_by(&mut t, vorinclex, Entity::Player(P1), counters::POISON, 1),
        2
    );
    // P1 puts counters on P0's creature: halved, rounded down.
    assert_eq!(
        put_by(&mut t, theirs, Entity::Object(mine), counters::PLUS1, 3),
        1
    );
    // A Star Pupil entering under P1's control: P1 puts its counter on it (halved to
    // none), and under P0's control: doubled.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    crate::r_s25_common::cast_new(&mut t, P1, "Star Pupil", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Star Pupil"));
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let pupil = crate::r_s25_common::cast_new(&mut t, P0, "Star Pupil", &[]);
    t.resolve_all();
    assert_eq!(t.counters(pupil, counters::PLUS1), 2);
}

#[test]
fn you_order_vorinclex_and_another_counter_effect_for_your_permanent() {
    cr!("616.1", "616.1e");
    ruling!(
        "Vorinclex, Monstrous Raider",
        "If two or more effects attempt to modify how many counters would be put onto a permanent you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Hardened Scales");
    // Vorinclex (P0) doubles; Hardened Scales (P0): "If one or more +1/+1 counters would
    // be put on a creature you control, that many plus one +1/+1 counters are put on it
    // instead." One counter: doubled then plus one (3), or plus one then doubled (4).
    for (order, expected) in [(twice_first as Responder, 3), (plus_one_first, 4)] {
        let mut t = TestGame::new(2);
        let vorinclex = t.battlefield(P0, "Vorinclex, Monstrous Raider");
        t.battlefield(P0, "Hardened Scales");
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_s03_common::respond(&mut t, P0, order);
        let from = t.asked().len();
        put_by(&mut t, vorinclex, Entity::Object(bears), counters::PLUS1, 1);
        assert_eq!(t.counters(bears, counters::PLUS1), expected);
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

#[test]
fn a_player_who_moves_a_counter_puts_it_on_the_second_permanent() {
    cr!("122.5", "122.6", "614.1a");
    ruling!(
        "Simic Fluxmage",
        "Any abilities that care about a counter being placed on the second creature will apply."
    );
    supported("Simic Fluxmage");
    // P0's Simic Fluxmage ("{1}{U}, {T}: Move a +1/+1 counter from this creature onto
    // target creature.") moves its counter onto P1's Grizzly Bears while P0 controls
    // Vorinclex: P0 puts the counter on the Bears, so it's doubled (not halved, as it
    // would be if P1, the Bears' controller, put it).
    let mut t = TestGame::new(2);
    let fluxmage = t.battlefield(P0, "Simic Fluxmage");
    put_counters(&mut t, fluxmage, counters::PLUS1, 1);
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::U, 1);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    crate::r_s06_common::activate_containing(&mut t, P0, fluxmage, "Move").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(fluxmage, counters::PLUS1), 0);
    assert_eq!(t.counters(theirs, counters::PLUS1), 2);
}
