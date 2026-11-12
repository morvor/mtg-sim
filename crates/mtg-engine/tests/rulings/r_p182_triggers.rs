//! Rulings batch P182 — the triggered abilities around searches: reflexive triggers
//! (CR 603.12), intervening "if" clauses (CR 603.4), leaving the battlefield before a
//! trigger resolves (CR 603.10, 400.7), linked abilities (CR 607) and control changes.

use crate::r_p182_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The targets of the topmost stack object.
fn top_targets(t: &TestGame) -> Vec<Entity> {
    let top = *t.g.stack.last().expect("empty stack");
    crate::r_s25_common::targets_of(t, top)
}

#[test]
fn prishes_wanderings_targets_with_a_reflexive_trigger_after_the_search() {
    cr!("603.12", "601.2c", "701.23a");
    ruling!(
        "Prishe's Wanderings",
        "You don't choose a target for Prishe's Wanderings at the time you cast it. Rather, a second \"reflexive\" ability triggers when you search your library this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast(&mut t, P0, "Prishe's Wanderings", "2G", &[]);
    assert!(crate::r_s25_common::targets_of(&t, spell).is_empty());
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert!(tapped(&t, lib[0]), "the land is found");
    // The reflexive trigger waits on the stack, targeting the Bears; P1 can respond.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(top_targets(&t), vec![obj(bears)]);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    t.cast(P1, shock).target(obj(bears)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn sutina_targets_with_a_reflexive_trigger_after_the_land_returns() {
    cr!("603.12");
    ruling!(
        "Sutina, Speaker of the Tajuru",
        "You don't choose a target for Sutina's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you return a land you control to its owner's hand this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    let mut t = TestGame::new(2);
    let sutina = t.battlefield(P0, "Sutina, Speaker of the Tajuru");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(sutina, pl(P1))]);
    assert!(top_targets(&t).is_empty(), "the attack trigger has no target");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(top_targets(&t), vec![obj(bears)]);
    assert_eq!(t.counters(bears, counters::PLUS1), 0, "not yet resolved");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn proctors_gaze_needs_no_target_but_doesnt_search_if_its_target_is_illegal() {
    cr!("608.2b", "115.1d");
    ruling!(
        "Proctor's Gaze",
        "You don't have to choose a target for Proctor's Gaze. However, if you do and the target is illegal as Proctor's Gaze tries to resolve, it won't resolve and none of its effects will happen. You won't search your library."
    );
    // No target: the search happens.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    cast(&mut t, P0, "Proctor's Gaze", "2GU", &[]);
    t.resolve_all();
    assert!(tapped(&t, lib[0]));
    // The target leaves first: no search.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest"]);
    let stone = t.battlefield(P1, "Mind Stone");
    cast(&mut t, P0, "Proctor's Gaze", "2GU", &[obj(stone)]);
    destroy(&mut t, stone);
    t.resolve_all();
    assert_eq!(searches(&t, P0), 0);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
}

#[test]
fn endless_horizons_cards_stay_exiled_and_belong_to_that_object() {
    cr!("400.7", "607.2a", "610.3");
    ruling!(
        "Endless Horizons",
        "If Endless Horizons leaves the battlefield, the remaining Plains cards stay exiled. If Endless Horizons returns to the battlefield, it’s a new object with no relation to its previous existence. The previously exiled Plains cards will still stay exiled."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Plains", "Plains", "Forest"]);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    let eh = t.enter(P0, "Endless Horizons");
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Plains").len(), 2);
    destroy(&mut t, eh);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Plains").len(), 2);
    // It returns: a new Endless Horizons (its own search finds nothing).
    let card = t.g.current(eh);
    t.answer_choose(P0, &[]);
    t.g.move_object(card, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    t.g.flush_events();
    t.resolve_all();
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Plains").len(), 2);
}

#[test]
fn endless_horizons_under_another_players_control_triggers_but_does_nothing() {
    cr!("603.2", "108.3");
    ruling!(
        "Endless Horizons",
        "If a different player gains control of Endless Horizons, its ability will trigger at the beginning of that player’s upkeep, but it won’t do anything when it resolves."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Plains", "Forest"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    let eh = t.enter(P0, "Endless Horizons");
    t.resolve_all();
    give_control(&mut t, eh, P1);
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "exiled with"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Plains").len(), 1);
}

#[test]
fn keeper_of_the_accord_triggers_together_and_rechecks_on_resolution() {
    cr!("603.4", "603.3b");
    ruling!(
        "Keeper of the Accord",
        "Both triggered abilities trigger at the same time. You may put them on the stack in either order."
    );
    ruling!(
        "Keeper of the Accord",
        "If the opponent doesn't control more creatures than you as their end step begins, the first ability won't trigger. The same is true for lands and the second ability. Each of those abilities will check again when it tries to resolve. If the opponent no longer controls more creatures (or lands) than you, the ability won't resolve and will have no effect."
    );
    // Both conditions hold: both trigger, P0 orders them.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Plains"]);
    let keeper = t.battlefield(P0, "Keeper of the Accord");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    let from = t.asked().len();
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, keeper), 2);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Order { .. })));
    // P0 catches up on both before they resolve: neither does anything.
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    assert_eq!(searches(&t, P0), 0);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
    // Neither condition holds: nothing triggers.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Keeper of the Accord");
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, keeper), 0);
}

