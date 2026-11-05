//! Rulings batch P192 — static abilities of cards with unusual mana costs or P/T
//! (Godhead of Awe and Omnibian set base P/T without removing abilities, CR 613.4b;
//! Nobilis of War; Primalcrux's chroma; Wilt-Leaf Liege; Force of Savagery's 0 toughness,
//! CR 704.5f), total costs (Karador, Maelstrom Muse, Etherium Sculptor, CR 601.2f), and
//! Wastes and Snow-Covered Wastes in constructed and limited decks (CR 100.2).

use crate::r_p194_common::{assert_total_cost, casts_for, mana};
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::has_kw;
use crate::r_s08_common::mana_value;
use crate::r_s29_common::put_counters;
use mtg_engine::card::{card, CardDef};
use mtg_engine::combat;
use mtg_engine::deck::{check_constructed, check_limited, DeckProblem};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------- P/T setting and boosts

#[test]
fn setting_base_power_and_toughness_keeps_abilities() {
    cr!("613.4b", "613.1f");
    ruling!(
        "Godhead of Awe",
        "This doesn't cause creatures to lose their abilities."
    );
    ruling!(
        "Omnibian",
        "This effect changes only the power, toughness, and creature type of the targeted creature. It does not cause that creature to lose any abilities."
    );
    supported("Godhead of Awe");
    supported("Omnibian");
    // Godhead of Awe: other creatures have base P/T 1/1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Godhead of Awe");
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.recompute();
    assert_eq!(t.pt(angel), (1, 1));
    assert!(has_kw(&t, angel, KeywordKind::Flying));
    assert!(has_kw(&t, angel, KeywordKind::Vigilance));
    // Omnibian: a 3/3 Frog until end of turn.
    let mut t = TestGame::new(2);
    let omni = t.battlefield(P0, "Omnibian");
    let angel = t.battlefield(P1, "Serra Angel");
    t.activate(P0, omni, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(angel), (3, 3));
    let o = t.obj_now(angel);
    assert!(o.chars.has_subtype("Frog") && !o.chars.has_subtype("Angel"));
    assert!(has_kw(&t, angel, KeywordKind::Flying));
    assert!(has_kw(&t, angel, KeywordKind::Vigilance));
}

#[test]
fn nobilis_of_war_boosts_attackers_only_while_attacking() {
    cr!("506.4", "511.3", "611.3a");
    ruling!(
        "Nobilis of War",
        "Attacking creatures you control will get +2/+0 from the moment they're declared as attackers"
    );
    supported("Nobilis of War");
    let mut t = TestGame::new(2);
    let nobilis = t.battlefield(P0, "Nobilis of War");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(a), (2, 2));
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.g.recompute();
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(home), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    // Removed from combat: no longer boosted.
    combat::remove_from_combat(&mut t.g, a);
    t.g.recompute();
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (4, 2));
    // Nobilis leaves: neither.
    move_to(&mut t, nobilis, Zone::Graveyard(P0));
    t.g.recompute();
    assert_eq!(t.pt(b), (2, 2));
    // With Nobilis back, the boost ends with the combat phase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nobilis of War");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.g.recompute();
    assert_eq!(t.pt(a), (4, 2));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
    t.g.recompute();
    assert_eq!(t.pt(a), (2, 2));
}

#[test]
fn primalcrux_counts_itself_only_on_the_battlefield() {
    cr!("207.2c", "604.3", "208.3");
    ruling!(
        "Primalcrux",
        "The ability functions while it's in other zones, but since it isn't a permanent while in those zones it won't count itself."
    );
    supported("Primalcrux");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Primalcrux");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (0, 0));
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (1, 1));
    let on_bf = t.battlefield(P0, "Primalcrux");
    t.g.recompute();
    assert_eq!(t.pt(on_bf), (7, 7));
    assert_eq!(t.pt(in_hand), (7, 7));
    // An opponent's green permanents don't count.
    t.battlefield(P1, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(on_bf), (7, 7));
}

