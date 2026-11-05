//! Rulings batch P001 — combat: attack requirements combined with goad and with costs to
//! attack (CR 508.1c, 508.1d), "can't be blocked" granted after blockers are declared
//! (CR 509.1h), a creature cast with cascade entering before the spell resolves
//! (CR 702.85a), and a land that enters tapped unless (CR 614.1d, 614.12).

use crate::r_p001_common::*;
use crate::r_p006_common::old;
use crate::r_s01_common::{attack_with, stack_library, supported, tokens};
use crate::r_s25_common::lands_for_cost;
use mtg_engine::combat;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether declaring `decl` (and no other attackers) would be legal now, not counting
/// whether its costs can be paid.
fn legal(t: &mut TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    t.g.recompute();
    let options = combat::attack_options(&t.g);
    combat::attack_declaration_legal(&t.g, &options, decl)
}

/// The player `creature` is required to attack ("attacks that player this combat if
/// able").
fn required_player(t: &TestGame, creature: ObjectId) -> Option<PlayerId> {
    combat::attack_requirements(&t.g)
        .into_iter()
        .find_map(|r| match r {
            combat::AttackRequirement::AttacksPlayer(c, p) if c == creature => Some(p),
            _ => None,
        })
}

#[test]
fn knight_rampager_goaded_by_the_chosen_player_may_attack_anyone() {
    cr!("508.1d", "701.15b");
    ruling!(
        "Knight Rampager",
        "If Knight Rampager has multiple attack requirements applied to it, usually because it was also goaded, you must obey as many of the requirements as possible without violating any restrictions"
    );
    supported("Knight Rampager");
    let mut t = TestGame::new(3);
    let knight = t.battlefield(P0, "Knight Rampager");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let chosen = required_player(&t, knight).expect("an opponent was chosen");
    let other = if chosen == P1 { P2 } else { P1 };
    // Before it's goaded: it must attack the chosen player.
    assert!(legal(&mut t, &[(knight, pl(chosen))]));
    assert!(!legal(&mut t, &[(knight, pl(other))]));
    assert!(!legal(&mut t, &[]));
    // Goaded by that same player: it must attack, but any player will do.
    mtg_engine::kwa::goad::goad(&mut t.g, knight, chosen);
    assert!(legal(&mut t, &[(knight, pl(chosen))]));
    assert!(legal(&mut t, &[(knight, pl(other))]));
    assert!(!legal(&mut t, &[]));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(knight, pl(other))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(knight), Some(pl(other)));
}