#[test]
fn opponent_controls_more_lands_is_checked_when_it_triggers_and_resolves() {
    cr!("603.4");
    ruling!(
        "Space Marine Scout",
        "Space Marine Scout's triggered ability has an \"intervening if clause,\" which means that it checks to see whether or not an opponent controls more lands than you both when it triggers and when it resolves. If it isn't true both times, you won't search your library for a Plains card."
    );
    ruling!(
        "Claim Jumper",
        "When Claim Jumper enters the battlefield, its triggered ability will check to see if at least one opponent controls more lands than you. If none do, the ability won't trigger at all. If the ability does trigger, it will check again as it tries to resolve. If there's no longer an opponent that controls more lands than you, the ability won't do anything."
    );
    for name in ["Space Marine Scout", "Claim Jumper"] {
        // Not true as it enters: no trigger.
        let mut t = TestGame::new(2);
        let id = t.enter(P0, name);
        t.settle();
        assert_eq!(crate::r_s11_common::triggered_from(&t, id), 0, "{name}");
        // True as it triggers, not as it resolves: no search.
        let mut t = TestGame::new(2);
        set_library(&mut t, P0, &["Plains", "Plains"]);
        t.lands(P1, "Forest", 1);
        let id = t.enter(P0, name);
        t.settle();
        assert_eq!(crate::r_s11_common::triggered_from(&t, id), 1, "{name}");
        t.lands(P0, "Forest", 1);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(searches(&t, P0), 0, "{name}");
        assert_eq!(t.library_size(P0), 2, "{name}");
        // True both times: a Plains is found.
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Plains", "Plains"]);
        t.lands(P1, "Forest", 1);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(lib[0])]);
        t.answer_yes(P0, false);
        t.enter(P0, name);
        t.resolve_all();
        assert!(tapped(&t, lib[0]), "{name}");
    }
}

