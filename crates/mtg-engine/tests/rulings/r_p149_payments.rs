//! Rulings batch P149 — "unless a player pays" (rhystic) and other payments: Rhystic Cave
//! (a mana ability, CR 605), Rhystic Shield, Rhystic Deluge, Plague Fiend, Carrionette,
//! Reservoir Kraken; attack and block taxes against requirements (Archangel of Tithes,
//! Summon: Yojimbo, War Cadence); and the choices of Sphinx of Clear Skies, Explore the
//! Vastlands and Pippin.

use crate::r_p149_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

// --- Rhystic Cave ------------------------------------------------------------------------------

/// `p` activates Rhystic Cave choosing red; every player who's asked whether to pay {1}
/// answers `pays` for their player.
fn tap_cave(t: &mut TestGame, p: PlayerId, cave: ObjectId) {
    crate::r_p125_common::choose_color(t, p, Color::Red);
    t.activate(p, cave, 0, &[]).expect("Rhystic Cave");
    t.settle();
}

#[test]
fn rhystic_cave_each_player_in_turn_order_may_pay_as_it_resolves() {
    cr!("605.3b", "101.4", "118.12");
    ruling!(
        "Rhystic Cave",
        "Each player (starting with the current player and going in turn order) gets the option to pay when this ability resolves."
    );
    supported("Rhystic Cave");
    // P1's turn; P0 taps the Cave while P1 is the current player.
    let mut t = TestGame::new(3);
    let cave = t.battlefield(P0, "Rhystic Cave");
    t.set_step(P1, Step::PrecombatMain);
    for p in [P0, P1, P2] {
        t.lands(p, "Wastes", 1);
        t.answer_yes(p, false);
    }
    let from = t.asked().len();
    tap_cave(&mut t, P0, cave);
    assert_eq!(asked_who(&t, from, is_yes_no), vec![P1, P2, P0]);
    assert_eq!(t.g.players[0].mana_pool.mana.len(), 1);
    assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
}

#[test]
fn rhystic_cave_paid_for_produces_no_mana_and_isnt_tapped_for_mana() {
    cr!("106.12", "106.12a", "605.1a");
    ruling!(
        "Rhystic Cave",
        "If a player pays {1}, Rhystic Cave does not produce any mana, and is not considered to have been tapped for mana. Things which trigger of the land being tapped for mana will not trigger."
    );
    supported("Manabarbs");
    // Manabarbs: "Whenever a player taps a land for mana, Manabarbs deals 1 damage to that
    // player."
    for pays in [true, false] {
        let mut t = TestGame::new(2);
        let cave = t.battlefield(P0, "Rhystic Cave");
        t.battlefield(P1, "Manabarbs");
        t.lands(P1, "Wastes", 1);
        t.answer_yes(P1, pays);
        tap_cave(&mut t, P0, cave);
        assert!(t.obj_now(cave).tapped);
        t.resolve_all();
        let pool = t.g.players[0].mana_pool.mana.len();
        assert_eq!(pool, usize::from(!pays), "pays: {pays}");
        assert_eq!(t.life(P0), if pays { 20 } else { 19 }, "pays: {pays}");
    }
}

