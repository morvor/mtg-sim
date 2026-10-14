//! Rulings batch P121 — targets that must have a +1/+1 counter (checked as the ability is
//! put on the stack and as it resolves, CR 115.1, 608.2b; the effect lasts after the
//! counter goes away, CR 611.2c), targets without one, cumulative "enters with an
//! additional counter" effects (CR 614.1c), blocking restrictions and permissions that
//! apply only as blockers are declared (CR 509.1), tokens entering attacking (CR 508.4)
//! and copies (CR 707.2).

use crate::r_p121_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s21_common::{blocks_now, legal_blocks};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- targets with a counter -----------------------------------------------------------------

#[test]
fn target_with_a_counter_keeps_the_abilities_after_losing_it() {
    cr!("115.1", "611.2c");
    ruling!(
        "Ollenbock Escort",
        "The creature having a +1/+1 counter on it is part of the targeting requirement of Ollenbock Escort's ability. After the ability has resolved, the creature won't lose lifelink or indestructible if the counter goes away."
    );
    ruling!(
        "Sigardian Paladin",
        "Sigardian Paladin's last ability can target only a creature with a +1/+1 counter on it. Once that ability has resolved, the creature has trample and lifelink until end of turn even if it somehow loses its counters or stops being a creature (although trample doesn't help a noncreature permanent much)."
    );
    supported("Ollenbock Escort");
    supported("Sigardian Paladin");
    for (name, index, kws) in [
        (
            "Ollenbock Escort",
            0,
            [KeywordKind::Lifelink, KeywordKind::Indestructible],
        ),
        (
            "Sigardian Paladin",
            0,
            [KeywordKind::Trample, KeywordKind::Lifelink],
        ),
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Wastes", 1);
        let plain = t.battlefield(P0, "Hill Giant");
        let with = with_counters(&mut t, P0, "Grizzly Bears", 1);
        let from = t.asked().len();
        t.activate(P0, src, index, &[obj(with)]).expect("activate");
        let cands = &target_candidates(&t, P0, from)[0];
        assert!(cands.contains(&obj(with)) && !cands.contains(&obj(plain)), "{name}");
        t.resolve_all();
        remove_plus1(&mut t, with, 1);
        for k in kws {
            assert!(has_kw(&mut t, with, k), "{name}: {k:?}");
        }
    }
}

#[test]
fn arcbound_fiend_can_target_a_creature_without_counters() {
    cr!("115.1", "608.2c");
    ruling!(
        "Arcbound Fiend",
        "The creature that’s targeted by Arcbound Fiend’s middle ability doesn’t have to have any +1/+1 counters on it. If it doesn’t have a counter on it when the ability resolves, nothing happens."
    );
    supported("Arcbound Fiend");
    let mut t = TestGame::new(2);
    let fiend = with_counters(&mut t, P0, "Arcbound Fiend", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(bears))));
    assert_eq!((plus1(&t, fiend), plus1(&t, bears)), (3, 0));
    assert!(t.on_battlefield(bears));
}

#[test]
fn hog_monkey_needs_a_creature_with_a_counter_as_it_triggers() {
    cr!("603.3d", "115.1");
    ruling!(
        "Hog-Monkey",
        "The creature you target must have a +1/+1 counter on it when the ability goes on the stack at the beginning of combat. You cannot put the counter on it in response."
    );
    supported("Hog-Monkey");
    // No creature with a counter: the ability is removed from the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hog-Monkey");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(stack(&t), 0);
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert!(!has_kw(&mut t, bears, KeywordKind::Menace));
    // With one, only it can be targeted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hog-Monkey");
    let plain = t.battlefield(P0, "Hill Giant");
    let with = with_counters(&mut t, P0, "Grizzly Bears", 1);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(with)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let cands = &target_candidates(&t, P0, from)[0];
    assert!(cands.contains(&obj(with)) && !cands.contains(&obj(plain)));
    assert!(has_kw(&mut t, with, KeywordKind::Menace));
}

#[test]
fn high_sentinels_can_target_itself() {
    cr!("115.1", "115.5");
    ruling!(
        "High Sentinels of Arashin",
        "The activated ability can target and put +1/+1 counters on High Sentinels of Arashin itself."
    );
    supported("High Sentinels of Arashin");
    let mut t = TestGame::new(2);
    let hs = t.battlefield(P0, "High Sentinels of Arashin");
    t.lands(P0, "Plains", 4);
    t.activate(P0, hs, 0, &[obj(hs)]).expect("activate");
    t.resolve_all();
    assert_eq!(plus1(&t, hs), 1);
    assert_eq!(t.pt(hs), (4, 5));
}

