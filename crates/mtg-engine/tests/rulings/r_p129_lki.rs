//! Rulings batch P129: reflexive triggered abilities that use last known information about
//! a sacrificed, exiled or dead creature (CR 608.2h, 603.10a), and values fixed when they
//! resolve.

use crate::r_p129_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Grizzly Bears with two +1/+1 counters (a 4/4 on the battlefield, a 2/2 card in the
/// graveyard): a value of 4 shows last known information was used.
fn pumped_bears(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let bears = t.battlefield(p, "Grizzly Bears");
    put_counters(t, bears, counters::PLUS1, 2);
    assert_eq!(t.pt(bears), (4, 4));
    bears
}

#[test]
fn sacrificed_creatures_power_is_its_last_known_power() {
    cr!("608.2h", "603.12", "701.21a");
    ruling!(
        "Heart-Piercer Manticore",
        "The sacrificed creature's last known existence on the battlefield is checked to determine its power."
    );
    ruling!(
        "Kazuul's Fury // Kazuul's Cliffs",
        "The sacrificed creature's last known existence on the battlefield is checked to determine its power."
    );
    ruling!(
        "Pyrrhic Blast",
        "The sacrificed creature's last known existence on the battlefield is checked to determine its power."
    );
    // Heart-Piercer Manticore: "you may sacrifice another creature. When you do, ...".
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t, P0);
    t.enter(P0, "Heart-Piercer Manticore");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 16);
    // Kazuul's Fury and Pyrrhic Blast: sacrificed as an additional cost.
    for name in ["Kazuul's Fury // Kazuul's Cliffs", "Pyrrhic Blast"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = pumped_bears(&mut t, P0);
        t.answer_choose(P0, &[obj(bears)]);
        cast_new(&mut t, P0, name, &[Entity::Player(P1)]);
        assert!(!t.on_battlefield(bears), "{name}");
        t.resolve_all();
        assert_eq!(t.life(P1), 16, "{name}");
    }
}

#[test]
fn bolg_uses_the_sacrificed_creatures_last_known_power_and_amasses_the_excess() {
    cr!("608.2h", "603.12", "120.4a", "701.47a");
    ruling!(
        "Bolg of the North",
        "The sacrificed creature's last known existence on the battlefield is checked to determine its power."
    );
    supported("Bolg of the North");
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Bolg of the North");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // 4 damage to a 3/3: 1 excess, amass Goblins 1.
    let army = with_subtype(&t, P0, "Army");
    assert_eq!(army.len(), 1);
    assert_eq!(t.counters(army[0], counters::PLUS1), 1);
}

#[test]
fn tiger_tribe_hunter_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "603.12");
    ruling!(
        "Tiger-Tribe Hunter",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Tiger-Tribe Hunter");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Tiger-Tribe Hunter");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = pumped_bears(&mut t, P0);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(wurm)]);
    // Total power 4 + 3 = 7: pack tactics.
    attack_with(
        &mut t,
        &[(hunter, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // 4 damage (not 2) kills the 6/4 Craw Wurm.
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn tip_the_scales_uses_the_sacrificed_creatures_last_known_toughness() {
    cr!("608.2h", "603.12");
    ruling!(
        "Tip the Scales",
        "Use the sacrificed creature’s toughness as it last existed on the battlefield to determine the value of X."
    );
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t, P0);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Tip the Scales", &[]);
    t.resolve_all();
    // -4/-4 (not -2/-2) kills the 6/4 Craw Wurm.
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn ziatora_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "603.12", "513.1a");
    ruling!(
        "Ziatora, the Incinerator",
        "Use the creature's power as it last existed on the battlefield to determine how much damage Ziatora deals."
    );
    supported("Ziatora, the Incinerator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ziatora, the Incinerator");
    let bears = pumped_bears(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 3);
}

#[test]
fn evereth_uses_its_last_known_power() {
    cr!("608.2h", "603.12", "603.10a");
    ruling!(
        "Evereth, Viceroy of Plunder",
        "Use Evereth's power as it last existed on the battlefield to determine how much damage it deals with its last ability."
    );
    supported("Evereth, Viceroy of Plunder");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let ev = t.battlefield(P0, "Evereth, Viceroy of Plunder");
    put_counters(&mut t, ev, counters::PLUS1, 2);
    t.answer_yes(P0, true);
    destroy(&mut t, ev);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sandbender_scavengers_uses_its_last_known_power_for_the_target() {
    cr!("608.2h", "603.12", "603.10a", "115.1");
    ruling!(
        "Sandbender Scavengers",
        "Use Sandbender Scavengers's power as it last existed on the battlefield to determine which creature cards may be targeted with its last ability."
    );
    supported("Sandbender Scavengers");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Sandbender Scavengers");
    put_counters(&mut t, sc, counters::PLUS1, 3);
    // Hill Giant has mana value 4: targetable only with the last known power 4.
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    destroy(&mut t, sc);
    t.resolve_all();
    assert!(t.in_exile("Sandbender Scavengers"));
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(giant))));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn auron_exiled_creature_stays_exiled_when_aurons_power_drops() {
    cr!("603.12", "610.3");
    ruling!(
        "Auron, Venerated Guardian",
        "Reducing Auron's power below the exiled creature's power won't cause the exiled creature to return"
    );
    let mut t = TestGame::new(2);
    let auron = t.battlefield(P0, "Auron, Venerated Guardian");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(auron, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let auron = t.g.current(auron);
    t.g.remove_counters(Entity::Object(auron), counters::PLUS1, 1);
    t.g.add_counters(Entity::Object(auron), counters::MINUS1, 2, None);
    t.g.recompute();
    t.settle();
    assert_eq!(t.pt(auron).0, 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn armix_counts_the_discarded_artifact_and_fixes_x_on_resolution() {
    cr!("603.12", "608.2h", "701.9a");
    ruling!(
        "Armix, Filigree Thrasher",
        "If the discarded card is an artifact card, it will be counted as you determine the value of X"
    );
    ruling!(
        "Armix, Filigree Thrasher",
        "The value of X is determined only as Armix's ability resolves."
    );
    ruling!(
        "Armix, Filigree Thrasher",
        "While resolving Armix's ability, you can't discard more than one card to give more than one creature -X/-X."
    );
    supported("Armix, Filigree Thrasher");
    let mut t = TestGame::new(2);
    let armix = t.battlefield(P0, "Armix, Filigree Thrasher");
    let thopter = t.hand(P0, "Ornithopter");
    t.hand(P0, "Ornithopter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(wurm)]);
    attack_with(&mut t, &[(armix, Entity::Player(P1))]);
    t.resolve_all();
    // Only one card was discarded.
    assert_eq!(t.hand_size(P0), hand - 1);
    // X = Armix + the discarded Ornithopter = 2.
    assert_eq!(t.pt(wurm), (4, 2));
    // X doesn't change later in the turn.
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(wurm), (4, 2));
}