#[test]
fn sidequest_raise_a_chocobo_rechecks_its_birds() {
    cr!("603.4", "701.27a");
    ruling!(
        "Sidequest: Raise a Chocobo // Black Chocobo",
        "Sidequest: Raise a Chocobo's last ability checks at the moment it would trigger to see if you control four or more Birds. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control four or more Birds at that time, the ability won't resolve and none of its effects will happen."
    );
    for (birds, lose_one, transforms) in [(3, false, false), (4, true, false), (4, false, true)] {
        let mut t = TestGame::new(2);
        let side = t.battlefield(P0, "Sidequest: Raise a Chocobo // Black Chocobo");
        let ids: Vec<_> = (0..birds).map(|_| t.battlefield(P0, "Welkin Hawk")).collect();
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        let triggered = crate::r_s11_common::triggered_from(&t, side);
        assert_eq!(triggered, usize::from(birds >= 4), "{birds} birds");
        if lose_one {
            t.g.move_object(ids[0], Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
            t.g.flush_events();
        }
        t.resolve_all();
        let side = t.g.current(side);
        assert_eq!(
            t.g.obj(side).chars.name == "Black Chocobo",
            transforms,
            "{birds} birds, lose one: {lose_one}"
        );
    }
}

#[test]
fn sand_scout_doesnt_see_lands_put_into_the_graveyard_with_it() {
    cr!("603.10a", "603.2");
    ruling!(
        "Sand Scout",
        "Sand Scout must be on the battlefield for its last ability to trigger. If it is put into your graveyard at the same time as one or more land cards you own, its ability won’t trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sand Scout");
    t.lands(P0, "Plains", 2);
    t.set_step(P1, Step::PrecombatMain);
    cast(&mut t, P1, "Jokulhaups", "4RR", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sand Scout"));
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // On the battlefield, it sees the lands.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sand Scout");
    let plains = t.battlefield(P0, "Plains");
    destroy(&mut t, plains);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn cartographers_hawk_cant_return_from_a_new_zone() {
    cr!("400.7", "603.12");
    ruling!(
        "Cartographer's Hawk",
        "If Cartographer’s Hawk leaves the battlefield before its triggered ability resolves, you can’t return it to its owner’s hand from its new zone."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Plains"]);
    t.lands(P1, "Forest", 2);
    let hawk = t.battlefield(P0, "Cartographer's Hawk");
    attack_with(&mut t, &[(hawk, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, hawk), 1);
    destroy(&mut t, hawk);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cartographer's Hawk"));
    assert_eq!(searches(&t, P0), 0, "\"if you do\": no search");
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
}

#[test]
fn molten_man_leaving_first_sacrifices_a_land_before_the_search() {
    cr!("603.3b", "603.6c", "405.5");
    ruling!(
        "Molten Man, Inferno Incarnate",
        "If Molten Man leaves the battlefield before his enters triggered ability resolves, the leaves-the-battlefield triggered ability will resolve first, and you will sacrifice a land before searching your library for a Mountain."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Mountain"]);
    let forest = t.battlefield(P0, "Forest");
    let mm = t.enter(P0, "Molten Man, Inferno Incarnate");
    t.settle();
    destroy(&mut t, mm);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"), "the Forest is sacrificed first");
    assert!(!t.on_battlefield(forest));
    assert!(tapped(&t, lib[0]), "then the Mountain is found and stays");
}

#[test]
fn sylvan_primordial_targets_each_opponent_and_counts_what_it_destroyed() {
    cr!("701.8a", "702.12b", "115.1", "701.23a");
    ruling!(
        "Sylvan Primordial",
        "If an opponent has any legal permanents to target, you must target one of them."
    );
    ruling!(
        "Sylvan Primordial",
        "one of the target noncreature permanents isn’t destroyed (perhaps because it has indestructible or it regenerated), it won’t count toward the number of Forest cards you can put onto the battlefield"
    );
    ruling!(
        "Sylvan Primordial",
        "You search and shuffle your library only once."
    );
    let mut t = TestGame::new(3);
    set_library(&mut t, P0, &["Forest", "Forest", "Forest"]);
    let relic = t.battlefield(P1, "Darksteel Relic");
    let stone = t.battlefield(P2, "Mind Stone");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(relic)]);
    t.answer_targets(P0, &[obj(stone)]);
    t.enter(P0, "Sylvan Primordial");
    t.resolve_all();
    let targets: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { min, .. } if *p == P0 => Some(*min),
            _ => None,
        })
        .collect();
    assert_eq!(targets, vec![1, 1], "a target for each opponent is required");
    assert!(t.on_battlefield(relic));
    assert!(t.in_graveyard(P2, "Mind Stone"));
    let o = offer(&t, P0, from);
    assert_eq!(o.max, 1, "one permanent was destroyed");
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
}

#[test]
fn boonweaver_giant_puts_only_an_aura_that_can_enchant_it() {
    cr!("303.4f", "303.4g", "701.23a");
    ruling!(
        "Boonweaver Giant",
        "The Aura card must be able to legally enchant Boonweaver Giant. For example, you can’t put an Aura with enchant land onto the battlefield this way."
    );
    ruling!(
        "Boonweaver Giant",
        "If Boonweaver Giant isn’t on the battlefield when its ability resolves, you can’t put any Aura onto the battlefield. You may still search the appropriate zones and shuffle your library, if applicable."
    );
    // An enchant creature Aura enters attached to the Giant; an enchant land Aura stays
    // where it is (CR 303.4i).
    for (pick, enters) in [(1, true), (0, false)] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Wild Growth", "Holy Strength"]);
        t.lands(P0, "Forest", 1);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(lib[pick])]);
        let giant = t.enter(P0, "Boonweaver Giant");
        t.resolve_all();
        let aura = t.g.current(lib[pick]);
        assert_eq!(t.on_battlefield(aura), enters, "{pick}");
        if enters {
            assert_eq!(t.g.obj(aura).attached_to, Some(obj(giant)));
        } else {
            assert_eq!(t.zone(aura), Zone::Library(P0));
        }
    }
    // The Giant is gone: the library is still searched and shuffled, no Aura enters.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Holy Strength"]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    let giant = t.enter(P0, "Boonweaver Giant");
    t.settle();
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(lib[0])));
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
}

