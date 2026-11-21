//! Rulings residue `rulings-residue-hate-attack-block` (P079, P080): cards that punish or
//! answer attackers — Heavy Fog, Disharmony, Giant Trap Door Spider, Stalking Leonin,
//! Sandswirl Wanderglyph, Trouble in Pairs, Dwarven Sea Clan, Suppressor Skyguard, Veteran
//! Bodyguard, Weakstone, Broken Visage and Singing Tree.

use crate::r_p079_common::{back_face, can_cast_now, cast_card, concede};
use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::ability_targets;
use crate::r_s05_common::move_to;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Heavy Fog
// ---------------------------------------------------------------------------

#[test]
fn heavy_fog_needs_a_creature_attacking_you() {
    cr!("506.2", "508.1b", "601.3");
    ruling!(
        "Heavy Fog",
        "If all the attacking creatures attack your planeswalkers, you can’t cast Heavy Fog. To cast it, a creature needs to have attacked _you_."
    );
    supported("Heavy Fog");
    for at_you in [false, true] {
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P1, "Jace Beleren");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let to = if at_you { Entity::Player(P1) } else { obj(pw) };
        attacks(&mut t, &[(bears, to)]);
        crate::r_s25_common::lands_for_cost(&mut t, P1, "Heavy Fog");
        let fog = t.hand(P1, "Heavy Fog");
        assert_eq!(can_cast_now(&mut t, P1, fog), at_you);
    }
}

#[test]
fn heavy_fog_prevents_only_damage_to_you() {
    cr!("615.1", "510.2");
    ruling!(
        "Heavy Fog",
        "Heavy Fog prevents only damage that would be dealt to you by attacking creatures. It won’t prevent damage attacking creatures would deal to creatures or planeswalkers you control."
    );
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Jace Beleren");
    let wall = t.battlefield(P1, "Wall of Wood");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attacks(
        &mut t,
        &[
            (giant, Entity::Player(P1)),
            (bears, obj(pw)),
            (wurm, Entity::Player(P1)),
        ],
    );
    cast_card(&mut t, P1, "Heavy Fog", &[]).unwrap();
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, wurm)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(pw, counters::LOYALTY), 1);
    assert!(!t.on_battlefield(wall));
}

