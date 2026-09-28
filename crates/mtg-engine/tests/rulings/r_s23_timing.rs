//! Rulings batch S23 — when spells can be cast and abilities activated (CR 117.1a, 307.1,
//! 601.3, 602.5): spells cast from the top of a library or a graveyard by a permission
//! keep their normal timing, a player who can't cast spells can't suspend cards, an
//! activated ability with no timing restriction can be activated at instant speed, and
//! nothing happens between the parts of a resolving ability.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s14_common::triggers_from;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;
const JUMP_START: CastMethod = CastMethod::Keyword(KeywordKind::JumpStart);

#[test]
fn mystic_forge_spells_from_the_library_keep_their_normal_timing() {
    cr!("601.3", "307.1", "117.1a", "401.5");
    ruling!(
        "Mystic Forge",
        "You must follow the normal timing permissions and restrictions of the spells you cast from your library."
    );
    supported("Mystic Forge");
    // Mystic Forge: "You may cast artifact spells and colorless spells from the top of
    // your library." Bottle Gnomes (an artifact creature) is on top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mystic Forge");
    t.lands(P0, "Wastes", 3);
    let gnomes = t.library_top(P0, "Bottle Gnomes");
    t.g.recompute();
    // In P0's main phase with an empty stack, it can be cast.
    assert!(can_cast(&mut t, P0, gnomes, NORMAL));
    // Not while a spell is on the stack ...
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P1)).go();
    assert!(!can_cast(&mut t, P0, gnomes, NORMAL));
    t.resolve_all();
    assert!(can_cast(&mut t, P0, gnomes, NORMAL));
    // ... nor during P0's combat, nor during P1's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, gnomes, NORMAL));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, gnomes, NORMAL));
}

#[test]
fn a_sorcery_cast_with_jump_start_keeps_sorcery_timing() {
    cr!("702.133a", "307.1", "601.3");
    ruling!(
        "Start the TARDIS",
        "When casting a spell with jump-start, you must still follow any timing restrictions and permissions, including those based on the card's type."
    );
    supported("Start the TARDIS");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.hand(P0, "Forest");
    let tardis = t.graveyard(P0, "Start the TARDIS");
    assert!(can_cast(&mut t, P0, tardis, JUMP_START));
    // Not in P0's upkeep, nor in P1's turn, nor with a spell on the stack.
    t.set_step(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, tardis, JUMP_START));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, tardis, JUMP_START));
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P1)).go();
    assert!(!can_cast(&mut t, P0, tardis, JUMP_START));
    t.resolve_all();
    // In P0's main phase with an empty stack: cast it, discarding the Forest.
    t.cast(P0, tardis).method(JUMP_START).go();
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn moria_marauder_cards_follow_normal_timing_and_spells_cost_their_costs() {
    cr!("601.3", "307.1", "305.2", "601.2f");
    ruling!(
        "Moria Marauder",
        "You must follow all normal timing rules for a card you play using Moria Marauder's last ability and, if it's a spell, you must pay its costs to cast it."
    );
    supported("Moria Marauder");
    // "Whenever a Goblin or Orc you control deals combat damage to a player, exile the top
    // card of your library. You may play that card this turn." With double strike, two
    // cards are exiled: Divination and a Forest.
    let mut t = TestGame::new(2);
    let marauder = t.battlefield(P0, "Moria Marauder");
    stack_library(&mut t, P0, &["Divination", "Forest"]);
    attack_with(&mut t, &[(marauder, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    // (Moria Marauder is a 1/1 with double strike.)
    assert_eq!(t.life(P1), 18);
    let divination = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Divination")[0];
    let forest = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Forest")[0];
    t.lands(P0, "Island", 3);
    // During combat, the sorcery can't be cast and the land can't be played.
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    assert!(!can_cast(&mut t, P0, divination, NORMAL));
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    // In the postcombat main phase they can; the sorcery's mana cost is paid.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, divination, NORMAL));
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, forest));
    let hand = t.hand_size(P0);
    t.cast(P0, divination).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Without the mana, it can't be cast.
    let mut t = TestGame::new(2);
    let marauder = t.battlefield(P0, "Moria Marauder");
    stack_library(&mut t, P0, &["Divination"]);
    attack_with(&mut t, &[(marauder, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let divination = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Divination")[0];
    assert!(!can_cast(&mut t, P0, divination, NORMAL));
}

/// P1 suspends `card` (paying its suspend cost), as a special action.
fn try_suspend(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.perform_action(p, Action::Special(SpecialAction::Suspend { card }))
        .is_ok()
}

#[test]
fn a_player_who_cant_cast_spells_cant_suspend_a_card() {
    cr!("702.62a", "116.2f", "601.3");
    ruling!(
        "Silence",
        "A player who can't cast a spell can't suspend a card."
    );
    supported("Silence");
    supported("Rift Bolt");
    // In P1's main phase, P0 casts Silence: "Your opponents can't cast spells this turn."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    let silence = t.hand(P0, "Silence");
    t.cast(P0, silence).go();
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Rift Bolt");
    assert!(!try_suspend(&mut t, P1, bolt));
    assert!(t.in_hand(P1, "Rift Bolt"));
    // Without Silence, P1 could suspend it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Rift Bolt");
    assert!(try_suspend(&mut t, P1, bolt));
    assert!(t.in_exile("Rift Bolt"));
}

#[test]
fn dementia_bat_can_be_activated_in_the_opponents_draw_step() {
    cr!("602.5", "117.1b", "504.2");
    ruling!(
        "Dementia Bat",
        "Unlike most abilities that force a player to discard cards, this ability may be activated whenever you could cast an instant, including during your opponent’s draw step after they have drawn a card."
    );
    supported("Dementia Bat");
    let mut t = TestGame::new(2);
    let bat = t.battlefield(P0, "Dementia Bat");
    t.lands(P0, "Swamp", 5);
    t.hand(P1, "Forest");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Draw);
    // P1 has drawn for the turn: two cards in hand. P0 activates the Bat in response.
    assert_eq!(t.hand_size(P1), 2);
    t.g.turn.priority = Some(P0);
    t.activate(P0, bat, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 2);
}

/// Whether a card named Lightning Bolt is in P0's hand.
fn bolt_in_hand(g: &Game) -> bool {
    g.player(P0)
        .hand
        .iter()
        .any(|c| g.obj(*c).chars.name.as_str() == "Lightning Bolt")
}

#[test]
fn nothing_happens_between_merfolk_looters_draw_and_discard() {
    cr!("608.2c", "117.3", "608.2");
    ruling!(
        "Merfolk Looter",
        "You can't do anything in between drawing a card and discarding a card, including casting or cycling the card you drew."
    );
    supported("Merfolk Looter");
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Merfolk Looter");
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    // P0 never has priority with the drawn Lightning Bolt in hand: it's discarded as the
    // ability resolves.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        bolt_in_hand,
    );
    let at_discard = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        bolt_in_hand,
    );
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, looter, 0, &[]).unwrap();
    let ok = t.g.run_until(1000, |g| g.stack.is_empty());
    assert!(ok);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(at_discard.lock().unwrap().clone(), vec![true]);
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|x| !*x));
}

