//! Rulings batch P100 — the rest: choices made on resolution rather than as targets (CR
//! 115.1, 608.2d), "until it leaves the battlefield" exiles (610.3, 800.4a), targets
//! checked again on resolution (608.2b), each opponent choosing in turn order (101.4,
//! 608.2e), Two-Headed Giant (810.9), the first main phase (505.1a), revealed hands, and
//! an attack restriction created after attackers were declared.

use crate::r_p100_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn pair_o_dice_lost_chooses_after_rolling_and_exiles_itself_regardless() {
    cr!("115.1", "608.2d", "608.2c");
    ruling!(
        "Pair o' Dice Lost",
        "If Pair o' Dice Lost resolves, it will be exiled, even if you don't return any cards to your hand."
    );
    ruling!(
        "Pair o' Dice Lost",
        "Pair o' Dice Lost doesn't target any of the cards in the graveyard. You choose which cards you're returning to your hand, if any, after rolling the dice."
    );
    supported("Pair o' Dice Lost");
    // Two 1s: X is 2; Shock (1) may be returned, Craw Wurm (6) may not; no target was
    // chosen as it was cast, the choice came after the roll.
    let mut t = TestGame::new(2);
    let shock = t.graveyard(P0, "Shock");
    let wurm = t.graveyard(P0, "Craw Wurm");
    crate::r_s35_common::load_dice(&mut t, &[1, 1]);
    t.answer_choose_moved(P0, &[obj(shock)]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Pair o' Dice Lost", "3GG", &[]);
    assert!(
        asked_of(&t, P0, from).iter().all(|d| !matches!(d, Decision::ChooseTargets { .. })),
        "{:?}",
        asked_of(&t, P0, from)
    );
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(t.zone(wurm), Zone::Graveyard(P0));
    assert!(t.in_exile("Pair o' Dice Lost"));
    // Returning nothing: exiled all the same.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Shock");
    crate::r_s35_common::load_dice(&mut t, &[1, 1]);
    t.answer_choose(P0, &[]);
    cast_resolve(&mut t, P0, "Pair o' Dice Lost", "3GG", &[]);
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(t.in_exile("Pair o' Dice Lost"));
}

#[test]
fn temporary_lockdowns_exile_and_return_are_one_abilitys_two_one_shot_effects() {
    cr!("610.3", "610.3b", "800.4a");
    ruling!(
        "Temporary Lockdown",
        "If Temporary Lockdown leaves the battlefield before its enters-the-battlefield ability resolves, no permanents will be exiled."
    );
    ruling!(
        "Temporary Lockdown",
        "Temporary Lockdown’s ability is a single ability that creates two one-shot effects: one that exiles the permanents when the ability resolves, and another that returns the exiled cards to the battlefield immediately after Temporary Lockdown leaves the battlefield."
    );
    ruling!(
        "Temporary Lockdown",
        "In a multiplayer game, if Temporary Lockdown’s owner leaves the game, the exiled cards will return to the battlefield. Because the one-shot effect that returns the cards isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    supported("Temporary Lockdown");
    // Destroyed with its trigger on the stack: nothing is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lockdown = t.enter(P0, "Temporary Lockdown");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "exile"), 1);
    destroy(&mut t, lockdown);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Resolved: the Bears is exiled, and returns the moment the Lockdown leaves, with
    // nothing on the stack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lockdown = t.enter(P0, "Temporary Lockdown");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    destroy(&mut t, lockdown);
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
    // Three players: P0 (the owner) leaves the game: P1's Bears returns.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Temporary Lockdown");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    crate::r_p069_common::concede(&mut t, P0);
    assert!(t.has_lost(P0));
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn culling_scales_does_nothing_if_its_target_isnt_the_lowest_as_it_resolves() {
    cr!("608.2b");
    ruling!(
        "Culling Scales",
        "If the targeted permanent doesn't have the lowest mana value when the ability resolves, the ability doesn't resolve and the permanent isn't destroyed."
    );
    supported("Culling Scales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Culling Scales");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    assert_eq!(target_candidate_names(&t, P0, from), ["Grizzly Bears"]);
    // A Memnite (0) enters in response: the Bears no longer has the lowest mana value.
    t.battlefield(P1, "Memnite");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn end_of_the_hunt_lets_the_opponent_choose_among_tied_creatures() {
    cr!("608.2d");
    ruling!(
        "End of the Hunt",
        "If the target opponent has multiple creatures and/or planeswalkers tied for the greatest mana value, that player chooses which one to exile."
    );
    supported("End of the Hunt");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(b)]);
    cast_resolve(&mut t, P0, "End of the Hunt", "1B", &[pl(P1)]);
    assert_eq!(entity_candidates(&t, P1, from), vec![vec![obj(a), obj(b)]]);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(t.on_battlefield(a) && t.on_battlefield(bears));
}

#[test]
fn lukkas_minus_two_with_no_greater_creature_card_shuffles_and_moves_on() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Lukka, Coppercoat Outcast",
        "If you reveal your entire library without revealing a creature card with higher mana value, you put the library back in a random order and continue."
    );
    supported("Lukka, Coppercoat Outcast");
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let giant = t.battlefield(P0, "Hill Giant");
    set_library(&mut t, P0, &["Grizzly Bears", "Forest", "Memnite"]);
    t.activate(P0, lukka, 1, &[obj(giant)]).expect("Lukka −2");
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(t.library_size(P0), 3);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.counters(lukka, "loyalty"), 3);
}

