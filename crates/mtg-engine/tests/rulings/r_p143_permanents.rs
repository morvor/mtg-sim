//! Rulings batch P143 — static and activated abilities of card-advantage permanents:
//! Carth the Lion's loyalty cost change, Ajani's distributed counters, Vivien's Talent,
//! Kinnan's extra mana, Powerstones, Krallenhorde Howler's cost reduction, Satoru
//! Umezawa's ninjutsu, Forensic Gadgeteer, switching power and toughness, Voltaic Servant
//! and Invasion of Ravnica.

use crate::r_p107_common::{act, mana};
use crate::r_p108_common::put_counters;
use crate::r_p109_common::{divide, division_of};
use crate::r_p143_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s02_common::{can_activate, can_cast, destroy, target_candidates};
use crate::r_s04_common::can_cycle;
use crate::r_s06_common::attach_new;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn loyalty(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::LOYALTY)
}

/// Activates the loyalty ability of `pw` containing `needle` and resolves the stack.
fn loyalty_ability(t: &mut TestGame, pw: ObjectId, needle: &str, targets: &[Entity]) {
    t.set_step(P0, Step::PrecombatMain);
    act(t, P0, pw, needle, targets).unwrap();
    t.resolve_all();
}

#[test]
fn carth_adds_one_to_each_loyalty_cost() {
    cr!("606.4", "118.7");
    supported("Carth the Lion");
    ruling!(
        "Carth the Lion",
        "If a planeswalker's loyalty ability normally has a cost of [+1], Carth's ability makes it cost [+2] instead. A cost of [0] would become [+1], and a cost of [-6] would become [-5]."
    );
    // Karn, Living Legacy (4): +1 becomes +2, −1 becomes 0 and −7 becomes −6.
    for (needle, start, want) in [
        ("Create a tapped Powerstone", 4, 6),
        ("Pay any amount of mana", 4, 4),
        ("You get an emblem", 7, 1),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Carth the Lion");
        let karn = t.battlefield(P0, "Karn, Living Legacy");
        put_counters(&mut t, karn, counters::LOYALTY, start - 4);
        loyalty_ability(&mut t, karn, needle, &[]);
        assert_eq!(loyalty(&t, karn), want, "{needle}");
    }
    // Jace, the Mind Sculptor's 0 becomes +1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carth the Lion");
    let jace = t.battlefield(P0, "Jace, the Mind Sculptor");
    let before = loyalty(&t, jace);
    loyalty_ability(&mut t, jace, "Draw three cards", &[]);
    assert_eq!(loyalty(&t, jace), before + 1);
}

#[test]
fn carth_and_counter_doubling_and_two_carths() {
    cr!("606.4", "614.1a", "118.7");
    // Vorinclex, Monstrous Raider: "If you would put one or more counters on a permanent
    // or player, put twice that many of each of those kinds of counters on that permanent
    // or player instead."
    supported("Vorinclex, Monstrous Raider");
    ruling!(
        "Carth the Lion",
        "If an effect replaces the number of counters that would be placed on a planeswalker, such as that of Vorinclex, Monstrous Raider, that replacement happens only once, at the time payment is made."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carth the Lion");
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let karn = t.battlefield(P0, "Karn, Living Legacy");
    let before = loyalty(&t, karn);
    loyalty_ability(&mut t, karn, "Create a tapped Powerstone", &[]);
    // +1 becomes +2, doubled once: +4.
    assert_eq!(loyalty(&t, karn), before + 4);

    ruling!(
        "Carth the Lion",
        "If you somehow manage to control two Carths (perhaps because of Spark Double), the cost-changing effect is cumulative. In total, loyalty abilities will cost an additional [+2] to activate."
    );
    let mut t = TestGame::new(2);
    let carth = t.battlefield(P0, "Carth the Lion");
    t.answer_choose(P0, &[obj(carth)]);
    t.enter(P0, "Spark Double");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Carth the Lion").len(), 2);
    let karn = t.battlefield(P0, "Karn, Living Legacy");
    let before = loyalty(&t, karn);
    loyalty_ability(&mut t, karn, "Create a tapped Powerstone", &[]);
    assert_eq!(loyalty(&t, karn), before + 3);
}

#[test]
fn carth_nets_the_loyalty_cost_before_paying_it() {
    cr!("606.4", "118.7");
    ruling!(
        "Carth the Lion",
        "The total cost of a planeswalker's loyalty ability is calculated before any counters are added or removed."
    );
    // Liliana of the Veil (3)'s −2 becomes −1: one counter is removed, once; nothing is
    // put on her.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carth the Lion");
    let lili = t.battlefield(P0, "Liliana of the Veil");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.g.recompute();
    t.g.flush_events();
    let uid = t
        .g
        .obj(lili)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("Target player sacrifices"))
        .unwrap()
        .uid;
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, lili, uid).unwrap();
    let moved: Vec<(bool, u32)> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::CountersAdded { kind, n, .. } if kind.as_str() == counters::LOYALTY => {
                Some((true, *n))
            }
            Event::CountersRemoved { kind, n, .. } if kind.as_str() == counters::LOYALTY => {
                Some((false, *n))
            }
            _ => None,
        })
        .collect();
    assert_eq!(moved, vec![(false, 1)]);
    assert_eq!(loyalty(&t, lili), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(target));
}

