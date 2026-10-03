//! Scoped replacement effects compiled by the replacement grammar
//! (`src/oracle/patterns/replacement_grammar_scopes.rs`): "until end of turn, [replacement
//! effect]", mana type replacements, control on entering, and skipping steps and turns
//! (CR 611.2a, 614.1, 614.10, 106.12b).

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn tap(t: &mut TestGame, p: PlayerId, land: ObjectId) -> Vec<ManaType> {
    t.activate(p, land, 0, &[]).unwrap();
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    t.g.players[p.idx()].mana_pool.empty();
    t.g.untap(land);
    v
}

#[test]
fn scope_grammar_cards_compile() {
    compiles(&[
        "Kiora, the Crashing Wave",
        "Kaya, Geist Hunter",
        "Dovin, Hand of Control",
        "Elfhame Sanctuary",
        "Collective Inferno",
        "Gather Specimens",
        "Fasting",
        "Pale Moon",
        "Prairie Dog",
        "Magosi, the Waterveil",
        "Deep Water",
    ]);
}

#[test]
fn pale_moon_makes_nonbasic_lands_produce_colorless_this_turn() {
    cr!("106.12b", "611.2a");
    let mut t = TestGame::new(2);
    let dual = t.battlefield(P0, "Tropical Island");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Island", 2);
    let moon = t.hand(P0, "Pale Moon");
    t.cast(P0, moon).go();
    t.resolve();
    t.g.untap(dual);
    t.g.untap(forest);
    assert_eq!(tap(&mut t, P0, dual), vec![ManaType::C]);
    assert_eq!(tap(&mut t, P0, forest), vec![ManaType::G]);
}

#[test]
fn prairie_dog_adds_one_more_counter_until_end_of_turn() {
    cr!("614.1a", "122.6", "611.2a");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    t.activate(P0, dog, 0, &[]).unwrap();
    t.resolve();
    t.g.add_counters(Entity::Object(bears), "+1/+1".into(), 1, Some(dog));
    t.settle();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn gather_specimens_steals_creatures_entering_under_an_opponents_control() {
    cr!("614.1c", "614.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let g = t.hand(P0, "Gather Specimens");
    t.cast(P0, g).go();
    t.resolve();
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(bears)).controller, P0);
}

#[test]
fn magosi_skips_your_next_turn() {
    cr!("614.10", "614.10a");
    let mut t = TestGame::new(2);
    let magosi = t.battlefield(P0, "Magosi, the Waterveil");
    t.lands(P0, "Island", 1);
    t.activate(P0, magosi, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(magosi, "eon"), 1);
    assert_eq!(t.g.player(P0).skips.len(), 1);
}

#[test]
fn fasting_may_skip_your_draw_step_to_gain_life() {
    cr!("614.10", "614.10b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fasting");
    t.advance_to(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
    assert_eq!(t.life(P0), 22);
}

/// P1 is protected from all damage this turn.
fn shield_p1(t: &mut TestGame) {
    t.lands(P1, "Plains", 3);
    let sp = t.hand(P1, "Safe Passage");
    t.cast(P1, sp).go();
    t.resolve();
}

#[test]
fn urzas_rage_kicked_deals_ten_unpreventable_damage() {
    cr!("615.12", "608.2c");
    compiles(&["Urza's Rage", "Lightning Surge", "Arrow Storm", "Demonfire", "Banefire"]);
    let mut t = TestGame::new(2);
    shield_p1(&mut t);
    t.lands(P0, "Mountain", 12);
    let rage = t.hand(P0, "Urza's Rage");
    t.cast(P0, rage).kicked(true).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 10);
    let rage2 = t.hand(P0, "Urza's Rage");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, rage2).kicked(false).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn banefire_damage_cant_be_prevented_if_x_is_5_or_more() {
    cr!("615.12", "107.3");
    let mut t = TestGame::new(2);
    shield_p1(&mut t);
    t.lands(P0, "Mountain", 10);
    let small = t.hand(P0, "Banefire");
    t.cast(P0, small).x(4).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    let big = t.hand(P0, "Banefire");
    t.lands(P0, "Mountain", 6);
    t.cast(P0, big).x(5).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn woolly_razorback_prevents_its_combat_damage_while_it_has_an_ice_counter() {
    cr!("615.1a", "611.3a");
    compiles(&["Woolly Razorback"]);
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Woolly Razorback");
    t.g.objects[boar.0 as usize].counters.insert("ice".into(), 1);
    t.g.dirty = true;
    t.g.recompute();
    assert!(t.g.obj(boar).has_keyword(mtg_engine::keywords::KeywordKind::Defender));
    t.g.deal_damage(boar, Entity::Player(P1), 7, true);
    t.settle();
    assert_eq!(t.life(P1), 20);
    t.g.objects[boar.0 as usize].counters.clear();
    t.g.dirty = true;
    t.g.deal_damage(boar, Entity::Player(P1), 7, true);
    t.settle();
    assert_eq!(t.life(P1), 13);
}

#[test]
fn hundred_battle_veteran_cast_from_the_graveyard_gets_a_finality_counter() {
    cr!("614.1c", "601.2");
    compiles(&["Hundred-Battle Veteran", "Mariposa Military Base"]);
    let mut t = TestGame::new(2);
    let v = t.graveyard(P0, "Hundred-Battle Veteran");
    t.lands(P0, "Swamp", 4);
    t.cast(P0, v).go();
    t.resolve();
    let v = t.named_on_battlefield("Hundred-Battle Veteran")[0];
    assert_eq!(t.counters(v, "finality"), 1);
}

#[test]
fn mariposa_military_base_may_enter_tapped_for_rad_counters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let base = t.enter(P0, "Mariposa Military Base");
    t.resolve_all();
    assert!(t.g.obj(t.g.current(base)).tapped);
    assert_eq!(t.g.player(P0).counters.get("rad").copied().unwrap_or(0), 2);
}
