//! Cost changes and prevention (cards3-costs-mana-replacements, `oracle/patterns/
//! costs_4.rs`, `prevention_4.rs`, `replacements_4.rs`): a next-spell reduction locked
//! "as this ability resolves", life for each time a repeatable additional cost was paid,
//! "Prevent X of that damage, where X is the amount of mana that player paid this way",
//! damage between creatures that share a color, and creatures dealt damage by a source
//! that can't be regenerated.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn kaza_locks_its_reduction_as_the_ability_resolves() {
    cr!("601.2f", "608.2h");
    ruling!(
        "Kaza, Roil Chaser",
        "Kaza's cost reduction can reduce only generic mana in a spell's cost."
    );
    assert_supported("Kaza, Roil Chaser");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kaza = t.battlefield(P0, "Kaza, Roil Chaser");
    t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, kaza, 0, &[]).unwrap();
    t.resolve();
    // Two Wizards as it resolved; two more later don't count.
    t.battlefield(P0, "Prodigal Sorcerer");
    t.battlefield(P0, "Prodigal Sorcerer");
    t.lands(P0, "Island", 1);
    // Fact or Fiction ({3}{U}) costs {1}{U}: one Island isn't enough.
    let fof = t.hand(P0, "Fact or Fiction");
    assert!(t.cast(P0, fof).try_go().is_err());
    // Counterspell's {U}{U} isn't reduced.
    let cs = t.hand(P0, "Counterspell");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let b = t.cast(P1, bolt).target(P0).go();
    assert!(t.cast(P0, cs).target(b).try_go().is_err());
    t.lands(P0, "Island", 1);
    assert!(t.cast(P0, fof).try_go().is_ok());
}

#[test]
fn maelstrom_muse_reduction_is_its_power_as_the_trigger_resolves() {
    cr!("601.2f", "608.2h");
    assert_supported("Maelstrom Muse");
    let mut t = TestGame::new(2);
    let muse = t.battlefield(P0, "Maelstrom Muse");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(muse, Entity::Player(P1))]),
    );
    t.advance_to_step(Step::DeclareBlockers);
    t.resolve_all();
    // Power 2: Fact or Fiction ({3}{U}) costs {1}{U}.
    t.lands(P0, "Island", 1);
    let fof = t.hand(P0, "Fact or Fiction");
    assert!(t.cast(P0, fof).try_go().is_err());
    t.lands(P0, "Island", 1);
    assert!(t.cast(P0, fof).try_go().is_ok());
}

#[test]
fn taste_of_paradise_gains_three_more_for_each_additional_payment() {
    cr!("601.2b", "119.3");
    assert_supported("Taste of Paradise");
    for (times, life) in [(2, 29), (0, 23)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 4 + 2 * times as usize);
        let taste = t.hand(P0, "Taste of Paradise");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(times));
        t.cast(P0, taste).go();
        t.resolve_all();
        assert_eq!(t.life(P0), life);
    }
}

/// Errant Minion on P1's Grizzly Bears; P1's upkeep with `islands` Islands, paying `pay`.
fn errant_minion_upkeep(islands: usize, pay: i64) -> TestGame {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let minion = t.battlefield(P0, "Errant Minion");
    t.g.attach(minion, Entity::Object(bears));
    t.lands(P1, "Island", islands);
    t.answer(P1, DecisionKind::X, Answer::Number(pay));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t
}

#[test]
fn errant_minion_prevents_as_much_damage_as_the_mana_paid() {
    cr!("615.7", "107.1c");
    assert_supported("Errant Minion");
    assert_eq!(errant_minion_upkeep(2, 0).life(P1), 18);
    assert_eq!(errant_minion_upkeep(2, 1).life(P1), 19);
    assert_eq!(errant_minion_upkeep(2, 2).life(P1), 20);
    // Paying more than the damage prevents it all.
    let t = errant_minion_upkeep(3, 3);
    assert_eq!(t.life(P1), 20);
    // P1 may choose at most what they could pay.
    let t = errant_minion_upkeep(1, 2);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn power_leak_prevents_as_much_damage_as_the_mana_paid() {
    cr!("615.7");
    assert_supported("Power Leak");
    let mut t = TestGame::new(2);
    let ench = t.battlefield(P1, "Glorious Anthem");
    let leak = t.battlefield(P0, "Power Leak");
    t.g.attach(leak, Entity::Object(ench));
    t.lands(P1, "Island", 1);
    t.answer(P1, DecisionKind::X, Answer::Number(1));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn well_laid_plans_prevents_damage_between_creatures_that_share_a_color() {
    cr!("615.1a", "105.2");
    assert_supported("Well-Laid Plans");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Well-Laid Plans");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    // Green and green: prevented.
    t.lands(P0, "Forest", 1);
    let fight = t.hand(P0, "Prey Upon");
    t.cast(P0, fight)
        .target(bears).target(elves)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    assert_eq!(t.g.obj(bears).damage, 0);
    // Green and red: dealt.
    t.lands(P0, "Forest", 1);
    let fight = t.hand(P0, "Prey Upon");
    t.cast(P0, fight)
        .target(bears).target(giant)
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.obj(giant).damage, 2);
    // A spell isn't a creature: Shock still deals damage to a green creature.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(elves).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn bone_shaman_stops_regeneration_of_creatures_it_damaged() {
    cr!("701.19c");
    assert_supported("Bone Shaman");
    for activate in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let shaman = t.battlefield(P0, "Bone Shaman");
        t.lands(P0, "Swamp", 1);
        let skel = t.battlefield(P1, "Drudge Skeletons");
        t.lands(P1, "Swamp", 1);
        // A regeneration shield.
        t.activate(P1, skel, 0, &[]).unwrap();
        t.resolve();
        if activate {
            t.activate(P0, shaman, 0, &[]).unwrap();
            t.resolve();
        }
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(shaman, Entity::Player(P1))], &[(skel, shaman)]);
        if activate {
            assert!(t.in_graveyard(P1, "Drudge Skeletons"));
        } else {
            assert!(t.on_battlefield(skel));
            assert!(t.g.obj(skel).tapped);
        }
    }
}