#[test]
fn wilt_leaf_liege_bonuses_are_cumulative_for_green_white_creatures() {
    cr!("613.4c");
    ruling!(
        "Wilt-Leaf Liege",
        "If another creature you control is both of the listed colors, it will get a total of +2/+2."
    );
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Wilt-Leaf Liege");
    let wolf = t.battlefield(P0, "Watchwolf");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let ogre = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Watchwolf");
    t.g.recompute();
    assert_eq!(t.pt(wolf), (5, 5));
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(angel), (5, 5));
    assert_eq!(t.pt(ogre), (3, 3));
    assert_eq!(t.pt(theirs), (3, 3));
    assert_eq!(t.pt(liege), (4, 4));
}

#[test]
fn force_of_savagery_survives_only_a_static_boost_or_an_entering_counter() {
    cr!("704.5f", "614.1c", "603.2");
    ruling!(
        "Force of Savagery",
        "A triggered or activated ability that boosts toughness won't have its effect fast enough to save it."
    );
    supported("Force of Savagery");
    supported("Grumgully, the Generous");
    supported("Juniper Order Ranger");
    // (helper on the battlefield, Force survives)
    for (helper, survives) in [
        (None, false),
        (Some("Glorious Anthem"), true),
        (Some("Grumgully, the Generous"), true),
        (Some("Juniper Order Ranger"), false),
    ] {
        let mut t = TestGame::new(2);
        if let Some(h) = helper {
            t.battlefield(P0, h);
        }
        t.enter(P0, "Force of Savagery");
        t.settle();
        assert_eq!(
            t.in_graveyard(P0, "Force of Savagery"),
            !survives,
            "{helper:?}"
        );
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Force of Savagery").len(),
            usize::from(survives),
            "{helper:?}"
        );
    }
}

// ---------------------------------------------------------------- total cost

#[test]
fn karador_reduction_applies_after_cost_increases() {
    cr!("601.2f", "202.3");
    ruling!(
        "Karador, Ghost Chieftain",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as the commander tax), then apply any cost reductions (such as that of Karador)."
    );
    supported("Sphere of Resistance");
    let colored = [(ManaType::W, 1), (ManaType::B, 1), (ManaType::G, 1)];
    let setup = |creatures: usize| {
        move |t: &mut TestGame| {
            t.battlefield(P1, "Sphere of Resistance");
            for _ in 0..creatures {
                t.graveyard(P0, "Grizzly Bears");
            }
        }
    };
    // {5} + {1} - 3 = {3}.
    assert_total_cost(&setup(3), P0, "Karador, Ghost Chieftain", &colored, 3);
    // {5} + {1} - 7 = {0} (applying the reduction first would leave {1}).
    assert!(casts_for(
        &setup(7),
        P0,
        "Karador, Ghost Chieftain",
        &colored,
        0
    ));
    // The spell's mana value is still 8.
    let mut t = TestGame::new(2);
    setup(7)(&mut t);
    let k = t.hand(P0, "Karador, Ghost Chieftain");
    for (ty, n) in colored {
        mana(&mut t, P0, ty, n);
    }
    let spell = t.cast(P0, k).go();
    assert_eq!(mana_value(&t, spell), 8);
}

#[test]
fn maelstrom_muse_reduction_applies_after_cost_increases() {
    cr!("601.2f", "202.3", "608.2h");
    ruling!(
        "Maelstrom Muse",
        "add any cost increases, then apply any cost reductions (such as that of Maelstrom Muse’s triggered ability). The mana value of the spell is determined only by its mana cost"
    );
    supported("Maelstrom Muse");
    // (Muse's power, Sphere of Resistance, castable for {U}): Divination's {2}{U}, plus
    // {1}, minus the power. Reducing first would make power 3 with Sphere cost {1}{U}.
    for (power, sphere, castable) in [
        (4, true, true),
        (3, true, true),
        (2, true, false),
        (2, false, true),
    ] {
        let mut t = TestGame::new(2);
        if sphere {
            t.battlefield(P1, "Sphere of Resistance");
        }
        let muse = t.battlefield(P0, "Maelstrom Muse");
        put_counters(&mut t, muse, counters::PLUS1, (power - 2) as u32);
        attack_with(&mut t, &[(muse, Entity::Player(P1))]);
        t.resolve_all();
        t.advance_to(P0, Step::PostcombatMain);
        let div = t.hand(P0, "Divination");
        mana(&mut t, P0, ManaType::U, 1);
        let r = t.cast(P0, div).try_go();
        assert_eq!(r.is_ok(), castable, "power {power} sphere {sphere}");
        if let Ok(spell) = r {
            assert_eq!(mana_value(&t, spell), 3);
        }
    }
}

