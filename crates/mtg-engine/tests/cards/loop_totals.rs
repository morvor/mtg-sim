//! Open-ended processes in which the players take turns, and what each player did over
//! all of them (CR 608.2c, 101.4b, 110.2a, 119.4, 303.4g, 810.9): Eureka, Hypergenesis,
//! Plague of Vermin, Thieves' Auction, Liege of the Hollows, Kamahl's Summons.

use crate::basic_effects_common::assert_supported;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

fn controller_of(t: &TestGame, name: &str) -> Vec<PlayerId> {
    t.named_on_battlefield(name)
        .iter()
        .map(|o| t.obj_now(*o).controller)
        .collect()
}

fn cast_eureka(t: &mut TestGame) {
    let spell = t.hand(P0, "Eureka");
    t.lands(P0, "Forest", 4);
    t.cast(P0, spell).go();
}

#[test]
fn eureka_goes_around_until_a_full_pass_puts_nothing() {
    cr!("608.2c", "101.4b", "110.2a");
    ruling!(
        "Eureka",
        "choose not to put a card onto the battlefield. It doesn"
    );
    assert_supported("Eureka");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    let giant = t.hand(P1, "Hill Giant");
    // Pass 1: P0 declines, P1 puts Hill Giant. Pass 2: P0 puts Grizzly Bears. Pass 3: P0
    // puts the Forest. Pass 4: no one puts anything.
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    cast_eureka(&mut t);
    t.resolve_all();
    assert_eq!(controller_of(&t, "Grizzly Bears"), vec![P0]);
    assert_eq!(controller_of(&t, "Hill Giant"), vec![P1]);
    assert!(t
        .g
        .permanents()
        .any(|o| o.chars.name == "Forest" && o.controller == P0 && !o.tapped));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn eureka_puts_permanents_sequentially_and_triggers_wait() {
    cr!("608.2c", "603.3");
    ruling!(
        "Eureka",
        "Anything that triggers during the resolution of this will wait"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sovereign = t.hand(P0, "Imposing Sovereign");
    let visionary = t.hand(P0, "Elvish Visionary");
    let giant = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(sovereign)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(visionary)]);
    cast_eureka(&mut t);
    t.resolve();
    // Imposing Sovereign entered first, so Hill Giant entered tapped.
    let g = t.named_on_battlefield("Hill Giant")[0];
    assert!(t.obj_now(g).tapped);
    // Elvish Visionary's trigger waited for Eureka to finish resolving.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn hypergenesis_lets_a_player_who_declined_put_a_card_later() {
    cr!("608.2c", "101.4b");
    ruling!(
        "Hypergenesis",
        "choose not to put a card onto the battlefield during one loop of the process"
    );
    ruling!("Hypergenesis", "The permanents enter the battlefield sequentially");
    ruling!(
        "Hypergenesis",
        "If a player chooses not to put a card onto the battlefield but the process repeats"
    );
    assert_supported("Hypergenesis");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let sovereign = t.hand(P0, "Imposing Sovereign");
    let visionary = t.hand(P0, "Elvish Visionary");
    let giant = t.hand(P1, "Hill Giant");
    // Suspended with one time counter: cast at P0's upkeep.
    let hyper = t.exile(P0, "Hypergenesis");
    t.g.objects[hyper.0 as usize].counters.insert("time".into(), 1);
    t.answer_yes(P0, true);
    // Pass 1: P0 puts Imposing Sovereign, P1 declines. Pass 2: P0 puts Elvish Visionary,
    // P1 puts Hill Giant (tapped). Pass 3: no one.
    t.answer_choose(P0, &[Entity::Object(sovereign)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P0, &[Entity::Object(visionary)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.obj_now(g).controller, P1);
    assert!(t.obj_now(g).tapped);
    assert_eq!(t.named_on_battlefield("Elvish Visionary").len(), 1);
}

#[test]
fn eureka_cant_put_an_aura_with_nothing_to_enchant() {
    cr!("303.4g");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P0, "Pacifism");
    let from = t.asked().len();
    cast_eureka(&mut t);
    t.resolve_all();
    assert!(t.in_hand(P0, "Pacifism"));
    // P0 had nothing to choose from.
    assert!(t.asked()[from..].iter().all(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => candidates.is_empty(),
        _ => true,
    }));
}

fn plague(t: &mut TestGame) {
    let spell = t.hand(P0, "Plague of Vermin");
    t.lands(P0, "Swamp", 7);
    t.cast(P0, spell).go();
}

fn rats(t: &TestGame, p: PlayerId) -> usize {
    t.named_on_battlefield("Rat Token")
        .iter()
        .filter(|o| t.obj_now(**o).controller == p)
        .count()
}