#[test]
fn heavy_fog_prevents_noncombat_damage_from_an_attacking_creature() {
    cr!("615.1", "608.2h");
    ruling!(
        "Heavy Fog",
        "Heavy Fog prevents all damage attacking creatures would deal to you this turn, not just combat damage. Noncombat damage from those creatures is prevented only if they’re still attacking at the time that damage would be dealt or, if the creature has left the battlefield by then, if it was an attacking creature at the time it left."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let attacking_fanatic = t.battlefield(P0, "Mogg Fanatic");
    let home_fanatic = t.battlefield(P0, "Mogg Fanatic");
    attacks(
        &mut t,
        &[
            (giant, Entity::Player(P1)),
            (attacking_fanatic, Entity::Player(P1)),
        ],
    );
    cast_card(&mut t, P1, "Heavy Fog", &[]).unwrap();
    t.resolve_all();
    // The attacking Fanatic is sacrificed: it was attacking as it left, so its damage is
    // prevented.
    t.activate(P0, attacking_fanatic, 0, &[Entity::Player(P1)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    // The Fanatic that didn't attack deals its damage.
    t.activate(P0, home_fanatic, 0, &[Entity::Player(P1)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------
// Disharmony
// ---------------------------------------------------------------------------

#[test]
fn disharmony_untaps_removes_from_combat_and_steals_until_end_of_turn() {
    cr!("506.4", "701.26b", "611.2a");
    ruling!(
        "Disharmony",
        "Disharmony works even if the attacker was not tapped to attack."
    );
    ruling!(
        "Disharmony",
        "The current wording explicitly removes the creature from combat. It is no longer an attacking or blocking creature."
    );
    supported("Disharmony");
    // A Serra Angel (vigilance, untapped) and a tapped Hill Giant.
    for name in ["Serra Angel", "Hill Giant"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, name);
        attacks(&mut t, &[(a, Entity::Player(P1))]);
        assert_eq!(tapped(&t, a), name == "Hill Giant");
        cast_card(&mut t, P1, "Disharmony", &[obj(a)]).unwrap();
        t.resolve_all();
        assert!(!attacking(&t, a), "{name}");
        assert!(!tapped(&t, a), "{name}");
        assert_eq!(controller(&t, a), P1, "{name}");
        no_blocks(&mut t, P1);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}");
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(controller(&t, a), P0, "{name}: until end of turn");
    }
}

#[test]
fn disharmony_untaps_the_creature_before_you_gain_control_of_it() {
    // Servant of Tymaret's inspired ability ("Whenever this creature becomes untapped,
    // each opponent loses 1 life. You gain life equal to the life lost this way.")
    // triggers as Disharmony untaps it, while P0 still controls it: P0 controls the
    // trigger (CR 603.3a), so P1, who cast Disharmony and now controls the Servant, is the
    // one who loses the life.
    cr!("608.2c", "603.3a");
    ruling!(
        "Disharmony",
        "The creature untaps (if it was tapped) before you get control of it."
    );
    supported("Servant of Tymaret");
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Servant of Tymaret");
    attacks(&mut t, &[(servant, Entity::Player(P1))]);
    assert!(tapped(&t, servant));
    cast_card(&mut t, P1, "Disharmony", &[obj(servant)]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&t, servant), P1);
    assert!(!tapped(&t, servant));
    assert_eq!((t.life(P0), t.life(P1)), (21, 19), "{}", t.dump_log());
}

#[test]
fn disharmony_can_target_your_own_attacker() {
    cr!("115.1", "506.4");
    ruling!(
        "Disharmony",
        "Disharmony can target a creature you control."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    cast_card(&mut t, P0, "Disharmony", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!attacking(&t, giant) && !tapped(&t, giant));
    assert_eq!(controller(&t, giant), P0);
}

#[test]
fn disharmony_on_an_opponents_turn_lets_you_block_with_the_creature() {
    cr!("509.1a", "506.4");
    ruling!(
        "Disharmony",
        "If you cast Disharmony during an opponent’s turn, you may then block with the creature you gain control of (assuming there’s an attacking creature it can block)."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attacks(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    cast_card(&mut t, P1, "Disharmony", &[obj(giant)]).unwrap();
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(t.g.current(giant), bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocking(&t, giant), vec![bears], "{}", t.dump_log());
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Giant Trap Door Spider
// ---------------------------------------------------------------------------

/// P0 attacks P1 with a Hill Giant; P1 activates Giant Trap Door Spider targeting it (the
/// ability on the stack). (Spider, Giant.)
fn spider_on_giant(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let spider = t.battlefield(P1, "Giant Trap Door Spider");
    for land in ["Mountain", "Forest", "Wastes"] {
        t.lands(P1, land, 1);
    }
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, spider, 0, &[obj(giant)]).unwrap();
    (spider, giant)
}

#[test]
fn giant_trap_door_spider_exiles_the_target_even_if_it_left() {
    cr!("608.2b", "113.7a");
    ruling!(
        "Giant Trap Door Spider",
        "If Giant Trap Door Spider leaves the battlefield before its ability resolves, the ability will still resolve as normal. The targeted creature will be exiled."
    );
    let mut t = TestGame::new(2);
    let (spider, giant) = spider_on_giant(&mut t);
    move_to(&mut t, spider, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.on_battlefield(giant));
    assert!(t.in_hand(P1, "Giant Trap Door Spider"));
}

#[test]
fn giant_trap_door_spider_stays_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Giant Trap Door Spider",
        "If the targeted creature leaves the battlefield (or otherwise becomes an illegal target) before the ability resolves, the ability won’t resolve. Giant Trap Door Spider will remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    let (spider, giant) = spider_on_giant(&mut t);
    move_to(&mut t, giant, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(spider));
}

// ---------------------------------------------------------------------------
// Stalking Leonin
// ---------------------------------------------------------------------------

#[test]
fn stalking_leonin_cant_target_a_creature_attacking_your_planeswalker() {
    cr!("506.2", "508.1b", "115.1");
    ruling!(
        "Stalking Leonin",
        "If a creature is attacking a planeswalker you control, it's not attacking you. Stalking Leonin's last ability can't target it."
    );
    supported("Stalking Leonin");
    let mut t = TestGame::new(2);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    let leonin = t.enter(P1, "Stalking Leonin");
    t.resolve_all();
    let pw = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(bears, obj(pw)), (giant, Entity::Player(P1))]);
    let c = ability_targets(&mut t, leonin, 0);
    assert!(!c.contains(&obj(bears)));
    assert!(c.contains(&obj(giant)));
}

#[test]
fn stalking_leonin_returned_is_a_new_object_with_a_new_choice() {
    cr!("400.7");
    ruling!(
        "Stalking Leonin",
        "If Stalking Leonin leaves the battlefield and returns, it's considered a new object. You may choose a new opponent and you may activate its last ability again."
    );
    let mut t = TestGame::new(3);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    let leonin = t.enter(P1, "Stalking Leonin");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    attacks(&mut t, &[(bears, Entity::Player(P1))]);
    t.activate(P1, leonin, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Activated once (its limit).
    assert_eq!(t.obj_now(leonin).activations.values().sum::<u32>(), 1);
    // It leaves and returns: P1 chooses P2 this time and may activate it again.
    move_to(&mut t, leonin, Zone::Exile);
    t.answer_choose(P1, &[Entity::Player(P2)]);
    let back = move_to(&mut t, leonin, Zone::Battlefield).unwrap();
    t.resolve_all();
    assert_eq!(t.g.obj(back).choices.player, Some(P2));
    assert!(t.g.obj(back).activations.is_empty());
    let wurm = t.battlefield(P2, "Craw Wurm");
    t.advance_to(P2, Step::BeginningOfCombat);
    attacks(&mut t, &[(wurm, Entity::Player(P1))]);
    t.activate(P1, back, 0, &[obj(wurm)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Sandswirl Wanderglyph, Trouble in Pairs, Suppressor Skyguard
// ---------------------------------------------------------------------------

#[test]
fn sandswirl_wanderglyph_restriction_ends_when_it_leaves() {
    cr!("611.3a", "604.2");
    ruling!(
        "Unstable Glyphbridge // Sandswirl Wanderglyph",
        "If Sandswirl Wanderglyph leaves the battlefield on a turn in which an opponent attacked you or a planeswalker you control (perhaps because it died in combat or another player cast a spell that destroyed it), that opponent can cast spells."
    );
    let mut t = TestGame::new(2);
    let glyph = back_face(&mut t, P1, "Unstable Glyphbridge // Sandswirl Wanderglyph");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Shock");
    let shock = t.hand(P0, "Shock");
    attacks(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!can_cast_now(&mut t, P0, shock));
    crate::r_s02_common::destroy(&mut t, glyph);
    assert!(can_cast_now(&mut t, P0, shock));
}

#[test]
fn trouble_in_pairs_triggers_in_each_combat() {
    cr!("508.3e", "500.8");
    ruling!(
        "Trouble in Pairs",
        "If an opponent attacks you with two or more creatures more than once in a turn (probably because they've generated extra combat phases), Trouble in Pairs's last ability will trigger each time that happens."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trouble in Pairs");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P1);
    let both = vec![(a, Entity::Player(P1)), (b, Entity::Player(P1))];
    attacks(&mut t, &both);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::PostcombatMain);
    // Relentless Assault: the attackers untap and there's an additional combat phase.
    cast_card(&mut t, P0, "Relentless Assault", &[]).unwrap();
    t.resolve_all();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(both));
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.hand_size(P1), hand + 2, "{}", t.dump_log());
    assert_eq!(t.life(P1), 8);
}

#[test]
fn suppressor_skyguard_checks_its_intervening_if_on_resolution() {
    cr!("603.4", "800.4a");
    ruling!(
        "Suppressor Skyguard",
        "If the \"intervening if\" clause of Suppressor Skyguard's last ability is no longer met when the ability tries to resolve, it won't resolve and none of its effects will happen."
    );
    for p2_leaves in [false, true] {
        let mut t = TestGame::new(3);
        t.battlefield(P1, "Suppressor Skyguard");
        let giant = t.battlefield(P0, "Hill Giant");
        attacks(&mut t, &[(giant, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        if p2_leaves {
            concede(&mut t, P2);
        }
        t.resolve_all();
        no_blocks(&mut t, P1);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), if p2_leaves { 17 } else { 20 });
    }
}

// ---------------------------------------------------------------------------
// Dwarven Sea Clan, Veteran Bodyguard
// ---------------------------------------------------------------------------

#[test]
fn dwarven_sea_clan_asks_about_the_targets_new_controller() {
    cr!("608.2b", "506.4");
    ruling!(
        "Dwarven Sea Clan",
        "If control of the targeted creature changes between the activating and resolution of the ability (ie, while it's on the stack), then the player who must control an Island as the ability resolves is the creature's new controller."
    );
    supported("Dwarven Sea Clan");
    // P0 (who controls an Island) attacks with a Hill Giant; P1 targets it with Dwarven
    // Sea Clan; P1 (who controls no Island) gains control of it in response: the target
    // is illegal (its controller controls no Island; it also left combat, CR 506.4), so
    // no damage is dealt at end of combat.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let clan = t.battlefield(P1, "Dwarven Sea Clan");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, clan, 0, &[obj(giant)]).unwrap();
    give_control(&mut t, giant, P1);
    t.resolve_all();
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0, "{}", t.dump_log());
    // Without the control change, the Giant is dealt 2 damage at end of combat.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let clan = t.battlefield(P1, "Dwarven Sea Clan");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, clan, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    no_blocks(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2, "{}", t.dump_log());
}

#[test]
fn several_veteran_bodyguards_their_controller_chooses_which_each_time() {
    cr!("616.1", "616.1e");
    ruling!(
        "Veteran Bodyguard",
        "If you have multiple Veteran Bodyguards, you can decide which one receives the redirected damage each time damage would be dealt to you."
    );
    supported("Veteran Bodyguard");
    let mut damaged = Vec::new();
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        let g1 = t.battlefield(P1, "Veteran Bodyguard");
        let g2 = t.battlefield(P1, "Veteran Bodyguard");
        let giant = t.battlefield(P0, "Hill Giant");
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        attacks(&mut t, &[(giant, Entity::Player(P1))]);
        no_blocks(&mut t, P1);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20);
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseReplacement { .. })));
        let d = (t.obj_now(g1).damage, t.obj_now(g2).damage);
        assert!(d == (3, 0) || d == (0, 3), "{d:?}");
        damaged.push(d);
    }
    assert_ne!(damaged[0], damaged[1], "the choice decides which");
}

