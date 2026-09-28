//! Rulings on tempting offer ("[effect]. Each opponent may [effect]. For each opponent who
//! does, [effect].", `oracle/patterns/each_opponent_may.rs`): the opponents decide in turn
//! order (CR 101.4), then the effect happens for those who accepted, then again for you
//! once per opponent who accepted.

use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

/// The public record of the players who accepted so far.
fn acceptances(g: &Game) -> Vec<String> {
    g.log
        .iter()
        .filter(|l| l.text.ends_with("chooses to take part"))
        .map(|l| l.text.clone())
        .collect()
}

/// The +1/+1 counters on the creatures each player controls, by player.
fn counters_by_player(g: &Game) -> Vec<u32> {
    (0..g.players.len())
        .map(|i| {
            g.permanents()
                .filter(|o| o.controller.idx() == i && o.is(CardType::Creature))
                .map(|o| o.counter(counters::PLUS1))
                .sum()
        })
        .collect()
}

/// `caster` casts Tempt with Glory ("Put a +1/+1 counter on each creature you control.
/// Each opponent may put a +1/+1 counter on each creature they control. For each opponent
/// who does, put a +1/+1 counter on each creature you control.") in their main phase, with
/// a Grizzly Bears for each of the four players.
fn glory_game(caster: PlayerId, accept: &[(PlayerId, bool)]) -> TestGame {
    supported("Tempt with Glory");
    let mut t = TestGame::new(4);
    t.set_step(caster, Step::PrecombatMain);
    for p in [P0, P1, P2, P3] {
        t.battlefield(p, "Grizzly Bears");
    }
    for (p, yes) in accept {
        t.answer_yes(*p, *yes);
    }
    t
}

fn cast_glory(t: &mut TestGame, caster: PlayerId) {
    t.lands(caster, "Plains", 6);
    let spell = t.hand(caster, "Tempt with Glory");
    t.cast(caster, spell).go();
    t.resolve_all();
}

#[test]
fn opponents_decide_in_turn_order_from_your_left_knowing_earlier_choices() {
    cr!("101.4", "101.4b");
    ruling!(
        "Tempt with Glory",
        "Your opponents decide in turn order whether or not they accept the offer, starting with the opponent on your left. Each opponent will know the decisions of previous opponents in turn order when making their decision."
    );
    // P2 casts it during P2's turn: P3 decides first, then P0, then P1.
    let mut t = glory_game(P2, &[(P3, true), (P0, false), (P1, true)]);
    let seen: Vec<Seen<Vec<String>>> = [P3, P0, P1]
        .iter()
        .map(|p| watch(&mut t, *p, is_yes_no, acceptances))
        .collect();
    let from = t.asked().len();
    cast_glory(&mut t, P2);
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_yes_no(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P3, P0, P1]);
    // Each one knows the choices made before theirs: P3 accepted, P0 (asked already)
    // didn't.
    assert_eq!(*seen[0].lock().unwrap(), vec![Vec::<String>::new()]);
    assert_eq!(
        *seen[1].lock().unwrap(),
        vec![vec!["P3 chooses to take part".to_string()]]
    );
    assert_eq!(
        *seen[2].lock().unwrap(),
        vec![vec!["P3 chooses to take part".to_string()]]
    );
    assert_eq!(counters_by_player(&t.g), vec![0, 1, 3, 1]);
}

#[test]
fn the_effect_happens_for_each_accepting_opponent_after_all_decided_then_again_for_you() {
    cr!("101.4");
    ruling!(
        "Tempt with Glory",
        "After each opponent has decided, the effect happens simultaneously for each one who accepted the offer. Then, the effect happens again for you a number of times equal to the number of opponents who accepted."
    );
    let mut t = glory_game(P0, &[(P1, true), (P2, false), (P3, true)]);
    // What the last opponent to decide sees: P0's first counter, but nothing yet for P1,
    // who accepted before.
    let seen = watch(&mut t, P3, is_yes_no, counters_by_player);
    cast_glory(&mut t, P0);
    assert_eq!(*seen.lock().unwrap(), vec![vec![1, 0, 0, 0]]);
    // P1 and P3 got theirs; P0 got one more for each of them.
    assert_eq!(counters_by_player(&t.g), vec![3, 1, 0, 1]);
}

#[test]
fn tempt_with_vengeance_gives_you_x_tokens_again_for_each_opponent_who_accepted() {
    cr!("101.4", "107.3a");
    ruling!(
        "Tempt with Vengeance",
        "After each opponent has decided, the effect happens simultaneously for each one who accepted the offer. Then, the effect happens again for you a number of times equal to the number of opponents who accepted."
    );
    supported("Tempt with Vengeance");
    // Tempt with Vengeance ({X}{R}): "Create X 1/1 red Elemental creature tokens with
    // haste. Each opponent may create X 1/1 red Elemental creature tokens with haste. For
    // each opponent who does, create X 1/1 red Elemental creature tokens with haste."
    let mut t = TestGame::new(4);
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    t.answer_yes(P3, false);
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Tempt with Vengeance");
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    let elementals: Vec<usize> = [P0, P1, P2, P3]
        .iter()
        .map(|p| with_subtype(&t, *p, "Elemental").len())
        .collect();
    assert_eq!(elementals, vec![2 + 2 * 2, 2, 2, 0]);
    for p in [P0, P1, P2] {
        for e in with_subtype(&t, p, "Elemental") {
            let o = t.obj(e);
            assert!(o.is_token() && o.controller == p && o.owner == p);
            assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
        }
    }
}

