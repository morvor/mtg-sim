//! Rulings batch P021 — assorted rulings about cards that copy creatures: optional
//! payments made once per trigger (CR 603.5), optional exiling and linked "exiled with"
//! cards (CR 607.2a), costs paid on activation (CR 602.2b), delayed triggers and
//! replacement-multiplied tokens (CR 603.7, 614.1a), extra triggers (CR 603.2d), a
//! copy effect's duration (CR 611.2a), and "until leaves" exile (CR 610.3b, 610.3c).

use crate::r_p021_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::lands_for;
use crate::r_s26_common::{mv, new_tokens};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn riku_pays_once_per_trigger_for_one_copy() {
    cr!("603.5", "707.10");
    ruling!(
        "Riku of Two Reflections",
        "Each time the first ability triggers, you can pay {U}{R} only one time to get one copy of the spell."
    );
    supported("Riku of Two Reflections");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    lands_for(&mut t, P0, "{U}{R}{U}{R}{U}{R}{G}{U}{G}{U}{G}{U}{R}");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    // The Bolt and one copy.
    assert_eq!(t.life(P1), 20 - 3 - 3);
    t.clear_answers();
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    let before = t.g.battlefield.clone();
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);
}

/// P0's Mimic Vat with Grizzly Bears exiled with it (P1's Bears died and P0 exiled it).
fn vat_with_bears(t: &mut TestGame) -> ObjectId {
    supported("Mimic Vat");
    let vat = t.battlefield(P0, "Mimic Vat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    destroy(t, bears);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    vat
}

#[test]
fn mimic_vat_exiling_is_optional_and_does_nothing_if_the_card_is_gone() {
    cr!("603.5", "607.2a", "400.7");
    ruling!(
        "Mimic Vat",
        "Exiling the card as the first ability resolves is optional."
    );
    let mut t = TestGame::new(2);
    vat_with_bears(&mut t);
    // Choosing not to exile Hill Giant: nothing happens; the Bears stays exiled.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_exile("Grizzly Bears"));
    // Serra Angel dies, but leaves the graveyard before the ability resolves.
    let angel = t.battlefield(P1, "Serra Angel");
    destroy(&mut t, angel);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.current(angel);
    t.g.move_object(
        card,
        Zone::Hand(P1),
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    );
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P1, "Serra Angel"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn mimic_vat_token_made_in_an_end_step_is_exiled_at_the_next_end_step() {
    cr!("603.7c", "513.2");
    ruling!(
        "Mimic Vat",
        "If Mimic Vat's second ability is activated during a turn's end step, the token will be exiled at the beginning of the following turn's end step."
    );
    let mut t = TestGame::new(2);
    let vat = vat_with_bears(&mut t);
    t.set_step(P0, Step::End);
    lands_for(&mut t, P0, "{3}");
    let before = t.g.battlefield.clone();
    t.answer_choose(P0, &[]);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(toks[0]));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(toks[0]));
}

#[test]
fn wire_surgeons_encore_exiles_the_card_as_a_cost() {
    cr!("602.2b", "702.141a", "601.2h");
    ruling!(
        "Wire Surgeons",
        "Exiling the card with encore is a cost to activate the ability."
    );
    supported("Wire Surgeons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wire Surgeons");
    let thopter = t.graveyard(P0, "Ornithopter");
    activate_containing(&mut t, P0, thopter, "Encore").expect("encore");
    // The card is already exiled while the ability is on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Ornithopter"));
    assert!(!t.in_graveyard(P0, "Ornithopter"));
    let before = t.g.battlefield.clone();
    t.resolve_all();
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);
}

