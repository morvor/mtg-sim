//! Rulings batch P029 — Fork's copy (red, same mana cost; no card; buyback, CR 707.10,
//! 707.10b, 702.27a), a copy of a spliced Arcane spell (CR 702.47c), copies of extra-turn
//! spells (CR 500.7), Tempt with Mayhem's target (CR 115.5), Lithoform Engine's copies of
//! abilities (CR 707.10, 113.7, 603.1, 702.108a), Myra the Magnificent's target (CR 115.1),
//! Unbound Flourishing and {X} in additional costs (CR 107.3, 202.1), and turning a face-down
//! instant or sorcery card face up (CR 708.8).

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{AbilityKind, Modification};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Fork ------------------------------------------------------------------------------------

#[test]
fn fork_s_copy_has_the_original_s_mana_cost_but_is_red() {
    cr!("707.10", "105.2", "202.1");
    ruling!(
        "Fork",
        "It does copy the mana symbols in the mana cost for the card it is copying, but it uses its own color definition and not the one from those mana symbols. This is so it maintains its color just like the text says."
    );
    ruling!(
        "Fork",
        "The copy will have the same mana cost as the original spell, but will be red rather than whatever color that mana cost would have made it."
    );
    supported("Fork");
    let mut t = TestGame::new(2);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    (FORK.start)(&mut t, div);
    let copies = resolve_until_copies(&mut t);
    let copy = t.obj(copies[0]);
    assert_eq!(
        copy.chars.mana_cost.as_ref().map(|c| c.to_string()),
        Some("{2}{U}".to_string())
    );
    assert_eq!(copy.chars.colors, ColorSet::single(Color::Red));
    assert!(t.obj(div).chars.colors.contains(Color::Blue));
    assert_eq!(t.g.mana_value_of(copies[0]), 3);
}

#[test]
fn fork_goes_to_the_graveyard_and_its_copy_has_no_card() {
    cr!("707.10", "608.2n", "704.5e");
    ruling!(
        "Fork",
        "The Fork card goes to the graveyard when it resolves and leaves the copy on the stack. There is no card representing the copy on the stack."
    );
    let mut t = TestGame::new(2);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    (FORK.start)(&mut t, div);
    t.resolve();
    assert!(t.in_graveyard(P0, "Fork"));
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).kind, ObjKind::SpellCopy);
    t.resolve();
    assert!(!t.g.stack.contains(&copies[0]));
    // The copy ceased to exist: only Fork in the graveyard; Divination is still on the
    // stack.
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.g.stack, vec![div]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn a_fork_copy_of_a_buyback_spell_returns_nothing() {
    cr!("702.27a", "707.10", "608.2n");
    ruling!(
        "Fork",
        "If you copy a spell for which Buyback has been paid, you get nothing back since the copied spell does not have a card to give you."
    );
    supported("Searing Touch");
    // Searing Touch: "Buyback {4}. Searing Touch deals 1 damage to any target." Bought back.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Searing Touch");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let touch = t.cast(P0, card).target(P1).go();
    (FORK.start)(&mut t, touch);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The Searing Touch card came back; the copy brought nothing.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.g.find_in_zone(Zone::Hand(P0), "Searing Touch").len(), 1);
    assert!(t.in_graveyard(P0, "Fork"));
}

// --- Uyo, Silent Prophet --------------------------------------------------------------------

