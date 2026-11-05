//! Rulings batch P087 — cards that hate on a mechanic ("hate-set-mechanic"), part 1:
//! monstrous, non-mana untap abilities, multicolored permanents, snow, untap locks,
//! uncounterable spells, Equipment, "if it would die" exile, until-leaves exile,
//! Adventures, graveyard targeting, warp, mana spent versus mana value, morph, token
//! targets and other players' graveyards.

use crate::r_p087_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn polis_crusher_must_be_monstrous_as_it_deals_damage_and_the_trigger_outlives_it() {
    cr!("603.4", "701.37b", "603.2");
    ruling!(
        "Polis Crusher",
        "Polis Crusher must be monstrous as it deals combat damage to a player in order for its last ability to trigger. Once the ability triggers, it resolves even if Polis Crusher isn't on the battlefield at that time."
    );
    supported("Polis Crusher");
    // Not monstrous: no trigger.
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Polis Crusher");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.attack(&[(crusher, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(anthem));
    // Monstrous: the trigger destroys the enchantment, even after Crusher has left.
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Polis Crusher");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    add_mana(&mut t, P0, ManaType::C, 4);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::G, 1);
    activate(&mut t, P0, crusher, "Monstrosity", &[]);
    t.resolve_all();
    assert_eq!(t.counters(crusher, counters::PLUS1), 3);
    t.answer_targets(P0, &[obj(anthem)]);
    to_blockers(&mut t, &[(crusher, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 13);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, crusher);
    assert!(!t.on_battlefield(crusher));
    t.resolve_all();
    assert!(!t.on_battlefield(anthem));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
}

#[test]
fn portent_tracker_untap_isnt_a_mana_ability() {
    cr!("605.1a", "605.3a", "601.2g");
    ruling!(
        "Portent Tracker",
        "Portent Tracker’s first ability isn’t a mana ability. Notably, it can’t be used while you’re activating an ability or casting a spell. If you plan to use it to produce enough mana to activate an ability or cast a spell, you must do so ahead of time."
    );
    supported("Portent Tracker");
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Portent Tracker");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    // One land can't pay {1}{G}: the untap can't be used during payment.
    assert!(!castable(&mut t, P0, bears));
    // Ahead of time: tap the Forest for mana, untap it with the Tracker (the ability uses
    // the stack), and tap it again while casting.
    assert!(tap_for_mana(&mut t, P0, forest, "Add"));
    activate(&mut t, P0, tracker, "Untap target land", &[obj(forest)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!is_tapped(&t, forest));
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
}

fn multicolored(g: &mtg_engine::game::Game, p: PlayerId) -> usize {
    g.permanents()
        .filter(|o| o.controller == p && o.chars.colors.is_multicolored())
        .count()
}

#[test]
fn renounce_the_guilds_doesnt_target_and_sacrifices_simultaneously_in_apnap_order() {
    cr!("101.4", "701.21a", "115.1");
    ruling!(
        "Renounce the Guilds",
        "Renounce the Guilds doesn’t target any player and may be cast even if a player doesn’t control a multicolored permanent."
    );
    ruling!(
        "Renounce the Guilds",
        "The active player (the player whose turn it is) chooses which multicolored permanent will be sacrificed, then each other player in turn order does the same. Then all chosen permanents are sacrificed at the same time."
    );
    supported("Renounce the Guilds");
    // No multicolored permanents: it can be cast and does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(could_cast(&mut t, P0, "Renounce the Guilds"));
    cast_new(&mut t, P0, "Renounce the Guilds", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // No targets: a player with hexproof (Leyline of Sanctity) still sacrifices.
    // Each player chooses in APNAP order; the sacrifices happen together.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Leyline of Sanctity");
    let wolf0 = t.battlefield(P0, "Watchwolf");
    let blade0 = t.battlefield(P0, "Boros Swiftblade");
    let wolf1 = t.battlefield(P1, "Watchwolf");
    let blade1 = t.battlefield(P1, "Boros Swiftblade");
    let wolf2 = t.battlefield(P2, "Watchwolf");
    t.answer_choose(P0, &[obj(wolf0)]);
    t.answer_choose(P1, &[obj(blade1)]);
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| multicolored(g, P0),
    );
    let from = t.asked().len();
    cast_new(&mut t, P0, "Renounce the Guilds", &[]);
    t.resolve_all();
    // P0 chose first; when P1 chose, nothing had been sacrificed yet.
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![2]);
    assert!(!t.on_battlefield(wolf0) && t.on_battlefield(blade0));
    assert!(t.on_battlefield(wolf1) && !t.on_battlefield(blade1));
    // P2's only multicolored permanent goes without a choice.
    assert!(!t.on_battlefield(wolf2));
}

#[test]
fn ronom_hulk_protection_from_snow() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f", "205.4g");
    ruling!(
        "Ronom Hulk",
        "Ronom Hulk can’t be targeted by snow spells"
    );
    ruling!(
        "Ronom Hulk",
        "or abilities of snow sources, it can’t be enchanted by snow Auras, it can’t be equipped by snow Equipment (though there are none), it can’t be blocked by snow creatures, and all damage that would be dealt to it by snow sources is prevented."
    );
    supported("Ronom Hulk");
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Ronom Hulk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A snow instant (Frost Bite, Kaldheim).
    let c = spell_targets(&mut t, P1, "Frost Bite");
    assert!(c.contains(&obj(bears)) && !c.contains(&obj(hulk)));
    assert!(spell_targets(&mut t, P1, "Shock").contains(&obj(hulk)));
    // An ability of a snow source (Priest of the Haunted Edge, a snow creature).
    let priest = t.battlefield(P1, "Priest of the Haunted Edge");
    let c = ability_targets(&mut t, priest, 0);
    assert!(c.contains(&obj(bears)) && !c.contains(&obj(hulk)));
    // A snow Aura (Glacial Plating) can't enchant it.
    let plating = t.battlefield(P1, "Glacial Plating");
    assert!(!t.g.attach(plating, obj(hulk)));
    assert!(t.g.attach(plating, obj(bears)));
    // Damage from a snow source is prevented; from another source it isn't.
    let musher = t.battlefield(P1, "Zombie Musher");
    t.g.deal_damage(musher, obj(hulk), 3, false);
    t.g.flush_events();
    assert_eq!(damage_marked(&t, hulk), 0);
    let ogre = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(ogre, obj(hulk), 3, false);
    t.g.flush_events();
    assert_eq!(damage_marked(&t, hulk), 3);
    // A snow creature can't block it.
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(hulk, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(musher, hulk)]));
    assert!(legal_blocks(&mut t, P1, &[(giant, hulk)]));
}

