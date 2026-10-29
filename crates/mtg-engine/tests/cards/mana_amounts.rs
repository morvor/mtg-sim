//! Amounts of mana (CR 106.1, 107.3): "Add X mana of any one color, where X is ~'s
//! power", "Add three mana in any combination of {R} and/or {G}", "Remove X ki counters
//! from ~: Add X mana of any one color", storage lands ("Remove any number of storage
//! counters from ~: Add {W} for each storage counter removed this way"), and mana that
//! isn't lost as steps and phases end (CR 106.4).

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

const W: usize = 0;
const R: usize = 3;

#[test]
fn amount_families_compile() {
    assert_supported(&[
        "Heronblade Elite",
        "Sanctum Weaver",
        "Harabaz Druid",
        "Wirewood Channeler",
        "Lotus Blossom",
        "Goblin Clearcutter",
        "Orcish Lumberjack",
        "Relic of Sauron",
        "Burnt Offering",
        "Haruspex",
        "Petalmane Baku",
        "Wizard's Rockets",
        "Fountain of Cho",
        "Dwarven Hold",
        "Mage-Ring Network",
        "Black Mana Battery",
        "Molten Slagheap",
        "Calciform Pools",
        "Savage Ventmaw",
        "Brazen Collector",
        "Sakura-Tribe Springcaller",
        "Su-Chi Cave Guard",
        "Colossal Plow",
        "Karn, Legacy Reforged",
    ]);
}

#[test]
fn x_mana_of_one_color_where_x_is_its_power() {
    cr!("107.3c", "106.1a");
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Heronblade Elite");
    t.g.add_counters(Entity::Object(elite), "+1/+1", 2, None);
    t.answer(P0, DecisionKind::Option, Answer::Index(R));
    t.activate(P0, elite, 0, &[]).unwrap();
    // Power 3: three red mana, one color chosen for all of it.
    assert_eq!(pool(&t, P0), vec![ManaType::R; 3]);
    let asked = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .count();
    assert_eq!(asked, 1);
}

#[test]
fn x_mana_counts_as_the_ability_resolves() {
    cr!("107.3c", "605.3b");
    ruling!(
        "Sanctum Weaver",
        "is a mana ability and does not use the stack"
    );
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P0, "Sanctum Weaver");
    t.battlefield(P0, "Pacifism");
    t.activate(P0, weaver, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    // Sanctum Weaver itself and Pacifism.
    assert_eq!(pool(&t, P0), vec![ManaType::W; 2]);
}

#[test]
fn counters_on_a_sacrificed_source_use_its_last_known_information() {
    cr!("107.3c", "608.2h");
    let mut t = TestGame::new(2);
    let lotus = t.battlefield(P0, "Lotus Blossom");
    t.g.add_counters(Entity::Object(lotus), "petal", 3, None);
    t.answer(P0, DecisionKind::Option, Answer::Index(R));
    t.activate(P0, lotus, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Lotus Blossom"));
    assert_eq!(pool(&t, P0), vec![ManaType::R; 3]);
}

#[test]
fn any_combination_of_two_types() {
    cr!("106.1a");
    ruling!("Goblin Clearcutter", "{R}{R}{G}");
    let mut t = TestGame::new(2);
    let clearcutter = t.battlefield(P0, "Goblin Clearcutter");
    let forest = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    for i in [0, 1, 0] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    t.activate(P0, clearcutter, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R, ManaType::G]);
    // Each mana was a choice between red and green only.
    for (_, d) in t.asked() {
        if let Decision::ChooseOption { options, .. } = d {
            assert_eq!(options, vec!["R".to_string(), "G".to_string()]);
        }
    }
}

