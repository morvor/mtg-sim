//! cards6-per-player-results: each player's own result of a per-player step (Guff Rewrites
//! History, Rootweaver Druid), the players whose card has the greatest mana value with
//! their tie loop (Timesifter), a number chosen once per number and guessed by an opponent
//! (The Toymaker's Trap), and a paid loop over the cards looked at (Lim-Dûl's Vault).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The card compiles fully.
fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The names of the top `n` cards of `p`'s library, top first.
fn top_names(t: &TestGame, p: PlayerId, n: usize) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .take(n)
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn names(v: &[&str]) -> Vec<String> {
    sorted(v.iter().map(|s| s.to_string()).collect())
}

/// Puts these cards on top of `p`'s library, the first on top.
fn stack_library(t: &mut TestGame, p: PlayerId, cards: &[&str]) {
    for c in cards.iter().rev() {
        t.library_top(p, c);
    }
}

// ---------------------------------------------------------------------------
// Lim-Dûl's Vault
// ---------------------------------------------------------------------------

const FIRST: [&str; 5] = ["Forest", "Island", "Swamp", "Mountain", "Plains"];
const SECOND: [&str; 5] = [
    "Grizzly Bears",
    "Llanowar Elves",
    "Shock",
    "Giant Growth",
    "Divination",
];

fn cast_vault(t: &mut TestGame) {
    t.lands(P0, "Underground Sea", 2);
    let vault = t.hand(P0, "Lim-Dûl's Vault");
    t.cast(P0, vault).go();
    t.resolve_all();
}

#[test]
fn lim_duls_vault_without_paying_life_puts_the_five_back_on_top() {
    cr!("608.2c", "701.24g");
    ruling!(
        "Lim-Dûl's Vault",
        "If you pay no life, you shuffle the rest of your library and then put the top five cards back on top in any order."
    );
    compiles("Lim-Dûl's Vault");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &FIRST);
    let lib = t.library_size(P0);
    t.answer_yes(P0, false);
    cast_vault(&mut t);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(sorted(top_names(&t, P0, 5)), names(&FIRST));
}

#[test]
fn lim_duls_vault_pays_life_to_look_at_the_next_five() {
    cr!("608.2c", "119.4");
    ruling!("Lim-Dûl's Vault", "Step 3b: Put those five cards on the bottom of your library in the order you want. Pay 1 life. Return to step 1.");
    compiles("Lim-Dûl's Vault");
    let mut t = TestGame::new(2);
    let mut cards: Vec<&str> = FIRST.to_vec();
    cards.extend(SECOND);
    stack_library(&mut t, P0, &cards);
    // Pay once (the first five go to the bottom), then keep the second five.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_vault(&mut t);
    assert_eq!(t.life(P0), 19);
    assert_eq!(sorted(top_names(&t, P0, 5)), names(&SECOND));
    // The first five were put on the bottom, then shuffled with the rest of the library.
    let all: Vec<String> =
        t.g.player(P0)
            .library
            .iter()
            .map(|id| t.g.obj(*id).chars.name.to_string())
            .collect();
    for c in FIRST {
        assert!(all.contains(&c.to_string()), "{c} stays in the library");
    }
}

#[test]
fn lim_duls_vault_can_pay_only_as_much_life_as_it_has() {
    cr!("119.4");
    compiles("Lim-Dûl's Vault");
    let mut t = TestGame::new(2);
    let mut cards: Vec<&str> = FIRST.to_vec();
    cards.extend(SECOND);
    stack_library(&mut t, P0, &cards);
    t.g.players[0].life = 1;
    // The second offer can't be accepted: 0 life is less than 1 (CR 119.4).
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_vault(&mut t);
    assert_eq!(t.life(P0), 0);
    assert_eq!(sorted(top_names(&t, P0, 5)), names(&SECOND));
}

#[test]
fn lim_duls_vault_with_five_cards_keeps_seeing_the_same_cards() {
    cr!("608.2c");
    ruling!(
        "Lim-Dûl's Vault",
        "If you have five or fewer cards in your library, you can pay 1 life as many times as you like, but you'll keep seeing the same cards."
    );
    compiles("Lim-Dûl's Vault");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack_library(&mut t, P0, &FIRST);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_vault(&mut t);
    assert_eq!(t.life(P0), 18);
    assert_eq!(sorted(top_names(&t, P0, 5)), names(&FIRST));
    assert_eq!(t.library_size(P0), 5);
    // Each pass offered the same five cards' look again.
    let offers = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(offers, 3);
}

