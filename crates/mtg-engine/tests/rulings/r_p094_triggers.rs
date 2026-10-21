//! Rulings batch P094 — triggered and activated abilities that cost or lose life: Elenda
//! and Azor, The Day of the Doctor, Blood Tyrant, Grim Servant's devotion, Cryptolith
//! Fragment (mana ability, intervening "if", Two-Headed Giant), Champion of the Weird,
//! Netherborn Altar, and Night Shift of the Living Dead's die modifier.

use crate::r_s01_common::{supported, tokens, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s13_common::{commander, commander_game};
use crate::r_s19_common::{planechase_game, start_planar_deck};
use crate::r_s20_common::tap_for_mana;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn elenda_and_azor_pays_4_life_only_once() {
    cr!("603.5", "119.4");
    ruling!(
        "Elenda and Azor",
        "As the last ability resolves, you choose whether to pay 4 life. You can’t pay it more than once to create additional Vampire Knight tokens."
    );
    supported("Elenda and Azor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elenda and Azor");
    t.g.draw_cards(P0, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(tokens(&t, P0).len(), 2);
    let yes_no = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 1);
}

#[test]
fn day_of_the_doctor_no_doctors_chosen_exiles_all_creatures() {
    cr!("714.2c", "608.2c");
    ruling!(
        "The Day of the Doctor",
        "As the last chapter ability resolves, you may choose to exile all other creatures even if you didn't choose any Doctors. In that case, all creatures are exiled."
    );
    supported("The Day of the Doctor");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Day of the Doctor");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, true);
    t.g.add_counters(obj(saga), counters::LORE, 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P0), 7);
}

#[test]
fn blood_tyrant_triggers_however_a_player_loses() {
    cr!("104.3a", "104.3b", "104.3c", "104.3d", "800.4a");
    ruling!(
        "Blood Tyrant",
        "Blood Tyrant’s third ability will trigger no matter how a player loses the game: due to a state-based action (as a result of having a life total of 0 or less, trying to draw a card from an empty library, or having ten poison counters), a spell or ability that says that player loses the game, a concession, or a game loss awarded by a judge."
    );
    supported("Blood Tyrant");
    type Loss = fn(&mut TestGame, PlayerId);
    let ways: [(&str, Loss); 5] = [
        ("0 life", |t, p| t.g.players[p.idx()].life = 0),
        ("empty library", |t, p| {
            t.g.players[p.idx()].library.clear();
            t.g.draw_cards(p, 1);
        }),
        ("poison", |t, p| {
            t.g.players[p.idx()]
                .counters
                .insert(counters::POISON.into(), 10);
        }),
        ("loses the game", |t, p| t.g.lose_game(p)),
        ("concession", |t, p| {
            t.g.perform_action(p, Action::Concede).unwrap();
        }),
    ];
    for (how, lose) in ways {
        let mut t = TestGame::new(3);
        let tyrant = t.battlefield(P0, "Blood Tyrant");
        lose(&mut t, P2);
        t.g.flush_events();
        t.settle();
        assert!(t.has_lost(P2), "{how}");
        t.resolve_all();
        assert_eq!(t.counters(tyrant, counters::PLUS1), 5, "{how}");
    }
}

