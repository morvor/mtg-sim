//! Rulings batch P159 — "your life total becomes N" (CR 119.5, 810.9c): it is gaining or
//! losing the difference; Two-Headed Giant team totals; exchanging and redistributing life
//! totals (CR 701.12c, 810.9e); and Exquisite Archangel's "if you would lose the game"
//! replacement (CR 104.3, 614.1a, 616.1).

use crate::r_p044_common::cast_spell;
use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn gained(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_gained.get(&p).copied().unwrap_or(0)
}

fn lost(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_lost.get(&p).copied().unwrap_or(0)
}

/// Advances to `p`'s next upkeep (from the other player's end step) with the upkeep
/// triggers on the stack.
fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = PlayerId(((p.idx() + 1) % t.g.players.len()) as u8);
    t.set_step(other, Step::End);
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Makes P0's life total "become" a number with `card`'s ability, starting from `from`.
fn set_life_with(card: &str, from: i32) -> TestGame {
    supported(card);
    let mut t = TestGame::new(2);
    t.g.players[0].life = from;
    match card {
        "Magister Sphinx" => {
            t.answer_targets(P0, &[Entity::Player(P0)]);
            t.enter(P0, card);
        }
        "Form of the Dinosaur" | "Resolute Archangel" => {
            t.enter(P0, card);
        }
        "Loxodon Lifechanter" => {
            t.answer_yes(P0, true);
            t.enter(P0, card);
        }
        "Invincible Hymn" | "Oketra's Last Mercy" => {
            cast_spell(&mut t, P0, card, &[]);
        }
        "Blessed Wind" => {
            cast_spell(&mut t, P0, card, &[Entity::Player(P0)]);
        }
        "Touch of the Eternal" => {
            t.battlefield(P0, card);
            to_upkeep(&mut t, P0);
        }
        "Exquisite Archangel" => {
            t.battlefield(P0, card);
            // 10 poison counters: P0 would lose the game whatever their life total.
            t.g.players[0].counters.insert("poison".into(), 10);
        }
        _ => unreachable!(),
    }
    t.resolve_all();
    t
}

#[test]
fn a_life_total_becoming_a_number_is_a_gain_or_a_loss() {
    cr!("119.5", "119.3", "603.2");
    ruling!(
        "Magister Sphinx",
        "For a player's life total to become 10, what actually happens is that the player gains or loses the appropriate amount of life."
    );
    ruling!(
        "Form of the Dinosaur",
        "For your life total to become 15, you gain or lose the appropriate amount of life."
    );
    ruling!(
        "Invincible Hymn",
        "For your life total to become the appropriate number, you actually gain or lose the necessary amount of life."
    );
    ruling!(
        "Exquisite Archangel",
        "For your life total to become your starting life total (normally 20), you gain or lose the appropriate amount of life."
    );
    ruling!(
        "Oketra's Last Mercy",
        "For your life total to become your starting life total (normally 20), you gain or lose the appropriate amount of life."
    );
    ruling!(
        "Resolute Archangel",
        "If your life total is less than your starting life total, you actually gain the appropriate amount of life."
    );
    ruling!(
        "Touch of the Eternal",
        "This change of life total counts as life gain or life loss, as appropriate."
    );
    ruling!(
        "Blessed Wind",
        "This counts as gaining or losing life as appropriate."
    );
    ruling!(
        "Loxodon Lifechanter",
        "For your life total to become a certain number, you gain or lose the corresponding amount of life."
    );
    // (card, life before, life after; the after values are those each card sets).
    // Invincible Hymn: 30 cards in the library; Touch of the Eternal: 1 permanent;
    // Loxodon Lifechanter: 6 total toughness.
    let cases: &[(&str, i32, i32)] = &[
        ("Magister Sphinx", 4, 10),
        ("Magister Sphinx", 17, 10),
        ("Form of the Dinosaur", 4, 15),
        ("Form of the Dinosaur", 40, 15),
        ("Invincible Hymn", 4, 30),
        ("Invincible Hymn", 40, 30),
        ("Exquisite Archangel", 4, 20),
        ("Exquisite Archangel", 40, 20),
        ("Oketra's Last Mercy", 4, 20),
        ("Oketra's Last Mercy", 25, 20),
        ("Resolute Archangel", 4, 20),
        ("Touch of the Eternal", 4, 1),
        ("Blessed Wind", 4, 20),
        ("Blessed Wind", 25, 20),
        ("Loxodon Lifechanter", 4, 6),
        ("Loxodon Lifechanter", 10, 6),
    ];
    for &(card, from, to) in cases {
        let t = set_life_with(card, from);
        assert_eq!(t.life(P0), to, "{card} from {from}");
        let (g, l) = ((to - from).max(0) as u32, (from - to).max(0) as u32);
        assert_eq!((gained(&t, P0), lost(&t, P0)), (g, l), "{card} from {from}");
    }
    // Other cards that care about life gain or loss see it: Ajani's Pridemate ("Whenever
    // you gain life") and Exquisite Blood ("Whenever an opponent loses life, you gain that
    // much life").
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[0].life = 4;
    cast_spell(&mut t, P0, "Invincible Hymn", &[]);
    assert_eq!(t.counters(mate, "+1/+1"), 1);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Exquisite Blood");
    t.g.players[0].life = 40;
    cast_spell(&mut t, P0, "Oketra's Last Mercy", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 40));
}