#[test]
fn rhystic_cave_cant_be_activated_while_casting_a_spell() {
    cr!("602.5e", "605.3a", "601.2g");
    ruling!(
        "Rhystic Cave",
        "The card’s ability has errata so you can’t activate the ability during casting of a spell or activating of an ability. This prevents you from getting into a position where someone paying {1} could stop you from having enough mana to pay for the spell. If you want to use it pay for a spell or ability, you need to use this card before you start the casting or activating."
    );
    // With only the Cave for mana, Shock can't be paid for while it's being cast.
    let mut t = TestGame::new(2);
    let cave = t.battlefield(P0, "Rhystic Cave");
    let shock = t.hand(P0, "Shock");
    t.answer_targets(P0, &[pl(P1)]);
    assert!(t.cast(P0, shock).try_go().is_err());
    assert!(!t.obj_now(cave).tapped);
    // Tapped first, with priority: the red mana pays for it.
    tap_cave(&mut t, P0, cave);
    assert_eq!(t.g.players[0].mana_pool.mana.len(), 1);
    t.answer_targets(P0, &[pl(P1)]);
    let shock = t.g.current(shock);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- Other "unless ... pays" ------------------------------------------------------------------

#[test]
fn rhystic_shield_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Rhystic Shield",
        "Only affects creatures you control when it resolves."
    );
    supported("Rhystic Shield");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    cast_at(&mut t, P0, "Rhystic Shield", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 5));
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn rhystic_shield_paid_by_one_player_is_still_offered_to_the_next() {
    cr!("118.12a", "101.4");
    interp!("unless-any-player-pays-offers-everyone");
    // Rhystic Shield: "Creatures you control get +0/+1 until end of turn. They get an
    // additional +0/+2 until end of turn unless any player pays {2}."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for p in [P1, P2] {
        t.lands(p, "Wastes", 2);
    }
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    cast_at(&mut t, P0, "Rhystic Shield", &[]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(asked_who(&t, from, is_yes_no), vec![P0, P1, P2]);
    assert_eq!(t.pt(bears), (2, 3));
}

#[test]
fn rhystic_deluge_and_plague_fiend_the_creatures_controller_may_pay_on_resolution() {
    cr!("608.2d", "118.12");
    ruling!(
        "Rhystic Deluge",
        "The creature’s controller gets the option to pay when this ability resolves."
    );
    ruling!(
        "Plague Fiend",
        "The creature’s controller gets the option to pay when the triggered ability resolves."
    );
    supported("Rhystic Deluge");
    supported("Plague Fiend");
    // Rhystic Deluge ("{U}: Tap target creature unless its controller pays {1}.").
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let deluge = t.battlefield(P0, "Rhystic Deluge");
        let bears = t.battlefield(P1, "Grizzly Bears");
        lots(&mut t, P1);
        t.answer_yes(P1, pays);
        act_only(&mut t, P0, deluge, 0, &[obj(bears)]);
        let from = t.asked().len();
        assert!(
            asked_who(&t, 0, is_yes_no).is_empty(),
            "not while activating"
        );
        t.resolve_all();
        assert_eq!(asked_who(&t, from, is_yes_no), vec![P1]);
        assert_eq!(t.obj_now(bears).tapped, !pays, "pays: {pays}");
    }
    // Plague Fiend ("Whenever this creature deals combat damage to a creature, destroy
    // that creature unless its controller pays {2}.").
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let fiend = t.battlefield(P0, "Plague Fiend");
        let giant = t.battlefield(P1, "Hill Giant");
        lots(&mut t, P0);
        lots(&mut t, P1);
        t.answer_yes(P1, pays);
        let from = t.asked().len();
        t.g.deal_damage(fiend, obj(giant), 1, true);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        assert!(asked_who(&t, from, is_yes_no).is_empty());
        t.resolve_all();
        assert_eq!(asked_who(&t, from, is_yes_no), vec![P1]);
        assert_eq!(t.on_battlefield(giant), pays, "pays: {pays}");
    }
}

#[test]
fn carrionette_cant_target_protection_from_creatures() {
    cr!("702.16b", "115.4");
    ruling!(
        "Carrionette",
        "A creature with Protection from Creatures can't be targeted by this card's ability."
    );
    supported("Carrionette");
    supported("Beloved Chaplain");
    let mut t = TestGame::new(2);
    let carrionette = t.graveyard(P0, "Carrionette");
    let chaplain = t.battlefield(P1, "Beloved Chaplain");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cands = crate::r_s04_common::ability_targets(&mut t, carrionette, 0);
    assert!(cands.contains(&obj(bears)));
    assert!(!cands.contains(&obj(chaplain)));
}

