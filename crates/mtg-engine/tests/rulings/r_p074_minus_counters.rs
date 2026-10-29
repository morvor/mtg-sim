//! Rulings batch P074 — -1/-1 counters: mandatory triggered targets (Contagion Clasp,
//! Grief Tyrant, Blowfly Infestation, Cultbrand Cinder), targets that become illegal
//! (Serrated Biskelion, Splendid Agony, Dust, Torment of Venom), Everlasting Torment's
//! wither for all damage, last known toughness (Massacre Girl), Pyrrhic Revival's 0-toughness
//! creatures, and Serrated Arrows' intervening "if".

use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s03_common::respond;
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, attach_new, give_control};
use crate::r_s13_common::add;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MM: &str = "-1/-1";

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    let mut cb = t.cast(p, c);
    for e in targets {
        cb = cb.target(*e);
    }
    cb.go();
    t.resolve_all();
}

#[test]
fn mandatory_triggers_must_target_your_own_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Contagion Clasp",
        "Contagion Clasp's first ability is mandatory. If you're the only player who controls a creature, you must target one of them."
    );
    ruling!(
        "Grief Tyrant",
        "The triggered ability is mandatory. If you’re the only player who controls any creatures when it triggers, you must target one of them."
    );
    ruling!(
        "Blowfly Infestation",
        "This ability is mandatory. If you're the only player who controls any creatures when this ability triggers, you must target one of them."
    );
    ruling!(
        "Cultbrand Cinder",
        "If there are no other creatures on the battlefield when Cultbrand Cinder enters, it must target itself."
    );
    for name in ["Grief Tyrant", "Blowfly Infestation"] {
        supported(name);
    }
    // Contagion Clasp.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    enter(&mut t, P0, "Contagion Clasp");
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 1);
    // Grief Tyrant dies with four -1/-1 counters: four on the Wurm.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let tyrant = enter(&mut t, P0, "Grief Tyrant");
    assert_eq!(t.counters(tyrant, MM), 4);
    destroy(&mut t, tyrant);
    t.resolve_all();
    // A 6/4 with four -1/-1 counters dies.
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    // Blowfly Infestation: a creature with a -1/-1 counter dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blowfly Infestation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    add(&mut t, obj(bears), MM, 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 1);
    // Cultbrand Cinder, alone.
    let mut t = TestGame::new(2);
    let cinder = enter(&mut t, P0, "Cultbrand Cinder");
    t.resolve_all();
    assert_eq!(t.counters(cinder, MM), 1);
}

#[test]
fn snuffers_and_snapper_put_one_counter_each() {
    cr!("122.1", "608.2c");
    ruling!(
        "Soul Snuffers",
        "The ability puts a -1/-1 counter on Soul Snuffers, too."
    );
    ruling!(
        "Lockjaw Snapper",
        "Each of those creatures will get just one new -1/-1 counter, regardless of how many it already has."
    );
    supported("Soul Snuffers");
    supported("Lockjaw Snapper");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let snuffers = enter(&mut t, P0, "Soul Snuffers");
    t.resolve_all();
    assert_eq!((t.counters(snuffers, MM), t.counters(wurm, MM)), (1, 1));
    // Lockjaw Snapper dies: the Wurm (2 counters) and Snuffers (1) get one more each;
    // the Hill Giant (none) gets none.
    let giant = t.battlefield(P1, "Hill Giant");
    add(&mut t, obj(wurm), MM, 1);
    let snapper = t.battlefield(P0, "Lockjaw Snapper");
    destroy(&mut t, snapper);
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 3);
    assert_eq!(t.counters(snuffers, MM), 2);
    assert_eq!(t.counters(giant, MM), 0);
}

#[test]
fn grim_poppet_targets_another_creature_repeatedly() {
    cr!("115.1", "602.2", "109.5");
    ruling!(
        "Grim Poppet",
        "The second ability can target any creature except the Grim Poppet whose ability is being activated. It can target a different Grim Poppet. It can be activated multiple times targeting the same creature."
    );
    supported("Grim Poppet");
    let mut t = TestGame::new(2);
    let p1 = t.battlefield(P0, "Grim Poppet");
    let p2 = t.battlefield(P0, "Grim Poppet");
    add(&mut t, obj(p1), MM, 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, p1, "another target").unwrap();
    t.resolve_all();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!cands[0].contains(&obj(p1)));
    assert!(cands[0].contains(&obj(p2)) && cands[0].contains(&obj(wurm)));
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, p1, "another target").unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[obj(p2)]);
    activate_containing(&mut t, P0, p1, "another target").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 2);
    assert_eq!(t.counters(p2, MM), 1);
    assert_eq!(t.counters(p1, MM), 0);
}

