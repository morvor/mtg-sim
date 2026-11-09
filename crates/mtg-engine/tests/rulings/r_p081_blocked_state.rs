//! Rulings batch P081 — being blocked and blocking as states: a blocked creature stays
//! blocked when its blockers leave (CR 509.1h, 510.1c), a blocker stays blocking through
//! the end of combat step (CR 506.4), spells that act on blocked or blocking creatures
//! (Fight to the Death, Outmaneuver, Gaze of the Gorgon, Sorrow's Path, The Wretched),
//! combat-timed abilities (Maze of Ith, Dwarven Sea Clan, Plasma Caster) and prevention
//! spells for the defending player (Forcefield, Snag, Revealing Wind).

use crate::r_p081_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// What the blocker (followed across zone changes) is blocking now.
fn blocking(t: &TestGame, b: ObjectId) -> Vec<ObjectId> {
    let b = t.g.current(b);
    t.g.combat.as_ref().map(|c| c.blocking(b)).unwrap_or_default()
}

#[test]
fn destroying_a_blocker_doesnt_unblock_the_attacker() {
    cr!("509.1h", "510.1c");
    ruling!(
        "Nahiri's Machinations",
        "Destroying a blocking creature doesn't cause the creature it was blocking to become unblocked."
    );
    ruling!(
        "Sizzling Barrage",
        "Destroying a blocking creature doesn’t cause the creature it was blocking to become unblocked."
    );
    ruling!(
        "Dissension in the Ranks",
        "Destroying a blocking creature doesn’t cause the attacking creature it blocked to become unblocked."
    );
    supported("Nahiri's Machinations");
    supported("Sizzling Barrage");
    supported("Dissension in the Ranks");
    // Nahiri's Machinations: 1 damage to the 1/1 blocker.
    let mut t = TestGame::new(2);
    let nahiri = t.battlefield(P0, "Nahiri's Machinations");
    t.lands(P0, "Mountain", 2);
    let (a, bs) = block_with(&mut t, "Hill Giant", &["Llanowar Elves"]);
    t.activate(P0, nahiri, 0, &[obj(bs[0])]).unwrap();
    t.resolve_all();
    assert!(died(&t, bs[0]));
    assert!(is_blocked(&t, a));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    // Sizzling Barrage: 4 damage to a creature that blocked this turn.
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Hill Giant", &["Hill Giant"]);
    cast_new(&mut t, P0, "Sizzling Barrage", &[obj(bs[0])]);
    t.resolve_all();
    assert!(died(&t, bs[0]));
    assert!(is_blocked(&t, a));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert_eq!(marked(&t, a), 0);
    // Dissension in the Ranks: two blockers fight, the 1/1 dies.
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Hill Giant");
    let b2 = t.battlefield(P1, "Llanowar Elves");
    blocked(&mut t, &[a1, a2], &[(b1, a1), (b2, a2)]);
    cast_new(&mut t, P0, "Dissension in the Ranks", &[obj(b1), obj(b2)]);
    t.resolve_all();
    assert!(died(&t, b2));
    assert_eq!(marked(&t, b1), 1);
    assert!(is_blocked(&t, a2));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn tide_of_war_sacrificed_blockers_leave_the_attackers_blocked() {
    cr!("509.1h", "510.1c", "705.1");
    ruling!(
        "Tide of War",
        "Even if the blockers are sacrificed, any attackers that were blocked are still considered blocked."
    );
    supported("Tide of War");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tide of War");
    let a1 = t.battlefield(P0, "Hill Giant");
    let a2 = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    // P0 wins the flip: each blocking creature is sacrificed.
    crate::r_p017_common::load_coins(&mut t, &[true]);
    blocked(&mut t, &[a1, a2], &[(b1, a1), (b2, a2)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(died(&t, b1) && died(&t, b2));
    assert!(is_blocked(&t, a1) && is_blocked(&t, a2));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn fight_to_the_death_spares_unblocked_attackers_and_needs_blocks() {
    cr!("509.1h", "506.4", "511.3");
    ruling!(
        "Fight to the Death",
        "Attacking creatures that haven’t been blocked are unaffected by Fight to the Death."
    );
    ruling!(
        "Fight to the Death",
        "If Fight to the Death is cast before blockers are declared or after combat ends, it won’t do anything."
    );
    supported("Fight to the Death");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Hill Giant");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[a1, a2], &[(b1, a1)]);
    cast_new(&mut t, P1, "Fight to the Death", &[]);
    t.resolve_all();
    assert!(died(&t, a1) && died(&t, b1));
    assert!(t.on_battlefield(a2));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
    // Before blockers are declared: nothing is blocked or blocking yet.
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a1, pl(P1))]);
    cast_new(&mut t, P1, "Fight to the Death", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(a1) && t.on_battlefield(b1));
    // After combat (no blocks were declared): nothing is blocked or blocking any more.
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    cast_new(&mut t, P1, "Fight to the Death", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(a1) && t.on_battlefield(b1));
}

#[test]
fn outmaneuver_damages_the_player_even_without_the_blockers() {
    cr!("510.1c", "510.1a");
    ruling!(
        "Outmaneuver",
        "Creatures damage the defending player or planeswalker even if the blocking creatures are no longer there at that time."
    );
    supported("Outmaneuver");
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Hill Giant", &["Llanowar Elves"]);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Outmaneuver");
    t.answer_targets(P0, &[obj(a)]);
    t.cast(P0, card).x(1).go();
    t.resolve();
    cast_new(&mut t, P0, "Lightning Bolt", &[obj(bs[0])]);
    t.resolve_all();
    assert!(died(&t, bs[0]));
    assert!(is_blocked(&t, a));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn the_wretched_doesnt_take_blockers_that_regenerated_out_of_combat() {
    cr!("701.19a", "506.4", "608.2h");
    ruling!(
        "The Wretched",
        "Any blocking creatures that regenerated during combat will have been removed from combat. Since such creatures are no longer in combat, they cannot be blocking The Wretched, which means you won’t be able to gain control of them."
    );
    supported("The Wretched");
    // A blocker that survives is taken at end of combat.
    let mut t = TestGame::new(2);
    let (w, bs) = block_with(&mut t, "The Wretched", &["Hill Giant"]);
    t.resolve_all();
    finish_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.obj_now(bs[0]).controller, P0);
    assert!(t.on_battlefield(w));
    // Drudge Skeletons with a regeneration shield: regenerated, removed from combat, kept.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 1);
    let (_, bs) = block_with(&mut t, "The Wretched", &["Drudge Skeletons"]);
    t.activate(P1, bs[0], 0, &[]).unwrap();
    t.resolve_all();
    finish_combat(&mut t);
    assert!(t.on_battlefield(bs[0]));
    assert!(tapped(&t, bs[0]), "regenerated");
    assert!(blocking(&t, bs[0]).is_empty());
    t.resolve_all();
    assert_eq!(t.obj_now(bs[0]).controller, P1);
}

