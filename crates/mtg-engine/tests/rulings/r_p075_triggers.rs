//! Rulings batch P075 — triggered abilities that put +1/+1 counters on a group: sacrifice
//! triggers (the legend rule isn't a sacrifice, simultaneous sacrifices, sacrifices as
//! costs), leaves-the-battlefield triggers on simultaneous zone changes (CR 603.10a),
//! "choose one that hasn't been chosen this turn" (Genku, Lita, Breeches, Galadriel), and
//! creatures entering together (Thalia's Lieutenant).

use crate::r_p075_common::*;
use crate::r_p108_common::tokens_with;
use crate::r_s01_common::with_subtype;
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn legend_rule_isnt_a_sacrifice() {
    cr!("704.5j", "701.21a");
    ruling!(
        "Kels, Fight Fixer",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    ruling!(
        "Korvold, Fae-Cursed King",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    ruling!(
        "Mayhem Devil",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    ruling!(
        "Mazirek, Kraul Death Priest",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    for name in [
        "Kels, Fight Fixer",
        "Korvold, Fae-Cursed King",
        "Mayhem Devil",
        "Mazirek, Kraul Death Priest",
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.settle();
        assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
        assert_eq!(t.stack_len(), 0, "{name}: no sacrifice trigger");
        // A real sacrifice does trigger it.
        let bears = t.battlefield(P0, "Grizzly Bears");
        sacrifice(&mut t, P0, bears);
        assert_eq!(t.stack_len(), 1, "{name}");
    }
}

#[test]
fn mazirek_sacrificed_with_other_permanents_triggers_for_them() {
    cr!("603.10a", "701.21a");
    ruling!(
        "Mazirek, Kraul Death Priest",
        "If you sacrifice Mazirek and other permanents at the same time, Mazirek's last ability triggers for the other permanents."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mazirek, Kraul Death Priest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(m), obj(bears)]);
    cast_card(&mut t, P0, "Barter in Blood", &[]);
    t.resolve();
    assert!(!t.on_battlefield(m) && !t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(
        plus(&t, giant),
        3,
        "one trigger per other sacrificed permanent"
    );
}

#[test]
fn mazirek_trigger_from_a_cost_resolves_first() {
    cr!("603.3", "601.2h", "602.2b");
    ruling!(
        "Mazirek, Kraul Death Priest",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Mazirek's last ability will resolve before that spell or ability."
    );
    ruling!(
        "Mazirek, Kraul Death Priest",
        "Mazirek's last ability is a triggered ability, not an activated ability."
    );
    ruling!(
        "Indulgent Aristocrat",
        "You can sacrifice Indulgent Aristocrat to pay the cost of its own activated ability."
    );
    supported("Indulgent Aristocrat");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mazirek, Kraul Death Priest");
    assert!(!can_activate(&mut t, P0, m), "no activated ability");
    let a = t.battlefield(P0, "Indulgent Aristocrat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, a, 0, &[]).expect("activates");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(plus(&t, a), 1, "Mazirek's trigger resolved first");
    assert_eq!(
        t.stack_len(),
        1,
        "the Aristocrat's ability is still waiting"
    );
    t.resolve_all();
    assert_eq!(plus(&t, a), 2);
    // The Aristocrat can be sacrificed to pay for its own ability.
    let vamp = t.battlefield(P0, "Vampire Noble");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(a)]);
    t.activate(P0, a, 0, &[]).expect("activates");
    assert!(!t.on_battlefield(a));
    t.resolve_all();
    assert_eq!(plus(&t, vamp), 2, "Mazirek's trigger and the ability");
}

#[test]
fn leaving_together_triggers_for_each_other_permanent() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Genku, Future Shaper",
        "If Genku and one or more other nontoken permanents you control leave the battlefield at the same time, Genku's ability will trigger for each of those other nontoken permanents."
    );
    ruling!(
        "Krenko, Baron of Tin Street",
        "If Krenko, Baron of Tin Street dies at the same time as one or more artifacts are put into a graveyard from the battlefield, its last ability will still trigger for each of those artifacts."
    );
    // Genku.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Genku, Future Shaper");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Forest");
    leave_together(&mut t, &[g, a, b]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    // Krenko.
    supported("Krenko, Baron of Tin Street");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Krenko, Baron of Tin Street");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    leave_together(&mut t, &[k, a, b]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Goblin").len(), 2);
}

