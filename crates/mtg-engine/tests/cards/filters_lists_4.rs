//! Lists of kinds and qualities in object phrases (`oracle/patterns/filter_qualities.rs`):
//! "legendary or Rat card", "each Spider and legendary creature you control", "instant,
//! sorcery, and/or Tamiyo planeswalker cards", "creature without first strike, double
//! strike, or vigilance", "creature card ... that's an artifact or that has mana value 3
//! or less", "each creature that player controls with power greater than the number of
//! cards in their hand".

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

/// The candidates of the latest target choice `p` was asked to make, sorted.
fn offered(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    let mut v = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } if q == p => {
                Some(candidates)
            }
            _ => None,
        })
        .expect("no target choice was asked");
    v.sort_by_key(|e| e.object());
    v
}

fn objs(v: &[ObjectId]) -> Vec<Entity> {
    let mut v: Vec<Entity> = v.iter().map(|o| Entity::Object(*o)).collect();
    v.sort_by_key(|e| e.object());
    v
}

#[test]
fn nashi_targets_legendary_cards_and_rat_cards() {
    cr!("205.4a", "205.3m");
    assert_supported("Nashi, Moon's Legacy");
    let mut t = TestGame::new(2);
    let nashi = t.battlefield(P0, "Nashi, Moon's Legacy");
    let legend = t.graveyard(P0, "Isamaru, Hound of Konda");
    let rat = t.graveyard(P0, "Tangled Colony");
    let pw = t.graveyard(P0, "Jace Beleren");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Tangled Colony");
    t.answer_targets(P0, &[]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(nashi, Entity::Player(P1))], &[]);
    assert_eq!(offered(&t, P0), objs(&[legend, rat, pw]));
}

#[test]
fn ultimate_spider_man_doubles_counters_on_spiders_and_legendary_creatures() {
    cr!("205.4a", "122.1");
    assert_supported("Miles Morales // Ultimate Spider-Man");
    let mut t = TestGame::new(2);
    let usm = t.battlefield(P0, "Miles Morales // Ultimate Spider-Man");
    mtg_engine::dfc::transform(&mut t.g, usm);
    let spider = t.battlefield(P0, "Giant Spider");
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Giant Spider");
    for o in [spider, legend, bears, theirs] {
        t.g.add_counters(Entity::Object(o), "+1/+1", 1, None);
    }
    t.g.flush_events();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(spider, "+1/+1"), 2);
    assert_eq!(t.counters(legend, "+1/+1"), 2);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
}

#[test]
fn the_tale_of_tamiyo_targets_instants_sorceries_and_tamiyo_planeswalkers() {
    cr!("714.2b", "205.3j");
    assert_supported("The Tale of Tamiyo");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Tale of Tamiyo");
    let i = t.graveyard(P0, "Shock");
    let s = t.graveyard(P0, "Divination");
    let tam = t.graveyard(P0, "Tamiyo, the Moon Sage");
    t.graveyard(P0, "Jace Beleren");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Shock");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 3);
    t.answer_targets(P0, &[]);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(offered(&t, P0), objs(&[i, s, tam]));
}

#[test]
fn grand_master_of_flowers_targets_a_creature_without_those_keywords() {
    cr!("702.7a", "702.4a", "702.20a");
    assert_supported("Grand Master of Flowers");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    let plain = t.battlefield(P1, "Grizzly Bears");
    let haste = t.battlefield(P1, "Raging Goblin");
    t.battlefield(P1, "Youthful Knight");
    t.battlefield(P1, "Serra Angel");
    t.activate(P0, gm, 0, &[Entity::Object(plain)]).unwrap();
    assert_eq!(offered(&t, P0), objs(&[plain, haste]));
    t.resolve_all();
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!t.g.can_attack(plain));
    assert!(t.g.can_attack(haste));
}

#[test]
fn desdemona_targets_an_artifact_creature_card_or_one_with_mana_value_3_or_less() {
    cr!("608.2b");
    assert_supported("Desdemona, Freedom's Edge");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Desdemona, Freedom's Edge");
    let artifact = t.graveyard(P0, "Myr Enforcer");
    let cheap = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Craw Wurm");
    t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[Entity::Object(cheap)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(d, Entity::Player(P1))], &[]);
    assert_eq!(offered(&t, P0), objs(&[artifact, cheap]));
}

#[test]
fn noetic_scales_compares_power_with_the_controllers_hand() {
    cr!("603.2");
    assert_supported("Noetic Scales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Noetic Scales");
    let giant = t.battlefield(P1, "Hill Giant");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Hill Giant");
    // P1 will have one card in hand at their upkeep (none drawn yet).
    t.g.players[P1.idx()].hand.clear();
    t.hand(P1, "Shock");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"), "power 3 is greater than 1");
    assert!(t.on_battlefield(elf), "power 1 isn't greater than 1");
    assert!(t.on_battlefield(mine), "only that player's creatures");
    let _ = giant;
}
