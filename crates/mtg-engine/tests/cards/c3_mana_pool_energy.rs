//! Unspent mana, mana kept as steps end, energy paid, and changes to the mana a land is
//! tapped for (cards3-costs-mana-replacements, `oracle/patterns/mana_4.rs`,
//! `mana_change.rs`, `kw/energy_paid.rs`, `kw/unspent_mana_value.rs`).

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
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

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

fn color(t: &mut TestGame, p: PlayerId, ty: ManaType) {
    let i = [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G]
        .iter()
        .position(|x| *x == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Taps each land for its first mana ability.
fn tap_for_mana(t: &mut TestGame, p: PlayerId, lands: &[ObjectId]) {
    for l in lands {
        t.activate(p, *l, 0, &[]).unwrap();
    }
}

fn pt_now(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

#[test]
fn omnath_grows_with_unspent_green_mana() {
    cr!("106.4", "613.4c");
    ruling!(
        "Omnath, Locus of Mana",
        "You can keep unspent green mana indefinitely while Omnath is on the battlefield."
    );
    assert_supported("Omnath, Locus of Mana");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let omnath = t.battlefield(P0, "Omnath, Locus of Mana");
    let forests = t.lands(P0, "Forest", 3);
    let mountain = t.battlefield(P0, "Mountain");
    tap_for_mana(&mut t, P0, &forests);
    tap_for_mana(&mut t, P0, &[mountain]);
    // Red mana doesn't count.
    assert_eq!(pt_now(&mut t, omnath), (4, 4));
    // The green mana stays as the step ends; it still counts.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t, P0), vec![ManaType::G; 3]);
    assert_eq!(pt_now(&mut t, omnath), (4, 4));
    // Spending it shrinks Omnath.
    t.advance_to(P0, Step::PostcombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    assert_eq!(pt_now(&mut t, omnath), (2, 2));
}

#[test]
fn ozai_has_flying_and_indestructible_with_six_unspent_mana() {
    cr!("106.4", "613.1f");
    assert_supported("Ozai, the Phoenix King");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ozai = t.battlefield(P0, "Ozai, the Phoenix King");
    let lands = t.lands(P0, "Mountain", 6);
    let has = |t: &mut TestGame, k: KeywordKind| {
        t.g.recompute();
        t.obj_now(ozai).chars.has_keyword(k)
    };
    tap_for_mana(&mut t, P0, &lands[..5]);
    assert!(!has(&mut t, KeywordKind::Flying));
    tap_for_mana(&mut t, P0, &lands[5..]);
    assert!(has(&mut t, KeywordKind::Flying));
    assert!(has(&mut t, KeywordKind::Indestructible));
    // Spending one leaves five.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    assert!(!has(&mut t, KeywordKind::Flying));
}

#[test]
fn avatar_roku_mana_lasts_until_end_of_combat() {
    cr!("500.4", "106.4");
    assert_supported("Avatar Roku, Firebender");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Avatar Roku, Firebender");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P0))]),
    );
    // P1 attacks: P0 adds six {R}.
    t.advance_to_step(Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(pool(&t, P0), vec![ManaType::R; 6]);
    t.advance_to_step(Step::CombatDamage);
    assert_eq!(pool(&t, P0), vec![ManaType::R; 6]);
    t.advance_to_step(Step::EndOfCombat);
    assert_eq!(pool(&t, P0), vec![ManaType::R; 6]);
    // Lost as the combat phase ends.
    t.advance_to_step(Step::PostcombatMain);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn fire_lord_ozai_adds_red_mana_equal_to_the_sacrificed_creatures_power() {
    cr!("500.4", "701.21a");
    assert_supported("Fire Lord Ozai");
    for sacrifice in [true, false] {
        let mut t = TestGame::new(2);
        let ozai = t.battlefield(P0, "Fire Lord Ozai");
        let giant = t.battlefield(P0, "Hill Giant");
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(ozai, Entity::Player(P1))]),
        );
        t.answer_yes(P0, sacrifice);
        if sacrifice {
            t.answer_choose(P0, &[Entity::Object(giant)]);
        }
        t.advance_to_step(Step::DeclareBlockers);
        t.resolve_all();
        if !sacrifice {
            assert!(t.on_battlefield(giant));
            assert!(pool(&t, P0).is_empty());
            continue;
        }
        assert!(t.in_graveyard(P0, "Hill Giant"));
        assert_eq!(pool(&t, P0), vec![ManaType::R; 3]);
        t.advance_to_step(Step::EndOfCombat);
        assert_eq!(pool(&t, P0), vec![ManaType::R; 3]);
        t.advance_to_step(Step::PostcombatMain);
        assert!(pool(&t, P0).is_empty());
    }
}

#[test]
fn shizuko_mana_stays_until_the_end_of_that_players_turn() {
    cr!("106.4", "514.2");
    assert_supported("Shizuko, Caller of Autumn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shizuko, Caller of Autumn");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(pool(&t, P1), vec![ManaType::G; 3]);
    assert!(pool(&t, P0).is_empty());
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(pool(&t, P1), vec![ManaType::G; 3]);
    t.advance_to(P1, Step::End);
    assert_eq!(pool(&t, P1), vec![ManaType::G; 3]);
    // Gone once the turn ends.
    t.advance_to(P0, Step::Upkeep);
    assert!(pool(&t, P1).is_empty());
}

#[test]
fn blaster_hulk_costs_less_for_energy_paid_this_turn() {
    cr!("107.14", "601.2f");
    ruling!(
        "Blaster Hulk",
        "It counts only {E} you've paid or lost that turn before you cast it."
    );
    assert_supported("Blaster Hulk");
    let setup = |pay: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let hub = t.enter(P0, "Aether Hub");
        t.resolve_all();
        assert_eq!(t.g.player(P0).counter("energy"), 1);
        if pay {
            color(&mut t, P0, ManaType::R);
            t.activate(P0, hub, 1, &[]).unwrap();
            assert_eq!(t.g.player(P0).counter("energy"), 0);
        } else {
            t.activate(P0, hub, 0, &[]).unwrap();
        }
        t.lands(P0, "Mountain", 6);
        t
    };
    // Seven mana: {6}{R}{R} less {1}.
    let mut t = setup(true);
    let hulk = t.hand(P0, "Blaster Hulk");
    assert!(t.cast(P0, hulk).try_go().is_ok());
    // Without paying energy, seven mana isn't enough.
    let mut t = setup(false);
    let hulk = t.hand(P0, "Blaster Hulk");
    assert!(t.cast(P0, hulk).try_go().is_err());
    // The next turn it counts again from zero: Aether Hub and six Mountains (seven mana)
    // aren't enough.
    let mut t = setup(true);
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.g.player(P0).counter("energy"), 0);
    let hulk = t.hand(P0, "Blaster Hulk");
    assert!(t.cast(P0, hulk).try_go().is_err());
}

