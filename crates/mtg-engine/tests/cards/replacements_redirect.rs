//! "All damage that would be dealt to X is dealt to Y instead" — redirection effects
//! (CR 614.9; patterns in `src/oracle/patterns/replacements_redirect.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn redirection_cards_compile() {
    assert_compiles(&[
        "Empyrial Archangel",
        "Pariah",
        "Pariah's Shield",
        "Palisade Giant",
        "Treacherous Link",
        "Veteran Bodyguard",
        "Weathered Bodyguards",
        "Martyrs of Korlis",
        "Sivvi's Valor",
        "Saving Grace",
        "Kjeldoran Royal Guard",
        "Shimian Night Stalker",
        "Turn the Tables",
        "Mirror Strike",
    ]);
}

/// The active player attacks with `attackers`, the defending player declares `blocks`, and
/// the game stops in the declare blockers step with the active player holding priority.
fn to_declare_blockers(
    t: &mut TestGame,
    attackers: &[(ObjectId, Entity)],
    blocks: &[(ObjectId, ObjectId)],
) {
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    if !blocks.is_empty() {
        let dp = t.g.obj(blocks[0].0).controller;
        t.answer(dp, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
}

#[test]
fn empyrial_archangel_takes_the_damage_dealt_to_you() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Empyrial Archangel");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(angel).damage, 3);
}

#[test]
fn pariah_redirects_to_the_enchanted_creature_as_combat_damage() {
    cr!("614.9");
    ruling!(
        "Pariah",
        "If you would be dealt combat damage, the damage dealt to the enchanted creature instead is still combat damage."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let pariah = t.battlefield(P0, "Pariah");
    assert!(t.g.attach(pariah, Entity::Object(wurm)));
    t.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(wurm).damage, 3);

    // Still combat damage: Fog Bank ("Prevent all combat damage that would be dealt to
    // and dealt by this creature") prevents it once it's redirected there, but not
    // noncombat damage.
    let mut t = TestGame::new(2);
    let fog = t.battlefield(P0, "Fog Bank");
    let pariah = t.battlefield(P0, "Pariah");
    assert!(t.g.attach(pariah, Entity::Object(fog)));
    t.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(fog).damage, 0);
    t.g.deal_damage(giant, Entity::Player(P0), 1, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(fog).damage, 1);
}

#[test]
fn pariahs_shield_does_nothing_while_unattached() {
    cr!("614.9");
    ruling!(
        "Pariah's Shield",
        "If Pariah’s Shield isn’t attached to a creature, all damage that would be dealt to you is dealt to you normally."
    );
    let mut t = TestGame::new(2);
    let shield = t.battlefield(P0, "Pariah's Shield");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 17);
    assert!(t.g.attach(shield, Entity::Object(bears)));
    t.recompute();
    t.g.deal_damage(giant, Entity::Player(P0), 1, false);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn palisade_giant_takes_damage_for_you_and_your_other_permanents() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Palisade Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, Entity::Player(P0), 2, true);
    t.g.deal_damage(src, Entity::Object(bears), 2, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 4);
    // Damage to the opponent's creatures isn't redirected.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(theirs), 1, false);
    assert_eq!(t.obj_now(theirs).damage, 1);
}

#[test]
fn treacherous_link_sends_the_damage_to_the_creatures_controller() {
    cr!("614.9");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let link = t.battlefield(P0, "Treacherous Link");
    assert!(t.g.attach(link, Entity::Object(theirs)));
    t.recompute();
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(theirs), 3, true);
    assert_eq!(t.obj_now(theirs).damage, 0);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn veteran_bodyguard_takes_damage_from_unblocked_creatures_while_untapped() {
    cr!("614.9", "702.19b", "509.1h");
    ruling!(
        "Veteran Bodyguard",
        "If a creature is blocked but Trample damage is still done to a player, this damage can't be redirected to the Bodyguard"
    );
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Veteran Bodyguard");
    let wall = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(giant, Entity::Player(P0)), (maw, Entity::Player(P0))],
        &[(wall, maw)],
    );
    // The unblocked Giant's 3 goes to the Bodyguard; the blocked Dreadmaw's 4 excess
    // trample damage is dealt to P0 (it's a blocked creature, CR 509.1h).
    assert_eq!(t.obj_now(guard).damage, 3);
    assert!(!t.on_battlefield(wall));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn veteran_bodyguard_doesnt_redirect_while_tapped() {
    cr!("614.9");
    ruling!(
        "Veteran Bodyguard",
        "Damage goes to the Bodyguard as long as he is untapped."
    );
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Veteran Bodyguard");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(guard);
    t.recompute();
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.obj_now(guard).damage, 0);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn sivvis_valor_redirects_damage_from_the_target_creature_to_you() {
    cr!("614.9", "115.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let valor = t.hand(P0, "Sivvi's Valor");
    t.cast(P0, valor).target(bears).go();
    t.resolve();
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(bears), 3, false);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P0), 17);
    // Other creatures aren't affected.
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(giant, Entity::Object(other), 1, false);
    assert_eq!(t.obj_now(other).damage, 1);
}

