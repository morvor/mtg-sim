//! Rulings batch P222 — reinforce (CR 702.77), renew, renown (CR 702.112), replicate
//! (CR 702.56), retrace (CR 702.81), the Revelation mode of The Spear of Leonidas, riot
//! (CR 702.136), saddle (CR 702.171) and scavenge (CR 702.97).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s03_common::{in_hand_with_mana, respond};
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s11_common::spells_copied;
use crate::r_s14_common::{cast_from_hand, triggers_on_stack_now};
use crate::r_s17_common::become_copy;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::saddle::is_saddled;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

fn haste(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Haste)
}

// ---------------------------------------------------------------------------------------
// Reinforce and scavenge
// ---------------------------------------------------------------------------------------

/// P0's answer to a priority decision while Bannerhide Krushok is in P0's graveyard and
/// its scavenge ability can be activated: activate it.
fn scavenge_krushok(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Priority { actions } = d else {
        return None;
    };
    actions
        .iter()
        .find(|a| {
            matches!(a, Action::Activate { source, .. }
                if g.obj(*source).chars.name == "Bannerhide Krushok"
                    && g.obj(*source).zone == Zone::Graveyard(P0))
        })
        .map(|a| Answer::Action(a.clone()))
}

#[test]
fn bannerhide_krushok_reinforce_gives_opponents_a_window_before_scavenge() {
    cr!("702.77a", "702.97a", "117.3b", "117.3c");
    ruling!("Bannerhide Krushok", "If you activate Bannerhide Krushok's reinforce ability during your main phase, your opponents will have the chance to respond to that ability before you can activate the scavenge ability, perhaps by removing Bannerhide Krushok from your graveyard. However, if Bannerhide Krushok ends up in your graveyard during your main phase in such a way that there are no spells or abilities on the stack, you'll have priority to activate the scavenge ability before anyone can remove Bannerhide Krushok from your graveyard.");
    supported("Bannerhide Krushok");
    // Reinforce: discarding it is part of the cost; with the ability on the stack, the
    // sorcery-speed scavenge ability can't be activated yet.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let krushok = t.hand(P0, "Bannerhide Krushok");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Wastes", 6);
    activate_containing(&mut t, P0, krushok, "Reinforce").unwrap();
    assert!(t.in_graveyard(P0, "Bannerhide Krushok"));
    assert_eq!(t.stack_len(), 1);
    assert!(!can_activate(&mut t, P0, krushok));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Once the stack is empty, P0 has priority and can activate scavenge.
    assert!(can_activate(&mut t, P0, krushok));
    // Krushok dies to P0's own Doom Blade: P0 activates scavenge as soon as the Bolt
    // has resolved, before P1 ever has priority while it's in the graveyard.
    let mut t = TestGame::new(2);
    let krushok = t.battlefield(P0, "Bannerhide Krushok");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Wastes", 6);
    let in_graveyard = |g: &mtg_engine::game::Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name == "Bannerhide Krushok")
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let p1_saw = watch(&mut t, P1, is_priority, in_graveyard);
    respond(&mut t, P0, scavenge_krushok);
    cast_from_hand(&mut t, P0, "Doom Blade", &[Entity::Object(krushok)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(t.in_exile("Bannerhide Krushok"), "scavenge was activated");
    assert_eq!(t.counters(bears, counters::PLUS1), 4);
    let seen = p1_saw.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|x| !x), "{seen:?}");
}

#[test]
fn dodgy_jalopy_scavenge_uses_its_power_as_it_last_existed_in_the_graveyard() {
    cr!("702.97a", "602.2b", "608.2h", "604.3");
    ruling!("Dodgy Jalopy", "Exiling the card with scavenge is part of the cost of activating the scavenge ability. Once the ability is activated and the cost is paid, it's too late to stop the ability from being activated by trying to remove the card from the graveyard.");
    ruling!("Dodgy Jalopy", "The number of counters given by Dodgy Jalopy's scavenge ability is its power as it last existed in your graveyard before exiling it. Players can't change how many counters it gives out after you've activated the ability by destroying the creature you control with the highest mana value.");
    supported("Dodgy Jalopy");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jalopy = t.graveyard(P0, "Dodgy Jalopy");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(jalopy).0, 6);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[bears.into()]);
    activate_containing(&mut t, P0, jalopy, "Scavenge").unwrap();
    // Already exiled: nothing can stop the ability any more.
    assert!(t.in_exile("Dodgy Jalopy"));
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, maw);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 6);
}

