//! Rulings batch P062 — "Creatures target player controls don't untap during that player's
//! next untap step." and similar: a rule-modifying effect on the permanents a player
//! controls, not locked to the permanents there as it resolves (CR 611.2c): whichever of
//! them that player controls during that untap step don't untap (CR 502.3), including ones
//! that entered or became tapped later. Only that player's next untap step is affected:
//! they can untap during other players' untap steps (Seedborn Muse). Also "Creatures target
//! player controls attack this turn if able" (CR 508.1d), and "Tap all creatures target
//! player controls. Those creatures don't untap during that player's next untap step."
//! (the creatures there as it resolves).

use crate::r_p062_common::*;
use crate::r_s01_common::{asked_since, supported};
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s09_common::legal_attack;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn misstep_affects_creatures_that_entered_after_it_resolved() {
    cr!("502.3", "611.2c");
    ruling!(
        "Misstep",
        "No creature controlled by the player will untap during their next untap step. This includes creatures which entered the battlefield after Misstep resolved."
    );
    supported("Misstep");
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Hill Giant");
    let mine = tapped_creature(&mut t, P0, "Grizzly Bears");
    let land = t.lands(P1, "Forest", 1)[0];
    cast_and_resolve(&mut t, P0, "Misstep", &[Entity::Player(P1)]);
    // Later: a creature enters under P1's control tapped, and the Giant becomes tapped.
    let later = tapped_creature(&mut t, P1, "Craw Wurm");
    tap(&mut t, untapped);
    tap(&mut t, land);
    through_untap_step(&mut t, P1);
    for c in [bears, untapped, later] {
        assert!(is_tapped(&t, c));
    }
    // Only creatures, and only P1's: P0's creature untaps during P0's untap step.
    assert!(!is_tapped(&t, land));
    through_untap_step(&mut t, P0);
    assert!(!is_tapped(&t, mine));
    through_untap_step(&mut t, P1);
    for c in [bears, untapped, later] {
        assert!(!is_tapped(&t, c));
    }
}

/// A three-player game: P0 casts `spell` targeting P2, who controls Seedborn Muse
/// ("Untap all permanents you control during each other player's untap step.") and a
/// tapped `name` (a real card). During P1's untap step it untaps; tapped again, it doesn't
/// untap during P2's untap step.
fn untaps_during_others_untap_steps(spell: &str, name: &str) {
    supported(spell);
    supported("Seedborn Muse");
    let mut t = TestGame::new(3);
    t.battlefield(P2, "Seedborn Muse");
    let perm = t.battlefield(P2, name);
    tap(&mut t, perm);
    cast_and_resolve(&mut t, P0, spell, &[Entity::Player(P2)]);
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_tapped(&t, perm), "it untapped during P1's untap step");
    tap(&mut t, perm);
    t.advance_to(P2, Step::Upkeep);
    assert!(is_tapped(&t, perm), "it untapped during P2's next untap step");
    t.advance_to(P0, Step::Upkeep);
    assert!(!is_tapped(&t, perm));
}

#[test]
fn misstep_affects_only_the_targeted_players_next_untap_step() {
    cr!("502.3");
    ruling!(
        "Misstep",
        "The creatures are only prevented from untapping during the targeted player’s next untap step. They can still can untap during other player’s untap steps."
    );
    untaps_during_others_untap_steps("Misstep", "Grizzly Bears");
}

#[test]
fn exhaustion_affects_creatures_and_lands_that_entered_later() {
    cr!("502.3", "611.2c");
    ruling!(
        "Exhaustion",
        "No creatures or lands controlled by the player will untap during their next untap step. This includes those which entered the battlefield after Exhaustion resolved."
    );
    supported("Exhaustion");
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    let thopter = tapped_creature(&mut t, P1, "Ornithopter");
    cast_and_resolve(&mut t, P0, "Exhaustion", &[Entity::Player(P1)]);
    let land = t.lands(P1, "Forest", 1)[0];
    tap(&mut t, land);
    let later = tapped_creature(&mut t, P1, "Hill Giant");
    // An artifact that isn't a creature isn't affected.
    let rock = t.battlefield(P1, "Mind Stone");
    tap(&mut t, rock);
    through_untap_step(&mut t, P1);
    for c in [bears, thopter, land, later] {
        assert!(is_tapped(&t, c));
    }
    assert!(!is_tapped(&t, rock));
    through_untap_step(&mut t, P1);
    for c in [bears, thopter, land, later] {
        assert!(!is_tapped(&t, c));
    }
}

#[test]
fn exhaustion_affects_only_the_targeted_players_next_untap_step() {
    cr!("502.3");
    ruling!(
        "Exhaustion",
        "The creatures and lands are only prevented from untapping during the targeted player’s next untap step. They can still can untap during other player’s untap steps."
    );
    untaps_during_others_untap_steps("Exhaustion", "Forest");
}

