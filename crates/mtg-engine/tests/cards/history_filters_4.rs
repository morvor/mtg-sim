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

/// The candidates of the latest choice of objects `p` was asked to make.
fn offered_choice(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } if q == p => {
                Some(candidates)
            }
            _ => None,
        })
        .expect("no choice was asked")
}

#[test]
fn jabaris_influence_takes_a_creature_that_attacked_you_this_turn() {
    cr!("508.1", "608.2b");
    assert_supported("Jabari's Influence");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let stayed = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P0))], &[]);
    t.advance_to(P1, Step::PostcombatMain);
    t.lands(P0, "Plains", 5);
    let ji = t.hand(P0, "Jabari's Influence");
    t.cast_with(P0, ji, &[Entity::Object(attacker)]).unwrap();
    assert_eq!(offered(&t, P0), vec![Entity::Object(attacker)]);
    t.resolve_all();
    assert_eq!(t.obj_now(attacker).controller, P0);
    assert_eq!(t.counters(attacker, "-1/-0"), 1);
    assert_eq!(t.obj_now(stayed).controller, P1);
}

#[test]
fn ghost_of_ramirez_depietro_returns_a_card_discarded_or_milled_this_turn() {
    cr!("603.4", "701.9a", "701.17a");
    assert_supported("Ghost of Ramirez DePietro");
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Ghost of Ramirez DePietro");
    let old = t.graveyard(P1, "Hill Giant");
    let d = t.hand(P1, "Grizzly Bears");
    let discarded = t.g.discard(P1, d, None).unwrap();
    t.library_top(P1, "Llanowar Elves");
    let milled = t.g.mill(P1, 1)[0];
    t.g.flush_events();
    t.answer_targets(P0, &[Entity::Object(discarded)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ghost, Entity::Player(P1))], &[]);
    t.resolve_all();
    let mut c = offered(&t, P0);
    c.sort_by_key(|e| e.object());
    let mut want = vec![Entity::Object(discarded), Entity::Object(milled)];
    want.sort_by_key(|e| e.object());
    assert_eq!(c, want, "not the card that was there before this turn");
    assert!(!c.contains(&Entity::Object(old)));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn emrakul_the_world_anew_has_protection_from_permanents_cast_this_turn() {
    cr!("702.16e");
    assert_supported("Emrakul, the World Anew");
    let mut t = TestGame::new(2);
    let emrakul = t.battlefield(P0, "Emrakul, the World Anew");
    let placed = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let b = t.hand(P1, "Grizzly Bears");
    t.cast_with(P1, b, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(b);
    t.g.deal_damage(bears, Entity::Object(emrakul), 2, false);
    t.g.flush_events();
    assert_eq!(t.obj_now(emrakul).damage, 0, "the bears were cast this turn");
    t.g.deal_damage(placed, Entity::Object(emrakul), 3, false);
    t.g.flush_events();
    assert_eq!(t.obj_now(emrakul).damage, 3, "the giant wasn't cast");
}

#[test]
fn activated_sleeper_copies_a_creature_card_put_into_a_graveyard_from_the_battlefield() {
    cr!("707.9", "400.7");
    assert_supported("Activated Sleeper");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.destroy(giant, None);
    let died = t.g.current(giant);
    let d = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, d, None);
    t.graveyard(P0, "Llanowar Elves");
    t.g.flush_events();
    t.answer_choose(P0, &[Entity::Object(died)]);
    let s = t.enter(P0, "Activated Sleeper");
    assert_eq!(offered_choice(&t, P0), vec![Entity::Object(died)]);
    assert_eq!(t.obj_now(s).chars.name.as_str(), "Hill Giant");
    assert!(t.obj_now(s).chars.subtypes.iter().any(|x| x.as_str() == "Phyrexian"));
}

#[test]
fn kutzils_flanker_counts_creatures_that_left_the_battlefield_under_your_control() {
    cr!("603.10a", "700.4");
    assert_supported("Kutzil's Flanker");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.destroy(a, None);
    t.g.move_object(b, Zone::Hand(P0), mtg_engine::events::MoveCause::Return, None);
    t.g.destroy(theirs, None);
    t.g.flush_events();
    t.lands(P0, "Plains", 3);
    let k = t.hand(P0, "Kutzil's Flanker");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.cast(P0, k).go();
    t.resolve_all();
    assert_eq!(t.counters(k, "+1/+1"), 2);
}

#[test]
fn wreck_hunter_counts_nonland_cards_put_into_that_graveyard_from_the_battlefield() {
    cr!("700.4", "400.7");
    assert_supported("Wreck Hunter");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.g.destroy(c, None);
    t.g.destroy(land, None);
    let d = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, d, None);
    t.g.flush_events();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Wreck Hunter");
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Powerstone"), 1);
}

