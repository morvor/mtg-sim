//! Rulings batch P022 — what happens to token copies over time: a token exiled "until"
//! ceases to exist (CR 111.7, 111.8, 610.3c), the delayed triggers that exile or sacrifice
//! them (CR 603.7, 603.7c, 513.1), several tokens from a replacement effect (CR 614.1a),
//! control changes (CR 701.21a), attack requirements and costs (CR 508.1d, 508.1h), copied
//! permanent spells (CR 707.10, 608.3f) and choices made for copies (CR 707.10, 616.1).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Exiled tokens ----------------------------------------------------------------------------

#[test]
fn an_exiled_token_doesnt_come_back_or_get_copied() {
    cr!("111.7", "111.8", "610.3c");
    ruling!("Phantom Steed", "If a token is exiled this way, it will cease to exist and won't return to the battlefield. Phantom Steed's last ability won't create any tokens.");
    ruling!("Mardu Siegebreaker", "If a token is exiled with Mardu Siegebreaker’s third ability, it will cease to exist and won’t return to the battlefield. Mardu Siegebreaker’s last ability won’t create copies of that token.");
    for name in ["Phantom Steed", "Mardu Siegebreaker"] {
        supported(name);
        let mut t = TestGame::new(2);
        let wolf = wolf_token(&mut t, P0);
        t.answer_targets(P0, &[obj(wolf)]);
        let steed = t.enter(P0, name);
        t.resolve_all();
        assert!(!t.on_battlefield(wolf), "{name}");
        assert!(!t.in_exile("Wolf"), "{name}: the token ceased to exist");
        unsick(&mut t, steed);
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(steed, Entity::Player(P1))]),
        );
        let before = t.g.battlefield.clone();
        t.advance_to(P0, Step::DeclareAttackers);
        t.resolve_all();
        assert!(new_tokens(&t, P0, &before).is_empty(), "{name}");
        // It left the battlefield: no Wolf returns.
        kill(&mut t, steed);
        t.resolve_all();
        assert!(with_subtype(&t, P0, "Wolf").is_empty(), "{name}");
    }
}

// --- Arcane Artisan -----------------------------------------------------------------------------

/// P0's Arcane Artisan activates: P0 draws, then exiles `card` (a creature card in P0's
/// hand) and creates token copies of it. Returns the tokens.
fn artisan_makes(t: &mut TestGame, artisan: ObjectId, card: ObjectId) -> Vec<ObjectId> {
    lands_for(t, P0, "{2}{U}");
    t.answer_choose(P0, &[obj(card)]);
    activate_for_tokens(t, P0, artisan, "draws a card", &[Entity::Player(P0)])
}

#[test]
fn arcane_artisan_exiles_all_its_tokens_but_not_copies_of_them() {
    cr!("603.7c", "614.1a", "707.2");
    ruling!("Arcane Artisan", "If an effect such as that of Doubling Season causes Arcane Artisan’s first ability to create two tokens, its last ability will exile both of those tokens. However, if something becomes a copy of the token or creates a copy of that token later (such as Evil Twin or Cackling Counterpart), that copy won’t be exiled.");
    supported("Arcane Artisan");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let artisan = t.battlefield(P0, "Arcane Artisan");
    let card = t.hand(P0, "Hill Giant");
    let toks = artisan_makes(&mut t, artisan, card);
    assert_eq!(toks.len(), 2);
    // Evil Twin enters as a copy of one; Cackling Counterpart copies the other.
    let twin = enter_copying(&mut t, P0, "Evil Twin", toks[0]);
    let counterpart = cast_spell(
        &mut t,
        P0,
        "Cackling Counterpart",
        "{1}{U}{U}",
        &[obj(toks[1])],
        None,
        None,
    );
    assert_eq!(counterpart.len(), 2, "Doubling Season doubles it too");
    kill(&mut t, artisan);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.on_battlefield(toks[0]) && !t.on_battlefield(toks[1]));
    assert!(t.on_battlefield(twin));
    assert!(counterpart.iter().all(|c| t.on_battlefield(*c)));
}

