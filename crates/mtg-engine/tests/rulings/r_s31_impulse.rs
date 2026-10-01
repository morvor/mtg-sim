//! Rulings batch S31 — impulse draw: cards exiled face up that "you may play" for a while.
//! They're played with the normal timing rules and costs (CR 305.1, 305.2, 307.1, 601.2),
//! only from exile (a card played this way is a new object, CR 400.7), and a card that
//! isn't played stays in exile once the permission ends (CR 611.2a).

use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s29_common::choose_modes;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether P0 could play the card `id` (followed across zone changes) now: play it as a
/// land or cast it.
fn playable(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(P0).iter().any(|a| match a {
        Action::PlayLand { card } => *card == c,
        Action::Cast { card, .. } => *card == c,
        _ => false,
    })
}

/// Checks, from P0's precombat main phase with the stack empty and no land played this
/// turn, that P0 may play the exiled land card `land` only in a main phase with the stack
/// empty and only while they have a land play available (CR 305.1, 305.2, 116.2a).
fn check_land_timing(t: &mut TestGame, land: ObjectId) {
    assert_eq!(t.zone(land), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    assert!(playable(t, land));
    // Not while a spell is on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!playable(t, land));
    t.resolve_all();
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable(t, land));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(playable(t, land));
    // Not once P0 has played a land this turn.
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("play a land from hand");
    assert!(!playable(t, land));
    assert_eq!(t.zone(land), Zone::Exile);
}

/// Checks that P0 has to pay the mana cost of the exiled nonland card `card` ({1}{G},
/// Grizzly Bears) to cast it: not without mana, then with it.
fn check_pays_costs(t: &mut TestGame, card: ObjectId) {
    assert_eq!(t.zone(card), Zone::Exile);
    t.g.players[P0.idx()].mana_pool = Default::default();
    for l in t.g.battlefield.clone() {
        if t.g.obj(l).controller == P0 {
            t.g.objects[l.0 as usize].tapped = true;
        }
    }
    assert!(!playable(t, card));
    add_mana(t, P0, ManaType::G, 2);
    assert!(playable(t, card));
    let c = t.g.current(card);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn cori_mountain_monastery_follows_land_timing_rules() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Cori Mountain Monastery",
        "You pay all costs and follow all timing rules for cards played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Cori Mountain Monastery");
    // "{3}{R}, {T}: Exile the top card of your library. Until the end of your next turn,
    // you may play that card."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let monastery = t.battlefield(P0, "Cori Mountain Monastery");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, monastery, "Exile the top").expect("activate");
    t.resolve_all();
    check_land_timing(&mut t, forest);
}

#[test]
fn blazing_crescendo_follows_land_timing_rules() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Blazing Crescendo",
        "You pay all costs and follow all normal timing rules for a card played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Blazing Crescendo");
    // "Target creature gets +3/+1 until end of turn. Exile the top card of your library.
    // Until the end of your next turn, you may play that card."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::R, 2);
    let spell = t.hand(P0, "Blazing Crescendo");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 3));
    check_land_timing(&mut t, forest);
}

#[test]
fn monastery_raid_exiled_cards_follow_timing_rules_and_cost_mana() {
    cr!("305.1", "305.2", "601.2f", "601.2g");
    ruling!(
        "Monastery Raid",
        "You pay all costs and follow all normal timing rules for cards played this way. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Monastery Raid");
    // "Exile the top two cards of your library. ... You may play the exiled cards until
    // the end of your next turn."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    add_mana(&mut t, P0, ManaType::R, 3);
    let spell = t.hand(P0, "Monastery Raid");
    t.cast(P0, spell).go();
    t.resolve_all();
    check_land_timing(&mut t, cards[0]);
    check_pays_costs(&mut t, cards[1]);
}

#[test]
fn seize_opportunity_exiled_cards_follow_timing_rules() {
    cr!("305.1", "305.2", "601.2f");
    ruling!(
        "Seize Opportunity",
        "You pay all costs and follow all timing rules for cards played this way. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Seize Opportunity");
    // "Choose one — • Exile the top two cards of your library. Until the end of your next
    // turn, you may play those cards. • ..."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    add_mana(&mut t, P0, ManaType::R, 3);
    let spell = t.hand(P0, "Seize Opportunity");
    t.cast(P0, spell).modes(&[0]).go();
    t.resolve_all();
    check_land_timing(&mut t, cards[0]);
    check_pays_costs(&mut t, cards[1]);
}