#[test]
fn a_creature_cast_with_cascade_isnt_damaged_by_let_the_galaxy_burn() {
    cr!("702.85a", "608.2h");
    ruling!(
        "Knight Rampager",
        "If the spell you cast with the cascade ability is a creature, it will have entered the battlefield this turn when Let the Galaxy Burn resolves and will not take damage from Let the Galaxy Burn's effect."
    );
    supported("Let the Galaxy Burn");
    // A misattached ruling: Scryfall files it under Knight Rampager, but it describes Let
    // the Galaxy Burn ("Cascade ... Let the Galaxy Burn deals X plus 2 damage to each
    // creature that didn't enter this turn."). It's tested on Let the Galaxy Burn, with
    // Knight Rampager as the creature cast with cascade (user policy, Nov 5).
    // Let the Galaxy Burn with X = 0 (mana value 6) cascades into Knight Rampager (5).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, bears);
    old(&mut t, giant);
    stack_library(&mut t, P0, &["Island", "Knight Rampager"]);
    lands_for_cost(&mut t, P0, "Let the Galaxy Burn");
    let card = t.hand(P0, "Let the Galaxy Burn");
    t.answer_yes(P0, true);
    t.cast(P0, card).x(0).go();
    t.resolve_all();
    let knight = t.named_on_battlefield("Knight Rampager");
    assert_eq!(knight.len(), 1);
    assert_eq!(t.obj_now(knight[0]).damage, 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn tin_street_dodger_stays_blocked() {
    cr!("509.1h");
    ruling!(
        "Tin Street Dodger",
        "Activating Tin Street Dodger's ability after it has become blocked by a creature without defender won't cause it to become unblocked."
    );
    supported("Tin Street Dodger");
    let mut t = TestGame::new(2);
    let dodger = t.battlefield(P0, "Tin Street Dodger");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(dodger, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, dodger)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(dodger));
    t.activate(P0, dodger, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(dodger));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn blackcleave_cliffs_enters_tapped_with_three_other_lands() {
    cr!("614.1d", "614.12");
    ruling!(
        "Blackcleave Cliffs",
        "If one of these lands enters the battlefield under your control and you control zero, one, or two other lands, it enters the battlefield untapped. If you control three or more other lands, it enters the battlefield tapped."
    );
    supported("Blackcleave Cliffs");
    for others in 0..=4usize {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", others);
        // Other permanents don't count, nor do the opponent's lands.
        t.battlefield(P0, "Ornithopter");
        t.lands(P1, "Swamp", 3);
        let card = t.hand(P0, "Blackcleave Cliffs");
        t.play_land(P0, card).unwrap();
        let cliffs = t.g.current(card);
        assert_eq!(t.obj_now(cliffs).tapped, others >= 3, "{others} other lands");
    }
}

/// P0 casts Furygale Flocking in a three-player game (with nine instant and sorcery
/// cards in the graveyard: {R}{R}) and goes to the beginning of combat. Returns the two
/// tokens that must attack P1 and the two that must attack P2.
fn furygale(t: &mut TestGame) -> (Vec<ObjectId>, Vec<ObjectId>) {
    supported("Furygale Flocking");
    graveyard_n(t, P0, "Shock", 9);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Furygale Flocking");
    t.cast(P0, card).go();
    t.resolve_all();
    let all = tokens(t, P0);
    assert_eq!(all.len(), 4);
    t.advance_to(P0, Step::BeginningOfCombat);
    let (a, b): (Vec<ObjectId>, Vec<ObjectId>) = all
        .into_iter()
        .partition(|c| required_player(t, *c) == Some(P1));
    assert_eq!(a.len(), 2);
    assert!(b.iter().all(|c| required_player(t, *c) == Some(P2)));
    (a, b)
}

#[test]
fn furygale_tokens_that_are_tapped_or_cant_attack_dont() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Furygale Flocking",
        "If one of the Elemental tokens is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack."
    );
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    // One P1 token is tapped, the other can't attack (Pacifism).
    t.g.tap(p1s[0]);
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, obj(p1s[1]));
    let both: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    assert!(legal(&mut t, &both));
    assert!(!legal(&mut t, &both[..1]));
    assert!(!legal(&mut t, &[(p1s[1], pl(P1)), both[0], both[1]]));
}

