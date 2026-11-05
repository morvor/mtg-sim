//! Rulings batch P179 — trigger doublers (Yarok, the Desecrated; Starfield Vocalist;
//! Delney, Streetwise Lookout; Veyran; Chief of the Wilds; Hama Pashar), plus Invoke the
//! Ancients' counter choices and Seraph of New Phyrexia.

use crate::r_p179_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Elite Arcanist enters with Starfield Vocalist (its "enters" ability triggers twice:
/// Opt and Shock are exiled), then its copy ability is activated with `lands` Wastes.
#[test]
fn starfield_vocalist_linked_abilities_get_every_exiled_card() {
    cr!("607.2a", "603.2d", "107.3");
    ruling!(
        "Starfield Vocalist",
        "When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used."
    );
    supported("Starfield Vocalist");
    supported("Elite Arcanist");
    // (cast Opt's copy, cast Shock's copy)
    for (cast_opt, cast_shock) in [(true, true), (false, true), (false, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Starfield Vocalist");
        let opt = t.hand(P0, "Opt");
        let shock = t.hand(P0, "Shock");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(opt)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(shock)]);
        let arc = cast_resolve(&mut t, P0, "Elite Arcanist", &[]);
        assert!(t.in_exile("Opt") && t.in_exile("Shock"));
        let arc = t.g.current(arc);
        t.g.objects[arc.0 as usize].summoning_sick = false;
        // X is the sum of the exiled cards' mana values: 2.
        t.lands(P0, "Wastes", 1);
        assert!(!can_activate(&mut t, P0, arc));
        t.lands(P0, "Wastes", 1);
        assert!(can_activate(&mut t, P0, arc));
        let from = t.asked().len();
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
        // Copies of both are created; cast them in the order Opt, Shock, each optionally.
        t.answer(
            P0,
            DecisionKind::Order,
            mtg_engine::decision::Answer::Indices(vec![0, 1]),
        );
        t.answer_yes(P0, cast_opt);
        t.answer_yes(P0, cast_shock);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        activate_containing(&mut t, P0, arc, "Copy").unwrap();
        t.resolve_all();
        let xs: Vec<(i64, i64)> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                mtg_engine::decision::Decision::ChooseX { min, .. } => Some((*min, 0)),
                _ => None,
            })
            .collect();
        assert_eq!(xs, vec![(2, 0)]);
        assert_eq!(t.life(P1), if cast_shock { 18 } else { 20 }, "{cast_opt}");
        let scried = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Scry { .. }));
        assert_eq!(scried, cast_opt);
    }
}

/// P0's Angelic Chorus triggers ("whenever a creature you control enters, you gain life
/// equal to its toughness") on the stack.
fn chorus_triggers(t: &TestGame) -> usize {
    triggers_on_stack(t, "you gain life equal to its toughness")
}

#[test]
fn entering_with_the_doubler_doubles_the_trigger() {
    cr!("603.2d", "603.6a", "603.10");
    ruling!(
        "Starfield Vocalist",
        "If a permanent entering the battlefield at the same time as Starfield Vocalist (including Starfield Vocalist itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    ruling!(
        "Yarok, the Desecrated",
        "If a permanent entering the battlefield at the same time as Yarok (including Yarok itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    supported("Yarok, the Desecrated");
    // (doubler, its toughness)
    for (doubler, tough) in [("Starfield Vocalist", 4), ("Yarok, the Desecrated", 5)] {
        supported(doubler);
        // Itself.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Angelic Chorus");
        t.enter(P0, doubler);
        t.settle();
        assert_eq!(chorus_triggers(&t), 2, "{doubler}");
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + 2 * tough, "{doubler}");
        // With Grizzly Bears at the same time: each triggers twice.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Angelic Chorus");
        enter_together(&mut t, &[(P0, doubler), (P0, "Grizzly Bears")]);
        assert_eq!(chorus_triggers(&t), 4, "{doubler}");
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + 2 * tough + 4, "{doubler}");
    }
}