#[test]
fn rust_tick_targets_tapped_artifacts_and_doesnt_track_the_controller() {
    cr!("502.3", "611.2a", "115.1");
    ruling!(
        "Rust Tick",
        "You may target a tapped artifact with Rust Tick’s activated ability."
    );
    ruling!(
        "Rust Tick",
        "Rust Tick doesn’t track the artifact’s controller. If the affected artifact changes controllers, Rust Tick will prevent it from being untapped during its new controller’s untap step."
    );
    supported("Rust Tick");
    let mut t = TestGame::new(3);
    let tick = t.battlefield(P0, "Rust Tick");
    let thopter = t.battlefield(P1, "Ornithopter");
    tap(&mut t, thopter);
    assert!(ability_targets(&mut t, tick, 0).contains(&obj(thopter)));
    t.lands(P0, "Wastes", 1);
    activate(&mut t, P0, tick, "Tap target artifact", &[obj(thopter)]);
    t.resolve_all();
    assert!(is_tapped(&t, tick) && is_tapped(&t, thopter));
    // P2 gains control of it: it stays tapped during P2's untap step.
    give_control(&mut t, thopter, P2);
    next_upkeep(&mut t, P2);
    assert_eq!(t.obj_now(thopter).controller, P2);
    assert!(is_tapped(&t, tick));
    assert!(is_tapped(&t, thopter));
}

#[test]
fn aven_interrupter_exiles_a_spell_that_cant_be_countered() {
    cr!("701.5a", "702.170a", "608.2c");
    ruling!(
        "Aven Interrupter",
        "Spells that can't be countered can still be exiled by Aven Interrupter's triggered ability. They won't resolve."
    );
    supported("Aven Interrupter");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let baloth = cast_new(&mut t, P1, "Frenzied Baloth", &[]);
    t.answer_targets(P0, &[obj(baloth)]);
    t.enter(P0, "Aven Interrupter");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Frenzied Baloth"));
    assert!(t.named_on_battlefield("Frenzied Baloth").is_empty());
}