#[test]
fn furygale_tokens_with_a_cost_to_attack_that_player_or_to_attack_at_all() {
    cr!("508.1d", "508.1h");
    interp!("attack-requirements-once-a-cost-is-paid");
    ruling!(
        "Furygale Flocking",
        "If one of the tokens can't attack the appropriate player, its controller is free to have it attack any other player, planeswalker, or battle or to not attack with it. If there's a cost associated with attacking the appropriate player, its controller is free to have it attack any other player, planeswalker, or battle or not attack with it. However, if there's a cost associated with that token attacking at all, that token has to attack the appropriate player or not attack."
    );
    supported("Ghostly Prison");
    supported("Brainwash");
    supported("Blazing Archon");
    // P1 controls Blazing Archon ("Creatures can't attack you."): the P1 tokens can't
    // attack P1, so they may attack P2 or not attack; the P2 tokens still must attack P2.
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    t.battlefield(P1, "Blazing Archon");
    let p2_attack: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    assert!(legal(&mut t, &p2_attack));
    let mut decl = p2_attack.clone();
    decl.push((p1s[0], pl(P2)));
    assert!(legal(&mut t, &decl));
    decl.push((p1s[1], pl(P1)));
    assert!(!legal(&mut t, &decl));
    assert!(!legal(&mut t, &p2_attack[..1]));
    // P1 controls Ghostly Prison: a cost to attack P1. The P1 tokens may attack P2 or
    // not attack; the P2 tokens still must attack P2.
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    t.battlefield(P1, "Ghostly Prison");
    let p2_attack: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    assert!(legal(&mut t, &p2_attack));
    let mut decl = p2_attack.clone();
    decl.push((p1s[0], pl(P2)));
    decl.push((p1s[1], pl(P2)));
    assert!(legal(&mut t, &decl));
    let mut decl = p2_attack.clone();
    decl.push((p1s[0], pl(P1)));
    assert!(legal(&mut t, &decl));
    assert!(!legal(&mut t, &p2_attack[..1]));
    // Paying to attack P1 with one doesn't excuse a P2 token from attacking P2.
    assert!(!legal(&mut t, &[(p1s[0], pl(P1)), p2_attack[0]]));
    // Brainwash on a P1 token: a cost to attack at all. It may not attack, or attack P1
    // paying the cost, but not attack P2.
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    let brainwash = t.battlefield(P0, "Brainwash");
    t.g.attach(brainwash, obj(p1s[0]));
    let mut base: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    base.push((p1s[1], pl(P1)));
    assert!(legal(&mut t, &base));
    let mut to_p1 = base.clone();
    to_p1.push((p1s[0], pl(P1)));
    assert!(legal(&mut t, &to_p1));
    let mut to_p2 = base.clone();
    to_p2.push((p1s[0], pl(P2)));
    assert!(!legal(&mut t, &to_p2));
    // When attacking P1 costs more (Ghostly Prison's {2} on top), P0 needn't pay that
    // cost: paying only Brainwash's, the token may attack P2 (provisional).
    let prison = t.battlefield(P1, "Ghostly Prison");
    assert!(legal(&mut t, &to_p2));
    assert!(legal(&mut t, &to_p1));
    let mut not_attacking = base.clone();
    not_attacking.retain(|(c, _)| *c != p1s[1]);
    assert!(legal(&mut t, &not_attacking));
    crate::r_s02_common::destroy(&mut t, prison);
    // Declared that way, it's refused and the token attacks P1 (paying {3}) or not at all.
    t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(to_p2));
    t.advance_to(P0, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert_ne!(c.attack_target(p1s[0]), Some(pl(P2)));
    // Declared attacking P1, it attacks, and the {3} is paid.
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    let brainwash = t.battlefield(P0, "Brainwash");
    t.g.attach(brainwash, obj(p1s[0]));
    let mut decl: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    decl.push((p1s[1], pl(P1)));
    decl.push((p1s[0], pl(P1)));
    let mountains = t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(decl));
    t.advance_to(P0, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(p1s[0]), Some(pl(P1)));
    assert_eq!(c.attackers.len(), 4);
    assert!(mountains.iter().all(|m| t.obj_now(*m).tapped));
    // P1 and P2 each control Ghostly Prison: every attack has a cost, but each is a cost
    // to attack one player. A P1 token may attack P2 paying P2's cost: P0 needn't pay
    // P1's instead.
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    t.battlefield(P1, "Ghostly Prison");
    t.battlefield(P2, "Ghostly Prison");
    assert!(legal(&mut t, &[]));
    assert!(legal(&mut t, &[(p1s[0], pl(P2))]));
    assert!(legal(&mut t, &[(p1s[0], pl(P2)), (p2s[0], pl(P1))]));
    // A cost to attack P1 and the planeswalkers P1 controls (Baird), with P2 free to
    // attack: a P1 token may attack P1's planeswalker paying it, but a P2 token must still
    // attack P2.
    supported("Baird, Steward of Argive");
    let mut t = TestGame::new(3);
    let (p1s, p2s) = furygale(&mut t);
    t.battlefield(P1, "Baird, Steward of Argive");
    let jace = t.battlefield(P1, "Jace Beleren");
    let p2_attack: Vec<(ObjectId, Entity)> = p2s.iter().map(|c| (*c, pl(P2))).collect();
    let mut decl = p2_attack.clone();
    decl.push((p1s[0], obj(jace)));
    assert!(legal(&mut t, &decl));
    decl.push((p1s[1], pl(P1)));
    assert!(legal(&mut t, &decl));
    decl.retain(|d| *d != p2_attack[1]);
    assert!(!legal(&mut t, &decl));
    // In a two-player game Baird's cost is one to attack at all: paying it, a token
    // attacks P1, not P1's planeswalker (provisional).
    let mut t = TestGame::new(2);
    graveyard_n(&mut t, P0, "Shock", 9);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Furygale Flocking");
    t.cast(P0, card).go();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.battlefield(P1, "Baird, Steward of Argive");
    let jace = t.battlefield(P1, "Jace Beleren");
    assert!(legal(&mut t, &[]));
    assert!(legal(&mut t, &[(toks[0], pl(P1))]));
    assert!(!legal(&mut t, &[(toks[0], obj(jace))]));
}

