//! CR 721: station cards (and the station keyword, CR 702.184).

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Wedgelight Rammer ({3}{W} Artifact — Spacecraft): "When this Spacecraft enters,
/// create a 2/2 colorless Robot artifact creature token. Station. 9+ | Flying, first
/// strike" with a 3/4 power/toughness box in that striation.
const RAMMER: &str = "Wedgelight Rammer";
/// Lumen-Class Frigate ({1}{W}): "Station. 2+ | Other creatures you control get +1/+1.
/// 12+ | Flying, lifelink" (3/5 box at 12+).
const FRIGATE: &str = "Lumen-Class Frigate";

fn charge(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), n);
    t.g.recompute();
}

fn station_statics(id: &str) -> usize {
    card(id)
        .front()
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if matches!(&s.condition, Some(Condition::Compare(Value::CountersOn(_, Some(k)), Cmp::Ge, _)) if k.as_str() == counters::CHARGE))
        })
        .count()
}

#[test]
fn a_station_card_has_station_symbols_and_the_station_ability() {
    cr!("721.1");
    ruling!(
        "Atmospheric Greenhouse",
        "Each station card has one or more striations in its text box"
    );
    for (name, symbols) in [(RAMMER, 1), (FRIGATE, 2)] {
        supported(name);
        assert_eq!(station_statics(name), symbols, "{name}");
        assert!(card(name).front().chars.has_keyword(KeywordKind::Station));
    }
}

#[test]
fn a_station_symbol_grants_its_abilities_at_n_or_more_charge_counters() {
    cr!("721.2", "721.2a", "721.3");
    ruling!(
        "Atmospheric Greenhouse",
        "if a permanent with station gets charge counters due to some other effect (such as proliferate) or loses charge counters somehow, its abilities change accordingly"
    );
    let mut t = TestGame::new(2);
    let frigate = t.battlefield(P0, FRIGATE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    charge(&mut t, frigate, 1);
    assert_eq!(t.pt(bears), (2, 2));
    // 2+: "Other creatures you control get +1/+1." — only that striation's abilities.
    charge(&mut t, frigate, 2);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.obj(frigate).chars.has_keyword(KeywordKind::Flying));
    charge(&mut t, frigate, 11);
    assert!(!t.obj(frigate).chars.has_keyword(KeywordKind::Flying));
    // 12+: flying and lifelink too; the 2+ ability still applies.
    charge(&mut t, frigate, 12);
    assert!(t.obj(frigate).chars.has_keyword(KeywordKind::Flying));
    assert!(t.obj(frigate).chars.has_keyword(KeywordKind::Lifelink));
    assert_eq!(t.pt(bears), (3, 3));
    // Losing counters loses the abilities.
    charge(&mut t, frigate, 0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn a_station_symbol_with_a_power_toughness_box_makes_it_a_creature() {
    cr!("721.2b");
    ruling!(
        "Atmospheric Greenhouse",
        "it's not a creature unless it has the appropriate number of charge counters on it"
    );
    let mut t = TestGame::new(2);
    let rammer = t.battlefield(P0, RAMMER);
    charge(&mut t, rammer, 8);
    let c = &t.obj(rammer).chars;
    assert!(!c.is_creature());
    assert_eq!((c.power, c.toughness), (None, None));
    charge(&mut t, rammer, 9);
    let c = &t.obj(rammer).chars;
    // A creature in addition to its other types, with base power and toughness 3/4.
    assert!(c.is_creature() && c.is(CardType::Artifact) && c.has_subtype("Spacecraft"));
    assert!(c.has_keyword(KeywordKind::Flying) && c.has_keyword(KeywordKind::FirstStrike));
    assert_eq!(t.pt(rammer), (3, 4));
    // It's base power and toughness: modifications apply on top.
    t.g.add_counters(Entity::Object(rammer), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(rammer), (4, 5));
}

#[test]
fn outside_the_battlefield_a_station_card_has_no_power_or_toughness() {
    cr!("721.2c");
    ruling!(
        "Atmospheric Greenhouse",
        "That card also doesn't have that power and toughness in any zone other than the battlefield"
    );
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, RAMMER);
    let gy = t.graveyard(P0, FRIGATE);
    t.g.recompute();
    for id in [hand, gy] {
        let c = &t.obj(id).chars;
        assert_eq!((c.power, c.toughness), (None, None));
        assert!(!c.is_creature());
    }
}

#[test]
fn the_station_ability_can_always_be_activated() {
    cr!("721.4");
    ruling!(
        "Atmospheric Greenhouse",
        "Even if a permanent with station has equal or greater charge counters than the number in its last station symbol, you can still activate its station ability"
    );
    ruling!(
        "Atmospheric Greenhouse",
        "Use the tapped creature’s power as the station ability resolves to determine how many charge counters to put on the permanent with station"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let rammer = t.battlefield(P0, RAMMER);
    let giant = t.battlefield(P0, "Hill Giant");
    // "Station": tap another untapped creature you control, put charge counters equal to
    // its power; the Hill Giant has power 3.
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, rammer, 0, &[]).unwrap();
    assert!(t.obj(giant).tapped);
    t.resolve_all();
    assert_eq!(t.counters(rammer, counters::CHARGE), 3);
    // With more counters than its last station symbol, it can still be activated.
    charge(&mut t, rammer, 20);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, rammer, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(rammer, counters::CHARGE), 22);
    // Only as a sorcery.
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_choose(P0, &[Entity::Object(ogre)]);
    assert!(t.activate(P0, rammer, 0, &[]).is_err());
}