#[test]
fn etherium_sculptor_reduces_only_generic_mana_and_not_mana_value() {
    cr!("601.2f", "118.7c", "202.3");
    ruling!(
        "Etherium Sculptor",
        "This effect can reduce only the generic portion of the artifact spell's total cost."
    );
    ruling!(
        "Etherium Sculptor",
        "This effect doesn't change the mana cost or mana value of an artifact spell."
    );
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Etherium Sculptor");
    };
    // Sol Ring {1} costs {0}; Tidehollow Sculler {W}{B} still costs {W}{B}.
    assert!(casts_for(&setup, P0, "Sol Ring", &[], 0));
    assert_total_cost(
        &setup,
        P0,
        "Tidehollow Sculler",
        &[(ManaType::W, 1), (ManaType::B, 1)],
        0,
    );
    assert!(!casts_for(
        &setup,
        P0,
        "Tidehollow Sculler",
        &[(ManaType::W, 1)],
        0
    ));
    // A nonartifact spell isn't reduced.
    assert_total_cost(&setup, P0, "Grizzly Bears", &[(ManaType::G, 1)], 1);
    let mut t = TestGame::new(2);
    setup(&mut t);
    let ring = t.hand(P0, "Sol Ring");
    let spell = t.cast(P0, ring).go();
    assert_eq!(mana_value(&t, spell), 1);
    assert_eq!(
        format!("{}", t.obj_now(spell).chars.mana_cost.clone().unwrap()),
        "{1}"
    );
}

// ---------------------------------------------------------------- Wastes in decks

fn cards(list: &[(&str, usize)]) -> Vec<Arc<CardDef>> {
    list.iter()
        .flat_map(|(n, k)| std::iter::repeat_n(card(n), *k))
        .collect()
}

#[test]
fn wastes_are_basic_lands_for_constructed_decks() {
    cr!("100.2a", "205.4c");
    ruling!(
        "Wastes",
        "Because Wastes is a basic land, you can include as many of them as you like in your Constructed decks."
    );
    ruling!(
        "Snow-Covered Wastes",
        "Because Snow-Covered Wastes is a basic land, you can include as many of them as you like in your Constructed decks."
    );
    for land in ["Wastes", "Snow-Covered Wastes"] {
        supported(land);
        assert!(check_constructed(&cards(&[("Grizzly Bears", 4), (land, 56)])).is_empty());
    }
    // A nonbasic colorless land is limited to four.
    assert!(!check_constructed(&cards(&[("Grizzly Bears", 4), ("Ghost Quarter", 56)])).is_empty());
}

#[test]
fn wastes_must_come_from_the_limited_card_pool() {
    cr!("100.2b");
    ruling!(
        "Wastes",
        "In Limited events (including Sealed Deck and Booster Draft), Wastes must be in your card pool to be included in your deck."
    );
    ruling!(
        "Snow-Covered Wastes",
        "In Limited events (including Sealed Deck and Booster Draft), Snow-Covered Wastes must be in your card pool to be included in your deck."
    );
    for land in ["Wastes", "Snow-Covered Wastes"] {
        let pool = cards(&[("Grizzly Bears", 23), (land, 2)]);
        let ok = cards(&[("Grizzly Bears", 23), (land, 2), ("Forest", 15)]);
        assert!(check_limited(&ok, &pool).is_empty(), "{land}");
        let too_many = cards(&[("Grizzly Bears", 23), (land, 17)]);
        assert_eq!(
            check_limited(&too_many, &pool),
            vec![DeckProblem::NotInPool {
                name: land.into(),
                have: 17,
                available: 2,
            }],
            "{land}"
        );
    }
}