#[test]
fn any_combination_pays_a_cost_with_several_types() {
    cr!("106.1a", "601.2g");
    // Relic of Sauron's two mana pay the {U}{B} of Recoil ({1}{U}{B}) in one activation.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Relic of Sauron");
    t.battlefield(P0, "Wastes");
    let spell = t.hand(P0, "Recoil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.cast(P0, spell).target(bears).go();
    assert!(t.obj_now(relic).tapped);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn remove_x_counters_for_x_mana() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    let haruspex = t.battlefield(P0, "Haruspex");
    t.g.add_counters(Entity::Object(haruspex), "+1/+1", 3, None);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(W));
    t.activate(P0, haruspex, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::W; 2]);
    assert_eq!(t.counters(haruspex, "+1/+1"), 1);
}

#[test]
fn storage_land_removes_any_number_of_counters() {
    cr!("107.1c");
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Fountain of Cho");
    t.g.add_counters(Entity::Object(fountain), "storage", 3, None);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, fountain, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::W; 2]);
    assert_eq!(t.counters(fountain, "storage"), 1);
    assert!(t.obj_now(fountain).tapped);
}

#[test]
fn storage_land_cant_remove_more_counters_than_it_has() {
    cr!("118.3");
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Fountain of Cho");
    t.g.add_counters(Entity::Object(fountain), "storage", 1, None);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    assert!(t.activate(P0, fountain, 1, &[]).is_err());
    assert!(pool(&t, P0).is_empty());
    assert_eq!(t.counters(fountain, "storage"), 1);
    assert!(!t.obj_now(fountain).tapped);
}

#[test]
fn mana_battery_adds_one_plus_the_counters_removed() {
    cr!("107.1c");
    let mut t = TestGame::new(2);
    let battery = t.battlefield(P0, "Black Mana Battery");
    t.g.add_counters(Entity::Object(battery), "charge", 2, None);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, battery, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::B; 3]);
    assert_eq!(t.counters(battery, "charge"), 0);
}

#[test]
fn storage_land_with_x_in_its_cost_and_a_combination() {
    cr!("107.3a", "106.1a");
    let mut t = TestGame::new(2);
    let slagheap = t.battlefield(P0, "Molten Slagheap");
    t.g.add_counters(Entity::Object(slagheap), "storage", 2, None);
    t.battlefield(P0, "Wastes");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, slagheap, 2, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::B, ManaType::R]);
    assert_eq!(t.counters(slagheap, "storage"), 0);
}

#[test]
fn attack_trigger_mana_stays_until_end_of_turn() {
    cr!("106.4", "514.2");
    let mut t = TestGame::new(2);
    let collector = t.battlefield(P0, "Brazen Collector");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(collector, Entity::Player(P1))], &[]);
    // Steps and phases ended since the mana was added; it's still there.
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    t.advance_to(P0, Step::End);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // The turn ends: it's lost.
    t.advance_to(P1, Step::Upkeep);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn other_mana_is_still_lost_as_steps_end() {
    cr!("106.4");
    let mut t = TestGame::new(2);
    let collector = t.battlefield(P0, "Brazen Collector");
    let mountain = t.battlefield(P0, "Mountain");
    t.set_step(P0, Step::BeginningOfCombat);
    t.activate(P0, mountain, 0, &[]).unwrap();
    t.attack(&[(collector, Entity::Player(P1))], &[]);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    assert!(t.g.player(P0).mana_pool.mana[0].persistent);
}

#[test]
fn mana_battery_with_no_counters() {
    cr!("107.1c");
    ruling!(
        "Black Mana Battery",
        "Can be tapped even if it has no counters."
    );
    let mut t = TestGame::new(2);
    let battery = t.battlefield(P0, "Black Mana Battery");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, battery, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::B]);
}

#[test]
fn the_mana_stays_after_its_source_leaves() {
    cr!("106.4");
    ruling!(
        "Savage Ventmaw",
        "even if Savage Ventmaw leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let ventmaw = t.battlefield(P0, "Savage Ventmaw");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ventmaw, Entity::Player(P1))], &[]);
    t.g.destroy(ventmaw, None);
    assert!(t.in_graveyard(P0, "Savage Ventmaw"));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(
        pool(&t, P0),
        vec![
            ManaType::R,
            ManaType::R,
            ManaType::R,
            ManaType::G,
            ManaType::G,
            ManaType::G
        ]
    );
}