#[test]
fn plague_of_vermin_gives_each_player_rats_for_all_the_life_they_paid() {
    cr!("119.4", "608.2c", "101.4b");
    ruling!(
        "Plague of Vermin",
        "the total amount of life each player paid this way is calculated"
    );
    ruling!(
        "Plague of Vermin",
        "counts as three separate instances of losing life"
    );
    ruling!(
        "Plague of Vermin",
        "the process ends when all N players in sequence"
    );
    ruling!(
        "Plague of Vermin",
        "you can't spend life you haven't got"
    );
    assert_supported("Plague of Vermin");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Pass 1: P0 3, P1 0. Pass 2: P0 1, P1 2. Pass 3: P0 1, P1 0. Pass 4: nothing.
    for n in [3, 1, 1, 0] {
        t.answer(P0, DecisionKind::Number, Answer::Number(n));
    }
    for n in [0, 2, 0, 0] {
        t.answer(P1, DecisionKind::Number, Answer::Number(n));
    }
    let from = t.g.turn_events.len();
    plague(&mut t);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (15, 18));
    assert_eq!((rats(&t, P0), rats(&t, P1)), (5, 2));
    let losses = t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::LifeLost { player, .. } if *player == P0))
        .count();
    assert_eq!(losses, 3);
    // Each amount offered was at most the player's life at the time.
    let maxes: Vec<i64> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseNumber { max, .. } if *p == P0 => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![20, 17, 16, 15]);
}

#[test]
fn plague_of_vermin_in_two_headed_giant_adjusts_the_team_total_in_between() {
    cr!("119.4a", "810.9");
    ruling!("Plague of Vermin", "The team's life total is adjusted in between");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    let team_life = t.life(P0);
    t.answer(P0, DecisionKind::Number, Answer::Number(10));
    let from = t.asked().len();
    plague(&mut t);
    t.resolve_all();
    // P1, P0's teammate, could pay at most what was left of the team's life.
    let max = t.asked()[from..].iter().find_map(|(p, d)| match d {
        Decision::ChooseNumber { max, .. } if *p == P1 => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some((team_life - 10) as i64));
    assert_eq!(rats(&t, P0), 10);
}

#[test]
fn thieves_auction_puts_each_card_back_one_at_a_time_in_turn_order() {
    cr!("608.2c", "303.4g", "110.2a");
    ruling!(
        "Thieves' Auction",
        "You can’t pick a card that can’t legally be returned to the battlefield"
    );
    ruling!(
        "Thieves' Auction",
        "can still be chosen by you or another player later in the auction"
    );
    ruling!(
        "Thieves' Auction",
        "players make choices in turn order"
    );
    assert_supported("Thieves' Auction");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.objects[pacifism.0 as usize].attached_to = Some(Entity::Object(bears));
    let spell = t.hand(P0, "Thieves' Auction");
    t.lands(P0, "Mountain", 7);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let asked = t.asked();
    let asks: Vec<&Decision> = asked[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(_, d)| d)
        .collect();
    // The first choice: no creature to enchant yet, so not Pacifism.
    let Decision::ChooseEntities { candidates, .. } = asks[0] else {
        unreachable!()
    };
    assert!(!candidates.iter().any(|e| t.g.obj(e.object().unwrap()).chars.name == "Pacifism"));
    // Every card came back, tapped; Pacifism once a creature was there.
    assert!(t.g.permanents().all(|o| o.tapped || o.is(CardType::Enchantment)));
    assert_eq!(t.named_on_battlefield("Pacifism").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Mountain").len(), 7);
    assert!(!t.in_exile("Mountain"));
    // The players alternated, P0 first.
    let who = choosers_since(&t, from);
    assert_eq!(who[0], P0);
    assert_eq!(who[1], P1);
}

#[test]
fn liege_of_the_hollows_gives_each_player_squirrels_for_their_own_mana() {
    cr!("101.4", "608.2f");
    ruling!(
        "Liege of the Hollows",
        "then all the Squirrel tokens are put onto the battlefield at the same time"
    );
    assert_supported("Liege of the Hollows");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let liege = t.battlefield(P0, "Liege of the Hollows");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Swamp", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P1, DecisionKind::X, Answer::Number(1));
    let blade = t.hand(P1, "Doom Blade");
    t.g.turn.priority = Some(P1);
    t.cast(P1, blade).target(liege).go();
    let from = t.asked().len();
    t.resolve_all();
    // The player whose turn it is pays first, then the others in turn order.
    let payers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseX { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(payers, vec![P0, P1]);
    let squirrels = |p: PlayerId| {
        t.named_on_battlefield("Squirrel Token")
            .iter()
            .filter(|o| t.obj_now(**o).controller == p)
            .count()
    };
    assert_eq!((squirrels(P0), squirrels(P1)), (2, 1));
}

#[test]
fn kamahls_summons_counts_each_players_own_revealed_cards() {
    cr!("608.2f");
    assert_supported("Kamahl's Summons");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.hand(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let spell = t.hand(P0, "Kamahl's Summons");
    t.lands(P0, "Forest", 4);
    t.cast(P0, spell).go();
    t.resolve_all();
    let bears = |p: PlayerId| {
        t.named_on_battlefield("Bear Token")
            .iter()
            .filter(|o| t.obj_now(**o).controller == p)
            .count()
    };
    assert_eq!((bears(P0), bears(P1)), (2, 0));
}
