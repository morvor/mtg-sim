//! Rulings batch P173 — resolution of spells and abilities that tap creatures or use their
//! power or toughness: tapping as an effect isn't a cost (CR 608.2c), an ability whose
//! targets are all illegal doesn't resolve (CR 608.2b), a spell with some illegal targets
//! doesn't affect them (CR 608.2b), state-based actions wait until the spell has resolved
//! (CR 704.3), and values from objects that left use last known information (CR 608.2h,
//! 113.7a).

use crate::r_p212_common::{back_face, boost};
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s24_common::tapped;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| obj(*i)).collect()
}

/// Makes a permanent an illegal target: it leaves the battlefield.
fn remove(t: &mut TestGame, id: ObjectId) {
    destroy(t, id);
    assert!(!t.on_battlefield(id));
}

// --- Tapping as an effect: responses change what's tapped -----------------------------------

#[test]
fn master_of_the_wild_hunt_taps_the_wolves_there_as_it_resolves() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Master of the Wild Hunt",
        "All Wolves you control are tapped as part of the activated ability's effect, not as a cost. Players may respond by attempting to tap or untap Wolves, or to create or remove them."
    );
    supported("Master of the Wild Hunt");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of the Wild Hunt");
    let w1 = t.battlefield(P0, "Watchwolf");
    let w2 = t.battlefield(P0, "Watchwolf");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, m, "Tap all untapped Wolf").expect("activates");
    assert!(!tapped(&t, w1) && !tapped(&t, w2), "not a cost");
    // In response: one Wolf is removed, the other tapped, and a new one arrives.
    remove(&mut t, w1);
    t.g.tap(w2);
    let w3 = t.battlefield(P0, "Watchwolf");
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![6]));
    t.resolve_all();
    // Only the new Wolf was tapped and dealt its 3 damage; it took the Wurm's 6.
    assert_eq!(t.obj_now(wurm).damage, 3, "{}", t.dump_log());
    assert!(!t.on_battlefield(w3));
    assert!(t.on_battlefield(w2));
}

// --- Illegal targets: the ability doesn't resolve --------------------------------------------

#[test]
fn abilities_with_an_illegal_target_do_nothing_at_all() {
    cr!("608.2b");
    ruling!(
        "Master of the Wild Hunt",
        "If the target creature is an illegal target by the time Master of the Wild Hunt's ability tries to resolve, the ability doesn't resolve. You won't tap your Wolves, and nothing will deal or be dealt damage."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of the Wild Hunt");
    let w = t.battlefield(P0, "Watchwolf");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, m, "Tap all untapped Wolf").expect("activates");
    remove(&mut t, wurm);
    t.resolve_all();
    assert!(!tapped(&t, w) && t.obj_now(w).damage == 0);

    ruling!(
        "Aryel, Knight of Windgrace",
        "If the target creature's power is greater than X as Aryel's last ability tries to resolve, the ability doesn't resolve. You can't tap extra Knights once the ability has been activated."
    );
    let mut t = TestGame::new(2);
    let aryel = t.battlefield(P0, "Aryel, Knight of Windgrace");
    let k = t.battlefield(P0, "Knight of Malice");
    let spare = t.battlefield(P0, "Knight of Malice");
    t.battlefield(P0, "Swamp");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(k), obj(spare)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, aryel, "Destroy target").expect("X = 2");
    t.clear_answers();
    t.g.untap(spare);
    boost(&mut t, bears, 1, 0);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "power 3 > X: illegal target");
    assert!(!tapped(&t, spare), "no extra Knights tapped");
    assert!(crate::r_s01_common::asked_since(&t, from).is_empty());

    ruling!(
        "Toralf, God of Fury",
        "If the target is an illegal target as the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't return Toralf's Hammer to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let hammer = back_face(&mut t, P0, "Toralf, God of Fury // Toralf's Hammer");
    let c = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(hammer, obj(c)));
    t.lands(P0, "Mountain", 2);
    let target = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(&mut t, P0, c, "Unattach").expect("activates");
    remove(&mut t, target);
    t.resolve_all();
    assert!(t.on_battlefield(hammer));
    assert!(!t.in_hand(P0, "Toralf, God of Fury // Toralf's Hammer"));

    ruling!(
        "Gallows at Willow Hill",
        "If the targeted creature is an illegal target when the ability tries to resolve, the ability won’t resolve and none of its effects will happen. The creature’s controller won’t put a Spirit token onto the battlefield."
    );
    supported("Gallows at Willow Hill");
    let mut t = TestGame::new(2);
    let gal = t.battlefield(P0, "Gallows at Willow Hill");
    for _ in 0..3 {
        t.battlefield(P0, "Alaborn Trooper");
    }
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, gal, "Destroy target").expect("activates");
    // The Bears leave (bounced) in response.
    let b = t.g.current(bears);
    crate::r_s05_common::move_to(&mut t, b, mtg_engine::object::Zone::Hand(P1));
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

