//! Rulings batch P037 — bodies in combat: tokens put onto the battlefield attacking (CR
//! 508.4, 508.3a), "can't attack or block alone" (CR 506.5, 508.1c), attack triggers that
//! ask for a payment on resolution (CR 603.4, 608.2), creatures that leave combat (CR
//! 506.4, 509.1h), and haste for the creatures there on resolution (CR 611.2c).

use crate::r_p037_common::*;
use crate::r_p050_common::two_headed_giant;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{abilities_from, targets_of};
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn ready(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].summoning_sick = false;
}

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(t.g.current(id))
}

// --- Entering attacking isn't attacking --------------------------------------------------

#[test]
fn tokens_put_onto_the_battlefield_attacking_werent_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Geist of Saint Traft",
        "Although the Angel is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    ruling!(
        "Kavaron Harrier",
        "Although the token you created is put onto the battlefield attacking, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won’t trigger when that creature enters attacking."
    );
    supported("Geist of Saint Traft");
    supported("Kavaron Harrier");
    supported("Hellrider");
    for card in ["Geist of Saint Traft", "Kavaron Harrier"] {
        let mut t = TestGame::new(2);
        // Hellrider: "Whenever a creature you control attacks, Hellrider deals 1 damage to
        // the player or planeswalker it's attacking."
        t.battlefield(P0, "Hellrider");
        let c = t.battlefield(P0, card);
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        t.resolve_all();
        let tokens = crate::r_s01_common::tokens(&t, P0);
        assert_eq!(tokens.len(), 1, "{card}");
        assert!(attacking(&t, tokens[0]), "{card}");
        assert!(t.obj_now(tokens[0]).tapped, "{card}");
        assert_eq!(t.life(P1), 19, "{card}: only one Hellrider trigger");
    }
}

#[test]
fn geist_of_saint_traft_s_angel_ignores_attack_restrictions() {
    cr!("508.4", "508.1d");
    ruling!(
        "Geist of Saint Traft",
        "Any effects that say that the Angel can't attack (such as that of Propaganda) affect only the declaration of attackers. They won't stop the Angel token from entering the battlefield attacking."
    );
    supported("Propaganda");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    let wastes = t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    assert!(wastes.iter().all(|w| t.obj(*w).tapped), "Geist's {{2}} was paid");
    t.resolve_all();
    let angel = crate::r_s01_common::tokens(&t, P0)[0];
    assert!(attacking(&t, angel));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}

#[test]
fn geist_of_saint_traft_exiles_each_angel_but_not_a_copy_of_one() {
    cr!("603.7c", "707.2");
    ruling!(
        "Geist of Saint Traft",
        "If you create more than one Angel token (most likely due to Doubling Season), both are exiled at end of combat. On the other hand, if something else becomes a copy of the Angel token, the copy isn't exiled."
    );
    supported("Doubling Season");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    t.resolve_all();
    let angels = with_name(&t, "Angel Token");
    assert_eq!(angels.len(), 2);
    // Clone enters as a copy of one Angel.
    t.answer_choose(P0, &[obj(angels[0])]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Angel Token");
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(angels.iter().all(|a| !t.g.is_live(*a)));
    assert!(t.on_battlefield(clone));
}

fn with_name(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

// --- Toby's Beast: "can't attack or block alone" -------------------------------------------

/// Toby enters under P0's control and its Beast token is created; returns the Beast (ready
/// to attack).
fn beast(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let before = crate::r_s01_common::tokens(t, p);
    t.enter(p, "Toby, Beastie Befriender");
    t.resolve_all();
    let b = *crate::r_s01_common::tokens(t, p)
        .iter()
        .find(|x| !before.contains(x))
        .expect("Beast token");
    ready(t, b);
    b
}

fn legal(t: &TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    attack_declaration_legal(&t.g, &attack_options(&t.g), decl)
}

#[test]
fn toby_s_beast_can_t_attack_alone_but_its_partners_choose_their_own_targets() {
    cr!("506.5", "508.1c");
    ruling!(
        "Toby, Beastie Befriender",
        "Although the Beast token can't attack alone, other attacking creatures don't have to attack the same player, planeswalker, or battle. For example, the Beast token could attack an opponent and another creature could attack a planeswalker. Similarly, other blocking creatures don't have to block the same creature that the Beast token blocks."
    );
    ruling!(
        "Toby, Beastie Befriender",
        "If you control more than one creature with \"This creature can't attack or block alone\" they can attack or block together, even if no other creatures attack or block."
    );
    supported("Toby, Beastie Befriender");
    let mut t = TestGame::new(2);
    let b = beast(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    let p1 = Entity::Player(P1);
    assert!(!legal(&t, &[(b, p1)]));
    assert!(legal(&t, &[(b, p1), (bears, obj(jace))]));
    // Two Beasts attack together.
    let mut t = TestGame::new(2);
    let b1 = beast(&mut t, P0);
    let b2 = beast(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal(&t, &[(b1, p1), (b2, p1)]));
    // Blocking: the Beast and the Bears block different attackers.
    let mut t = TestGame::new(2);
    let b = beast(&mut t, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let a1 = t.battlefield(P0, "Hill Giant");
    let a2 = t.battlefield(P0, "Hill Giant");
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b, a1), (bears, a2)]),
    );
    attack_with(&mut t, &[(a1, Entity::Player(P1)), (a2, Entity::Player(P1))]);
    t.advance_to(P0, Step::DeclareBlockers);
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.is_blocked(a1) && c.is_blocked(a2));
}