#[test]
fn fleeting_reflection_can_target_an_untapped_creature() {
    cr!("115.1", "707.2");
    ruling!(
        "Fleeting Reflection",
        "Fleeting Reflection’s first target can be a creature that’s already untapped."
    );
    supported("Fleeting Reflection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(!t.obj_now(bears).tapped);
    lands_for(&mut t, P0, "{1}{U}");
    let spell = t.hand(P0, "Fleeting Reflection");
    t.cast(P0, spell).target(obj(bears)).target(obj(giant)).go();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn tawnos_copies_of_x_cost_cards_and_tokens_have_x_zero() {
    cr!("202.3e", "707.2");
    ruling!(
        "Tawnos, Solemn Survivor",
        "For both abilities, if a token is created copying a token or card with {X} in its mana cost, X is 0."
    );
    supported("Tawnos, Solemn Survivor");
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Solemn Survivor");
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Treasure");
    let chalice = t.graveyard(P0, "Chalice of the Void");
    lands_for(&mut t, P0, "{1}{W}{U}{B}");
    let before = t.g.battlefield.clone();
    t.answer_choose(P0, &[obj(chalice)]);
    activate_containing(&mut t, P0, tawnos, "Sacrifice two artifact tokens").expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Chalice of the Void");
    assert_eq!(mv(&mut t, toks[0]), 0);
    assert_eq!(t.counters(toks[0], counters::CHARGE), 0);
    // The first ability copying that token (with {X}{X} in its mana cost): X is 0 too.
    let tawnos = t.g.current(tawnos);
    t.g.untap(tawnos);
    lands_for(&mut t, P0, "{2}");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(toks[0])]);
    activate_containing(&mut t, P0, tawnos, "up to one target artifact token").expect("activate");
    t.resolve_all();
    let toks2: Vec<ObjectId> = new_tokens(&t, P0, &before);
    assert_eq!(toks2.len(), 1);
    assert_eq!(mv(&mut t, toks2[0]), 0);
}

#[test]
fn crime_triggers_and_cast_triggers_trigger_together_and_resolve_first() {
    cr!("603.3b", "700.13", "601.2i");
    ruling!(
        "Vadmir, New Blood",
        "For example, an ability that triggers when you cast a spell that targets an opponent will trigger at the same time as an ability that triggers whenever you commit a crime."
    );
    supported("Vadmir, New Blood");
    supported("Legolas, Master Archer");
    let mut t = TestGame::new(2);
    let vadmir = t.battlefield(P0, "Vadmir, New Blood");
    let legolas = t.battlefield(P0, "Legolas, Master Archer");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{R}");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(obj(giant)).go();
    t.settle();
    // Both triggered (one a crime trigger, one a cast trigger), above the Bolt.
    assert_eq!(t.stack_len(), 3);
    let bottom = t.g.stack[0];
    assert_eq!(t.obj_now(bottom).chars.name, "Lightning Bolt");
    // Both resolve before the Bolt does.
    t.resolve();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(vadmir, counters::PLUS1), 1);
    assert!(t.on_battlefield(giant));
    let _ = legolas;
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

/// P0's Mirror Room // Fractured Realm with Fractured Realm unlocked.
fn fractured_realm(t: &mut TestGame) -> ObjectId {
    supported("Mirror Room // Fractured Realm");
    let room = t.battlefield(P0, "Mirror Room // Fractured Realm");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.g.flush_events();
    t.resolve_all();
    t.g.recompute();
    room
}

#[test]
fn fractured_realm_triggers_have_separate_choices() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Mirror Room // Fractured Realm",
        "Fractured Realm's ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    supported("Flametongue Kavu");
    let mut t = TestGame::new(2);
    fractured_realm(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Two instances of Flametongue Kavu's trigger, each with its own target.
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Flametongue Kavu");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(giant) && !t.on_battlefield(bears));
    // Choices on resolution too: Renegade Doppelganger's "you may" is answered for each.
    let dopp = t.battlefield(P0, "Renegade Doppelganger");
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.enter(P0, "Serra Angel");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.obj_now(dopp).chars.name, "Renegade Doppelganger");
    t.resolve();
    assert_eq!(t.obj_now(dopp).chars.name, "Serra Angel");
}

/// The number of activated encore abilities the object has.
fn encore_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))
                && a.text.contains("Encore")
        })
        .count()
}

#[test]
fn graywaters_fixer_itself_doesnt_have_encore() {
    cr!("611.3a", "702.141a");
    ruling!(
        "Graywater's Fixer",
        "Graywater's Fixer itself doesn't have encore. Its ability applies only while it's on the battlefield."
    );
    supported("Graywater's Fixer");
    let mut t = TestGame::new(2);
    let fixer = t.battlefield(P0, "Graywater's Fixer");
    let miner = t.graveyard(P0, "Forsaken Miner");
    assert_eq!(encore_abilities(&t, fixer), 0);
    assert_eq!(encore_abilities(&t, miner), 1);
    destroy(&mut t, fixer);
    assert!(t.in_graveyard(P0, "Graywater's Fixer"));
    assert_eq!(encore_abilities(&t, fixer), 0);
    assert_eq!(encore_abilities(&t, miner), 0);
}

