//! Rulings batch P065 — triggered abilities: objects entering or leaving at the same time
//! (CR 603.6a, 603.10a), "if" clauses checked on trigger and resolution (603.4), targets
//! illegal on resolution (608.2b), "creatures you control" fixed as the ability resolves
//! (611.2c), "a card" excluding tokens and copies, one trigger for many cards leaving a
//! graveyard at once, and the number of targets fixed as the ability is put on the stack.

use crate::r_s01_common::{creatures, supported, tokens, watch};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s04_common::hand_names;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::has_kw;
use crate::r_s10_common::idol;
use crate::r_s14_common::triggers_from;
use crate::r_s24_common::enter_together;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn ambuscade_shaman_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Ambuscade Shaman",
        "If Ambuscade Shaman enters the battlefield at the same time as other creatures you control, its ability will trigger for each of those creatures."
    );
    supported("Ambuscade Shaman");
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Ambuscade Shaman"), (P0, "Grizzly Bears"), (P0, "Savannah Lions")],
    );
    assert_eq!(triggers_from(&t, ids[0]), 3);
    t.resolve_all();
    assert_eq!(t.pt(ids[0]), (4, 4));
    assert_eq!(t.pt(ids[1]), (4, 4));
    assert_eq!(t.pt(ids[2]), (4, 3));
}

#[test]
fn extractor_demon_sees_creatures_leaving_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Extractor Demon",
        "If Extractor Demon and another creature leave the battlefield at the same time, Extractor Demon’s triggered ability will trigger."
    );
    supported("Extractor Demon");
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Extractor Demon");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_and_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(!t.on_battlefield(demon));
    // One trigger (for the Bears): P1 milled two cards.
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn extractor_demon_can_target_any_player() {
    cr!("603.3d", "115.1");
    ruling!(
        "Extractor Demon",
        "You can target any player with Extractor Demon’s triggered ability. The target doesn’t have to be the controller of the creature that left the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Extractor Demon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2, "its controller milled itself");
    assert_eq!(t.graveyard_size(P1), 1, "just the Bears");
}

#[test]
fn silumgar_scavenger_dealt_lethal_damage_with_another_creature_isnt_saved() {
    cr!("603.10a", "704.5g");
    ruling!(
        "Silumgar Scavenger",
        "If Silumgar Scavenger is dealt lethal damage at the same time as another creature you control, it won't receive a counter from its last ability in time to save it."
    );
    supported("Silumgar Scavenger");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Silumgar Scavenger");
    t.battlefield(P0, "Grizzly Bears");
    cast_and_resolve(&mut t, P0, "Blasphemous Act", &[]);
    assert!(!t.on_battlefield(s));
    assert!(t.in_graveyard(P0, "Silumgar Scavenger"));
}

#[test]
fn nihilith_counts_only_cards_put_into_an_opponents_graveyard() {
    cr!("108.2", "603.2");
    ruling!(
        "Nihilith",
        "If a token is put into an opponent's graveyard from the battlefield, or a copy of a spell is put into an opponent's graveyard from the stack, Nihilith's ability will not trigger"
    );
    supported("Nihilith");
    let mut t = TestGame::new(2);
    let n = t.exile(P0, "Nihilith");
    t.g.objects[n.0 as usize]
        .counters
        .insert(counters::TIME.into(), 7);
    let token = create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    assert_eq!(triggers_from(&t, n), 0, "a token isn't a card");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    destroy(&mut t, bears);
    assert_eq!(triggers_from(&t, n), 1, "a card");
    t.resolve_all();
    assert_eq!(t.counters(n, counters::TIME), 6);
}

#[test]
fn lamentation_with_an_illegal_target_doesnt_gain_life() {
    cr!("608.2b");
    ruling!(
        "Lamentation",
        "If the target creature is an illegal target as Lamentation's first ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    supported("Lamentation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Lamentation");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn amphin_mutineer_with_an_illegal_target_creates_no_token() {
    cr!("608.2b");
    ruling!(
        "Amphin Mutineer",
        "If the target non-Salamander creature is an illegal target by the time Amphin Mutineer's ability tries to resolve, the ability doesn't resolve. No player creates a Salamander Warrior token."
    );
    supported("Amphin Mutineer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Amphin Mutineer");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn nantuko_shaman_checks_for_tapped_lands_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Nantuko Shaman",
        "If you control any tapped lands immediately after Nantuko Shaman enters the battlefield, its ability doesn't trigger at all."
    );
    supported("Nantuko Shaman");
    // A tapped land as it enters: no trigger.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    t.g.tap(land);
    let s = enter(&mut t, P0, "Nantuko Shaman");
    assert_eq!(triggers_from(&t, s), 0);
    // A land tapped while the ability is on the stack: no card.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    let s = enter(&mut t, P0, "Nantuko Shaman");
    assert_eq!(triggers_from(&t, s), 1);
    t.g.tap(land);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Tapped and untapped again in between: a card.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    enter(&mut t, P0, "Nantuko Shaman");
    t.g.tap(land);
    t.g.untap(land);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn platoon_dispenser_checks_for_two_other_creatures_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Platoon Dispenser",
        "You must control two or more other creatures both when the first ability triggers and when it resolves."
    );
    supported("Platoon Dispenser");
    // Two others at both times: a card.
    let mut t = TestGame::new(2);
    let pd = t.battlefield(P0, "Platoon Dispenser");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Savannah Lions");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, pd), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // One of them leaves before it resolves: no card.
    let mut t = TestGame::new(2);
    let pd = t.battlefield(P0, "Platoon Dispenser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Savannah Lions");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, pd), 1);
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Only one other at the beginning of the end step: no trigger.
    let mut t = TestGame::new(2);
    let pd = t.battlefield(P0, "Platoon Dispenser");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, pd), 0);
}