#[test]
fn disarm_leaves_the_equipment_on_the_battlefield_unattached() {
    cr!("701.3d", "301.5c");
    ruling!(
        "Disarm",
        "The Equipment remains on the battlefield under its controller’s control, but is no longer attached to that creature."
    );
    supported("Disarm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let eq = attach_new(&mut t, P1, "Bonesplitter", bears);
    cast_new(&mut t, P0, "Disarm", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(eq));
    assert_eq!(attached_to(&t, eq), None);
    assert_eq!(t.obj_now(eq).controller, P1);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn nine_ringed_bo_exile_works_without_the_bo() {
    cr!("614.1a", "614.6");
    ruling!(
        "Nine-Ringed Bo",
        "The damaged creature will be exiled even if Nine-Ringed Bo isn't on the battlefield when that creature would be put into a graveyard."
    );
    supported("Nine-Ringed Bo");
    let mut t = TestGame::new(2);
    let bo = t.battlefield(P0, "Nine-Ringed Bo");
    let kami = t.battlefield(P1, "Kami of Ancient Law");
    activate(&mut t, P0, bo, "damage", &[obj(kami)]);
    t.resolve_all();
    assert_eq!(damage_marked(&t, kami), 1);
    destroy(&mut t, bo);
    destroy(&mut t, kami);
    assert!(t.in_exile("Kami of Ancient Law"));
    assert!(!t.in_graveyard(P1, "Kami of Ancient Law"));
}

#[test]
fn consulate_crackdown_returns_the_artifacts_together_immediately() {
    cr!("610.3a", "610.3", "603.2c");
    ruling!(
        "Consulate Crackdown",
        "The exiled cards return to the battlefield immediately after Consulate Crackdown leaves the battlefield. Nothing happens between the two events, including state-based actions. If Consulate Crackdown exiles multiple artifacts, those cards all return to the battlefield at the same time."
    );
    supported("Consulate Crackdown");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let sable = t.battlefield(P1, "Bronze Sable");
    let crack = t.enter(P0, "Consulate Crackdown");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(thopter)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(sable)), Zone::Exile);
    // Ingenious Artillerist: "Whenever one or more artifacts you control enter, this
    // creature deals that much damage to each opponent."
    t.battlefield(P1, "Ingenious Artillerist");
    t.g.destroy(crack, None);
    t.g.flush_events();
    // Back as part of the event, before state-based actions or triggers are handled.
    assert!(t.on_battlefield(thopter) && t.on_battlefield(sable));
    t.settle();
    // One trigger for both: they entered at the same time.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn sentinel_of_lost_lore_adventure_modes_and_the_graveyard_mode() {
    cr!("700.2a", "700.2b", "715.3", "603.3c", "115.4");
    ruling!(
        "Sentinel of Lost Lore",
        "The first and second modes of Sentinel of Lost Lore's ability can target any face-up exiled card that has an Adventure as long as it's owned by an appropriate player. It doesn't matter whether the card was cast as an Adventure."
    );
    ruling!(
        "Sentinel of Lost Lore",
        "You must choose at least one mode if you can. If there are no legal targets available for the first two modes, you must choose the last mode. Notably, the last mode can target any player, even one who has no cards in their graveyard. In the unusual case there are no appropriate cards in exile to target and no player may legally be chosen as a target, the ability is removed from the stack with no effect."
    );
    supported("Sentinel of Lost Lore");
    // Adventurer cards exiled some other way (not cast as Adventures).
    let mut t = TestGame::new(2);
    let mine = t.exile(P0, "Bonecrusher Giant // Stomp");
    let theirs = t.exile(P1, "Bonecrusher Giant // Stomp");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    t.enter(P0, "Sentinel of Lost Lore");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(mine)), Zone::Hand(P0));
    assert_eq!(t.g.player(P1).library.first(), Some(&t.g.current(theirs)));
    // Nothing in exile: only the last mode, at a player with an empty graveyard.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Sentinel of Lost Lore");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(crate::r_s07_common::chosen_modes(&t, trig), vec![2]);
    assert_eq!(crate::r_s25_common::targets_of(&t, trig), vec![Entity::Player(P1)]);
    let _ = from;
    t.resolve_all();
    // No player can be targeted (both have shroud from True Believer): removed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "True Believer");
    t.battlefield(P1, "True Believer");
    t.enter(P0, "Sentinel of Lost Lore");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ground_seal_works_only_on_the_battlefield() {
    cr!("115.4", "113.6");
    ruling!(
        "Ground Seal",
        "The last ability only works when Ground Seal is on the battlefield."
    );
    supported("Ground Seal");
    for zone in [Zone::Graveyard(P1), Zone::Hand(P1), Zone::Battlefield] {
        let mut t = TestGame::new(2);
        match zone {
            Zone::Battlefield => {
                t.battlefield(P1, "Ground Seal");
            }
            Zone::Hand(_) => {
                t.hand(P1, "Ground Seal");
            }
            _ => {
                t.graveyard(P1, "Ground Seal");
            }
        }
        t.graveyard(P0, "Grizzly Bears");
        assert_eq!(
            could_cast(&mut t, P0, "Raise Dead"),
            zone != Zone::Battlefield,
            "{zone:?}"
        );
    }
}

#[test]
fn blade_of_the_swarm_targets_any_exiled_warp_card() {
    cr!("702.185a", "115.1");
    ruling!(
        "Blade of the Swarm",
        "The last mode of Blade of the Swarm’s ability can target any card in exile that has the warp ability, not just one that was exiled due to the delayed triggered ability created by a warp ability."
    );
    supported("Blade of the Swarm");
    let mut t = TestGame::new(2);
    let whale = t.exile(P1, "Starbreach Whale");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(whale)]);
    t.enter(P0, "Blade of the Swarm");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.player(P1).library.first(), Some(&t.g.current(whale)));
}