#[test]
fn lukkas_ultimate_hits_each_opponent_so_a_two_headed_giant_team_loses_twice() {
    cr!("810.9");
    ruling!(
        "Lukka, Coppercoat Outcast",
        "In a Two-Headed Giant game, Lukka's last ability causes the opposing team to lose twice as much life as the power of your creatures."
    );
    supported("Lukka, Coppercoat Outcast");
    let mut t = two_headed_giant();
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    put(&mut t, lukka, "loyalty", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let team = t.life(P2);
    t.activate(P0, lukka, 2, &[]).expect("Lukka −7");
    t.resolve_all();
    assert_eq!(t.life(P2), team - 4, "2 to each of P2 and P3");
    assert_eq!(t.life(P3), t.life(P2));
}

#[test]
fn polygraph_orb_in_two_headed_giant_hits_both_opponents() {
    cr!("810.9");
    ruling!(
        "Polygraph Orb",
        "In a Two-Headed Giant game, Polygraph Orb's last ability causes both opponents to sacrifice a creature, discard a card, or lose 3 life."
    );
    supported("Polygraph Orb");
    let mut t = two_headed_giant();
    let orb = t.battlefield(P0, "Polygraph Orb");
    let giant = t.graveyard(P0, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, "2");
    t.answer_choose(P0, &[obj(giant)]);
    let team = t.life(P2);
    act(&mut t, P0, orb, "Each opponent loses 3 life", &[]).expect("Orb");
    t.resolve_all();
    assert_eq!(t.life(P2), team - 6, "{}", t.dump_log());
}

#[test]
fn each_opponent_chooses_in_turn_order_then_the_actions_happen_together() {
    cr!("101.4", "608.2e");
    ruling!(
        "Polygraph Orb",
        "In a multiplayer game, each opponent in turn order makes their choice once, then all of the actions occur simultaneously. Opponents will know choices made by earlier opponents when making their choices, although a card chosen to be discarded this way isn't revealed until it's discarded."
    );
    ruling!(
        "Perforating Artist",
        "In a multiplayer game, each opponent in turn order makes their choice once, then all of the actions occur simultaneously. Opponents will know choices made by earlier opponents when making their choices, although a card chosen to be discarded this way isn't revealed until it's discarded."
    );
    supported("Polygraph Orb");
    supported("Perforating Artist");
    // Polygraph Orb: P1 discards Hill Giant, P2 sacrifices Grizzly Bears; P1 was asked
    // before P2.
    let mut t = TestGame::new(3);
    let orb = t.battlefield(P0, "Polygraph Orb");
    let evidence = t.graveyard(P0, "Hill Giant");
    let card = t.hand(P1, "Hill Giant");
    let bears = t.battlefield(P2, "Grizzly Bears");
    mana(&mut t, P0, "2");
    t.answer_choose(P0, &[obj(evidence)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(card)]);
    // (P2 has no card to discard: only the sacrifice is offered.)
    t.answer_yes(P2, true);
    t.answer_choose(P2, &[obj(bears)]);
    let from = t.asked().len();
    act(&mut t, P0, orb, "Each opponent loses 3 life", &[]).expect("Orb");
    t.resolve_all();
    let mut order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != P0 && matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    order.dedup();
    assert_eq!(order, vec![P1, P2], "{:?}", t.asked()[from..].to_vec());
    assert!(t.in_graveyard(P1, "Hill Giant"), "{}", t.dump_log());
    assert!(t.in_graveyard(P2, "Grizzly Bears"), "{:?}", t.asked()[from..].to_vec());
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 20);
    // Perforating Artist's raid trigger at the end step: the same, with P2 taking 3.
    let mut t = TestGame::new(3);
    let artist = t.battlefield(P0, "Perforating Artist");
    let card = t.hand(P1, "Hill Giant");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(card)]);
    t.battlefield(P2, "Grizzly Bears");
    t.answer_yes(P2, false);
    t.attack(&[(artist, pl(P1))], &[]);
    let from = t.asked().len();
    end_step(&mut t, P0);
    t.resolve_all();
    let mut order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p != P0 && matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    order.dedup();
    assert_eq!(order, vec![P1, P2], "{:?}", t.asked()[from..].to_vec());
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17, "the Artist's 3 combat damage only");
    assert_eq!(t.life(P2), 17);
}

