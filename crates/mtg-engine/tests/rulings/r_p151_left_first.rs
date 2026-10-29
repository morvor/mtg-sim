//! Rulings batch P151 — "sacrifice this permanent" in a triggered ability whose source has
//! left the battlefield (or changed controllers) before it resolves: the sacrifice can't
//! happen (CR 701.21a), so "If you do" riders don't happen, while instructions that don't
//! depend on it still do (CR 608.2c), and the ability itself still resolves (CR 113.7a).

use crate::r_p151_common::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;
use mtg_engine::testing::*;

/// Puts `n` "Snow-Covered Island"s onto the battlefield for `p`.
fn snow_lands(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.lands(p, "Snow-Covered Island", n)
}

#[test]
fn aangs_iceberg_sacrificed_by_its_own_ability_exiles_nothing() {
    cr!("113.7a");
    ruling!(
        "Aang's Iceberg",
        "If Aang's Iceberg leaves the battlefield before its triggered ability resolves, including if it is sacrificed by its activated ability, the target permanent won't be exiled."
    );
    supported("Aang's Iceberg");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[obj(bears)]);
    let iceberg = t.enter(P0, "Aang's Iceberg");
    t.settle();
    assert_eq!(abilities_on_stack(&t), 1, "the enters trigger");
    // In response, waterbend {3}: sacrifice it (scry 2).
    t.activate(P0, iceberg, 0, &[]).expect("waterbend");
    t.resolve();
    assert!(t.in_graveyard(P0, "Aang's Iceberg"));
    t.resolve_all();
    assert!(t.on_battlefield(bears), "nothing to exile it until");
}