#[test]
fn mana_vapors_affects_the_lands_target_player_controls_then() {
    cr!("502.3", "611.2c");
    supported("Mana Vapors");
    // "Lands target player controls don't untap during their next untap step."
    let mut t = TestGame::new(2);
    let forest = t.lands(P1, "Forest", 1)[0];
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    let mine = t.lands(P0, "Island", 1)[0];
    cast_and_resolve(&mut t, P0, "Mana Vapors", &[Entity::Player(P1)]);
    tap(&mut t, forest);
    let later = t.lands(P1, "Mountain", 1)[0];
    tap(&mut t, later);
    tap(&mut t, mine);
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, forest) && is_tapped(&t, later));
    assert!(!is_tapped(&t, bears));
    through_untap_step(&mut t, P0);
    assert!(!is_tapped(&t, mine));
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, forest) && !is_tapped(&t, later));
}

/// Icebreaker Kraken enters under P0's control, its ability targeting P1.
fn kraken_enters(t: &mut TestGame) -> ObjectId {
    supported("Icebreaker Kraken");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let k = crate::r_s05_common::enter(t, P0, "Icebreaker Kraken");
    t.resolve_all();
    k
}

#[test]
fn icebreaker_krakens_ability_doesnt_tap_anything() {
    cr!("502.3");
    ruling!(
        "Icebreaker Kraken",
        "Icebreaker Kraken's enters-the-battlefield ability won't tap any artifacts or creatures."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    kraken_enters(&mut t);
    assert!(!is_tapped(&t, bears));
    assert!(!is_tapped(&t, thopter));
}

#[test]
fn icebreaker_kraken_affects_artifacts_and_creatures_untapped_or_gained_later() {
    cr!("502.3", "611.2c");
    ruling!(
        "Icebreaker Kraken",
        "No artifacts or creatures controlled by the player affected by Icebreaker Kraken's enters-the-battlefield ability will untap during that player's next untap step, even if they were untapped as the ability resolved or that player didn't control them at that time."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let rock = t.battlefield(P1, "Mind Stone");
    let gift = t.battlefield(P0, "Hill Giant");
    let land = t.lands(P1, "Forest", 1)[0];
    kraken_enters(&mut t);
    // Afterwards: P1's artifacts become tapped; P1 gains control of P0's Giant, tapped.
    tap(&mut t, thopter);
    tap(&mut t, rock);
    give_control(&mut t, gift, P1);
    tap(&mut t, gift);
    tap(&mut t, land);
    through_untap_step(&mut t, P1);
    for c in [thopter, rock, gift] {
        assert!(is_tapped(&t, c));
    }
    assert!(!is_tapped(&t, land), "lands aren't affected");
    through_untap_step(&mut t, P1);
    for c in [thopter, rock, gift] {
        assert!(!is_tapped(&t, c));
    }
}

#[test]
fn icebreaker_krakens_return_cost_is_paid_before_anyone_can_respond() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Icebreaker Kraken",
        "Once you announce that you're activating the last ability, no player may take actions until the ability has been paid for. Notably, opponents can't try to stop you from activating it by removing your snow lands."
    );
    supported("Icebreaker Kraken");
    let mut t = TestGame::new(2);
    let kraken = t.battlefield(P0, "Icebreaker Kraken");
    let lands = t.lands(P0, "Snow-Covered Island", 3);
    let from = t.asked().len();
    let e: Vec<Entity> = lands.iter().map(|l| obj(*l)).collect();
    t.answer_choose(P0, &e);
    activate_containing(&mut t, P0, kraken, "Return three snow lands").expect("activate");
    // The lands were returned as the cost was paid; P1 was asked nothing meanwhile.
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.stack_len(), 1);
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Icebreaker Kraken"));
}

/// P0 casts Imaginary Threats targeting P1 in P1's beginning of combat step.
fn imaginary_threats(t: &mut TestGame) {
    supported("Imaginary Threats");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s25_common::cast_new(t, P0, "Imaginary Threats", &[Entity::Player(P1)]);
    t.resolve_all();
}

