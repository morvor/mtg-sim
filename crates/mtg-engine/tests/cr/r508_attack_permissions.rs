//! CR 302.6 and 508.1a with 609.4: "can attack as though it had haste" (Instill Energy,
//! Frenzied Saddlebrute); CR 508.1: another player choosing which creatures attack
//! (Master Warcraft); CR 508.4: each creature put onto the battlefield attacking has its
//! own attack target, chosen by its controller.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::combat::attack_options;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// What `id` may attack in the current combat, if it can attack at all.
fn targets_of(t: &TestGame, id: ObjectId) -> Option<Vec<Entity>> {
    attack_options(&t.g)
        .into_iter()
        .find(|(c, _)| *c == id)
        .map(|(_, ts)| ts)
}

#[test]
fn enchanted_creature_can_attack_as_though_it_had_haste() {
    cr!("302.6", "508.1a", "609.4");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield_sick(P0, "Prodigal Sorcerer");
    let other = t.battlefield_sick(P0, "Grizzly Bears");
    let energy = t.battlefield(P0, "Instill Energy");
    assert!(t.g.attach(energy, Entity::Object(sorcerer)));
    to_combat(&mut t, P0);
    t.g.recompute();
    assert!(t.g.can_attack(sorcerer), "it can attack though summoning sick");
    assert!(!t.g.can_attack(other), "other creatures are still summoning sick");
    // It doesn't have haste: its {T} ability still can't be activated (CR 302.6).
    assert!(!t.obj_now(sorcerer).has_keyword(KeywordKind::Haste));
    assert!(t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).is_err());
    declare(&mut t, &[(sorcerer, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(sorcerer));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn instill_energy_untaps_the_creature_only_as_its_ability_resolves() {
    cr!("602.2", "608.2");
    ruling!(
        "Instill Energy",
        "Instill Energy's untap ability will not untap the creature until it resolves"
    );
    ruling!(
        "Instill Energy",
        "If attached to an opponent's creature, you can untap their creature during your turn"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let energy = t.battlefield(P0, "Instill Energy");
    assert!(t.g.attach(energy, Entity::Object(bears)));
    t.g.tap(bears);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, energy, 0, &[]).expect("activated on its controller's turn");
    assert!(t.obj_now(bears).tapped, "still tapped while the ability is on the stack");
    t.resolve();
    assert!(!t.obj_now(bears).tapped, "the opponent's creature untaps");
    // Only once each turn.
    t.g.tap(bears);
    assert!(t.activate(P0, energy, 0, &[]).is_err());
}

#[test]
fn instill_energy_untap_ignores_effects_on_the_untap_step() {
    cr!("502.3", "602.2");
    ruling!(
        "Instill Energy",
        "Any Auras (or other effects) which are on the creature that would cause it to not be untapped"
    );
    supported("Claustrophobia");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let energy = t.battlefield(P0, "Instill Energy");
    assert!(t.g.attach(energy, Entity::Object(bears)));
    // Claustrophobia: "enchanted creature doesn't untap during its controller's untap step".
    let claus = t.battlefield(P1, "Claustrophobia");
    assert!(t.g.attach(claus, Entity::Object(bears)));
    t.g.tap(bears);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, energy, 0, &[]).expect("activated");
    t.resolve();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn saddlebrute_lets_creatures_attack_its_controllers_opponents_as_though_they_had_haste() {
    cr!("302.6", "508.1a", "508.1b", "609.4");
    ruling!(
        "Frenzied Saddlebrute",
        "Frenzied Saddlebrute doesn't cause any creatures to gain haste"
    );
    supported("Frenzied Saddlebrute");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Frenzied Saddlebrute");
    let jace0 = t.battlefield(P0, "Jace Beleren");
    let jace2 = t.battlefield(P2, "Jace Beleren");
    // P1's freshly arrived creature, on P1's turn: it may attack P0's opponents (P2) and
    // their planeswalkers, but not P0 or P0's planeswalker.
    let bears = t.battlefield_sick(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    t.g.recompute();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
    let ts = targets_of(&t, bears).expect("it can attack");
    assert!(ts.contains(&Entity::Player(P2)));
    assert!(ts.contains(&Entity::Object(jace2)));
    assert!(!ts.contains(&Entity::Player(P0)));
    assert!(!ts.contains(&Entity::Object(jace0)));
    // Its {T} abilities still can't be activated: it doesn't have haste.
    let sorcerer = t.battlefield_sick(P1, "Prodigal Sorcerer");
    t.g.recompute();
    assert!(targets_of(&t, sorcerer).is_some());
    assert!(t.activate(P1, sorcerer, 0, &[Entity::Player(P2)]).is_err());
    // Attacking P0 with it is illegal: the engine declares a legal attack instead.
    declare(&mut t, &[(bears, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.is_attacking(bears));
}

#[test]
fn saddlebrute_controllers_own_new_creatures_may_attack() {
    cr!("508.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frenzied Saddlebrute");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let theirs = t.battlefield_sick(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    t.advance_to(P1, Step::BeginningOfCombat);
    // On P1's turn, P1's creatures may attack only P0's opponents: P1 has none to attack.
    t.g.objects[theirs.0 as usize].summoning_sick = true;
    t.g.recompute();
    assert!(targets_of(&t, theirs).is_none());
}

/// Casts Master Warcraft for `p` (with four Mountains).
fn cast_master_warcraft(t: &mut TestGame, p: PlayerId) -> Result<ObjectId, ()> {
    t.lands(p, "Mountain", 4);
    let mw = t.hand(p, "Master Warcraft");
    let r = t.cast(p, mw).try_go().map_err(|_| ());
    if r.is_ok() {
        t.resolve_all();
    }
    r
}

#[test]
fn master_warcraft_caster_chooses_attackers_and_active_player_chooses_targets() {
    cr!("508.1", "508.1a", "508.1b");
    ruling!(
        "Master Warcraft",
        "the person who cast Master Warcraft first chooses the complete group of creatures"
    );
    ruling!(
        "Master Warcraft",
        "You choose attackers and make blocking assignments regardless of whether it's your turn"
    );
    supported("Master Warcraft");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    cast_master_warcraft(&mut t, P1).expect("cast before attackers are declared");
    // P1 chooses the group: only the bears (proposing they attack P1).
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    // P0 chooses what the bears attack: Jace.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Object(jace))]),
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacking(&t), vec![(bears, Some(Entity::Object(jace)))]);
    assert!(!t.g.is_attacking(giant));
    // P0 was asked only about the creatures P1 chose.
    let p0_options: Vec<Vec<ObjectId>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::DeclareAttackers { options } if p == P0 => {
                Some(options.into_iter().map(|(c, _)| c).collect())
            }
            _ => None,
        })
        .collect();
    assert_eq!(p0_options, vec![vec![bears]]);
}

#[test]
fn master_warcraft_active_player_cant_change_the_group() {
    cr!("508.1a");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    cast_master_warcraft(&mut t, P1).expect("cast");
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (bears, Entity::Object(jace)),
            (giant, Entity::Player(P1)),
        ]),
    );
    // P0 tries to attack with the bears alone: the group stays, with P1's targets.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::DeclareAttackers);
    let mut a = attacking(&t);
    a.sort();
    let mut want = vec![
        (bears, Some(Entity::Object(jace))),
        (giant, Some(Entity::Player(P1))),
    ];
    want.sort();
    assert_eq!(a, want);
}

