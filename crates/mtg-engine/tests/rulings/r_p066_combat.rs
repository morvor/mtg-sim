//! Rulings batch P066 — combat: evasion gained after blocks (CR 509.1h), vigilance gained
//! or lost after attackers are declared (CR 508.1f, 702.20b), "attacks alone" (CR 506.5),
//! attack and block triggers that put -1/-1 counters, trample checked as damage is
//! assigned (CR 702.19b, 510.1), Endling, undying returning a new object (CR 400.7) and
//! Act of Heroism.

use crate::r_p066_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::Decision;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

// ---------------------------------------------------------------------------
// Evasion gained (or blockers changed) after blocks: the attacker stays blocked
// ---------------------------------------------------------------------------

/// P0's `attacker` is blocked by P1's `blocker`; `act` then changes something; the
/// attacker stays blocked by it and deals no damage to P1.
fn stays_blocked(
    t: &mut TestGame,
    attacker: ObjectId,
    blocker: ObjectId,
    act: impl FnOnce(&mut TestGame),
) {
    to_blockers(t, &[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    assert!(is_blocked(t, attacker));
    act(t);
    t.resolve_all();
    assert!(is_blocked(t, attacker), "the attacker became unblocked");
    assert_eq!(blockers_of(t, attacker), vec![t.g.current(blocker)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the blocked attacker dealt damage to the player");
}

#[test]
fn embraal_bruiser_gaining_menace_after_blocks_stays_blocked() {
    cr!("509.1h");
    ruling!(
        "Embraal Bruiser",
        "Once a creature has blocked Embraal Bruiser, gaining menace won't cause Embraal Bruiser to become unblocked."
    );
    supported("Embraal Bruiser");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Embraal Bruiser");
    let bears = t.battlefield(P1, "Grizzly Bears");
    stays_blocked(&mut t, b, bears, |t| {
        assert!(!has_kw(t, b, KeywordKind::Menace));
        t.battlefield(P0, "Bonesplitter");
        t.g.recompute();
        assert!(has_kw(t, b, KeywordKind::Menace));
    });
    // The Bruiser's damage went to the Bears.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn weldfast_monitor_gaining_menace_after_blocks_stays_blocked() {
    cr!("509.1h");
    ruling!(
        "Weldfast Monitor",
        "Once a creature has blocked Weldfast Monitor, gaining menace won't cause Weldfast Monitor to become unblocked."
    );
    supported("Weldfast Monitor");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Weldfast Monitor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    stays_blocked(&mut t, m, bears, |t| {
        mana(t, P0, ManaType::R, 1);
        t.activate(P0, m, 0, &[]).unwrap();
        t.resolve_all();
        assert!(has_kw(t, m, KeywordKind::Menace));
    });
}

#[test]
fn desperate_lunge_flying_after_blocks_stays_blocked() {
    cr!("509.1h");
    ruling!(
        "Desperate Lunge",
        "A creature gaining flying after it has been blocked by a creature without flying or reach won't remove the blocking creature from combat or cause the creature it blocked to become unblocked."
    );
    supported("Desperate Lunge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    stays_blocked(&mut t, bears, giant, |t| {
        cast_new(t, P0, "Desperate Lunge", &[o(bears)]);
        t.resolve_all();
        assert!(has_kw(t, bears, KeywordKind::Flying));
        assert!(t.g.is_blocking(giant), "the blocker was removed from combat");
    });
    // The 4/4 Bears killed the Giant.
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn dread_linnorm_stays_blocked_when_the_blockers_power_drops() {
    cr!("509.1h");
    ruling!(
        "Dread Linnorm // Scale Deflection",
        "Reducing a creature's power to 3 or less after it has blocked Dread Linnorm won't cause it to become unblocked."
    );
    supported("Dread Linnorm // Scale Deflection");
    supported("Leeching Bite");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Dread Linnorm // Scale Deflection");
    let baloth = t.battlefield(P1, "Rumbling Baloth");
    let small = t.battlefield(P1, "Hill Giant");
    // A power-3 creature can't block it.
    assert!(!t.g.can_block(small, l));
    stays_blocked(&mut t, l, baloth, |t| {
        // +1/+1 on the Linnorm, -1/-1 on the 4/4 Baloth.
        cast_new(t, P0, "Leeching Bite", &[o(l), o(baloth)]);
        t.resolve_all();
        assert_eq!(t.pt(baloth), (3, 3));
    });
}

#[test]
fn removing_a_blocker_of_a_menace_creature_keeps_the_block() {
    cr!("509.1h", "506.4");
    ruling!(
        "Goblin Glory Chaser",
        "Once an attacking creature with menace is legally blocked by two or more creatures, removing one or more of those blockers from combat won’t change or undo that block."
    );
    supported("Goblin Glory Chaser");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Glory Chaser");
    t.g.objects[g.0 as usize].renowned = true;
    t.g.recompute();
    assert!(has_kw(&t, g, KeywordKind::Menace));
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(g, Entity::Player(P1))], &[(a, g), (b, g)]);
    assert_eq!(blockers_of(&t, g), vec![a, b]);
    remove_from_combat(&mut t.g, b);
    t.settle();
    assert!(is_blocked(&t, g));
    assert_eq!(blockers_of(&t, g), vec![a]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Its 1 damage went to the remaining blocker.
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 0);
}

// ---------------------------------------------------------------------------
// Vigilance gained or lost after attackers are declared
// ---------------------------------------------------------------------------

#[test]
fn sun_crested_pterodon_vigilance_changes_after_attacking_dont_tap_or_untap() {
    cr!("508.1f", "702.20b");
    ruling!(
        "Sun-Crested Pterodon",
        "Gaining vigilance any time after the moment you choose to attack with Sun-Crested Pterodon won't cause it to become untapped, and losing vigilance after that time won't cause it to become tapped."
    );
    supported("Sun-Crested Pterodon");
    // Attacks without vigilance (tapped), then gains it.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Sun-Crested Pterodon");
    attack_with(&mut t, &[(p, Entity::Player(P1))]);
    assert!(tapped(&t, p));
    t.battlefield(P0, "Resolute Veggiesaur");
    t.g.recompute();
    assert!(has_kw(&t, p, KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(tapped(&t, p));
    assert_eq!(t.life(P1), 18);
    // Attacks with vigilance (untapped), then loses it.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Sun-Crested Pterodon");
    let dino = t.battlefield(P0, "Resolute Veggiesaur");
    attack_with(&mut t, &[(p, Entity::Player(P1))]);
    assert!(!tapped(&t, p));
    destroy(&mut t, dino);
    assert!(!has_kw(&t, p, KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(!tapped(&t, p));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tenacious_hunter_counts_any_players_creature_and_stays_untapped() {
    cr!("508.1f", "702.20b");
    ruling!(
        "Tenacious Hunter",
        "A creature with a -1/-1 counter on it controlled by any player satisfies Tenacious Hunter’s ability."
    );
    ruling!(
        "Tenacious Hunter",
        "If Tenacious Hunter loses vigilance after attacking, it will remain untapped."
    );
    supported("Tenacious Hunter");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Tenacious Hunter");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(!has_kw(&t, h, KeywordKind::Vigilance));
    put_counters(&mut t, theirs, counters::MINUS1, 1);
    assert!(has_kw(&t, h, KeywordKind::Vigilance));
    assert!(has_kw(&t, h, KeywordKind::Deathtouch));
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    assert!(!tapped(&t, h));
    destroy(&mut t, theirs);
    assert!(!has_kw(&t, h, KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(!tapped(&t, h));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn bastion_mastodon_gaining_vigilance_after_attacking_doesnt_untap_it() {
    cr!("508.1f", "702.20b");
    ruling!(
        "Bastion Mastodon",
        "Once Bastion Mastodon is tapped and attacking, gaining vigilance won't cause it to become untapped."
    );
    supported("Bastion Mastodon");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Bastion Mastodon");
    attack_with(&mut t, &[(m, Entity::Player(P1))]);
    assert!(tapped(&t, m));
    mana(&mut t, P0, ManaType::W, 1);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, m, KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(tapped(&t, m));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn armory_guard_losing_its_gate_doesnt_tap_it_or_remove_it_from_combat() {
    cr!("508.1f", "702.20b", "506.4");
    ruling!(
        "Armory Guard",
        "Vigilance only matters when Armory Guard is being declared as an attacking creature. If Armory Guard is already attacking, losing control of your only Gate won’t cause Armory Guard to become tapped or to leave combat."
    );
    supported("Armory Guard");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Armory Guard");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    assert!(!tapped(&t, g));
    crate::r_s06_common::give_control(&mut t, gate, P1);
    assert!(!has_kw(&t, g, KeywordKind::Vigilance));
    assert!(t.g.is_attacking(g));
    block_and_finish(&mut t, P1, &[]);
    assert!(!tapped(&t, g));
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// -1/-1 counters from attack and block triggers
// ---------------------------------------------------------------------------

#[test]
fn dusk_urchins_dying_as_it_blocks_leaves_the_attacker_blocked() {
    cr!("509.1h", "506.4", "704.5f");
    ruling!(
        "Dusk Urchins",
        "If Dusk Urchins has just 1 toughness when it blocks, it will get a -1/-1 counter and be put into the graveyard. The creature that it blocked remains blocked, however."
    );
    supported("Dusk Urchins");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let u = t.battlefield(P0, "Dusk Urchins");
    put_counters(&mut t, u, counters::MINUS1, 2);
    assert_eq!(t.pt(u), (2, 1));
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(giant, Entity::Player(P0))], &[(u, giant)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dusk Urchins"));
    // Its dies trigger counted the three -1/-1 counters.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(is_blocked(&t, giant));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn dusk_urchins_gets_its_counter_before_damage_wicker_warcrawler_after() {
    cr!("508.1m", "509.2a", "511.2", "603.7a");
    ruling!(
        "Dusk Urchins",
        "The -1/-1 counter is put on Dusk Urchins during the declare attackers or declare blockers step, whichever is appropriate, so it will be smaller by the time combat damage is dealt. (Note that this is different from how the _Shadowmoor_ card Wicker Warcrawler works.)"
    );
    ruling!(
        "Wicker Warcrawler",
        "The -1/-1 counter is put on Wicker Warcrawler at end of combat, after combat damage has already been dealt. (Note that this is different from how Dusk Urchins works.)"
    );
    supported("Dusk Urchins");
    supported("Wicker Warcrawler");
    // Dusk Urchins attacking: the counter is there in the declare attackers step.
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Dusk Urchins");
    attack_with(&mut t, &[(u, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.counters(u, counters::MINUS1), 1);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
    // Dusk Urchins blocking: the counter is there in the declare blockers step.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let u = t.battlefield(P0, "Dusk Urchins");
    let w = t.battlefield(P1, "Wicker Warcrawler");
    to_blockers(&mut t, &[(w, Entity::Player(P0))], &[(u, w)]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(t.counters(u, counters::MINUS1), 1);
    // The Warcrawler (attacking) has no counter yet.
    assert_eq!(t.counters(w, counters::MINUS1), 0);
    t.advance_to(P1, Step::EndOfCombat);
    // The 3/2 Urchins dealt 3 to the 6/6 Warcrawler, which dealt its full 6.
    assert_eq!(t.obj_now(w).damage, 3);
    assert!(t.in_graveyard(P0, "Dusk Urchins"));
    assert_eq!(t.counters(w, counters::MINUS1), 0);
    // At end of combat it gets its counter.
    t.resolve_all();
    assert_eq!(t.counters(w, counters::MINUS1), 1);
    // Unblocked Warcrawler: all 6 damage is dealt before the counter.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wicker Warcrawler");
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(w, counters::MINUS1), 0);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.counters(w, counters::MINUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(w, counters::MINUS1), 1);
}

// ---------------------------------------------------------------------------
// Trample is checked as combat damage is assigned
// ---------------------------------------------------------------------------

#[test]
fn rhox_pummeler_tramples_over_a_blocker_that_removes_its_shield() {
    cr!("510.1", "510.2", "702.19b", "122.1c");
    ruling!(
        "Rhox Pummeler",
        "Combat damage is assigned to creatures and players and then dealt simultaneously. Notably, this means blocking a Rhox Pummeler with a shield counter on it won't normally prevent Rhox Pummeler from dealing damage to the defending player due to its trample ability."
    );
    supported("Rhox Pummeler");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rhox Pummeler");
    put_counters(&mut t, r, counters::SHIELD, 1);
    assert!(has_kw(&t, r, KeywordKind::Trample));
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(r, Entity::Player(P1))], &[(bears, r)]);
    t.advance_to(P0, Step::EndOfCombat);
    // 2 to the Bears, 4 trampled over; the Bears' damage removed the shield counter.
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.counters(r, counters::SHIELD), 0);
    assert_eq!(t.obj_now(r).damage, 0);
    assert!(!has_kw(&t, r, KeywordKind::Trample));
}

#[test]
fn rhox_pummeler_loses_trample_to_a_first_strike_blocker() {
    cr!("510.4", "702.7b", "702.19b", "122.1c");
    ruling!(
        "Rhox Pummeler",
        "If an attacking Rhox Pummeler with one shield counter on it is blocked by a creature with first strike, the shield counter will be removed in the first combat damage step. In that case, Rhox Pummeler won't have trample when it deals combat damage in the second combat damage step."
    );
    supported("Rhox Pummeler");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rhox Pummeler");
    put_counters(&mut t, r, counters::SHIELD, 1);
    let knight = t.battlefield(P1, "White Knight");
    to_blockers(&mut t, &[(r, Entity::Player(P1))], &[(knight, r)]);
    t.advance_to(P0, Step::CombatDamage);
    // First combat damage step: the shield counter is removed instead of the damage.
    assert_eq!(t.counters(r, counters::SHIELD), 0);
    assert!(!has_kw(&t, r, KeywordKind::Trample));
    t.advance_to(P0, Step::EndOfCombat);
    // No trample: all 6 damage to the White Knight.
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "White Knight"));
}

#[test]
fn kessig_dire_swine_gaining_trample_after_damage_is_assigned_changes_nothing() {
    cr!("702.19b", "510.1", "510.2");
    ruling!(
        "Kessig Dire Swine",
        "Trample matters only as combat damage is being assigned, before it's dealt. If Kessig Dire Swine doesn't have trample while assigning its combat damage, it won't matter if a creature dying due to combat damage causes Kessig Dire Swine to gain trample."
    );
    supported("Kessig Dire Swine");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Kessig Dire Swine");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Three card types: instant, sorcery, land.
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Forest");
    assert!(!has_kw(&t, s, KeywordKind::Trample));
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    to_blockers(
        &mut t,
        &[(s, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[(g1, s), (g2, bears)],
    );
    t.advance_to(P0, Step::EndOfCombat);
    // The Bears died to combat damage: a creature card in the graveyard is delirium.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(has_kw(&t, s, KeywordKind::Trample));
    // But all 6 of the Swine's damage was assigned (and dealt) to the Giant.
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Attacks alone (CR 506.5)
// ---------------------------------------------------------------------------

#[test]
fn lunk_errant_attacks_alone_only_when_declared_alone() {
    cr!("506.5", "508.4");
    ruling!(
        "Lunk Errant",
        "Lunk Errant “attacks alone” if it’s the only creature declared as an attacker as the declare attackers step begins. The ability won’t trigger if Lunk Errant and some other creatures are declared as attackers, then those other creatures leave the battlefield or are otherwise removed from combat. The ability also won’t trigger if Lunk Errant is put onto the battlefield attacking, even if no other creatures are attacking at that time."
    );
    supported("Lunk Errant");
    supported("Ilharg, the Raze-Boar");
    // Declared alone: it triggers.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lunk Errant");
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(l), (5, 5));
    assert!(has_kw(&t, l, KeywordKind::Trample));
    // Declared with another creature that's then removed from combat or leaves: no.
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, "Lunk Errant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        attack_with(
            &mut t,
            &[(l, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        if leave {
            destroy(&mut t, bears);
        } else {
            remove_from_combat(&mut t.g, bears);
        }
        t.settle();
        t.resolve_all();
        assert_eq!(t.pt(l), (4, 4), "leave: {leave}");
    }
    // Put onto the battlefield attacking (by Ilharg, which was then removed from combat
    // before its ability resolved): Lunk Errant is the only attacker, but no trigger.
    let mut t = TestGame::new(2);
    let ilharg = t.battlefield(P0, "Ilharg, the Raze-Boar");
    let l = t.hand(P0, "Lunk Errant");
    attack_with(&mut t, &[(ilharg, Entity::Player(P1))]);
    remove_from_combat(&mut t.g, ilharg);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(l)]);
    t.resolve_all();
    let l = t.g.current(l);
    assert!(t.on_battlefield(l));
    assert!(t.g.is_attacking(l));
    assert!(!t.g.is_attacking(ilharg));
    assert_eq!(t.pt(l), (4, 4));
}

#[test]
fn lunk_errant_doesnt_attack_alone_with_a_teammates_attacker() {
    cr!("506.5", "805.10a");
    ruling!(
        "Lunk Errant",
        "In a Two-Headed Giant game, Lunk Errant must be the only creature declared as an attacker by the entire attacking team for it to be considered “attacking alone.”"
    );
    supported("Lunk Errant");
    let config = || GameConfig {
        variant: Variant::TwoHeadedGiant,
        teams: Some(vec![0, 0, 1, 1]),
        ..Default::default()
    };
    let mut t = TestGame::with_config(4, config());
    let l = t.battlefield(P0, "Lunk Errant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(l, Entity::Player(P2)), (theirs, Entity::Player(P2))],
    );
    assert!(t.g.is_attacking(theirs));
    t.resolve_all();
    assert_eq!(t.pt(l), (4, 4));
    // Alone for the whole team: it triggers.
    let mut t = TestGame::with_config(4, config());
    let l = t.battlefield(P0, "Lunk Errant");
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(l, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(t.pt(l), (5, 5));
}

#[test]
fn intrepid_trufflesnout_attacks_alone_only_when_declared_alone() {
    cr!("506.5");
    ruling!(
        "Intrepid Trufflesnout // Go Hog Wild",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, Intrepid Trufflesnout's ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    supported("Intrepid Trufflesnout // Go Hog Wild");
    let foods = |t: &TestGame| with_subtype(t, P0, "Food").len();
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Intrepid Trufflesnout // Go Hog Wild");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(s, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    remove_from_combat(&mut t.g, bears);
    t.settle();
    t.resolve_all();
    assert_eq!(foods(&t), 0);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Intrepid Trufflesnout // Go Hog Wild");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(foods(&t), 1);
}

// ---------------------------------------------------------------------------
// Attack triggers counted on resolution; spells cast during an attack trigger
// ---------------------------------------------------------------------------

#[test]
fn charging_hooligan_counts_attackers_as_its_ability_resolves() {
    cr!("608.2h", "508.1m");
    ruling!(
        "Charging Hooligan",
        "Count the number of attacking creatures when Charging Hooligan's ability resolves, including Charging Hooligan itself, to determine the increase to its power."
    );
    supported("Charging Hooligan");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Charging Hooligan");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (h, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
    );
    // Four attackers declared; one leaves before the ability resolves.
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(t.pt(h), (6, 3));
}

#[test]
fn a_creature_cast_during_etalis_trigger_cant_join_the_attack() {
    cr!("508.1a");
    ruling!(
        "Etali, Primal Storm",
        "Because all attacking creatures are chosen at once, a creature cast this way can't attack during the same combat as Etali, even if it has haste."
    );
    supported("Etali, Primal Storm");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Etali, Primal Storm");
    let goblin = t.library_top(P0, "Raging Goblin");
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    t.answer_choose_moved(P0, &[o(goblin)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let goblin = t.g.current(goblin);
    assert!(t.on_battlefield(goblin), "the Goblin wasn't cast");
    assert!(has_kw(&t, goblin, KeywordKind::Haste));
    assert!(!t.g.is_attacking(goblin));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// Endling
// ---------------------------------------------------------------------------

/// Activates Endling's last ability `n` times, choosing -1/+1 each time.
fn endling_minus(t: &mut TestGame, e: ObjectId, n: usize) {
    for _ in 0..n {
        mana(t, P0, ManaType::C, 1);
        t.activate(P0, e, 3, &[]).unwrap();
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(1),
        );
        t.resolve_all();
    }
}

fn options_asked(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn endlings_plus_or_minus_choice_is_made_on_resolution() {
    cr!("608.2d");
    ruling!(
        "Endling",
        "If you activate Endling's last ability, you don't choose whether Endling gets +1/-1 or -1/+1 until that ability resolves."
    );
    supported("Endling");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Endling");
    mana(&mut t, P0, ManaType::C, 1);
    let from = t.asked().len();
    t.activate(P0, e, 3, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert!(options_asked(&t, from).is_empty(), "chosen on activation");
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.resolve_all();
    assert_eq!(options_asked(&t, from).len(), 1);
    assert_eq!(t.pt(e), (4, 2));
    endling_minus(&mut t, e, 2);
    assert_eq!(t.pt(e), (2, 4));
}

#[test]
fn endling_with_zero_power_deals_no_damage_so_deathtouch_doesnt_apply() {
    cr!("510.1a", "702.2b");
    ruling!(
        "Endling",
        "If Endling's power is reduced to 0 or less while in combat and it gains deathtouch, it won't deal damage, so deathtouch won't apply."
    );
    supported("Endling");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Endling");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(e, Entity::Player(P1))], &[(giant, e)]);
    endling_minus(&mut t, e, 3);
    mana(&mut t, P0, ManaType::B, 1);
    t.activate(P0, e, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(e), (0, 6));
    assert!(has_kw(&t, e, KeywordKind::Deathtouch));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.obj_now(e).damage, 3);
}

/// `c` (with an Aura, an Equipment and a -1/-1 counter) dies and undying returns it: a
/// new object with only undying's +1/+1 counter.
fn undying_returns_a_new_object(t: &mut TestGame, c: ObjectId) {
    let aura = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(aura, o(c)));
    let eq = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(eq, o(c)));
    put_counters(t, c, counters::MINUS1, 1);
    put_counters(t, c, counters::CHARGE, 2);
    destroy(t, c);
    t.resolve_all();
    let now = t.g.current(c);
    assert_ne!(now, c);
    assert!(t.on_battlefield(now), "undying didn't return it");
    assert_eq!(t.counters(now, counters::PLUS1), 1);
    assert_eq!(t.counters(now, counters::MINUS1), 0);
    assert_eq!(t.counters(now, counters::CHARGE), 0);
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(t.on_battlefield(eq));
    assert_eq!(t.obj_now(eq).attached_to, None);
    assert!(t.g.attachments_of(o(now)).is_empty());
}

#[test]
fn undying_returns_a_new_object_without_auras_equipment_or_counters() {
    cr!("702.93a", "400.7", "704.5m");
    ruling!(
        "Gleeful Arsonist",
        "Once undying returns the creature, it's considered a new object with no relation to the object that it was. Auras that were attached to it won't return to the battlefield. Equipment that was attached to it will remain unattached. Any counters that were on it won't be put on the new creature."
    );
    ruling!(
        "Endling",
        "Once undying returns the creature, it's considered a new object with no relation to the object that it was. Auras that were attached to it won't return to the battlefield. Equipment that was attached to it will remain unattached. Any counters that were on it won't be put on the new creature."
    );
    supported("Gleeful Arsonist");
    supported("Endling");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Gleeful Arsonist");
    undying_returns_a_new_object(&mut t, a);
    // Endling, which gained undying until end of turn: the returned Endling is a new
    // object that doesn't have it.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Endling");
    mana(&mut t, P0, ManaType::B, 1);
    t.activate(P0, e, 2, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, e, KeywordKind::Undying));
    let before = e;
    undying_returns_a_new_object(&mut t, e);
    assert!(!has_kw(&t, before, KeywordKind::Undying));
}

// ---------------------------------------------------------------------------
// Act of Heroism
// ---------------------------------------------------------------------------

#[test]
fn act_of_heroism_on_an_untapped_creature_and_cumulative_extra_blocks() {
    cr!("509.1a", "509.1b");
    ruling!(
        "Act of Heroism",
        "Act of Heroism can target an untapped creature. It still gets +2/+2 and can block an additional creature."
    );
    ruling!(
        "Act of Heroism",
        "The effects of Act of Heroism are cumulative. If multiples resolve targeting the same creature, that creature can block that many additional creatures this turn."
    );
    supported("Act of Heroism");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let w = t.battlefield(P0, "Hill Giant");
    assert!(!tapped(&t, w));
    assert_eq!(t.g.max_blocks(w), Some(1));
    cast_new(&mut t, P0, "Act of Heroism", &[o(w)]);
    t.resolve_all();
    assert_eq!(t.pt(w), (5, 5));
    assert_eq!(t.g.max_blocks(w), Some(2));
    cast_new(&mut t, P0, "Act of Heroism", &[o(w)]);
    t.resolve_all();
    assert_eq!(t.pt(w), (7, 7));
    assert_eq!(t.g.max_blocks(w), Some(3));
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[
            (a, Entity::Player(P0)),
            (b, Entity::Player(P0)),
            (c, Entity::Player(P0)),
        ],
        &[(w, a), (w, b), (w, c)],
    );
    for x in [a, b, c] {
        assert_eq!(blockers_of(&t, x), vec![w]);
    }
}

#[test]
fn act_of_heroism_can_be_cast_by_the_attacking_player() {
    cr!("509.1a");
    ruling!(
        "Act of Heroism",
        "You can cast Act of Heroism even if the target creature won't be able to block right away, perhaps because you're the attacking player."
    );
    supported("Act of Heroism");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    assert!(tapped(&t, w));
    cast_new(&mut t, P0, "Act of Heroism", &[o(w)]);
    t.resolve_all();
    assert!(!tapped(&t, w));
    assert_eq!(t.pt(w), (5, 5));
    // It's still attacking, and can block two creatures this turn.
    assert!(t.g.is_attacking(w));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.g.max_blocks(w), Some(2));
}
