//! Rulings batch P227 — vigilance (CR 702.20): vigilance gained or lost after attackers
//! are declared, statics granting vigilance (and abilities to creatures with vigilance),
//! and counting creatures with vigilance.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::has_kw;
use crate::r_s10_common::attacking;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// P0 controls `granter` (which gives `attacker` vigilance) and attacks with `attacker`;
/// then the granter is destroyed: the attacker stays untapped and attacking, and deals its
/// combat damage.
fn loses_vigilance_after_attacking(granter: &str, attacker: &str) {
    supported(granter);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, granter);
    let a = t.battlefield(P0, attacker);
    assert!(has_kw(&t, a, KeywordKind::Vigilance), "{granter}");
    let power = t.pt(a).0;
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert!(attacking(&t, a));
    assert!(!tapped(&t, a));
    destroy(&mut t, g);
    assert!(!t.on_battlefield(g));
    assert!(!has_kw(&t, a, KeywordKind::Vigilance));
    assert!(!tapped(&t, a), "{granter}: the attacker became tapped");
    assert!(attacking(&t, a));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - power);
    assert!(!tapped(&t, a));
}

#[test]
fn serras_guardian_leaving_doesnt_tap_attackers() {
    cr!("702.20b", "508.1f", "506.4");
    ruling!(
        "Serra's Guardian",
        "A creature losing vigilance after it has attacked (most likely because Serra’s Guardian leaves the battlefield) won’t cause it to become tapped."
    );
    loses_vigilance_after_attacking("Serra's Guardian", "Grizzly Bears");
}

#[test]
fn esika_leaving_doesnt_tap_attackers() {
    cr!("702.20b", "508.1f", "506.4");
    ruling!(
        "Esika, God of the Tree // The Prismatic Bridge",
        "If a creature loses vigilance after it attacks (perhaps because Esika leaves the battlefield), that creature will continue attacking. It won't become tapped."
    );
    loses_vigilance_after_attacking(
        "Esika, God of the Tree // The Prismatic Bridge",
        "Isamaru, Hound of Konda",
    );
}

#[test]
fn herald_of_dromoka_leaving_doesnt_tap_attackers() {
    cr!("702.20b", "508.1f", "506.4");
    ruling!(
        "Herald of Dromoka",
        "If an attacking creature loses vigilance, it remains untapped and attacking."
    );
    loses_vigilance_after_attacking("Herald of Dromoka", "Hardy Veteran");
}

#[test]
fn captain_of_the_watch_affects_all_other_soldiers() {
    cr!("613.4c", "611.3a");
    ruling!(
        "Captain of the Watch",
        "Captain of the Watch grants +1/+1 and vigilance to all other Soldier creatures you control, not just the ones its ability puts onto the battlefield."
    );
    supported("Captain of the Watch");
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Elite Vanguard"); // a 2/1 Human Soldier
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Elite Vanguard");
    let captain = t.enter(P0, "Captain of the Watch");
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 5); // the Captain, the Vanguard and three tokens
    for s in soldiers.iter().filter(|s| **s != captain) {
        assert!(has_kw(&t, *s, KeywordKind::Vigilance));
    }
    assert_eq!(t.pt(vanguard), (3, 2));
    assert!(has_kw(&t, vanguard, KeywordKind::Vigilance));
    for tok in tokens(&t, P0) {
        assert_eq!(t.pt(tok), (2, 2));
    }
    // Not itself, not non-Soldiers, not other players' Soldiers.
    assert_eq!(t.pt(captain), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has_kw(&t, bears, KeywordKind::Vigilance));
    assert_eq!(t.pt(theirs), (2, 1));
    assert!(!has_kw(&t, theirs, KeywordKind::Vigilance));
}