#[test]
fn set_life_totals_are_counted_once_as_they_resolve() {
    cr!("608.2h", "119.5");
    ruling!(
        "Invincible Hymn",
        "Invincible Hymn checks the number of cards in your library just once: when it resolves."
    );
    ruling!(
        "Loxodon Lifechanter",
        "If you set your life total to become equal to the toughness of creatures you control, this happens once."
    );
    let mut t = TestGame::new(2);
    cast_spell(&mut t, P0, "Invincible Hymn", &[]);
    assert_eq!(t.life(P0), 30);
    t.g.draw_cards(P0, 3);
    assert_eq!(t.life(P0), 30);
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let lifechanter = t.enter(P0, "Loxodon Lifechanter");
    t.resolve_all();
    assert_eq!(t.life(P0), 6);
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(lifechanter).1, 6);
    assert_eq!(t.life(P0), 6);
}

/// A Two-Headed Giant game (P0 and P1 against P2 and P3) with each team's life total.
fn two_headed_giant(team0: i32, team1: i32) -> TestGame {
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    for (p, life) in [(0, team0), (1, team0), (2, team1), (3, team1)] {
        t.g.players[p].life = life;
    }
    t
}

#[test]
fn in_two_headed_giant_the_team_total_becomes_the_number_but_only_you_gain_or_lose() {
    cr!("810.9c", "810.9a", "119.1a", "119.5");
    ruling!(
        "Exquisite Archangel",
        "In a Two-Headed Giant game, Exquisite Archangel’s ability causes the team’s life total to become the team’s starting life total (normally 30), but only you actually gain or lose life."
    );
    ruling!(
        "Form of the Dinosaur",
        "In a Two-Headed Giant game, Form of the Dinosaur’s first ability causes your team’s life total to become 15. Only you gain or lose life this way."
    );
    ruling!(
        "Invincible Hymn",
        "In a Two-Headed Giant game, Invincible Hymn will essentially set your team’s life total to the number of cards in your library."
    );
    ruling!(
        "Oketra's Last Mercy",
        "In a Two-Headed Giant game, Oketra’s Last Mercy causes the team’s life total to become the team’s starting life total (normally 30), but only you actually gain or lose life."
    );
    // Form of the Dinosaur: the team at 22 goes to 15; P0 loses 7, P1 nothing.
    let mut t = two_headed_giant(22, 30);
    t.enter(P0, "Form of the Dinosaur");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (15, 15));
    assert_eq!((lost(&t, P0), lost(&t, P1)), (7, 0));
    // Invincible Hymn: the team at 8 with 30 cards in P0's library goes to 30, a gain of
    // 22 for P0 only.
    let mut t = two_headed_giant(8, 30);
    cast_spell(&mut t, P0, "Invincible Hymn", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (30, 30));
    assert_eq!((gained(&t, P0), gained(&t, P1)), (22, 0));
    // Oketra's Last Mercy: the team at 12 goes to its starting 30.
    let mut t = two_headed_giant(12, 30);
    cast_spell(&mut t, P0, "Oketra's Last Mercy", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (30, 30));
    assert_eq!((gained(&t, P0), gained(&t, P1)), (18, 0));
    // Exquisite Archangel: the team at 0 would lose; it goes to 30 instead.
    let mut t = two_headed_giant(0, 30);
    t.battlefield(P0, "Exquisite Archangel");
    t.settle();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!((t.life(P0), t.life(P1)), (30, 30));
    assert_eq!((gained(&t, P0), gained(&t, P1)), (30, 0));
    assert!(t.in_exile("Exquisite Archangel"));
}

// --- Exquisite Archangel ---------------------------------------------------------------