// ---------------------------------------------------------------------------
// The Toymaker's Trap
// ---------------------------------------------------------------------------

/// The labels of the last option decision `p` was asked.
fn last_options(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. } if *q == p => Some(options.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Answers the option decision labelled `label` for `p`, given the labels it will offer.
fn pick(t: &mut TestGame, p: PlayerId, labels: &[&str], label: &str) {
    let i = labels
        .iter()
        .position(|l| *l == label)
        .expect("an offered label");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn the_toymakers_trap_wrong_guess_costs_the_guessed_life_and_draws() {
    cr!("608.2c");
    interp!("number-guess-range");
    compiles("The Toymaker's Trap");
    let mut t = TestGame::new(2);
    let trap = t.battlefield(P0, "The Toymaker's Trap");
    // P0 chooses 2; P1 guesses 4.
    pick(&mut t, P0, &["1", "2", "3", "4", "5"], "2");
    pick(&mut t, P1, &["1", "2", "3", "4", "5"], "4");
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(trap));
    // The guess offered every number of the range.
    assert_eq!(last_options(&t, P1), ["1", "2", "3", "4", "5"]);
    // The chosen number was revealed.
    assert!(t.g.obj(trap).choices.secret.is_empty());
    assert_eq!(t.g.obj(trap).choices.number, Some(2));

    // Next upkeep: 2 has been chosen. P0 chooses 3; P1 guesses right.
    pick(&mut t, P0, &["1", "3", "4", "5"], "3");
    pick(&mut t, P1, &["1", "2", "3", "4", "5"], "3");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(last_options(&t, P0), ["1", "3", "4", "5"]);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "The Toymaker's Trap"));
}

#[test]
fn the_toymakers_trap_remembers_numbers_only_for_its_own_object() {
    cr!("608.2c");
    ruling!(
        "The Toymaker's Trap",
        "it has no memory of which numbers were chosen for its effect before. The same is true if you control a second copy of it."
    );
    interp!("toymakers-trap-every-number-chosen");
    compiles("The Toymaker's Trap");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "The Toymaker's Trap");
    // Every number has been chosen for the first one: it does nothing ("If you do" fails).
    t.g.modal_history.numbers.insert(old, vec![1, 2, 3, 4, 5]);
    let fresh = t.battlefield(P0, "The Toymaker's Trap");
    // The second copy offers all five numbers; P1 guesses wrong (1 against 5).
    pick(&mut t, P0, &["1", "2", "3", "4", "5"], "5");
    pick(&mut t, P1, &["1", "2", "3", "4", "5"], "1");
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(old) && t.on_battlefield(fresh));
    // Only one number choice and one guess were asked.
    let p1_guesses = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. }))
        .count();
    assert_eq!(p1_guesses, 1);
    assert_eq!(last_options(&t, P0), ["1", "2", "3", "4", "5"]);
}

#[test]
fn the_toymakers_trap_that_left_and_returned_has_no_memory_of_its_numbers() {
    cr!("400.7");
    ruling!(
        "The Toymaker's Trap",
        "If The Toymaker's Trap leaves the battlefield and later returns to the battlefield, it has no memory of which numbers were chosen for its effect before."
    );
    compiles("The Toymaker's Trap");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "The Toymaker's Trap");
    t.g.modal_history.numbers.insert(old, vec![1, 2, 3, 4]);
    // It's exiled and returns: a new object (CR 400.7).
    let mv = |t: &mut TestGame, id: ObjectId, to: mtg_engine::object::Zone| {
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .expect("moved")
    };
    let exiled = mv(&mut t, old, mtg_engine::object::Zone::Exile);
    let back = mv(&mut t, exiled, mtg_engine::object::Zone::Battlefield);
    pick(&mut t, P0, &["1", "2", "3", "4", "5"], "1");
    pick(&mut t, P1, &["1", "2", "3", "4", "5"], "2");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(last_options(&t, P0), ["1", "2", "3", "4", "5"]);
    assert!(t.on_battlefield(back));
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// Timesifter
// ---------------------------------------------------------------------------