#[test]
fn imaginary_threats_affects_creatures_that_dont_attack_or_enter_later() {
    cr!("502.3", "508.1d", "611.2c");
    ruling!(
        "Imaginary Threats",
        "No creatures that player controls will untap during their next untap step, even creatures that don’t attack. This includes creatures that enter the battlefield or become tapped after this spell resolves."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Vertigo Spawn");
    let land = t.lands(P1, "Forest", 1)[0];
    imaginary_threats(&mut t);
    // The Giant must attack.
    assert!(!legal_attack(&mut t, &[]));
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert!(is_tapped(&t, giant));
    let later = tapped_creature(&mut t, P1, "Grizzly Bears");
    tap(&mut t, wall);
    tap(&mut t, land);
    through_untap_step(&mut t, P1);
    for c in [giant, wall, later] {
        assert!(is_tapped(&t, c));
    }
    assert!(!is_tapped(&t, land));
    through_untap_step(&mut t, P1);
    for c in [giant, wall, later] {
        assert!(!is_tapped(&t, c));
    }
}

#[test]
fn imaginary_threats_doesnt_force_a_creature_that_cant_attack_or_must_pay() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Imaginary Threats",
        "If, during that player’s declare attackers step, a creature that player controls is tapped or is affected by a spell or ability that says it can’t attack, then it doesn’t attack. If there’s a cost associated with having a creature attack, its controller isn’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    // A tapped creature and a creature with defender: no attack is required.
    let mut t = TestGame::new(2);
    tapped_creature(&mut t, P1, "Grizzly Bears");
    t.battlefield(P1, "Vertigo Spawn");
    imaginary_threats(&mut t);
    assert!(legal_attack(&mut t, &[]));
    // An untapped Hill Giant must attack; not if attacking P0 has a cost (Archangel of
    // Tithes: "creatures can't attack you ... unless their controller pays {1} for each").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    imaginary_threats(&mut t);
    assert!(!legal_attack(&mut t, &[]));
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archangel of Tithes");
    t.battlefield(P1, "Hill Giant");
    imaginary_threats(&mut t);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn imaginary_threats_and_exert_apply_in_the_same_untap_step() {
    cr!("701.43a", "502.3");
    ruling!(
        "Imaginary Threats",
        "If the opponent exerts any creatures they control, exert and the effect from Imaginary Threats stopping them from untapping both apply in the same untap step. Those creatures will untap as normal in the player’s subsequent untap step."
    );
    supported("Ahn-Crop Crasher");
    // Ahn-Crop Crasher: "You may exert this creature as it attacks."
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P1, "Ahn-Crop Crasher");
    imaginary_threats(&mut t);
    t.answer_yes(P1, true);
    t.attack(&[(crasher, Entity::Player(P0))], &[]);
    assert!(t.obj_now(crasher).exerted);
    misses_one_untap(&mut t, crasher, P1);
}

#[test]
fn instigator_makes_creatures_target_player_controls_attack() {
    cr!("508.1d", "611.2c");
    supported("Instigator");
    // "{1}{B}{B}, {T}, Discard a card: Creatures target player controls attack this turn
    // if able." Activated in P1's upkeep; a creature P1 gets afterwards must attack too.
    let mut t = TestGame::new(2);
    let inst = t.battlefield(P0, "Instigator");
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, inst, "attack this turn").expect("activate");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    let later = t.battlefield(P1, "Grizzly Bears");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
    assert!(legal_attack(
        &mut t,
        &[(giant, Entity::Player(P0)), (later, Entity::Player(P0))]
    ));
    // P0's creatures aren't affected (next turn, P0 needn't attack).
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    t.battlefield(P0, "Craw Wurm");
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn incite_war_modes() {
    cr!("700.2", "508.1d", "702.42a");
    supported("Incite War");
    // "Choose one — • Creatures target player controls attack this turn if able.
    // • Creatures you control gain first strike until end of turn. Entwine {2}"
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Incite War");
    t.cast(P0, card)
        .kicked(false)
        .modes(&[0])
        .target(P1)
        .go();
    t.resolve_all();
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
    // The second mode.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Incite War");
    t.cast(P0, card).kicked(false).modes(&[1]).go();
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(
        &t,
        bears,
        mtg_engine::keywords::KeywordKind::FirstStrike
    ));
    // Entwined: both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Incite War");
    t.cast(P0, card).kicked(true).target(P1).go();
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(
        &t,
        bears,
        mtg_engine::keywords::KeywordKind::FirstStrike
    ));
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
}

#[test]
fn sleep_affects_all_creatures_the_player_controls_as_it_resolves() {
    cr!("502.3", "608.2c");
    ruling!(
        "Sleep",
        "The second part of Sleep’s ability affects all creatures the targeted player controls as Sleep resolves, not only the ones that Sleep actually caused to become tapped."
    );
    supported("Sleep");
    let mut t = TestGame::new(2);
    let tapped = tapped_creature(&mut t, P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Craw Wurm");
    cast_and_resolve(&mut t, P0, "Sleep", &[Entity::Player(P1)]);
    assert!(is_tapped(&t, untapped));
    assert!(!is_tapped(&t, mine));
    // A creature P1 gets later isn't affected.
    let later = tapped_creature(&mut t, P1, "Ornithopter");
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, tapped));
    assert!(is_tapped(&t, untapped));
    assert!(!is_tapped(&t, later));
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, tapped));
    assert!(!is_tapped(&t, untapped));
}