#[test]
fn serrated_biskelion_leaving_vs_its_target_leaving() {
    cr!("113.7a", "608.2b");
    ruling!(
        "Serrated Biskelion",
        "If Serrated Biskelion leaves the battlefield before its ability resolves, you’ll still put a -1/-1 counter on the targeted creature. If the target leaves the battlefield or otherwise becomes an illegal target before the ability resolves, the ability doesn’t resolve and you won’t put a -1/-1 counter on Serrated Biskelion."
    );
    supported("Serrated Biskelion");
    let mut t = TestGame::new(2);
    let bisk = t.battlefield(P0, "Serrated Biskelion");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, bisk, "target creature").unwrap();
    destroy(&mut t, bisk);
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 1);
    let mut t = TestGame::new(2);
    let bisk = t.battlefield(P0, "Serrated Biskelion");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, bisk, "target creature").unwrap();
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.counters(bisk, MM), 0);
}

#[test]
fn splendid_agony_lost_counter_and_dust_partial_targets() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Splendid Agony",
        "If one of two target creatures becomes an illegal target in response to Splendid Agony, the -1/-1 counter that would have been put on that creature is lost. It can’t be put on the legal target."
    );
    ruling!(
        "Grind // Dust",
        "If one of Dust's target creatures loses its -1/-1 counters, leaves the battlefield, or otherwise becomes an illegal target before the spell resolves, it won't be exiled, but the remaining legal targets will be exiled."
    );
    supported("Splendid Agony");
    supported("Grind // Dust");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Splendid Agony");
    let sa = t.hand(P0, "Splendid Agony");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P0, sa).targets(&[obj(wurm), obj(giant)]).go();
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(wurm, MM), 1);
    // Dust, from the graveyard: the Wurm loses its counter in response.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for c in [wurm, giant, bears] {
        add(&mut t, obj(c), MM, 1);
    }
    let gd = t.graveyard(P0, "Grind // Dust");
    t.lands(P0, "Plains", 4);
    t.cast(P0, gd)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Aftermath))
        .targets(&[obj(wurm), obj(giant), obj(bears)])
        .go();
    t.g.remove_counters(obj(wurm), MM, 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

fn sacrifice_the_aura(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::YesNo { prompt, .. } if prompt.contains("Sacrifice") => Some(Answer::Bool(true)),
        Decision::ChooseEntities { candidates, .. } => candidates
            .iter()
            .find(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == "Holy Strength"))
            .map(|e| Answer::Entities(vec![*e])),
        _ => None,
    }
}

#[test]
fn torment_of_venom_target_still_there_while_tormented() {
    cr!("704.3", "608.2b", "701.50a");
    ruling!(
        "Torment of Venom",
        "Because state-based actions aren’t performed while Torment of Venom is resolving, the target creature is still on the battlefield while its controller is tormented. If that creature has an Aura attached to it, that Aura can be sacrificed."
    );
    ruling!(
        "Torment of Venom",
        "If the target creature is an illegal target by the time Torment of Venom resolves, the entire spell doesn’t resolve. The creature’s controller won’t be tormented."
    );
    supported("Torment of Venom");
    // Grizzly Bears with Holy Strength (3/4): three counters, then P1 sacrifices the
    // Aura; the Bears die afterwards.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Holy Strength", obj(bears));
    respond(&mut t, P1, sacrifice_the_aura);
    cast_resolve(&mut t, P0, "Torment of Venom", &[obj(bears)]);
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    // Illegal target: no torment.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Torment of Venom");
    let tv = t.hand(P0, "Torment of Venom");
    t.cast(P0, tv).target(bears).go();
    destroy(&mut t, bears);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(crate::r_s01_common::asked_since(&t, from)
        .iter()
        .all(|(p, d)| *p != P1 || matches!(d, Decision::Priority { .. })));
}

