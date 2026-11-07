//! CR 702.193 Power-up (`src/kw/power_up.rs`; "Power-up — ..." in
//! `oracle/patterns/k702_179_195.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Brave Brawler ({1}{W} 2/1): "Lifelink. Power-up — {4}{W}: Put two +1/+1 counters on
/// this creature."
const BRAWLER: &str = "Brave Brawler";

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(CardType::Land) && !o.tapped)
        .count()
}

/// Activates the power-up ability (the `i`th activated ability) of `src`; returns the
/// number of lands tapped for it, or None if it couldn't be activated.
fn power_up(t: &mut TestGame, src: ObjectId, i: usize) -> Option<usize> {
    let before = untapped_lands(t);
    t.activate(P0, src, i, &[]).ok()?;
    let spent = before - untapped_lands(t);
    t.resolve_all();
    Some(spent)
}

/// Puts a card onto the battlefield on an earlier turn (it didn't enter this turn).
fn old(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.battlefield(P0, name);
    t.g.turn.number = 3;
    t.g.objects[id.0 as usize].entered_turn = 1;
    id
}

#[test]
fn power_up_cards_compile() {
    assert_supported(&[
        BRAWLER,
        "Serpent Specialist",
        "Unliving Legionnaire",
        "Ultron Drone",
        "Hulk, Gamma Goliath",
        "Wonder Man, Hollywood Hero",
    ]);
}

#[test]
fn a_power_up_ability_can_be_activated_only_once() {
    cr!("702.193a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 10);
    let b = old(&mut t, BRAWLER);
    // It didn't enter this turn: the full {4}{W}.
    assert_eq!(power_up(&mut t, b, 0), Some(5));
    assert_eq!(t.counters(b, counters::PLUS1), 2);
    // Only once: not again this turn, nor on a later turn.
    assert_eq!(power_up(&mut t, b, 0), None);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(power_up(&mut t, b, 0), None);
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}

#[test]
fn a_new_object_has_a_new_power_up_ability() {
    cr!("702.193a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 10);
    let b = old(&mut t, BRAWLER);
    assert!(power_up(&mut t, b, 0).is_some());
    // It leaves and returns: a new object (CR 400.7) that entered this turn.
    for to in [ZoneKind::Exile, ZoneKind::Battlefield] {
        let cur = now(&t, b);
        run(
            &mut t,
            P0,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(to),
            },
            &[Entity::Object(cur)],
        );
    }
    let b2 = now(&t, b);
    assert_ne!(b2, b);
    assert!(t.on_battlefield(b2));
    // Reduced by its mana cost: {4}{W} - {1}{W} = {3}.
    assert_eq!(power_up(&mut t, b2, 0), Some(3));
    assert_eq!(t.counters(b2, counters::PLUS1), 2);
}

#[test]
fn reduced_by_its_mana_cost_the_turn_it_entered() {
    cr!("702.193a", "702.193b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let c = t.hand(P0, BRAWLER);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(untapped_lands(&t), 3);
    let b = named(&t, BRAWLER)[0];
    // {4}{W} reduced by {1}{W}: {3}.
    assert_eq!(power_up(&mut t, b, 0), Some(3));
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}

#[test]
fn colored_mana_reduces_the_same_color_and_any_excess_reduces_generic() {
    cr!("702.193b");
    // Serpent Specialist ({G}): "Power-up — {3}{G}": the {G} reduces the {G}, leaving {3}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let s = t.enter(P0, "Serpent Specialist");
    assert_eq!(power_up(&mut t, s, 0), Some(3));
    assert_eq!(t.counters(s, counters::PLUS1), 2);
    // A {1}{U}{U} permanent with "Power-up — {4}{U}": one {U} reduces the {U}, the other
    // {U} and the {1} reduce generic mana: {2}.
    let def = custom_card(
        "Blue Upstart",
        "{1}{U}{U}",
        "Creature — Human Hero",
        Some((1, 1)),
        "Power-up — {4}{U}: Put two +1/+1 counters on this creature.",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let id = t.g.create_card_object(std::sync::Arc::new(def), P0, Zone::Nowhere);
    let moved = t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    });
    let u = moved.unwrap();
    assert_eq!(power_up(&mut t, u, 0), Some(2));
    // Generic mana in the mana cost reduces only generic: Ultron Drone ({3}) with
    // "Power-up — {6}" costs {3}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let d = t.enter(P0, "Ultron Drone");
    assert_eq!(power_up(&mut t, d, 0), Some(3));
}

#[test]
fn power_up_abilities_can_be_activated_an_additional_time() {
    cr!("702.193a");
    // Wonder Man: "Each power-up ability of permanents you control can be activated an
    // additional time."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 15);
    old(&mut t, "Wonder Man, Hollywood Hero");
    let b = old(&mut t, BRAWLER);
    assert!(power_up(&mut t, b, 0).is_some());
    assert!(power_up(&mut t, b, 0).is_some());
    assert!(power_up(&mut t, b, 0).is_none());
    assert_eq!(t.counters(b, counters::PLUS1), 4);
}

#[test]
fn power_up_abilities_of_other_creatures_cost_less() {
    cr!("702.193a");
    // Hulk, Gamma Goliath: "Power-up abilities of other creatures you control cost {3}
    // less to activate."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 10);
    old(&mut t, "Hulk, Gamma Goliath");
    let b = old(&mut t, BRAWLER);
    // {4}{W} - {3} = {1}{W}.
    assert_eq!(power_up(&mut t, b, 0), Some(2));
    // An opponent's Hulk doesn't reduce them.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 10);
    let hulk = t.battlefield(P1, "Hulk, Gamma Goliath");
    t.g.objects[hulk.0 as usize].entered_turn = 0;
    let b = old(&mut t, BRAWLER);
    assert_eq!(power_up(&mut t, b, 0), Some(5));
}

#[test]
fn power_up_abilities_of_named_characters() {
    cr!("702.193a");
    // Abomination, Terrifying Titan ({3}{R/G} 4/4): "Power-up — {5}{R/G}{R/G}: Put a
    // +1/+1 counter on Abomination. He fights up to one target creature an opponent
    // controls."; Donald Blake, Guise of Thor ({1}{W} 1/3): "Power-up — {4}{W}{W}: Put two
    // +1/+1 counters and a flying counter on Donald Blake. He becomes a God Warrior Hero.";
    // Quicksilver, Brash Blur ({R} 1/1): "Power-up — {4}{R}: Put a +1/+1 counter and a
    // double strike counter on Quicksilver."
    const ABOMINATION: &str = "Abomination, Terrifying Titan";
    const BLAKE: &str = "Donald Blake, Guise of Thor";
    const QUICKSILVER: &str = "Quicksilver, Brash Blur";
    assert_supported(&[
        ABOMINATION,
        BLAKE,
        QUICKSILVER,
        "Captain Marvel, Earth's Protector",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let a = old(&mut t, ABOMINATION);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    assert_eq!(power_up(&mut t, a, 0), Some(7));
    // A 5/5 fights the 6/4.
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(a));
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let b = old(&mut t, BLAKE);
    assert_eq!(power_up(&mut t, b, 0), Some(6));
    assert_eq!(t.pt(b), (3, 5));
    let o = t.obj(b);
    assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(o.chars.has_subtype("God") && o.chars.has_subtype("Hero"));
    assert!(!o.chars.has_subtype("Human") && !o.chars.has_subtype("Doctor"));
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let q = old(&mut t, QUICKSILVER);
    assert_eq!(power_up(&mut t, q, 0), Some(5));
    assert_eq!(t.counters(q, "double strike"), 1);
    assert!(t
        .obj(q)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
}
