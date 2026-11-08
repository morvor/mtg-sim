//! Rulings batch P061 — spells and abilities that make players draw: draws that aren't
//! optional, discards and draws within one resolution (CR 608.2), counts fixed as the
//! effect begins (CR 608.2h) and draws replaced afterwards (CR 121.6, 614.11), {X}{X}
//! costs (CR 107.3b), targets (CR 115), modes (CR 700.2), and characteristic-setting
//! effects in layers (CR 613.1d, 613.4b).

use crate::r_p061_common::*;
use crate::r_s02_common::target_candidates;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::types::*;
use mtg_engine::*;

/// The decisions asked since `from`, without priority.
fn decisions_since(t: &TestGame, from: usize) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .cloned()
        .collect()
}

/// Whether any player got priority among the decisions asked since `from`, before the
/// first priority decision after them (i.e. during the resolution that followed `from`).
fn priority_before(t: &TestGame, from: usize, pred: fn(&Decision) -> bool) -> bool {
    let asked = t.asked();
    let mut seen_priority = false;
    for (_, d) in &asked[from..] {
        if pred(d) {
            return seen_priority;
        }
        if matches!(d, Decision::Priority { .. }) {
            seen_priority = true;
        }
    }
    panic!("no such decision");
}

#[test]
fn nin_draws_x_cards_regardless_of_toughness_and_not_optionally() {
    cr!("608.2c", "107.3");
    ruling!(
        "Nin, the Pain Artist",
        "The effect is not optional. The creature's controller can't choose to draw fewer than X cards."
    );
    ruling!(
        "Nin, the Pain Artist",
        "You may choose a value for X that's greater than the creature's toughness. Its controller will draw that many cards."
    );
    // "{X}{U}{R}, {T}: Nin deals X damage to target creature. That creature's controller
    // draws X cards." X = 4 at P1's Grizzly Bears (toughness 2).
    let mut t = TestGame::new(2);
    let nin = t.battlefield(P0, "Nin, the Pain Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    let before = libraries(&t);
    let from = t.asked().len();
    t.activate(P0, nin, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 4]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // P1 wasn't asked anything (no choice to draw fewer).
    assert!(decisions_since(&t, from).iter().all(|(p, _)| *p == P0));
}

#[test]
fn barbed_shocker_redraws_the_number_discarded() {
    cr!("608.2c");
    ruling!(
        "Barbed Shocker",
        "The number of cards the player draws is equal to the number of cards they discarded, not the amount of damage dealt by Barbed Shocker."
    );
    let mut t = TestGame::new(2);
    let shocker = t.battlefield(P0, "Barbed Shocker");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    let before = libraries(&t);
    declare_from(&mut t, P0, &[(shocker, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(drawn_since(&t, &before), vec![0, 3]);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn compulsive_research_discards_a_land_or_two_cards() {
    cr!("608.2c", "118.12");
    ruling!(
        "Compulsive Research",
        "The target player can discard either one land card or two cards which may or may not be lands. The player can discard one land and one other card or two land cards if they choose."
    );
    // P0's hand before: Forest, Mountain, Grizzly Bears; P0 draws three fillers.
    for case in 0..4 {
        let mut t = TestGame::new(2);
        let forest = t.hand(P0, "Forest");
        let mountain = t.hand(P0, "Mountain");
        let bears = t.hand(P0, "Grizzly Bears");
        let (land, discard): (bool, Vec<ObjectId>) = match case {
            0 => (true, vec![forest]),
            1 => (false, vec![forest, mountain]),
            2 => (false, vec![forest, bears]),
            _ => (false, vec![bears]),
        };
        t.answer_yes(P0, land);
        if case == 3 {
            // Two cards, neither a land: Grizzly Bears and a drawn filler (chosen by the
            // engine's default for the rest).
            t.answer_choose(P0, &[o(bears)]);
        } else {
            t.answer_choose(P0, &discard.iter().map(|c| o(*c)).collect::<Vec<_>>());
        }
        cast_resolve(&mut t, P0, "Compulsive Research", &[pl(P0)]);
        let discarded = if land { 1 } else { 2 };
        assert_eq!(t.graveyard_size(P0), discarded + 1, "case {case}");
        assert_eq!(t.hand_size(P0), 3 + 3 - discarded, "case {case}");
        if case < 3 {
            for c in &discard {
                assert_eq!(t.zone(*c), mtg_engine::object::Zone::Graveyard(P0));
            }
        }
    }
}

#[test]
fn peer_into_the_abyss_draws_can_be_replaced() {
    cr!("608.2h", "121.6", "614.11");
    ruling!(
        "Peer into the Abyss",
        "To draw the cards, first the player determines how many cards to draw, then draws them. This event may be modified so that a different number of cards are actually drawn."
    );
    // P0's library: 20 cards → 10 draws; Thought Reflection ("If you would draw a card,
    // draw two cards instead.") makes them 20.
    for reflection in [false, true] {
        let mut t = TestGame::new(2);
        library_of(&mut t, P0, 20);
        if reflection {
            t.battlefield(P0, "Thought Reflection");
        }
        let before = libraries(&t);
        cast_resolve(&mut t, P0, "Peer into the Abyss", &[pl(P0)]);
        let n = if reflection { 20 } else { 10 };
        assert_eq!(drawn_since(&t, &before)[0], n);
        assert_eq!(t.life(P0), 10);
    }
}

#[test]
fn folio_of_fancies_pays_twice_x() {
    cr!("107.3b", "107.3a");
    ruling!(
        "Folio of Fancies",
        "To pay {X}{X}, choose a value for X, and pay twice that much mana."
    );
    for (lands, ok) in [(4, true), (3, false)] {
        let mut t = TestGame::new(2);
        let folio = t.battlefield(P0, "Folio of Fancies");
        t.lands(P0, "Wastes", lands);
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        let before = libraries(&t);
        let r = t.activate(P0, folio, 0, &[]);
        assert_eq!(r.is_ok(), ok, "{lands} lands");
        if ok {
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
            t.resolve_all();
            assert_eq!(drawn_since(&t, &before), vec![2, 2]);
        }
    }
}

#[test]
fn geier_reach_sanitarium_discards_in_turn_order() {
    cr!("101.4", "101.4a", "608.2c");
    ruling!(
        "Geier Reach Sanitarium",
        "When you activate Geier Reach Sanitarium's last ability, first each player draws a card. Then the player whose turn it is selects a card from their hand and sets it aside without revealing it; proceeding in turn order, each other player does the same. Then the cards that were set aside are discarded at once."
    );
    // P0 activates it on P1's turn: P1 chooses first, then P2, then P0.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let g = t.battlefield(P0, "Geier Reach Sanitarium");
    t.lands(P0, "Wastes", 2);
    let before = libraries(&t);
    let from = t.asked().len();
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("discard")),
        |g| g.players.iter().map(|p| p.graveyard.len()).sum::<usize>(),
    );
    let i = ability(&mut t, g, "{2}");
    t.activate(P0, g, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![1, 1, 1]);
    let order: Vec<PlayerId> = decisions_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P1, P2, P0]);
    assert_eq!(
        (t.graveyard_size(P0), t.graveyard_size(P1), t.graveyard_size(P2)),
        (1, 1, 1)
    );
    // Set aside, then discarded at once: as P0 (the last) chooses, nobody has discarded.
    assert_eq!(*seen.lock().unwrap(), vec![0]);
}

#[test]
fn forced_fruition_isnt_optional() {
    cr!("603.2", "608.2c");
    ruling!(
        "Forced Fruition",
        "Yes, the opponent draws seven cards. It’s not optional."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forced Fruition");
    t.set_step(P1, Step::PrecombatMain);
    let before = libraries(&t);
    let from = t.asked().len();
    cast_resolve(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    assert_eq!(drawn_since(&t, &before), vec![0, 7]);
    assert!(!decisions_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
}

#[test]
fn wheel_and_deal_cant_target_you() {
    cr!("115.1", "102.3");
    ruling!("Wheel and Deal", "You can't target yourself.");
    let mut t = TestGame::new(3);
    let from = t.asked().len();
    cast(&mut t, P0, "Wheel and Deal", &[pl(P1)]);
    let cands = target_candidates(&t, P0, from);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&pl(P0))), "{cands:?}");
    assert!(cands[0].contains(&pl(P1)) && cands[0].contains(&pl(P2)));
}

#[test]
fn cephalid_broker_draws_and_discards_in_one_resolution() {
    cr!("608.2", "117.1");
    ruling!(
        "Cephalid Broker",
        "You draw two cards and discard two cards all while Cephalid Broker’s ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    let mut t = TestGame::new(2);
    let broker = t.battlefield(P0, "Cephalid Broker");
    stack_library(&mut t, P0, &["Lightning Bolt", "Shock"]);
    t.lands(P0, "Mountain", 1);
    let from = t.asked().len();
    t.activate(P0, broker, 0, &[pl(P0)]).unwrap();
    t.resolve_all();
    assert!(!priority_before(&t, from + 1, |d| matches!(
        d,
        Decision::ChooseEntities { .. }
    )));
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt") && t.in_graveyard(P0, "Shock"));
    assert_eq!(t.life(P1), 20, "nothing was cast in between");
}

#[test]
fn dakra_mystic_decides_after_seeing_the_cards() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Dakra Mystic",
        "You make the decision after all the cards are revealed. If you decide not to put the cards into their owners' graveyards, the card each player draws will be the revealed card."
    );
    for mill in [false, true] {
        let mut t = TestGame::new(2);
        let dakra = t.battlefield(P0, "Dakra Mystic");
        let a = stack_library(&mut t, P0, &["Lightning Bolt"])[0];
        let b = stack_library(&mut t, P1, &["Shock"])[0];
        t.lands(P0, "Island", 1);
        t.answer_yes(P0, mill);
        t.activate(P0, dakra, 0, &[]).unwrap();
        t.resolve_all();
        if mill {
            assert!(t.in_graveyard(P0, "Lightning Bolt") && t.in_graveyard(P1, "Shock"));
            assert_eq!(t.hand_size(P0) + t.hand_size(P1), 0);
        } else {
            assert!(t.in_hand(P0, "Lightning Bolt") && t.in_hand(P1, "Shock"));
            assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 1));
        }
        let _ = (a, b);
    }
}