#[test]
fn ajani_distributes_counters_as_it_is_activated() {
    cr!("601.2d", "602.2b", "608.2b");
    supported("Ajani, Mentor of Heroes");
    ruling!(
        "Ajani, Mentor of Heroes",
        "You announce how many counters will be placed on each target creature as you activate Ajani's first ability. Each target must receive at least one counter."
    );
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Mentor of Heroes");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Memnite");
    t.set_step(P0, Step::PrecombatMain);
    divide(&mut t, P0, &[2, 1]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let from = n_asked(&t);
    let ab = act(&mut t, P0, ajani, "Distribute three", &[])
        .unwrap()
        .unwrap();
    // A Divide decision with a minimum of one each, while activating.
    let min_each: Vec<u32> = asked_of(&t, P0, from)
        .into_iter()
        .filter_map(|d| match d {
            mtg_engine::decision::Decision::Divide { min_each, .. } => Some(min_each),
            _ => None,
        })
        .collect();
    assert_eq!(min_each, vec![1]);
    assert_eq!(division_of(&t, ab), vec![2, 1]);

    ruling!(
        "Ajani, Mentor of Heroes",
        "If one or more, but not all, of the targets of the first ability are illegal when that ability tries to resolve, no counters will be placed on illegal targets. You can't change the distribution you announced when you activated the ability; the excess counters are just lost."
    );
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
}

#[test]
fn viviens_talent_ability_shares_the_once_per_turn_limit() {
    cr!("606.3");
    supported("Vivien's Talent");
    ruling!(
        "Vivien's Talent",
        "You can activate one loyalty ability of each planeswalker you control during each of your turns, whether that ability is one it always has or the one it gains because of Vivien's Talent."
    );
    for granted_first in [true, false] {
        let mut t = TestGame::new(2);
        let karn = t.battlefield(P0, "Karn, Living Legacy");
        attach_new(&mut t, P0, "Vivien's Talent", obj(karn));
        let (first, second) = if granted_first {
            ("Look at the top four", "Create a tapped Powerstone")
        } else {
            ("Create a tapped Powerstone", "Look at the top four")
        };
        loyalty_ability(&mut t, karn, first, &[]);
        assert!(act(&mut t, P0, karn, second, &[]).is_err());
    }
}