#[test]
fn angelic_renewal_cases() {
    // Leaves before its trigger resolves: can't be sacrificed, nothing returns.
    ruling!(
        "Angelic Renewal",
        "If Angelic Renewal leaves the battlefield before its triggered ability resolves, you won’t be able to sacrifice it and the creature can’t be returned to the battlefield."
    );
    supported("Angelic Renewal");
    let mut t = TestGame::new(2);
    let renewal = t.battlefield(P0, "Angelic Renewal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, renewal);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());

    // A creature it owns dies under another player's control.
    ruling!(
        "Angelic Renewal",
        "Angelic Renewal’s ability triggers if a creature you own dies under another player’s control."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Renewal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    gain_control(&mut t, P1, bears);
    destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.answer_yes(P0, true);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert!(t.in_graveyard(P0, "Angelic Renewal"));

    // A token: it may be sacrificed, but nothing returns.
    ruling!(
        "Angelic Renewal",
        "If a token creature is put into your graveyard from the battlefield, you can sacrifice Angelic Renewal, but nothing will be returned to the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Renewal");
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    assert_eq!(abilities_on_stack(&t), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Angelic Renewal"));
    assert!(tokens(&t, P0).is_empty());

    // An animated noncreature permanent returns (as what it is now: a noncreature).
    ruling!(
        "Angelic Renewal",
        "If an animated permanent that’s not normally a creature is put into your graveyard from the battlefield, Angelic Renewal can return that card to the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Renewal");
    let coils = t.battlefield(P0, "Chimeric Coils");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.activate(P0, coils, 0, &[]).expect("Chimeric Coils");
    t.resolve_all();
    assert!(t.obj_now(coils).is(CardType::Creature));
    destroy(&mut t, coils);
    t.answer_yes(P0, true);
    t.resolve_all();
    let back = t.named_on_battlefield("Chimeric Coils");
    assert_eq!(back.len(), 1);
    assert!(!t.g.obj(back[0]).is(CardType::Creature));

    // Several creatures at once: several triggers, one sacrifice, one creature back.
    ruling!(
        "Angelic Renewal",
        "If multiple creatures are put into your graveyard at once, Angelic Renewal triggers multiple times, but it can only be sacrificed once and only brings back one creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Renewal");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    wrath(&mut t);
    assert_eq!(abilities_on_stack(&t), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    let back =
        t.named_on_battlefield("Grizzly Bears").len() + t.named_on_battlefield("Hill Giant").len();
    assert_eq!(back, 1);
    assert!(t.in_graveyard(P0, "Angelic Renewal"));
}

#[test]
fn awaken_the_sky_tyrant_sacrificed_once_and_only_if_still_there() {
    cr!("701.21a");
    ruling!(
        "Awaken the Sky Tyrant",
        "Awaken the Sky Tyrant will trigger once for each source an opponent controls that deals damage to you, but you can sacrifice it only once."
    );
    supported("Awaken the Sky Tyrant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Awaken the Sky Tyrant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(vec![(a, Entity::Player(P0), 2), (b, Entity::Player(P0), 2)], false);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_on_stack(&t), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "one Dragon");
    assert!(t.in_graveyard(P0, "Awaken the Sky Tyrant"));

    ruling!(
        "Awaken the Sky Tyrant",
        "If Awaken the Sky Tyrant isn’t on the battlefield as its ability resolves, you can’t sacrifice it. You won’t get the Dragon in that case."
    );
    let mut t = TestGame::new(2);
    let awaken = t.battlefield(P0, "Awaken the Sky Tyrant");
    let a = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, a, Entity::Player(P0), 2);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, awaken);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

/// P0 crews Daredevil Dragster (with `velocity` velocity counters) with a Hill Giant and
/// attacks P1 with it; the game stops in the end of combat step, its trigger on the stack.
fn dragster_attacks(t: &mut TestGame, velocity: u32) -> ObjectId {
    let dragster = t.battlefield(P0, "Daredevil Dragster");
    if velocity > 0 {
        put_counters(t, dragster, "velocity", velocity);
    }
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_choose(P0, &[obj(giant)]);
    t.activate(P0, dragster, 0, &[]).expect("crew 2");
    t.resolve_all();
    attack_with(t, &[(dragster, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    dragster
}

#[test]
fn daredevil_dragster_cases() {
    supported("Daredevil Dragster");
    ruling!(
        "Daredevil Dragster",
        "If Daredevil Dragster has one velocity counter on it and leaves the battlefield while its triggered ability is on the stack, it won't get a second velocity counter and you won't draw cards."
    );
    let mut t = TestGame::new(2);
    let dragster = dragster_attacks(&mut t, 1);
    assert_eq!(t.life(P1), 16);
    assert_eq!(abilities_on_stack(&t), 1);
    let hand = t.hand_size(P0);
    exile(&mut t, dragster);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);

    ruling!(
        "Daredevil Dragster",
        "If Daredevil Dragster somehow has two velocity counters on it and leaves the battlefield while its triggered ability is on the stack, you'll draw two cards even though you can't sacrifice it."
    );
    let mut t = TestGame::new(2);
    let dragster = dragster_attacks(&mut t, 2);
    assert_eq!(abilities_on_stack(&t), 1);
    let hand = t.hand_size(P0);
    exile(&mut t, dragster);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);

    // With one counter, the trigger adds the second and the Dragster is sacrificed.
    let mut t = TestGame::new(2);
    let dragster = dragster_attacks(&mut t, 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(dragster));
    assert_eq!(t.hand_size(P0), hand + 2);

    ruling!(
        "Daredevil Dragster",
        "If a second velocity counter is put on Daredevil Dragster by something other than the resolution of its triggered ability, you won't sacrifice it yet."
    );
    let mut t = TestGame::new(2);
    let dragster = t.battlefield(P0, "Daredevil Dragster");
    put_counters(&mut t, dragster, "velocity", 1);
    put_counters(&mut t, dragster, "velocity", 1);
    t.resolve_all();
    assert!(t.on_battlefield(dragster));
    assert_eq!(abilities_on_stack(&t), 0);

    ruling!(
        "Daredevil Dragster",
        "If Daredevil Dragster doesn't survive the combat damage step, its ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let dragster = t.battlefield(P0, "Daredevil Dragster");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_choose(P0, &[obj(giant)]);
    t.activate(P0, dragster, 0, &[]).expect("crew 2");
    t.resolve_all();
    attack_with(&mut t, &[(dragster, Entity::Player(P1))]);
    let hand = t.hand_size(P0);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(wurm, dragster)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(dragster));
    assert_eq!(abilities_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn dark_depths_left_first_and_no_mana_ability() {
    cr!("701.21a");
    ruling!(
        "Dark Depths",
        "If Dark Depths leaves the battlefield before its triggered ability resolves, you won't be able to sacrifice it, so you won't create Marit Lage."
    );
    supported("Dark Depths");
    let mut t = TestGame::new(2);
    let depths = t.battlefield(P0, "Dark Depths");
    put_counters(&mut t, depths, "ice", 1);
    remove_counters(&mut t, depths, "ice", 1);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, depths);
    t.resolve_all();
    assert!(t.named_on_battlefield("Marit Lage").is_empty());

    ruling!(
        "Dark Depths",
        "Dark Depths doesn't have a mana ability. It doesn't tap for colorless mana."
    );
    let mut t = TestGame::new(2);
    let depths = t.battlefield(P0, "Dark Depths");
    t.g.recompute();
    assert!(!t
        .g
        .obj(depths)
        .chars
        .abilities
        .iter()
        .any(|a| a.is_mana_ability()));
}

/// P0 controls Defense of the Heart with two Hill Giants in its library; the game advances
/// to P0's next upkeep. Returns Defense of the Heart and the Giants.
fn defense_upkeep(t: &mut TestGame, setup: impl FnOnce(&mut TestGame)) -> (ObjectId, Vec<ObjectId>) {
    let defense = t.battlefield(P0, "Defense of the Heart");
    let giants = vec![t.library_top(P0, "Hill Giant"), t.library_top(P0, "Hill Giant")];
    setup(t);
    next_upkeep(t);
    (defense, giants)
}

#[test]
fn defense_of_the_heart_checks_on_trigger_and_resolution_and_resolves_without_its_source() {
    supported("Defense of the Heart");
    cr!("603.4");
    ruling!(
        "Defense of the Heart",
        "Defense of the Heart's ability will check if an opponent controls three or more creatures as your upkeep begins. If none of them do, the ability won't trigger at all. If an opponent does, the ability will check again as it tries to resolve. If none of your opponents control three or more creatures at that point, the ability won't do anything. The opponent that controls three or more creatures as the ability resolves doesn't have to be the same opponent that controlled three or more creatures as it triggered."
    );
    // No opponent with three creatures: no trigger.
    let mut t = TestGame::new(2);
    defense_upkeep(&mut t, |t| {
        t.lands(P1, "Grizzly Bears", 2);
    });
    assert_eq!(abilities_on_stack(&t), 0);

    // Fewer than three as it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let mut bears = vec![];
    let (defense, _) = defense_upkeep(&mut t, |t| {
        bears = t.lands(P1, "Grizzly Bears", 3);
    });
    assert_eq!(abilities_on_stack(&t), 1);
    destroy(&mut t, bears[0]);
    t.resolve_all();
    assert!(t.on_battlefield(defense));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());

    // A different opponent controls three as it resolves.
    let mut t = TestGame::new(3);
    let mut bears = vec![];
    let (defense, giants) = defense_upkeep(&mut t, |t| {
        bears = t.lands(P1, "Grizzly Bears", 3);
        t.lands(P2, "Grizzly Bears", 2);
    });
    assert_eq!(abilities_on_stack(&t), 1);
    destroy(&mut t, bears[0]);
    t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giants[0]), obj(giants[1])]);
    t.resolve_all();
    assert!(!t.on_battlefield(defense));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);

    ruling!(
        "Defense of the Heart",
        "If Defense of the Heart leaves the battlefield while its triggered ability is on the stack, the ability will still resolve as normal."
    );
    let mut t = TestGame::new(2);
    let (defense, giants) = defense_upkeep(&mut t, |t| {
        t.lands(P1, "Grizzly Bears", 3);
    });
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, defense);
    t.answer_choose(P0, &[obj(giants[0]), obj(giants[1])]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
}

#[test]
fn marit_lages_slumber_cases() {
    supported("Marit Lage's Slumber");
    ruling!(
        "Marit Lage's Slumber",
        "If Marit Lage’s Slumber leaves the battlefield before its last ability resolves, you won’t be able to sacrifice it, so you won’t create Marit Lage."
    );
    let mut t = TestGame::new(2);
    let slumber = t.battlefield(P0, "Marit Lage's Slumber");
    snow_lands(&mut t, P0, 9);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, slumber);
    t.resolve_all();
    assert!(t.named_on_battlefield("Marit Lage").is_empty());

    ruling!(
        "Marit Lage's Slumber",
        "If you don’t control ten snow permanents as the last ability resolves, you can’t sacrifice Marit Lage’s Slumber."
    );
    let mut t = TestGame::new(2);
    let slumber = t.battlefield(P0, "Marit Lage's Slumber");
    let lands = snow_lands(&mut t, P0, 9);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 1);
    destroy(&mut t, lands[0]);
    t.resolve_all();
    assert!(t.on_battlefield(slumber));
    assert!(t.named_on_battlefield("Marit Lage").is_empty());

    ruling!(
        "Marit Lage's Slumber",
        "If you don’t control ten or more snow permanents as your upkeep begins, the last ability won’t trigger."
    );
    let mut t = TestGame::new(2);
    let slumber = t.battlefield(P0, "Marit Lage's Slumber");
    snow_lands(&mut t, P0, 8);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 0);
    // A tenth snow permanent during the upkeep is too late.
    snow_lands(&mut t, P0, 1);
    t.resolve_all();
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(slumber));
    assert!(t.named_on_battlefield("Marit Lage").is_empty());

    // With ten, it's sacrificed and Marit Lage is created.
    let mut t = TestGame::new(2);
    let slumber = t.battlefield(P0, "Marit Lage's Slumber");
    snow_lands(&mut t, P0, 9);
    next_upkeep(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(slumber));
    assert_eq!(t.named_on_battlefield("Marit Lage").len(), 1);
}