// ---------------------------------------------------------------------------------------
// Renew
// ---------------------------------------------------------------------------------------

/// Activates Naga Fleshcrafter's renew ability from P0's graveyard targeting `target`.
fn naga_renew(t: &mut TestGame, target: ObjectId) {
    let naga = t.graveyard(P0, "Naga Fleshcrafter");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[target.into()]);
    activate_containing(t, P0, naga, "Exile ~ from your graveyard").unwrap();
    t.resolve_all();
}

/// Naga Fleshcrafter enters as a copy of `of`.
fn naga_enters_as(t: &mut TestGame, of: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(of)]);
    let naga = t.enter(P0, "Naga Fleshcrafter");
    t.settle();
    t.g.current(naga)
}

#[test]
fn naga_fleshcrafter_enters_abilities_work_on_entering_but_not_with_renew() {
    cr!("707.5", "614.1c", "603.6a", "707.2");
    ruling!("Naga Fleshcrafter", "Any “enters” abilities of the copied creature will trigger when Naga Fleshcrafter enters. Any “as this creature enters” or “this creature enters with” abilities of the copied creature will also work. Activating its renew ability doesn't cause any creatures to enter the battlefield, and as such, these kinds of abilities won't work with Naga Fleshcrafter's renew ability.");
    supported("Naga Fleshcrafter");
    // Entering as a copy of Elvish Visionary ("When this creature enters, draw a card."):
    // draws. As a copy of Arcbound Worker (modular 1): enters with a +1/+1 counter.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let worker = t.enter(P0, "Arcbound Worker");
    let hand = t.hand_size(P0);
    let naga = naga_enters_as(&mut t, visionary);
    t.resolve_all();
    assert_eq!(t.obj_now(naga).chars.name, "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand + 1);
    let naga2 = naga_enters_as(&mut t, worker);
    assert_eq!(t.obj_now(naga2).chars.name, "Arcbound Worker");
    assert_eq!(t.counters(naga2, counters::PLUS1), 1);
    // Renew: Grizzly Bears becomes a copy of Elvish Visionary (no draw), then of Arcbound
    // Worker (no counter: a 0/0 that dies).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    naga_renew(&mut t, visionary);
    assert_eq!(t.obj_now(bears).chars.name, "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let worker = t.enter(P0, "Arcbound Worker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    naga_renew(&mut t, worker);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.counters(worker, counters::PLUS1), 2);
}

#[test]
fn naga_fleshcrafter_copies_a_token_as_its_creating_effect_said() {
    cr!("707.2", "111.4");
    ruling!("Naga Fleshcrafter", "If the copied creature is a token, Naga Fleshcrafter (or in the case of its renew ability, each other creature you control) copies the original characteristics of that token as stated by the effect that created that token.");
    supported("Naga Fleshcrafter");
    for renew in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        cast(&mut t, P0, "Raise the Alarm", &[]);
        t.resolve_all();
        let soldier = tokens(&t, P0)[0];
        // The token has a +1/+1 counter and +3/+3 until end of turn.
        cast(&mut t, P0, "Battlegrowth", &[soldier.into()]);
        t.resolve_all();
        cast(&mut t, P0, "Giant Growth", &[soldier.into()]);
        t.resolve_all();
        assert_eq!(t.pt(soldier), (5, 5));
        let copy = if renew {
            let bears = t.battlefield(P0, "Grizzly Bears");
            naga_renew(&mut t, soldier);
            bears
        } else {
            naga_enters_as(&mut t, soldier)
        };
        let name = t.obj(soldier).chars.name.clone();
        let o = t.obj_now(copy);
        assert_eq!(o.chars.name, name, "renew {renew}");
        assert!(o.chars.colors.contains(types::Color::White));
        assert_eq!(t.pt(copy), (1, 1), "renew {renew}");
    }
}