#[test]
fn exquisite_archangel_applies_to_any_loss_but_a_new_empty_draw_loses_again() {
    cr!("104.3c", "704.5b", "614.1a", "121.4");
    ruling!(
        "Exquisite Archangel",
        "Exquisite Archangel’s effect applies any time you would lose the game, even if you’re not losing due to your life total being 0 or less."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exquisite Archangel");
    t.g.players[0].life = 7;
    let lib = t.g.players[0].library.clone();
    for c in lib {
        t.g.move_object(
            c,
            mtg_engine::object::Zone::Graveyard(P0),
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    assert!(t.in_exile("Exquisite Archangel"));
    // The failed draw doesn't make P0 lose again at the next check...
    t.settle();
    assert!(!t.has_lost(P0));
    // ... only trying to draw again from the empty library does.
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn exquisite_archangel_does_nothing_against_conceding() {
    cr!("104.3a");
    ruling!(
        "Exquisite Archangel",
        "Exquisite Archangel’s effect does nothing if you concede the game."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Exquisite Archangel");
    t.g.perform_action(P0, Action::Concede).unwrap();
    t.settle();
    assert!(t.has_lost(P0));
    assert!(!t.in_exile("Exquisite Archangel"));
}

#[test]
fn exquisite_archangel_dealt_lethal_damage_with_you_still_applies() {
    cr!("704.3", "704.5a", "704.5g", "616.1");
    ruling!(
        "Exquisite Archangel",
        "If Exquisite Archangel is dealt lethal damage at the same time that you’re dealt damage that brings your life total to 0 or less, its effect applies"
    );
    for exile in [true, false] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P0, "Exquisite Archangel");
        t.g.players[0].life = 3;
        let src = t.battlefield(P1, "Mountain");
        t.answer_yes(P0, exile);
        t.g.deal_damage_batch(
            vec![(src, Entity::Player(P0), 3), (src, Entity::Object(angel), 5)],
            false,
        );
        t.settle();
        assert!(!t.has_lost(P0));
        assert_eq!(t.life(P0), 20);
        assert_eq!(t.in_exile("Exquisite Archangel"), exile);
        assert_eq!(t.in_graveyard(P0, "Exquisite Archangel"), !exile);
    }
}

#[test]
fn exquisite_archangel_and_cant_lose_or_an_opponent_winning() {
    cr!("101.2", "104.2b", "104.3b");
    ruling!(
        "Exquisite Archangel",
        "If an effect says that you can’t lose the game, Exquisite Archangel’s effect doesn’t apply."
    );
    ruling!(
        "Exquisite Archangel",
        "If an effect states that an opponent wins the game, Exquisite Archangel’s ability doesn’t apply."
    );
    // Platinum Angel: P0 at 0 life doesn't lose, and the Archangel stays.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exquisite Archangel");
    t.battlefield(P0, "Platinum Angel");
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.named_on_battlefield("Exquisite Archangel").len(), 1);
    // Felidar Sovereign: P1 at 40 life wins at their upkeep; P0's Archangel doesn't help.
    supported("Felidar Sovereign");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exquisite Archangel");
    t.battlefield(P1, "Felidar Sovereign");
    t.g.players[1].life = 40;
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert_eq!(t.named_on_battlefield("Exquisite Archangel").len(), 1);
}

#[test]
fn with_two_exquisite_archangels_only_one_applies() {
    cr!("616.1", "616.1f", "614.5");
    ruling!(
        "Exquisite Archangel",
        "If you control two Exquisite Archangels, you choose which one’s effect applies."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Exquisite Archangel");
    let b = t.battlefield(P0, "Exquisite Archangel");
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "one Archangel was exiled, the other stays"
    );
    // The next time P0 would lose, the remaining one applies.
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

// --- Exchanging and redistributing life totals ----------------------------------------

#[test]
fn psychic_transfer_uses_actual_negative_life_totals() {
    cr!("701.12c", "119.5");
    ruling!(
        "Psychic Transfer",
        "If one of the players has a negative life total, you use their actual life total for determining the difference."
    );
    supported("Psychic Transfer");
    // P1 (who can't lose, Platinum Angel) is at -2 and P0 at 3: a difference of 5.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Platinum Angel");
    t.g.players[1].life = -2;
    t.g.players[0].life = 3;
    cast_spell(&mut t, P0, "Psychic Transfer", &[Entity::Player(P1)]);
    assert_eq!((t.life(P0), t.life(P1)), (-2, 3));
    assert!(t.has_lost(P0));
    // At -3 against 3 (a difference of 6), nothing happens.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Platinum Angel");
    t.g.players[1].life = -3;
    t.g.players[0].life = 3;
    cast_spell(&mut t, P0, "Psychic Transfer", &[Entity::Player(P1)]);
    assert_eq!((t.life(P0), t.life(P1)), (3, -3));
}

/// P0's `card` targets P0 and P1 to exchange their life totals (Soul Conduit's ability is
/// activated; Axis of Mortality's upkeep trigger is put on the stack). Returns once it's on
/// the stack.
fn exchange_on_stack(t: &mut TestGame, card: &str, targets: [PlayerId; 2]) {
    supported(card);
    let source = t.battlefield(P0, card);
    for p in targets {
        t.answer_targets(P0, &[Entity::Player(p)]);
    }
    if card == "Soul Conduit" {
        t.lands(P0, "Wastes", 6);
        crate::r_s06_common::activate_containing(t, P0, source, "exchange").expect("activate");
    } else {
        t.answer_yes(P0, true);
        to_upkeep(t, P0);
    }
}

#[test]
fn an_illegal_target_player_stops_the_exchange() {
    cr!("608.2b", "701.12a");
    ruling!(
        "Soul Conduit",
        "If one of the targeted players is an illegal target when the activated ability tries to resolve, the exchange won’t happen."
    );
    ruling!(
        "Axis of Mortality",
        "If one of the targeted players is an illegal target when the triggered ability of Axis of Mortality tries to resolve, the exchange won’t happen."
    );
    for card in ["Soul Conduit", "Axis of Mortality"] {
        let mut t = TestGame::new(2);
        t.g.players[1].life = 7;
        exchange_on_stack(&mut t, card, [P0, P1]);
        assert_eq!(t.stack_len(), 1, "{card}");
        assert_eq!(
            crate::r_s25_common::targets_of(&t, t.g.stack[0]),
            vec![Entity::Player(P0), Entity::Player(P1)]
        );
        // P1 gets hexproof in response.
        t.battlefield(P1, "Leyline of Sanctity");
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (20, 7), "{card}");
    }
}

#[test]
fn exchanging_life_totals_is_gaining_and_losing_life() {
    cr!("701.12c", "119.5", "603.2");
    ruling!(
        "Soul Conduit",
        "When the life totals are exchanged, each player gains or loses the amount of life necessary to equal the other player’s previous life total."
    );
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P1, "Ajani's Pridemate");
    t.g.players[0].life = 5;
    t.g.players[1].life = 3;
    exchange_on_stack(&mut t, "Soul Conduit", [P0, P1]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (3, 5));
    assert_eq!((lost(&t, P0), gained(&t, P1)), (2, 2));
    assert_eq!(t.counters(mate, "+1/+1"), 1);
}

#[test]
fn axis_of_mortality_in_two_headed_giant_exchanges_team_totals() {
    cr!("810.9e", "810.9a", "701.12c");
    ruling!(
        "Axis of Mortality",
        "If the two targeted players are on different teams, those players gain or lose the appropriate amount of life so that the teams end up exchanging life totals."
    );
    let mut t = two_headed_giant(30, 12);
    exchange_on_stack(&mut t, "Axis of Mortality", [P1, P2]);
    t.resolve_all();
    assert_eq!(
        [t.life(P0), t.life(P1), t.life(P2), t.life(P3)],
        [12, 12, 30, 30],
    );
    assert_eq!((lost(&t, P1), lost(&t, P0)), (18, 0));
    assert_eq!((gained(&t, P2), gained(&t, P3)), (18, 0));
    // Teammates can't exchange.
    let mut t = two_headed_giant(30, 12);
    exchange_on_stack(&mut t, "Axis of Mortality", [P0, P1]);
    t.resolve_all();
    assert_eq!([t.life(P0), t.life(P2)], [30, 12]);
}

#[test]
fn reverse_the_sands_gives_whole_life_totals_chosen_as_it_resolves() {
    cr!("608.2", "119.5");
    ruling!(
        "Reverse the Sands",
        "You can’t split up a life total when you redistribute it."
    );
    ruling!(
        "Reverse the Sands",
        "You choose which player gets which life total when the spell resolves."
    );
    supported("Reverse the Sands");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    t.g.players[1].life = 15;
    let asked = t.asked().len();
    let options = |t: &TestGame| {
        asked_since(t, asked)
            .into_iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseOption { options, .. } => Some(options),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    crate::r_p044_common::cast_only(&mut t, P0, "Reverse the Sands", &[]);
    assert_eq!(t.stack_len(), 1);
    assert!(options(&t).is_empty(), "nothing is chosen while casting");
    t.answer_choose(P0, &[Entity::Player(P0), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    // P0 picked between the two whole life totals: no 20 / 0 split is possible.
    assert_eq!(options(&t)[0], vec!["5".to_string(), "15".to_string()]);
    assert_eq!((t.life(P0), t.life(P1)), (15, 5));
}
