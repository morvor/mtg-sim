//! Modal spells and abilities with headers beyond "Choose one —" (CR 700.2): modes with
//! flavor words (CR 207.2d), "choose up to one", "choose one at random", "choose one that
//! hasn't been chosen [this turn]", and "Choose one. If [condition], you may choose both
//! instead."

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for name in names {
        let c = card(name);
        assert!(
            c.unsupported_text().is_empty(),
            "{name} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// The chosen mode indices of a spell or ability on the stack.
fn modes_of(t: &TestGame, id: ObjectId) -> Vec<usize> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| si.chosen.iter().filter_map(|c| c.mode).collect())
        .unwrap_or_default()
}

fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

fn choose_modes(t: &mut TestGame, p: PlayerId, modes: &[usize]) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
}

/// (min, max) of the last "choose modes" decision asked of `p`.
fn last_mode_bounds(t: &TestGame, p: PlayerId) -> (u32, u32) {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseModes { min, max, .. } if *q == p => Some((*min, *max)),
            _ => None,
        })
        .expect("no mode choice asked")
}

/// Advances to P0's next upkeep (leaving the current one first).
fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

fn mode_choices_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::ChooseModes { .. }))
        .count()
}

// ---------------------------------------------------------------------------
// Flavor words on modes (CR 207.2d)
// ---------------------------------------------------------------------------

#[test]
fn flavor_words_on_modes_have_no_rules_meaning() {
    cr!("207.2d", "700.2a");
    assert_supported(&[
        "Battle Menu",
        "Megaton's Fate",
        "Dawnbringer Cleric",
        "You See a Pair of Goblins",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    // "• Item — You gain 4 life."
    let menu = t.hand(P0, "Battle Menu");
    let s = t.cast(P0, menu).modes(&[3]).go();
    assert_eq!(modes_of(&t, s), vec![3]);
    t.resolve();
    assert_eq!(t.life(P0), 24);
    // "• Attack — Create a 2/2 white Knight creature token."
    let menu = t.hand(P0, "Battle Menu");
    t.cast(P0, menu).modes(&[0]).go();
    t.resolve();
    let knights = t.named_on_battlefield("Knight Token");
    assert_eq!(knights.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(knights[0]), (2, 2));
}

#[test]
fn flavor_worded_mode_of_a_trigger_refers_to_the_triggering_creature() {
    cr!("207.2d", "700.2b");
    assert_supported(&["Pip-Boy 3000", "The Spear of Leonidas"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let pip = t.battlefield(P0, "Pip-Boy 3000");
    t.g.attach(pip, Entity::Object(bear));
    // "Whenever equipped creature attacks, choose one — ... • Pick a Perk — Put a +1/+1
    // counter on that creature."
    choose_modes(&mut t, P0, &[1]);
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bear, "+1/+1"), 1, "{}", t.dump_log());
    assert_eq!(t.counters(other, "+1/+1"), 0);
    assert_eq!(t.counters(pip, "+1/+1"), 0);
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------
// "Choose up to one —"
// ---------------------------------------------------------------------------

#[test]
fn choose_up_to_one_allows_choosing_no_mode() {
    cr!("700.2b");
    assert_supported(&["Sawblade Slinger", "Hullbreaker Horror", "Ravager Wurm"]);
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Ornithopter");
    // No mode chosen: the ability is removed from the stack and does nothing.
    choose_modes(&mut t, P0, &[]);
    t.enter(P0, "Sawblade Slinger");
    t.settle();
    assert_eq!(last_mode_bounds(&t, P0), (0, 1));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(relic));
    // "• Destroy target artifact an opponent controls."
    choose_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[Entity::Object(relic)]);
    t.enter(P0, "Sawblade Slinger");
    t.resolve_all();
    assert!(!t.g.is_live(relic));
    assert_eq!(last_mode_bounds(&t, P0).0, 0);
}

// ---------------------------------------------------------------------------
// "Choose one at random —"
// ---------------------------------------------------------------------------

#[test]
fn a_random_mode_is_chosen_as_the_trigger_is_put_on_the_stack() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Umaro, Raging Yeti",
        "As you put Umaro's triggered ability on the stack, you choose a mode at random."
    );
    assert_supported(&["Umaro, Raging Yeti", "Cult of Skaro", "Summon: Magus Sisters"]);
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..8 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        t.battlefield(P0, "Umaro, Raging Yeti");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        // Players can respond knowing which mode was chosen; nobody chose it.
        let ability = top(&t);
        let modes = modes_of(&t, ability);
        assert_eq!(modes.len(), 1);
        assert_eq!(mode_choices_asked(&t, P0), 0);
        seen.insert(modes[0]);
        let hand = t.hand_size(P0);
        t.resolve();
        match modes[0] {
            1 => assert_eq!(t.hand_size(P0), 4, "hand was {hand}"),
            2 => assert_eq!(t.life(P1), 15),
            _ => assert_eq!(t.life(P1), 20),
        }
    }
    assert!(seen.len() >= 2, "the mode is random: {seen:?}");
}