#[test]
fn unravel_compares_mana_spent_with_the_unchanged_mana_value() {
    cr!("202.3", "202.3d", "601.2f", "702.41a");
    ruling!(
        "Unravel",
        "The mana value of a spell isn’t changed by alternative costs, cost increases, or cost reductions. For example, if you cast Thrumming Hivepool (an artifact with affinity for Slivers), its mana value is 6 no matter how many Slivers you controlled when you cast it."
    );
    supported("Unravel");
    supported("Thrumming Hivepool");
    for (spell, slivers) in [("Thrumming Hivepool", 2), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        for _ in 0..slivers {
            t.battlefield(P0, "Muscle Sliver");
        }
        let card = t.hand(P0, spell);
        // Affinity: Hivepool costs {4} with two Slivers.
        let generic = if slivers > 0 { 4 } else { 1 };
        t.lands(P0, "Wastes", generic);
        if spell == "Grizzly Bears" {
            t.lands(P0, "Forest", 1);
        }
        let s = t.cast(P0, card).go();
        assert_eq!(crate::r_s26_common::mv(&mut t, s), if slivers > 0 { 6 } else { 2 });
        let hand = t.hand_size(P1);
        cast_new(&mut t, P1, "Unravel", &[obj(s)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, spell));
        // 4 mana spent on a mana value 6 spell: a card; 2 on Grizzly Bears: none.
        assert_eq!(t.hand_size(P1) - hand, usize::from(slivers > 0), "{spell}");
    }
}

#[test]
fn timebender_mode_and_target_are_chosen_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3c", "700.2b");
    ruling!(
        "Timebender",
        "The mode and target of the triggered ability are chosen when it's put on the stack."
    );
    supported("Timebender");
    let mut t = TestGame::new(2);
    let fb = crate::r_s12_common::morph(&mut t, P0, "Timebender");
    let clock = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, clock, counters::TIME, 3);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(clock)]);
    t.lands(P0, "Island", 1);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fb));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(crate::r_s07_common::chosen_modes(&t, trig), vec![1]);
    assert_eq!(crate::r_s25_common::targets_of(&t, trig), vec![obj(clock)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. } | Decision::ChooseTargets { .. })));
    assert_eq!(t.counters(clock, counters::TIME), 5);
}

#[test]
fn break_open_turns_a_creature_face_up_without_its_morph_cost() {
    cr!("701.40a", "708.8");
    ruling!(
        "Break Open",
        "The player does not have to pay the Morph cost (if any) of the creature."
    );
    supported("Break Open");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let angel = crate::r_s12_common::morph(&mut t, P1, "Exalted Angel");
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P1), 0);
    cast_new(&mut t, P0, "Break Open", &[obj(angel)]);
    t.resolve_all();
    assert!(!t.obj_now(angel).face_down);
    assert_eq!(t.obj_now(angel).chars.name, "Exalted Angel");
    assert_eq!(t.pt(angel), (4, 5));
}

#[test]
fn conduct_electricity_can_target_one_token_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Conduct Electricity",
        "The two targets can be the same creature as long as that creature is a token."
    );
    supported("Conduct Electricity");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    let tok = token_copy(&mut t, P1, colossus)[0];
    let from = t.asked().len();
    cast_new(&mut t, P0, "Conduct Electricity", &[obj(tok), obj(tok)]);
    // A nontoken creature isn't a legal second target.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().any(|c| c.contains(&obj(colossus))));
    assert!(offered.iter().any(|c| c.contains(&obj(tok)) && !c.contains(&obj(colossus))));
    t.resolve_all();
    assert_eq!(damage_marked(&t, tok), 8);
    assert_eq!(damage_marked(&t, colossus), 0);
}

#[test]
fn shamans_trance_doesnt_let_you_activate_abilities_in_other_graveyards() {
    cr!("601.3", "602.2");
    ruling!(
        "Shaman's Trance",
        "This card does not allow you to activate the abilities of cards in other people’s graveyards."
    );
    supported("Shaman's Trance");
    let mut t = TestGame::new(2);
    let skel = t.graveyard(P1, "Reassembling Skeleton");
    let think = t.graveyard(P1, "Think Twice");
    cast_new(&mut t, P0, "Shaman's Trance", &[]);
    t.resolve_all();
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Island", 3);
    // P0 may cast Think Twice from P1's graveyard (flashback, as though it were in P0's
    // graveyard), but not activate the Skeleton's ability.
    assert!(castable(&mut t, P0, think));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, skel));
    // Its owner still can.
    t.lands(P1, "Swamp", 2);
    assert!(crate::r_s02_common::can_activate(&mut t, P1, skel));
}