#[test]
fn solidarity_of_heroes_targets_any_creature_and_heroic_resolves_first() {
    cr!("603.3", "701.10e");
    ruling!(
        "Solidarity of Heroes",
        "Solidarity of Heroes can target any creatures, not just ones with +1/+1 counters on them. Notably, heroic abilities of any target creatures that put +1/+1 counters on that creature will resolve before Solidarity of Heroes."
    );
    supported("Solidarity of Heroes");
    supported("Wingsteed Rider");
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Wingsteed Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Strive: {1}{G} more for the second target.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let spell = cast_slots(&mut t, P0, "Solidarity of Heroes", &[&[obj(rider), obj(bears)]]);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    assert_eq!(t.g.obj(spell).stack.as_ref().unwrap().chosen[0].targets[0].len(), 2);
    t.resolve_all();
    assert_eq!((plus1(&t, rider), plus1(&t, bears)), (2, 0));
}

// --- enters with an additional counter -------------------------------------------------------

#[test]
fn enters_with_an_additional_counter_effects_are_cumulative() {
    cr!("614.1c", "616.1");
    ruling!(
        "Bramblewood Paragon",
        "The effects from more than one Bramblewood Paragon are cumulative. That is, if you have more than one Bramblewood Paragon on the battlefield, Warrior creatures you control enter with that many additional +1/+1 counters on them."
    );
    ruling!(
        "Oona's Blackguard",
        "The effects from more than one of these cards are cumulative. That is, if you have two Oona's Blackguard on the battlefield, Rogue creatures you control enter with two additional +1/+1 counters on them."
    );
    for (lord, creature) in [
        ("Bramblewood Paragon", "Elvish Warrior"),
        ("Oona's Blackguard", "Oona's Prowler"),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        t.battlefield(P0, lord);
        let c = t.enter(P0, creature);
        assert_eq!(plus1(&t, c), 2, "{lord}");
    }
}

// --- blocking -------------------------------------------------------------------------------

#[test]
fn battlefront_krushok_restriction_applies_only_as_blockers_are_declared() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Battlefront Krushok",
        "The second ability applies only as blockers are declared. If a creature you control with no +1/+1 counters on it becomes blocked by more than one creature, putting a +1/+1 counter on it won’t change or undo the block."
    );
    supported("Battlefront Krushok");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battlefront Krushok");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, bears), (b, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t).len(), 2);
    give_plus1(&mut t, bears, 1);
    assert!(!legal_blocks(&mut t, P1, &[(a, bears), (b, bears)]));
    assert_eq!(blocks_now(&t).len(), 2);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn cenns_tacticians_are_cumulative() {
    cr!("509.1b", "509.1a");
    ruling!(
        "Cenn's Tactician",
        "The second ability is cumulative. That is, each creature you control with any +1/+1 counters on it can block an additional creature for each Cenn’s Tactician you control. Cenn’s Tactician doesn’t care whether the creature has any +1/+1 counters on it beyond the first."
    );
    supported("Cenn's Tactician");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Cenn's Tactician");
    let wall = with_counters(&mut t, P1, "Hill Giant", 3);
    let plain = t.battlefield(P1, "Hill Giant");
    let attackers: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    attack_with(&mut t, &decl);
    let two: Vec<(ObjectId, ObjectId)> = attackers[..2].iter().map(|a| (wall, *a)).collect();
    let three: Vec<(ObjectId, ObjectId)> = attackers.iter().map(|a| (wall, *a)).collect();
    // One Tactician: two creatures, despite three counters; the plain one blocks one.
    assert!(legal_blocks(&mut t, P1, &two));
    assert!(!legal_blocks(&mut t, P1, &three));
    assert!(!legal_blocks(
        &mut t,
        P1,
        &[(plain, attackers[0]), (plain, attackers[1])]
    ));
    // Two Tacticians: three creatures.
    t.battlefield(P1, "Cenn's Tactician");
    assert!(legal_blocks(&mut t, P1, &three));
    block_and_finish(&mut t, P1, &three);
    assert_eq!(t.life(P1), 20);
}

// --- tokens entering attacking, copies -------------------------------------------------------

/// P0's transformed Furnace-Blessed Conqueror with `n` +1/+1 counters, able to attack.
fn conqueror(t: &mut TestGame, n: u32) -> ObjectId {
    let c = enter_with(
        t,
        P0,
        "Sun-Blessed Guardian // Furnace-Blessed Conqueror",
        n,
        true,
    );
    let c = t.g.current(c);
    t.g.objects[c.0 as usize].summoning_sick = false;
    c
}

fn conqueror_token(t: &TestGame, original: ObjectId) -> ObjectId {
    t.g.permanents()
        .find(|o| o.is_token() && o.chars.name.contains("Furnace-Blessed") && o.id != original)
        .map(|o| o.id)
        .expect("the token")
}

