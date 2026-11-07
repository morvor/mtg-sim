//! Rulings batch P007 — burn planeswalkers: Chandra's loyalty abilities (targets, the
//! stack, play permissions, counting damage, emblems), Nicol Bolas's faces and Chandra,
//! Pyromaster's free copies.

use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn chandra_bold_pyromancer_plus_one_targets_only_players() {
    cr!("115.1", "601.2c");
    ruling!(
        "Chandra, Bold Pyromancer",
        "Chandra's first ability can't target a planeswalker."
    );
    supported("Chandra, Bold Pyromancer");
    let mut t = TestGame::new(2);
    let theirs = pw(&mut t, P1, "Vivien, Arkbow Ranger", 4);
    let c = pw(&mut t, P0, "Chandra, Bold Pyromancer", 5);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(theirs)]);
    let r = t.activate(P0, c, 0, &[]);
    t.clear_answers();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&pl(P1)));
    assert!(!offered.contains(&obj(theirs)), "{offered:?}");
    if r.is_ok() {
        t.resolve_all();
    }
    assert_eq!(t.counters(theirs, "loyalty"), 4);
}

#[test]
fn chandra_flameshaper_mana_ability_uses_the_stack() {
    cr!("605.1a", "606.3");
    ruling!(
        "Chandra, Flameshaper",
        "Chandra's first ability is not a mana ability. It uses the stack and players can respond to it."
    );
    supported("Chandra, Flameshaper");
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Flameshaper", 6);
    loyalty(&mut t, P0, c, 0, &[]);
    assert_eq!(t.stack_len(), 1, "on the stack");
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::R), 3);
}

#[test]
fn chandra_fire_artisan_exiled_cards_keep_their_timing() {
    cr!("601.3", "305.2", "305.1");
    ruling!(
        "Chandra, Fire Artisan",
        "Chandra's loyalty abilities don't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    supported("Chandra, Fire Artisan");
    // A sorcery: not while a spell is on the stack; then yes.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
    let spike = t.library_top(P0, "Lava Spike");
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    let spike = t.g.current(spike);
    assert_eq!(t.zone(spike), Zone::Exile);
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[pl(P1)]);
    assert!(
        t.cast(P0, spike).try_go().is_err(),
        "not with the stack nonempty"
    );
    t.clear_answers();
    t.resolve_all();
    t.answer_targets(P0, &[pl(P1)]);
    assert!(t.cast(P0, spike).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // A land: only with a land play left.
    for land_played in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
        let m = t.library_top(P0, "Mountain");
        loyalty(&mut t, P0, c, 0, &[]);
        t.resolve_all();
        let m = t.g.current(m);
        if land_played {
            let other = t.hand(P0, "Forest");
            t.play_land(P0, other).expect("the turn's land");
        }
        assert_eq!(
            t.play_land(P0, m).is_ok(),
            !land_played,
            "land played {land_played}"
        );
    }
}

#[test]
fn chandra_fire_of_kaladesh_counts_her_combat_damage() {
    cr!("120.2", "701.27a");
    ruling!(
        "Chandra, Fire of Kaladesh // Chandra, Roaring Flame",
        "Chandra, Fire of Kaladesh's activated ability will count any damage Chandra has dealt during the turn to any permanent or player, including combat damage."
    );
    supported("Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    attack_with(&mut t, &[(c, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18, "2 combat damage");
    // A red spell untaps her; then {T}: 1 damage, 3 in all this turn: she transforms.
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    let c = t.g.current(c);
    assert!(!t.obj_now(c).tapped);
    t.activate(P0, c, 0, &[pl(P1)]).expect("{T}");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(
        t.named_on_battlefield("Chandra, Roaring Flame").len(),
        1,
        "exiled and returned transformed"
    );
}

#[test]
fn chandra_roaring_flame_emblems_belong_to_their_players() {
    cr!("114.2", "114.4", "800.4a");
    ruling!(
        "Chandra, Fire of Kaladesh // Chandra, Roaring Flame",
        "Each player who gets Chandra's emblem is the owner of that emblem. In multiplayer games, that emblem will remain in the game as long as its owner does, even if Chandra's owner leaves the game."
    );
    let mut t = TestGame::new(3);
    let c = t.battlefield(P0, "Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    assert!(mtg_engine::dfc::transform(&mut t.g, c));
    let c = t.g.current(c);
    t.g.objects[c.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (14, 14));
    let emblems = |t: &TestGame| -> Vec<PlayerId> {
        t.g.objects
            .iter()
            .filter(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command)
            .map(|o| o.owner)
            .collect()
    };
    let mut owners = emblems(&t);
    owners.sort();
    assert_eq!(owners, vec![P1, P2]);
    t.g.lose_game(P0);
    t.settle();
    let mut owners = emblems(&t);
    owners.sort();
    assert_eq!(
        owners,
        vec![P1, P2],
        "the emblems stay after Chandra's owner leaves"
    );
    // Each emblem damages its owner at their upkeep.
    t.set_step(P2, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (11, 14));
}

#[test]
fn nicol_bolas_uses_only_the_face_thats_up() {
    cr!("712.8d", "712.8e");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "Each face of a double-faced card has its own set of characteristics: name, types, subtypes, abilities, and so on. While a double-faced card is on the battlefield, consider only the characteristics of the face that's currently up."
    );
    supported("Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.battlefield(P0, "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    let o = t.obj_now(b);
    assert_eq!(o.chars.name.as_str(), "Nicol Bolas, the Ravager");
    assert!(o.is(CardType::Creature));
    assert!(o.chars.has_keyword(KeywordKind::Flying));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    t.activate(P0, b, 0, &[]).expect("transform ability");
    t.resolve_all();
    let arisen = t.named_on_battlefield("Nicol Bolas, the Arisen");
    assert_eq!(arisen.len(), 1);
    let o = t.obj_now(arisen[0]);
    assert!(o.is(CardType::Planeswalker));
    assert!(!o.is(CardType::Creature));
    assert!(!o.chars.has_keyword(KeywordKind::Flying));
    assert_eq!(o.chars.power, None);
    assert!(o.chars.has_subtype("Bolas"));
    assert!(!o.chars.has_subtype("Dragon"));
}

#[test]
fn chandra_pyromaster_copies_can_be_kicked() {
    cr!("118.9a", "118.9d", "702.33a");
    ruling!(
        "Chandra, Pyromaster",
        "Because you're casting the copies \"without paying their mana costs,\" you can't pay any alternative costs for the copies. You can pay additional costs, such as kicker costs."
    );
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Pyromaster", 7);
    let bolt = t.library_top(P0, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    t.answer_choose_moved(P0, &[obj(bolt)]);
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
    }
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(false),
    );
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(false),
    );
    loyalty(&mut t, P0, c, 2, &[]);
    t.resolve_all();
    // One kicked copy (4) and two unkicked (2 each).
    assert_eq!(t.life(P1), 12);
}