/// The creature `name` enters under P0's control with another creature; after the bonus
/// resolves, a creature that begins to be controlled later doesn't get it.
fn bonus_only_for_creatures_controlled_on_resolution(name: &str, others_only: bool, bonus: (i32, i32)) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = enter(&mut t, P0, name);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + bonus.0, 2 + bonus.1), "{name}");
    let base = card_pt(name);
    if others_only {
        assert_eq!(t.pt(c), base);
    } else {
        assert_eq!(t.pt(c), (base.0 + bonus.0, base.1 + bonus.1), "{name} itself");
    }
    let later = t.battlefield(P0, "Savannah Lions");
    let stolen = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    assert_eq!(t.pt(later), (2, 1), "{name}: a creature entering later");
    assert_eq!(t.pt(stolen), (3, 3), "{name}: a creature gained later");
}

fn card_pt(name: &str) -> (i32, i32) {
    let c = mtg_engine::card::card(name);
    let ch = &c.front().chars;
    (ch.power.unwrap_or(0), ch.toughness.unwrap_or(0))
}

#[test]
fn scourge_devil_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Scourge Devil",
        "Scourge Devil’s triggered ability affects only creatures you control at the time it resolves, including Scourge Devil itself. Creatures you begin to control later in the turn won’t get +1/+0."
    );
    bonus_only_for_creatures_controlled_on_resolution("Scourge Devil", false, (1, 0));
}

#[test]
fn knight_of_old_benalia_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Knight of Old Benalia",
        "The last ability of Knight of Old Benalia affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+1."
    );
    bonus_only_for_creatures_controlled_on_resolution("Knight of Old Benalia", true, (1, 1));
}

#[test]
fn treetop_ambusher_can_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Treetop Ambusher",
        "Treetop Ambusher can be the target of its own ability."
    );
    supported("Treetop Ambusher");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Treetop Ambusher");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(a)]);
    crate::r_s09_common::declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 2));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn mogiss_marauder_locks_in_its_number_of_targets() {
    cr!("700.5", "601.2c", "603.3d");
    ruling!(
        "Mogis's Marauder",
        "Use your devotion to black as you put the ability on the stack. Mogis's Marauder's mana cost will always count toward your devotion to black."
    );
    supported("Mogis's Marauder");
    let mut t = TestGame::new(2);
    let altar = idol(&mut t, P0, "{B}{B}");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    let c = t.battlefield(P0, "Hill Giant");
    let d = t.battlefield(P0, "Llanowar Elves");
    let targets: Vec<Entity> = [a, b, c, d].iter().map(|x| Entity::Object(*x)).collect();
    let from = t.asked().len();
    t.answer_targets(P0, &targets[..3]);
    let m = enter(&mut t, P0, "Mogis's Marauder");
    // Devotion 3: two {B} on the idol and the Marauder's own {B}.
    let max = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .expect("targets chosen");
    assert_eq!(max, 3);
    assert_eq!(triggers_from(&t, m), 1);
    // The devotion drops; the targets stay.
    destroy(&mut t, altar);
    t.resolve_all();
    for x in [a, b, c] {
        assert!(has_kw(&t, x, KeywordKind::Intimidate));
        assert!(has_kw(&t, x, KeywordKind::Haste));
    }
    assert!(!has_kw(&t, d, KeywordKind::Intimidate));
}