#[test]
fn furnace_blessed_conqueror_token_copies_only_copiable_values() {
    cr!("707.2", "707.4");
    ruling!(
        "Sun-Blessed Guardian // Furnace-Blessed Conqueror",
        "The token doesn’t copy whether Furnace-Blessed Conqueror is tapped or untapped (although the effect itself causes it to enter tapped), whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Sun-Blessed Guardian // Furnace-Blessed Conqueror");
    let mut t = TestGame::new(2);
    let c = conqueror(&mut t, 2);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(c)]);
    assert_eq!(t.pt(c), (8, 8));
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    let tok = conqueror_token(&t, c);
    // 3/3 with the two counters the ability puts on it; no Giant Growth bonus.
    assert_eq!(t.pt(tok), (5, 5));
    assert!(t.obj_now(tok).tapped);
    assert!(t.g.combat.as_ref().unwrap().attackers.iter().any(|a| a.id == tok));
}

#[test]
fn furnace_blessed_conqueror_token_enters_before_getting_counters() {
    cr!("707.2", "603.6a");
    ruling!(
        "Sun-Blessed Guardian // Furnace-Blessed Conqueror",
        "The token will enter the battlefield and then have counters put on it. In most cases, the game will see a 3/3 creature entering the battlefield for the purposes of any abilities that may trigger."
    );
    supported("Garruk's Uprising");
    let mut t = TestGame::new(2);
    let c = conqueror(&mut t, 2);
    // "Whenever a creature with power 4 or greater you control enters, draw a card."
    t.battlefield(P0, "Garruk's Uprising");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    let tok = conqueror_token(&t, c);
    assert_eq!(t.pt(tok), (5, 5));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn anim_pakal_gnomes_can_each_attack_a_different_player() {
    cr!("508.4", "508.4a");
    ruling!(
        "Anim Pakal, Thousandth Moon",
        "You choose which player, planeswalker, or battle the Gnome tokens are attacking. Each Gnome token can enter attacking a different player, planeswalker, or battle, and they don't need to be the same ones that the non-Gnome creatures are attacking."
    );
    supported("Anim Pakal, Thousandth Moon");
    let mut t = crate::r_p177_common::ffa(3);
    with_counters(&mut t, P0, "Anim Pakal, Thousandth Moon", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pw = t.battlefield(P1, "Jace Beleren");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[obj(pw)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let combat = t.g.combat.as_ref().unwrap();
    let mut gnome_targets: Vec<Option<Entity>> = combat
        .attackers
        .iter()
        .filter(|a| t.g.obj(a.id).chars.has_subtype("Gnome"))
        .map(|a| a.target)
        .collect();
    gnome_targets.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![Some(Entity::Player(P2)), Some(obj(pw))];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(gnome_targets, want);
}

// --- modular and moving counters ----------------------------------------------------------

#[test]
fn modular_puts_all_the_counters_on_one_target() {
    cr!("702.43a", "115.1");
    ruling!(
        "Arcbound Wanderer",
        "When the Wanderer leaves the battlefield, you may put the counters onto a single target artifact creature. You can't split them among two or more artifact creatures."
    );
    ruling!(
        "Power Depot",
        "Yes, this land enters the battlefield with a +1/+1 counter, which does nothing while it's a land. When it's put into the graveyard from the battlefield, you may put its +1/+1 counters onto target artifact creature, no matter what Power Depot's card types are at the time."
    );
    supported("Arcbound Wanderer");
    supported("Power Depot");
    let mut t = TestGame::new(2);
    let w = with_counters(&mut t, P0, "Arcbound Wanderer", 3);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    destroy(&mut t, w);
    t.resolve_all();
    assert!(t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { max: 1, .. }
    )));
    assert_eq!((plus1(&t, a), plus1(&t, b)), (3, 0));
    // Power Depot: a land with a counter that does nothing.
    let mut t = TestGame::new(2);
    let depot = t.enter(P0, "Power Depot");
    t.resolve_all();
    assert_eq!(plus1(&t, depot), 1);
    assert_eq!(t.obj_now(depot).chars.power, None);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(thopter)]);
    t.answer_yes(P0, true);
    destroy(&mut t, depot);
    t.resolve_all();
    assert_eq!(plus1(&t, thopter), 1);
}

#[test]
fn aetherborn_marauder_chooses_on_resolution_and_may_move_none() {
    cr!("608.2c", "122.5");
    ruling!(
        "Aetherborn Marauder",
        "You choose which counters to move as Aetherborn Marauder's triggered ability resolves."
    );
    ruling!(
        "Aetherborn Marauder",
        "You may choose to move zero +1/+1 counters."
    );
    supported("Aetherborn Marauder");
    for n in [2u32, 0] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let m = t.enter(P0, "Aetherborn Marauder");
        t.settle();
        assert_eq!(stack(&t), 1);
        // Counters put on the Bears in response can be moved.
        give_plus1(&mut t, bears, 2);
        t.answer(P0, DecisionKind::Number, Answer::Number(n as i64));
        t.resolve_all();
        assert_eq!((plus1(&t, m), plus1(&t, bears)), (n, 2 - n));
    }
}