#[test]
fn carrionette_can_be_activated_more_than_once() {
    cr!("602.2", "608.2b");
    ruling!(
        "Carrionette",
        "Since it is exiled as part of the effect and not part of the cost, you can activate its ability more than once."
    );
    let mut t = TestGame::new(2);
    let carrionette = t.graveyard(P0, "Carrionette");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    act_only(&mut t, P0, carrionette, 0, &[obj(bears)]);
    act_only(&mut t, P0, carrionette, 0, &[obj(giant)]);
    assert_eq!(t.stack_len(), 2);
    // P1 can't pay: both creatures are exiled (Carrionette with the first to resolve).
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert!(t.in_exile("Carrionette"));
}

// --- Reservoir Kraken --------------------------------------------------------------------------

#[test]
fn reservoir_kraken_each_opponent_may_tap_but_only_one_fish() {
    cr!("101.4", "603.4");
    ruling!(
        "Reservoir Kraken",
        "Each opponent in turn order may choose to tap an untapped creature as Reservoir Kraken's triggered ability resolves, even if a player earlier in the turn order already chose to tap a creature. Reservoir Kraken's controller will create a maximum of one Fish creature token each combat, no matter how many creatures were tapped."
    );
    interp!("any-player-may-after-a-payment");
    supported("Reservoir Kraken");
    let mut t = TestGame::new(3);
    let kraken = t.battlefield(P0, "Reservoir Kraken");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(asked_who(&t, from, is_yes_no), vec![P1, P2]);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert!(t.obj_now(kraken).tapped);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Fish").len(), 1);
}

#[test]
fn reservoir_kraken_opponents_decide_before_attacks_are_declared() {
    cr!("507.1", "508.1");
    ruling!(
        "Reservoir Kraken",
        "Players won't know which player or planeswalker Reservoir Kraken will attack, if any, when deciding whether to tap an untapped creature for its triggered ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reservoir Kraken");
    t.battlefield(P1, "Grizzly Bears");
    let seen = crate::r_s01_common::watch(&mut t, P1, is_yes_no, |g| {
        (
            g.turn.step,
            g.combat.as_ref().map_or(0, |c| c.attackers.len()),
        )
    });
    t.answer_yes(P1, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![(Step::BeginningOfCombat, 0)]);
}

// --- Attack and block taxes against requirements ---------------------------------------------

#[test]
fn archangel_of_tithes_creatures_that_must_attack_or_block_need_not_pay() {
    cr!("508.1d", "509.1c", "508.1h");
    ruling!(
        "Archangel of Tithes",
        "If you control an untapped Archangel of Tithes, your opponents can choose to not attack with a creature that must attack if able. The same is true with respect to an attacking Archangel of Tithes and a creature that must block if able."
    );
    supported("Archangel of Tithes");
    supported("Juggernaut");
    supported("Watchdog");
    // Juggernaut ("attacks each combat if able") under an untapped Archangel: it may stay
    // home; without the Archangel, it may not.
    for archangel in [true, false] {
        let mut t = TestGame::new(2);
        if archangel {
            t.battlefield(P0, "Archangel of Tithes");
        }
        let jugg = t.battlefield(P1, "Juggernaut");
        t.set_step(P1, Step::BeginningOfCombat);
        let legal = crate::r_s09_common::legal_attack(&mut t, &[]);
        assert_eq!(legal, archangel, "archangel: {archangel}");
        assert!(crate::r_s09_common::legal_attack(&mut t, &[(jugg, pl(P0))]));
    }
    // Watchdog ("blocks each combat if able") against an attacking Archangel: it may not
    // block; against an attacking Hill Giant, it must.
    for attacker in ["Archangel of Tithes", "Hill Giant"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, attacker);
        t.battlefield(P1, "Watchdog");
        attack_with(&mut t, &[(a, pl(P1))]);
        let legal = crate::r_s21_common::legal_blocks(&mut t, P1, &[]);
        assert_eq!(legal, attacker == "Archangel of Tithes", "{attacker}");
    }
}