#[test]
fn several_doublers_add_one_trigger_each() {
    cr!("603.2d");
    ruling!(
        "Starfield Vocalist",
        "If you control two Starfield Vocalists, a permanent entering the battlefield causes abilities to trigger three times, not four."
    );
    ruling!(
        "Yarok, the Desecrated",
        "If you somehow control two Yaroks, a permanent entering the battlefield causes abilities to trigger three times, not four."
    );
    ruling!(
        "Delney, Streetwise Lookout",
        "If you somehow control two Delney, Streetwise Lookouts, abilities of creatures you control with power 2 or less trigger three times."
    );
    ruling!(
        "Veyran, Voice of Duality",
        "If you somehow control two Veyrans (which would be poetic), you casting an instant or sorcery spell causes abilities to trigger three times."
    );
    supported("Mirror Gallery");
    for (doubler, n) in [
        ("Starfield Vocalist", 2),
        ("Starfield Vocalist", 3),
        ("Yarok, the Desecrated", 2),
        ("Yarok, the Desecrated", 3),
    ] {
        let mut t = TestGame::new(2);
        // Mirror Gallery: no legend rule.
        t.battlefield(P0, "Mirror Gallery");
        t.lands(P0, doubler, n);
        t.battlefield(P0, "Angelic Chorus");
        t.enter(P0, "Grizzly Bears");
        t.settle();
        assert_eq!(chorus_triggers(&t), n + 1, "{doubler} x{n}");
    }
    // Delney: Man-o'-War's (2/2) "enters" trigger, three times with two Delneys.
    for n in [1, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        t.lands(P0, "Delney, Streetwise Lookout", n);
        let ogre = t.battlefield(P1, "Gray Ogre");
        for _ in 0..=n {
            t.answer_targets(P0, &[obj(ogre)]);
        }
        let mow = t.enter(P0, "Man-o'-War");
        t.settle();
        assert_eq!(triggers_from(&t, mow), n + 1, "Delney x{n}");
    }
    // Veyran: each Veyran's magecraft triggers three times with two Veyrans.
    supported("Veyran, Voice of Duality");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    let vs = t.lands(P0, "Veyran, Voice of Duality", 2);
    cast_targeting(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, vs[0]), 3);
    assert_eq!(triggers_from(&t, vs[1]), 3);
    t.resolve_all();
    assert_eq!(t.pt(vs[0]), (5, 5));
}

#[test]
fn as_enters_abilities_are_not_doubled() {
    cr!("614.1c", "603.2d");
    ruling!(
        "Yarok, the Desecrated",
        "Abilities that apply \"as [this creature] enters the battlefield,\" such as choosing a color with Diamond Knight, are also unaffected."
    );
    ruling!(
        "Delney, Streetwise Lookout",
        "Abilities that apply \"as [this creature] enters the battlefield\" or \"as [this creature] is turned face up\" are also unaffected."
    );
    ruling!(
        "Starfield Vocalist",
        "Abilities that apply “as [this permanent] enters,” such as Famished Worldsire’s devour land ability, are also unaffected."
    );
    supported("Diamond Knight");
    supported("Famished Worldsire");
    // Diamond Knight (1/1) under Yarok or Delney: one color choice.
    for doubler in ["Yarok, the Desecrated", "Delney, Streetwise Lookout"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let from = t.asked().len();
        t.enter(P0, "Diamond Knight");
        t.settle();
        let colors = options_asked_since(&t, P0, from)
            .into_iter()
            .filter(|o| o.iter().any(|c| c == "white"))
            .count();
        assert_eq!(colors, 1, "{doubler}");
    }
    // Famished Worldsire under Starfield Vocalist: devour land once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starfield Vocalist");
    let lands = t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(lands[0]), obj(lands[1])]);
    let ws = t.enter(P0, "Famished Worldsire");
    t.settle();
    let devours = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .count();
    assert_eq!(devours, 1);
    // Devouring two lands: three counters each, once.
    assert_eq!(t.counters(ws, "+1/+1"), 6);
}