#[test]
fn riverwheel_sweeps_chosen_land_follows_timing_rules() {
    cr!("305.1", "305.2");
    ruling!(
        "Riverwheel Sweep",
        "You pay all costs and follow all timing rules for cards played this way. For example, if the chosen exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Riverwheel Sweep");
    // "Tap target creature. Put three stun counters on it. Exile the top two cards of your
    // library. Choose one of them. Until the end of your next turn, you may play that
    // card."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    let target = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    let spell = t.hand(P0, "Riverwheel Sweep");
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    assert_eq!(t.counters(target, "stun"), 3);
    // Only the chosen card may be played.
    assert!(!playable(&mut t, cards[1]));
    check_land_timing(&mut t, cards[0]);
}

#[test]
fn fateful_tempests_exiled_land_needs_a_land_play() {
    cr!("305.1", "305.2", "701.38a");
    ruling!(
        "Fateful Tempest",
        "You pay all costs and follow all timing rules for cards played this way. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    supported("Fateful Tempest");
    // "Council's dilemma — Starting with you, each player votes for past or present. ...
    // Exile the top card of your library for each present vote. Until the end of your
    // next turn, you may play the exiled cards."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    add_mana(&mut t, P0, ManaType::R, 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let spell = t.hand(P0, "Fateful Tempest");
    t.cast(P0, spell).go();
    t.resolve_all();
    check_land_timing(&mut t, cards[0]);
    check_pays_costs(&mut t, cards[1]);
}

#[test]
fn grotag_night_runners_card_follows_timing_rules_and_stays_exiled() {
    cr!("305.2", "611.2a");
    ruling!(
        "Grotag Night-Runner",
        "You must follow the normal timing permissions and restrictions for the exiled card. If it’s a land, you can’t play it unless you have land plays available."
    );
    ruling!(
        "Grotag Night-Runner",
        "If you don’t play the exiled card, it remains exiled."
    );
    supported("Grotag Night-Runner");
    // "Whenever this creature deals combat damage to a player, exile the top card of your
    // library. You may play that card this turn."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let grotag = t.battlefield(P0, "Grotag Night-Runner");
    let played = t.hand(P0, "Mountain");
    t.play_land(P0, played).expect("play a land");
    attack_with(&mut t, &[(grotag, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(forest), Zone::Exile);
    t.advance_to(P0, Step::PostcombatMain);
    // P0 already played a land this turn: the Forest can't be played.
    assert!(!playable(&mut t, forest));
    // It stays in exile after the turn, and can't be played any more.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!playable(&mut t, forest));
}

#[test]
fn elkin_bottles_card_is_played_with_its_normal_timing_and_costs() {
    cr!("307.1", "601.2f", "305.2", "611.2a");
    ruling!(
        "Elkin Bottle",
        "The exiled card is played using the normal timing rules for its card type, as well as any other applicable restrictions such as “Cast [this card] only during combat.” For example, you can’t play the card during an opponent’s turn unless it’s an instant or has flash."
    );
    supported("Elkin Bottle");
    // "{3}, {T}: Exile the top card of your library. Until the beginning of your next
    // upkeep, you may play that card."
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let bottle = t.battlefield(P0, "Elkin Bottle");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, bottle, "Exile the top").expect("activate");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // P0 has to pay its mana cost.
    assert!(!playable(&mut t, bears));
    // During the opponent's turn a creature card can't be cast, even with the mana.
    t.advance_to(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(!playable(&mut t, bears));
    // An instant exiled with it can be cast then.
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let bottle = t.battlefield(P0, "Elkin Bottle");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, bottle, "Exile the top").expect("activate");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(playable(&mut t, bolt));
}

#[test]
fn a_card_played_from_exile_leaves_exile_and_cant_be_played_again() {
    cr!("400.7", "611.2a", "305.1");
    ruling!(
        "Tectonic Giant",
        "Playing an exiled card causes it to leave exile. You can't play it multiple times."
    );
    supported("Tectonic Giant");
    // "Whenever this creature attacks or becomes the target of a spell an opponent
    // controls, choose one — • This creature deals 3 damage to each opponent. • Exile the
    // top two cards of your library. Choose one of them. Until the end of your next turn,
    // you may play that card."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    let giant = t.battlefield(P0, "Tectonic Giant");
    choose_modes(&mut t, P0, &[1]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    let forest = t.g.current(cards[0]);
    t.play_land(P0, forest).expect("play the exiled land");
    assert_eq!(t.zone(cards[0]), Zone::Battlefield);
    // Returned to exile, it's a new object: it can't be played again, even next turn.
    let on_bf = t.g.current(cards[0]);
    t.g.move_object(
        on_bf,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.settle();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!playable(&mut t, cards[0]));
}

#[test]
fn act_on_impulse_exiles_the_cards_face_up_and_unplayed_ones_stay_exiled() {
    cr!("406.3", "611.2a");
    ruling!("Act on Impulse", "The cards are exiled face up.");
    ruling!(
        "Act on Impulse",
        "Any cards you don't play will remain exiled."
    );
    supported("Act on Impulse");
    // "Exile the top three cards of your library. Until end of turn, you may play those
    // cards."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
    add_mana(&mut t, P0, ManaType::R, 3);
    let spell = t.hand(P0, "Act on Impulse");
    t.cast(P0, spell).go();
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(!t.obj_now(*c).face_down);
    }
    assert!(playable(&mut t, cards[0]));
    // P0 plays none of them: they stay in exile, no longer playable.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    assert!(!playable(&mut t, cards[0]));
}

#[test]
fn commune_with_lavas_unplayed_cards_stay_exiled() {
    cr!("611.2a", "107.3a");
    ruling!(
        "Commune with Lava",
        "Any cards you don't play will remain exiled."
    );
    supported("Commune with Lava");
    // "Exile the top X cards of your library. Until the end of your next turn, you may
    // play those cards."
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    add_mana(&mut t, P0, ManaType::R, 4);
    let spell = t.hand(P0, "Commune with Lava");
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    // Still playable on P0's next turn, then no more; they stay in exile.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(playable(&mut t, cards[0]));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    assert!(!playable(&mut t, cards[0]));
}

/// Checks that `card`, exiled by an effect that let P0 play it this turn, stays in exile
/// once the turn is over, and can't be played any more.
fn stays_exiled_after_this_turn(t: &mut TestGame, card: ObjectId) {
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(playable(t, card));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(!playable(t, card));
}

#[test]
fn abbot_of_keral_keeps_card_stays_exiled_if_not_played() {
    cr!("611.2a");
    ruling!(
        "Abbot of Keral Keep",
        "If you don't play the card, it will remain exiled."
    );
    supported("Abbot of Keral Keep");
    // "When this creature enters, exile the top card of your library. Until end of turn,
    // you may play that card."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.enter(P0, "Abbot of Keral Keep");
    t.resolve_all();
    stays_exiled_after_this_turn(&mut t, forest);
}

#[test]
fn dark_dweller_oracles_card_stays_exiled_if_not_cast() {
    cr!("611.2a");
    ruling!(
        "Dark-Dweller Oracle",
        "If you don't cast the exiled card, it remains in exile."
    );
    supported("Dark-Dweller Oracle");
    // "{1}, Sacrifice a creature: Exile the top card of your library. You may play that
    // card this turn."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let oracle = t.battlefield(P0, "Dark-Dweller Oracle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, oracle, "Exile the top").expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    stays_exiled_after_this_turn(&mut t, forest);
}

#[test]
fn hedron_detonators_card_stays_exiled_if_not_played() {
    cr!("611.2a");
    ruling!(
        "Hedron Detonator",
        "If you don’t play the card, it will remain exiled."
    );
    supported("Hedron Detonator");
    // "{T}, Sacrifice two artifacts: Exile the top card of your library. You may play that
    // card this turn."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let detonator = t.battlefield(P0, "Hedron Detonator");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    activate_containing(&mut t, P0, detonator, "Exile the top").expect("activate");
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    stays_exiled_after_this_turn(&mut t, forest);
}

#[test]
fn haste_magics_card_follows_timing_rules_until_your_next_end_step() {
    cr!("305.1", "305.2", "500.4", "601.2f");
    ruling!(
        "Haste Magic",
        "You pay all costs and follow all timing rules for cards played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    supported("Haste Magic");
    // "Target creature gets +3/+1 and gains haste until end of turn. Exile the top card of
    // your library. You may play it until your next end step."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::R, 2);
    let spell = t.hand(P0, "Haste Magic");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 3));
    check_land_timing(&mut t, forest);
    // A nonland card costs its mana cost, and can be played only until P0's end step
    // begins.
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, "Grizzly Bears");
    let bears = t.battlefield(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 2);
    let spell = t.hand(P0, "Haste Magic");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    check_pays_costs(&mut t, card);
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let giant = t.battlefield(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 2);
    let spell = t.hand(P0, "Haste Magic");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(playable(&mut t, bolt));
    t.advance_to(P0, Step::End);
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(!playable(&mut t, bolt));
    assert_eq!(t.zone(bolt), Zone::Exile);
}

