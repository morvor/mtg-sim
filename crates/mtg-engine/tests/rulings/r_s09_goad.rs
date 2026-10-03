//! Rulings batch S09 — goad (CR 701.15): until the goading player's next turn, a goaded
//! creature attacks each combat if able and attacks a player other than the goading
//! player if able (CR 701.15a–b). Each player who goads it adds requirements (701.15c).

use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s09_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Disrupt Decorum ("Goad all creatures you don't control.") in their main
/// phase and it resolves.
fn disrupt_decorum(t: &mut TestGame, p: PlayerId) {
    supported("Disrupt Decorum");
    t.set_step(p, Step::PrecombatMain);
    t.lands(p, "Mountain", 4);
    let dd = t.hand(p, "Disrupt Decorum");
    t.cast(p, dd).go();
    t.resolve_all();
}

/// Attaches `p`'s Aura that goads the enchanted creature (e.g. Acquired Mutation) to
/// `creature`.
fn goading_aura(t: &mut TestGame, p: PlayerId, aura: &str, creature: ObjectId) -> ObjectId {
    supported(aura);
    let a = attach_new(t, p, aura, creature);
    t.settle();
    a
}

// ---------------------------------------------------------------------------
// Being goaded isn't an ability.
// ---------------------------------------------------------------------------

#[test]
fn a_goaded_creature_that_loses_all_abilities_still_must_attack() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Disrupt Decorum",
        "Being goaded isn't an ability the creature has. Once it's been goaded, it must attack as detailed above even if it loses all abilities."
    );
    supported("Humility");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    disrupt_decorum(&mut t, P0);
    // Humility then takes away all of the Angel's abilities.
    t.battlefield(P1, "Humility");
    t.settle();
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    assert_eq!(t.pt(angel), (1, 1));
    assert_eq!(t.g.goaders(angel), vec![P0]);
    // P1 declares no attackers, but the Angel must attack.
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(angel, Entity::Player(P0))]);
}

#[test]
fn a_creature_goaded_by_an_aura_must_attack_even_without_abilities() {
    cr!("701.15b", "613.1f");
    ruling!(
        "Acquired Mutation",
        "Being goaded isn’t an ability the creature has. Once it’s been goaded, it must attack as detailed above even if it loses all abilities."
    );
    supported("Humility");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    goading_aura(&mut t, P0, "Acquired Mutation", angel);
    t.battlefield(P1, "Humility");
    t.settle();
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    assert_eq!(t.pt(angel), (3, 3));
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(angel, Entity::Player(P0))]);
}

// ---------------------------------------------------------------------------
// Exceptions: tapped, "can't attack", summoning sick, costs to attack.
// ---------------------------------------------------------------------------

#[test]
fn tapped_restricted_or_summoning_sick_goaded_creatures_dont_attack() {
    cr!("701.15b", "508.1a", "508.1c", "302.6");
    ruling!(
        "Disrupt Decorum",
        "If, during a player's declare attackers step, a creature that player controls that's been goaded is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under that player's control continuously since the turn began (and doesn't have haste), then it doesn't attack."
    );
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    let pacified = t.battlefield(P1, "Hill Giant");
    let sick = t.battlefield_sick(P1, "Llanowar Elves");
    let able = t.battlefield(P1, "Serra Angel");
    disrupt_decorum(&mut t, P0);
    for c in [tapped, pacified, sick, able] {
        assert_eq!(t.g.goaders(c), vec![P0]);
    }
    t.g.tap(tapped);
    attach_new(&mut t, P0, "Pacifism", pacified);
    t.settle();
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(able, Entity::Player(P0))]);
}

#[test]
fn a_goaded_creature_isnt_forced_to_pay_a_cost_to_attack() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Disrupt Decorum",
        "If there's a cost associated with having a creature attack a player, its controller isn't forced to pay that cost, so it doesn't have to attack that player."
    );
    supported("Propaganda");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    disrupt_decorum(&mut t, P0);
    // Attacking P0 costs {2}: P1 could pay it, but doesn't have to.
    t.battlefield(P0, "Propaganda");
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
    assert!(declare(&mut t, P1, &[]).is_empty());
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn an_aura_goaded_creature_follows_the_same_exceptions() {
    cr!("701.15b", "508.1a", "508.1d");
    ruling!(
        "Acquired Mutation",
        "If, during a player’s declare attackers step, a creature that player controls that’s been goaded is tapped"
    );
    supported("Propaganda");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    goading_aura(&mut t, P0, "Acquired Mutation", bears);
    // Tapped: it doesn't attack.
    t.g.tap(bears);
    assert!(declare(&mut t, P1, &[]).is_empty());
    // Untapped, but attacking P0 costs {2} (Propaganda): it still doesn't have to.
    t.g.untap(bears);
    let prop = t.battlefield(P0, "Propaganda");
    assert!(declare(&mut t, P1, &[]).is_empty());
    // Without the cost, it attacks.
    move_to(&mut t, prop, mtg_engine::object::Zone::Graveyard(P0));
    assert_eq!(
        declare(&mut t, P1, &[]),
        vec![(bears, Entity::Player(P0))]
    );
}

