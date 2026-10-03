//! Rulings batch P062 — free sacrifice outlets ("Sacrifice a creature: ..."). A creature
//! can be sacrificed to pay the cost of its own ability (CR 602.2b, 118.3); the ability
//! then resolves with whatever it can still do, and the sacrifice is seen by "dies"
//! abilities (CR 700.4). The sacrificed object's characteristics are its last known
//! information (CR 608.2h). Sacrificing an attacking creature removes it from combat
//! (CR 506.4).

use crate::r_p062_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s03_common::respond;
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, attach_new, attached_to, has_kw};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` activates the ability of `source` whose text contains `needle`, sacrificing `fodder`
/// to pay its cost; `targets` are its targets (one per slot).
fn sac_to(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    fodder: ObjectId,
    targets: &[Entity],
) {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.answer_choose(p, &[obj(fodder)]);
    activate_containing(t, p, source, needle)
        .unwrap_or_else(|e| panic!("activation failed: {e:?}"));
    t.settle();
    t.clear_answers();
}

/// The regeneration shield works: destroyed, the creature stays, tapped.
fn survives_destruction(t: &mut TestGame, id: ObjectId) {
    destroy(t, id);
    assert!(t.on_battlefield(id), "it wasn't regenerated");
    assert!(is_tapped(t, id));
}

#[test]
fn falkenrath_aristocrat_sacrificing_a_human_gains_indestructible_and_a_counter() {
    cr!("602.2b", "608.2h");
    ruling!(
        "Falkenrath Aristocrat",
        "If the sacrificed creature was a Human, Falkenrath Aristocrat gains indestructible and also gets a +1/+1 counter."
    );
    supported("Falkenrath Aristocrat");
    let mut t = TestGame::new(2);
    let fa = t.battlefield(P0, "Falkenrath Aristocrat");
    let human = t.battlefield(P0, "Elite Vanguard");
    sac_to(&mut t, P0, fa, "gains indestructible", human, &[]);
    t.resolve_all();
    assert!(has_kw(&t, fa, KeywordKind::Indestructible));
    assert_eq!(t.counters(fa, "+1/+1"), 1);
    // A non-Human: indestructible only.
    let bears = t.battlefield(P0, "Grizzly Bears");
    sac_to(&mut t, P0, fa, "gains indestructible", bears, &[]);
    t.resolve_all();
    assert_eq!(t.counters(fa, "+1/+1"), 1);
}

/// P0 controls Blood Artist ("Whenever this creature or another creature dies, target
/// player loses 1 life and you gain 1 life."), targeting P1.
fn blood_artist(t: &mut TestGame) {
    supported("Blood Artist");
    t.battlefield(P0, "Blood Artist");
    t.answer_targets(P0, &[Entity::Player(P1)]);
}

/// The Blood Artist's ability (targeting P1) saw one creature die.
fn one_death_seen(t: &mut TestGame) {
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn falkenrath_aristocrat_can_sacrifice_itself() {
    cr!("602.2b", "118.3", "700.4");
    ruling!(
        "Falkenrath Aristocrat",
        "You can sacrifice Falkenrath Aristocrat to pay the cost of its own ability. It won't receive a counter or gain indestructible, but it will enable abilities that check when or if a creature dies."
    );
    supported("Falkenrath Aristocrat");
    let mut t = TestGame::new(2);
    blood_artist(&mut t);
    let fa = t.battlefield(P0, "Falkenrath Aristocrat");
    sac_to(&mut t, P0, fa, "gains indestructible", fa, &[]);
    assert!(t.in_graveyard(P0, "Falkenrath Aristocrat"));
    assert_eq!(triggers_on_stack(&t, "dies"), 1);
    one_death_seen(&mut t);
    assert!(t.in_graveyard(P0, "Falkenrath Aristocrat"));
}

#[test]
fn defiant_salvager_sacrificing_an_artifact_creature_gets_one_counter() {
    cr!("602.2b");
    ruling!(
        "Defiant Salvager",
        "If you sacrifice an artifact creature to activate Defiant Salvager's ability, you put one +1/+1 counter on Defiant Salvager, not two."
    );
    supported("Defiant Salvager");
    let mut t = TestGame::new(2);
    let salvager = t.battlefield(P0, "Defiant Salvager");
    let thopter = t.battlefield(P0, "Ornithopter");
    sac_to(&mut t, P0, salvager, "+1/+1 counter", thopter, &[]);
    t.resolve_all();
    assert_eq!(t.counters(salvager, "+1/+1"), 1);
}

#[test]
fn defiant_salvager_can_sacrifice_itself() {
    cr!("602.2b", "118.3", "700.4");
    ruling!(
        "Defiant Salvager",
        "You can sacrifice Defiant Salvager to activate its own ability. It won't receive a counter, but abilities that trigger on the sacrifice or watch for a creature dying will see it."
    );
    supported("Defiant Salvager");
    let mut t = TestGame::new(2);
    blood_artist(&mut t);
    let salvager = t.battlefield(P0, "Defiant Salvager");
    sac_to(&mut t, P0, salvager, "+1/+1 counter", salvager, &[]);
    assert_eq!(triggers_on_stack(&t, "dies"), 1);
    one_death_seen(&mut t);
    assert!(t.in_graveyard(P0, "Defiant Salvager"));
}

#[test]
fn bloodflow_connoisseur_can_sacrifice_itself_and_nothing_happens() {
    cr!("602.2b", "118.3");
    ruling!(
        "Bloodflow Connoisseur",
        "You can sacrifice Bloodflow Connoisseur to pay the cost for its own ability. In this case, nothing happens as the ability resolves."
    );
    supported("Bloodflow Connoisseur");
    let mut t = TestGame::new(2);
    let bc = t.battlefield(P0, "Bloodflow Connoisseur");
    let bears = t.battlefield(P0, "Grizzly Bears");
    sac_to(&mut t, P0, bc, "+1/+1 counter", bc, &[]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bloodflow Connoisseur"));
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn goblin_trashmaster_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Goblin Trashmaster",
        "You can sacrifice Goblin Trashmaster to pay the cost for its own ability."
    );
    supported("Goblin Trashmaster");
    let mut t = TestGame::new(2);
    let tm = t.battlefield(P0, "Goblin Trashmaster");
    let thopter = t.battlefield(P1, "Ornithopter");
    sac_to(&mut t, P0, tm, "Destroy target artifact", tm, &[obj(thopter)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Trashmaster"));
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn viscera_seer_can_sacrifice_itself() {
    cr!("602.2b", "118.3", "701.22a");
    ruling!(
        "Viscera Seer",
        "You can sacrifice Viscera Seer to activate its own ability."
    );
    supported("Viscera Seer");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Viscera Seer");
    sac_to(&mut t, P0, seer, "Scry 1", seer, &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Viscera Seer"));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { cards } if cards.len() == 1)));
}

#[test]
fn skirk_prospector_can_sacrifice_any_goblin_including_itself() {
    cr!("602.2b", "118.3", "605.1a");
    ruling!(
        "Skirk Prospector",
        "You can sacrifice any Goblin you control to activate Skirk Prospector’s activated ability, including Skirk Prospector itself."
    );
    supported("Skirk Prospector");
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Skirk Prospector");
    let other = t.battlefield(P0, "Raging Goblin");
    let last = t.battlefield(P0, "Raging Goblin");
    sac_to(&mut t, P0, sp, "Add {R}", other, &[]);
    sac_to(&mut t, P0, sp, "Add {R}", sp, &[]);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    // Once it's sacrificed, its ability can't be activated again.
    assert!(!t.on_battlefield(sp));
    assert!(t.on_battlefield(last));
    assert!(!can_activate(&mut t, P0, sp));
}

#[test]
fn thalias_geistcaller_can_sacrifice_any_spirit() {
    cr!("602.2b", "205.3m");
    ruling!(
        "Thalia's Geistcaller",
        "You can sacrifice any Spirit you control to activate the last ability of Thalia's Geistcaller, not just a Spirit token it created."
    );
    supported("Thalia's Geistcaller");
    let mut t = TestGame::new(2);
    let tg = t.battlefield(P0, "Thalia's Geistcaller");
    let spirit = t.battlefield(P0, "Spectral Sailor");
    sac_to(&mut t, P0, tg, "indestructible", spirit, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Spectral Sailor"));
    assert!(has_kw(&t, tg, KeywordKind::Indestructible));
}

#[test]
fn skeletal_vampire_can_sacrifice_any_bat() {
    cr!("602.2b", "701.19a");
    ruling!(
        "Skeletal Vampire",
        "You may pay the cost of Skeletal Vampire's activated abilities by sacrificing any Bat, not just a Bat token it created."
    );
    supported("Skeletal Vampire");
    let mut t = TestGame::new(2);
    let sv = t.battlefield(P0, "Skeletal Vampire");
    let bat = t.battlefield(P0, "Dakmor Bat");
    sac_to(&mut t, P0, sv, "Regenerate", bat, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dakmor Bat"));
    survives_destruction(&mut t, sv);
    // The token-making ability too.
    let bat = t.battlefield(P0, "Dakmor Bat");
    t.lands(P0, "Swamp", 5);
    sac_to(&mut t, P0, sv, "Create two", bat, &[]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
}

#[test]
fn gatherer_of_graces_can_sacrifice_an_aura_attached_to_something_else() {
    cr!("602.2b", "701.19a");
    ruling!(
        "Gatherer of Graces",
        "The Aura you sacrifice doesn't have to be attached to Gatherer of Graces."
    );
    supported("Gatherer of Graces");
    let mut t = TestGame::new(2);
    let gatherer = t.battlefield(P0, "Gatherer of Graces");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    sac_to(&mut t, P0, gatherer, "Regenerate", aura, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    survives_destruction(&mut t, gatherer);
}

#[test]
fn vampire_warlords_regeneration_shield_must_exist_before_it_is_destroyed() {
    cr!("701.19a", "701.19b");
    ruling!(
        "Vampire Warlord",
        "To work, the regeneration shield must be created before Vampire Warlord is destroyed. This usually means activating its ability during the declare blockers step, or in response to a spell or ability that would destroy it."
    );
    supported("Vampire Warlord");
    supported("Murder");
    // In response to Murder, P0 regenerates the Warlord: it survives.
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P0, "Vampire Warlord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast_with(P1, murder, &[obj(warlord)]).expect("cast Murder");
    sac_to(&mut t, P0, warlord, "Regenerate", bears, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(warlord));
    assert!(is_tapped(&t, warlord));
    // Without a shield, Murder destroys it, and then there's nothing to regenerate.
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P0, "Vampire Warlord");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast_with(P1, murder, &[obj(warlord)]).expect("cast Murder");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Vampire Warlord"));
}

#[test]
fn goblin_turncoat_can_regenerate_when_not_at_risk() {
    cr!("701.19a", "602.2");
    ruling!(
        "Goblin Turncoat",
        "You can activate the regeneration ability even if Goblin Turncoat isn’t at risk of being destroyed."
    );
    supported("Goblin Turncoat");
    let mut t = TestGame::new(2);
    let gt = t.battlefield(P0, "Goblin Turncoat");
    let goblin = t.battlefield(P0, "Raging Goblin");
    sac_to(&mut t, P0, gt, "Regenerate", goblin, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Raging Goblin"));
    // The shield waits for a destruction later this turn.
    survives_destruction(&mut t, gt);
}

#[test]
fn darkheart_slivers_granted_ability_gains_life_for_the_slivers_controller() {
    cr!("602.2", "109.5");
    ruling!(
        "Darkheart Sliver",
        "The player who controlled the Sliver that's sacrificed gains the life, not the controller of Darkheart Sliver."
    );
    supported("Darkheart Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darkheart Sliver");
    let theirs = t.battlefield(P1, "Muscle Sliver");
    activate_containing(&mut t, P1, theirs, "gain 3 life").expect("P1 activates it");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Muscle Sliver"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 23));
}

#[test]
fn varolz_scavenge_counts_the_cards_power_as_it_last_existed_in_the_graveyard() {
    cr!("702.97a", "608.2h", "604.3");
    ruling!(
        "Varolz, the Scar-Striped",
        "The number of counters that a card's scavenge ability puts on a creature is based on the card's power as it last existed in the graveyard."
    );
    supported("Varolz, the Scar-Striped");
    // Tarmogoyf in P0's graveyard with Giant Growth: two card types, so it's 2/3 there.
    // Exiled to pay the scavenge cost, it would be 1/2 (only an instant remains).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Varolz, the Scar-Striped");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P0, "Giant Growth");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    t.lands(P0, "Forest", 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, goyf, "Scavenge").expect("scavenge");
    assert!(t.in_exile("Tarmogoyf"));
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn altar_of_dementia_mills_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "701.17a");
    ruling!(
        "Altar of Dementia",
        "The number of cards the target player puts into their graveyard is equal to the power of the sacrificed creature as it last existed on the battlefield."
    );
    supported("Altar of Dementia");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of Dementia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p057_common::pump(&mut t, bears, 3, 3);
    assert_eq!(t.pt(bears), (5, 5));
    sac_to(&mut t, P0, altar, "mills", bears, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 5);
}

#[test]
fn altar_of_dementia_mills_a_short_library_without_losing_yet() {
    cr!("701.17b", "704.5b");
    ruling!(
        "Altar of Dementia",
        "If the target player's library contains fewer cards than the sacrificed creature's power, all of those cards are put into that player's graveyard. That player won't lose the game until they try to draw from the empty library."
    );
    supported("Altar of Dementia");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of Dementia");
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s11_common::empty_library(&mut t, P1);
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    sac_to(&mut t, P0, altar, "mills", giant, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 2);
    assert!(!t.has_lost(P1));
    crate::r_p030_common::draw(&mut t, P1, 1);
    assert!(t.has_lost(P1));
}

#[test]
fn piston_sledge_stays_unattached_with_no_creature_to_attach_to() {
    cr!("603.3d", "301.5");
    ruling!(
        "Piston Sledge",
        "If there are no creatures on the battlefield that Piston Sledge could be attached to when its \"enters\" triggered ability resolves, it remains on the battlefield unattached."
    );
    supported("Piston Sledge");
    // P0 controls no creatures (P1's creature can't be targeted: "you control").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let sledge = enter(&mut t, P0, "Piston Sledge");
    assert_eq!(triggers_on_stack(&t, "attach it"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(sledge));
    assert_eq!(attached_to(&t, sledge), None);
}

#[test]
fn viscera_seer_sacrificing_an_attacker_or_too_late() {
    cr!("506.4", "510.2", "704.3");
    ruling!(
        "Viscera Seer",
        "If you sacrifice an attacking or blocking creature during the declare blockers step, it won't deal combat damage. If you wait until the combat damage step, but that creature is dealt lethal damage, it'll be destroyed before you get a chance to sacrifice it."
    );
    supported("Viscera Seer");
    // Unblocked Bears sacrificed in the declare blockers step deal no combat damage.
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Viscera Seer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    sac_to(&mut t, P0, seer, "Scry 1", bears, &[]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Bears blocked by Hill Giant: when P0 gets priority in the combat damage step, the
    // Bears are already gone (state-based actions are checked first).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Viscera Seer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, bears)]));
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (g.turn.step, graveyard_names_of(g, P0)),
    );
    t.advance_to(P0, Step::EndOfCombat);
    let seen = seen.lock().unwrap();
    let in_damage_step: Vec<&Vec<String>> = seen
        .iter()
        .filter(|(s, _)| *s == Step::CombatDamage)
        .map(|(_, gy)| gy)
        .collect();
    assert!(!in_damage_step.is_empty());
    assert!(in_damage_step
        .iter()
        .all(|gy| gy.contains(&"Grizzly Bears".to_string())));
}

/// The names of the cards in `p`'s graveyard.
fn graveyard_names_of(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p)
        .graveyard
        .iter()
        .map(|c| g.obj(*c).chars.name.to_string())
        .collect()
}

#[test]
fn grafted_wargear_cant_sacrifice_an_opponents_creature() {
    cr!("701.21a", "301.5d");
    ruling!(
        "Grafted Wargear",
        "If your Grafted Wargear somehow ends up on an opponent's creature and it then becomes unattached from that creature, you won't be able to sacrifice that creature because you don't control it."
    );
    supported("Grafted Wargear");
    // P0's Wargear is on P1's Bears; P0 equips it to P0's Hill Giant.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let wargear = attach_new(&mut t, P0, "Grafted Wargear", theirs);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, wargear, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, wargear), Some(obj(giant)));
    assert!(t.on_battlefield(theirs), "P0 can't sacrifice P1's creature");
    // Moved off P0's own Giant, the Giant is sacrificed.
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    activate_containing(&mut t, P0, wargear, "Equip").expect("equip");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn training_still_adds_a_counter_after_the_other_attacker_is_destroyed() {
    cr!("702.149a", "603.2");
    ruling!(
        "Elder Arthur Maxson",
        "Once a creature’s training ability has triggered, destroying the other attacking creature or reducing its power won’t stop the creature with training from getting a +1/+1 counter."
    );
    supported("Elder Arthur Maxson");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elder Arthur Maxson");
    let token = create_token(&mut t, P0, "Soldier");
    crate::r_p023_common::unsick(&mut t, token);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[(token, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Training"), 1);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(token, "+1/+1"), 1);
}

/// The color choices asked since decision `from`.
fn color_choices_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { prompt, .. }
                if prompt.to_lowercase().contains("color"))
        })
        .count()
}

#[test]
fn cartel_aristocrat_chooses_the_color_on_resolution() {
    cr!("608.2d", "702.16a");
    ruling!(
        "Cartel Aristocrat",
        "You choose the color when the ability resolves."
    );
    supported("Cartel Aristocrat");
    let mut t = TestGame::new(2);
    let ca = t.battlefield(P0, "Cartel Aristocrat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    sac_to(&mut t, P0, ca, "protection", bears, &[]);
    assert_eq!(color_choices_since(&t, from), 0, "chosen as it was activated");
    t.resolve_all();
    assert_eq!(color_choices_since(&t, from), 1);
    assert!(has_kw(&t, ca, KeywordKind::Protection));
}

/// Answers a priority decision by casting a spell, if one can be cast.
fn cast_first(_g: &Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Priority { actions } => actions
            .iter()
            .find(|a| matches!(a, Action::Cast { .. }))
            .cloned()
            .map(Answer::Action),
        _ => None,
    }
}

#[test]
fn after_omniscience_resolves_on_your_turn_you_cast_a_spell_first() {
    cr!("117.3b", "118.9");
    ruling!(
        "Omniscience",
        "Once you cast Omniscience, if it's your turn, you'll have priority immediately after it resolves. You can cast another spell before any player can attempt to remove Omniscience with spells or abilities."
    );
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 10);
    let omni = t.hand(P0, "Omniscience");
    t.cast(P0, omni).go();
    // P0's only other card: Craw Wurm (with all of P0's lands tapped).
    t.hand(P0, "Craw Wurm");
    let from = t.asked().len();
    respond(&mut t, P0, cast_first);
    let ok = t.g.run_until(1000, |g| {
        g.stack
            .iter()
            .any(|id| g.obj(*id).chars.name.as_str() == "Craw Wurm")
    });
    assert!(ok, "{}", t.dump_log());
    // P0 passed with Omniscience on the stack, P1 passed, it resolved, and P0 got
    // priority first and cast Craw Wurm for free.
    assert!(t.named_on_battlefield("Omniscience").len() == 1);
    assert_eq!(
        crate::r_s03_common::priority_asked_since(&t, from),
        vec![P0, P1, P0]
    );
}