#[test]
fn naga_fleshcrafter_copying_a_copy_gets_what_it_copied() {
    cr!("707.3");
    ruling!("Naga Fleshcrafter", "If the copied creature is copying something else, then Naga Fleshcrafter enters as whatever that creature copied. Similarly, its renew ability will cause each other creature you control to become a copy of whatever that creature copied.");
    supported("Naga Fleshcrafter");
    for renew in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        // P0's Grizzly Bears is a copy of Hill Giant.
        let bears = t.battlefield(P0, "Grizzly Bears");
        become_copy(&mut t, bears, giant);
        assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
        let copy = if renew {
            let other = t.battlefield(P0, "Llanowar Elves");
            naga_renew(&mut t, bears);
            other
        } else {
            naga_enters_as(&mut t, bears)
        };
        assert_eq!(t.obj_now(copy).chars.name, "Hill Giant", "renew {renew}");
        assert_eq!(t.pt(copy), (3, 3), "renew {renew}");
    }
}

#[test]
fn lasyd_prowler_renew_counts_lands_as_it_resolves() {
    cr!("608.2h", "107.3a");
    ruling!("Lasyd Prowler", "The value of X is calculated only once, as Lasyd Prowler’s renew ability resolves.");
    supported("Lasyd Prowler");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let prowler = t.graveyard(P0, "Lasyd Prowler");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[bears.into()]);
    activate_containing(&mut t, P0, prowler, "Exile ~ from your graveyard").unwrap();
    // A third land card is put into the graveyard before it resolves.
    t.graveyard(P0, "Island");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    // Later lands don't change anything.
    t.graveyard(P0, "Island");
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
}

// ---------------------------------------------------------------------------------------
// Renown
// ---------------------------------------------------------------------------------------

#[test]
fn constable_of_the_realm_triggers_on_counters_from_any_source() {
    cr!("603.2", "702.112a");
    ruling!("Constable of the Realm", "Constable of the Realm's last ability triggers whenever a +1/+1 counter is put on it for any reason, not just because of its renown ability.");
    supported("Constable of the Realm");
    // A +1/+1 counter from Battlegrowth.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let constable = t.battlefield(P0, "Constable of the Realm");
    let thopter = t.battlefield(P1, "Ornithopter");
    cast(&mut t, P0, "Battlegrowth", &[constable.into()]);
    t.answer_targets(P0, &[thopter.into()]);
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    // And from its renown ability.
    let mut t = TestGame::new(2);
    let constable = t.battlefield(P0, "Constable of the Realm");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[thopter.into()]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(constable, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(constable, counters::PLUS1), 2);
    assert!(t.in_exile("Ornithopter"));
}

// ---------------------------------------------------------------------------------------
// Replicate
// ---------------------------------------------------------------------------------------

fn copies_on_stack(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).kind == ObjKind::SpellCopy)
        .count()
}