#[test]
fn kjeldoran_royal_guard_redirect_ends_if_it_leaves() {
    cr!("614.9");
    ruling!(
        "Kjeldoran Royal Guard",
        "If you activate the ability but Kjeldoran Royal Guard leaves the battlefield before combat damage is dealt, the combat damage from unblocked creatures won’t be redirected."
    );
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Kjeldoran Royal Guard");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.activate(P0, guard, 0, &[]).unwrap();
    t.resolve();
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(guard).damage, 3);

    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Kjeldoran Royal Guard");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.activate(P0, guard, 0, &[]).unwrap();
    t.resolve();
    t.lands(P1, "Swamp", 2);
    let kill = t.hand(P1, "Murder");
    t.lands(P1, "Swamp", 1);
    t.cast(P1, kill).target(guard).go();
    t.resolve();
    assert!(!t.on_battlefield(guard));
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn turn_the_tables_sends_combat_damage_to_the_target_attacker() {
    cr!("614.9", "115.1");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_declare_blockers(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[],
    );
    t.lands(P0, "Plains", 5);
    let tables = t.hand(P0, "Turn the Tables");
    t.cast(P0, tables).target(giant).go();
    t.resolve();
    t.advance_to(P1, Step::EndOfCombat);
    // All the combat damage that would have been dealt to P0 went to the Giant: its own
    // 3 and the Bears' 2.
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn turn_the_tables_does_nothing_once_the_target_is_gone() {
    cr!("614.9");
    ruling!(
        "Turn the Tables",
        "If the targeted creature isn’t on the battlefield when the combat damage would be dealt, the damage isn’t redirected."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_declare_blockers(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[],
    );
    t.lands(P0, "Plains", 5);
    let tables = t.hand(P0, "Turn the Tables");
    t.cast(P0, tables).target(bears).go();
    t.resolve();
    t.lands(P0, "Island", 1);
    let bounce = t.hand(P0, "Unsummon");
    t.cast(P0, bounce).target(bears).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 17);
    assert!(t.on_battlefield(giant));
}

#[test]
fn mirror_strike_sends_the_damage_to_the_attackers_controller() {
    cr!("614.9", "115.1");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_declare_blockers(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[],
    );
    t.lands(P0, "Plains", 4);
    let strike = t.hand(P0, "Mirror Strike");
    t.cast(P0, strike).target(giant).go();
    t.resolve();
    t.advance_to(P1, Step::EndOfCombat);
    // Only the target's damage is redirected, and to its controller.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 18);
    assert!(t.on_battlefield(giant));
}

#[test]
fn saving_grace_keeps_redirecting_to_the_creature_it_enchanted() {
    cr!("614.9", "611.2c");
    ruling!(
        "Saving Grace",
        "If Saving Grace leaves the battlefield during the turn its triggered ability resolved, damage will continue to be redirected to the creature it enchanted before it left the battlefield."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let grace = t.hand(P0, "Saving Grace");
    t.cast(P0, grace).target(wurm).go();
    t.resolve_all();
    let aura = t.named_on_battlefield("Saving Grace");
    assert_eq!(aura.len(), 1);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.g.deal_damage(giant, Entity::Object(bears), 1, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(wurm).damage, 3);
    // The Aura leaves: the damage still goes to the creature it enchanted.
    t.g.destroy(aura[0], None);
    t.settle();
    t.g.deal_damage(giant, Entity::Player(P0), 1, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(wurm).damage, 4);
    // Damage to an opponent's creature isn't redirected.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(giant, Entity::Object(theirs), 1, false);
    assert_eq!(t.obj_now(theirs).damage, 1);
}
