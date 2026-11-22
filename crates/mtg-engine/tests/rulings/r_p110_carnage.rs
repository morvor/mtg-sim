//! Rulings batch P110 — Maximum Carnage's chapter I ("Until your next turn, each creature
//! attacks each combat if able and attacks a player other than you if able."): attack
//! requirements (CR 508.1c, 508.1d) that apply to every creature, including ones that
//! arrive later (CR 611.2c doesn't apply: it changes rules, not objects), without goading
//! anything (CR 701.15).

use crate::r_p110_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Maximum Carnage resolves its chapter I ability during P0's turn.
fn carnage(t: &mut TestGame) -> ObjectId {
    supported("Maximum Carnage");
    let saga = t.battlefield(P0, "Maximum Carnage");
    add_lore(t, saga, 1);
    t.resolve_all();
    saga
}

/// Advances to P1's declare attackers step (P1 declaring no attackers unless required)
/// and returns what each creature attacks.
fn p1_attacks(t: &mut TestGame, creatures: &[ObjectId]) -> Vec<Option<Entity>> {
    t.advance_to(P1, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().expect("combat");
    creatures
        .iter()
        .map(|id| c.attacker(*id).and_then(|a| a.target))
        .collect()
}

#[test]
fn maximum_carnage_affects_creatures_that_arrive_later() {
    cr!("508.1d", "611.2c");
    ruling!(
        "Maximum Carnage",
        "Maximum Carnage's first chapter ability affects all creatures until your next turn, regardless of whether or not they were on the battlefield at the time the ability resolved."
    );
    let mut t = TestGame::new(2);
    carnage(&mut t);
    // P1's Grizzly Bears arrives after the ability resolved.
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![Some(Entity::Player(P0))]);
}

#[test]
fn maximum_carnage_a_creature_attacks_you_if_it_cant_attack_anyone_else() {
    cr!("508.1d", "508.1c", "508.1h");
    ruling!(
        "Maximum Carnage",
        "However, if such a creature can't attack players other than you, it must attack you (unless you control it) or a planeswalker or battle it can attack."
    );
    // Three players: attacking P2 (Propaganda) has a cost, so P1's Bears isn't required
    // to; it must attack P0 instead.
    let mut t = TestGame::new(3);
    carnage(&mut t);
    t.battlefield(P2, "Propaganda");
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![Some(Entity::Player(P0))]);
    // Without Propaganda, it attacks P2 (a player other than P0).
    let mut t = TestGame::new(3);
    carnage(&mut t);
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![Some(Entity::Player(P2))]);
}

#[test]
fn maximum_carnage_doesnt_force_unable_or_costly_attacks() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Maximum Carnage",
        "If a creature under the effects of the first chapter ability can't attack for any reason (such as being tapped or having come under your control that turn), then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    // A tapped Bears doesn't attack; one that would have to pay for Ghostly Prison doesn't
    // have to.
    let mut t = TestGame::new(2);
    carnage(&mut t);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.g.objects[tapped.0 as usize].tapped = true;
    assert_eq!(p1_attacks(&mut t, &[tapped]), vec![None]);
    let mut t = TestGame::new(2);
    carnage(&mut t);
    t.battlefield(P0, "Ghostly Prison");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![None]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn maximum_carnage_doesnt_goad() {
    cr!("701.15a", "701.15b");
    ruling!(
        "Maximum Carnage",
        "Although the effects of the first chapter ability are the same as the goad keyword action, that ability doesn't cause any creatures to become goaded. Effects that refer to \"goaded creatures\" won't apply."
    );
    supported("Vengeful Ancestor");
    // P0's Vengeful Ancestor: "Whenever a goaded creature attacks, it deals 1 damage to
    // its controller." P1's Bears attacks without being goaded.
    let mut t = TestGame::new(2);
    carnage(&mut t);
    // (Summoning sick, so it doesn't have to attack during P0's turn.)
    t.battlefield_sick(P0, "Vengeful Ancestor");
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert!(t.g.goaders(bear).is_empty());
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![Some(Entity::Player(P0))]);
    assert!(t.g.goaders(bear).is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.g.pending_triggers.is_empty());
    // A goaded Bears (goaded by the Ancestor's enters ability) is dealt that 1 damage.
    let mut t = TestGame::new(2);
    carnage(&mut t);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bear)]);
    t.enter(P0, "Vengeful Ancestor");
    t.resolve_all();
    assert_eq!(t.g.goaders(bear), vec![P0]);
    assert_eq!(p1_attacks(&mut t, &[bear]), vec![Some(Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
