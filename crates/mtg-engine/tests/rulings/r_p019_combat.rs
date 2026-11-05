//! Rulings batch P019 — abilities that change what an attacking creature attacks or what a
//! blocker blocks: Capricopian ("{2}: Put a +1/+1 counter on this creature, then you may
//! reselect which player this creature is attacking. Only the player this creature is
//! attacking may activate this ability and only during the declare attackers step.", CR
//! 508.7) and Balduvian Warlord ("{T}: Remove target blocking creature from combat.
//! Creatures it was blocking that hadn't become blocked by another creature this combat
//! become unblocked, then it blocks an attacking creature of your choice. Activate only
//! during the declare blockers step.", CR 506.4, 509.1h).

use crate::r_p148_common::declare_blocks;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A three-player game in which a creature may attack any opponent.
fn three_players() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

/// P0's Capricopian (with two +1/+1 counters) attacks `at`; the game is in the declare
/// attackers step. Returns it.
fn goat_attacks(t: &mut TestGame, at: Entity) -> ObjectId {
    let goat = t.battlefield(P0, "Capricopian");
    t.g.add_counters(Entity::Object(goat), counters::PLUS1, 2, None);
    attack_with(t, &[(goat, at)]);
    goat
}

/// What the attacking creature is attacking now.
fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat.as_ref().and_then(|c| c.attack_target(id))
}

/// `p` activates Capricopian's ability, reselecting `to`.
fn reselect(t: &mut TestGame, p: PlayerId, goat: ObjectId, to: PlayerId) {
    t.lands(p, "Forest", 2);
    t.answer_yes(p, true);
    t.answer_choose(p, &[Entity::Player(to)]);
    t.activate(p, goat, 0, &[]).expect("Capricopian's ability");
    t.resolve_all();
}

#[test]
fn a_reselected_creature_still_attacked_the_player_it_was_declared_attacking() {
    cr!("508.7", "508.7a");
    ruling!(
        "Capricopian",
        "If you reselect which player an attacking creature is attacking, that creature is still considered to have attacked the player as declared, but it is now attacking the new player."
    );
    supported("Capricopian");
    supported("Avenge");
    // P1 sends it to P2. On P1's turn, Avenge ("This spell costs {2} less to cast if a
    // player attacked you during their last turn.") costs P1 four mana.
    let mut t = three_players();
    let goat = goat_attacks(&mut t, Entity::Player(P1));
    reselect(&mut t, P1, goat, P2);
    assert_eq!(attacking(&t, goat), Some(Entity::Player(P2)));
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 2);
    t.lands(P1, "Wastes", 2);
    let avenge = t.hand(P1, "Avenge");
    t.g.turn.priority = Some(P1);
    assert!(t.g.cast_spell(P1, avenge, CastMethod::Normal).is_ok());
}

