//! Counters on permanents: "each of up to two target creatures", distributing counters
//! among targets (CR 601.2d), doubling counters (CR 701.10e), amounts counted from
//! counters ("draw a card for each +1/+1 counter on it", "add {C} for each charge counter
//! on ~"), and conditions on counters ("if there are no arrowhead counters on ~", "if it
//! had a +1/+1 counter on it").

use mtg_engine::decision::Answer;
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

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
}

#[test]
fn travel_preparations_puts_a_counter_on_each_of_two_targets() {
    cr!("601.2c", "122.1a");
    assert_supported(&["Travel Preparations"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let tp = t.hand(P0, "Travel Preparations");
    t.cast(P0, tp)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.pt(a), (3, 3));
}

#[test]
fn travel_preparations_still_counters_the_legal_target() {
    cr!("608.2b");
    ruling!(
        "Travel Preparations",
        "you'll still put a +1/+1 counter on the other creature"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let tp = t.hand(P0, "Travel Preparations");
    t.cast(P0, tp)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.g.destroy(b, None);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
}

#[test]
fn armament_corps_distributes_counters_among_targets() {
    cr!("601.2d", "603.3d");
    assert_supported(&["Armament Corps"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bear), Entity::Object(elf)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.enter(P0, "Armament Corps");
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 1);
    assert_eq!(t.counters(elf, "+1/+1"), 1);
}

#[test]
fn armament_corps_can_put_both_counters_on_one_target() {
    cr!("601.2d");
    ruling!(
        "Armament Corps",
        "The enters-the-battlefield ability can target Armament Corps itself."
    );
    let mut t = TestGame::new(2);
    // Its only target is itself: both counters go on it.
    let corps = t.enter(P0, "Armament Corps");
    t.answer_targets(P0, &[Entity::Object(corps)]);
    t.resolve_all();
    assert_eq!(t.counters(corps, "+1/+1"), 2);
}

#[test]
fn gearhulk_counters_for_an_illegal_target_are_lost() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Verdurous Gearhulk",
        "the +1/+1 counters that would have been put on that creature are lost"
    );
    assert_supported(&["Verdurous Gearhulk"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bear), Entity::Object(elf)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    let hulk = t.enter(P0, "Verdurous Gearhulk");
    t.settle();
    // The Elves leave before the trigger resolves.
    t.g.destroy(elf, None);
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 3);
    assert_eq!(t.counters(hulk, "+1/+1"), 0);
}

#[test]
fn kalonian_hydra_doubles_counters_on_each_creature() {
    cr!("701.10e");
    assert_supported(&["Kalonian Hydra"]);
    let mut t = TestGame::new(2);
    let hydra = t.enter(P0, "Kalonian Hydra");
    t.g.objects[hydra.0 as usize].summoning_sick = false;
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, "+1/+1", 1);
    let other = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, other, "+1/+1", 1);
    assert_eq!(t.counters(hydra, "+1/+1"), 4);
    t.attack(&[(hydra, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(hydra, "+1/+1"), 8);
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    // Only creatures you control.
    assert_eq!(t.counters(other, "+1/+1"), 1);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn gilder_bairn_doubles_each_kind_of_counter() {
    cr!("701.10e", "122.1b");
    assert_supported(&["Gilder Bairn"]);
    let mut t = TestGame::new(2);
    let bairn = t.battlefield(P0, "Gilder Bairn");
    t.g.objects[bairn.0 as usize].tapped = true;
    let bear = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, bear, "+1/+1", 2);
    add(&mut t, bear, "flying", 1);
    t.lands(P0, "Forest", 3);
    t.activate(P0, bairn, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 4);
    assert_eq!(t.counters(bear, "flying"), 2);
    // {Q}: the Bairn untapped as a cost.
    assert!(!t.obj_now(bairn).tapped);
}

#[test]
fn marketback_walker_draws_for_each_counter_it_had() {
    cr!("603.10a", "122.1a");
    assert_supported(&["Marketback Walker"]);
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Marketback Walker");
    add(&mut t, w, "+1/+1", 3);
    let hand = t.hand_size(P0);
    t.g.destroy(w, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn everflowing_chalice_adds_mana_per_charge_counter() {
    cr!("106.4", "122.1");
    assert_supported(&["Everflowing Chalice"]);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Everflowing Chalice");
    add(&mut t, c, "charge", 2);
    t.activate(P0, c, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 2);
}

#[test]
fn promising_duskmage_checks_counters_it_had() {
    cr!("603.4", "603.10a");
    assert_supported(&["Promising Duskmage"]);
    let mut t = TestGame::new(2);
    let with = t.battlefield(P0, "Promising Duskmage");
    add(&mut t, with, "+1/+1", 1);
    let without = t.battlefield(P0, "Promising Duskmage");
    let hand = t.hand_size(P0);
    t.g.destroy(with, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.g.destroy(without, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn serrated_arrows_is_sacrificed_with_no_arrowhead_counters() {
    cr!("603.4", "122.1");
    assert_supported(&["Serrated Arrows"]);
    let mut t = TestGame::new(2);
    let full = t.battlefield(P0, "Serrated Arrows");
    add(&mut t, full, "arrowhead", 1);
    let empty = t.battlefield(P0, "Serrated Arrows");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(full));
    assert!(!t.on_battlefield(empty));
}

#[test]
fn heliophial_deals_damage_equal_to_its_charge_counters() {
    cr!("122.1");
    assert_supported(&["Heliophial"]);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Heliophial");
    add(&mut t, h, "charge", 3);
    t.lands(P0, "Plains", 2);
    t.activate(P0, h, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // It was sacrificed as a cost: the damage uses the counters it last had.
    assert_eq!(t.life(P1), 17);
}