#[test]
fn sovereigns_of_lost_alara_search_without_an_aura_or_an_attacker_just_shuffles() {
    cr!("701.23a", "303.4f");
    ruling!(
        "Sovereigns of Lost Alara",
        "If the attacking creature has left the battlefield by the time Sovereigns of Lost Alara's second ability resolves, you can't put an Aura onto the battlefield this way. You may still choose to search your library just to shuffle it."
    );
    ruling!(
        "Sovereigns of Lost Alara",
        "If you choose to search your library but no applicable Aura card is left in your deck (or you choose not to find one), then nothing enters and you simply shuffle your library."
    );
    ruling!(
        "Sovereigns of Lost Alara",
        "If you choose to search your library but no applicable Aura card is left in your library (or you choose not to find one), then nothing enters the battlefield and you simply shuffle your library."
    );
    // The attacker is gone.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Holy Strength"]);
    t.battlefield(P0, "Sovereigns of Lost Alara");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(lib[0])));
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    // Only an Aura that can't enchant the attacker; or one P0 chooses not to find.
    for lib_cards in [&["Wild Growth"][..], &["Holy Strength"][..]] {
        let mut t = TestGame::new(2);
        set_library(&mut t, P0, lib_cards);
        t.battlefield(P0, "Sovereigns of Lost Alara");
        let bears = t.battlefield(P0, "Grizzly Bears");
        attack_with(&mut t, &[(bears, pl(P1))]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[]);
        t.resolve_all();
        assert_eq!(t.library_size(P0), 1, "{lib_cards:?}");
        assert_eq!(permanents_of(&t, P0), names(&["Grizzly Bears", "Sovereigns of Lost Alara"]));
        assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    }
}

#[test]
fn lost_auramancers_aura_isnt_targeted_and_stays_put_if_it_cant_attach() {
    cr!("303.4f", "702.18a", "115.1");
    ruling!(
        "Lost Auramancers",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to a permanent with shroud, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Holy Strength"]);
    let enchantress = t.battlefield(P0, "Argothian Enchantress");
    let auramancers = t.battlefield(P0, "Lost Auramancers");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    destroy(&mut t, auramancers);
    t.resolve_all();
    let aura = t.g.current(lib[0]);
    assert_eq!(t.g.obj(aura).attached_to, Some(obj(enchantress)), "shroud doesn't stop it");
    // Nothing to enchant: the Aura stays in the library.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Holy Strength"]);
    let auramancers = t.battlefield(P0, "Lost Auramancers");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    destroy(&mut t, auramancers);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(lib[0])), Zone::Library(P0));
}