/// P2 casts Disrupt Decorum ("Goad all creatures you don't control.") in a three-player
/// game; P0's Grizzly Bears, goaded by P2, are at P0's beginning of combat.
fn bears_goaded_by_p2(t: &mut TestGame) -> ObjectId {
    supported("Disrupt Decorum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P2, Step::PrecombatMain);
    t.lands(P2, "Mountain", 4);
    let dd = t.hand(P2, "Disrupt Decorum");
    t.cast(P2, dd).go();
    t.resolve_all();
    assert_eq!(t.g.goaders(bears), vec![P2]);
    crate::r_s09_common::to_combat(t, P0);
    bears
}

#[test]
fn a_goaded_creature_paying_one_players_cost_isnt_forced_to_pay_anothers() {
    cr!("701.15b", "508.1d");
    interp!("attack-requirements-once-a-cost-is-paid");
    ruling!(
        "Disrupt Decorum",
        "If there's a cost associated with having a creature attack a player, its controller isn't forced to pay that cost, so it doesn't have to attack that player."
    );
    supported("Propaganda");
    supported("Brainwash");
    // Attacking P1 costs {2} (P1's Propaganda): the Bears attack P2 for free, or P1
    // paying; they can't stay home.
    let mut t = TestGame::new(3);
    let bears = bears_goaded_by_p2(&mut t);
    let p1_prop = t.battlefield(P1, "Propaganda");
    assert!(legal(&mut t, &[(bears, pl(P2))]));
    assert!(legal(&mut t, &[(bears, pl(P1))]));
    assert!(!legal(&mut t, &[]));
    // P2 has Propaganda too: no attack is free, so they needn't attack, and paying P2's
    // cost to attack P2 doesn't oblige P0 to pay P1's and attack P1 instead.
    let p2_prop = t.battlefield(P2, "Propaganda");
    assert!(legal(&mut t, &[]));
    assert!(legal(&mut t, &[(bears, pl(P2))]));
    assert!(legal(&mut t, &[(bears, pl(P1))]));
    // A cost to attack at all (Brainwash) is the same whatever they attack: they needn't
    // attack, but attacking, paying it, they attack a player other than P2.
    crate::r_s02_common::destroy(&mut t, p1_prop);
    crate::r_s02_common::destroy(&mut t, p2_prop);
    let brainwash = t.battlefield(P1, "Brainwash");
    t.g.attach(brainwash, obj(bears));
    assert!(legal(&mut t, &[]));
    assert!(legal(&mut t, &[(bears, pl(P1))]));
    assert!(!legal(&mut t, &[(bears, pl(P2))]));
}
