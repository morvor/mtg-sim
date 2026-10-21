//! Choices each player makes for the same object, read by who chose what (CR 607.2d;
//! `oracle/patterns/choice_grammar_per_player.rs`): "each player chooses war or peace"
//! with "creatures controlled by players who chose war", "for each player who chose
//! money", "each opponent who chose silence", "if each opponent chose snitch".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

/// Queues `p`'s answer to the next "choose one" decision.
fn pick(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn archangel_of_strife_each_players_creatures_get_the_bonus_that_player_chose() {
    cr!("607.2d", "101.4", "614.12");
    ruling!(
        "Archangel of Strife",
        "The active player chooses war or peace first, followed by each other player in turn order"
    );
    ruling!(
        "Archangel of Strife",
        "depends only on the choice that player made when Archangel of Strife entered"
    );
    compiles("Archangel of Strife");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 1); // peace
    let angel = t.enter(P0, "Archangel of Strife");
    t.settle();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P0, P1], "the active player chooses first");
    assert_eq!(t.pt(mine), (5, 2), "war: +3/+0");
    assert_eq!(t.pt(theirs), (2, 5), "peace: +0/+3");
    assert_eq!(t.pt(angel), (9, 6), "it enters as a 9/6 with its controller's war");
}

#[test]
fn archangel_of_strife_two_archangels_with_different_choices_give_both_bonuses() {
    cr!("607.2d");
    ruling!(
        "Archangel of Strife",
        "you choose war for one and peace for the other, your creatures get +3/+3"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 0); // war
    t.enter(P0, "Archangel of Strife");
    pick(&mut t, P0, 1); // peace
    pick(&mut t, P1, 0); // war
    t.enter(P0, "Archangel of Strife");
    t.settle();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn archangel_of_strife_leaving_ends_the_bonus_and_a_new_object_chooses_again() {
    cr!("400.7", "607.2d");
    ruling!(
        "Archangel of Strife",
        "As soon as Archangel of Strife leaves the battlefield, the bonus stops applying"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 0);
    let angel = t.enter(P0, "Archangel of Strife");
    t.settle();
    assert_eq!(t.pt(bears), (5, 2));
    t.g.destroy(angel, None);
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn master_of_ceremonies_each_opponents_choice_gives_both_of_you_its_reward() {
    cr!("101.4", "608.2c");
    compiles("Master of Ceremonies");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Master of Ceremonies");
    pick(&mut t, P1, 0); // money
    pick(&mut t, P2, 2); // secrets
    t.advance_to(P2, Step::End);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let treasures = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == p && t.g.obj(**id).chars.has_subtype("Treasure"))
            .count()
    };
    let citizens = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype("Citizen"))
        .count();
    assert_eq!(treasures(&t, P0), 1, "P1 chose money");
    assert_eq!(treasures(&t, P1), 1);
    assert_eq!(treasures(&t, P2), 0);
    assert_eq!(citizens, 0, "no one chose friends");
    assert_eq!(t.hand_size(P0), hands.0 + 1, "P2 chose secrets");
    assert_eq!(t.hand_size(P2), hands.2 + 1);
    assert_eq!(t.hand_size(P1), hands.1);
}

/// Prisoner's Dilemma cast by P0 against two opponents choosing `p1` and `p2` (0 silence,
/// 1 snitch): their life totals after it resolves.
fn dilemma(p1: usize, p2: usize) -> (i32, i32) {
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Prisoner's Dilemma");
    t.advance_to(P0, Step::PrecombatMain);
    pick(&mut t, P1, p1);
    pick(&mut t, P2, p2);
    t.cast(P0, spell).go();
    t.resolve_all();
    (t.life(P1), t.life(P2))
}

#[test]
fn prisoners_dilemma_deals_damage_by_what_the_opponents_secretly_chose() {
    cr!("608.2c", "608.2e");
    compiles("Prisoner's Dilemma");
    assert_eq!(dilemma(0, 0), (16, 16), "each chose silence: 4 each");
    assert_eq!(dilemma(1, 1), (12, 12), "each chose snitch: 8 each");
    assert_eq!(dilemma(0, 1), (8, 20), "otherwise: 12 to who chose silence");
}