#[test]
fn delney_turned_face_up_trigger_needs_power_2_or_less_face_up() {
    cr!("603.2d", "708.8");
    ruling!(
        "Delney, Streetwise Lookout",
        "Abilities that apply \"when [this creature] is turned face up\" will trigger an additional time only if that creature's power is 2 or less once it has been turned face up."
    );
    supported("Fathom Seer");
    supported("Ashcloud Phoenix");
    // Fathom Seer (1/3): twice. Ashcloud Phoenix (4/1): once.
    for (name, n) in [("Fathom Seer", 2), ("Ashcloud Phoenix", 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Delney, Streetwise Lookout");
        let fd = crate::r_s12_common::morph(&mut t, P0, name);
        let islands = t.lands(P0, "Island", 2);
        t.lands(P0, "Mountain", 6);
        t.answer_choose(P0, &[obj(islands[0]), obj(islands[1])]);
        assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fd), "{name}");
        t.settle();
        assert_eq!(t.obj_now(fd).chars.name.as_str(), name);
        assert_eq!(triggers_from(&t, t.g.current(fd)), n, "{name}");
    }
}

#[test]
fn doubled_triggers_choose_targets_and_options_separately() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Delney, Streetwise Lookout",
        "Delney, Streetwise Lookout's last ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    ruling!(
        "Chief of the Wilds",
        "Chief of the Wilds's last ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    // Delney + Man-o'-War: two triggers, each with its own target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Delney, Streetwise Lookout");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(ogre)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Man-o'-War");
    t.resolve_all();
    assert!(t.in_hand(P1, "Gray Ogre") && t.in_hand(P1, "Grizzly Bears"));
    // Chief of the Wilds + Pack Guardian (a Wolf): "you may discard a land card" is
    // chosen separately for each instance.
    supported("Chief of the Wilds");
    supported("Pack Guardian");
    let mut t = TestGame::new(2);
    let chief = t.battlefield(P0, "Chief of the Wilds");
    let forest = t.hand(P0, "Forest");
    t.hand(P0, "Plains");
    let pg = t.enter(P0, "Pack Guardian");
    t.settle();
    assert_eq!(triggers_from(&t, pg), 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Plains"));
    // One Wolf token from the discard; Chief's own trigger for that Wolf token gives it 2
    // counters, and for Pack Guardian 2.
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.counters(chief, "+1/+1"), 4);
}

#[test]
fn hama_pashar_repeats_the_room_without_moving_on() {
    cr!("309.4c", "603.2d");
    ruling!(
        "Hama Pashar, Ruin Seeker",
        "Hama Pashar, Ruin Seeker's ability causes the same room you ventured into to trigger an additional time. It does not cause you to move to the next room."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hama Pashar, Ruin Seeker");
    t.custom(
        P0,
        (*mtg_engine::card::card("Lost Mine of Phandelver")).clone(),
        Zone::Outside(P0),
    );
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::KeywordAction {
            action: mtg_engine::ability::KeywordAction::Venture,
            who: mtg_engine::ability::PlayerRef::You,
            what: mtg_engine::ability::Sel::None,
            n: mtg_engine::ability::Value::c(1),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
    // Cave Entrance's "Scry 1" twice, and the marker stays in the first room.
    assert_eq!(triggers_on_stack(&t, "Scry"), 2);
    t.resolve_all();
    assert_eq!(mtg_engine::dungeons::marker(&t.g, P0).map(|(_, r)| r), Some(0));
}

#[test]
fn delney_evasion_is_checked_only_as_blockers_are_declared() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Delney, Streetwise Lookout",
        "Once a creature you control has become blocked, reducing its power to 2 or less and/or increasing the power of the creature blocking it to 3 or greater won't cause it to become unblocked."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Delney, Streetwise Lookout");
    let ogre = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Gray Ogre");
    let big = t.battlefield(P1, "Craw Wurm");
    crate::r_s01_common::attack_with(&mut t, &[(ogre, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    // The interface offers no block of the 2-power Bears by the 6-power Wurm.
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P1, &[(big, bears)]));
    assert!(crate::r_s21_common::legal_blocks(&mut t, P1, &[(blocker, ogre)]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(blocker, ogre)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // Now the attacking Hill Giant becomes 2/3 and the blocker 3/2: still blocked.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        ogre,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(-1),
            mtg_engine::ability::Value::c(0),
        )],
    );
    crate::r_s26_common::modify_until_eot(
        &mut t,
        blocker,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(1),
            mtg_engine::ability::Value::c(0),
        )],
    );
    t.advance_to(P0, Step::EndOfCombat);
    // Only the unblocked Bears hit P1.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn delney_power_changes_after_triggering_dont_matter() {
    cr!("603.2d", "603.3");
    ruling!(
        "Delney, Streetwise Lookout",
        "Once an ability of a creature you control has triggered, reducing its power to 2 or less won't cause that ability to trigger an additional time."
    );
    supported("Flametongue Kavu");
    // Man-o'-War (2/2) triggers twice; growing it now leaves both on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Delney, Streetwise Lookout");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(ogre)]);
    t.answer_targets(P0, &[obj(bears)]);
    let mow = t.enter(P0, "Man-o'-War");
    t.settle();
    crate::r_s26_common::modify_until_eot(
        &mut t,
        mow,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(3),
            mtg_engine::ability::Value::c(0),
        )],
    );
    assert_eq!(triggers_from(&t, mow), 2);
    // Flametongue Kavu (4/2) triggers once; shrinking it now adds nothing.
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    let fk = t.enter(P0, "Flametongue Kavu");
    t.settle();
    crate::r_s26_common::modify_until_eot(
        &mut t,
        fk,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(-3),
            mtg_engine::ability::Value::c(0),
        )],
    );
    t.settle();
    assert_eq!(triggers_from(&t, fk), 1);
}

