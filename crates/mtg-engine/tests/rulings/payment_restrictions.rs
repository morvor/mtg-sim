//! "Players can't pay life [or sacrifice (permanents)] to cast spells or activate
//! abilities [that aren't mana abilities]" (Karn's Sylex, Yasharn, Implacable Earth,
//! Angel of Jubilation): CR 118.3, 119.4 (`rule_statics::payment`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn sylex_stops_life_payments_for_spells_and_non_mana_abilities() {
    cr!("118.3", "119.4", "107.4f");
    ruling!("Karn's Sylex", "has a cost that requires a player to pay life, that spell or ability can’t be cast or activated");
    ruling!("Karn's Sylex", "An activated mana ability is one that produces mana as it resolves");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Karn's Sylex");
    // "Pay 7 life: Draw seven cards."
    let gris = t.battlefield(P0, "Griselbrand");
    assert!(t.activate(P0, gris, 0, &[]).is_err());
    assert_eq!(t.life(P0), 20);
    // Gitaxian Probe ({U/P}) can't be paid for with life.
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(t.cast(P0, probe).target(P1).try_go().is_err());
    assert_eq!(t.life(P0), 20);
    // ...but can with mana.
    t.lands(P0, "Island", 1);
    assert!(t.cast(P0, probe).target(P1).try_go().is_ok());
    // A mana ability that costs life may still be activated.
    let town = t.battlefield(P0, "Starting Town");
    assert!(t.activate(P0, town, 1, &[]).is_ok());
    assert_eq!(t.life(P0), 19);
}

#[test]
fn life_paid_as_a_spell_or_ability_resolves_is_still_paid() {
    cr!("118.3", "702.21a");
    ruling!("Karn's Sylex", "Other things may still cause players to pay life, such as a resolving spell or ability");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Karn's Sylex");
    let witch = t.battlefield(P1, "Sedgemoor Witch");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.cast(P0, bolt).target(witch).go();
    t.resolve_all();
    // Ward's "unless that player pays 3 life" was paid.
    assert_eq!(t.life(P0), 17);
    assert!(!t.on_battlefield(witch));
}

#[test]
fn players_may_always_pay_zero_life() {
    cr!("119.4b");
    ruling!("Karn's Sylex", "Players may always pay 0 life");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Karn's Sylex");
    assert!(t.g.may_pay_life_for_cost(
        P0,
        0,
        &{
            let mut c = eval::Ctx::new(None, P0);
            c.cost_of = Some(rule_statics::payment::CostOf::Spell);
            c
        }
    ));
}

#[test]
fn angel_of_jubilation_stops_sacrificing_creatures_to_pay_costs() {
    cr!("118.3");
    ruling!("Angel of Jubilation", "by sacrificing an artifact creature");
    ruling!("Angel of Jubilation", "or sacrifice a creature (as Fling does), that spell or ability can’t be cast or activated");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Angel of Jubilation");
    let atog = t.battlefield(P0, "Atog");
    let thopter = t.battlefield(P0, "Ornithopter");
    // "Sacrifice an artifact: Atog gets +2/+2": the only artifact is a creature.
    assert!(t.activate(P0, atog, 0, &[]).is_err());
    assert!(t.on_battlefield(thopter));
    // A noncreature artifact may be sacrificed.
    let stone = t.battlefield(P0, "Mind Stone");
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    assert!(t.activate(P0, atog, 0, &[]).is_ok());
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(stone));
    // Fling ("As an additional cost to cast this spell, sacrifice a creature") can't be
    // cast.
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    assert!(t.cast(P0, fling).target(P1).try_go().is_err());
    assert!(t.on_battlefield(atog));
}

#[test]
fn a_resolving_spell_still_makes_players_sacrifice_creatures() {
    cr!("118.3", "701.21a");
    ruling!("Angel of Jubilation", "Other things may still cause players to pay life or sacrifice creatures, such as a resolving spell or ability");
    ruling!("Yasharn, Implacable Earth", "Other things may still cause players to pay life or sacrifice creatures, such as a resolving spell or ability");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Jubilation");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let e = t.hand(P0, "Diabolic Edict");
    t.cast(P0, e).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn yasharn_stops_sacrificing_nonland_permanents_even_for_mana() {
    cr!("118.3", "605.3a");
    ruling!("Yasharn, Implacable Earth", "that spell or ability can’t be cast or activated");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yasharn, Implacable Earth");
    let petal = t.battlefield(P0, "Lotus Petal");
    assert!(t.activate(P0, petal, 0, &[]).is_err());
    assert!(t.on_battlefield(petal));
    // Life for a mana ability can't be paid either.
    let town = t.battlefield(P0, "Starting Town");
    assert!(t.activate(P0, town, 1, &[]).is_err());
    assert_eq!(t.life(P0), 20);
    // A land may be sacrificed.
    let wilds = t.battlefield(P0, "Evolving Wilds");
    t.library_top(P0, "Forest");
    assert!(t.activate(P0, wilds, 0, &[]).is_ok());
    assert!(!t.on_battlefield(wilds));
}