#[test]
fn kinnan_adds_unrestricted_mana_of_its_own() {
    cr!("605.1b", "106.6");
    supported("Kinnan, Bonder Prodigy");
    ruling!(
        "Kinnan, Bonder Prodigy",
        "Kinnan doesn’t care about any restrictions or riders the nonland permanent put on the mana it produced. The additional mana Kinnan produces won’t have any restrictions or riders."
    );
    ruling!(
        "Kinnan, Bonder Prodigy",
        "The additional mana is produced by Kinnan, not by the nonland permanent that you tapped for mana."
    );
    let mut t = TestGame::new(2);
    let kinnan = t.battlefield(P0, "Kinnan, Bonder Prodigy");
    let stone = crate::r_s02_common::create_token(&mut t, P0, "Powerstone");
    act(&mut t, P0, stone, "Add {C}", &[]).unwrap();
    t.resolve_all();
    let pool = &t.g.player(P0).mana_pool.mana;
    assert_eq!(pool.len(), 2, "{pool:?}");
    let from_stone = pool.iter().find(|m| m.source == Some(stone)).expect("Powerstone mana");
    assert!(from_stone.restriction.is_some());
    let from_kinnan = pool.iter().find(|m| m.source == Some(kinnan)).expect("Kinnan mana");
    assert_eq!(from_kinnan.ty, ManaType::C);
    assert!(from_kinnan.restriction.is_none() && from_kinnan.rider.is_none());
    // The unrestricted {C} (with a {G}) pays for Grizzly Bears.
    mana(&mut t, P0, ManaType::G, 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn powerstone_tokens() {
    cr!("111.10s", "106.6");
    ruling!(
        "Karn, Living Legacy",
        "Powerstone tokens are a new kind of predefined token. Each one has the artifact subtype “Powerstone” and the ability “{T}: Add {C}. This mana can’t be spent to cast a nonartifact spell.”"
    );
    ruling!(
        "The Mana Rig",
        "Powerstone tokens are a new kind of predefined token. Each one has the artifact subtype “Powerstone” and the ability “{T}: Add {C}. This mana can’t be spent to cast a nonartifact spell.”"
    );
    supported("Karn, Living Legacy");
    supported("The Mana Rig");
    let check = |t: &mut TestGame| {
        let stones = with_subtype(t, P0, "Powerstone");
        assert_eq!(stones.len(), 1);
        let s = stones[0];
        assert!(t.obj_now(s).is(mtg_engine::types::CardType::Artifact));
        assert!(t.obj_now(s).tapped);
        t.g.obj_mut(s).tapped = false;
        act(t, P0, s, "Add {C}", &[]).unwrap();
        // {C} from it can't pay for Grizzly Bears' generic mana, but pays for Sol Ring.
        mana(t, P0, ManaType::G, 1);
        let bears = t.hand(P0, "Grizzly Bears");
        assert!(t.cast(P0, bears).try_go().is_err());
        assert!(is_in(t, bears, Zone::Hand(P0)));
        // Only the Powerstone's {C} is left to pay for Sol Ring.
        t.g.players[0].mana_pool.mana.retain(|m| m.ty == ManaType::C);
        assert_eq!(t.g.player(P0).mana_pool.mana.len(), 1);
        let ring = t.hand(P0, "Sol Ring");
        t.cast(P0, ring).go();
        t.resolve_all();
        assert!(t.on_battlefield(ring));
    };
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Living Legacy");
    loyalty_ability(&mut t, karn, "Create a tapped Powerstone", &[]);
    check(&mut t);
    // The Mana Rig: "Whenever you cast a multicolored spell, create a tapped Powerstone
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Mana Rig");
    cast_new(&mut t, P0, "Maestros Diabolist", &[]);
    t.resolve_all();
    check(&mut t);
}

#[test]
fn krallenhorde_howler_reduces_generic_mana_including_madness() {
    cr!("601.2f", "118.7d", "702.35");
    supported("Duskwatch Recruiter // Krallenhorde Howler");
    ruling!(
        "Duskwatch Recruiter // Krallenhorde Howler",
        "Krallenhorde Howler's first ability can only reduce the generic mana portion of a creature spell's cost."
    );
    let mut t = TestGame::new(2);
    let howler = t.battlefield(P0, "Duskwatch Recruiter");
    mtg_engine::dfc::transform(&mut t.g, howler);
    t.g.recompute();
    assert_eq!(t.obj_now(howler).chars.name.as_str(), "Krallenhorde Howler");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    let bears = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));

    ruling!(
        "Duskwatch Recruiter // Krallenhorde Howler",
        "Krallenhorde Howler's first ability can reduce alternative costs such as madness costs."
    );
    // Arrogant Wurm: madness {2}{G}, paid with two Forests.
    supported("Arrogant Wurm");
    let mut t = TestGame::new(2);
    let howler = t.battlefield(P0, "Duskwatch Recruiter");
    mtg_engine::dfc::transform(&mut t.g, howler);
    t.lands(P0, "Forest", 2);
    let wurm = t.hand(P0, "Arrogant Wurm");
    t.g.discard(P0, wurm, None);
    t.g.flush_events();
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(wurm), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
}