#[test]
fn genku_different_modes_and_only_three() {
    cr!("700.2", "603.3c");
    ruling!(
        "Genku, Future Shaper",
        "If multiple other nontoken permanents you control leave the battlefield simultaneously, you must still choose different modes for each instance"
    );
    ruling!(
        "Genku, Future Shaper",
        "If you can't legally choose a mode because all three have been chosen that turn, that instance of the ability is removed from the stack with no effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Genku, Future Shaper");
    let ids: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Forest")).collect();
    leave_together(&mut t, &ids);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(
        toks.len(),
        3,
        "only three of the four instances do anything"
    );
    for sub in ["Fox", "Moonfolk", "Rat"] {
        assert_eq!(tokens_with(&t, P0, sub), 1, "{sub}");
    }
}

#[test]
fn two_genkus_track_modes_separately() {
    cr!("700.2", "704.5j");
    ruling!(
        "Genku, Future Shaper",
        "If you somehow control two or more Genku, Future Shapers, track which modes have been chosen each turn for each one's ability separately."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Genku, Future Shaper");
    t.battlefield(P0, "Genku, Future Shaper");
    t.settle();
    assert_eq!(t.named_on_battlefield("Genku, Future Shaper").len(), 2);
    let f = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    leave_together(&mut t, &[f]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Fox").len(), 2, "each chose the Fox");
}

#[test]
fn lita_and_breeches_fourth_instance_does_nothing() {
    cr!("700.2", "603.3c");
    ruling!(
        "Lita, Little Orphan Amphibian",
        "If you can't legally choose a mode because all three have been chosen that turn, that instance of the ability is removed from the stack with no effect."
    );
    ruling!(
        "Breeches, Eager Pillager",
        "If you can't legally choose a mode because all three have been chosen that turn, that instance of the ability is removed from the stack with no effect."
    );
    // Lita: four creatures enter together.
    let mut t = TestGame::new(2);
    let lita = t.battlefield(P0, "Lita, Little Orphan Amphibian");
    enter_together(
        &mut t,
        &[
            (P0, "Grizzly Bears"),
            (P0, "Grizzly Bears"),
            (P0, "Grizzly Bears"),
            (P0, "Grizzly Bears"),
        ],
    );
    t.resolve_all();
    assert_eq!(plus(&t, lita), 1);
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
    // Breeches: four Pirates attack.
    let mut t = TestGame::new(2);
    let mut pirates = vec![t.battlefield(P0, "Breeches, Eager Pillager")];
    for _ in 0..3 {
        pirates.push(t.battlefield(P0, "Dire Fleet Hoarder"));
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&pirates));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert_eq!(t.g.exile.len(), 1, "one card exiled");
}

#[test]
fn galadriel_modes_are_tracked_for_the_permanent_not_the_player() {
    cr!("700.2");
    ruling!(
        "Galadriel, Light of Valinor",
        "It doesn't matter who has chosen any particular mode."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Galadriel, Light of Valinor");
    choose_mode(&mut t, P0, 0);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    choose_mode(&mut t, P0, 1);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    give_control(&mut t, g, P1);
    let hand = t.hand_size(P1);
    let from = t.asked().len();
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    let avail = asked_since(&t, from)
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseModes { available, .. } if p == P1 => Some(available),
            _ => None,
        });
    if let Some(avail) = avail {
        assert_eq!(avail, vec![2]);
    }
    assert_eq!(t.hand_size(P1), hand + 1, "only the third mode was left");
}

#[test]
fn thalias_lieutenant_entering_with_another_human() {
    cr!("603.6a", "603.2");
    ruling!(
        "Thalia's Lieutenant",
        "If Thalia’s Lieutenant enters the battlefield at the same time as another Human, each of Thalia’s Lieutenant’s abilities will trigger."
    );
    supported("Thalia's Lieutenant");
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Thalia's Lieutenant"), (P0, "Elite Vanguard")],
    );
    t.resolve_all();
    assert_eq!((plus(&t, ids[0]), plus(&t, ids[1])), (1, 1));
}