#[test]
fn lose_focus_replicate_copies_even_if_the_original_was_countered() {
    cr!("702.56a", "608.2h", "707.10");
    ruling!("Lose Focus", "As the replicate triggered ability resolves, you'll copy Lose Focus for each time you paid its replicate cost, even if the original spell is no longer on the stack at that time (perhaps because it was countered).");
    supported("Lose Focus");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = cast(&mut t, P1, "Grizzly Bears", &[]);
    t.lands(P0, "Island", 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let focus = cast(&mut t, P0, "Lose Focus", &[bears.into()]);
    t.settle();
    assert_eq!(triggers_on_stack_now(&t), 1);
    cast(&mut t, P1, "Counterspell", &[focus.into()]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Lose Focus"));
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(copies_on_stack(&t), 1);
    t.resolve_all();
    // P1 can't pay {2}: the copy counters Grizzly Bears.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn replicate_copies_are_not_cast() {
    cr!("702.56a", "707.10", "702.108a");
    ruling!("Lose Focus", "The copies that replicate creates are created on the stack, so they're not “cast.” Abilities that trigger when a player casts a spell won't trigger.");
    ruling!("Stream of Thought", "The copies that replicate creates are created on the stack, so they’re not “cast.” Abilities that trigger when a player casts a spell won’t trigger.");
    supported("Lose Focus");
    supported("Stream of Thought");
    // Monastery Swiftspear's prowess triggers once, for the original only.
    for name in ["Lose Focus", "Stream of Thought"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let spear = t.battlefield(P0, "Monastery Swiftspear");
        let bears = if name == "Lose Focus" {
            t.set_step(P1, Step::PrecombatMain);
            let b = cast(&mut t, P1, "Grizzly Bears", &[]);
            t.lands(P0, "Island", 2);
            Entity::Object(b)
        } else {
            t.lands(P0, "Island", 4);
            t.lands(P0, "Wastes", 4);
            Entity::Player(P1)
        };
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
        cast(&mut t, P0, name, &[bears]);
        t.settle();
        // Replicate and prowess.
        assert_eq!(triggers_on_stack_now(&t), 2, "{name}");
        t.answer_yes(P0, false);
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(spells_copied(&t), 2, "{name}");
        assert_eq!(t.pt(spear), (2, 3), "{name}: prowess triggered once");
        assert_eq!(t.g.history.spells_cast.len(), 1 + (name == "Lose Focus") as usize);
    }
}

/// The Forests in P1's graveyard.
fn milled(t: &TestGame) -> usize {
    t.g.player(P1)
        .graveyard
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == "Forest")
        .count()
}

#[test]
fn stream_of_thought_copies_resolve_first_and_even_if_it_is_countered() {
    cr!("702.56a", "608.2h", "405.5", "701.24a");
    ruling!("Stream of Thought", "If you replicate Stream of Thought, the copies resolve before the original spell, so they can’t shuffle Stream of Thought into your library.");
    ruling!("Stream of Thought", "You’ll copy Stream of Thought for each time you paid its replicate cost, even if it’s countered.");
    supported("Stream of Thought");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        for _ in 0..12 {
            t.library_top(P1, "Forest");
        }
        let lib = t.library_size(P0);
        t.graveyard(P0, "Hill Giant");
        t.lands(P0, "Island", 3);
        t.lands(P0, "Wastes", 2);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
        let stream = cast(&mut t, P0, "Stream of Thought", &[Entity::Player(P1)]);
        t.settle();
        if counter {
            cast(&mut t, P1, "Counterspell", &[stream.into()]);
            t.resolve();
            assert!(t.in_graveyard(P0, "Stream of Thought"));
        }
        t.answer_yes(P0, false);
        t.resolve();
        assert_eq!(copies_on_stack(&t), 1, "counter {counter}");
        // The copy resolves (with Stream of Thought itself on the stack under it, or
        // countered): P1 mills four, and P0 shuffles up to four cards from P0's graveyard,
        // all of them (Hill Giant, and Stream of Thought if it was countered).
        let gy: Vec<ObjectId> = t.g.player(P0).graveyard.clone();
        let chosen: Vec<Entity> = gy.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &chosen);
        t.resolve();
        assert_eq!(milled(&t), 4);
        assert!(!t.in_graveyard(P0, "Hill Giant"));
        assert_eq!(t.library_size(P0), lib + gy.len());
        if counter {
            assert_eq!(gy.len(), 2);
        } else {
            // Stream of Thought was still on the stack: not shuffled.
            assert_eq!(gy.len(), 1);
            assert_eq!(t.stack_len(), 1);
            t.answer_choose(P0, &[]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Stream of Thought"));
            assert_eq!(milled(&t), 8);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Retrace
// ---------------------------------------------------------------------------------------

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

/// P0's answer to a priority decision while Reality Scramble is in P0's graveyard: cast it
/// with retrace.
fn retrace_scramble(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Priority { actions } = d else {
        return None;
    };
    actions
        .iter()
        .find(|a| {
            matches!(a, Action::Cast { card, method }
                if *method == RETRACE
                    && g.obj(*card).chars.name == "Reality Scramble"
                    && g.obj(*card).zone == Zone::Graveyard(P0))
        })
        .map(|a| Answer::Action(a.clone()))
}

#[test]
fn reality_scramble_can_be_retraced_again_before_anyone_else_acts() {
    cr!("702.81a", "117.3b", "608.2n");
    ruling!("Reality Scramble", "When a retrace card you cast from your graveyard resolves or is countered, it’s put back into your graveyard. You may use the retrace ability to cast it again. If it’s your turn, you may do so before any other player may take actions to try to remove it from your graveyard.");
    supported("Reality Scramble");
    // P0 casts Reality Scramble from the graveyard twice in a row, each time targeting an
    // Ornithopter and putting the next artifact revealed (an Ornithopter) onto the
    // battlefield.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    t.graveyard(P0, "Reality Scramble");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    t.library_top(P0, "Mind Stone");
    t.library_top(P0, "Mind Stone");
    t.answer_targets(P0, &[a.into()]);
    t.answer_targets(P0, &[b.into()]);
    let in_graveyard = |g: &mtg_engine::game::Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name == "Reality Scramble")
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let p1_saw = watch(&mut t, P1, is_priority, in_graveyard);
    respond(&mut t, P0, retrace_scramble);
    let ok = t.g.run_until(2000, |g| {
        g.history.spells_cast.len() == 2 && g.stack.is_empty()
    });
    assert!(ok, "cast twice");
    assert!(t.in_graveyard(P0, "Reality Scramble"));
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 2);
    let seen = p1_saw.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|x| !x), "{seen:?}");
}