#[test]
fn duskwatch_recruiter_can_respond_to_its_own_trigger() {
    cr!("603.3", "602.2");
    ruling!(
        "Duskwatch Recruiter // Krallenhorde Howler",
        "You can activate Duskwatch Recruiter's first ability in response to its triggered ability."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Duskwatch Recruiter");
    t.lands(P0, "Forest", 3);
    let uid = t
        .g
        .obj(d)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("Look at the top three"))
        .unwrap()
        .uid;
    // No spells were cast in P0's turn: it triggers in P1's upkeep, and P0 activates the
    // look in response (with the trigger on the stack).
    answer_once_when(
        &mut t,
        P0,
        |g, d| {
            matches!(d, mtg_engine::decision::Decision::Priority { .. })
                && g.turn.step == Step::Upkeep
                && g.stack.len() == 1
        },
        mtg_engine::decision::Answer::Action(mtg_engine::decision::Action::Activate {
            source: d,
            ability: uid,
        }),
    );
    let lib = set_library(&mut t, P0, &["Grizzly Bears", "Island", "Swamp"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.advance_to(P1, Step::Draw);
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
    let log = t.dump_log();
    let activated = log.find("P0 activates Duskwatch Recruiter").expect("activated");
    let transformed = log.rfind("Resolving Duskwatch Recruiter").expect("trigger resolved");
    assert!(activated < transformed, "{log}");
    assert_eq!(t.obj_now(d).chars.name.as_str(), "Krallenhorde Howler");
}

#[test]
fn satoru_umezawa_and_ninjutsu() {
    cr!("702.49a", "702.49c", "603.3");
    supported("Satoru Umezawa");
    ruling!(
        "Satoru Umezawa",
        "Satoru Umezawa's triggered ability triggers whenever you activate a ninjustsu ability, and it will resolve before that ability resolves. The card with ninjustsu will still be in your hand while resolving the ability"
    );
    ruling!(
        "Satoru Umezawa",
        "Once you activate a ninjutsu ability, no player can affect that ability by removing Satoru Umezawa from the battlefield. Even if Satoru leaves the battlefield before it resolves, you'll still be able to put the creature onto the battlefield tapped and attacking."
    );
    let mut t = TestGame::new(2);
    let satoru = t.battlefield(P0, "Satoru Umezawa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    crate::r_s09_common::declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::DeclareBlockers);
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Memnite", "Plains"]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, giant, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // Satoru's trigger resolves first, with the Giant still in hand.
    t.answer_choose(P0, &[obj(lib[2])]);
    t.resolve();
    assert!(is_in(&t, lib[2], Zone::Hand(P0)));
    assert!(is_in(&t, giant, Zone::Hand(P0)));
    assert_eq!(t.stack_len(), 1);
    // Satoru leaves; the ninjutsu ability still puts the Giant onto the battlefield
    // attacking.
    destroy(&mut t, satoru);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(t.obj_now(giant).tapped);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn forensic_gadgeteer_reduces_only_activated_abilities_on_the_battlefield() {
    cr!("602.1", "601.2f");
    supported("Forensic Gadgeteer");
    ruling!(
        "Forensic Gadgeteer",
        "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by the cost reduction ability of Forensic Gadgeteer."
    );
    // A Clue's {2} costs {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forensic Gadgeteer");
    let clue = crate::r_s02_common::create_token(&mut t, P0, "Clue");
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_activate(&mut t, P0, clue));
    // A triggered "you may pay {2}" still costs {2}.
    let c = custom_card(
        "Toll Engine",
        "Artifact",
        "{1}",
        None,
        "At the beginning of your upkeep, you may pay {2}. If you do, you gain 5 life.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forensic Gadgeteer");
    t.custom(P0, c, Zone::Battlefield);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);

    ruling!(
        "Forensic Gadgeteer",
        "The costs of activated abilities of artifact cards that work in other zones, such as cycling, won't be reduced."
    );
    // Indatha Crystal: "Cycling {2}".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forensic Gadgeteer");
    let crystal = t.hand(P0, "Indatha Crystal");
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cycle(&mut t, P0, crystal));
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cycle(&mut t, P0, crystal));
}

