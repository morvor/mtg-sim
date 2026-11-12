//! Rulings batch P138 — "choose some permanents, then sacrifice the rest" (Cataclysm,
//! Cataclysmic Gearhulk, Ajani, Nacatl Avenger, Slaughter the Strong), "sacrifices N
//! creatures" with fewer than N (Abolisher of Bloodlines), Tyrant of Discord's repeated
//! random sacrifices, and Awaken the Blood Avatar.

use crate::r_p138_common::*;
use crate::r_s13_common::{commander, commander_game};
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
const EXTUS: &str = "Extus, Oriq Overlord // Awaken the Blood Avatar";

#[test]
fn cataclysm_a_permanent_with_two_types_represents_either() {
    cr!("101.4", "205.2a");
    ruling!(
        "Cataclysm",
        "If you control a permanent with more than one type, you can choose that permanent to be the representative of either type. This makes it possible to have more than one creature (or other permanent type) on the battlefield."
    );
    supported("Cataclysm");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let plains = t.battlefield(P0, "Plains");
    // Artifact: Ornithopter. Creature: Grizzly Bears. Enchantment, land.
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(anthem)]);
    t.answer_choose(P0, &[obj(plains)]);
    cast(&mut t, P0, "Cataclysm", &[]);
    t.resolve_all();
    for kept in [thopter, bears, anthem, plains] {
        assert!(t.on_battlefield(kept));
    }
    assert!(!t.on_battlefield(giant));
    assert_eq!(creatures(&t, P0).len(), 2);
}

#[test]
fn gearhulk_and_ajani_lands_arent_chosen_or_sacrificed_and_missing_types_are_skipped() {
    cr!("101.4", "205.2a");
    ruling!(
        "Cataclysmic Gearhulk",
        "Lands can't be chosen and won't be sacrificed, even if they have other types (such as an artifact land)."
    );
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "Lands can't be chosen and won't be sacrificed, even if they have any of the types referenced by Ajani, Nacatl Avenger's last ability."
    );
    ruling!(
        "Cataclysmic Gearhulk",
        "If you don't control any permanents of one of the types, you'll still choose the permanents of the types you do control."
    );
    for gearhulk in [true, false] {
        let mut t = TestGame::new(2);
        let ajani = if gearhulk {
            None
        } else {
            supported(AJANI);
            let card = t.graveyard(P0, AJANI);
            let a = put_onto_battlefield(&mut t, P0, card, true).expect("Ajani entered");
            let have = t.counters(a, counters::LOYALTY);
            t.g.add_counters(Entity::Object(a), counters::LOYALTY, 4 - have, None);
            Some(a)
        };
        // P1: an artifact land, a basic, an artifact, two creatures; no enchantment or
        // planeswalker.
        let den = t.battlefield(P1, "Ancient Den");
        let forest = t.battlefield(P1, "Forest");
        let ring = t.battlefield(P1, "Sol Ring");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_choose(P1, &[obj(ring)]);
        t.answer_choose(P1, &[obj(giant)]);
        let from = t.asked().len();
        match ajani {
            None => {
                let g = t.battlefield(P0, "Grizzly Bears");
                t.answer_choose(P0, &[obj(g)]);
                t.enter(P0, "Cataclysmic Gearhulk");
                t.settle();
            }
            Some(a) => {
                t.activate(P0, a, 2, &[]).expect("−4");
            }
        }
        t.resolve_all();
        let what = if gearhulk { "Gearhulk" } else { "Ajani" };
        let offer = first_offer(&t, P1, from);
        assert!(offer.contains(&ring) && !offer.contains(&den), "{what}: {offer:?}");
        for kept in [den, forest, ring, giant] {
            assert!(t.on_battlefield(kept), "{what}");
        }
        assert!(!t.on_battlefield(bears), "{what}");
        // P1 was asked only for the types they control (artifact, creature).
        let p1_choices = choosers_since(&t, from)
            .into_iter()
            .filter(|p| *p == P1)
            .count();
        assert_eq!(p1_choices, 2, "{what}");
    }
}

#[test]
fn slaughter_the_strong_checks_the_total_power_of_the_creatures_chosen() {
    cr!("101.4", "608.2d");
    ruling!(
        "Slaughter the Strong",
        "Slaughter the Strong has each player choose any number of creatures and then checks that the total power of creatures each player chose this way is 4 or less. For example, you could save two 2/2 creatures, or a 1/1 and a 3/3 creature, but not all four of those creatures."
    );
    supported("Slaughter the Strong");
    // P1 keeps (a) the two 2/2s, (b) the 1/1 and the 3/3; (c) all four can't be kept.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P1, "Grizzly Bears");
        let one = t.battlefield(P1, "Memnite");
        let three = t.battlefield(P1, "Hill Giant");
        let keep = match case {
            0 => vec![b1, b2],
            1 => vec![one, three],
            _ => vec![b1, b2, one, three],
        };
        t.answer_choose(P1, &keep.iter().map(|id| obj(*id)).collect::<Vec<_>>());
        cast(&mut t, P0, "Slaughter the Strong", &[]);
        t.resolve_all();
        let left = creatures(&t, P1);
        let power: i32 = left.iter().map(|id| t.obj(*id).power()).sum();
        assert!(power <= 4, "case {case}: {power}");
        if case < 2 {
            assert!(keep.iter().all(|k| t.on_battlefield(*k)), "case {case}");
            assert_eq!(left.len(), 2, "case {case}");
        } else {
            assert!(left.len() < 4, "case {case}");
        }
    }
}