#[test]
fn spells_with_an_illegal_target_do_nothing_at_all() {
    cr!("608.2b");
    ruling!(
        "Urge to Feed",
        "If the targeted creature is an illegal target by the time Urge to Feed resolves, the entire spell doesn't resolve. You can't tap any Vampires."
    );
    supported("Urge to Feed");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Noble");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 2);
    let s = t.hand(P0, "Urge to Feed");
    t.cast(P0, s).target(bears).go();
    remove(&mut t, bears);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(v)]);
    t.resolve_all();
    assert!(!tapped(&t, v) && t.counters(v, counters::PLUS1) == 0);
    assert!(crate::r_s01_common::asked_since(&t, from).is_empty());

    ruling!(
        "Swallow Whole",
        "If the target creature is an illegal target by the time Swallow Whole tries to resolve, the spell won’t resolve. You won’t put a +1/+1 counter on any creature."
    );
    supported("Swallow Whole");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(theirs);
    t.battlefield(P0, "Plains");
    let s = t.hand(P0, "Swallow Whole");
    t.answer_choose(P0, &[obj(mine)]);
    t.cast(P0, s).target(theirs).go();
    assert!(tapped(&t, mine));
    t.g.untap(theirs);
    t.resolve_all();
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.counters(mine, counters::PLUS1), 0);

    ruling!(
        "Crashing Wave",
        "If all targets of Crashing Wave become illegal before it resolves, the spell will not resolve at all and you will not distribute any stun counters, even if the opponent still controls some tapped creatures."
    );
    supported("Crashing Wave");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let already = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(already);
    add_mana(&mut t, P0, ManaType::U, 3);
    let s = t.hand(P0, "Crashing Wave");
    t.cast(P0, s).x(1).target(a).go();
    remove(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(already, counters::STUN), 0, "{}", t.dump_log());
    // With its target still legal, the counters can go on the already-tapped one.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let already = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(already);
    add_mana(&mut t, P0, ManaType::U, 3);
    let s = t.hand(P0, "Crashing Wave");
    t.cast(P0, s).x(1).target(a).go();
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![0, 3]));
    t.resolve_all();
    assert_eq!(
        t.counters(a, counters::STUN) + t.counters(already, counters::STUN),
        3
    );
}

#[test]
fn coordinated_clobbering_illegal_targets_deal_or_get_no_damage() {
    cr!("608.2b");
    ruling!(
        "Coordinated Clobbering",
        "If one of the target creatures you control is an illegal target when Coordinated Clobbering begins to resolve (perhaps because it's no longer on the battlefield or because it's tapped), that creature won't deal any damage. If the target creature an opponent controls is an illegal target, no damage will be dealt."
    );
    supported("Coordinated Clobbering");
    let setup = |t: &mut TestGame| {
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let wurm = t.battlefield(P1, "Craw Wurm");
        add_mana(t, P0, ManaType::G, 1);
        let s = t.hand(P0, "Coordinated Clobbering");
        t.answer_targets(P0, &[obj(a), obj(b)]);
        t.answer_targets(P0, &[obj(wurm)]);
        t.g.turn.priority = Some(P0);
        t.g.cast_spell(P0, s, mtg_engine::object::CastMethod::Normal)
            .expect("cast");
        (a, b, wurm)
    };
    // One of P0's creatures is tapped in response (an untapped target is required): it
    // deals no damage; one left the battlefield: none either.
    for tap_it in [true, false] {
        let mut t = TestGame::new(2);
        let (a, b, wurm) = setup(&mut t);
        if tap_it {
            t.g.tap(b);
        } else {
            remove(&mut t, b);
        }
        t.resolve_all();
        assert!(tapped(&t, a));
        assert_eq!(t.obj_now(wurm).damage, 2, "tap: {tap_it}: {}", t.dump_log());
    }
    // The opponent's creature is illegal: no damage (the spell still resolves for its
    // other targets, which are tapped, CR 608.2b).
    let mut t = TestGame::new(2);
    let (a, b, wurm) = setup(&mut t);
    crate::r_s05_common::move_to(&mut t, wurm, mtg_engine::object::Zone::Hand(P1));
    t.resolve_all();
    assert!(tapped(&t, a) && tapped(&t, b));
    assert_eq!(t.life(P1), 20);
    assert!(t.g.permanents().all(|o| o.damage == 0));
}

// --- State-based actions wait for the spell ---------------------------------------------------