#[test]
fn master_warcraft_choices_must_follow_the_rules_for_attacking() {
    cr!("508.1a");
    ruling!(
        "Master Warcraft",
        "Your choices must be legal within the normal rules for attacking and blocking"
    );
    let mut t = TestGame::new(2);
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    cast_master_warcraft(&mut t, P1).expect("cast");
    // A summoning-sick creature can't be chosen: the declaration is illegal and a legal
    // one (no attack) is made instead.
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(sick, Entity::Player(P1)), (giant, Entity::Player(P1))]),
    );
    // The active player's own choice is ignored.
    declare(&mut t, &[(giant, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attacking(&t).is_empty());
    // Without planeswalkers there's nothing more for the active player to choose.
    assert_eq!(
        count_asked(&t, P0, |d| matches!(d, Decision::DeclareAttackers { .. })),
        0
    );
}

#[test]
fn master_warcraft_cast_only_before_the_first_combats_attackers() {
    cr!("506.8d");
    ruling!(
        "Master Warcraft",
        "this spell can only be cast before the beginning of the Declare Attackers Step of the first combat phase"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    go_to(&mut t, Step::PostcombatMain);
    // Before a second combat phase this turn: still too late.
    assert!(cast_master_warcraft(&mut t, P1).is_err());
}

#[test]
fn master_warcraft_on_your_own_turn_changes_nothing() {
    cr!("508.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    cast_master_warcraft(&mut t, P0).expect("cast");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    assert_eq!(
        count_asked(&t, P0, |d| matches!(d, Decision::DeclareAttackers { .. })),
        1
    );
}

#[test]
fn each_card_put_onto_the_battlefield_attacking_gets_its_own_target() {
    cr!("508.4");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P0, "Craw Wurm");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    to_combat(&mut t, P0);
    go_to(&mut t, Step::DeclareAttackers);
    // "Return all creature cards from your graveyard to the battlefield tapped and
    // attacking": the controller chooses what each one attacks, as it enters.
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let mut spell = custom_card(
        "Mass Charge",
        "Instant",
        None,
        "Return all creature cards from your graveyard to the battlefield tapped and attacking.",
    );
    spell.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    let spell = t.custom(P0, spell, mtg_engine::object::Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    t.g.flush_events();
    let na = t.g.current(a);
    let nb = t.g.current(b);
    let targets = [attack_target(&t, na), attack_target(&t, nb)];
    assert!(targets.contains(&Some(Entity::Object(jace))));
    assert!(targets.contains(&Some(Entity::Player(P1))));
}