#[test]
fn toby_s_beast_must_attack_with_another_creature_if_required() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Toby, Beastie Befriender",
        "If an effect says the Beast token attacks or blocks \"if able\" and you control one or more other creatures that are able to attack or block, you must attack or block with the Beast token and at least one other creature."
    );
    supported("Grand Melee");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grand Melee");
    let b = beast(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let p1 = Entity::Player(P1);
    assert!(legal(&t, &[(b, p1), (bears, p1)]));
    assert!(!legal(&t, &[(bears, p1)]));
    assert!(!legal(&t, &[(b, p1)]));
    assert!(!legal(&t, &[]));
}

#[test]
fn toby_s_beast_keeps_attacking_after_the_others_leave() {
    cr!("506.4", "508.1c");
    ruling!(
        "Toby, Beastie Befriender",
        "Once the Beast token has attacked or blocked, removing all of your other attacking or blocking creatures won't cause it to stop attacking or blocking."
    );
    let mut t = TestGame::new(2);
    let b = beast(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(b, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    t.g.destroy(bears, None);
    t.settle();
    assert!(attacking(&t, b));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn toby_s_beast_attacks_with_a_teammate_s_creature_in_two_headed_giant() {
    cr!("810.7", "506.5");
    ruling!(
        "Toby, Beastie Befriender",
        "In a Two-Headed Giant game, the Beast token can attack or block with a creature controlled by your teammate, even if no other creatures you control are attacking or blocking."
    );
    let mut t = two_headed_giant();
    let b = beast(&mut t, P0);
    let mate = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let p2 = Entity::Player(P2);
    assert!(!legal(&t, &[(b, p2)]));
    assert!(legal(&t, &[(b, p2), (mate, p2)]));
}

// --- Speaker of the Heavens in Two-Headed Giant ---------------------------------------------

#[test]
fn speaker_of_the_heavens_uses_the_team_s_life_in_two_headed_giant() {
    cr!("810.9", "810.9a", "103.4a");
    ruling!(
        "Speaker of the Heavens",
        "In a Two-Headed Giant game, you can activate the ability only if your team's life total is at least 7 more than your team's starting life total."
    );
    supported("Speaker of the Heavens");
    for (life, ok) in [(36, false), (37, true)] {
        let mut t = two_headed_giant();
        let sp = t.battlefield(P0, "Speaker of the Heavens");
        // The teammate gains the life: it's the team's total.
        t.g.gain_life(P1, (life - 30) as u32);
        assert_eq!(t.life(P0), life);
        let r = activate_containing(&mut t, P0, sp, "Angel");
        assert_eq!(r.is_ok(), ok, "team life {life}");
    }
}

// --- Attack triggers that ask for a payment ------------------------------------------------

#[test]
fn flameblast_dragon_targets_on_trigger_and_pays_on_resolution() {
    cr!("603.3d", "608.2", "107.3f");
    ruling!(
        "Flameblast Dragon",
        "You choose the target when the ability triggers. When the ability resolves, you choose a value for X and decide whether to pay {X}{R}. If you do decide to pay {X}{R}, it's too late for any player to respond since the ability is already in the midst of resolving."
    );
    supported("Flameblast Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Flameblast Dragon");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(d, Entity::Player(P1))]);
    let trig = abilities_from(&t, d);
    assert_eq!(trig.len(), 1);
    assert_eq!(targets_of(&t, trig[0]), vec![Entity::Player(P1)]);
    assert!(
        !t.asked().iter().any(|(_, d)| matches!(d, Decision::ChooseX { .. })),
        "X isn't chosen yet"
    );
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn fathom_fleet_captain_checks_for_another_pirate_and_pays_once() {
    cr!("603.4", "118.12");
    ruling!(
        "Fathom Fleet Captain",
        "If you don't control another nontoken Pirate at the moment Fathom Fleet Captain attacks, its triggered ability won't trigger. If you don't control another nontoken Pirate as that ability resolves, you can't pay {2}."
    );
    ruling!(
        "Fathom Fleet Captain",
        "While resolving Fathom Fleet Captain's triggered ability, you can't pay {2} multiple times to create more than one Pirate token."
    );
    supported("Fathom Fleet Captain");
    // No other Pirate: no trigger.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Fathom Fleet Captain");
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    assert!(abilities_from(&t, c).is_empty());
    // The other Pirate leaves before it resolves: no token.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Fathom Fleet Captain");
    let other = t.battlefield(P0, "Fathom Fleet Captain");
    t.lands(P0, "Wastes", 4);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, c).len(), 1);
    t.g.destroy(other, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // With four mana, still only one token.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Fathom Fleet Captain");
    t.battlefield(P0, "Fathom Fleet Captain");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

// --- Leaving combat ----------------------------------------------------------------------

#[test]
fn renowned_weaver_sacrificed_while_blocking_leaves_the_attacker_blocked() {
    cr!("506.4", "509.1h", "510.1c");
    ruling!(
        "Renowned Weaver",
        "If Renowned Weaver is attacking or blocking, and you activate its ability before combat damage is assigned and dealt, it won't deal combat damage. If it was blocking a creature, that creature will remain blocked. If you don't activate the ability before the combat damage step, it must survive combat for its ability to be activated later."
    );
    supported("Renowned Weaver");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Renowned Weaver");
    t.lands(P0, "Forest", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(w, giant)]));
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.advance_to(P1, Step::DeclareBlockers);
    activate_containing(&mut t, P0, w, "Spider").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Renowned Weaver"));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(giant));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(giant).damage, 0);
}