#[test]
fn hope_of_ghirapur_targets_a_player_it_dealt_combat_damage_to() {
    cr!("510.2", "115.1");
    assert_supported("Hope of Ghirapur");
    let mut t = TestGame::new(3);
    let hope = t.battlefield(P0, "Hope of Ghirapur");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(hope, Entity::Player(P1)), (bears, Entity::Player(P2))],
        &[],
    );
    t.advance_to(P0, Step::PostcombatMain);
    t.activate(P0, hope, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(offered(&t, P0), vec![Entity::Player(P1)]);
}

// --- more history conditions (`history_conditions_4.rs`) -------------------------------

#[test]
fn ludevic_lets_the_player_draw_if_a_player_other_than_you_lost_life() {
    cr!("603.4");
    assert_supported("Ludevic, Necro-Alchemist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ludevic, Necro-Alchemist");
    t.set_step(P1, Step::PrecombatMain);
    t.g.lose_life(P0, 2);
    t.g.flush_events();
    t.answer_yes(P1, true);
    let before = t.hand_size(P1);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), before, "only you lost life");
    // Next turn (P0's): P1 loses life.
    t.advance_to(P0, Step::PrecombatMain);
    t.g.lose_life(P1, 1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
}

#[test]
fn gimli_is_indestructible_after_two_of_your_creatures_died_this_turn() {
    cr!("700.4", "702.12b");
    assert_supported("Gimli, Mournful Avenger");
    let mut t = TestGame::new(2);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    let a = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(a, None);
    t.g.destroy(theirs, None);
    t.resolve_all();
    let b = t.battlefield(P0, "Llanowar Elves");
    t.g.destroy(b, None);
    t.resolve_all();
    t.g.destroy(gimli, None);
    t.resolve_all();
    assert!(t.on_battlefield(gimli), "two of your creatures died");
}

#[test]
fn gimli_isnt_indestructible_after_one_of_your_creatures_died() {
    cr!("700.4");
    let mut t = TestGame::new(2);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    let a = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(a, None);
    t.g.destroy(theirs, None);
    t.resolve_all();
    t.g.destroy(gimli, None);
    t.resolve_all();
    assert!(!t.on_battlefield(gimli), "an opponent's creature doesn't count");
}

#[test]
fn masters_manufactory_needs_it_or_another_artifact_to_have_entered_under_your_control() {
    cr!("602.5b", "603.6a");
    assert_supported("Master's Guide-Mural // Master's Manufactory");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master's Guide-Mural // Master's Manufactory");
    mtg_engine::dfc::transform(&mut t.g, m);
    t.g.recompute();
    assert!(t.activate(P0, m, 0, &[]).is_err(), "nothing entered this turn");
    t.enter(P1, "Ornithopter");
    t.g.flush_events();
    assert!(t.activate(P0, m, 0, &[]).is_err(), "not under your control");
    t.enter(P0, "Ornithopter");
    t.g.flush_events();
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Golem"), 1);
}

#[test]
fn sea_troll_regenerates_only_after_blocking_or_being_blocked_by_a_blue_creature() {
    cr!("509.1", "602.5b");
    assert_supported("Sea Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Sea Troll");
    t.lands(P0, "Island", 2);
    assert!(t.activate(P0, troll, 0, &[]).is_err());
    t.clear_answers();
    let blue = t.battlefield(P1, "Coral Merfolk");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(troll, Entity::Player(P1))], &[(blue, troll)]);
    t.activate(P0, troll, 0, &[]).unwrap();
}

#[test]
fn sea_troll_cant_regenerate_after_blocking_a_creature_that_isnt_blue() {
    cr!("509.1", "602.5b");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Sea Troll");
    t.lands(P0, "Island", 2);
    let green = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(green, Entity::Player(P0))], &[(troll, green)]);
    assert!(t.activate(P0, troll, 0, &[]).is_err());
}

#[test]
fn dream_salvage_draws_for_each_card_the_opponent_discarded_this_turn() {
    cr!("701.9a");
    assert_supported("Dream Salvage");
    let mut t = TestGame::new(3);
    for n in ["Grizzly Bears", "Hill Giant"] {
        let c = t.hand(P1, n);
        t.g.discard(P1, c, None);
    }
    let c = t.hand(P2, "Grizzly Bears");
    t.g.discard(P2, c, None);
    t.g.flush_events();
    t.lands(P0, "Island", 1);
    let ds = t.hand(P0, "Dream Salvage");
    let before = t.hand_size(P0);
    t.cast_with(P0, ds, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1 + 2);
}

#[test]
fn tangled_colony_makes_a_rat_for_each_damage_dealt_to_it_this_turn() {
    cr!("120.3", "603.10a");
    assert_supported("Tangled Colony");
    let mut t = TestGame::new(2);
    let tc = t.battlefield(P0, "Tangled Colony");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(tc), 1, false);
    t.g.flush_events();
    t.resolve_all();
    t.g.deal_damage(giant, Entity::Object(tc), 3, false);
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.on_battlefield(tc));
    assert_eq!(count_named(&t, P0, "Rat"), 4);
}