#[test]
fn invoke_the_ancients_counter_chosen_for_each_token() {
    cr!("616.1", "614.1a");
    ruling!(
        "Invoke the Ancients",
        "You choose the type of counter that each of the two green Spirit creature tokens will get separately."
    );
    ruling!(
        "Invoke the Ancients",
        "If a replacement effect causes you to create more than two tokens, you will make that choice for each of those tokens individually."
    );
    supported("Invoke the Ancients");
    for lives in [false, true] {
        let mut t = TestGame::new(2);
        if lives {
            t.battlefield(P0, "Parallel Lives");
        }
        let n = if lives { 4 } else { 2 };
        // reach, vigilance, trample, reach
        for i in [0, 1, 2, 0].into_iter().take(n) {
            t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(i));
        }
        let from = t.asked().len();
        cast_resolve(&mut t, P0, "Invoke the Ancients", &[]);
        let asked = options_asked_since(&t, P0, from);
        assert_eq!(asked.len(), n);
        let mut got: Vec<&str> = tokens(&t, P0)
            .iter()
            .map(|id| {
                let o = t.obj_now(*id);
                ["reach", "vigilance", "trample"]
                    .into_iter()
                    .find(|k| o.counter(k) == 1)
                    .unwrap_or("none")
            })
            .collect();
        got.sort();
        let mut want: Vec<&str> = ["reach", "vigilance", "trample", "reach"][..n].to_vec();
        want.sort();
        assert_eq!(got, want, "Parallel Lives {lives}");
    }
}

#[test]
fn seraph_of_new_phyrexia_sacrifice_and_bonus_happen_on_resolution() {
    cr!("603.5", "608.2c");
    ruling!(
        "Seraph of New Capenna // Seraph of New Phyrexia",
        "once that ability starts to resolve, no player may take any actions between you deciding to sacrifice another creature or artifact and Seraph of New Phyrexia getting +2/+1."
    );
    supported("Seraph of New Capenna // Seraph of New Phyrexia");
    let mut t = TestGame::new(2);
    let seraph = t.battlefield(P0, "Seraph of New Capenna // Seraph of New Phyrexia");
    crate::r_s17_common::transform(&mut t, seraph);
    assert_eq!(t.obj_now(seraph).chars.name.as_str(), "Seraph of New Phyrexia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(seraph, Entity::Player(P1))]);
    // The trigger is on the stack: players may respond.
    assert_eq!(triggers_from(&t, t.g.current(seraph)), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(seraph), (5, 4));
}