// ---------------------------------------------------------------------------
// "Choose one that hasn't been chosen —"
// ---------------------------------------------------------------------------

#[test]
fn demonic_pact_modes_are_used_up_until_only_losing_remains() {
    cr!("700.2b");
    ruling!(
        "Demonic Pact",
        "you may not be able to choose a mode, either because all modes have previously been chosen"
    );
    assert_supported(&["Demonic Pact", "Kimoyo Beads"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demonic Pact");
    // First upkeep: "Draw two cards."
    choose_modes(&mut t, P0, &[2]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(modes_of(&t, top(&t)), vec![2]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Second upkeep: "Draw two cards" again isn't a legal choice.
    choose_modes(&mut t, P0, &[2]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    next_upkeep(&mut t);
    t.settle();
    let modes = modes_of(&t, top(&t));
    assert_eq!(modes.len(), 1);
    assert_ne!(modes, vec![2]);
    t.resolve();
    // Third upkeep: the remaining one of the first two modes.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    next_upkeep(&mut t);
    t.settle();
    let third = modes_of(&t, top(&t));
    assert_ne!(third, modes);
    assert_ne!(third, vec![2]);
    assert_ne!(third, vec![3]);
    t.resolve();
    assert!(!t.has_lost(P0));
    // Fourth upkeep: only "You lose the game." is left.
    next_upkeep(&mut t);
    t.settle();
    assert_eq!(modes_of(&t, top(&t)), vec![3]);
    t.resolve();
    assert!(t.has_lost(P0));
}

#[test]
fn each_demonic_pact_tracks_its_own_modes() {
    cr!("700.2b", "400.7");
    ruling!(
        "Demonic Pact",
        "refers only to that specific Demonic Pact"
    );
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Demonic Pact");
    choose_modes(&mut t, P0, &[2]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    // A second Pact can still choose "Draw two cards"; the first can't.
    t.battlefield(P0, "Demonic Pact");
    choose_modes(&mut t, P0, &[2]);
    choose_modes(&mut t, P0, &[2]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    next_upkeep(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let mut drew = 0;
    for id in t.g.stack.clone() {
        let source = match &t.g.obj(id).stack.as_ref().unwrap().kind {
            mtg_engine::object::StackKind::Triggered { source, .. } => *source,
            _ => unreachable!(),
        };
        let modes = modes_of(&t, id);
        if source == first {
            assert_ne!(modes, vec![2]);
        } else {
            assert_eq!(modes, vec![2]);
            drew += 1;
        }
    }
    assert_eq!(drew, 1);
}

#[test]
fn modes_not_chosen_this_turn_reset_each_turn() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Lita, Little Orphan Amphibian",
        "If you can't legally choose a mode because all three have been chosen that turn, that instance of the ability is removed from the stack with no effect."
    );
    assert_supported(&[
        "Galadriel, Light of Valinor",
        "Lita, Little Orphan Amphibian",
        "Monument to Endurance",
        "Teval's Judgment",
    ]);
    let mut t = TestGame::new(2);
    let gal = t.battlefield(P0, "Galadriel, Light of Valinor");
    // "• Put a +1/+1 counter on each creature you control."
    choose_modes(&mut t, P0, &[1]);
    let b1 = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(gal, "+1/+1"), 1);
    assert_eq!(t.counters(b1, "+1/+1"), 1);
    // The same mode can't be chosen again this turn.
    choose_modes(&mut t, P0, &[1]);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_ne!(modes_of(&t, top(&t)), vec![1]);
    t.resolve_all();
    assert_eq!(t.counters(gal, "+1/+1"), 1);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    // All three have been chosen this turn: the fourth trigger is removed.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(gal, "+1/+1"), 1);
    // Next turn, every mode can be chosen again.
    t.advance_to(P1, Step::Upkeep);
    choose_modes(&mut t, P0, &[1]);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(modes_of(&t, top(&t)), vec![1]);
    t.resolve_all();
    assert_eq!(t.counters(gal, "+1/+1"), 2);
}

// ---------------------------------------------------------------------------
// "Choose one. If [condition], you may choose both instead."
// ---------------------------------------------------------------------------

#[test]
fn a_condition_as_you_cast_lets_you_choose_more_modes() {
    cr!("601.2b", "700.2a");
    ruling!(
        "Flame of Anor",
        "it doesn't matter what happens to the Wizard you control in response"
    );
    assert_supported(&[
        "Flame of Anor",
        "Akroma's Will",
        "Klauth's Will",
        "Molten Collapse",
        "Let's Play a Game",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // Without a Wizard: only one mode.
    let flame = t.hand(P0, "Flame of Anor");
    let s = t
        .cast(P0, flame)
        .modes(&[0, 2])
        .target(Entity::Player(P0))
        .go();
    assert_eq!(last_mode_bounds(&t, P0), (1, 1));
    assert_eq!(modes_of(&t, s).len(), 1);
    t.resolve();
    // With a Wizard: two modes, and losing the Wizard in response doesn't matter.
    let wizard = t.battlefield(P0, "Prodigal Sorcerer");
    let flame = t.hand(P0, "Flame of Anor");
    let hand = t.hand_size(P0);
    let s = t
        .cast(P0, flame)
        .modes(&[0, 2])
        .target(Entity::Player(P0))
        .target(giant)
        .go();
    assert_eq!(last_mode_bounds(&t, P0), (1, 2));
    assert_eq!(modes_of(&t, s), vec![0, 2]);
    t.g.destroy(wizard, None);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(!t.g.is_live(giant));
}

#[test]
fn kicked_inscription_chooses_any_number_of_modes() {
    cr!("601.2b", "702.33d");
    ruling!(
        "Inscription of Ruin",
        "If you kick Inscription of Ruin, you can't choose any one mode more than once."
    );
    assert_supported(&["Inscription of Ruin"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Hill Giant");
    // Unkicked: one mode.
    let ins = t.hand(P0, "Inscription of Ruin");
    t.cast(P0, ins)
        .kicked(false)
        .modes(&[0, 2])
        .target(Entity::Player(P1))
        .go();
    assert_eq!(last_mode_bounds(&t, P0), (1, 1));
    t.resolve();
    t.lands(P0, "Swamp", 3);
    for l in t.g.battlefield.clone() {
        t.g.objects[l.0 as usize].tapped = false;
    }
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Hill Giant");
    let hand = t.hand_size(P1);
    // Kicked: "Target opponent discards two cards" and "Destroy target creature with mana
    // value 3 or less".
    let ins = t.hand(P0, "Inscription of Ruin");
    let s = t
        .cast(P0, ins)
        .kicked(true)
        .modes(&[0, 2])
        .target(Entity::Player(P1))
        .target(bear)
        .go();
    let (min, max) = last_mode_bounds(&t, P0);
    assert_eq!((min, max), (0, 3));
    assert_eq!(modes_of(&t, s), vec![0, 2]);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand - 2);
    assert!(!t.g.is_live(bear));
}

#[test]
fn a_trigger_checks_its_mode_condition_as_it_is_put_on_the_stack() {
    cr!("603.3c", "700.2b");
    assert_supported(&["Disciple of Perdition", "Prophetic Titan"]);
    let mut t = TestGame::new(2);
    let disciple = t.battlefield(P0, "Disciple of Perdition");
    t.g.players[0].life = 13;
    t.graveyard(P1, "Hill Giant");
    // "If you have exactly 13 life, you may choose both instead."
    choose_modes(&mut t, P0, &[0, 1]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    t.g.destroy(disciple, None);
    t.settle();
    assert_eq!(last_mode_bounds(&t, P0), (1, 2));
    assert_eq!(modes_of(&t, top(&t)), vec![0, 1]);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 12);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.graveyard_size(P1), 0);
}
