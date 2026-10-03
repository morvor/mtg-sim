//! Delayed triggered abilities created by instructions (CR 603.7): "[instruction] at the
//! beginning of the next / your next / that player's next [step]", "When that creature
//! [event] this turn, ...", and the durations "until ~ leaves the battlefield" (CR 610.3,
//! 611.2b) and "until the end of your next turn" (see `oracle/patterns/delayed_grammar.rs`).

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.players[p.idx()].mana_pool.total()
}

#[test]
fn stone_idol_trap_token_is_exiled_at_its_casters_next_end_step() {
    cr!("603.7", "603.7d");
    ruling!(
        "Stone Idol Trap",
        "the token remains on the battlefield after that player"
    );
    assert_supported("Stone Idol Trap");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    t.set_step(P1, Step::PrecombatMain);
    let trap = t.hand(P0, "Stone Idol Trap");
    t.cast(P0, trap).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Construct Token").len(), 1);
    // Not at the end step of the turn it was cast in: "your" next end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Construct Token").len(), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Construct Token").is_empty());
}

#[test]
fn eternal_wanderer_returns_the_card_at_its_owners_next_end_step() {
    cr!("603.7c");
    // (Its last loyalty ability isn't supported; the first one is what's tested.)
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "The Eternal Wanderer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, w, 0, &objs(&[bears])).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // That player is the card's owner: not at the end step of the turn it was exiled.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn phytotitan_returns_tapped_at_its_owners_next_upkeep() {
    cr!("603.7c", "400.7");
    assert_supported("Phytotitan");
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Phytotitan");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(titan).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phytotitan"));
    t.advance_to(P1, Step::Draw);
    assert!(t.in_graveyard(P0, "Phytotitan"));
    t.advance_to(P0, Step::Draw);
    let back = t.named_on_battlefield("Phytotitan");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
}

#[test]
fn mana_drain_adds_colorless_at_your_next_main_phase() {
    cr!("603.7", "505.1");
    ruling!(
        "Mana Drain",
        "will usually trigger at the beginning of your precombat main phase"
    );
    assert_supported("Mana Drain");
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    t.lands(P0, "Island", 2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    let drain = t.hand(P0, "Mana Drain");
    t.cast(P0, drain).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
}

#[test]
fn conduit_of_storms_adds_mana_at_the_next_main_phase_this_turn() {
    cr!("603.7b", "505.1");
    assert_supported("Conduit of Storms // Conduit of Emrakul");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Conduit of Storms // Conduit of Emrakul");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert_eq!(pool(&t, P0), 0);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 1);
}

#[test]
fn obzedat_returns_at_your_next_upkeep_with_haste() {
    cr!("603.7c", "400.7");
    ruling!(
        "Obzedat, Ghost Council",
        "it has haste for as long as it remains on the battlefield"
    );
    assert_supported("Obzedat, Ghost Council");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obzedat, Ghost Council");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Obzedat, Ghost Council"));
    t.advance_to(P1, Step::End);
    assert!(t.in_exile("Obzedat, Ghost Council"));
    t.advance_to(P0, Step::Draw);
    let back = t.named_on_battlefield("Obzedat, Ghost Council");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).has_keyword(KeywordKind::Haste));
}