#[test]
fn frondland_felidar_has_its_own_granted_ability() {
    cr!("613.1f", "602.2");
    ruling!(
        "Frondland Felidar",
        "Frondland Felidar’s last ability applies to itself as long as it still has vigilance."
    );
    supported("Frondland Felidar");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let felidar = t.battlefield(P0, "Frondland Felidar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    // Vigilance, the static, and the granted "{1}, {T}: Tap target creature."
    t.activate(P0, felidar, 0, &[Entity::Object(bears)])
        .expect("Felidar activates its granted ability");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert!(tapped(&t, felidar));
    // A creature without vigilance doesn't have it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frondland Felidar");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!t
        .obj_now(mine)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))));
}

#[test]
fn loxodon_sergeant_vigilance_after_attacking_doesnt_untap() {
    cr!("702.20b", "508.1f", "611.2c");
    ruling!(
        "Loxodon Sergeant",
        "Gaining vigilance any time after the moment you choose to attack with a creature won’t cause it to become untapped."
    );
    supported("Loxodon Sergeant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(tapped(&t, bears));
    // The Sergeant enters during combat (as if flashed in): the Bears gain vigilance but
    // stay tapped.
    t.enter(P0, "Loxodon Sergeant");
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    assert!(tapped(&t, bears));
    assert!(attacking(&t, bears));
}

#[test]
fn loxodon_sergeant_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Loxodon Sergeant",
        "Loxodon Sergeant’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain vigilance."
    );
    supported("Loxodon Sergeant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Loxodon Sergeant");
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    let late = t.battlefield(P0, "Hill Giant");
    assert!(!has_kw(&t, late, KeywordKind::Vigilance));
    // The late creature taps to attack; the Bears don't.
    t.g.objects[late.0 as usize].summoning_sick = false;
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (late, Entity::Player(P1))],
    );
    assert!(!tapped(&t, bears));
    assert!(tapped(&t, late));
}

#[test]
fn alert_heedbonder_counts_as_its_ability_resolves() {
    cr!("608.2h", "603.2");
    ruling!(
        "Alert Heedbonder",
        "The number of creatures you control with vigilance is determined as Alert Heedbonder’s last ability resolves. If it’s still on the battlefield, the count will include Alert Heedbonder itself."
    );
    supported("Alert Heedbonder");
    // The Heedbonder and Herald of Dromoka have vigilance; Grizzly Bears doesn't: 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alert Heedbonder");
    t.battlefield(P0, "Herald of Dromoka");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // With the trigger on the stack, the Heedbonder leaves and another creature with
    // vigilance arrives: still counted as it resolves (1 + 1 = 2, not the Heedbonder).
    let mut t = TestGame::new(2);
    let heed = t.battlefield(P0, "Alert Heedbonder");
    t.battlefield(P0, "Herald of Dromoka");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, heed);
    t.battlefield(P0, "Serra Angel");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Just the Heedbonder leaving: the Herald alone.
    let mut t = TestGame::new(2);
    let heed = t.battlefield(P0, "Alert Heedbonder");
    t.battlefield(P0, "Herald of Dromoka");
    t.advance_to(P0, Step::End);
    t.settle();
    destroy(&mut t, heed);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn village_survivors_vigilance_after_paying_an_attack_cost_doesnt_untap() {
    cr!("508.1f", "508.1g", "508.1h", "702.20b");
    ruling!(
        "Village Survivors",
        "Tapping creatures to attack with them happens just before any required costs to attack with those creatures are paid. If paying such a cost causes your life total to fall below 6, your tapped attackers will have vigilance, but the ability will have no benefit that combat."
    );
    supported("Village Survivors");
    supported("Norn's Annex");
    // P0 is at 7 life; P1's Norn's Annex makes attacking P1 cost {W/P} per creature,
    // paid with 2 life: P0 drops to 5 and the Bears get vigilance but stay tapped.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 7;
    t.battlefield(P0, "Village Survivors");
    t.battlefield(P1, "Norn's Annex");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!has_kw(&t, bears, KeywordKind::Vigilance));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(attacking(&t, bears));
    assert_eq!(t.life(P0), 5);
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
    assert!(tapped(&t, bears));
}