#[test]
fn arcane_artisan_tokens_end_step_triggers_resolve_after_theyre_exiled() {
    cr!("603.3b", "603.10a", "113.7a");
    ruling!("Arcane Artisan", "If any of those tokens have an ability that triggers at the beginning of the end step, that ability will trigger and resolve even if the token is exiled during the end step before that ability resolves.");
    supported("Alert Heedbonder");
    // Alert Heedbonder: "At the beginning of your end step, you gain 1 life for each
    // creature you control with vigilance." P0 also controls Serra Angel (vigilance).
    let mut gains = Vec::new();
    for order in [[0, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        let artisan = t.battlefield(P0, "Arcane Artisan");
        t.battlefield(P0, "Serra Angel");
        let card = t.hand(P0, "Alert Heedbonder");
        let tok = one(artisan_makes(&mut t, artisan, card));
        kill(&mut t, artisan);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(!t.on_battlefield(tok));
        gains.push(t.life(P0) - 20);
    }
    // Either order: the Heedbonder's ability resolved (2 life before the token was
    // exiled, 1 after).
    gains.sort();
    assert_eq!(gains, vec![1, 2]);
}

#[test]
fn arcane_artisans_each_exile_only_their_own_tokens() {
    cr!("603.7c", "607.2c");
    ruling!("Arcane Artisan", "If multiple Arcane Artisans are creating tokens, each will exile only the ones it created.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Arcane Artisan");
    let b = t.battlefield(P0, "Arcane Artisan");
    let giant = t.hand(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    let ta = one(artisan_makes(&mut t, a, giant));
    let tb = one(artisan_makes(&mut t, b, bears));
    kill(&mut t, a);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.on_battlefield(ta));
    assert!(t.on_battlefield(tb));
}

// --- Several tokens from Doubling Season ------------------------------------------------------

#[test]
fn doubled_temporary_tokens_are_each_exiled() {
    cr!("614.1a", "603.7c");
    ruling!("Twinflame", "However, if Twinflame creates multiple tokens copying a single creature due to a replacement effect (like the one Doubling Season creates), you'll exile each of them.");
    ruling!("Felhide Spiritbinder", "However, if Felhide Spiritbinder creates multiple tokens due to a replacement effect (like the one Doubling Season creates), each of those tokens will have haste and you’ll exile each of them.");
    ruling!("Flamerush Rider", "However, if Flamerush Rider creates multiple tokens due to a replacement effect (like the one Doubling Season creates), you'll exile each of those tokens.");
    // Twinflame.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let giant = t.battlefield(P0, "Hill Giant");
    let toks = cast_spell(&mut t, P0, "Twinflame", "{1}{R}", &[obj(giant)], None, None);
    assert_eq!(toks.len(), 2);
    t.advance_to(P1, Step::Upkeep);
    assert!(toks.iter().all(|x| !t.on_battlefield(*x)));
    // Felhide Spiritbinder (inspired: it becomes untapped).
    supported("Felhide Spiritbinder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let s = t.battlefield(P0, "Felhide Spiritbinder");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.tap(s);
    lands_for(&mut t, P0, "{1}{R}");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.g.untap(s);
    t.g.flush_events();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| has_kw(&t, *x, KeywordKind::Haste)));
    t.advance_to(P1, Step::Upkeep);
    assert!(toks.iter().all(|x| !t.on_battlefield(*x)));
    // Flamerush Rider: exiled at end of combat.
    supported("Flamerush Rider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let rider = t.battlefield(P0, "Flamerush Rider");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(rider, Entity::Player(P1)), (giant, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(toks.iter().all(|x| !t.on_battlefield(*x)));
}

// --- "the next end step" during an end step -------------------------------------------------

#[test]
fn tokens_made_during_an_end_step_are_exiled_at_the_next_turns_end_step() {
    cr!("603.7c", "513.1a");
    ruling!("Splinter Twin", "If the ability is activated during a turn's end step, the token will be exiled at the beginning of the following turn's end step.");
    ruling!("Flameshadow Conjuring", "If the ability resolves during a turn's end step, the token will be exiled at the beginning of the next turn's end step.");
    supported("Splinter Twin");
    supported("Flameshadow Conjuring");
    // Splinter Twin on P0's Grizzly Bears, activated in P0's end step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Splinter Twin", obj(bears));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let toks = activate_for_tokens(&mut t, P0, bears, "Create a token", &[]);
    let tok = one(toks);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(tok), "not exiled in the end step it was made in");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
    // Flameshadow Conjuring: a creature enters in P0's end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flameshadow Conjuring");
    t.lands(P0, "Mountain", 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(tok));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
}

// --- Encore tokens ----------------------------------------------------------------------------

/// P0 activates the encore ability of Eumidian Wastewaker in their graveyard (P0's main
/// phase); returns the tokens.
fn encore_wastewaker(t: &mut TestGame) -> Vec<ObjectId> {
    supported("Eumidian Wastewaker");
    let card = t.graveyard(P0, "Eumidian Wastewaker");
    lands_for(t, P0, "{6}{B}{B}");
    let before = t.g.battlefield.clone();
    activate_containing(t, P0, card, "Encore").expect("encore");
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn an_encore_token_another_player_controls_isnt_sacrificed() {
    cr!("701.21a", "603.7c");
    ruling!("Eumidian Wastewaker", "If one of the tokens somehow is under another player’s control as the delayed triggered ability resolves, you can’t sacrifice that token. It remains on the battlefield indefinitely, even if you regain control of it later.");
    let mut t = TestGame::new(2);
    let tok = one(encore_wastewaker(&mut t));
    give_control(&mut t, tok, P1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(tok));
    give_control(&mut t, tok, P0);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(tok));
    // Under P0's control it would have been sacrificed.
    let mut t = TestGame::new(2);
    let tok = one(encore_wastewaker(&mut t));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.on_battlefield(tok));
}

#[test]
fn an_encore_token_that_cant_attack_or_would_need_a_cost_doesnt_have_to() {
    cr!("508.1d", "508.1h");
    ruling!("Eumidian Wastewaker", "If one of the tokens can’t attack for any reason (such as being tapped), then it doesn’t attack. If there’s a cost associated with having it attack, you aren’t forced to pay that cost, so it doesn’t have to attack in that case either.");
    // Untapped and with no cost, the requirement makes it attack even if P0 declares no
    // attackers.
    let mut t = TestGame::new(2);
    let tok = one(encore_wastewaker(&mut t));
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.is_attacking(tok));
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    let tok = one(encore_wastewaker(&mut t));
    t.g.tap(tok);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!t.g.is_attacking(tok));
    // Ghostly Prison ("Creatures can't attack you unless their controller pays {2} for
    // each creature they control that's attacking you"): it needn't attack.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghostly Prison");
    let tok = one(encore_wastewaker(&mut t));
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!t.g.is_attacking(tok));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 8);
}