#[test]
fn deadly_designs_not_sacrificed_by_a_new_controller_or_with_illegal_targets() {
    cr!("608.2b");
    ruling!(
        "Deadly Designs",
        "If the player who controlled Deadly Designs as its last ability triggered doesn't control it as that ability resolves, Deadly Designs won't be sacrificed. The same is true if each of the ability's targets are all illegal by the time the ability resolves. However, if it's still on the battlefield and has five or more plot counters on it, its last ability will immediately trigger again."
    );
    supported("Deadly Designs");
    // Control changed.
    let mut t = TestGame::new(2);
    let designs = t.battlefield(P0, "Deadly Designs");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    put_counters(&mut t, designs, "plot", 5);
    assert_eq!(abilities_on_stack(&t), 1);
    gain_control(&mut t, P1, designs);
    t.resolve();
    assert!(t.on_battlefield(designs), "not sacrificed");
    assert!(t.on_battlefield(a) && t.on_battlefield(b), "nothing destroyed");
    assert_eq!(abilities_on_stack(&t), 1, "triggers again");

    // All targets illegal.
    let mut t = TestGame::new(2);
    let designs = t.battlefield(P0, "Deadly Designs");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    put_counters(&mut t, designs, "plot", 5);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, a);
    t.resolve();
    assert!(t.on_battlefield(designs), "not sacrificed");
    assert_eq!(abilities_on_stack(&t), 1, "triggers again");
    t.resolve_all();
    assert!(!t.on_battlefield(designs), "sacrificed by the new trigger");
}