#[test]
fn uyo_s_copy_of_a_spliced_arcane_spell_has_the_spliced_text() {
    cr!("702.47c", "707.10");
    ruling!(
        "Uyo, Silent Prophet",
        "If you copy an Arcane spell that has cards spliced onto it, you’ll copy the spliced text as well as the original spell text."
    );
    supported("Uyo, Silent Prophet");
    supported("Kodama's Might");
    supported("Glacial Ray");
    // Kodama's Might (Arcane: "Target creature gets +2/+2 until end of turn.") with Glacial
    // Ray ("Glacial Ray deals 2 damage to any target. Splice onto Arcane {1}{R}") spliced
    // onto it, at the Bears and P1.
    let mut t = TestGame::new(2);
    (UYO.setup)(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Glacial Ray");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P0, might).target(bears).target(P1).go();
    (UYO.start)(&mut t, spell);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    let spell_abilities = |t: &TestGame, id: ObjectId| {
        t.obj(id)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Spell(_)))
            .count()
    };
    assert_eq!(spell_abilities(&t, copies[0]), 2);
    assert_eq!(
        targets_of(&t, copies[0]),
        vec![obj(bears), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(t.life(P1), 16);
}

// --- Tempt with Mayhem -------------------------------------------------------------------------

#[test]
fn copies_of_an_extra_turn_spell_add_turns_one_at_a_time_most_recent_first() {
    cr!("500.7", "707.10");
    ruling!(
        "Tempt with Mayhem",
        "In the case where the spell being copied grants one or more players extra turns, those extra turns are added one at a time. When the current turn ends, the most recently created turn will be taken first."
    );
    supported("Time Walk");
    // P0's Time Walk ("Take an extra turn after this one."); P0's Tempt with Mayhem: P1
    // copies it, so P0 copies it twice.
    let mut t = TestGame::new(2);
    let walk = cast_new(&mut t, P0, "Time Walk", &[]);
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Tempt with Mayhem", &[obj(walk)]);
    for _ in 0..3 {
        keep_copy_targets(&mut t, P0);
        keep_copy_targets(&mut t, P1);
    }
    t.resolve();
    // Who takes each extra turn, in the order the Time Walks resolve.
    let mut resolved = vec![];
    while let Some(&top) = t.g.stack.last() {
        if t.g.obj(top).chars.name == "Time Walk" {
            resolved.push(t.g.obj(top).controller);
        }
        t.resolve();
    }
    let mut want = resolved.clone();
    assert_eq!(want.len(), 4);
    assert_eq!(want.iter().filter(|p| **p == P1).count(), 1);
    want.reverse();
    // The turns taken after this one: the last created first.
    let mut taken = vec![];
    for _ in 0..4 {
        let turn = t.g.turn.number;
        let ok = t.g.run_until(10_000, |g| {
            g.turn.number > turn && g.turn.step == Step::Upkeep
        });
        assert!(ok);
        taken.push(t.g.turn.active);
    }
    assert_eq!(taken, want);
}

#[test]
fn tempt_with_mayhem_cant_target_itself() {
    cr!("115.5", "601.2c");
    ruling!(
        "Tempt with Mayhem",
        "Tempt with Mayhem can't target itself. There are limits, after all."
    );
    // With nothing else on the stack, it can't be cast.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Tempt with Mayhem");
    let card = t.hand(P0, "Tempt with Mayhem");
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card)));
    assert!(t.cast(P0, card).target(card).try_go().is_err() || t.g.stack.is_empty());
    // With a Lightning Bolt on the stack, it can target only the Bolt.
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    let from = t.asked().len();
    let tempt = t.cast(P0, card).target(bolt).go();
    let candidates: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!candidates.is_empty());
    for c in &candidates {
        assert_eq!(c, &vec![obj(bolt)]);
        assert!(!c.contains(&obj(tempt)));
    }
}

// --- Lithoform Engine ---------------------------------------------------------------------------

#[test]
fn lithoform_engine_copies_an_ability_on_the_stack_with_the_same_source() {
    cr!("707.10", "707.10b", "113.7");
    ruling!(
        "Lithoform Engine",
        "Lithoform Engine's first ability targets an activated or triggered ability that is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    ruling!(
        "Lithoform Engine",
        "The source of the copy from Lithoform Engine's first ability is the same as the source of the original ability."
    );
    supported("Prodigal Pyromancer");
    // Prodigal Pyromancer ("{T}: This creature deals 1 damage to any target.") with lifelink
    // pings P1; the Engine copies the ability: the copy's damage is dealt by the Pyromancer
    // too, so P0 gains 2 life in all.
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    modify_until_eot(
        &mut t,
        pyro,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Lifelink,
        ))],
    );
    let count = |t: &TestGame, id: ObjectId| t.obj_now(id).chars.abilities.len();
    let (n_engine, n_pyro) = (count(&t, engine), count(&t, pyro));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage")
        .unwrap()
        .unwrap();
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(ping)]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let from_pyro = abilities_from(&t, pyro);
    assert_eq!(from_pyro.len(), 2);
    let copy = *from_pyro.iter().find(|a| **a != ping).unwrap();
    assert_eq!(ability_source(&t, copy), Some(pyro));
    assert!(abilities_from(&t, engine).is_empty());
    // No object gained an ability.
    assert_eq!((count(&t, engine), count(&t, pyro)), (n_engine, n_pyro));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn lithoform_engine_copies_a_prowess_trigger() {
    cr!("603.1", "702.108a", "707.10");
    ruling!(
        "Lithoform Engine",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They're often written as “[Trigger condition], [effect].” Some keywords (such as prowess) are triggered abilities and will use “when,” “whenever,” or “at” in their reminder text."
    );
    supported("Monastery Swiftspear");
    // Monastery Swiftspear (1/2, prowess): Opt triggers prowess; the Engine copies the
    // trigger: +2/+2 in all.
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    let prowess = abilities_from(&t, spear);
    assert_eq!(prowess.len(), 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(prowess[0])]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(spear), (3, 4));
}

// --- Myra the Magnificent -------------------------------------------------------------------------