#[test]
fn urge_to_feed_can_save_the_vampire_it_targets() {
    cr!("704.3", "608.2c", "704.5f");
    ruling!(
        "Urge to Feed",
        "If you cast Urge to Feed targeting an untapped Vampire creature you control with toughness 3, its toughness will be reduced to 0 by the -3/-3 effect. However, state-based actions aren't checked until the spell has finished resolving. You may then tap that Vampire and put a +1/+1 counter on it, raising its toughness to 1. It will remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Aristocrat");
    assert_eq!(t.pt(v).1, 2);
    boost(&mut t, v, 0, 1);
    assert_eq!(t.pt(v).1, 3);
    add_mana(&mut t, P0, ManaType::B, 2);
    let s = t.hand(P0, "Urge to Feed");
    t.cast(P0, s).target(v).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(v)]);
    t.resolve_all();
    assert!(t.on_battlefield(v), "{}", t.dump_log());
    assert!(tapped(&t, v));
    assert_eq!(t.pt(v).1, 1);
}

// --- Power and toughness from last known information ----------------------------------------

#[test]
fn dead_before_sunrise_uses_the_creatures_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Dead Before Sunrise",
        "If a creature leaves the battlefield while the ability granted to it by Dead Before Sunrise is still on the stack, use that creature’s power as it last existed on the battlefield to determine how much damage to deal."
    );
    supported("Dead Before Sunrise");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Merfolk Looter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    add_mana(&mut t, P0, ManaType::R, 4);
    let s = t.hand(P0, "Dead Before Sunrise");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(rogue).0, 2);
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, rogue, "damage equal to its power").expect("granted");
    boost(&mut t, rogue, 2, 0);
    remove(&mut t, rogue);
    t.resolve_all();
    // 4 damage: the Wurm (6/4) died.
    assert!(!t.on_battlefield(wurm), "{}", t.dump_log());
}

#[test]
fn phenax_mills_by_toughness_as_it_resolves_or_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Phenax, God of Deception",
        "If you activate the ability Phenax grants to creatures you control, the toughness of the creature is calculated as the ability resolves. If the creature is no longer on the battlefield at that time, use its toughness when it was last on the battlefield."
    );
    supported("Phenax, God of Deception");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Phenax, God of Deception");
        let c = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        activate_containing(&mut t, P0, c, "mills X").expect("granted");
        boost(&mut t, c, 0, 3);
        if leaves {
            remove(&mut t, c);
        }
        t.resolve_all();
        assert_eq!(t.graveyard_size(P1), 5, "leaves: {leaves}");
    }
}

/// P0's Phenax with devotion seven (Phenax counts {U}{B}; five more from a custom
/// permanent), so it's a creature.
fn creature_phenax(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let phenax = t.battlefield(P0, "Phenax, God of Deception");
    let devotion = t.custom(
        P0,
        crate::r_s01_common::custom_card(
            "Devotion Source",
            "Enchantment",
            "{U}{U}{B}{B}{B}",
            None,
            "",
        ),
        mtg_engine::object::Zone::Battlefield,
    );
    assert!(t.obj_now(phenax).is(CardType::Creature));
    (phenax, devotion)
}

#[test]
fn phenax_grants_itself_the_ability_and_mills_nothing_once_not_a_creature() {
    cr!("608.2h", "113.7a", "700.5");
    ruling!(
        "Phenax, God of Deception",
        "If Phenax is a creature, it will grant itself the activated ability."
    );
    let mut t = TestGame::new(2);
    let (phenax, _) = creature_phenax(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, phenax, "mills X").expect("Phenax has it");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 7);

    ruling!(
        "Phenax, God of Deception",
        "If you tap Phenax to activate the ability it grants itself, and Phenax is no longer a creature but still on the battlefield as that ability resolves, no cards will be put into the target player's graveyard. Similarly, if Phenax isn't on the battlefield as the ability resolves and wasn't a creature when it left the battlefield, no cards will be put into the graveyard."
    );
    // Still on the battlefield, no longer a creature.
    let mut t = TestGame::new(2);
    let (phenax, devotion) = creature_phenax(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, phenax, "mills X").expect("activates");
    remove(&mut t, devotion);
    assert!(!t.obj_now(phenax).is(CardType::Creature));
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
    // Left the battlefield (bounced: it's indestructible) while not a creature.
    let mut t = TestGame::new(2);
    let (phenax, devotion) = creature_phenax(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, phenax, "mills X").expect("activates");
    remove(&mut t, devotion);
    crate::r_s05_common::move_to(&mut t, phenax, mtg_engine::object::Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
}

#[test]
fn coordinated_clobbering_targets_two_of_your_creatures() {
    cr!("601.2c");
    // (Setup check for the clobbering test: both creatures deal damage when legal.)
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    add_mana(&mut t, P0, ManaType::G, 1);
    let s = t.hand(P0, "Coordinated Clobbering");
    t.answer_targets(P0, &objs(&[a, b]));
    t.answer_targets(P0, &[obj(wurm)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, s, mtg_engine::object::CastMethod::Normal)
        .expect("cast");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}