#[test]
fn summon_yojimbo_must_attack_creatures_attack_a_player_without_a_cost_if_there_is_one() {
    cr!("508.1d", "508.1h", "714.2b");
    ruling!(
        "Summon: Yojimbo",
        "On turns where the effect of Summon: Yojimbo's second or third chapter ability is applying, your opponents can choose not to attack with a creature that must attack if able as long as there is no other player, planeswalker, or battle for that creature to attack that wouldn't require a cost."
    );
    supported("Summon: Yojimbo");
    for players in [2, 3] {
        let mut t = TestGame::new(players);
        let yojimbo = t.battlefield(P0, "Summon: Yojimbo");
        put(&mut t, yojimbo, counters::LORE, 2);
        t.resolve_all();
        let jugg = t.battlefield(P1, "Juggernaut");
        t.set_step(P1, Step::BeginningOfCombat);
        // Attacking P0 costs {2}; with a third player to attack for free, Juggernaut must
        // attack (that player).
        let stay_home = crate::r_s09_common::legal_attack(&mut t, &[]);
        assert_eq!(stay_home, players == 2, "{players} players");
        if players == 3 {
            assert!(crate::r_s09_common::legal_attack(&mut t, &[(jugg, pl(P2))]));
        }
    }
}

#[test]
fn war_cadence_only_taxes_blocks_declared_after_it_resolves() {
    cr!("509.1c", "509.1d", "611.2c");
    ruling!(
        "War Cadence",
        "The ability only applies to blocks declared after it resolves. It will not add costs to any blockers already announced."
    );
    supported("War Cadence");
    let mut t = TestGame::new(2);
    let cadence = t.battlefield(P0, "War Cadence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(crate::r_s21_common::blocks_now(&t), vec![(giant, bears)]);
    let from = t.asked().len();
    crate::r_p153_common::answer_x(&mut t, P0, 1);
    act(&mut t, P0, cadence, 0, &[]);
    // The Giant still blocks, and nothing was asked of P1; a block declared from now on
    // would cost {1} per blocker.
    assert_eq!(crate::r_s21_common::blocks_now(&t), vec![(giant, bears)]);
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p != P1 || matches!(d, Decision::Priority { .. })));
    assert!(mtg_engine::combat::required_block_cost(&t.g, elves).is_some());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

// --- Choices ---------------------------------------------------------------------------------