#[test]
fn myra_targets_only_the_card_and_chooses_the_attraction_on_resolution() {
    cr!("115.1", "608.2c");
    ruling!(
        "Myra the Magnificent",
        "Myra’s activated ability targets only the instant or sorcery card in your graveyard. It doesn’t target the Attraction. You choose the Attraction as the ability is resolving."
    );
    supported("Myra the Magnificent");
    // "{X}, {T}: Exile target instant or sorcery card with mana value X from your graveyard
    // and choose an Attraction you control that doesn't have a midway counter on it. Put a
    // midway counter on it. ..."
    let mut t = TestGame::new(2);
    let myra = t.battlefield(P0, "Myra the Magnificent");
    let tower = t.battlefield(P0, "Drop Tower");
    let wheel = t.battlefield(P0, "Ferris Wheel");
    let div = t.graveyard(P0, "Divination");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[obj(div)]);
    t.answer_choose(P0, &[obj(wheel)]);
    let from = t.asked().len();
    let ability = activate_containing(&mut t, P0, myra, "Exile target")
        .unwrap()
        .unwrap();
    assert_eq!(targets_of(&t, ability), vec![obj(div)]);
    let choices = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(tower))))
            .count()
    };
    // No Attraction chosen yet.
    assert_eq!(choices(&t), 0);
    t.resolve();
    assert_eq!(choices(&t), 1);
    assert!(t.in_exile("Divination"));
    assert_eq!(t.counters(wheel, "midway"), 1);
    assert_eq!(t.counters(tower, "midway"), 0);
}

// --- Unbound Flourishing --------------------------------------------------------------------------

#[test]
fn unbound_flourishing_ignores_x_in_additional_costs() {
    cr!("107.3", "202.1", "601.2b");
    ruling!(
        "Unbound Flourishing",
        "Spells with additional costs that include {X} won't be affected by Unbound Flourishing. {X} must be in the spell's mana cost."
    );
    supported("Waterbender's Restoration");
    // Waterbender's Restoration ({U}{U}; "As an additional cost to cast this spell,
    // waterbend {X}. Exile X target creatures you control. Return those cards to the
    // battlefield under their owner's control at the beginning of the next end step."),
    // X = 1: not copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    let card = t.hand(P0, "Waterbender's Restoration");
    let spell = t.cast(P0, card).x(1).target(bears).go();
    assert_eq!(x_of(&t, spell), Some(1));
    t.settle();
    assert_eq!(t.stack_len(), 1, "Unbound Flourishing didn't trigger");
    assert!(resolve_collecting_copies(&mut t).is_empty());
    assert!(t.in_exile("Grizzly Bears"));
    // {X} in the mana cost: Blaze is copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(1).target(P1).go();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- Magar of the Magic Strings ---------------------------------------------------------------------

/// Whether the face-down permanent `id` was revealed this turn.
fn revealed(t: &TestGame, id: ObjectId) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == mtg_engine::facedown::REVEALED && *o == id)
    })
}

#[test]
fn turning_magar_s_face_down_instant_face_up_reveals_it_and_it_stays_face_down() {
    cr!("708.8", "708.2");
    interp!("face-down-instant-or-sorcery-stays-face-down");
    ruling!(
        "Magar of the Magic Strings",
        "There are no cards in this set that would turn a face-down instant or sorcery card on the battlefield face up, but some older cards can try to do this. If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it’s an instant or sorcery card. The permanent remains on the battlefield face down. Abilities that trigger when a permanent turns face up won’t trigger, because even though you revealed the card, it never turned face up."
    );
    supported("Magar of the Magic Strings");
    supported("Break Open");
    supported("Secret Plans");
    supported("Exalted Angel");
    // P0's Secret Plans: "Whenever a permanent you control is turned face up, draw a card."
    // P1's Break Open: "Turn target face-down creature an opponent controls face up."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Secret Plans");
    let magar = t.battlefield(P0, "Magar of the Magic Strings");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(&mut t, P0, magar, "Note the name").unwrap();
    t.resolve_all();
    let fd = t.g.current(bolt);
    assert!(t.on_battlefield(fd) && t.obj(fd).face_down);
    // A face-down Exalted Angel (morph) for comparison.
    let angel = t.hand(P0, "Exalted Angel");
    t.lands(P0, "Wastes", 3);
    t.cast(P0, angel)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let angel = t.g.current(angel);
    assert!(t.obj(angel).face_down);
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Break Open", &[obj(fd)]);
    t.resolve_all();
    assert!(revealed(&t, fd));
    assert!(t.on_battlefield(fd) && t.obj(fd).face_down);
    assert_eq!(t.pt(fd), (3, 4));
    assert_eq!(t.hand_size(P0), hand, "Secret Plans didn't trigger");
    // The Angel turns face up, with its own characteristics again: Secret Plans triggers.
    cast_new(&mut t, P1, "Break Open", &[obj(angel)]);
    t.resolve_all();
    assert!(!t.obj_now(angel).face_down);
    assert_eq!(t.obj_now(angel).chars.name, "Exalted Angel");
    assert_eq!(t.pt(angel), (4, 5));
    assert_eq!(t.hand_size(P0), hand + 1);
}
