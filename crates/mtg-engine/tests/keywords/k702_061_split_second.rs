//! CR 702.61 Split second.

use crate::common_k702_011_017::{assert_supported, custom_card, keyword_count};
use crate::common_k702_038_051::with_cost;
use crate::common_k702_052_066::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn no_other_spells_or_nonmana_abilities_while_a_split_second_spell_is_on_the_stack() {
    cr!("702.61", "702.61a");
    ruling!(
        "Krosan Grip",
        "Casting a spell with split second won't affect spells and abilities that are already on the stack."
    );
    assert_supported("Krosan Grip");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    // P1's Shock is already on the stack.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(P0).go();
    t.lands(P0, "Forest", 3);
    let grip = t.hand(P0, "Krosan Grip");
    let grip_spell = t.cast(P0, grip).target(anthem).go();
    // Neither player can cast a spell or activate a nonmana ability.
    assert!(t.cast(P1, cs).target(grip_spell).try_go().is_err());
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_err());
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    t.clear_answers();
    t.resolve();
    assert!(!t.on_battlefield(anthem));
    // Once it has left the stack, players may again, before Shock resolves.
    assert_eq!(t.stack_len(), 1);
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_ok());
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn mana_abilities_and_special_actions_are_allowed() {
    cr!("702.61b");
    ruling!(
        "Trickbind",
        "Players may turn face-down creatures face up while a spell with split second is on the stack."
    );
    assert_supported("Scornful Egotist");
    let mut t = TestGame::new(2);
    // A face-down Scornful Egotist (morph {U}).
    t.lands(P0, "Island", 3);
    let egotist = t.hand(P0, "Scornful Egotist");
    t.cast(P0, egotist)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let face_down = t.g.current(egotist);
    assert!(t.obj_now(face_down).face_down);
    // P1 casts Sudden Shock (split second).
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Sudden Shock");
    t.cast(P1, shock).target(P1).go();
    // P0 activates mana abilities (a creature's and a land's), then turns Scornful
    // Egotist face up, paying {U} from the mana pool.
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(t.activate(P0, elves, 0, &[]).is_ok());
    let island = t.battlefield(P0, "Island");
    assert!(t.activate(P0, island, 0, &[]).is_ok());
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    t.g.turn.priority = Some(P0);
    let turn_up = Action::Special(SpecialAction::TurnFaceUp { obj: face_down });
    assert!(t.g.legal_actions(P0).contains(&turn_up));
    t.g.perform_action(P0, turn_up).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(!t.obj_now(face_down).face_down);
    assert_eq!(t.obj_now(face_down).chars.name.as_str(), "Scornful Egotist");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn triggered_abilities_trigger_as_normal() {
    cr!("702.61b");
    ruling!(
        "Krosan Grip",
        "Split second doesn't stop triggered abilities from triggering, such as that of Chalice of the Void."
    );
    assert_supported("Chalice of the Void");
    assert_supported("Kiln Fiend");
    let mut t = TestGame::new(2);
    // Chalice of the Void with two charge counters (Sudden Shock's mana value).
    let chalice = t.enter(P1, "Chalice of the Void");
    let chalice = t.g.current(chalice);
    run_effect(
        &mut t,
        None,
        P1,
        mtg_engine::ability::Effect::AddCounters {
            what: mtg_engine::ability::Sel::Target(0),
            kind: "charge".into(),
            n: mtg_engine::ability::Value::c(2),
        },
        &[Entity::Object(chalice)],
    );
    let fiend = t.battlefield(P0, "Kiln Fiend");
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P0, "Sudden Shock");
    t.cast(P0, shock).target(P1).go();
    t.settle();
    // Kiln Fiend's and Chalice of the Void's abilities triggered and are on the stack
    // above the split second spell; they resolve as normal.
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.pt(fiend), (4, 2));
    assert!(t.in_graveyard(P0, "Sudden Shock"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_spell_cast_by_a_resolving_triggered_ability_is_prohibited_too() {
    cr!("702.61a", "702.61b");
    ruling!(
        "Krosan Grip",
        "If the resolution of a triggered ability involves casting a spell, that spell can't be cast if a spell with split second is on the stack."
    );
    let mut t = TestGame::new(2);
    // Thrumming Stone gives Krosan Grip ripple 4: the ripple ability triggers and
    // resolves, but the revealed Krosan Grip can't be cast.
    t.battlefield(P0, "Thrumming Stone");
    on_top(&mut t, P0, "Krosan Grip");
    let target = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Forest", 3);
    let grip = t.hand(P0, "Krosan Grip");
    t.cast(P0, grip).target(target).go();
    t.settle();
    assert_eq!(stack_triggers(&t, "Ripple").len(), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    // It wasn't cast: it's among the revealed cards put on the bottom of the library.
    assert_eq!(t.g.history.spells_cast.len(), 1);
    let bottom4: Vec<&str> = t.g.player(P0).library[..4]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.as_str())
        .collect();
    assert!(bottom4.contains(&"Krosan Grip"));
    t.resolve_all();
    assert!(!t.on_battlefield(target));
}

#[test]
fn multiple_instances_of_split_second_are_redundant() {
    cr!("702.61c");
    let mut t = TestGame::new(2);
    // Samut gives instant and sorcery spells split second; Sudden Shock already has it.
    t.battlefield(P0, "Samut, Tyrant of Naktamun");
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P0, "Sudden Shock");
    let spell = t.cast(P0, shock).target(P1).go();
    assert_eq!(keyword_count(&t, spell, KeywordKind::SplitSecond), 2);
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_err());
    t.resolve();
    // Once it leaves the stack, there's no lingering effect.
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_ok());
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // A custom spell printed with two instances works like one with a single instance.
    let def = with_cost(
        custom_card(
            "Double-Quick Shock",
            "Instant",
            None,
            "Split second\nSplit second\n~ deals 2 damage to any target.",
        ),
        "{R}",
    );
    let quick = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Mountain", 1);
    t.cast(P0, quick).target(P1).go();
    assert!(t
        .activate(P1, pyromancer, 0, &[Entity::Player(P0)])
        .is_err());
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