#[test]
fn squirming_emergence_counts_the_graveyard_as_it_resolves() {
    cr!("608.2b");
    ruling!(
        "Squirming Emergence",
        "Like all spells that require targets, Squirming Emergence will check to make sure its target is still legal as it tries to resolve. Use the number of permanent cards in your graveyard at that time, even though it may be a different number than the number that were there as you cast the spell. If the permanent card's mana value is greater than the number of permanent cards in your graveyard at that time, Squirming Emergence won't resolve."
    );
    supported("Squirming Emergence");
    // Hill Giant (4) with four permanent cards in the graveyard: a legal target. A Mind
    // Stone card is exiled from the graveyard in response: three, so the Giant stays.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Memnite");
    let stone = t.graveyard(P0, "Mind Stone");
    t.graveyard(P0, "Shock");
    let spell = cast(&mut t, P0, "Squirming Emergence", "1BG", &[obj(giant)]);
    exile(&mut t, stone);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    // Undisturbed, it returns.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Memnite");
    t.graveyard(P0, "Mind Stone");
    cast_resolve(&mut t, P0, "Squirming Emergence", "1BG", &[obj(giant)]);
    assert!(t.on_battlefield(giant));
}

#[test]
fn elemental_resonance_triggers_in_the_first_main_phase_only() {
    cr!("505.1a", "505.1b", "500.8");
    ruling!(
        "Elemental Resonance",
        "Only the first main phase in a turn is the \"precombat main phase,\" even if additional main phases are generated by some effect."
    );
    supported("Elemental Resonance");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Elemental Resonance");
    assert!(t.g.attach(aura, obj(bears)));
    t.settle();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "add mana"), 1, "{:?}", stack_items(&t));
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2, "{{1}}{{G}}: {{C}}{{G}}");
    // Relentless Assault: an additional combat phase and main phase follow this one.
    t.g.players[P0.idx()].mana_pool = Default::default();
    cast_resolve(&mut t, P0, "Relentless Assault", "2RR", &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.g.turn.step, Step::PostcombatMain);
    assert_eq!(triggers_on_stack(&t, "add mana"), 0, "{:?}", stack_items(&t));
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn tariff_has_the_player_choose_among_tied_creatures_as_it_resolves() {
    cr!("608.2d", "608.2e");
    ruling!("Tariff", "The choice of creatures is made on resolution and not on announcement.");
    supported("Tariff");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(b)]);
    let from = t.asked().len();
    cast(&mut t, P0, "Tariff", "1W", &[]);
    assert!(
        entity_candidates(&t, P1, from).is_empty(),
        "no choice as it's cast: {:?}",
        asked_of(&t, P1, from)
    );
    t.resolve_all();
    assert_eq!(entity_candidates(&t, P1, from), vec![vec![obj(a), obj(b)]]);
    assert!(t.in_graveyard(P1, "Hill Giant"), "neither player could pay");
    assert!(t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let _ = bears;
}