#[test]
fn gaze_of_the_gorgon_destroys_everything_that_blocked_the_creature_this_turn() {
    cr!("603.7a", "608.2h");
    ruling!(
        "Gaze of the Gorgon",
        "At the next end of combat step in the turn, all creatures that had blocked the targeted creature at any point during the turn will be destroyed. This includes creatures that blocked it before Gaze of the Gorgon was cast. It doesn’t matter whether the targeted creature is still on the battlefield at that point."
    );
    supported("Gaze of the Gorgon");
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Grizzly Bears", &["Hill Giant"]);
    // Cast after the block, then the Bears are exiled.
    cast_new(&mut t, P0, "Gaze of the Gorgon", &[obj(a)]);
    t.resolve();
    cast_new(&mut t, P1, "Swords to Plowshares", &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert!(t.on_battlefield(bs[0]));
    finish_combat(&mut t);
    t.resolve_all();
    assert!(died(&t, bs[0]));
}

#[test]
fn sorrows_path_swap_retriggers_block_and_per_blocker_triggers_only() {
    cr!("509.3a", "509.3c", "509.3d", "506.4");
    ruling!(
        "Sorrow's Path",
        "Abilities that trigger whenever one of the targeted creatures blocks will trigger when the first ability resolves, because those creatures will change from not blocking (since they’re removed from combat) to blocking. It doesn’t matter if those abilities triggered when those creatures blocked the first time. Abilities that trigger whenever one of the attacking creatures becomes blocked will not trigger again, because they never stopped being blocked creatures. Abilities that trigger whenever a creature blocks one of the attacking creatures will trigger again, though; those kinds of abilities trigger once for each creature that blocks."
    );
    supported("Rabid Wolverines");
    supported("Benefactor's Draught");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    // Deeproot Warrior: "Whenever this creature becomes blocked, it gets +1/+1".
    // Rabid Wolverines: "Whenever this creature becomes blocked by a creature, it gets +1/+1".
    let deep = t.battlefield(P0, "Deeproot Warrior");
    let wolv = t.battlefield(P0, "Rabid Wolverines");
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    // Benefactor's Draught: "Until end of turn, whenever a creature an opponent controls
    // blocks, draw a card."
    t.set_step(P0, Step::BeginningOfCombat);
    cast_new(&mut t, P0, "Benefactor's Draught", &[]);
    t.resolve();
    let hand = t.hand_size(P0);
    blocked(&mut t, &[deep, wolv], &[(g1, deep), (g2, wolv)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.pt(deep), (3, 3));
    assert_eq!(t.pt(wolv), (5, 5));
    // The swap: g1 now blocks the Wolverines, g2 the Warrior.
    t.answer_targets(P0, &[obj(g1), obj(g2)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(blocking(&t, g1), vec![wolv]);
    assert_eq!(blocking(&t, g2), vec![deep]);
    assert_eq!(t.hand_size(P0), hand + 4, "each blocked again");
    assert_eq!(t.pt(deep), (3, 3), "never stopped being blocked");
    assert_eq!(t.pt(wolv), (6, 6), "blocked by a creature again");
}

#[test]
fn a_blocking_creature_stays_blocking_after_its_attacker_left() {
    cr!("506.4", "509.1h");
    ruling!(
        "Rix Maadi Guildmage",
        "A blocking creature is one that has been declared as a blocker this combat, or one that was put onto the battlefield blocking this combat. Unless that creature leaves combat, it continues to be a blocking creature through the end of combat step, even if the creature or creatures it was blocking are no longer on the battlefield or have otherwise left combat."
    );
    supported("Rix Maadi Guildmage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P1, "Rix Maadi Guildmage");
    let (a, bs) = block_with(&mut t, "Grizzly Bears", &["Hill Giant"]);
    cast_new(&mut t, P0, "Unsummon", &[obj(a)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.g.is_blocking(bs[0]));
    assert!(ability_targets(&mut t, mage, 0).contains(&obj(bs[0])));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.g.is_blocking(bs[0]));
    assert!(ability_targets(&mut t, mage, 0).contains(&obj(bs[0])));
    // Not after combat.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!ability_targets(&mut t, mage, 0).contains(&obj(bs[0])));
}

#[test]
fn dwarven_sea_clan_needs_the_island_on_activation_and_resolution() {
    cr!("115.1a", "608.2b", "603.7a");
    ruling!(
        "Dwarven Sea Clan",
        "You can't target a creature if its controller doesn't control an Island as you are activating the ability, and the ability won't resolve if the creature's controller doesn't control an Island as the ability resolves."
    );
    supported("Dwarven Sea Clan");
    let mut t = TestGame::new(2);
    let clan = t.battlefield(P0, "Dwarven Sea Clan");
    let a = t.battlefield(P1, "Hill Giant");
    blocked_by_p0(&mut t, &[a], &[]);
    assert!(!ability_targets(&mut t, clan, 0).contains(&obj(a)), "no Island");
    let island = t.battlefield(P1, "Island");
    assert!(ability_targets(&mut t, clan, 0).contains(&obj(a)));
    t.activate(P0, clan, 0, &[obj(a)]).unwrap();
    // The Island leaves in response: the ability's target is illegal as it resolves.
    cast_new(&mut t, P0, "Boomerang", &[obj(island)]);
    t.resolve();
    assert!(t.in_hand(P1, "Island"));
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(marked(&t, a), 0, "no damage at end of combat");
    assert_eq!(t.life(P0), 17);
    // With the Island staying, the 2 damage is dealt at end of combat.
    let mut t = TestGame::new(2);
    let clan = t.battlefield(P0, "Dwarven Sea Clan");
    t.battlefield(P1, "Island");
    let a = t.battlefield(P1, "Hill Giant");
    blocked_by_p0(&mut t, &[a], &[]);
    t.activate(P0, clan, 0, &[obj(a)]).unwrap();
    t.resolve_all();
    assert_eq!(marked(&t, a), 0);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(marked(&t, a), 2);
}

#[test]
fn plasma_caster_resolves_after_leaving_or_coming_unattached() {
    cr!("113.7a", "608.2h", "705.1");
    ruling!(
        "Plasma Caster",
        "If Plasma Caster is no longer on the battlefield or no longer attached to the same creature when its third ability resolves, the ability will still resolve as normal."
    );
    supported("Plasma Caster");
    for gone in [true, false] {
        let mut t = TestGame::new(2);
        let caster = t.battlefield(P0, "Plasma Caster");
        let a = t.battlefield(P0, "Hill Giant");
        assert!(t.g.attach(caster, obj(a)));
        let other = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        // Attacking gives {E}{E}.
        blocked(&mut t, &[a], &[(b, a)]);
        t.resolve_all();
        assert_eq!(player_counters(&t, P0, "energy"), 2);
        // Lose the flip: 1 damage to the blocker.
        crate::r_p017_common::load_coins(&mut t, &[false]);
        t.activate(P0, caster, 0, &[obj(b)]).unwrap();
        if gone {
            cast_new(&mut t, P1, "Disenchant", &[obj(caster)]);
            t.resolve();
            assert!(died(&t, caster));
        } else {
            // Equip to another creature in response (equip is sorcery-speed: reattach
            // directly).
            assert!(t.g.attach(caster, obj(other)));
            t.g.recompute();
        }
        t.resolve_all();
        assert_eq!(marked(&t, b), 1, "gone: {gone}");
    }
}

#[test]
fn maze_of_ith_and_ith_untap_an_attacker_after_damage_without_undoing_it() {
    cr!("506.4b", "615.4");
    ruling!(
        "Maze of Ith",
        "You can activate Maze of Ith's ability targeting an attacking creature you control during the combat damage step or the end of combat step, even though it will already have dealt combat damage. This will untap the creature. The damage it dealt will be unaffected."
    );
    ruling!(
        "Ith, High Arcanist",
        "You can activate Ith's ability targeting an attacking creature you control during the combat damage step or the end of combat step. It'll untap and the damage it had already dealt won't be undone."
    );
    for (name, step) in [
        ("Maze of Ith", Step::CombatDamage),
        ("Ith, High Arcanist", Step::CombatDamage),
        ("Maze of Ith", Step::EndOfCombat),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let a = t.battlefield(P0, "Hill Giant");
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(a, pl(P1))]));
        t.advance_to(P0, Step::BeginningOfCombat);
        t.advance_to(P0, step);
        assert_eq!(t.life(P1), 17, "{name}");
        assert!(tapped(&t, a));
        assert!(t.g.is_attacking(a));
        t.activate(P0, src, 0, &[obj(a)]).unwrap();
        t.resolve_all();
        assert!(!tapped(&t, a), "{name}");
        assert_eq!(t.life(P1), 17, "{name}: the damage stays");
    }
}

#[test]
fn ice_floe_may_target_a_tapped_attacker() {
    cr!("115.1a");
    ruling!(
        "Ice Floe",
        "You may target a tapped creature with Ice Floe’s activated ability."
    );
    supported("Ice Floe");
    let mut t = TestGame::new(2);
    let floe = t.battlefield(P0, "Ice Floe");
    let a = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[]);
    assert!(tapped(&t, a));
    assert!(ability_targets(&mut t, floe, 0).contains(&obj(a)));
    t.activate(P0, floe, 0, &[obj(a)]).unwrap();
    t.resolve_all();
    // It doesn't untap during P1's next untap step while Ice Floe stays tapped.
    t.advance_to(P1, Step::Upkeep);
    assert!(tapped(&t, a));
}

#[test]
fn forcefield_cant_stop_trample_damage_from_a_blocked_creature() {
    cr!("615.1", "509.1h", "702.19b");
    ruling!(
        "Forcefield",
        "This can’t be used to prevent damage caused by a blocked creature with Trample ability."
    );
    supported("Forcefield");
    // An unblocked attacker: all but 1 of its damage is prevented.
    let mut t = TestGame::new(2);
    let ff = t.battlefield(P0, "Forcefield");
    t.lands(P0, "Wastes", 1);
    let a = t.battlefield(P1, "Hill Giant");
    blocked_by_p0(&mut t, &[a], &[]);
    t.answer_choose(P0, &[obj(a)]);
    t.activate(P0, ff, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 19);
    // A blocked trampler (Grizzly Bears with Rancor: 4/2 trample) blocked by a 1/1: the
    // 3 damage that tramples over is dealt in full.
    let mut t = TestGame::new(2);
    let ff = t.battlefield(P0, "Forcefield");
    t.lands(P0, "Wastes", 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P1, "Rancor", a);
    let b = t.battlefield(P0, "Llanowar Elves");
    blocked_by_p0(&mut t, &[a], &[(b, a)]);
    let from = mark(&t);
    t.answer_choose(P0, &[obj(a)]);
    t.activate(P0, ff, 0, &[]).unwrap();
    t.resolve_all();
    for cands in crate::r_p017_common::choice_candidates(&t, from) {
        assert!(!cands.contains(&obj(a)), "a blocked creature isn't a choice");
    }
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn snag_may_be_cast_by_discarding_any_forest_card() {
    cr!("118.9", "205.3i", "305.6");
    ruling!(
        "Snag",
        "You may discard any card with the land type Forest, including Snow-covered Forest or non-Basic lands like Tropical Island, Murmuring Bosk, or Stomping Ground. You can even discard a card that is a land in addition to other types, such as Dryad Arbor."
    );
    supported("Snag");
    for forest in [
        "Snow-Covered Forest",
        "Tropical Island",
        "Murmuring Bosk",
        "Stomping Ground",
        "Dryad Arbor",
    ] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Hill Giant");
        blocked_by_p0(&mut t, &[a], &[]);
        let land = t.hand(P0, forest);
        let snag = t.hand(P0, "Snag");
        let method = crate::r_s34_common::alternative(&mut t, snag);
        t.answer_choose(P0, &[obj(land)]);
        t.cast(P0, snag).method(method).go();
        assert!(t.in_graveyard(P0, forest), "{forest}");
        t.resolve_all();
        t.advance_to(P1, Step::EndOfCombat);
        assert_eq!(t.life(P0), 20, "{forest}: unblocked damage prevented");
    }
    // A Plains isn't a Forest card.
    let mut t = TestGame::new(2);
    t.hand(P0, "Plains");
    let snag = t.hand(P0, "Snag");
    assert!(!crate::r_s07_common::cast_methods(&mut t, P0, snag)
        .iter()
        .any(|m| matches!(m, CastMethod::Alternative(_))));
}

#[test]
fn revealing_wind_and_hydrolash_may_be_cast_without_their_subjects() {
    cr!("608.2c", "615.1");
    ruling!(
        "Revealing Wind",
        "You can cast Revealing Wind even if there are no face-down creatures attacking or blocking. The damage-prevention effect will still apply."
    );
    ruling!(
        "Hydrolash",
        "You may cast Hydrolash even if there aren’t any attacking creatures simply to draw a card."
    );
    supported("Revealing Wind");
    supported("Hydrolash");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[(b, a)]);
    cast_new(&mut t, P0, "Revealing Wind", &[]);
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(marked(&t, a), 0);
    assert_eq!(marked(&t, b), 0);
    assert!(t.on_battlefield(b));
    // Hydrolash in a main phase.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Hydrolash", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn flash_foliage_is_cast_only_after_blockers_are_declared() {
    cr!("601.3", "304.1", "508.1", "509.1");
    ruling!(
        "Flash Foliage",
        "With the latest Oracle update, you can only cast this during combat, after blockers have been declared."
    );
    supported("Flash Foliage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast_now(&mut t, P0, "Flash Foliage"));
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, pl(P0))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!can_cast_now(&mut t, P0, "Flash Foliage"));
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(can_cast_now(&mut t, P0, "Flash Foliage"));
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Flash Foliage", &[obj(a)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(is_blocked(&t, a));
    assert_eq!(t.hand_size(P0), hand + 1 - 1 + 1, "draw a card");
}

#[test]
fn choking_vines_may_be_cast_with_x_zero_and_no_targets() {
    cr!("601.2c", "107.3", "115.1a");
    ruling!("Choking Vines", "You can declare zero targets, with X=0.");
    supported("Choking Vines");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    blocked_by_p0(&mut t, &[a], &[]);
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Choking Vines");
    let spell = t.cast(P0, card).x(0).try_go().expect("cast with X = 0");
    assert!(crate::r_p006_common::targets_of(&t, spell).is_empty());
    t.resolve_all();
    assert!(!is_blocked(&t, a));
    assert_eq!(marked(&t, a), 0);
}

#[test]
fn glyph_of_life_may_target_an_opponents_wall() {
    cr!("115.1a", "603.7a");
    ruling!(
        "Glyph of Life",
        "You can cast this targeting your opponent’s walls to good effect."
    );
    supported("Glyph of Life");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Wood");
    let a = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Glyph of Life", &[obj(wall)]);
    t.resolve_all();
    blocked(&mut t, &[a], &[(wall, a)]);
    finish_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(died(&t, wall));
}