#[test]
fn timesifter_gives_the_extra_turn_to_the_player_with_the_greatest_mana_value() {
    cr!("500.7", "608.2c");
    compiles("Timesifter");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Timesifter");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Shock");
    t.library_top(P2, "Forest");
    // P1's upkeep: P0's Bears (2) beats Shock (1) and Forest (0).
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Shock") && t.in_exile("Forest"));
    assert_eq!(t.g.extra_turns, vec![P0]);
    // The extra turn comes right after this one, before P2's.
    assert!(t.g.run_until(10_000, |g| g.turn.active != P1));
    assert_eq!(t.g.turn.active, P0);
}

#[test]
fn timesifter_tied_players_repeat_until_the_tie_is_broken() {
    cr!("500.7", "608.2c");
    ruling!("Timesifter", "Each time there's a tie for the highest cost, only the players involved in the tie continue.");
    ruling!("Timesifter", "A land card has a mana value of 0.");
    interp!("greatest-card-unique-player");
    compiles("Timesifter");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Timesifter");
    // Pass 1: 2, 2, 1 (P0 and P1 tie). Pass 2: 1, 1 (tie). Pass 3: land (0) against 3.
    stack_library(&mut t, P0, &["Grizzly Bears", "Shock", "Forest"]);
    stack_library(&mut t, P1, &["Grizzly Bears", "Giant Growth", "Divination"]);
    stack_library(&mut t, P2, &["Lightning Bolt", "Mind Rot"]);
    let (l0, l1, l2) = (t.library_size(P0), t.library_size(P1), t.library_size(P2));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.library_size(P0), l0 - 3);
    assert_eq!(t.library_size(P1), l1 - 3);
    // P2 wasn't tied: only one card.
    assert_eq!(t.library_size(P2), l2 - 1);
    assert_eq!(t.g.extra_turns, vec![P1]);
}

#[test]
fn timesifter_a_player_who_exiles_no_card_cant_take_the_turn() {
    cr!("500.7");
    ruling!(
        "Timesifter",
        "If a player fails to exile a card, they can't take the extra turn."
    );
    compiles("Timesifter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Timesifter");
    t.g.players[0].library.clear();
    t.library_top(P1, "Forest");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // P1's land (0) is the only card exiled.
    assert_eq!(t.g.extra_turns, vec![P1]);
}

#[test]
fn two_timesifters_create_two_extra_turns() {
    cr!("500.7");
    ruling!(
        "Timesifter",
        "With two Timesifters on the battlefield, two extra turns are created for each turn taken."
    );
    compiles("Timesifter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Timesifter");
    t.battlefield(P1, "Timesifter");
    stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    stack_library(&mut t, P1, &["Forest", "Divination"]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // P0's Bears (2) beat the Forest; then Divination (3) beats Shock.
    let mut turns = t.g.extra_turns.clone();
    turns.sort_by_key(|p| p.0);
    assert_eq!(turns, vec![P0, P1]);
}

// ---------------------------------------------------------------------------
// Guff Rewrites History
// ---------------------------------------------------------------------------

/// Replaces `p`'s library with these cards (the first on top).
fn set_library(t: &mut TestGame, p: PlayerId, cards: &[&str]) {
    t.g.players[p.idx()].library.clear();
    stack_library(t, p, cards);
}

#[test]
fn guff_rewrites_history_each_controller_casts_the_card_they_exiled() {
    cr!("101.4", "608.2c", "608.2g", "608.2h");
    interp!("guff-players-who-controlled");
    compiles("Guff Rewrites History");
    let mut t = TestGame::new(3);
    // P0 controls P1's Llanowar Elves; P1 controls its own Grizzly Bears; P2 controls
    // only a land (no legal target for P2).
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.objects[elves.0 as usize].base_controller = P0;
    t.g.recompute();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Forest");
    set_library(&mut t, P0, &["Plains", "Plains", "Divination", "Plains"]);
    set_library(&mut t, P1, &["Island", "Island"]);
    set_library(&mut t, P2, &["Mountain", "Mountain", "Shock"]);
    t.lands(P0, "Mountain", 3);
    let guff = t.hand(P0, "Guff Rewrites History");
    // A target for P0 (the Elves it controls) and one for P1 (its Bears).
    t.cast(P0, guff).target(elves).target(bears).go();
    t.resolve();
    // Both creatures went into P1's library; P0 (who controlled the Elves) and P1 each
    // exiled until a nonland card: P0 found Divination, P1 one of its two creatures.
    assert!(!t.on_battlefield(elves) && !t.on_battlefield(bears));
    // P2 controlled none of those permanents: its library is untouched.
    assert_eq!(t.library_size(P2), 3);
    // The lands exiled before the nonland cards went to the bottom.
    assert_eq!(t.library_size(P0), 3);
    assert!(!t.in_exile("Plains"));
    // P0 cast Divination for free, P1 a creature.
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), 2);
    let p1_creatures =
        t.g.battlefield
            .iter()
            .filter(|id| {
                t.g.obj(**id).controller == P1
                    && t.g
                        .obj(**id)
                        .chars
                        .is(mtg_engine::types::CardType::Creature)
            })
            .count();
    assert_eq!(p1_creatures, 1);
}