#[test]
fn snowslope_hunters_card_follows_timing_rules_and_costs() {
    cr!("305.1", "305.2", "601.2f", "602.5b");
    ruling!(
        "Snowslope Hunter",
        "You pay all costs and follow all timing rules for cards played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    supported("Snowslope Hunter");
    // "Sacrifice another creature or artifact: Exile the top card of your library. You may
    // play it until the end of your next turn. Activate only during your turn and only
    // once each turn."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let hunter = t.battlefield(P0, "Snowslope Hunter");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(a)]);
    activate_containing(&mut t, P0, hunter, "Exile the top").expect("activate");
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    // Only once each turn.
    t.answer_choose(P0, &[Entity::Object(b)]);
    assert!(activate_containing(&mut t, P0, hunter, "Exile the top").is_err());
    t.clear_answers();
    check_land_timing(&mut t, forest);
    // On P0's next turn, a nonland card exiled with it costs its mana cost.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(b)]);
    activate_containing(&mut t, P0, hunter, "Exile the top").expect("activate");
    t.resolve_all();
    check_pays_costs(&mut t, bears);
}

#[test]
fn valakut_explorations_card_follows_the_normal_timing_and_land_play_rules() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Valakut Exploration",
        "You must follow the normal timing permissions and restrictions for the exiled card. If it's a land, you can't play it unless you have land plays available."
    );
    supported("Valakut Exploration");
    // "Landfall — Whenever a land you control enters, exile the top card of your library.
    // You may play that card for as long as it remains exiled."
    // P0 plays a Mountain: the Forest exiled can't be played, as P0 has no land play left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut Exploration");
    let forest = t.library_top(P0, "Forest");
    let mountain = t.hand(P0, "Mountain");
    t.play_land(P0, mountain).expect("play a land");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!playable(&mut t, forest));
    // A land put onto the battlefield by an effect doesn't use the land play: the exiled
    // land may be played, with the normal timing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut Exploration");
    let forest = t.library_top(P0, "Forest");
    t.enter(P0, "Plains");
    t.resolve_all();
    check_land_timing(&mut t, forest);
}