#[test]
fn chandra_ablaze_minus_two_with_an_empty_hand() {
    cr!("606.3", "121.2");
    ruling!(
        "Chandra Ablaze",
        "You may activate Chandra's second ability even if your hand is empty. As it resolves, a player whose hand is empty simply draws three cards."
    );
    let mut t = TestGame::new(2);
    let ch = crate::r_p007_common::pw(&mut t, P0, "Chandra Ablaze", 5);
    t.hand(P1, "Grizzly Bears");
    assert_eq!(t.hand_size(P0), 0);
    let i = ability(&mut t, ch, "−2");
    t.activate(P0, ch, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (3, 3));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn mind_spiral_stuns_an_already_tapped_creature() {
    cr!("702.174a", "122.1d", "701.26a");
    ruling!(
        "Mind Spiral",
        "You may target a creature that is already tapped with Mind Spiral. If the target creature is already tapped as Mind Spiral resolves, you will still put a stun counter on it."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    cast_resolve(&mut t, P0, "Mind Spiral", &[pl(P0), o(giant)]);
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
}

#[test]
fn drown_in_dreams_both_modes_in_order() {
    cr!("700.2", "700.2d", "608.2c");
    ruling!(
        "Drown in Dreams",
        "You may target the same player with both modes or target different players. The effects happen in the order they're listed on the card."
    );
    use crate::r_s13_common::{commander, commander_game};
    for same in [true, false] {
        let mut t = commander_game();
        let c = commander(&mut t, P0, "Grizzly Bears");
        crate::r_s05_common::move_to(&mut t, c, mtg_engine::object::Zone::Battlefield);
        // P0 (or P1 for the mill) has a known top of library: X = 2 draws the top two,
        // then mills the next four.
        let mill_target = if same { P0 } else { P1 };
        let top = stack_library(&mut t, P0, &["Shock", "Lightning Bolt", "Forest", "Island"]);
        give_mana_for(&mut t, P0, "Drown in Dreams");
        t.lands(P0, "Island", 2);
        let s = t.hand(P0, "Drown in Dreams");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.answer_targets(P0, &[pl(P0)]);
        t.answer_targets(P0, &[pl(mill_target)]);
        let gy = t.graveyard_size(mill_target);
        t.cast(P0, s).go();
        t.resolve_all();
        // The draw happened first: the top two are in hand.
        assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Lightning Bolt"), "{same}");
        let milled = t.graveyard_size(mill_target) - gy - usize::from(same);
        assert_eq!(milled, 4, "{same}");
        if same {
            assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Island"));
        }
        let _ = top;
    }
}

#[test]
fn whispering_madness_draws_before_the_cipher_choice() {
    cr!("702.99a", "608.2c");
    ruling!(
        "Whispering Madness",
        "You’ll draw cards before deciding which creature (if any) Whispering Madness will be encoded on."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("cipher")),
        |g| g.player(PlayerId(0)).hand.len(),
    );
    cast_resolve(&mut t, P0, "Whispering Madness", &[]);
    assert_eq!(*seen.lock().unwrap(), vec![2], "P0 had drawn two");
}

#[test]
fn pact_of_the_serpent_choices_and_count() {
    cr!("601.2c", "608.2h", "205.3m");
    ruling!(
        "Pact of the Serpent",
        "You choose the target player as you cast Pact of the Serpent, but you don't choose the creature type until the spell resolves."
    );
    ruling!(
        "Pact of the Serpent",
        "The value of X is based on the number of creatures the player controls of the chosen type as Pact of the Serpent resolves."
    );
    ruling!(
        "Pact of the Serpent",
        "You must choose an existing creature type, such as Squirrel or Knight. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    let elf = subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Elf")
        .unwrap();
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Llanowar Elves");
    t.answer(P0, DecisionKind::Option, Answer::Index(elf));
    let from = t.asked().len();
    let pact = cast(&mut t, P0, "Pact of the Serpent", &[pl(P1)]);
    // Cast: the target was chosen, the type wasn't.
    assert!(!t
        .asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    // In response, a second Elf arrives.
    t.battlefield(P1, "Elvish Mystic");
    let before = libraries(&t);
    t.resolve();
    assert!(!t.g.stack.contains(&pact));
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    assert_eq!(t.life(P1), 18);
    // The options were creature types only.
    let options = t
        .asked()
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .unwrap();
    assert!(options.contains(&"Squirrel".to_string()) && options.contains(&"Knight".to_string()));
    for not in ["Artifact", "Snow", "Legendary", "Creature"] {
        assert!(!options.iter().any(|o| o == not), "{not}");
    }
}

#[test]
fn metamorphic_blast_turns_a_creature_into_a_white_rabbit() {
    cr!("613.1d", "613.1e", "613.4b", "613.4c", "702.172a");
    ruling!(
        "Metamorphic Blast",
        "The effect of Metamorphic Blast’s first mode causes the target creature to lose its other colors and creature types. It doesn’t lose any of its abilities."
    );
    ruling!(
        "Metamorphic Blast",
        "The effect of Metamorphic Blast’s first mode will overwrite any previous effects that set the creature’s power and toughness to specific values. Effects that otherwise modify the target creature’s power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    // Wind Drake (2/2 blue Drake, flying) with a +1/+1 counter, Giant Growth (+3/+3) and
    // Diminish (base 1/1) before: a white Rabbit with base 0/1, flying, 4/5.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    put(&mut t, drake, PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[o(drake)]);
    cast_resolve(&mut t, P0, "Diminish", &[o(drake)]);
    assert_eq!(t.pt(drake), (5, 5));
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Metamorphic Blast");
    t.cast(P0, s).modes(&[0]).target(drake).go();
    t.resolve_all();
    let d = t.obj_now(drake);
    assert_eq!(t.pt(drake), (4, 5));
    assert_eq!(d.chars.colors, ColorSet::single(Color::White));
    assert_eq!(d.chars.subtypes.len(), 1);
    assert!(d.chars.has_subtype("Rabbit") && !d.chars.has_subtype("Drake"));
    assert!(d.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn will_kenrith_overwrites_earlier_base_pt_and_is_overwritten_by_later() {
    cr!("613.4b", "613.7a");
    ruling!(
        "Will Kenrith",
        "Will's first ability overwrites all previous effects that set a creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after that ability resolves will overwrite this effect."
    );
    let mut t = TestGame::new(2);
    let will = crate::r_p007_common::pw(&mut t, P0, "Will Kenrith", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Diminish", &[o(giant)]);
    assert_eq!(t.pt(giant), (1, 1));
    let i = ability(&mut t, will, "+2");
    t.activate(P0, will, i, &[o(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (0, 3));
    cast_resolve(&mut t, P0, "Diminish", &[o(giant)]);
    assert_eq!(t.pt(giant), (1, 1));
}

/// Rankle (P0) deals combat damage to P1 with these modes answered.
fn rankle_hits(t: &mut TestGame, modes: Vec<usize>) {
    let rankle = t.battlefield(P0, "Rankle, Master of Pranks");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(modes));
    declare_from(t, P0, &[(rankle, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

#[test]
fn rankle_modes_once_each_even_if_nobody_is_affected() {
    cr!("700.2", "700.2d");
    ruling!(
        "Rankle, Master of Pranks",
        "You can choose a mode even if some or all players will be unaffected."
    );
    // No cards in hand: the discard mode still can be chosen.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    rankle_hits(&mut t, vec![0]);
    let offered = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, vec![0, 1, 2]);
    assert_eq!(t.life(P1), 17);

    ruling!(
        "Rankle, Master of Pranks",
        "You can't choose a mode more than once."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    rankle_hits(&mut t, vec![1, 1]);
    let repeat = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { allow_repeat, .. } => Some(*allow_repeat),
        _ => None,
    });
    assert_eq!(repeat, Some(false));
    // At most one life lost and one card drawn each.
    assert!(t.life(P0) >= 19, "{}", t.life(P0));
    assert!(t.life(P1) >= 16);
    assert!(t.hand_size(P0) <= 1);
}

#[test]
fn gluntchs_first_chosen_player_chooses_the_creature() {
    cr!("608.2c");
    ruling!(
        "Gluntch, the Bestower",
        "The first chosen player gets to choose which creature they will place the counters on."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gluntch, the Bestower");
    t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_choose(P1, &[o(giant)]);
    t.answer_choose(P0, &[pl(P2)]);
    t.answer_choose(P0, &[pl(P0)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(giant, PLUS1), 2);
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&o(giant)))));

    ruling!(
        "Gluntch, the Bestower",
        "You may choose a player who doesn't control any creatures as the first chosen player."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gluntch, the Bestower");
    t.answer_choose(P0, &[pl(P2)]);
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_choose(P0, &[pl(P0)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let first = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, prompt, .. }
                if prompt.contains("two +1/+1 counters") =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap();
    assert!(first.contains(&pl(P2)));
    // P2 was chosen: nothing gets counters; P1 drew; P0 got the Treasures.
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
}