/// P0's Sphinx of Clear Skies, with a Forest and an Island (two basic land types), deals
/// combat damage to P1; the top two cards of P0's library. The trigger is on the stack.
fn sphinx_hits(t: &mut TestGame) -> Vec<ObjectId> {
    supported("Sphinx of Clear Skies");
    let sphinx = t.battlefield(P0, "Sphinx of Clear Skies");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Island");
    let two = stack_library(t, P0, &["Grizzly Bears", "Shock"]);
    attack_with(t, &[(sphinx, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    two
}

#[test]
fn sphinx_of_clear_skies_one_pile_can_hold_everything() {
    cr!("700.3d");
    ruling!(
        "Sphinx of Clear Skies",
        "That opponent may choose to put all of the cards into one pile and leave the other pile empty. In that case, the ability’s controller chooses whether to put the revealed cards into their hand or into their graveyard."
    );
    for take in [0, 1] {
        let mut t = TestGame::new(2);
        let two = sphinx_hits(&mut t);
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        t.answer_choose(P1, &[obj(two[0]), obj(two[1])]);
        t.answer(P0, DecisionKind::Option, Answer::Index(take));
        t.resolve_all();
        let piles = asked_who(
            &t,
            from,
            |d| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a pile"),
        );
        assert_eq!(piles, vec![P0]);
        if take == 0 {
            assert_eq!(t.hand_size(P0), hand + 2);
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert!(t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Grizzly Bears"));
        }
    }
}

#[test]
fn sphinx_of_clear_skies_controller_picks_the_separating_opponent_after_revealing() {
    cr!("700.3", "701.20a");
    interp!("controller-picks-the-choosing-opponent");
    ruling!(
        "Sphinx of Clear Skies",
        "The player who controls the triggered ability chooses which opponent separates the cards into two piles. This choice is made after revealing those cards."
    );
    let mut t = TestGame::new(3);
    let two = sphinx_hits(&mut t);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Choose a player"),
        |g| {
            g.reveals
                .revealed
                .iter()
                .map(|r| r.card)
                .collect::<Vec<_>>()
        },
    );
    let from = t.asked().len();
    t.answer_choose(P0, &[pl(P2)]);
    t.answer_choose(P2, &[obj(two[0])]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    let revealed = seen.lock().unwrap().clone();
    assert_eq!(revealed.len(), 1);
    assert!(revealed[0].contains(&two[0]) && revealed[0].contains(&two[1]));
    let separators = asked_who(
        &t,
        from,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("piles")),
    );
    assert_eq!(separators, vec![P2]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn explore_the_vastlands_players_choose_in_turn_order_seeing_earlier_reveals() {
    cr!("101.4", "701.20a");
    ruling!(
        "Wandering Archaic // Explore the Vastlands",
        "While resolving Explore the Vastlands, first the active player reveals the cards they are going to put into their hand. Then each other player in turn order does the same, knowing choices made before they choose."
    );
    supported("Wandering Archaic // Explore the Vastlands");
    let mut t = TestGame::new(3);
    let mut tops = vec![];
    for p in [P0, P1, P2] {
        tops.push(stack_library(
            &mut t,
            p,
            &["Forest", "Shock", "Grizzly Bears", "Island", "Opt"],
        ));
    }
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Choose cards");
    // The cards revealed so far (as each later player chooses).
    fn revealed(g: &mtg_engine::game::Game) -> Vec<ObjectId> {
        g.turn_events
            .iter()
            .chain(g.events.iter())
            .filter_map(|e| match e {
                mtg_engine::events::Event::Custom { name, obj, .. }
                    if name == mtg_engine::reveal::REVEALED =>
                {
                    *obj
                }
                _ => None,
            })
            .collect()
    }
    fn choosing(d: &Decision) -> bool {
        matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Choose cards")
    }
    let seen1 = crate::r_s01_common::watch(&mut t, P1, choosing, revealed);
    let seen2 = crate::r_s01_common::watch(&mut t, P2, choosing, revealed);
    t.answer_choose(P0, &[obj(tops[0][0]), obj(tops[0][1])]);
    t.answer_choose(P1, &[obj(tops[1][0])]);
    t.answer_choose(P2, &[]);
    lots(&mut t, P0);
    let c = t.hand(P0, "Wandering Archaic // Explore the Vastlands");
    let from = t.asked().len();
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(asked_who(&t, from, is_choice), vec![P0, P1, P2]);
    // P1 chose with P0's two cards revealed; P2 with P1's too.
    let s1 = seen1.lock().unwrap().clone();
    let s2 = seen2.lock().unwrap().clone();
    assert_eq!(s1.len(), 1);
    assert_eq!(s1[0], vec![tops[0][0], tops[0][1]]);
    assert_eq!(s2[0], vec![tops[0][0], tops[0][1], tops[1][0]]);
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P1, "Forest"));
    assert_eq!(t.life(P2), 23);
}

#[test]
fn pippin_offers_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "205.3a");
    ruling!(
        "Pippin, Guard of the Citadel",
        "You may choose any card type. The ones that may appear in a normal game of Magic are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes such as legendary and basic may not be chosen. Subtypes such as Halfling, Aura, and Forest also may not be chosen."
    );
    supported("Pippin, Guard of the Citadel");
    let mut t = TestGame::new(2);
    let pippin = t.battlefield(P0, "Pippin, Guard of the Citadel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, pippin, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a card type" => Some(options.clone()),
            _ => None,
        })
        .expect("a card type choice");
    for ty in [
        "artifact",
        "battle",
        "creature",
        "enchantment",
        "instant",
        "kindred",
        "land",
        "planeswalker",
        "sorcery",
    ] {
        assert!(options.iter().any(|o| o == ty), "{ty}");
    }
    for not in ["legendary", "basic", "snow", "Halfling", "Aura", "Forest"] {
        assert!(
            !options.iter().any(|o| o.eq_ignore_ascii_case(not)),
            "{not}"
        );
    }
}