// ---------------------------------------------------------------------------
// Restatements: Weakstone, Broken Visage, Singing Tree
// ---------------------------------------------------------------------------

#[test]
fn weakstone_affects_every_players_attackers() {
    cr!("611.3a");
    ruling!(
        "Weakstone",
        "The -1/-0 applies to attacking creatures from all players."
    );
    supported("Weakstone");
    for attacker in [P0, P1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Weakstone");
        let giant = t.battlefield(attacker, "Hill Giant");
        t.set_step(attacker, Step::BeginningOfCombat);
        let defender = if attacker == P0 { P1 } else { P0 };
        attacks(&mut t, &[(giant, Entity::Player(defender))]);
        assert_eq!(t.pt(giant), (2, 3), "{attacker}");
    }
}

#[test]
fn broken_visage_token_is_yours_with_the_attackers_last_power_and_toughness() {
    cr!("608.2h", "111.2");
    ruling!(
        "Broken Visage",
        "The Spirit token has the power and toughness of the attacking creature as it last existed on the battlefield."
    );
    ruling!(
        "Broken Visage",
        "The token creature enters under your control regardless of who controlled the targeted creature."
    );
    supported("Broken Visage");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    cast_card(&mut t, P0, "Giant Growth", &[obj(giant)]).unwrap();
    t.resolve_all();
    cast_card(&mut t, P1, "Broken Visage", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    let toks = crate::r_s01_common::tokens(&t, P1);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (6, 6));
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
}