#[test]
fn valakut_explorations_card_played_and_exiled_again_cant_be_played() {
    cr!("400.7", "611.2a", "607.2a", "603.4");
    ruling!(
        "Valakut Exploration",
        "If you play a card this way, it leaves exile and becomes a new object. If it returns to exile later in the turn, you can't play it again."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut Exploration");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.enter(P0, "Plains");
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::G, 2);
    let c = t.g.current(bears);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Back in exile, it's a new object: not playable, and not exiled with Valakut
    // Exploration (whose end step ability then does nothing).
    let on_bf = t.g.current(bears);
    t.g.exile_object(on_bf, None);
    t.settle();
    assert_eq!(t.zone(bears), Zone::Exile);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(!playable(&mut t, bears));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn valakut_exploration_puts_the_unplayed_cards_into_the_graveyard_at_end_step() {
    cr!("607.2a", "603.4", "120.3a");
    supported("Valakut Exploration");
    // "At the beginning of your end step, if there are cards exiled with this enchantment,
    // put them into their owner's graveyard, then this enchantment deals that much damage
    // to each opponent." Two landfalls exile two cards; P0 plays neither.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Valakut Exploration");
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    t.enter(P0, "Plains");
    t.resolve_all();
    t.enter(P0, "Plains");
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Graveyard(P0));
    }
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn vances_blasting_cannons_creature_card_waits_for_the_main_phase() {
    cr!("307.1", "601.2f", "305.9");
    ruling!(
        "Vance's Blasting Cannons // Spitfire Bastion",
        "Casting the exiled card follows the normal rules for casting that card. You must pay its costs, and you must follow all applicable timing rules. For example, if you exile a creature card this way, you must wait until your main phase to cast it."
    );
    supported("Vance's Blasting Cannons // Spitfire Bastion");
    // "At the beginning of your upkeep, exile the top card of your library. If it's a
    // nonland card, you may cast that card this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vance's Blasting Cannons // Spitfire Bastion");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // Not in the upkeep, even with the mana.
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(!playable(&mut t, bears));
    // In P0's main phase: by paying its mana cost.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!playable(&mut t, bears));
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(playable(&mut t, bears));
    let c = t.g.current(bears);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // A land card exiled this way can't be played.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vance's Blasting Cannons // Spitfire Bastion");
    let forest = t.library_top(P0, "Forest");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!playable(&mut t, forest));
}