/// Names offered by the first library search P0 was asked since decision `from`.
fn search_names(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(
                candidates
                    .iter()
                    .filter_map(|e| e.object())
                    .map(|o| t.g.obj(o).chars.name.to_string())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn grim_servant_devotion_counted_on_resolution_x_is_0_in_library() {
    cr!("700.5", "202.3e", "603.2");
    ruling!(
        "Grim Servant",
        "Count the number of black mana symbols among the mana costs of permanents you control as Grim Servant's triggered ability resolves to determine your devotion to black. If Grim Servant is still on the battlefield at that time, it will be included in that count."
    );
    ruling!(
        "Grim Servant",
        "If a card in a player's library has {X} in its mana cost, X is 0 when determining that card's mana value."
    );
    supported("Grim Servant");
    for stays in [true, false] {
        let mut t = TestGame::new(2);
        for name in ["Dark Ritual", "Endless One", "Hill Giant"] {
            t.library_top(P0, name);
        }
        let gs = t.enter(P0, "Grim Servant");
        t.settle();
        if !stays {
            destroy(&mut t, gs);
        }
        let from = t.asked().len();
        t.resolve_all();
        let names = search_names(&t, from);
        // Endless One ({X}) has mana value 0 in the library.
        assert!(names.contains(&"Endless One".to_string()), "{names:?}");
        assert!(!names.contains(&"Hill Giant".to_string()), "{names:?}");
        // Dark Ritual ({B}): only with Grim Servant itself counted.
        assert_eq!(names.contains(&"Dark Ritual".to_string()), stays, "{names:?}");
        assert_eq!(t.life(P0), 17);
    }
}

#[test]
fn cryptolith_fragment_mana_ability_and_its_life_loss_cant_be_responded_to() {
    cr!("605.3a", "605.3b");
    ruling!(
        "Cryptolith Fragment // Aurora of Emrakul",
        "Cryptolith Fragment’s second ability is a mana ability. Players can’t respond to it or to the loss of life it causes."
    );
    let mut t = TestGame::new(2);
    let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, frag, "Add"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!((t.life(P0), t.life(P1)), (19, 19));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn cryptolith_fragment_transform_checks_life_on_trigger_and_resolution() {
    cr!("603.4", "701.27a");
    ruling!(
        "Cryptolith Fragment // Aurora of Emrakul",
        "If any player has 11 or more life as your upkeep begins, Cryptolith Fragment’s last ability doesn’t trigger. If any player has 11 or more life as the ability resolves, the ability has no effect."
    );
    // P1 at 11: no trigger.
    let mut t = TestGame::new(2);
    let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
    t.g.players[0].life = 10;
    t.g.players[1].life = 11;
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "transform"), 0);
    // Both at 10: it triggers; P1 gains 1 in response: no effect.
    for gain in [0, 1] {
        let mut t = TestGame::new(2);
        let frag2 = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
        t.g.players[0].life = 10;
        t.g.players[1].life = 10;
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "transform"), 1);
        t.g.gain_life(P1, gain);
        t.resolve_all();
        assert_eq!((t.obj_now(frag2).face == FaceState::Back), gain == 0, "gain {gain}");
    }
    assert!(!(t.obj_now(frag).face == FaceState::Back));
}