#[test]
fn plague_boiler_removing_a_counter_doesnt_stop_it_removing_the_boiler_does() {
    cr!("603.8");
    ruling!(
        "Plague Boiler",
        "If the third ability triggers, removing a counter in response won’t stop the effect. However, somehow removing Plague Boiler from the battlefield in response would stop the effect because then you wouldn’t be able to sacrifice Plague Boiler."
    );
    supported("Plague Boiler");
    for remove_boiler in [false, true] {
        let mut t = TestGame::new(2);
        let boiler = t.battlefield(P0, "Plague Boiler");
        let bears = t.battlefield(P1, "Grizzly Bears");
        put_counters(&mut t, boiler, "plague", 3);
        assert_eq!(abilities_on_stack(&t), 1);
        if remove_boiler {
            exile(&mut t, boiler);
        } else {
            remove_counters(&mut t, boiler, "plague", 1);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), remove_boiler);
        if !remove_boiler {
            assert!(t.in_graveyard(P0, "Plague Boiler"));
        }
    }
}

#[test]
fn impending_disaster_and_planar_collapse_destroy_even_without_the_sacrifice() {
    cr!("608.2c");
    ruling!(
        "Impending Disaster",
        "If this card is not on the battlefield at the times its ability resolves, then you can’t sacrifice it. However, you still destroy all lands."
    );
    supported("Impending Disaster");
    let mut t = TestGame::new(2);
    let disaster = t.battlefield(P0, "Impending Disaster");
    let lands = t.lands(P1, "Forest", 7);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, disaster);
    t.resolve_all();
    assert!(lands.iter().all(|l| !t.on_battlefield(*l)));

    ruling!(
        "Planar Collapse",
        "If this card is not on the battlefield when the ability resolves, then you can’t sacrifice it. However, you still destroy all creatures."
    );
    supported("Planar Collapse");
    let mut t = TestGame::new(2);
    let collapse = t.battlefield(P0, "Planar Collapse");
    let bears = t.lands(P1, "Grizzly Bears", 4);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 1);
    exile(&mut t, collapse);
    t.resolve_all();
    assert!(bears.iter().all(|b| !t.on_battlefield(*b)));
}