#[test]
fn wavebreak_hippocamp_triggers_only_on_the_first_spell_during_the_turn() {
    cr!("603.2", "603.2h");
    ruling!(
        "Wavebreak Hippocamp",
        "This ability triggers only on your very first spell during an opponent's turn, not the first spell after the card is on the battlefield."
    );
    supported("Wavebreak Hippocamp");
    supported("Vedalken Orrery");
    // "Whenever you cast your first spell during each opponent's turn, draw a card."
    let mut t = TestGame::new(2);
    let hippo = t.battlefield(P0, "Wavebreak Hippocamp");
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Island", 2);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, hippo), 1);
    t.resolve_all();
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, hippo), 0);
    t.resolve_all();
    // A spell cast before it's on the battlefield was the first: no trigger later.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Island", 2);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    let hippo = t.battlefield(P0, "Wavebreak Hippocamp");
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, hippo), 0);
    t.resolve_all();
    // Nor if Wavebreak Hippocamp itself was that spell (cast as though it had flash).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Wavebreak Hippocamp");
    t.cast(P0, card).go();
    t.resolve_all();
    let hippo = t.named_on_battlefield("Wavebreak Hippocamp")[0];
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, hippo), 0);
}

/// Whether P0 may look at the card on top of their library now.
fn sees_top(g: &Game) -> bool {
    g.library_top(P0)
        .is_some_and(|c| mtg_engine::zones::can_see_in_library(g, P0, c))
}

#[test]
fn the_new_top_card_cant_be_looked_at_until_the_spell_is_cast() {
    cr!("401.5", "601.2a", "601.2i");
    ruling!(
        "Hakoda, Selfless Commander",
        "If the top card of your library changes while you're casting a spell, playing a land, activating an ability, or taking a special action, you can't look at the new top card until you finish doing so."
    );
    supported("Hakoda, Selfless Commander");
    supported("Wandering Musicians");
    // Hakoda: "You may look at the top card of your library any time. You may cast Ally
    // spells from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hakoda, Selfless Commander");
    let cards = stack_library(&mut t, P0, &["Wandering Musicians", "Forest"]);
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(sees_top(&t.g));
    // Wandering Musicians ({3}{R/W}) is cast from the top; while paying for it (choosing
    // how to pay {R/W}), P0 can't look at the Forest, the new top card.
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::R, 1);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::W, 1);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 3);
    let paying = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("How will you pay")),
        sees_top,
    );
    t.cast(P0, cards[0]).go();
    assert_eq!(paying.lock().unwrap().clone(), vec![false]);
    // Once it's cast, the Forest can be looked at.
    assert_eq!(t.g.library_top(P0), Some(cards[1]));
    assert!(sees_top(&t.g));
}