#[test]
fn hall_of_mirrors_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c", "702.159a");
    ruling!(
        "Hall of Mirrors",
        "Hall of Mirrors affects only creatures you control at the time its ability resolves."
    );
    supported("Hall of Mirrors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hall of Mirrors");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lit = mtg_engine::card::card("Hall of Mirrors").attraction_lights[0];
    t.g.dice.loaded.push_back(lit);
    t.answer_targets(P0, &[obj(giant)]);
    mtg_engine::variants::roll_to_visit(&mut t.g, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    let later = t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.obj_now(later).chars.name, "Llanowar Elves");
}

#[test]
fn identity_thief_is_still_a_copy_when_the_exiled_card_returns() {
    cr!("611.2a", "603.7", "610.3c");
    ruling!(
        "Identity Thief",
        "Identity Thief will still be a copy of the creature during the next end step when the exiled card returns."
    );
    supported("Identity Thief");
    supported("Ball Lightning");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Identity Thief");
    let ball = t.battlefield(P1, "Ball Lightning");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(ball)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    assert_eq!(t.obj_now(thief).chars.name, "Ball Lightning");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Ball Lightning's "sacrifice it" triggered for the Thief, not for the returned card.
    assert!(!t.on_battlefield(thief));
    assert!(t.in_graveyard(P0, "Identity Thief"));
    let returned = t.named_on_battlefield("Ball Lightning");
    assert_eq!(returned.len(), 1);
    assert_eq!(t.obj_now(returned[0]).controller, P1);
}

/// Each of the tokens `make` creates with Parallel Lives on the battlefield has haste
/// and is sacrificed at the beginning of the next end step.
fn doubled_tokens_all_sacrificed(make: fn(&mut TestGame)) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Parallel Lives");
    let before = t.g.battlefield.clone();
    make(&mut t);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    for tok in &toks {
        assert!(t.obj_now(*tok).has_keyword(KeywordKind::Haste));
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for tok in &toks {
        assert!(!t.on_battlefield(*tok));
    }
}

#[test]
fn feldon_doubled_tokens_all_gain_haste_and_are_sacrificed() {
    cr!("614.1a", "603.7c", "111.1");
    ruling!(
        "Feldon of the Third Path",
        "If Feldon's ability creates multiple tokens due to a replacement effect (such as the one Doubling Season creates), each of those tokens will gain haste and you'll sacrifice each of them."
    );
    supported("Feldon of the Third Path");
    supported("Parallel Lives");
    doubled_tokens_all_sacrificed(|t| {
        let feldon = t.battlefield(P0, "Feldon of the Third Path");
        let giant = t.graveyard(P0, "Hill Giant");
        lands_for(t, P0, "{2}{R}");
        t.activate(P0, feldon, 0, &[obj(giant)]).expect("activate");
    });
}

#[test]
fn kiki_jiki_doubled_tokens_are_all_sacrificed() {
    cr!("614.1a", "603.7c", "111.1");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "If Kiki-Jiki's ability creates multiple tokens due to a replacement effect (such as the one Doubling Season creates), you'll sacrifice each of them."
    );
    supported("Kiki-Jiki, Mirror Breaker");
    supported("Parallel Lives");
    doubled_tokens_all_sacrificed(|t| {
        let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
        let giant = t.battlefield(P0, "Hill Giant");
        t.activate(P0, kiki, 0, &[obj(giant)]).expect("activate");
    });
}

#[test]
fn permeating_mass_copying_something_else_makes_the_damaged_creature_copy_that() {
    cr!("608.2h", "707.2", "510.3a");
    ruling!(
        "Permeating Mass",
        "If Permeating Mass becomes a copy of another creature before its triggered ability resolves, the damaged creature will become a copy of that creature."
    );
    supported("Permeating Mass");
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let mass = t.battlefield(P0, "Permeating Mass");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{G}{U}");
    let cyto = t.hand(P0, "Cytoshape");
    attack_with(&mut t, &[(mass, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, mass)]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::CombatDamage && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok, "no combat damage trigger");
    // In response, Cytoshape makes Permeating Mass a copy of Hill Giant.
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, cyto).target(obj(mass)).go();
    t.resolve();
    assert_eq!(t.obj_now(mass).chars.name, "Hill Giant");
    t.resolve();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn phantom_steed_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b");
    ruling!(
        "Phantom Steed",
        "If Phantom Steed leaves the battlefield before its ability resolves, the target creature won't be exiled."
    );
    supported("Phantom Steed");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let steed = t.enter(P0, "Phantom Steed");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, steed);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.current(bears), bears);
}