#[test]
fn time_elemental_under_another_players_control_still_deals_damage() {
    cr!("701.21a");
    ruling!(
        "Time Elemental",
        "If Time Elemental attacks or blocks, but you no longer control it at end of combat, you can’t sacrifice it. However, it will still deal 5 damage to you."
    );
    supported("Time Elemental");
    let mut t = TestGame::new(2);
    let te = t.battlefield(P0, "Time Elemental");
    attack_with(&mut t, &[(te, Entity::Player(P1))]);
    t.resolve_all();
    gain_control(&mut t, P1, te);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(te));
    assert_eq!(t.g.obj(t.g.current(te)).controller, P1);
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vexing_devil_is_sacrificed_even_if_the_damage_is_prevented() {
    cr!("615.1");
    ruling!(
        "Vexing Devil",
        "If a player chooses to have Vexing Devil deal 4 damage to them, but some or all of that damage is prevented or redirected, Vexing Devil will still be sacrificed."
    );
    supported("Vexing Devil");
    supported("Energy Field");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Energy Field");
    t.lands(P0, "Mountain", 1);
    let devil = t.hand(P0, "Vexing Devil");
    t.answer_yes(P1, true);
    t.cast(P0, devil).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "prevented");
    assert!(t.named_on_battlefield("Vexing Devil").is_empty());
    assert!(t.in_graveyard(P0, "Vexing Devil"));
}

#[test]
fn ajanis_last_stand_one_token_and_discard_trigger() {
    ruling!(
        "Ajani's Last Stand",
        "If multiple creatures and/or planeswalkers you control die at the same time, the first ability of Ajani’s Last Stand triggers that many times. However, you can sacrifice it only once, so you’ll get only one token."
    );
    supported("Ajani's Last Stand");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ajani's Last Stand");
    t.lands(P0, "Grizzly Bears", 3);
    wrath(&mut t);
    assert_eq!(abilities_on_stack(&t), 3);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(t.in_graveyard(P0, "Ajani's Last Stand"));

    ruling!(
        "Ajani's Last Stand",
        "If a spell or ability an opponent controls causes you to make a choice and you choose to discard Ajani’s Last Stand, its last ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plains");
    let ajani = t.hand(P0, "Ajani's Last Stand");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(ajani)]);
    // Raven's Crime: "Target player discards a card." (P0 chooses which.)
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Raven's Crime", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ajani's Last Stand"));
    assert_eq!(tokens(&t, P0).len(), 1, "4/4 Avatar");
}
