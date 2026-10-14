//! Structure-coverage tests, round 1: static-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks a real
//! card's ability does what its text says, including when it doesn't apply.

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

#[test]
fn wild_wasteland_skips_your_draw_step() {
    cr!("614.10", "504.1");
    // "Skip your draw step."
    supported("Wild Wasteland");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Wasteland");
    // The opponent still draws.
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    // P0's library only lost the two cards its upkeep trigger exiled: no draw.
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn lance_gives_the_enchanted_creature_first_strike() {
    cr!("303.4", "702.7b");
    // "Enchant creature / Enchanted creature has first strike."
    supported("Lance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let lance = t.hand(P0, "Lance");
    t.cast(P0, lance).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(blocker).has_keyword(KeywordKind::FirstStrike));
    // It kills an equal blocker first and survives.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn nahiris_wrath_discards_x_and_deals_their_total_mana_value() {
    cr!("601.2b", "601.2f", "118.8");
    // "As an additional cost to cast this spell, discard X cards. / Nahiri's Wrath deals
    // damage equal to the total mana value of the discarded cards to each of up to X
    // target creatures and/or planeswalkers."
    supported("Nahiri's Wrath");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let other = t.battlefield(P1, "Hill Giant");
    let wrath = t.hand(P0, "Nahiri's Wrath");
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(bears)]);
    t.cast(P0, wrath)
        .x(2)
        .targets(&[Entity::Object(giant), Entity::Object(wurm)])
        .go();
    // The discards are a cost: paid on casting.
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "3 damage (1 + 2) kills the 3/3");
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.obj_now(other).damage, 0);
}

#[test]
fn orcish_oriflamme_pumps_only_your_attacking_creatures() {
    cr!("613.4c", "508.1");
    // "Attacking creatures you control get +1/+0."
    supported("Orcish Oriflamme");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orcish Oriflamme");
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(attacker), (2, 2));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.pt(attacker), (3, 2));
    assert_eq!(t.pt(idle), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // The opponent's attackers don't get it.
    t.advance_to(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(theirs, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn boots_of_speed_gives_plus_one_power_and_haste() {
    cr!("301.5", "613.4c", "702.10b");
    // "Equipped creature gets +1/+0 and has haste."
    supported("Boots of Speed");
    let mut t = TestGame::new(2);
    let boots = t.battlefield(P0, "Boots of Speed");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let other = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, boots, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(boots).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(other), (2, 2));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Haste));
    // The summoning-sick creature can attack.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn jade_leech_makes_your_green_spells_cost_g_more() {
    cr!("601.2f", "118.7");
    // "Green spells you cast cost {G} more to cast."
    supported("Jade Leech");
    // Two Forests aren't enough for Grizzly Bears any more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jade Leech");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Three are, but {1} of the extra must be green: a Mountain doesn't help.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jade Leech");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jade Leech");
    let forests = t.lands(P0, "Forest", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(forests.iter().all(|f| t.obj_now(*f).tapped));
    // A nongreen spell isn't affected, nor are the opponent's green spells.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jade Leech");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.lands(P1, "Forest", 1);
    let target = t.battlefield(P1, "Grizzly Bears");
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(target).go();
    t.resolve_all();
    assert_eq!(t.pt(target), (5, 5));
}

#[test]
fn black_scarab_cant_be_blocked_by_black_and_pumps_against_black() {
    cr!("303.4", "509.1b", "613.4c");
    // "Enchanted creature can't be blocked by black creatures. / Enchanted creature gets
    // +2/+2 as long as an opponent controls a black permanent."
    supported("Black Scarab");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let green = t.battlefield(P1, "Grizzly Bears");
    let scarab = t.battlefield(P0, "Black Scarab");
    t.g.attach(scarab, Entity::Object(bears));
    t.g.recompute();
    // No black permanent: no bonus.
    assert_eq!(t.pt(bears), (2, 2));
    // P0's own black permanent doesn't count.
    t.battlefield(P0, "Ravenous Rats");
    assert_eq!(t.pt(bears), (2, 2));
    let rats = t.battlefield(P1, "Ravenous Rats");
    assert_eq!(t.pt(bears), (4, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(rats, bears), "a black creature can't block it");
    assert!(t.g.can_block(green, bears));
    // The black blocker's block is rejected; the attack connects.
    block(&mut t, P1, &[(rats, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(rats));
    // When the black permanent leaves, the bonus ends.
    t.g.destroy(rats, None);
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn crabapple_cohort_gets_plus_one_with_another_green_creature() {
    cr!("613.4c", "604.1");
    // "This creature gets +1/+1 as long as you control another green creature."
    supported("Crabapple Cohort");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crabapple Cohort");
    assert_eq!(t.pt(c), (4, 4), "it doesn't count itself");
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(c), (4, 4), "an opponent's green creature doesn't count");
    t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(c), (4, 4), "a red creature doesn't count");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(c), (5, 5));
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.pt(c), (4, 4));
}

#[test]
fn reckless_one_counts_every_goblin_on_the_battlefield() {
    cr!("604.3", "613.4a");
    // "Reckless One's power and toughness are each equal to the number of Goblins on the
    // battlefield."
    supported("Reckless One");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Reckless One");
    assert_eq!(t.pt(r), (1, 1), "it's a Goblin itself");
    t.battlefield(P1, "Goblin Piker");
    assert_eq!(t.pt(r), (2, 2), "an opponent's Goblin counts");
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(r), (2, 2));
    t.battlefield(P0, "Goblin Piker");
    assert_eq!(t.pt(r), (3, 3));
    // In the graveyard it counts nothing on the battlefield as itself.
    t.graveyard(P0, "Goblin Piker");
    assert_eq!(t.pt(r), (3, 3));
}