#[test]
fn black_suns_twilight_chooses_the_card_to_return_as_it_resolves() {
    cr!("115.1", "608.2d");
    ruling!(
        "Black Sun's Twilight",
        "The creature card you return, if any, isn't a target of Black Sun's Twilight. You choose it as Black Sun's Twilight resolves if X is 5 or more."
    );
    supported("Black Sun's Twilight");
    // X = 5 at a Hill Giant: it dies; a Grizzly Bears card (2) in the graveyard is chosen
    // as the spell resolves (never a target) and returns tapped.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    let spell = cast_x(&mut t, P0, "Black Sun's Twilight", 5, &[obj(giant)]);
    assert_eq!(targets_of(&t, spell), vec![obj(giant)]);
    assert!(entity_candidates(&t, P0, from).is_empty(), "{:?}", asked_of(&t, P0, from));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(bears) && t.obj_now(bears).tapped);
    // X = 4: no card is returned.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_x_resolve(&mut t, P0, "Black Sun's Twilight", 4, &[obj(giant)]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(!was_offered(&t, P0, from, bears));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}

#[test]
fn atemsis_can_reveal_a_hand_that_is_already_revealed() {
    cr!("701.20a", "701.20b");
    ruling!(
        "Atemsis, All-Seeing",
        "If an effect causes you to play with your hand revealed, you can still reveal your hand as Atemsis's last ability resolves. The ability works as normal."
    );
    supported("Atemsis, All-Seeing");
    supported("Telepathy");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Telepathy");
    let atemsis = t.battlefield(P0, "Atemsis, All-Seeing");
    for c in ["Forest", "Shock", "Grizzly Bears", "Hill Giant", "Air Elemental", "Craw Wurm"] {
        t.hand(P0, c);
    }
    t.settle();
    assert!(mtg_engine::reveal::hand_revealed(&t.g, P0), "Telepathy: P0 plays with their hand revealed");
    t.answer_yes(P0, true);
    crate::r_p151_common::damage(&mut t, atemsis, pl(P1), 4);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn droning_bureaucrats_after_attackers_are_declared_restricts_only_blockers() {
    cr!("508.1", "509.1a", "611.2c");
    ruling!(
        "Droning Bureaucrats",
        "If you use Droning Bureaucrats's ability after you have declared attackers but before your opponent declares blockers, the restriction will apply to only your opponent's creatures. Creatures that are already attacking will be unaffected."
    );
    supported("Droning Bureaucrats");
    let mut t = TestGame::new(2);
    let bureaucrats = t.battlefield(P0, "Droning Bureaucrats");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(mine, pl(P1))]);
    mana(&mut t, P0, "2");
    answer_x(&mut t, P0, 2);
    act(&mut t, P0, bureaucrats, "can't attack or block", &[]).expect("Bureaucrats");
    t.resolve_all();
    assert!(crate::r_s09_common::attacks_now(&t).iter().any(|(a, _)| *a == mine), "still attacking");
    assert!(!t.g.can_block(theirs, mine));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}
