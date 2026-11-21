//! "Create an X/X [token]" where X is a spell's X, a "where X is ..." value, or the X of an
//! earlier instruction ("You may pay X life, where X is .... If you do, create an X/X
//! ..."): the token's power and toughness are that number as it's created (CR 107.3).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The tokens `p` controls with `subtype` (their P/T).
fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<(i32, i32)> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| (o.power(), o.toughness()))
        .collect()
}

#[test]
fn slime_molding_creates_an_x_x_ooze_for_the_spells_x() {
    cr!("107.3a", "111.4");
    compiles("Slime Molding");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let slime = t.hand(P0, "Slime Molding");
    t.cast(P0, slime).x(3).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Ooze"), vec![(3, 3)]);
}

#[test]
fn miming_slime_x_is_the_greatest_power_among_your_creatures() {
    cr!("107.3c");
    compiles("Miming Slime");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 3);
    let slime = t.hand(P0, "Miming Slime");
    t.cast(P0, slime).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Ooze"), vec![(3, 3)]);
    // The token's size was locked in as it was created.
    crate_destroy(&mut t, giant);
    assert_eq!(tokens(&t, P0, "Ooze"), vec![(3, 3)]);
}

fn crate_destroy(t: &mut TestGame, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &mtg_engine::ability::Effect::Destroy {
            what: mtg_engine::ability::Sel::Target(0),
            no_regen: false,
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn primordial_ooze_pays_x_or_takes_x_damage() {
    cr!("107.3c", "118.12");
    compiles("Primordial Ooze");
    // "At the beginning of your upkeep, put a +1/+1 counter on this creature. Then you may
    // pay {X}, where X is the number of +1/+1 counters on it. If you don't, tap this
    // creature and it deals X damage to you."
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Primordial Ooze");
    t.g.add_counters(Entity::Object(ooze), "+1/+1", 1, None);
    let upkeep = |t: &mut TestGame, pay: bool| {
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.answer_yes(P0, pay);
        t.resolve_all();
    };
    // Two counters now, and three Forests: P0 declines to pay {2}, so the Ooze is tapped
    // and deals 2 damage to P0.
    t.lands(P0, "Forest", 3);
    upkeep(&mut t, false);
    assert_eq!(t.counters(ooze, "+1/+1"), 2);
    assert!(t.obj_now(ooze).tapped);
    assert_eq!(t.life(P0), 18);
    let yes_no = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 1);
    // Three counters: P0 pays {3} (all three Forests), so the Ooze stays untapped and
    // deals no damage.
    upkeep(&mut t, true);
    assert_eq!(t.counters(ooze, "+1/+1"), 3);
    assert!(!t.obj_now(ooze).tapped);
    assert_eq!(t.life(P0), 18);
    let untapped_forests = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Forest") && !o.tapped)
        .count();
    assert_eq!(untapped_forests, 0);
    // Four counters, and only three Forests: P0 can't pay {4}; 4 damage.
    upkeep(&mut t, true);
    assert_eq!(t.counters(ooze, "+1/+1"), 4);
    assert!(t.obj_now(ooze).tapped);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn chainers_torment_the_token_deals_its_x_to_you() {
    cr!("107.3c", "714.2b");
    compiles("Chainer's Torment");
    // "III — Create an X/X black Nightmare Horror creature token, where X is half your
    // life total, rounded up. It deals X damage to you."
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Chainer's Torment");
    let lore = |t: &mut TestGame, n: u32| {
        let id = t.g.current(saga);
        t.g.add_counters(Entity::Object(id), "lore", n, None);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
    };
    // Chapters I and II first.
    lore(&mut t, 2);
    t.g.players[0].life = 15;
    lore(&mut t, 1);
    assert_eq!(tokens(&t, P0, "Horror"), vec![(8, 8)]);
    assert_eq!(t.life(P0), 7);
}