#[test]
fn everlasting_torment_damage_is_wither_everywhere_and_unpreventable() {
    cr!("702.80a", "615.12", "702.16e");
    ruling!(
        "Everlasting Torment",
        "The last ability affects all damage, whether it's dealt by creatures, other permanents, spells, or cards that aren't on the battlefield. Wither works everywhere."
    );
    ruling!(
        "Everlasting Torment",
        "The \"damage can't be prevented\" statement overrides all forms of preventing damage, including protection abilities."
    );
    ruling!(
        "Everlasting Torment",
        "Spells and abilities that replace or redirect damage aren't affected by Everlasting Torment's second ability. They'll work as normal."
    );
    // A spell's damage to a creature: counters, no marked damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Everlasting Torment");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Lightning Bolt", &[obj(wurm)]);
    assert_eq!(t.counters(wurm, MM), 3);
    assert_eq!(t.obj_now(wurm).damage, 0);
    // Protection from red and a prevention shield don't stop Pyroclasm.
    let knight = t.battlefield(P1, "Silver Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_resolve(&mut t, P1, "Healing Salve", &[obj(bears)]);
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    assert!(t.in_graveyard(P1, "Silver Knight"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = knight;
    // Palisade Giant still redirects damage to it (as -1/-1 counters).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Everlasting Torment");
    let giant = t.battlefield(P0, "Palisade Giant");
    cast_resolve(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(giant, MM), 3);
}

#[test]
fn virulent_wound_poison_only_if_it_dies_this_turn() {
    cr!("603.7c", "514.2");
    ruling!(
        "Virulent Wound",
        "If the creature is not put into a graveyard on the turn Virulent Wound resolves, its controller won’t get a poison counter."
    );
    supported("Virulent Wound");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Virulent Wound", &[obj(bears)]);
    assert_eq!(t.counters(bears, MM), 1);
    t.advance_to(P1, Step::PrecombatMain);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(crate::r_s10_common::poison(&t, P1), 0);
    // Positive control: a 1/1 dies right away.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_resolve(&mut t, P0, "Virulent Wound", &[obj(elves)]);
    assert!(!t.on_battlefield(elves));
    assert_eq!(crate::r_s10_common::poison(&t, P1), 1);
}

#[test]
fn pyrrhic_revival_zero_toughness_creatures_still_trigger_on_entering() {
    cr!("704.5f", "603.6a", "122.6");
    ruling!(
        "Pyrrhic Revival",
        "Pyrrhic Revival can cause creatures to be put onto the battlefield that have 0 toughness due to the -1/-1 counter from this effect."
    );
    supported("Pyrrhic Revival");
    let mut t = TestGame::new(2);
    // Elvish Visionary: 1/1, "When this creature enters, draw a card."
    t.graveyard(P0, "Elvish Visionary");
    t.graveyard(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let pr = t.hand(P0, "Pyrrhic Revival");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::B, 3);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 3);
    t.cast(P0, pr).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Elvish Visionary"));
    assert_eq!(t.hand_size(P0), hand + 1, "its enters trigger resolved");
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P1);
    assert_eq!(t.pt(bears[0]), (1, 1));
}

#[test]
fn massacre_girl_uses_last_known_toughness() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Massacre Girl, Known Killer",
        "Use the toughness of the creature as it last existed on the battlefield to determine whether or not Massacre Girl's ability triggers."
    );
    supported("Massacre Girl, Known Killer");
    let mut t = TestGame::new(2);
    let mg = t.battlefield(P0, "Massacre Girl, Known Killer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // Destroyed with toughness 2: no draw.
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Massacre Girl's wither damage leaves the Elves at toughness -3: draw.
    crate::r_s06_common::damage(&mut t, mg, 4, elves);
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn serrated_arrows_checks_counters_at_upkeep_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Serrated Arrows",
        "The upkeep trigger checks the number of counters at the start of upkeep, and only goes on the stack if there are no arrowhead counters at that time. It will check again on resolution"
    );
    supported("Serrated Arrows");
    // With a counter: no trigger.
    let mut t = TestGame::new(2);
    let arrows = enter(&mut t, P0, "Serrated Arrows");
    assert_eq!(t.counters(arrows, "arrowhead"), 3);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // None: it triggers; a counter added in response stops the sacrifice.
    t.g.remove_counters(obj(arrows), "arrowhead", 3);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    add(&mut t, obj(arrows), "arrowhead", 1);
    t.resolve_all();
    assert!(t.on_battlefield(arrows));
    // And without the counter it's sacrificed.
    t.g.remove_counters(obj(arrows), "arrowhead", 1);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Serrated Arrows"));
}

#[test]
fn grafted_exoskeleton_cant_sacrifice_a_permanent_you_dont_control() {
    cr!("109.4", "701.21a");
    ruling!(
        "Grafted Exoskeleton",
        "If Grafted Exoskeleton's last ability triggers, but you don't control the permanent it became unattached from at the time that ability resolves (perhaps because another player has somehow gained control of it), you won't be able to sacrifice it."
    );
    for steal in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ex = attach_new(&mut t, P0, "Grafted Exoskeleton", obj(bears));
        destroy(&mut t, ex);
        assert_eq!(t.stack_len(), 1);
        if steal {
            give_control(&mut t, bears, P1);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), steal, "stolen {steal}");
    }
}

#[test]
fn inescapable_brute_must_be_blocked_if_able() {
    cr!("509.1c");
    ruling!(
        "Inescapable Brute",
        "If Inescapable Brute is attacking, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block Inescapable Brute."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Inescapable Brute");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(brute, pl(P1))]);
    // P1 declares no blocks: not allowed.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(wurm, MM), 3, "wither");
}