// ---------------------------------------------------------------------------
// Rootweaver Druid
// ---------------------------------------------------------------------------

#[test]
fn rootweaver_druid_each_opponent_splits_the_lands_they_find() {
    cr!("101.4", "101.4b", "701.23a");
    ruling!(
        "Rootweaver Druid",
        "If they find only one, they put it onto the battlefield tapped under your control."
    );
    ruling!("Rootweaver Druid", "Each opponent will know what players before them did when deciding what they are going to do.");
    compiles("Rootweaver Druid");
    let mut t = TestGame::new(3);
    let f1 = t.library_top(P1, "Forest");
    let f2 = t.library_top(P1, "Forest");
    let f3 = t.library_top(P1, "Forest");
    let p2_land = t.library_top(P2, "Island");
    // P1 searches for three Forests and gives P0 the first; P2 searches for one Island.
    t.answer_yes(P1, true);
    t.answer_choose(
        P1,
        &[Entity::Object(f1), Entity::Object(f2), Entity::Object(f3)],
    );
    t.answer_choose(P1, &[Entity::Object(f1)]);
    t.answer_yes(P2, true);
    t.answer_choose(P2, &[Entity::Object(p2_land)]);
    let from = t.asked().len();
    t.enter(P0, "Rootweaver Druid");
    t.resolve_all();
    let lands = |t: &TestGame, ctl: PlayerId, owner: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| {
                let o = t.g.obj(**id);
                o.controller == ctl && o.owner == owner && o.chars.is_land() && o.tapped
            })
            .count()
    };
    assert_eq!(lands(&t, P0, P1), 1);
    assert_eq!(lands(&t, P1, P1), 2);
    // P2 found only one: it's yours.
    assert_eq!(lands(&t, P0, P2), 1);
    assert_eq!(lands(&t, P2, P2), 0);
    // P1 decided, searched and split before P2 decided.
    let asked: Vec<(PlayerId, Decision)> = t.asked()[from..].to_vec();
    let p1_split = asked
        .iter()
        .rposition(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. }))
        .expect("P1 split its cards");
    let p2_decides = asked
        .iter()
        .position(|(p, d)| *p == P2 && matches!(d, Decision::YesNo { .. }))
        .expect("P2 decided");
    assert!(p1_split < p2_decides);
}

#[test]
fn rootweaver_druid_only_the_players_who_searched_shuffle() {
    cr!("701.24a");
    ruling!(
        "Rootweaver Druid",
        "Then each player who searched their library shuffles it."
    );
    compiles("Rootweaver Druid");
    let mut t = TestGame::new(3);
    t.library_top(P1, "Forest");
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    let p2_before: Vec<ObjectId> = t.g.player(P2).library.clone();
    let p1_before: Vec<ObjectId> = t.g.player(P1).library.clone();
    t.enter(P0, "Rootweaver Druid");
    t.resolve_all();
    // P1 didn't search: its library is as it was.
    assert_eq!(t.g.player(P1).library, p1_before);
    // P2 searched (finding nothing) and shuffled.
    let mut a = t.g.player(P2).library.clone();
    let mut b = p2_before.clone();
    assert_ne!(a, b, "P2's library was shuffled");
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

#[test]
fn rootweaver_druid_on_an_opponents_turn_that_opponent_decides_first() {
    cr!("101.4");
    ruling!("Rootweaver Druid", "first the next opponent in turn order (or, if it's an opponent's turn, the opponent whose turn it is) decides whether or not to search their library");
    compiles("Rootweaver Druid");
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    t.answer_yes(P1, false);
    t.answer_yes(P2, false);
    let from = t.asked().len();
    t.enter(P0, "Rootweaver Druid");
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P2, P1]);
}