#[test]
fn embrace_the_unknown_goes_back_to_the_graveyard_when_retraced() {
    cr!("702.81a", "608.2n", "701.6a");
    ruling!("Embrace the Unknown", "When a retrace spell you cast from your graveyard resolves, fails to resolve, or is countered, it’s put back into your graveyard. You may use the retrace ability to cast it again.");
    supported("Embrace the Unknown");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 9);
    for _ in 0..4 {
        t.library_top(P0, "Hill Giant");
    }
    let card = t.graveyard(P0, "Embrace the Unknown");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    // Countered: back in the graveyard.
    t.answer_choose(P0, &[f1.into()]);
    let spell = t.cast(P0, card).method(RETRACE).go();
    cast(&mut t, P1, "Counterspell", &[spell.into()]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Embrace the Unknown"));
    // Resolves: back in the graveyard, and it can be cast with retrace again.
    let card = t.g.current(card);
    t.answer_choose(P0, &[f2.into()]);
    t.cast(P0, card).method(RETRACE).go();
    t.resolve_all();
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.hand(P0, "Forest");
    assert!(crate::r_s02_common::can_cast(&mut t, P0, card, RETRACE));
}

// ---------------------------------------------------------------------------------------
// Revelation
// ---------------------------------------------------------------------------------------

#[test]
fn spear_of_leonidas_revelation_with_one_or_fewer_cards_in_hand() {
    cr!("700.2a", "609.3");
    ruling!("The Spear of Leonidas", "You may choose the “Revelation” mode even if you have one or fewer cards in your hand.");
    supported("The Spear of Leonidas");
    for in_hand in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spear = t.battlefield(P0, "The Spear of Leonidas");
        assert!(t.g.attach(spear, bears.into()));
        t.set_step(P0, Step::BeginningOfCombat);
        t.g.player_mut(P0).hand.clear();
        for _ in 0..in_hand {
            t.hand(P0, "Forest");
        }
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 2, "{in_hand} in hand");
        assert_eq!(t.graveyard_size(P0), in_hand);
    }
}

// ---------------------------------------------------------------------------------------
// Riot
// ---------------------------------------------------------------------------------------

