//! Rulings batch P115 — damage: excess damage dealt to a creature's controller (Flame
//! Spill, Pigment Storm; CR 120.4a) and "pingers" (Ancient Hellkite, Raging Regisaur,
//! Quicksmith Rebel, Hellkite Whelp, Heart-Piercer Bow, Rocket Launcher), including who
//! the defending player is.

use crate::r_p115_common::*;
use crate::r_s02_common::{can_activate, target_candidates};
use crate::r_s06_common::activate_containing;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn excess_damage_spells_do_nothing_if_the_creature_is_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Flame Spill",
        "If the target creature is an illegal target by the time Flame Spill tries to resolve, the spell won’t resolve. It won’t deal damage to any player."
    );
    ruling!(
        "Pigment Storm",
        "If the target creature is an illegal target by the time Pigment Storm tries to resolve, the spell won't resolve. It won't deal damage to any player."
    );
    for name in ["Flame Spill", "Pigment Storm"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_at(&mut t, P0, name, &[obj(bears)]);
        cast_at(&mut t, P1, "Unsummon", &[obj(bears)]);
        t.resolve_all();
        assert!(t.in_hand(P1, "Grizzly Bears"), "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        assert!(t.in_graveyard(P0, name));
    }
}

#[test]
fn flame_spills_two_damage_amounts_are_modified_separately() {
    cr!("120.4a", "120.4b", "615.1a", "614.1a");
    ruling!(
        "Flame Spill",
        "Once you’ve determined how much damage is excess, Flame Spill simultaneously deals damage to the creature and to its controller. This damage may be modified by replacement or prevention effects."
    );
    // 4 damage to Grizzly Bears (2/2): 2 lethal, 2 excess to P1, in one event.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.g.turn_events.len();
    cast_at(&mut t, P0, "Flame Spill", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let dealt: Vec<u32> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(dealt, vec![2, 2]);
    // A prevention shield on P1 prevents the excess only.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[1]).target(pl(P1)).go();
    t.resolve_all();
    cast_at(&mut t, P0, "Flame Spill", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Furnace of Rath doubles each part: 4 to the Bears, 4 to P1.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Furnace of Rath");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Flame Spill", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn pigment_storms_excess_is_determined_before_other_replacements() {
    cr!("120.4a", "614.15");
    ruling!(
        "Pigment Storm",
        "Pigment Storm's replacement effect is a self-replacement effect. It is applied before any other replacement effects that would change the amount of damage dealt to the target creature and its controller."
    );
    // 5 damage to Grizzly Bears: 2 lethal, 3 excess; then Furnace of Rath doubles each
    // (4 to the Bears, 6 to P1). Doubling first would have made the excess 8.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace of Rath");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Pigment Storm", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn ancient_hellkite_pings_only_while_attacking() {
    cr!("602.5", "511.3");
    ruling!(
        "Ancient Hellkite",
        "Unless Ancient Hellkite leaves combat, it continues to be attacking (so its ability may still be activated) through the end of combat step."
    );
    supported("Ancient Hellkite");
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Ancient Hellkite");
    let bears = t.battlefield(P1, "Llanowar Elves");
    let lions = t.battlefield(P1, "Savannah Lions");
    let mine = t.battlefield(P0, "Hill Giant");
    lots(&mut t, P0);
    assert!(!can_activate(&mut t, P0, kite));
    attack_with(&mut t, &[(kite, pl(P1))]);
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, kite));
    // Only creatures the defending player controls can be targeted.
    let from = t.asked().len();
    t.activate(P0, kite, 0, &[obj(bears)]).expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&obj(lions)));
    assert!(!cands[0].contains(&obj(mine)));
    t.advance_to(P0, Step::EndOfCombat);
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, kite));
    t.activate(P0, kite, 0, &[obj(lions)]).expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    t.advance_to(P0, Step::PostcombatMain);
    lots(&mut t, P0);
    assert!(!can_activate(&mut t, P0, kite));
}

