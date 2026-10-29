//! roundtrip-pairs-2 (docs/agents/items/roundtrip-pairs-2.md): the 3+-card round-trip
//! clusters after paper-scope-1, and the next 25 two-card clusters.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hellfire_deals_three_plus_the_creatures_that_died() {
    cr!("107.3c", "700.4");
    // "Destroy all nonblack creatures. Hellfire deals X plus 3 damage to you, where X is
    // the number of creatures that died this way."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Hellfire");
    t.cast(P0, spell).go();
    t.resolve_all();
    // Three nonblack creatures died; the black one didn't.
    assert_eq!(t.life(P0), 20 - 6, "{}", t.dump_log());
}

#[test]
fn norns_decree_poisons_the_opponent_whose_creatures_dealt_the_damage() {
    cr!("603.2c", "122.1f");
    // "Whenever one or more creatures an opponent controls deal combat damage to you,
    // that opponent gets a poison counter."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norn's Decree");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    t.attack(&[(bear, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.player(P1).counter("poison"), 1, "{}", t.dump_log());
    assert_eq!(t.g.player(P0).counter("poison"), 0);
}

#[test]
fn belligerent_yearling_keeps_the_power_it_copied() {
    cr!("608.2h", "613.4b");
    // "you may have this creature's base power become equal to that creature's power
    // until end of turn": the power as the ability resolves.
    let mut t = TestGame::new(2);
    let yearling = t.battlefield(P0, "Belligerent Yearling");
    t.answer_yes(P0, true);
    let dino = t.enter(P0, "Ancient Brontodon");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(yearling).0, 9, "{}", t.dump_log());
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(Entity::Object(dino)).go();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(dino).0, 12);
    assert_eq!(t.pt(yearling).0, 9);
}

#[test]
fn thief_of_blood_counts_the_counters_it_removed() {
    cr!("614.1c", "122.1");
    // "As this creature enters, remove all counters from all permanents. This creature
    // enters with a +1/+1 counter on it for each counter removed this way."
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(wurm), "-1/-1", 1, None);
    let thief = t.enter(P0, "Thief of Blood");
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 0);
    assert_eq!(t.counters(wurm, "-1/-1"), 0);
    assert_eq!(t.counters(thief, "+1/+1"), 3, "{}", t.dump_log());
}

#[test]
fn desolation_damages_the_player_who_sacrificed_a_stolen_plains() {
    cr!("701.21a", "603.10a");
    // "At the beginning of each end step, each player who tapped a land for mana this turn
    // sacrifices a land of their choice. This enchantment deals 2 damage to each player
    // who sacrificed a Plains this way." A player sacrifices a land they control, which
    // may be one another player owns.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Desolation");
    let plains = t.battlefield(P1, "Plains");
    gain_control(&mut t, P0, plains);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    t.activate(P0, t.g.current(plains), 0, &[]).expect("tap for mana");
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Plains"), "{}", t.dump_log());
    assert_eq!(t.life(P0), 18, "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

/// `p` gains control of the permanent.
fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &ability::Effect::GainControl {
            what: ability::Sel::Target(0),
            who: ability::PlayerRef::You,
            duration: ability::Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}