#[test]
fn nobody_accepting_means_the_effect_happens_only_once_for_you() {
    cr!("101.4");
    ruling!(
        "Tempt with Immortality",
        "After each opponent has decided, the effect happens simultaneously for each one who accepted the offer. Then, the effect happens again for you a number of times equal to the number of opponents who accepted."
    );
    supported("Tempt with Immortality");
    // Tempt with Immortality: "Return a creature card from your graveyard to the
    // battlefield. Each opponent may return a creature card from their graveyard to the
    // battlefield. For each opponent who does, return a creature card from your graveyard
    // to the battlefield."
    let mut t = TestGame::new(2);
    let mine = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Hill Giant"),
    ];
    let theirs = t.graveyard(P1, "Gray Ogre");
    t.answer_yes(P1, false);
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Tempt with Immortality");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(mine.iter().filter(|c| t.on_battlefield(**c)).count(), 1);
    assert!(!t.on_battlefield(theirs));
    // Accepting: P1 returns theirs (under their control), and P0 returns another.
    let mut t = TestGame::new(2);
    let mine = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Hill Giant"),
    ];
    let theirs = t.graveyard(P1, "Gray Ogre");
    t.answer_yes(P1, true);
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Tempt with Immortality");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(mine.iter().filter(|c| t.on_battlefield(**c)).count(), 2);
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.obj_now(theirs).controller, P1);
}

#[test]
fn tempt_with_reflections_copies_the_target_for_each_player() {
    cr!("101.4", "111.10");
    supported("Tempt with Reflections");
    // "Choose target creature you control. Create a token that's a copy of that creature.
    // Each opponent may create a token that's a copy of that creature. For each opponent
    // who does, create a token that's a copy of that creature."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Tempt with Reflections");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    let copies = |p: PlayerId| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.is_token() && o.chars.name == "Grizzly Bears")
            .count()
    };
    assert_eq!((copies(P0), copies(P1), copies(P2)), (2, 1, 0));
}

#[test]
fn tempt_with_discovery_searches_for_each_player_who_accepted() {
    cr!("101.4", "701.23a");
    supported("Tempt with Discovery");
    // "Search your library for a land card and put it onto the battlefield. Each opponent
    // may search their library for a land card and put it onto the battlefield. For each
    // opponent who searches a library this way, search your library for a land card and
    // put it onto the battlefield. Then each player who searched a library this way
    // shuffles."
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        for _ in 0..3 {
            t.library_top(p, "Mountain");
        }
    }
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Tempt with Discovery");
    t.cast(P0, spell).go();
    t.resolve_all();
    let mountains = |p: PlayerId| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.chars.name == "Mountain")
            .count()
    };
    assert_eq!((mountains(P0), mountains(P1), mountains(P2)), (2, 0, 1));
}

#[test]
fn tempt_with_bunnies_draws_and_makes_rabbits_for_each_player() {
    cr!("101.4");
    supported("Tempt with Bunnies");
    // "Draw a card and create a 1/1 white Rabbit creature token. Then each opponent may
    // draw a card and create a 1/1 white Rabbit creature token. For each opponent who
    // does, you draw a card and you create a 1/1 white Rabbit creature token."
    let mut t = TestGame::new(3);
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Tempt with Bunnies");
    let hands: Vec<usize> = [P0, P1, P2].iter().map(|p| t.hand_size(*p)).collect();
    t.cast(P0, spell).go();
    t.resolve_all();
    let rabbits: Vec<usize> = [P0, P1, P2]
        .iter()
        .map(|p| with_subtype(&t, *p, "Rabbit").len())
        .collect();
    assert_eq!(rabbits, vec![3, 1, 1]);
    // P0 cast the spell (one card fewer) and drew three.
    assert_eq!(t.hand_size(P0), hands[0] - 1 + 3);
    assert_eq!(t.hand_size(P1), hands[1] + 1);
    assert_eq!(t.hand_size(P2), hands[2] + 1);
}

#[test]
fn tempting_contract_gives_you_a_treasure_for_each_opponent_who_takes_one() {
    cr!("101.4", "111.10a");
    supported("Tempting Contract");
    // "At the beginning of your upkeep, each opponent may create a Treasure token. For
    // each opponent who does, you create a Treasure token."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Tempting Contract");
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    let treasures: Vec<usize> = [P0, P1, P2]
        .iter()
        .map(|p| with_subtype(&t, *p, "Treasure").len())
        .collect();
    assert_eq!(treasures, vec![1, 1, 0]);
}

#[test]
fn each_opponent_may_put_a_card_from_their_hand_onto_the_battlefield() {
    cr!("101.4");
    supported("Iwamori of the Open Fist");
    // "When Iwamori enters, each opponent may put a legendary creature card from their
    // hand onto the battlefield."
    let mut t = TestGame::new(3);
    let isamaru = t.hand(P1, "Isamaru, Hound of Konda");
    let other = t.hand(P2, "Isamaru, Hound of Konda");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(isamaru)]);
    t.answer_yes(P2, false);
    t.enter(P0, "Iwamori of the Open Fist");
    t.resolve_all();
    assert!(t.on_battlefield(isamaru));
    assert_eq!(t.obj_now(isamaru).controller, P1);
    assert!(t.in_hand(P2, "Isamaru, Hound of Konda"));
    assert!(!t.on_battlefield(other));
}