#[test]
fn cryptolith_fragment_in_two_headed_giant() {
    cr!("810.9", "810.9a", "810.2");
    ruling!(
        "Cryptolith Fragment // Aurora of Emrakul",
        "In a Two-Headed Giant game, Cryptolith Fragment’s mana ability causes each player to lose 1 life, so each team loses 2 life."
    );
    ruling!(
        "Cryptolith Fragment // Aurora of Emrakul",
        "In a Two-Headed Giant game, Cryptolith Fragment’s last ability checks whether each team’s life total is 10 or less."
    );
    ruling!(
        "Cryptolith Fragment // Aurora of Emrakul",
        "In a Two-Headed Giant game, Aurora of Emrakul’s last ability causes each opponent to lose 3 life, so the opposing team loses a total of 6 life."
    );
    // The mana ability: each team loses 2.
    let mut t = two_headed_giant();
    let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
    assert!(tap_for_mana(&mut t, P0, frag, "Add"));
    assert_eq!((t.life(P0), t.life(P2)), (28, 28));
    // The upkeep check uses team life totals: both teams at 10 → transforms; one team at
    // 11 → doesn't.
    for (team1, transforms) in [(10, true), (11, false)] {
        let mut t = two_headed_giant();
        let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
        for (p, life) in [(0, 10), (1, 10), (2, team1), (3, team1)] {
            t.g.players[p].life = life;
        }
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!((t.obj_now(frag).face == FaceState::Back), transforms, "{team1}");
    }
    // Aurora of Emrakul attacking: each opponent loses 3, the opposing team 6.
    let mut t = two_headed_giant();
    let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
    crate::r_s17_common::transform(&mut t, frag);
    let aurora = t.g.current(frag);
    crate::r_s01_common::attack_with(&mut t, &[(aurora, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(t.life(P2), 24);
}

#[test]
fn champion_of_the_weird_countered_beheld_card_stays_exiled() {
    cr!("701.4a", "118.8", "701.6a");
    ruling!(
        "Champion of the Weird",
        "If Champion of the Weird is countered or otherwise fails to resolve, the beheld card will remain in exile indefinitely."
    );
    supported("Champion of the Weird");
    supported("Cancel");
    let mut t = TestGame::new(2);
    let piker = t.hand(P0, "Goblin Piker");
    t.lands(P0, "Swamp", 4);
    let champ = t.hand(P0, "Champion of the Weird");
    t.answer_choose(P0, &[obj(piker)]);
    let spell = t.cast(P0, champ).go();
    assert!(t.in_exile("Goblin Piker"));
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cancel).target(obj(spell)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Champion of the Weird"));
    t.advance_to(P1, Step::End);
    assert!(t.in_exile("Goblin Piker"));
}

/// A Commander game with Netherborn Altar on P0's battlefield.
fn altar_game() -> (TestGame, ObjectId) {
    supported("Netherborn Altar");
    let mut t = commander_game();
    let altar = t.battlefield(P0, "Netherborn Altar");
    (t, altar)
}

#[test]
fn netherborn_altar_left_battlefield_uses_its_last_soul_counters() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Netherborn Altar",
        "If Netherborn Altar leaves the battlefield while its ability is on the stack, the amount of life you lose is determined by the number of soul counters it had on it immediately before it left the battlefield."
    );
    let (mut t, altar) = altar_game();
    commander(&mut t, P0, "Grizzly Bears");
    t.g.objects[altar.0 as usize]
        .counters
        .insert("soul".into(), 1);
    // Decline "put it into the command zone instead" (CR 903.9b).
    t.answer_yes(P0, false);
    t.activate(P0, altar, 0, &[]).unwrap();
    assert_eq!(t.counters(altar, "soul"), 2);
    destroy(&mut t, altar);
    t.resolve_all();
    assert_eq!(t.life(P0), 40 - 6);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn netherborn_altar_one_commander_of_two_or_just_life_loss() {
    cr!("903.3", "608.2c");
    ruling!(
        "Netherborn Altar",
        "If you have two commanders in the command zone, Netherborn Altar’s ability puts one of your choice into your hand, not both."
    );
    ruling!(
        "Netherborn Altar",
        "If your commander isn’t in the command zone (or if you’re not playing the Commander variant), Netherborn Altar only makes you lose life."
    );
    let (mut t, altar) = altar_game();
    commander(&mut t, P0, "Grizzly Bears");
    let giant = commander(&mut t, P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, false);
    t.activate(P0, altar, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 37);
    // Commander on the battlefield: only the life loss.
    let (mut t, altar) = altar_game();
    let cmd = commander(&mut t, P0, "Grizzly Bears");
    crate::r_s05_common::move_to(&mut t, cmd, Zone::Battlefield);
    let hand = t.hand_size(P0);
    t.activate(P0, altar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 37);
    // Not a Commander game: only the life loss.
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Netherborn Altar");
    let hand = t.hand_size(P0);
    t.activate(P0, altar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn night_shift_on_the_planar_die_pays_life_but_changes_nothing() {
    cr!("706.7", "706.2a", "901.9d");
    ruling!(
        "Night Shift of the Living Dead",
        "If you roll the planar die in a Planechase game, you can pay 1 life, but you can’t alter the result of the roll. This would still “use up” the ability for the turn."
    );
    ruling!(
        "Xenosquirrels",
        "If you roll the planar die in a Planechase game, you can remove a +1/+1 counter from Xenosquirrels, but you can’t alter the result of the roll."
    );
    supported("Night Shift of the Living Dead");
    supported("Xenosquirrels");
    for name in ["Night Shift of the Living Dead", "Xenosquirrels"] {
        let mut t = planechase_game(2);
        start_planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
        let src = t.battlefield(P0, name);
        if name == "Xenosquirrels" {
            t.g.objects[src.0 as usize]
                .counters
                .insert(counters::PLUS1.into(), 2);
        }
        t.answer_yes(P0, true);
        // A blank face (natural 4).
        t.g.dice.loaded.push_back(4);
        let face = mtg_engine::planechase::roll_planar_die(&mut t.g, P0);
        t.settle();
        assert_eq!(face, mtg_engine::planechase::PlanarFace::Blank, "{name}");
        if name == "Xenosquirrels" {
            assert_eq!(t.counters(src, counters::PLUS1), 1);
            continue;
        }
        assert_eq!(t.life(P0), 19);
        // Used up: a later six-sided roll of 5 isn't offered the change.
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.g.dice.loaded.push_back(5);
        let spec = mtg_engine::dice::DieRoll::new(6);
        let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
        mtg_engine::dice::roll(&mut t.g, &spec, &mut ctx);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.life(P0), 19);
        assert!(tokens(&t, P0).is_empty());
    }
}

#[test]
fn dissatisfied_customer_result_modified_to_0_loses_nothing() {
    cr!("706.2", "706.2a");
    ruling!(
        "Dissatisfied Customer",
        "If the result is 0 or less, because of modifications to the result, you won't lose any life. You also won't gain any life if the result is somehow negative."
    );
    supported("Dissatisfied Customer");
    // Natural 1, decreased by Night Shift (paying 1 life) to 0: no more life lost.
    // Natural 2 decreased to 1: 1 more.
    for (natural, expect) in [(1, 19), (2, 18)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Night Shift of the Living Dead");
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.g.dice.loaded.push_back(natural);
        t.enter(P0, "Dissatisfied Customer");
        t.resolve_all();
        assert_eq!(t.life(P0), expect, "natural {natural}");
    }
}