#[test]
fn a_riot_creature_that_cant_have_counters_gains_haste() {
    cr!("702.136a", "614.12");
    ruling!("Spider-Punk", "If a creature entering the battlefield has riot but can't have +1/+1 counters put on it, it gains haste.");
    ruling!("Arcbound Slasher", "If a creature entering the battlefield has riot but can't have a +1/+1 counter put onto it, it gains haste. In the case of Arcbound Slasher, however, it's toughness will probably be 0, and it will be put into its owner's graveyard before it can attack.");
    supported("Spider-Punk");
    supported("Arcbound Slasher");
    supported("Blightbeetle");
    for name in ["Spider-Punk", "Arcbound Slasher"] {
        for beetle in [true, false] {
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            // Blightbeetle: "Creatures your opponents control can't have +1/+1 counters
            // put on them."
            if beetle {
                t.battlefield(P1, "Blightbeetle");
            }
            t.answer_yes(P0, true);
            let spell = cast(&mut t, P0, name, &[]);
            t.resolve_all();
            let id = t.g.current(spell);
            if !beetle {
                assert!(!haste(&t, id), "{name}");
                assert!(t.on_battlefield(id));
                continue;
            }
            if name == "Arcbound Slasher" {
                // A 0/0 with haste: put into the graveyard.
                assert!(t.in_graveyard(P0, name));
                let last = t.g.obj(t.g.player(P0).graveyard[0]).prev.expect("was a permanent");
                assert!(t.g.obj(last).has_keyword(KeywordKind::Haste));
            } else {
                assert_eq!(t.counters(id, counters::PLUS1), 0);
                assert!(haste(&t, id), "{name}");
            }
        }
    }
}

#[test]
fn spider_punk_riot_haste_lasts_through_turns_and_control_changes() {
    cr!("702.136a", "611.2a", "613.1f");
    ruling!("Spider-Punk", "If you choose for a creature with riot to gain haste, it gains haste indefinitely. It doesn't lose it as the turn ends or if another player gains control of it.");
    supported("Spider-Punk");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, false);
    let spell = cast(&mut t, P0, "Spider-Punk", &[]);
    t.resolve_all();
    let punk = t.g.current(spell);
    assert_eq!(t.counters(punk, counters::PLUS1), 0);
    assert!(haste(&t, punk));
    t.advance_to(P1, Step::Upkeep);
    assert!(haste(&t, punk));
    give_control(&mut t, punk, P1);
    assert_eq!(t.obj_now(punk).controller, P1);
    assert!(haste(&t, punk));
    t.advance_to(P0, Step::Upkeep);
    assert!(haste(&t, punk));
}