#[test]
fn bitterheart_witch_curse_must_be_able_to_enchant_the_player() {
    cr!("303.4f", "702.16b");
    ruling!(
        "Bitterheart Witch",
        "The Curse must be legally able to enchant the player. For example, if the player has protection from red, you couldn’t put a red Curse onto the battlefield this way."
    );
    for protected in [false, true] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Curse of Stalked Prey"]);
        if protected {
            // Runed Halo: P1 has protection from the name Curse of Stalked Prey (a red
            // Curse), as protection from red would stop it.
            t.answer(
                P1,
                DecisionKind::Name,
                mtg_engine::decision::Answer::Text("Curse of Stalked Prey".into()),
            );
            t.enter(P1, "Runed Halo");
            t.resolve_all();
        }
        let witch = t.battlefield(P0, "Bitterheart Witch");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_choose(P0, &[obj(lib[0])]);
        destroy(&mut t, witch);
        t.resolve_all();
        let curse = t.g.current(lib[0]);
        assert_eq!(t.on_battlefield(curse), !protected, "protected: {protected}");
        if !protected {
            assert_eq!(t.g.obj(curse).attached_to, Some(pl(P1)));
        }
    }
}

#[test]
fn arena_rector_cant_be_exiled_after_leaving_the_graveyard() {
    cr!("400.7", "603.12", "608.2c");
    ruling!(
        "Arena Rector",
        "If Arena Rector leaves your graveyard before its triggered ability resolves, perhaps because it’s a token and ceased to exist, you can’t exile it and won’t be able to search your library for a planeswalker card."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Nissa Revane"]);
    let rector = t.battlefield(P0, "Arena Rector");
    destroy(&mut t, rector);
    // It leaves the graveyard (to exile by another effect) first.
    let card = t.g.current(rector);
    t.g.move_object(card, Zone::Library(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.resolve_all();
    assert_eq!(searches(&t, P0), 0);
    assert!(!t.on_battlefield(t.g.current(lib[0])));
}

#[test]
fn hoarding_dragon_has_its_own_hoard_and_dying_early_exiles_for_good() {
    cr!("607.2a", "603.3b", "400.7");
    ruling!(
        "Hoarding Dragon",
        "Hoarding Dragon's last two abilities are linked. The third ability refers only to the card exiled by Hoarding Dragon's second ability. In other words, each Dragon has its own hoard."
    );
    ruling!(
        "Hoarding Dragon",
        "If Hoarding Dragon dies before its second ability resolves, its third ability will trigger and do nothing. Then its second ability will resolve. If you choose to exile an artifact card from your library, it will be exiled indefinitely."
    );
    // Two Dragons, one hoard each: the one that dies returns only its own.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Mind Stone", "Sol Ring"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    let a = t.enter(P0, "Hoarding Dragon");
    t.resolve_all();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.enter(P0, "Hoarding Dragon");
    t.resolve_all();
    t.answer_yes(P0, true);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(hand_of(&t, P0), names(&["Mind Stone"]));
    assert!(t.in_exile("Sol Ring"));
    // It dies before its enters ability resolves.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Mind Stone"]);
    let d = t.enter(P0, "Hoarding Dragon");
    t.settle();
    t.answer_yes(P0, true);
    destroy(&mut t, d);
    t.resolve();
    assert_eq!(t.hand_size(P0), 0, "the dies trigger does nothing");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.resolve_all();
    assert!(t.in_exile("Mind Stone"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn llanowar_sentinels_chain() {
    cr!("603.2", "603.6a");
    ruling!(
        "Llanowar Sentinel",
        "Llanowar Sentinels can \"chain\": If you put a second Llanowar Sentinel onto the battlefield as a result of this ability, that creature's \"enters\" ability triggers and allows you to search for yet another Llanowar Sentinel."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Llanowar Sentinel", "Llanowar Sentinel"]);
    mana(&mut t, P0, "1G1G");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.answer_yes(P0, false);
    t.enter(P0, "Llanowar Sentinel");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Sentinel").len(), 3);
}

#[test]
fn bloomvine_regent_sees_each_dragon_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Bloomvine Regent // Claim Territory",
        "If Bloomvine Regent enters at the same time as one or more other Dragons you control, its triggered ability will trigger for each of those Dragons, including itself."
    );
    let mut t = TestGame::new(2);
    let ids = crate::r_p116_common::enter_together(
        &mut t,
        P0,
        &["Bloomvine Regent // Claim Territory", "Shivan Dragon", "Shivan Dragon"],
    );
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, ids[0]), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 29);
}