#[test]
fn switching_power_and_toughness_applies_last() {
    cr!("613.4d", "613.7");
    supported("Merfolk Thaumaturgist");
    supported("Twisted Reflection");
    ruling!(
        "Merfolk Thaumaturgist",
        "Effects that switch a creature’s power and toughness apply after all other effects, regardless of when those effects began to apply. For instance, if you target a 1/2 creature then give it +2/+0 later in the turn, it’s a 2/3 creature, not a 4/1 creature."
    );
    ruling!(
        "Twisted Reflection",
        "Effects that switch a creature’s power and toughness apply after all other effects, regardless of when those effects began to apply."
    );
    // Merfolk Thaumaturgist: "{T}: Switch target creature's power and toughness until end
    // of turn." Twisted Reflection's second mode does the same.
    for spell in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Merfolk Thaumaturgist");
        if spell {
            t.lands(P0, "Island", 2);
            let tr = t.hand(P0, "Twisted Reflection");
            t.cast(P0, tr).modes(&[1]).target(m).go();
        } else {
            act(&mut t, P0, m, "Switch", &[obj(m)]).unwrap();
        }
        t.resolve_all();
        assert_eq!(t.pt(m), (2, 1));
        crate::r_p117_common::modify(&mut t, m, vec![Modification::ModifyPT(Value::c(2), Value::c(0))]);
        assert_eq!(t.pt(m), (2, 3), "spell: {spell}");
    }

    ruling!(
        "Merfolk Thaumaturgist",
        "Switching a creature’s power and toughness twice (or any even number of times) effectively returns the creature to the power and toughness it had before any switches."
    );
    ruling!(
        "Twisted Reflection",
        "Switching a creature’s power and toughness twice (or any even number of times) effectively returns the creature to the power and toughness it had before any switches."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merfolk Thaumaturgist");
    let bears = t.battlefield(P0, "Hill Giant");
    crate::r_p117_common::modify(&mut t, bears, vec![Modification::ModifyPT(Value::c(1), Value::c(0))]);
    assert_eq!(t.pt(bears), (4, 3));
    act(&mut t, P0, m, "Switch", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 4));
    t.lands(P0, "Island", 2);
    let tr = t.hand(P0, "Twisted Reflection");
    t.cast(P0, tr).modes(&[1]).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 3));
}

#[test]
fn table_of_target_rulings() {
    cr!("115.1", "115.2");
    supported("Voltaic Servant");
    ruling!(
        "Voltaic Servant",
        "Voltaic Servant’s ability can target an artifact creature, including itself."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Voltaic Servant");
    t.g.obj_mut(v).tapped = true;
    t.answer_targets(P0, &[obj(v)]);
    let from = n_asked(&t);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(v))));
    assert!(!t.obj_now(v).tapped);

    supported("Invasion of Ravnica // Guildpact Paragon");
    ruling!(
        "Invasion of Ravnica // Guildpact Paragon",
        "A nonland permanent that isn’t exactly two colors may be colorless, monocolored, or have three or more colors."
    );
    let mut t = TestGame::new(2);
    let colorless = t.battlefield(P1, "Ornithopter");
    let mono = t.battlefield(P1, "Grizzly Bears");
    let two = t.battlefield(P1, "Kinnan, Bonder Prodigy");
    let three = t.battlefield(P1, "Maestros Diabolist");
    let land = t.battlefield(P1, "Island");
    t.answer_targets(P0, &[obj(three)]);
    let from = n_asked(&t);
    t.enter(P0, "Invasion of Ravnica");
    t.resolve_all();
    let cands: Vec<Entity> = target_candidates(&t, P0, from).concat();
    for (id, ok) in [(colorless, true), (mono, true), (three, true), (two, false), (land, false)] {
        assert_eq!(cands.contains(&obj(id)), ok, "{:?}", t.g.obj(id).chars.name);
    }
    assert!(is_in(&t, three, Zone::Exile));
}