#[test]
fn rotting_rats_active_player_chooses_first_then_all_discard_at_once() {
    cr!("101.4", "701.9a");
    ruling!(
        "Rotting Rats",
        "first the player whose turn it is chooses a card to discard, then each other player in turn order does the same, then all cards are discarded at the same time."
    );
    supported("Rotting Rats");
    let mut t = TestGame::new(3);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Savannah Lions");
    t.hand(P2, "Hill Giant");
    // P2 chooses while P0's and P1's chosen cards are still in their hands.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.player(P0).hand.len(), g.player(P1).hand.len()),
    );
    let from = t.asked().len();
    enter(&mut t, P0, "Rotting Rats");
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard")))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1, P2]);
    assert_eq!(seen.lock().unwrap().first().copied(), Some((1, 1)));
    assert!(hand_names(&t, P0).is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
}

/// Exiles every card in P0's graveyard at once (Bojuka Bog) and returns the number of
/// triggers from `source`.
fn exile_graveyard_at_once(t: &mut TestGame, source: ObjectId) -> usize {
    t.answer_targets(P0, &[Entity::Player(P0)]);
    enter(t, P0, "Bojuka Bog");
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    triggers_from(t, source)
}

#[test]
fn hardened_academic_triggers_once_for_cards_leaving_at_once() {
    cr!("603.2c");
    ruling!(
        "Hardened Academic",
        "If multiple cards leave your graveyard at the same time, Hardened Academic's last ability will trigger only once."
    );
    supported("Hardened Academic");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hardened Academic");
    for n in ["Grizzly Bears", "Lightning Bolt", "Forest"] {
        t.graveyard(P0, n);
    }
    assert_eq!(exile_graveyard_at_once(&mut t, h), 1);
    t.answer_targets(P0, &[Entity::Object(h)]);
    t.resolve_all();
    assert_eq!(t.counters(h, counters::PLUS1), 1);
}

#[test]
fn rot_farm_mortipede_triggers_once_for_creature_cards_leaving_at_once() {
    cr!("603.2c");
    ruling!(
        "Rot Farm Mortipede",
        "If multiple creature cards leave your graveyard at the same time, Rot Farm Mortipede's ability will trigger only once."
    );
    supported("Rot Farm Mortipede");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Rot Farm Mortipede");
    for n in ["Grizzly Bears", "Hill Giant", "Savannah Lions"] {
        t.graveyard(P0, n);
    }
    assert_eq!(exile_graveyard_at_once(&mut t, m), 1);
    t.resolve_all();
    assert_eq!(t.pt(m), (4, 4));
    assert!(has_kw(&t, m, KeywordKind::Menace));
}

#[test]
fn trench_stalker_counts_cards_drawn_before_it_entered() {
    cr!("611.3a");
    ruling!(
        "Trench Stalker",
        "Trench Stalker doesn't need to have been on the battlefield when you drew the cards."
    );
    supported("Trench Stalker");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.settle();
    let s = t.battlefield(P0, "Trench Stalker");
    assert!(has_kw(&t, s, KeywordKind::Deathtouch));
    assert!(has_kw(&t, s, KeywordKind::Lifelink));
    let other = t.battlefield(P1, "Trench Stalker");
    assert!(!has_kw(&t, other, KeywordKind::Deathtouch), "P1 drew nothing");
}

#[test]
fn case_of_the_gorgons_kiss_looks_at_the_card_types_in_the_graveyard() {
    cr!("719.3a", "400.7");
    ruling!(
        "Case of the Gorgon's Kiss",
        "looks at what type the cards are after they move to the graveyard to determine whether the ability should trigger"
    );
    supported("Case of the Gorgon's Kiss");
    use mtg_engine::ability::{Modification, Value};
    use crate::r_s26_common::modify_until_eot;
    // Two creature cards and a creature card that was a noncreature permanent: solved.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Gorgon's Kiss");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    let c = t.battlefield(P1, "Hill Giant");
    modify_until_eot(
        &mut t,
        c,
        vec![Modification::RemoveTypes(vec![CardType::Creature])],
    );
    assert!(creatures(&t, P1).len() == 2);
    for x in [a, b, c] {
        destroy(&mut t, x);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(mtg_engine::cases::is_solved(&t.g, case));
    // Two creature cards and an animated noncreature card: not solved.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, "Case of the Gorgon's Kiss");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    let c = t.battlefield(P1, "Darksteel Ingot");
    modify_until_eot(
        &mut t,
        c,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(2)), Some(Value::c(2))),
        ],
    );
    assert!(creatures(&t, P1).len() == 3);
    for x in [a, b] {
        destroy(&mut t, x);
    }
    t.g.sacrifice(c, P1);
    t.settle();
    assert!(t.in_graveyard(P1, "Darksteel Ingot"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!mtg_engine::cases::is_solved(&t.g, case));
}