#[test]
fn reselecting_doesnt_trigger_attack_abilities_and_ignores_attack_costs() {
    cr!("508.7a", "508.7b", "508.1h");
    ruling!(
        "Capricopian",
        "Reselecting which player a creature is attacking doesn’t cause “whenever this creature attacks” abilities to trigger."
    );
    ruling!(
        "Capricopian",
        "Reselecting which player a creature is attacking ignores all requirements, restrictions, and costs associated with attacking."
    );
    supported("Armory of Iroas");
    supported("Ghostly Prison");
    // Armory of Iroas: "Whenever equipped creature attacks, put a +1/+1 counter on it."
    // P2's Ghostly Prison: "Creatures can't attack you unless their controller pays {2}
    // for each creature they control that's attacking you."
    let mut t = three_players();
    t.battlefield(P2, "Ghostly Prison");
    let goat = t.battlefield(P0, "Capricopian");
    t.g.add_counters(Entity::Object(goat), counters::PLUS1, 2, None);
    crate::r_s06_common::attach_new(&mut t, P0, "Armory of Iroas", goat);
    attack_with(&mut t, &[(goat, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(goat, counters::PLUS1), 3);
    let from = t.asked().len();
    reselect(&mut t, P1, goat, P2);
    // One counter from the ability itself; no new attack trigger, and P0 wasn't asked to
    // pay for the Prison.
    assert_eq!(t.counters(goat, counters::PLUS1), 4);
    assert_eq!(triggers_on_stack(&t, "attacks"), 0);
    assert_eq!(attacking(&t, goat), Some(Entity::Player(P2)));
    assert!(!t.asked()[from..].iter().any(|(p, _)| *p == P0));
}

#[test]
fn the_new_defending_player_may_send_capricopian_back() {
    cr!("508.7", "602.5");
    ruling!(
        "Capricopian",
        "The player Capricopian becomes attacking may activate its ability, perhaps sending Capricopian right back towards its original destination."
    );
    let mut t = three_players();
    let goat = goat_attacks(&mut t, Entity::Player(P1));
    reselect(&mut t, P1, goat, P2);
    reselect(&mut t, P2, goat, P1);
    assert_eq!(attacking(&t, goat), Some(Entity::Player(P1)));
    assert_eq!(t.counters(goat, counters::PLUS1), 4);
}

#[test]
fn capricopian_cant_be_sent_at_or_away_from_a_planeswalker() {
    cr!("508.7c", "602.5");
    ruling!(
        "Capricopian",
        "The player Capricopian is attacking can’t reselect it to attack a planeswalker instead. If Capricopian is attacking a planeswalker, no player may activate its last ability."
    );
    supported("Chandra Nalaar");
    // Attacking P1: P1 may only choose a player (not P2's planeswalker).
    let mut t = three_players();
    let walker = t.battlefield(P2, "Chandra Nalaar");
    let goat = goat_attacks(&mut t, Entity::Player(P1));
    t.lands(P1, "Forest", 2);
    t.answer_yes(P1, true);
    let from = t.asked().len();
    t.activate(P1, goat, 0, &[]).unwrap();
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.is_empty());
    assert!(!offered.contains(&Entity::Object(walker)));
    assert!(matches!(attacking(&t, goat), Some(Entity::Player(_))));
    // Attacking P2's planeswalker: no player may activate it.
    let mut t = three_players();
    let walker = t.battlefield(P2, "Chandra Nalaar");
    let goat = goat_attacks(&mut t, Entity::Object(walker));
    for p in [P0, P1, P2] {
        t.lands(p, "Forest", 2);
        assert!(t.activate(p, goat, 0, &[]).is_err(), "{p:?}");
    }
}

/// P0 attacks P1 with `attackers`; P1 blocks with `blocks`; the game stops in the declare
/// blockers step.
fn combat(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(t, &decl);
    t.resolve_all();
    declare_blocks(t, P1, blocks);
    t.resolve_all();
}

/// `p` activates Balduvian Warlord targeting `blocker`; it then blocks `attacker`.
fn warlord(
    t: &mut TestGame,
    p: PlayerId,
    warlord: ObjectId,
    blocker: ObjectId,
    attacker: ObjectId,
) {
    t.answer_choose(p, &[Entity::Object(attacker)]);
    t.activate(p, warlord, 0, &[Entity::Object(blocker)])
        .expect("Balduvian Warlord's ability");
    t.settle();
}

#[test]
fn balduvian_warlord_works_on_any_players_turn() {
    cr!("509.1h", "506.4");
    ruling!(
        "Balduvian Warlord",
        "Balduvian Warlord’s ability may be activated during any player’s turn."
    );
    supported("Balduvian Warlord");
    // The attacking player P0 moves P1's Wall from the Hill Giant to the Bears.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Balduvian Warlord");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    combat(&mut t, &[giant, bears], &[(wall, giant)]);
    warlord(&mut t, P0, lord, wall, bears);
    t.resolve_all();
    let c = t.g.combat.as_ref().unwrap();
    assert!(!c.is_blocked(giant));
    assert!(c.is_blocked(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn balduvian_warlord_blocks_cause_block_triggers() {
    cr!("509.3a", "509.3c", "509.3d");
    ruling!(
        "Balduvian Warlord",
        "If an attacking creature has a “When this creature becomes blocked by [a creature]” ability, it triggers when a creature blocks it due to the Warlord’s ability."
    );
    ruling!(
        "Balduvian Warlord",
        "If an attacking creature has an ability that triggers “When this creature becomes blocked,” it triggers when a creature blocks it due to the Warlord’s ability only if it was unblocked at that point."
    );
    ruling!(
        "Balduvian Warlord",
        "If the targeted creature has a “When this creature blocks” ability, it will trigger when Balduvian Warlord’s ability resolves and the creature blocks again."
    );
    supported("Somberwald Vigilante");
    supported("Chambered Nautilus");
    supported("Pride Guardian");
    // P1's Pride Guardian ("Whenever this creature blocks, you gain 3 life.") blocks the
    // Bears, then is moved onto Somberwald Vigilante ("Whenever this creature becomes
    // blocked by a creature, this creature deals 1 damage to that creature.") and Chambered
    // Nautilus ("Whenever this creature becomes blocked, you may draw a card.").
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P1, "Balduvian Warlord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let vigilante = t.battlefield(P0, "Somberwald Vigilante");
    let guardian = t.battlefield(P1, "Pride Guardian");
    combat(&mut t, &[bears, vigilante], &[(guardian, bears)]);
    assert_eq!(t.life(P1), 23);
    warlord(&mut t, P1, lord, guardian, vigilante);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "blocks, you gain 3"), 1);
    assert_eq!(triggers_on_stack(&t, "becomes blocked by a creature"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 26);
    assert_eq!(t.obj_now(guardian).damage, 1);

    // Chambered Nautilus, unblocked: it becomes blocked and its ability triggers.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P1, "Balduvian Warlord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let nautilus = t.battlefield(P0, "Chambered Nautilus");
    let wall = t.battlefield(P1, "Wall of Wood");
    combat(&mut t, &[bears, nautilus], &[(wall, bears)]);
    warlord(&mut t, P1, lord, wall, nautilus);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "becomes blocked, you may draw"), 1);
    // Already blocked by another creature: it doesn't trigger again.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P1, "Balduvian Warlord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let nautilus = t.battlefield(P0, "Chambered Nautilus");
    let wall = t.battlefield(P1, "Wall of Wood");
    let other = t.battlefield(P1, "Wall of Wood");
    t.answer_yes(P0, false);
    combat(
        &mut t,
        &[bears, nautilus],
        &[(wall, bears), (other, nautilus)],
    );
    warlord(&mut t, P1, lord, wall, nautilus);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "becomes blocked, you may draw"), 0);
}