#[test]
fn raging_regisaurs_trigger_damage_isnt_combat_damage() {
    cr!("510.2", "615.1");
    ruling!(
        "Raging Regisaur",
        "Damage dealt by Raging Regisaur's triggered ability isn't combat damage."
    );
    // Fog in response to the trigger prevents only combat damage: the 1 damage is dealt.
    let mut t = TestGame::new(2);
    let rex = t.battlefield(P0, "Raging Regisaur");
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(rex, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P1, "Fog", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

/// P0's Mind Stone and a Quicksmith Rebel entering under P0's control, its trigger on
/// the stack targeting the Mind Stone.
fn rebel(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let stone = t.battlefield(P0, "Mind Stone");
    t.answer_targets(P0, &[obj(stone)]);
    let rebel = t.enter(P0, "Quicksmith Rebel");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (stone, rebel)
}

fn pings(t: &TestGame, stone: ObjectId) -> bool {
    t.obj_now(stone)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("deals 2 damage"))
}

#[test]
fn quicksmith_rebel_grants_only_while_you_control_it() {
    cr!("611.2b");
    ruling!(
        "Quicksmith Rebel",
        "If Quicksmith Rebel leaves the battlefield while its ability is on the stack, the artifact never gains the damage-dealing ability."
    );
    ruling!(
        "Quicksmith Rebel",
        "If Quicksmith Rebel leaves the battlefield, you no longer control it."
    );
    supported("Quicksmith Rebel");
    // It resolves: the Stone pings; when the Rebel leaves, it stops.
    let mut t = TestGame::new(2);
    let (stone, rebel_id) = rebel(&mut t);
    t.resolve_all();
    assert!(pings(&t, stone));
    kill(&mut t, rebel_id);
    assert!(!pings(&t, stone));
    // The Rebel leaves with the trigger on the stack: never gains it, even if the Rebel
    // comes back.
    let mut t = TestGame::new(2);
    let (stone, rebel_id) = rebel(&mut t);
    kill(&mut t, rebel_id);
    t.resolve_all();
    assert!(!pings(&t, stone));
    t.battlefield(P0, "Quicksmith Rebel");
    t.settle();
    assert!(!pings(&t, stone));
}

#[test]
fn quicksmith_rebels_ping_resolves_after_either_leaves() {
    cr!("113.7a");
    ruling!(
        "Quicksmith Rebel",
        "If Quicksmith Rebel or the artifact leaves the battlefield while the damage-dealing ability is on the stack, that ability resolves as normal."
    );
    for leaves in ["rebel", "stone"] {
        let mut t = TestGame::new(2);
        let (stone, rebel_id) = rebel(&mut t);
        t.resolve_all();
        t.answer_targets(P0, &[pl(P1)]);
        activate_containing(&mut t, P0, stone, "deals 2 damage")
            .unwrap_or_else(|e| panic!("{leaves}: {e:?}"));
        assert_eq!(t.stack_len(), 1);
        kill(&mut t, if leaves == "rebel" { rebel_id } else { stone });
        t.resolve_all();
        assert_eq!(t.life(P1), 18, "{leaves}");
    }
}

#[test]
fn quicksmith_rebels_grant_ends_for_good_on_a_control_change() {
    cr!("611.2b");
    ruling!(
        "Quicksmith Rebel",
        "If another player gains control of Quicksmith Rebel, the artifact will no longer have the damage-dealing ability, even if you regain control of Quicksmith Rebel."
    );
    let mut t = TestGame::new(2);
    let (stone, rebel_id) = rebel(&mut t);
    t.resolve_all();
    assert!(pings(&t, stone));
    give_control(&mut t, rebel_id, P1);
    assert!(!pings(&t, stone));
    give_control(&mut t, rebel_id, P0);
    assert_eq!(ctl(&t, rebel_id), P0);
    assert!(!pings(&t, stone));
}

/// The candidates offered for the target of the attack trigger of `attacker` (P0's,
/// attacking `target`).
fn attack_trigger_candidates(t: &mut TestGame, attacker: ObjectId, target: Entity) -> Vec<Entity> {
    let from = t.asked().len();
    attack_with(t, &[(attacker, target)]);
    target_candidates(t, P0, from)
        .into_iter()
        .next()
        .unwrap_or_default()
}

#[test]
fn hellkite_whelp_defending_player_is_the_planeswalkers_controller() {
    cr!("508.5", "802.2a");
    ruling!(
        "Hellkite Whelp",
        "If a creature attacks a planeswalker or battle, that permanent's controller is the defending player."
    );
    supported("Hellkite Whelp");
    // Three players: the Whelp attacks P2's planeswalker; P2's creatures can be targeted,
    // P1's can't.
    let mut t = TestGame::new(3);
    let whelp = t.battlefield(P0, "Hellkite Whelp");
    let jace = t.battlefield(P2, "Jace Beleren");
    let p1s = t.battlefield(P1, "Grizzly Bears");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    let cands = attack_trigger_candidates(&mut t, whelp, obj(jace));
    assert_eq!(cands, vec![obj(p2s)], "{cands:?}");
    assert!(!cands.contains(&obj(p1s)));
}

#[test]
fn heart_piercer_bow_defending_player_is_the_battles_protector() {
    cr!("508.5", "802.2a", "310.9e");
    ruling!(
        "Heart-Piercer Bow",
        "If a creature is attacking a battle, the protector of that battle is the defending player."
    );
    ruling!(
        "Heart-Piercer Bow",
        "If a creature is attacking a planeswalker, the controller of that planeswalker is the defending player."
    );
    supported("Heart-Piercer Bow");
    // Three players: P2 controls a Siege that P1 protects. Attacking it, P1 is the
    // defending player.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bow = t.battlefield(P0, "Heart-Piercer Bow");
    assert!(t.g.attach(bow, obj(bears)));
    let battle = t.battlefield(P2, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    t.g.dirty = true;
    let p1s = t.battlefield(P1, "Savannah Lions");
    let p2s = t.battlefield(P2, "Savannah Lions");
    let cands = attack_trigger_candidates(&mut t, bears, obj(battle));
    assert_eq!(cands, vec![obj(p1s)], "{cands:?}");
    assert!(!cands.contains(&obj(p2s)));
    // Attacking P2's planeswalker: P2 is the defending player.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bow = t.battlefield(P0, "Heart-Piercer Bow");
    assert!(t.g.attach(bow, obj(bears)));
    let jace = t.battlefield(P2, "Jace Beleren");
    t.battlefield(P1, "Savannah Lions");
    let p2s = t.battlefield(P2, "Savannah Lions");
    let cands = attack_trigger_candidates(&mut t, bears, obj(jace));
    assert_eq!(cands, vec![obj(p2s)], "{cands:?}");
}

#[test]
fn hellkite_whelp_in_two_headed_giant_targets_only_the_attacked_players_creatures() {
    cr!("805.10e", "810.7");
    ruling!(
        "Hellkite Whelp",
        "In a Two-Headed Giant game, Hellkite Whelp's triggered ability can target only a creature controlled by the player it's attacking. It can't target a creature controlled by that player's teammate."
    );
    let mut t = crate::r_p014_common::thg();
    let whelp = t.battlefield(P0, "Hellkite Whelp");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    let p3s = t.battlefield(P3, "Grizzly Bears");
    let cands = attack_trigger_candidates(&mut t, whelp, pl(P2));
    assert_eq!(cands, vec![obj(p2s)], "{cands:?}");
    assert!(!cands.contains(&obj(p3s)));
}

#[test]
fn rocket_launcher_can_target_differently_each_time() {
    cr!("602.2b", "601.2c");
    ruling!("Rocket Launcher", "You can choose different targets for each use.");
    supported("Rocket Launcher");
    let mut t = TestGame::new(2);
    let launcher = t.battlefield(P0, "Rocket Launcher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.activate(P0, launcher, 0, &[obj(bears)]).expect("first");
    t.activate(P0, launcher, 0, &[pl(P1)]).expect("second");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.life(P1), 19);
}
