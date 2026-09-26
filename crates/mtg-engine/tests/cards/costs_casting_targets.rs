//! Cost changes that depend on a spell's targets (CR 601.2c, 601.2f): strive ("This spell
//! costs {R}{W} more to cast for each target beyond the first") and "Spells your
//! opponents cast that target this creature cost {2} more to cast."

use mtg_engine::ability::Duration;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn tapped(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| t.obj_now(**l).tapped).count()
}

#[test]
fn strive_costs_more_for_each_target_beyond_the_first() {
    cr!("601.2c", "601.2f", "207.2c");
    ruling!(
        "Desperate Stand",
        "The mana cost and mana value of strive spells don't change no matter how many targets they have."
    );
    compiles("Desperate Stand");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Runeclaw Bear");
    let stand = t.hand(P0, "Desperate Stand");
    let mut lands = t.lands(P0, "Mountain", 2);
    lands.extend(t.lands(P0, "Plains", 1));
    lands.extend(t.lands(P0, "Island", 1));
    // Two targets: {R}{W}{R}{W}, which four lands without a second Plains can't pay.
    assert!(t
        .cast(P0, stand)
        .targets(&[a.into(), b.into()])
        .try_go()
        .is_err());
    assert!(t.in_hand(P0, "Desperate Stand"));
    t.clear_answers();
    // One target: {R}{W}.
    let s = t.cast(P0, stand).targets(&[a.into()]).go();
    assert_eq!(tapped(&t, &lands), 2);
    assert_eq!(t.obj(s).chars.mana_value(), 2);
    t.resolve();
    assert_eq!(t.pt(a), (4, 2));
    // Three targets: {R}{W} three times.
    let stand2 = t.hand(P0, "Desperate Stand");
    let more = [t.lands(P0, "Mountain", 3), t.lands(P0, "Plains", 3)].concat();
    t.cast(P0, stand2)
        .targets(&[a.into(), b.into(), c.into()])
        .go();
    // Two lands were tapped before; six more now.
    assert_eq!(tapped(&t, &lands) + tapped(&t, &more), 8);
    t.resolve();
    assert_eq!(t.pt(a), (6, 2));
    assert_eq!(t.pt(b), (5, 3));
    assert_eq!(t.pt(c), (4, 2));
}

#[test]
fn strive_generic_increase_and_casting_without_paying_the_mana_cost() {
    cr!("601.2f", "118.9d");
    ruling!(
        "Setessan Tactics",
        "If a spell or ability allows you to cast a strive spell without paying its mana cost, you must pay the additional costs for any targets beyond the first."
    );
    compiles("Setessan Tactics");
    compiles("Launch the Fleet");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let fleet = t.hand(P0, "Launch the Fleet");
    // {W} plus {1} for the second target.
    let lands = [t.lands(P0, "Plains", 1), t.lands(P0, "Island", 1)].concat();
    t.cast(P0, fleet).targets(&[a.into(), b.into()]).go();
    assert_eq!(tapped(&t, &lands), 2);
    t.resolve();

    // Cast without paying its mana cost: {G} for each target beyond the first remains.
    let tactics = t.hand(P0, "Setessan Tactics");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![tactics],
        Duration::EndOfTurn,
        true,
        None,
    );
    assert!(t
        .cast(P0, tactics)
        .method(CastMethod::Free)
        .targets(&[a.into(), b.into()])
        .try_go()
        .is_err());
    t.clear_answers();
    let forest = t.lands(P0, "Forest", 1);
    t.cast(P0, tactics)
        .method(CastMethod::Free)
        .targets(&[a.into(), b.into()])
        .go();
    assert_eq!(tapped(&t, &forest), 1);
    t.resolve();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn spells_opponents_cast_that_target_it_cost_more() {
    cr!("601.2f");
    ruling!(
        "Icefall Regent",
        "affects all spells cast by your opponents that target it, including Aura spells and spells with multiple targets. It doesn’t affect abilities."
    );
    compiles("Icefall Regent");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let regent = t.battlefield(P1, "Icefall Regent");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    let m = t.lands(P0, "Mountain", 1);
    // A spell targeting something else costs as usual; one targeting the Regent costs
    // {2} more.
    assert!(t.cast(P0, bolt).target(regent).try_go().is_err());
    t.clear_answers();
    t.cast(P0, bolt).target(bears).go();
    assert_eq!(tapped(&t, &m), 1);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // A spell with several targets, one of them the Regent.
    let arc = t.hand(P0, "Arc Lightning");
    let lands = t.lands(P0, "Mountain", 4);
    assert!(t
        .cast(P0, arc)
        .targets(&[regent.into(), P1.into()])
        .try_go()
        .is_err());
    t.clear_answers();
    let more = t.lands(P0, "Mountain", 1);
    t.cast(P0, arc)
        .targets(&[regent.into(), P1.into()])
        .go();
    assert_eq!(tapped(&t, &lands) + tapped(&t, &more), 5);
    // Abilities aren't affected; the Regent's controller's own spells aren't either.
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    assert!(t.activate(P0, pyro, 0, &[regent.into()]).is_ok());
    let own = t.hand(P1, "Lightning Bolt");
    let p1m = t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, own).target(regent).go();
    assert_eq!(tapped(&t, &p1m), 1);
}