#[test]
fn nobody_can_respond_to_the_riot_choice() {
    cr!("702.136a", "614.1c", "614.12");
    ruling!("Spider-Punk", "Riot is a replacement effect. Players can't respond to your choice of a +1/+1 counter or haste.");
    ruling!("Arcbound Slasher", "Riot is a replacement effect. Players can't respond to your choice of a +1/+1 counter or haste, and they can't take actions while the creature is on the battlefield without one or the other.");
    for name in ["Spider-Punk", "Arcbound Slasher"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let is_yes_no = |d: &Decision| matches!(d, Decision::YesNo { .. });
        let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
        // Whether each riot creature on the battlefield has its extra counter or haste
        // (Arcbound Slasher's four modular counters don't count).
        fn on_bf(g: &mtg_engine::game::Game) -> Vec<bool> {
            g.permanents()
                .filter(|o| o.chars.name == "Spider-Punk" || o.chars.name == "Arcbound Slasher")
                .map(|o| {
                    let base = if o.chars.name == "Arcbound Slasher" { 4 } else { 0 };
                    o.counter(counters::PLUS1) > base || o.has_keyword(KeywordKind::Haste)
                })
                .collect()
        }
        let at_choice = watch(&mut t, P0, is_yes_no, on_bf);
        let at_priority = watch(&mut t, P1, is_priority, on_bf);
        t.answer_yes(P0, true);
        let c = in_hand_with_mana(&mut t, P0, name);
        t.cast(P0, c).go();
        t.advance_to(P0, Step::BeginningOfCombat);
        let id = t.named_on_battlefield(name)[0];
        assert!(!haste(&t, id));
        let choices = at_choice.lock().unwrap().clone();
        assert_eq!(choices, vec![Vec::<bool>::new()], "{name}: chosen before it entered");
        let prios = at_priority.lock().unwrap().clone();
        assert!(prios.iter().any(|v| v == &vec![true]), "{name}");
        assert!(prios.iter().all(|v| v.iter().all(|x| *x)), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Saddle
// ---------------------------------------------------------------------------------------

/// Saddles `mount` by tapping `saddlers` (in P0's main phase).
fn saddle(t: &mut TestGame, mount: ObjectId, saddlers: &[ObjectId]) {
    let es: Vec<Entity> = saddlers.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &es);
    activate_containing(t, P0, mount, "Saddle").expect("saddle");
    t.resolve_all();
    assert!(is_saddled(&t.g, mount));
}

/// P0 attacks P1 with `mount` and the game stops with the attack trigger on the stack.
fn attack_with_trigger(t: &mut TestGame, mount: ObjectId) {
    crate::r_s01_common::attack_with(t, &[(mount, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack_now(t), 1);
}

#[test]
fn caustic_bronco_uses_its_last_known_saddled_status() {
    cr!("702.171b", "608.2h", "113.7a");
    ruling!("Caustic Bronco", "If Caustic Bronco isn’t on the battlefield as its triggered ability resolves, use whether it was saddled or not before it left the battlefield to determine who loses life.");
    supported("Caustic Bronco");
    for saddled in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bronco = t.battlefield(P0, "Caustic Bronco");
        let giant = t.battlefield(P0, "Hill Giant");
        t.library_top(P0, "Hill Giant");
        if saddled {
            saddle(&mut t, bronco, &[giant]);
        }
        attack_with_trigger(&mut t, bronco);
        destroy(&mut t, bronco);
        assert!(t.in_graveyard(P0, "Caustic Bronco"));
        t.resolve_all();
        assert!(t.in_hand(P0, "Hill Giant"));
        if saddled {
            assert_eq!((t.life(P0), t.life(P1)), (20, 16));
        } else {
            assert_eq!((t.life(P0), t.life(P1)), (16, 20));
        }
    }
}

#[test]
fn fortune_delayed_trigger_works_without_fortune_and_tokens_dont_return() {
    cr!("603.7a", "603.7c", "111.8", "702.171b");
    ruling!("Fortune, Loyal Steed", "Fortune's middle ability creates a delayed triggered ability that triggers at end of combat. This ability will trigger whether or not Fortune is still on the battlefield. If it's not, you can still exile up to one creature that saddled it that turn.");
    ruling!("Fortune, Loyal Steed", "Tokens exiled by the delayed triggered ability won't return to the battlefield. It's usually not a good idea to saddle Fortune if Fortune itself is a token.");
    supported("Fortune, Loyal Steed");
    for token in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let fortune = t.battlefield(P0, "Fortune, Loyal Steed");
        let saddler = if token {
            create_token(&mut t, P0, "Soldier")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        saddle(&mut t, fortune, &[saddler]);
        attack_with_trigger(&mut t, fortune);
        t.resolve_all();
        // Fortune leaves the battlefield before the end of combat.
        destroy(&mut t, fortune);
        assert!(t.in_graveyard(P0, "Fortune, Loyal Steed"));
        t.answer_choose(P0, &[saddler.into()]);
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        let now = t.g.current(saddler);
        if token {
            assert!(!t.g.is_live(now) || !t.on_battlefield(now));
            assert!(tokens(&t, P0).is_empty());
        } else {
            // Exiled and returned: a new object.
            assert_ne!(now, saddler);
            assert!(t.on_battlefield(now));
            assert_eq!(t.named_on_battlefield("Grizzly Bears"), vec![now]);
        }
        // Fortune stays in the graveyard.
        assert!(t.in_graveyard(P0, "Fortune, Loyal Steed"));
    }
}