#[test]
fn force_of_rage_tokens_are_sacrificed_at_your_next_upkeep() {
    cr!("603.7", "701.21a");
    assert_supported("Force of Rage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let f = t.hand(P0, "Force of Rage");
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Elemental Token").len(), 2);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.named_on_battlefield("Elemental Token").len(), 2);
    t.advance_to(P0, Step::Draw);
    assert!(t.named_on_battlefield("Elemental Token").is_empty());
}

#[test]
fn searing_blood_deals_damage_when_that_creature_dies_this_turn() {
    cr!("603.7a", "603.7c");
    assert_supported("Searing Blood");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sb = t.hand(P0, "Searing Blood");
    t.cast(P0, sb).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn time_to_feed_gains_life_whatever_kills_the_creature_later_this_turn() {
    cr!("603.7b", "701.14a");
    ruling!(
        "Time to Feed",
        "no matter what caused the creature to die"
    );
    assert_supported("Time to Feed");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ttf = t.hand(P0, "Time to Feed");
    t.cast(P0, ttf).targets(&objs(&[giant, bears])).go();
    t.resolve_all();
    // The Giant survived the fight; the Bears didn't.
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P0), 20);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(giant).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn hunters_insight_draws_that_many_cards() {
    cr!("603.7b", "510.2");
    assert_supported("Hunter's Insight");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let hi = t.hand(P0, "Hunter's Insight");
    t.cast(P0, hi).target(giant).go();
    t.resolve_all();
    let before = t.library_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), before - 3);
}

#[test]
fn barreling_attack_pumps_for_each_blocker() {
    cr!("603.7b", "509.1h");
    ruling!(
        "Barreling Attack",
        "will get +1/+1 for each creature blocking it as that ability resolves"
    );
    assert_supported("Barreling Attack");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let ba = t.hand(P0, "Barreling Attack");
    t.cast(P0, ba).target(giant).go();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(b1, giant), (b2, giant)]);
    assert_eq!(t.pt(giant), (5, 5));
    assert!(t.on_battlefield(giant));
}

#[test]
fn war_barge_destroys_that_creature_when_it_leaves_this_turn() {
    cr!("603.7c", "701.19c");
    assert_supported("War Barge");
    let mut t = TestGame::new(2);
    let barge = t.battlefield(P0, "War Barge");
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, barge, 0, &objs(&[bears])).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    let shatter = t.hand(P0, "Shatter");
    t.cast(P0, shatter).target(barge).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "War Barge"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn kitesail_freebooter_exiles_the_card_until_it_leaves() {
    cr!("610.3", "610.3c");
    assert_supported("Kitesail Freebooter");
    let mut t = TestGame::new(2);
    let shock = t.hand(P1, "Shock");
    t.answer_choose(P0, &objs(&[shock]));
    let fb = t.enter(P0, "Kitesail Freebooter");
    t.resolve_all();
    assert!(t.in_exile("Shock"));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(fb).go();
    t.resolve_all();
    // It returns to the zone it came from (CR 610.3).
    assert!(t.in_hand(P1, "Shock"));
}

#[test]
fn gruesome_encore_exiles_the_creature_if_it_would_leave() {
    cr!("614.1a", "603.7c");
    assert_supported("Gruesome Encore");
    let mut t = TestGame::new(2);
    let target = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 1);
    let ge = t.hand(P0, "Gruesome Encore");
    t.cast(P0, ge).target(target).go();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj_now(bears).controller, P0);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn blackstaff_animates_an_artifact_for_as_long_as_it_remains_tapped() {
    cr!("611.2b");
    assert_supported("The Blackstaff of Waterdeep");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "The Blackstaff of Waterdeep");
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Island", 2);
    t.activate(P0, staff, 0, &objs(&[ring])).unwrap();
    t.resolve_all();
    assert!(t.obj_now(ring).is(CardType::Creature));
    assert_eq!(t.pt(ring), (4, 4));
    t.g.obj_mut(staff).tapped = false;
    t.g.dirty = true;
    t.settle();
    assert!(!t.obj_now(ring).is(CardType::Creature));
}