#[test]
fn izzet_generatorium_needs_four_energy_paid_this_turn() {
    cr!("107.14", "602.1b");
    assert_supported("Izzet Generatorium");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gen = t.battlefield(P0, "Izzet Generatorium");
    assert!(t.activate(P0, gen, 0, &[]).is_err());
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let hl = t.hand(P0, "Harnessed Lightning");
    // {E}{E}{E} plus one; pay all four.
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.cast(P0, hl).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.g.player(P0).counter("energy"), 0);
    let hand = t.hand_size(P0);
    t.activate(P0, gen, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn aether_spike_counters_unless_its_controller_pays_for_the_energy_paid() {
    cr!("107.14", "118.12");
    ruling!(
        "Aether Spike",
        "You don't choose how much {E} to pay until Aether Spike is resolving."
    );
    assert_supported("Aether Spike");
    for (paid, p1_pays) in [(2, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Forest", 2);
        let bears = t.hand(P1, "Grizzly Bears");
        t.cast(P1, bears).go();
        if p1_pays {
            t.lands(P1, "Forest", 2);
        }
        t.lands(P0, "Island", 2);
        let spike = t.hand(P0, "Aether Spike");
        let spell = t.g.current(bears);
        t.cast(P0, spike).target(spell).go();
        t.answer(P0, DecisionKind::X, Answer::Number(paid));
        t.answer_yes(P1, p1_pays);
        t.resolve();
        assert_eq!(t.g.player(P0).counter("energy"), 0);
        t.resolve_all();
        assert_eq!(t.on_battlefield(t.g.current(bears)), p1_pays);
    }
}

#[test]
fn damping_sphere_makes_a_land_tapped_for_two_mana_produce_one_colorless() {
    cr!("106.12b", "614.1a");
    assert_supported("Damping Sphere");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Damping Sphere");
    let tomb = t.battlefield(P0, "Ancient Tomb");
    let plains = t.battlefield(P0, "Plains");
    let ring = t.battlefield(P0, "Sol Ring");
    tap_for_mana(&mut t, P0, &[tomb]);
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
    assert_eq!(t.life(P0), 18);
    tap_for_mana(&mut t, P0, &[plains]);
    assert_eq!(pool(&t, P0), vec![ManaType::W, ManaType::C]);
    // Not a land.
    tap_for_mana(&mut t, P0, &[ring]);
    assert_eq!(pool(&t, P0).len(), 4);
}

#[test]
fn damping_sphere_taxes_each_spell_after_the_first() {
    cr!("601.2f");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Damping Sphere");
    let a = t.hand(P0, "Ornithopter");
    let b = t.hand(P0, "Ornithopter");
    t.cast(P0, a).go();
    t.resolve_all();
    assert!(t.cast(P0, b).try_go().is_err());
    t.lands(P0, "Plains", 1);
    assert!(t.cast(P0, b).try_go().is_ok());
}

#[test]
fn pulse_of_llanowar_makes_your_basic_lands_produce_a_color_of_your_choice() {
    cr!("106.12b", "614.1a");
    assert_supported("Pulse of Llanowar");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Pulse of Llanowar");
    let plains = t.battlefield(P0, "Plains");
    let tomb = t.battlefield(P0, "Ancient Tomb");
    let theirs = t.battlefield(P1, "Plains");
    color(&mut t, P0, ManaType::G);
    tap_for_mana(&mut t, P0, &[plains]);
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    // Not a basic land: unchanged.
    tap_for_mana(&mut t, P0, &[tomb]);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::C, ManaType::C]);
    // An opponent's Plains: unchanged.
    tap_for_mana(&mut t, P1, &[theirs]);
    assert_eq!(pool(&t, P1), vec![ManaType::W]);
}