#[test]
fn singing_tree_sets_base_power_and_later_pumps_still_count() {
    // The ruling's "current power to zero" is outdated for a creature pumped before the
    // ability resolves (docs/RULING_DISCREPANCIES.md; exempt in rulings-P079.tsv): see
    // the next test. A creature not pumped before gets power 0, and a later pump counts.
    cr!("613.4b", "613.4c");
    ruling!(
        "Singing Tree",
        "does not prevent raising it after the Tree has been used on it"
    );
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P1, "Singing Tree");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    t.activate(P1, tree, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (0, 3));
    cast_card(&mut t, P0, "Giant Growth", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 6));
}

#[test]
fn singing_tree_after_a_pump_sets_only_the_base_power() {
    // The Oracle text sets base power (layer 7b): a +3/+3 from before still applies
    // (layer 7c), whatever the timestamps (CR 613.4b, 613.4c). The 2005 ruling's "Changes
    // creature's current power to zero" was right under the layer rules of its time
    // (sublayer 6b applied both effects in timestamp order) and is outdated since the
    // M10 rules (effective 2009-07-11) put effects that set power before those that
    // modify it.
    cr!("613.4b", "613.4c");
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P1, "Singing Tree");
    let giant = t.battlefield(P0, "Hill Giant");
    attacks(&mut t, &[(giant, Entity::Player(P1))]);
    cast_card(&mut t, P0, "Giant Growth", &[obj(giant)]).unwrap();
    t.resolve_all();
    t.activate(P1, tree, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 6));
}
