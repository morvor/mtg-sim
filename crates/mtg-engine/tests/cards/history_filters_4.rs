//! Objects and players described by their history (`oracle/patterns/history_filters_4.rs`,
//! `Filter::History`, `PlayerFilter::History`): "a creature that fought this turn",
//! "target nonblack creature that entered since your last turn ended", "target card in an
//! opponent's graveyard that was put there from their library this turn", "target player
//! who lost life this turn", "target player dealt damage by ~ this turn".

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn count_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| {
            let c = &t.g.obj(*o).chars;
            c.name == name || c.subtypes.iter().any(|s| s.as_str() == name)
        })
        .count()
}

/// The candidates of the latest target choice `p` was asked to make.
fn offered(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } if q == p => {
                Some(candidates)
            }
            _ => None,
        })
        .expect("no target choice was asked")
}

// --- objects ----------------------------------------------------------------------------

#[test]
fn boxing_ring_needs_a_creature_you_control_that_fought_this_turn() {
    cr!("602.5b", "701.14a");
    assert_supported("Boxing Ring");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Boxing Ring");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let foe = t.battlefield(P1, "Hill Giant");
    assert!(
        t.activate(P0, ring, 0, &[]).is_err(),
        "no creature of yours fought this turn"
    );
    // An opponent's creature fighting doesn't count.
    t.lands(P0, "Forest", 1);
    let prey = t.hand(P0, "Prey Upon");
    t.cast_with(P0, prey, &[Entity::Object(bear), Entity::Object(foe)])
        .unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(bear), "the bear lost the fight");
    // The bear fought this turn, but it's gone: you no longer control it.
    assert!(t.activate(P0, ring, 0, &[]).is_err());
    let bear2 = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, ring, 0, &[]).is_err(), "the new bear didn't fight");
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    let prey2 = t.hand(P0, "Prey Upon");
    t.cast_with(P0, prey2, &[Entity::Object(bear2), Entity::Object(elf)])
        .unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bear2));
    t.activate(P0, ring, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Treasure"), 1);
}

#[test]
fn premature_burial_destroys_a_creature_that_entered_since_your_last_turn_ended() {
    cr!("608.2b");
    assert_supported("Premature Burial");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P1, "Grizzly Bears");
    // P0's turn ends; P1's turn: a creature enters.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    let new = t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let pb = t.hand(P0, "Premature Burial");
    t.cast_with(P0, pb, &[Entity::Object(new)]).unwrap();
    assert_eq!(
        offered(&t, P0),
        vec![Entity::Object(new)],
        "the bear was there before your last turn ended"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(new));
    assert!(t.on_battlefield(old));
}

#[test]
fn captain_nghathrod_takes_a_card_milled_this_turn_not_one_discarded() {
    cr!("603.4", "404.2");
    assert_supported("Captain N'ghathrod");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Captain N'ghathrod");
    let discarded = t.hand(P1, "Hill Giant");
    t.g.discard(P1, discarded, None);
    let gy_before = t.g.player(P1).graveyard.clone();
    t.library_top(P1, "Grizzly Bears");
    t.g.mill(P1, 1);
    t.g.flush_events();
    let milled = *t
        .g
        .player(P1)
        .graveyard
        .iter()
        .find(|o| !gy_before.contains(o))
        .unwrap();
    assert_eq!(t.g.obj(milled).chars.name, "Grizzly Bears");
    let in_gy_discarded = *gy_before.iter().next().unwrap();
    t.answer_targets(P0, &[Entity::Object(in_gy_discarded)]);
    t.answer_targets(P0, &[Entity::Object(milled)]);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Grizzly Bears"), 1, "the milled card");
    assert_eq!(count_named(&t, P0, "Hill Giant"), 0, "not the discarded one");
}

#[test]
fn shaile_counts_only_creatures_that_entered_under_your_control_this_turn() {
    cr!("603.4");
    assert_supported("Shaile, Dean of Radiance // Embrose, Dean of Shadow");
    let mut t = TestGame::new(2);
    let shaile = t.battlefield(P0, "Shaile, Dean of Radiance // Embrose, Dean of Shadow");
    let old = t.battlefield(P0, "Grizzly Bears");
    // Creatures put onto the battlefield by the harness have entered this turn: make the
    // bear and Shaile old by passing a turn.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    let theirs = t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.advance_to(P0, Step::PrecombatMain);
    let new = t.enter(P0, "Llanowar Elves");
    t.g.flush_events();
    t.activate(P0, shaile, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(new, "+1/+1"), 1);
    assert_eq!(t.counters(old, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

// --- players ----------------------------------------------------------------------------

#[test]
fn rix_maadi_guildmage_targets_only_a_player_who_lost_life_this_turn() {
    cr!("115.1", "608.2b");
    assert_supported("Rix Maadi Guildmage");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Rix Maadi Guildmage");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    assert!(
        t.activate(P0, gm, 1, &[Entity::Player(P1)]).is_err(),
        "no one lost life this turn"
    );
    t.clear_answers();
    t.g.lose_life(P1, 2);
    t.g.flush_events();
    t.activate(P0, gm, 1, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(offered(&t, P0), vec![Entity::Player(P1)]);
}

#[test]
fn wicked_akuba_needs_a_player_it_dealt_damage_to_this_turn() {
    cr!("120.2", "115.1");
    assert_supported("Wicked Akuba");
    let mut t = TestGame::new(3);
    let akuba = t.battlefield(P0, "Wicked Akuba");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.g.deal_damage(other, Entity::Player(P2), 2, false);
    t.g.flush_events();
    assert!(
        t.activate(P0, akuba, 0, &[Entity::Player(P2)]).is_err(),
        "another source's damage doesn't count"
    );
    t.clear_answers();
    t.g.deal_damage(akuba, Entity::Player(P1), 2, false);
    t.g.flush_events();
    t.activate(P0, akuba, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 18);
}

#[test]
fn papalymo_totolymo_makes_only_opponents_who_lost_life_sacrifice() {
    cr!("101.4", "701.21a");
    assert_supported("Papalymo Totolymo");
    let mut t = TestGame::new(3);
    let pap = t.battlefield(P0, "Papalymo Totolymo");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Hill Giant");
    t.lands(P0, "Plains", 4);
    t.g.lose_life(P1, 1);
    t.g.flush_events();
    t.activate(P0, pap, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(a), "P1 lost life this turn");
    assert!(t.on_battlefield(b), "P2 didn't");
    assert!(matches!(t.zone(pap), Zone::Graveyard(_)));
}