// ---------------------------------------------------------------------------
// Attacking doesn't end goad: additional combats and control changes.
// ---------------------------------------------------------------------------

#[test]
fn a_goaded_creature_attacks_again_in_an_additional_combat_and_for_a_new_controller() {
    cr!("701.15a", "701.15b", "500.8");
    ruling!(
        "Disrupt Decorum",
        "Attacking with a goaded creature doesn't cause it to stop being goaded. If there is an additional combat phase that turn, or if another player gains control of it before it stops being goaded, it must attack again if able."
    );
    supported("Relentless Assault");
    supported("Act of Treason");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    disrupt_decorum(&mut t, P0);
    // P1's first combat: the Bears attack P2 (not P0).
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(bears, Entity::Player(P2))]);
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(t.life(P2), 18);
    // An additional combat phase: they're still goaded and attack again.
    t.lands(P1, "Mountain", 4);
    let ra = t.hand(P1, "Relentless Assault");
    t.cast(P1, ra).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    let attacks = declare_in_next_combat(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(bears, Entity::Player(P2))]);
    // P2 gains control of them (with haste): they must attack again, not P0.
    t.set_step(P2, Step::PrecombatMain);
    t.lands(P2, "Mountain", 3);
    let aot = t.hand(P2, "Act of Treason");
    t.cast(P2, aot).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P2);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    let attacks = declare(&mut t, P2, &[]);
    assert_eq!(attacks, vec![(bears, Entity::Player(P1))]);
}

#[test]
fn an_aura_goaded_creature_attacks_again_for_its_new_controller() {
    cr!("701.15b", "500.8");
    ruling!(
        "Acquired Mutation",
        "Attacking with a goaded creature doesn’t cause it to stop being goaded."
    );
    supported("Relentless Assault");
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    goading_aura(&mut t, P0, "Acquired Mutation", bears);
    // P1's turn: the Bears attack, then attack again in an additional combat.
    assert_eq!(declare(&mut t, P1, &[]), vec![(bears, Entity::Player(P0))]);
    t.advance_to(P1, Step::PostcombatMain);
    t.lands(P1, "Mountain", 4);
    let ra = t.hand(P1, "Relentless Assault");
    t.cast(P1, ra).go();
    t.resolve_all();
    assert_eq!(
        declare_in_next_combat(&mut t, P1, &[]),
        vec![(bears, Entity::Player(P0))]
    );
    // The goading player gains control of it: it attacks a player other than them.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let aot = t.hand(P0, "Act of Treason");
    t.cast(P0, aot).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(declare(&mut t, P0, &[]), vec![(bears, Entity::Player(P1))]);
}

// ---------------------------------------------------------------------------
// Whom a goaded creature attacks.
// ---------------------------------------------------------------------------

/// Three players; `setup` goads P1's Bears for P0. P2 controls Blazing Archon ("Creatures
/// can't attack you.") and P0 controls Jace Beleren. The Bears can attack only P0 or
/// Jace: they must attack one of them.
fn attacks_goader_or_planeswalker_when_no_one_else_can_be_attacked(
    setup: fn(&mut TestGame, ObjectId),
) {
    supported("Blazing Archon");
    supported("Jace Beleren");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    setup(&mut t, bears);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    t.battlefield(P2, "Blazing Archon");
    let jace = t.battlefield(P0, "Jace Beleren");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(jace))]));
    // Declaring no attackers is replaced by a legal declaration: the Bears attack.
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks.len(), 1);
    assert!(matches!(
        attacks[0],
        (b, Entity::Player(P0)) | (b, Entity::Object(_)) if b == bears
    ));
    // And a declaration attacking Jace is kept.
    let mut t2 = TestGame::new(3);
    let bears = t2.battlefield(P1, "Grizzly Bears");
    setup(&mut t2, bears);
    t2.battlefield(P2, "Blazing Archon");
    let jace = t2.battlefield(P0, "Jace Beleren");
    let attacks = declare(&mut t2, P1, &[(bears, Entity::Object(jace))]);
    assert_eq!(attacks, vec![(bears, Entity::Object(jace))]);
}