#[test]
fn ray_of_command_taps_the_creature_when_you_lose_control_of_it() {
    cr!("603.7c", "514.3a");
    assert_supported("Ray of Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    let ray = t.hand(P0, "Ray of Command");
    t.cast(P0, ray).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // Control returns in the cleanup step; the delayed ability taps it then, and P0's
    // untap step doesn't untap P1's creature.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn merieke_destroys_the_creature_when_it_leaves() {
    cr!("603.7c", "701.19c");
    assert_supported("Merieke Ri Berit");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Merieke Ri Berit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, m, 0, &objs(&[bears])).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(m).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn overpowering_attack_adds_a_combat_and_a_main_phase_in_your_main_phase() {
    cr!("500.8", "505.1a");
    assert_supported("Overpowering Attack");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let oa = t.hand(P0, "Overpowering Attack");
    t.cast(P0, oa).go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    let combats = t
        .g
        .turn
        .step_log
        .iter()
        .filter(|s| **s == Step::BeginningOfCombat)
        .count();
    assert_eq!(combats, 2);
}

#[test]
fn cait_sith_pumps_by_the_exiled_cards_mana_value() {
    cr!("603.12");
    assert_supported("Cait Sith, Fortune Teller");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cait Sith, Fortune Teller");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.answer_targets(P0, &objs(&[bears]));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.pt(bears), (6, 2));
}

#[test]
fn psychic_pickpocket_returns_a_permanent_when_it_connives() {
    cr!("603.12", "701.50a");
    assert_supported("Psychic Pickpocket");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &objs(&[bears]));
    t.enter(P0, "Psychic Pickpocket");
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn graceful_antelope_land_is_a_plains_until_it_leaves() {
    cr!("611.2b", "305.7");
    assert_supported("Graceful Antelope");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Graceful Antelope");
    let forest = t.battlefield(P1, "Forest");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &objs(&[forest]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.obj_now(forest).chars.subtypes.iter().any(|s| s == "Plains"));
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(a).go();
    t.resolve_all();
    assert!(!t.obj_now(forest).chars.subtypes.iter().any(|s| s == "Plains"));
}

#[test]
fn legions_initiative_returns_the_creatures_at_the_next_combat_with_haste() {
    cr!("603.7c", "610.3");
    assert_supported("Legion's Initiative");
    let mut t = TestGame::new(2);
    let li = t.battlefield(P0, "Legion's Initiative");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.activate(P0, li, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t.obj_now(bears[0]).has_keyword(KeywordKind::Haste));
}

#[test]
fn dalkovan_encampment_sacrifices_each_attacks_tokens_at_the_end_step() {
    cr!("603.7b", "603.7c");
    assert_supported("Dalkovan Encampment");
    let mut t = TestGame::new(2);
    let camp = t.battlefield(P0, "Dalkovan Encampment");
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, camp, 1, &[]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Warrior Token").len(), 2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Warrior Token").is_empty());
}

#[test]
fn fire_giants_fury_exiles_that_many_cards_and_lets_you_play_them() {
    cr!("603.7b", "603.7c");
    assert_supported("Fire Giant's Fury");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let f = t.hand(P0, "Fire Giant's Fury");
    t.cast(P0, f).target(giant).go();
    t.resolve_all();
    let before = t.library_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 3 + 2 = 5 combat damage: five cards exiled.
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.library_size(P0), before - 5);
}

#[test]
fn touch_of_moonglove_punishes_the_controller_of_each_creature_it_killed() {
    cr!("603.7b", "702.2b");
    assert_supported("Touch of Moonglove");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let tm = t.hand(P0, "Touch of Moonglove");
    t.cast(P0, tm).target(bears).go();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn devouring_tendrils_gains_life_when_the_permanent_you_dont_control_dies() {
    cr!("603.7a");
    assert_supported("Devouring Tendrils");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dt = t.hand(P0, "Devouring Tendrils");
    t.cast(P0, dt).targets(&objs(&[giant, bears])).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn venomous_breath_destroys_what_blocked_it_at_this_turns_end_of_combat() {
    cr!("603.7b", "509.1");
    assert_supported("Venomous Breath");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    let other = t.battlefield(P1, "Grizzly Bears");
    let vb = t.hand(P0, "Venomous Breath");
    t.cast(P0, vb).target(giant).go();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(wall, giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wall of Stone"));
    assert!(t.on_battlefield(other));
    assert!(t.on_battlefield(giant));
}