#[test]
fn omniscience_spells_keep_their_normal_timing() {
    cr!("601.3", "307.1", "117.1a", "118.9");
    ruling!(
        "Omniscience",
        "You must follow the normal timing permissions and restrictions of each spell you cast."
    );
    supported("Omniscience");
    // "You may cast spells from your hand without paying their mana costs."
    let free = CastMethod::Free;
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let divination = t.hand(P0, "Divination");
    let bolt = t.hand(P0, "Lightning Bolt");
    // In P0's main phase with an empty stack, the sorcery can be cast for free.
    assert!(can_cast(&mut t, P0, divination, free.clone()));
    // Not during P1's turn: only the instant can.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, divination, free.clone()));
    assert!(can_cast(&mut t, P0, bolt, free.clone()));
    // Nor in P0's main phase while a spell is on the stack.
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, bolt)
        .method(free.clone())
        .target(Entity::Player(P1))
        .go();
    assert!(!can_cast(&mut t, P0, divination, free.clone()));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // With the stack empty again, P0 casts Divination without paying its mana cost.
    let hand = t.hand_size(P0);
    t.cast(P0, divination).method(free).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn glarb_cards_from_the_library_follow_all_costs_and_timing_rules() {
    cr!("601.3", "305.2", "116.2a", "307.1", "601.2f");
    ruling!(
        "Glarb, Calamity's Augur",
        "You must pay all costs and follow all timing rules for lands played and spells cast from the top of your library this way."
    );
    supported("Glarb, Calamity's Augur");
    // "You may play lands and cast spells with mana value 4 or greater from the top of
    // your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glarb, Calamity's Augur");
    let giant = t.library_top(P0, "Hill Giant");
    t.g.recompute();
    // Hill Giant ({3}{R}) costs its mana cost: not castable without the mana.
    assert!(!can_cast(&mut t, P0, giant, NORMAL));
    t.lands(P0, "Mountain", 4);
    assert!(can_cast(&mut t, P0, giant, NORMAL));
    // Not during P1's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, giant, NORMAL));
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, giant).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    // A spell with mana value less than 4 can't be cast this way.
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.g.recompute();
    assert!(!can_cast(&mut t, P0, bears, NORMAL));
    // A land from the top is played as the land for the turn: not if one was already
    // played, and only in a main phase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glarb, Calamity's Augur");
    let forest = t.library_top(P0, "Forest");
    t.g.recompute();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::PrecombatMain);
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, forest));
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).unwrap();
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
}

/// P0's The Belligerent, crewed by a Hill Giant, attacks P1; its trigger resolves (in the
/// declare attackers step).
fn belligerent_attacks(t: &mut TestGame) {
    let belligerent = t.battlefield(P0, "The Belligerent");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s04_common::crew(t, P0, belligerent, &[giant]));
    attack_with(t, &[(belligerent, Entity::Player(P1))]);
    t.resolve();
}

#[test]
fn the_belligerent_can_cast_an_adventure_from_the_top_of_the_library() {
    cr!("715.3", "715.3a", "601.3e", "611.2a");
    ruling!(
        "The Belligerent",
        "If the top card of your library has an Adventure, you can cast the Adventure spell this way."
    );
    ruling!(
        "The Belligerent",
        "Once The Belligerent's triggered ability resolves, you can look at the top card of your library whenever you want until end of turn"
    );
    supported("The Belligerent");
    supported("Bonecrusher Giant // Stomp");
    // "Whenever The Belligerent attacks, create a Treasure token. Until end of turn, you
    // may look at the top card of your library any time, and you may play lands and cast
    // spells from the top of your library."
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    t.g.recompute();
    assert!(!sees_top(&t.g));
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, top).is_empty());
    belligerent_attacks(&mut t);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert!(sees_top(&t.g));
    // During combat only Stomp, the instant Adventure, can be cast.
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, top);
    assert!(!methods.is_empty());
    assert!(!methods.contains(&NORMAL));
    let stomp = methods[0].clone();
    t.cast(P0, top).method(stomp).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // The card goes on an adventure (exile).
    assert!(t.in_exile("Bonecrusher Giant // Stomp") || t.in_exile("Bonecrusher Giant"));
    // The permission ends with the turn.
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Bonecrusher Giant // Stomp");
    belligerent_attacks(&mut t);
    t.advance_to(P1, Step::Upkeep);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 5);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, top).is_empty());
    assert!(!sees_top(&t.g));
}

#[test]
fn the_belligerent_cards_from_the_library_follow_all_costs_and_timing_rules() {
    cr!("601.3", "305.2", "307.1", "601.2f", "611.2a");
    ruling!(
        "The Belligerent",
        "You must pay all costs and follow all timing rules for spells cast and lands played from the top of your library this way."
    );
    supported("The Belligerent");
    // A creature card on top can't be cast during combat, and costs its mana cost in the
    // postcombat main phase.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    belligerent_attacks(&mut t);
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, bears, NORMAL));
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, bears, NORMAL));
    t.cast(P0, bears).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // A land on top is played only in a main phase, as the land for the turn.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    belligerent_attacks(&mut t);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, forest));
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).unwrap();
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
}