#[test]
fn harvest_mage_makes_lands_produce_one_mana_of_a_chosen_color_this_turn() {
    cr!("106.12b", "614.1a", "514.2");
    assert_supported("Harvest Mage");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mage = t.battlefield(P0, "Harvest Mage");
    t.lands(P0, "Forest", 1);
    let discard = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.activate(P0, mage, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let tomb = t.battlefield(P0, "Ancient Tomb");
    color(&mut t, P0, ManaType::R);
    tap_for_mana(&mut t, P0, &[tomb]);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // Next turn it ends.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    tap_for_mana(&mut t, P0, &[tomb]);
    assert_eq!(pool(&t, P0), vec![ManaType::C, ManaType::C]);
}

#[test]
fn quarum_trench_gnomes_changes_one_plains_from_then_on() {
    cr!("106.12b", "611.2a");
    assert_supported("Quarum Trench Gnomes");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gnomes = t.battlefield(P0, "Quarum Trench Gnomes");
    let a = t.battlefield(P1, "Plains");
    let b = t.battlefield(P1, "Plains");
    t.activate(P0, gnomes, 0, &[Entity::Object(a)]).unwrap();
    t.resolve();
    tap_for_mana(&mut t, P1, &[a, b]);
    assert_eq!(pool(&t, P1), vec![ManaType::W, ManaType::C]);
    // It lasts indefinitely.
    t.g.players[P1.idx()].mana_pool.mana.clear();
    t.advance_to(P1, Step::PrecombatMain);
    tap_for_mana(&mut t, P1, &[a]);
    assert_eq!(pool(&t, P1), vec![ManaType::C]);
}