#[test]
fn practiced_scrollsmiths_sorcery_is_cast_with_sorcery_timing_and_its_cost() {
    cr!("307.1", "601.2f", "117.1a");
    ruling!(
        "Practiced Scrollsmith",
        "You pay all costs and follow all timing rules for spells cast this way. For example, if the exiled card is a sorcery, you may cast it only during your main phase while the stack is empty."
    );
    supported("Practiced Scrollsmith");
    // "When this creature enters, exile target noncreature, nonland card from your
    // graveyard. Until the end of your next turn, you may cast that card." Divination
    // {2}{U}: "Draw two cards."
    let mut t = TestGame::new(2);
    let divination = t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[Entity::Object(divination)]);
    t.enter(P0, "Practiced Scrollsmith");
    t.resolve_all();
    assert_eq!(t.zone(divination), Zone::Exile);
    // Its mana cost has to be paid.
    assert!(!playable(&mut t, divination));
    let with_mana = |t: &mut TestGame| {
        add_mana(t, P0, ManaType::U, 1);
        add_mana(t, P0, ManaType::C, 2);
    };
    with_mana(&mut t);
    assert!(playable(&mut t, divination));
    // Not while a spell is on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!playable(&mut t, divination));
    t.resolve_all();
    // Not during combat, nor in the opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    with_mana(&mut t);
    assert!(!playable(&mut t, divination));
    t.g.combat = None;
    t.set_step(P1, Step::PrecombatMain);
    with_mana(&mut t);
    assert!(!playable(&mut t, divination));
    // In P0's next main phase, it's cast.
    t.advance_to(P0, Step::PrecombatMain);
    with_mana(&mut t);
    let hand = t.hand_size(P0);
    let c = t.g.current(divination);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn syr_carahs_card_stays_exiled_if_not_played() {
    cr!("611.2a", "603.2", "120.3");
    ruling!(
        "Syr Carah, the Bold",
        "If you don't play the exiled card, it remains in exile."
    );
    supported("Syr Carah, the Bold");
    // "Whenever Syr Carah or an instant or sorcery spell you control deals damage to a
    // player, exile the top card of your library. You may play that card this turn.
    // {T}: Syr Carah deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let carah = t.battlefield(P0, "Syr Carah, the Bold");
    t.activate(P0, carah, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    stays_exiled_after_this_turn(&mut t, forest);
    // An instant P0 controls dealing damage to a player triggers it too; damage to a
    // creature doesn't.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    t.battlefield(P0, "Syr Carah, the Bold");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 2);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert!(matches!(t.zone(cards[0]), Zone::Library(_)));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    stays_exiled_after_this_turn(&mut t, cards[0]);
}