#[test]
fn abolisher_opponent_with_fewer_than_three_sacrifices_all() {
    cr!("701.21a", "701.28a");
    ruling!(
        "Voldaren Pariah // Abolisher of Bloodlines",
        "If the targeted opponent doesn't control three creatures as the triggered ability of Abolisher of Bloodlines resolves, that player sacrifices as many creatures as able."
    );
    let name = "Voldaren Pariah // Abolisher of Bloodlines";
    supported(name);
    let mut t = TestGame::new(2);
    let pariah = t.battlefield(P0, name);
    let fodder: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let theirs = [
        t.battlefield(P1, "Hill Giant"),
        t.battlefield(P1, "Gray Ogre"),
    ];
    t.answer_choose(P0, &fodder.iter().map(|id| obj(*id)).collect::<Vec<_>>());
    t.answer_targets(P0, &[pl(P1)]);
    t.activate(P0, pariah, 0, &[]).expect("transform");
    t.resolve_all();
    assert_eq!(t.obj_now(pariah).chars.name, "Abolisher of Bloodlines");
    assert!(theirs.iter().all(|c| !t.on_battlefield(*c)));
}

#[test]
fn tyrant_of_discord_repeats_for_the_same_opponent_and_holds_triggers() {
    cr!("608.2", "603.3", "603.3b", "101.4");
    ruling!(
        "Tyrant of Discord",
        "Repeating the process does not include targeting an opponent. Only one opponent will have to sacrifice permanents."
    );
    ruling!(
        "Tyrant of Discord",
        "If any abilities trigger while Tyrant of Discord's \"enters\" ability is resolving, those abilities will wait until the \"enters\" ability finishes resolving before going on the stack. Starting with the active player, each player puts their abilities on the stack in any order."
    );
    supported("Tyrant of Discord");
    let mut t = TestGame::new(3);
    // P0's It That Betrays triggers on each of P1's sacrifices; P1's Doomed Traveler
    // triggers as it dies. P1 has only nonland permanents: all are sacrificed.
    t.battlefield(P0, "It That Betrays");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let traveler = t.battlefield(P1, "Doomed Traveler");
    let p2 = [t.battlefield(P2, "Hill Giant"), t.battlefield(P2, "Forest")];
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Tyrant of Discord");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // Both were sacrificed during the one resolution; the triggers waited and then went
    // on the stack in APNAP order: P1's (nonactive) last, on top.
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(traveler));
    assert_eq!(t.stack_len(), 3);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P1);
    assert!(p2.iter().all(|c| t.on_battlefield(*c)));
    t.resolve_all();
    assert!(p2.iter().all(|c| t.on_battlefield(*c)));
}

#[test]
fn tyrant_of_discord_with_sigarda_chooses_but_doesnt_sacrifice() {
    cr!("701.21a", "608.2b");
    ruling!(
        "Tyrant of Discord",
        "If the target opponent can't sacrifice permanents (perhaps because of Sigarda, Host of Herons), the opponent will choose a permanent at random but not sacrifice it. The ability will then finish resolving."
    );
    let mut t = TestGame::new(2);
    let mine = [
        t.battlefield(P1, "Sigarda, Host of Herons"),
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Forest"),
    ];
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Tyrant of Discord");
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(mine.iter().all(|c| t.on_battlefield(*c)));
}

#[test]
fn awaken_the_blood_avatar_creates_the_avatar_with_no_creatures_to_sacrifice() {
    cr!("608.2c", "111.1");
    ruling!(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        "The Blood Avatar will be awakened even if no player has creatures to sacrifice when it resolves."
    );
    supported(EXTUS);
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    lots(&mut t, P0);
    let c = t.hand(P0, EXTUS);
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let avatars = t.named_on_battlefield("Avatar Token");
    let avatars = if avatars.is_empty() {
        permanents_of(&t, P0)
            .into_iter()
            .filter(|id| t.obj(*id).chars.has_subtype("Avatar"))
            .collect()
    } else {
        avatars
    };
    assert_eq!(avatars.len(), 1);
    assert_eq!(t.pt(avatars[0]), (3, 6));
}

#[test]
fn awaken_the_blood_avatar_reduction_applies_after_the_commander_tax() {
    cr!("601.2f", "903.8");
    ruling!(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        "Include any additional costs to cast Awaken the Blood Avatar before applying its own cost reduction. For example, if Extus is your commander, and an additional cost of {6} applies from the “commander tax,” you may cast Awaken the Blood Avatar by sacrificing four creatures and paying {4}{B}{R}."
    );
    // Commander tax {6} (cast three times before); four creatures sacrificed: {8} less.
    for (generic, castable) in [(4u32, true), (3, false)] {
        let mut t = commander_game();
        let extus = commander(&mut t, P0, EXTUS);
        let key = mtg_engine::kw::partner::commander_key(&t.g, extus);
        t.g.players[0].commander_casts.insert(key, 3);
        let fodder: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        t.answer(P0, DecisionKind::OptionalCost, decision::Answer::Number(4));
        t.answer_choose(P0, &fodder.iter().map(|id| obj(*id)).collect::<Vec<_>>());
        let pool = &mut t.g.players[0].mana_pool;
        pool.add_type(ManaType::B, 1);
        pool.add_type(ManaType::R, 1);
        pool.add_type(ManaType::C, generic);
        t.g.turn.priority = Some(P0);
        let r = t.g.cast_spell(P0, extus, CastMethod::Half(1));
        assert_eq!(r.is_ok(), castable, "{{{generic}}}{{B}}{{R}}: {r:?}");
        if castable {
            assert!(fodder.iter().all(|f| !t.on_battlefield(*f)));
            assert_eq!(t.g.player(P0).mana_pool.total(), 0);
            assert_eq!(t.zone(extus), Zone::Stack);
        }
    }
}