// --- Haste for the creatures there on resolution ------------------------------------------

#[test]
fn goro_goro_and_satoru_s_haste_affects_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Goro-Goro and Satoru",
        "The set of creatures affected by the last ability is determined as that ability resolves. Creatures you begin to control later in the turn and permanents that become creatures later in the turn won’t gain haste. As long as Goro-Goro and Satoru is still on the battlefield under your control, it will gain haste."
    );
    supported("Goro-Goro and Satoru");
    let mut t = TestGame::new(2);
    let gg = t.battlefield_sick(P0, "Goro-Goro and Satoru");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    activate_containing(&mut t, P0, gg, "haste").unwrap();
    t.resolve_all();
    assert!(t.obj_now(gg).has_keyword(KeywordKind::Haste));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    let later = t.battlefield_sick(P0, "Hill Giant");
    t.g.recompute();
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Haste));
    assert!(!t.g.can_attack(later));
}

// --- Bonny Pall ---------------------------------------------------------------------------

#[test]
fn bonny_pall_s_land_isn_t_a_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Bonny Pall, Clearcutter",
        "Putting a land card onto the battlefield with Bonny Pall's last ability doesn't count as playing a land. You can put a land card onto the battlefield even if you've already played your land for the turn."
    );
    supported("Bonny Pall, Clearcutter");
    let mut t = TestGame::new(2);
    let bp = t.battlefield(P0, "Bonny Pall, Clearcutter");
    let played = t.hand(P0, "Forest");
    t.play_land(P0, played).unwrap();
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    attack_with(&mut t, &[(bp, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}