#[test]
fn goaded_creature_that_cant_attack_others_attacks_a_planeswalker_or_the_goader() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Disrupt Decorum",
        "If the creature can't attack any of those players but could otherwise attack, it must attack a planeswalker an opponent controls, a battle an opponent controls, or a player who goaded it."
    );
    attacks_goader_or_planeswalker_when_no_one_else_can_be_attacked(|t, _| {
        disrupt_decorum(t, P0);
    });
}

#[test]
fn coercive_impetus_goaded_creature_attacks_a_planeswalker_or_the_goader() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Coercive Impetus",
        "If the creature can't attack any of those players but could otherwise attack, it must attack a planeswalker an opponent controls, a battle an opponent protects, or the player that goaded it."
    );
    attacks_goader_or_planeswalker_when_no_one_else_can_be_attacked(|t, bears| {
        goading_aura(t, P0, "Coercive Impetus", bears);
    });
}

#[test]
fn acquired_mutation_goaded_creature_attacks_a_planeswalker_or_the_goader() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Acquired Mutation",
        "If the creature can’t attack any of those players but could otherwise attack, it must attack a planeswalker an opponent controls, a battle an opponent protects, or the player that goaded it."
    );
    attacks_goader_or_planeswalker_when_no_one_else_can_be_attacked(|t, bears| {
        goading_aura(t, P0, "Acquired Mutation", bears);
    });
}

#[test]
fn goaded_by_several_opponents_it_attacks_one_who_didnt_goad_it_or_else_a_player() {
    cr!("701.15b", "701.15c", "508.1d");
    ruling!(
        "Disrupt Decorum",
        "If a creature you control has been goaded by multiple opponents, it must attack one of your opponents that hasn't goaded it, as that fulfills the maximum number of goad requirements. If a creature you control has been goaded by each of your opponents, the creature must attack an opponent (rather than a planeswalker or battle), but you choose which opponent it attacks."
    );
    supported("Jace Beleren");
    let mut t = TestGame::new(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P3, "Jace Beleren");
    disrupt_decorum(&mut t, P0);
    disrupt_decorum(&mut t, P2);
    assert_eq!(t.g.goaders(bears), vec![P0, P2]);
    // Goaded by P0 and P2: it must attack P3 (not a goader, and not Jace).
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P3))]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Object(jace))]));
    assert_eq!(
        declare(&mut t, P1, &[]),
        vec![(bears, Entity::Player(P3))]
    );
    // Goaded by each opponent: it must attack an opponent — any of them — rather than
    // Jace.
    let mut t = TestGame::new(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P3, "Jace Beleren");
    disrupt_decorum(&mut t, P0);
    disrupt_decorum(&mut t, P2);
    disrupt_decorum(&mut t, P3);
    assert_eq!(t.g.goaders(bears), vec![P0, P2, P3]);
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Object(jace))]));
    assert!(!legal_attack(&mut t, &[]));
    for p in [P0, P2, P3] {
        assert!(legal_attack(&mut t, &[(bears, Entity::Player(p))]));
    }
    assert_eq!(
        declare(&mut t, P1, &[(bears, Entity::Player(P2))]),
        vec![(bears, Entity::Player(P2))]
    );
}

// ---------------------------------------------------------------------------
// The goading player leaves the game.
// ---------------------------------------------------------------------------

#[test]
fn creatures_stay_goaded_until_the_leaving_players_next_turn_would_have_begun() {
    cr!("701.15a", "800.4m");
    ruling!(
        "Geode Rager",
        "If you leave the game, any creatures you've goaded remain goaded until your next turn would have begun. They don't immediately stop being goaded and don't remain goaded indefinitely."
    );
    supported("Geode Rager");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Geode Rager");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Landfall: goad each creature P1 controls.
    let land = t.hand(P0, "Mountain");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.g.goaders(bears), vec![P0]);
    // P0 leaves the game during P1's upkeep: the Bears are still goaded.
    t.advance_to(P1, Step::Upkeep);
    t.g.player_loses(P0);
    t.settle();
    assert!(!t.g.player(P0).in_game());
    assert_eq!(t.g.goaders(bears), vec![P0]);
    assert_eq!(
        declare_in_next_combat(&mut t, P1, &[]),
        vec![(bears, Entity::Player(P2))]
    );
    // Still goaded during P2's turn; P0's turn would have begun before P1's next turn,
    // and then the goad ends.
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.goaders(bears).is_empty());
    assert!(declare_in_next_combat(&mut t, P1, &[]).is_empty());
}
