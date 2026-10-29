//! Rulings batch P118 — targets whose legality depends on the source's power ("with mana
//! value less than or equal to this creature's power", "with lesser power"): checked as
//! the target is chosen and again as the ability resolves (CR 115.1, 608.2b); an ability
//! whose targets are all illegal doesn't resolve (CR 608.2b); once it has resolved, the
//! effect doesn't recheck (CR 611.2c).

use crate::r_p118_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- the source's power drops below the target's mana value -------------------------------

#[test]
fn alesha_target_becomes_illegal_if_its_power_drops() {
    cr!("608.2b", "115.1");
    ruling!(
        "Alesha, Who Laughs at Fate",
        "If another effect causes Alesha's power to be less than the mana value of the target card as its last ability tries to resolve, it won't resolve and none of its effects will happen."
    );
    supported("Alesha, Who Laughs at Fate");
    for shrink in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Alesha, Who Laughs at Fate");
        // Hill Giant: mana value 4; Alesha is 3/3 after attacking, 4/4 with a counter.
        give_plus1(&mut t, a, 1);
        let giant = t.graveyard(P0, "Hill Giant");
        attack_with(&mut t, &at(P1, &[a]));
        t.resolve_all();
        assert_eq!(t.pt(a), (4, 4));
        t.answer_targets(P0, &[obj(giant)]);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(stack(&t), 1);
        if shrink {
            minus_power(&mut t, a, 1);
        }
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Hill Giant"), shrink, "shrink {shrink}");
    }
}

#[test]
fn alesha_x_in_a_mana_cost_is_zero_in_the_graveyard() {
    cr!("202.3e", "115.1");
    ruling!(
        "Alesha, Who Laughs at Fate",
        "If the mana cost of a creature card in your graveyard includes {X}, X is 0 for the purpose of determining its mana value."
    );
    // Grenzo, Dungeon Warden ({X}{B}{R}, 2/2) has mana value 2 in the graveyard; Alesha
    // is 2/2 after attacking and shrinking by 1.
    supported("Grenzo, Dungeon Warden");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alesha, Who Laughs at Fate");
    let grenzo = t.graveyard(P0, "Grenzo, Dungeon Warden");
    attack_with(&mut t, &at(P1, &[a]));
    t.resolve_all();
    minus_power(&mut t, a, 1);
    assert_eq!(t.pt(a).0, 2);
    t.answer_targets(P0, &[obj(grenzo)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grenzo, Dungeon Warden").len(), 1);
}

#[test]
fn carmen_target_becomes_illegal_if_its_power_drops() {
    cr!("608.2b", "115.1");
    ruling!(
        "Carmen, Cruel Skymarcher",
        "If another effect causes Carmen, Cruel Skymarcher's power to be less than the mana value of the target card as its last ability tries to resolve, the target is illegal. You won't return the target card to the battlefield."
    );
    supported("Carmen, Cruel Skymarcher");
    for shrink in [false, true] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Carmen, Cruel Skymarcher");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &at(P1, &[c]));
        assert_eq!(stack(&t), 1);
        if shrink {
            minus_power(&mut t, c, 1);
        }
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Grizzly Bears"), shrink, "shrink {shrink}");
    }
}

#[test]
fn arcane_proxy_target_becomes_illegal_if_its_power_drops() {
    cr!("608.2b", "115.1", "707.10");
    ruling!(
        "Arcane Proxy",
        "If another effect causes Arcane Proxy's power to be less than the mana value of the target card as the ability tries to resolve, the target is illegal. The card won't be exiled, and you won't get to cast a copy."
    );
    supported("Arcane Proxy");
    // Divination (mana value 3) and a 4/3 Arcane Proxy.
    for shrink in [false, true] {
        let mut t = TestGame::new(2);
        for _ in 0..4 {
            t.library_top(P0, "Island");
        }
        let div = t.graveyard(P0, "Divination");
        lands_for_cost(&mut t, P0, "Arcane Proxy");
        let proxy = t.hand(P0, "Arcane Proxy");
        t.answer_targets(P0, &[obj(div)]);
        t.answer_yes(P0, true);
        t.cast(P0, proxy).go();
        t.resolve();
        t.settle();
        assert_eq!(stack(&t), 1, "enters trigger");
        if shrink {
            let p = t.g.current(proxy);
            minus_power(&mut t, p, 2);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Divination"), shrink, "shrink {shrink}");
        assert_eq!(t.hand_size(P0), hand + if shrink { 0 } else { 2 });
    }
}

#[test]
fn dreadhorde_arcanist_target_is_chosen_before_other_attack_triggers_resolve() {
    cr!("603.3d", "115.1", "508.1m");
    ruling!(
        "Dreadhorde Arcanist",
        "The instant or sorcery card you target must have mana value less than or equal to Dreadhorde Arcanist's power immediately after Dreadhorde Arcanist has attacked. Any other abilities that trigger when creatures you control attack won't have resolved yet."
    );
    // Fervent Charge: "Whenever a creature you control attacks, it gets +2/+2 until end
    // of turn." The 1-power Arcanist can't target a mana value 2 card.
    supported("Fervent Charge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervent Charge");
    let arc = t.battlefield(P0, "Dreadhorde Arcanist");
    let shock = t.graveyard(P0, "Shock");
    let strike = t.graveyard(P0, "Lightning Strike");
    t.answer_targets(P0, &[obj(strike)]);
    attack_with(&mut t, &at(P1, &[arc]));
    let targeted: Vec<Entity> = t
        .g
        .stack
        .iter()
        .flat_map(|id| crate::r_s25_common::targets_of(&t, *id))
        .collect();
    assert!(!targeted.contains(&obj(strike)));
    // Shock was a legal target (positive control).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervent Charge");
    let arc = t.battlefield(P0, "Dreadhorde Arcanist");
    let shock2 = t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[obj(shock2)]);
    attack_with(&mut t, &at(P1, &[arc]));
    let targeted: Vec<Entity> = t
        .g
        .stack
        .iter()
        .flat_map(|id| crate::r_s25_common::targets_of(&t, *id))
        .collect();
    assert!(targeted.contains(&obj(shock2)));
    let _ = shock;
}

#[test]
fn dreadhorde_arcanist_casts_during_resolution_ignoring_timing() {
    cr!("608.2g", "601.2", "307.1");
    ruling!(
        "Dreadhorde Arcanist",
        "If you cast the card, you do so as part of the resolution of Dreadhorde Arcanist's triggered ability. You can't wait to cast it later in the turn. Timing permissions based on a card's type are ignored, and the spell will resolve before blockers are declared."
    );
    // Lava Spike is a sorcery: "Lava Spike deals 3 damage to target player or planeswalker."
    supported("Lava Spike");
    let mut t = TestGame::new(2);
    let arc = t.battlefield(P0, "Dreadhorde Arcanist");
    let spike = t.graveyard(P0, "Lava Spike");
    t.answer_targets(P0, &[obj(spike)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &at(P1, &[arc]));
    t.resolve();
    assert_eq!(stack(&t), 1, "Lava Spike was cast during the resolution");
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(lost(&t, P1), 3);
    assert!(t.in_exile("Lava Spike"));
}

// --- "with lesser power" and "less than this creature's power" ------------------------------

#[test]
fn vodalian_mindsinger_rechecks_power_on_resolution_only() {
    cr!("608.2b", "611.2c");
    ruling!(
        "Vodalian Mindsinger",
        "If the target creature's power is greater than or equal to Vodalian Mindsinger's power when the triggered ability resolves, it will be an illegal target for the ability, and you will not gain control of it. Once the ability resolves, however, changing the power of either creature will not end the control-changing effect."
    );
    supported("Vodalian Mindsinger");
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.answer_targets(P0, &[obj(elves)]);
        let m = t.enter(P0, "Vodalian Mindsinger");
        t.g.flush_events();
        t.settle();
        assert_eq!(stack(&t), 1);
        if pump {
            giant_growth(&mut t, elves);
        }
        t.resolve_all();
        assert_eq!(t.obj_now(elves).controller, if pump { P1 } else { P0 });
        if !pump {
            giant_growth(&mut t, elves);
            minus_power(&mut t, m, 2);
            assert_eq!(t.obj_now(elves).controller, P0, "control kept");
        }
    }
}

#[test]
fn rangers_of_ithilien_checks_power_on_targeting_and_resolution_only() {
    cr!("608.2b", "611.2c", "115.1");
    ruling!(
        "Rangers of Ithilien",
        "Rangers of Ithilien's triggered ability lets you gain control of a creature if it has lesser power than Rangers of Ithilien at the time that you choose the target and at the time that the ability resolves. After that, it no longer matters what the creature's power is; you will continue to control it for as long as you control Rangers of Ithilien."
    );
    ruling!(
        "Rangers of Ithilien",
        "If you gain control of a creature with the triggered ability, you can immediately choose that creature as your Ring-bearer as the Ring tempts you."
    );
    supported("Rangers of Ithilien");
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_choose(P0, &[obj(bears)]);
        t.enter(P0, "Rangers of Ithilien");
        t.g.flush_events();
        t.settle();
        assert_eq!(stack(&t), 1);
        if pump {
            giant_growth(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.obj_now(bears).controller, if pump { P1 } else { P0 });
        if !pump {
            assert_eq!(t.g.player(P0).ring_bearer, Some(bears), "Ring-bearer");
            giant_growth(&mut t, bears);
            assert_eq!(t.obj_now(bears).controller, P0, "control kept");
        }
    }
    // A creature with greater power can't be targeted at all.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Rangers of Ithilien");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).controller, P1);
}

#[test]
fn unliving_psychopath_checks_power_on_activation_and_resolution() {
    cr!("602.2b", "608.2b", "115.1");
    ruling!(
        "Unliving Psychopath",
        "The creature must have less power than Unliving Psychopath both when the ability is activated and when it resolves."
    );
    supported("Unliving Psychopath");
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        let u = with_counters(&mut t, P0, "Unliving Psychopath", 2);
        let elves = t.battlefield(P1, "Llanowar Elves");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 1);
        // The 2-power Bears can't be targeted by a 2-power Psychopath.
        assert!(t.activate(P0, u, 1, &[obj(bears)]).is_err());
        t.g.objects[u.0 as usize].tapped = false;
        t.activate(P0, u, 1, &[obj(elves)]).unwrap();
        if pump {
            giant_growth(&mut t, elves);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(elves), pump, "pump {pump}");
    }
}

#[test]
fn old_man_of_the_sea_doesnt_care_about_targetability_after_resolving() {
    cr!("611.2b", "608.2b", "208.3");
    ruling!(
        "Old Man of the Sea",
        "Once the ability resolves, it doesn’t care whether the targeted permanent remains a legal target for the ability, only that it’s on the battlefield and its power remains small enough. The effect will continue to apply to it even if it gains shroud, stops being a creature, or would no longer be a legal target for any other reason. If it stops being a creature, its power is considered to be 0."
    );
    supported("Old Man of the Sea");
    use mtg_engine::ability::Modification;
    use mtg_engine::keywords::Keyword;
    use mtg_engine::types::CardType;
    let mut t = TestGame::new(2);
    let om = t.battlefield(P0, "Old Man of the Sea");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, om, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // It gains shroud and stops being a creature: still controlled.
    modify(&mut t, bears, vec![Modification::AddKeyword(Keyword::new(mtg_engine::keywords::KeywordKind::Shroud))]);
    modify(&mut t, bears, vec![Modification::RemoveTypes(vec![CardType::Creature])]);
    assert!(!t.obj_now(bears).is(CardType::Creature));
    assert_eq!(t.obj_now(bears).controller, P0);
    // Positive control: a creature whose power grows past Old Man's is returned.
    let mut t = TestGame::new(2);
    let om = t.battlefield(P0, "Old Man of the Sea");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, om, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    giant_growth(&mut t, bears);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn stone_giant_can_target_itself_when_its_toughness_is_less_than_its_power() {
    cr!("115.1", "601.2c");
    ruling!(
        "Stone Giant",
        "If the Giant’s power and/or toughness change so that its toughness is less than its power, you can have the ability target the Giant itself."
    );
    supported("Stone Giant");
    // Stone Giant is 3/4: it can't target itself, until it gets +2/+0.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Stone Giant");
    assert!(t.activate(P0, g, 0, &[obj(g)]).is_err());
    t.g.objects[g.0 as usize].tapped = false;
    plus_power(&mut t, g, 2);
    t.activate(P0, g, 0, &[obj(g)]).unwrap();
    t.resolve_all();
    assert!(has_kw(&mut t, g, mtg_engine::keywords::KeywordKind::Flying));
}

// --- all targets illegal ------------------------------------------------------------------------

#[test]
fn orca_rust_harvester_and_stalking_yeti_do_nothing_with_illegal_targets() {
    cr!("608.2b");
    ruling!(
        "Orca, Siege Demon",
        "If all of the targets are illegal when the ability tries to resolve, it won't resolve and none of its effects will happen. Nothing will be dealt damage."
    );
    ruling!(
        "Rust Harvester",
        "If the target is illegal as Rust Harvester’s last ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t put a +1/+1 counter on Rust Harvester."
    );
    supported("Rust Harvester");
    // Orca: its only target, an Air Elemental, is exiled in response.
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let orca = t.battlefield(P0, "Orca, Siege Demon");
        let air = t.battlefield(P1, "Air Elemental");
        t.answer_targets(P0, &[obj(air)]);
        destroy(&mut t, orca);
        if gone {
            exile(&mut t, air);
        }
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, "Air Elemental"), !gone, "gone {gone}");
        assert_eq!(t.life(P1), 20);
    }
    // Rust Harvester.
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Rust Harvester");
        t.graveyard(P0, "Ornithopter");
        t.lands(P0, "Wastes", 2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.answer_choose(P0, &[]);
        t.activate(P0, h, 0, &[obj(wurm)]).unwrap();
        if gone {
            exile(&mut t, wurm);
        }
        t.resolve_all();
        assert_eq!(plus1(&t, h), if gone { 0 } else { 1 }, "gone {gone}");
        if !gone {
            assert_eq!(damage(&t, wurm), 2);
        }
    }
}

#[test]
fn stalking_yeti_does_nothing_if_either_creature_is_gone() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Stalking Yeti",
        "If either Stalking Yeti or the targeted creature is no longer on the battlefield when the \"enters\" triggered ability resolves, the ability does nothing."
    );
    supported("Stalking Yeti");
    // (Yeti leaves, target leaves)
    for (yeti_gone, target_gone) in [(false, false), (true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.answer_targets(P0, &[obj(wurm)]);
        let y = t.enter(P0, "Stalking Yeti");
        t.g.flush_events();
        t.settle();
        assert_eq!(stack(&t), 1);
        if yeti_gone {
            exile(&mut t, y);
        }
        if target_gone {
            exile(&mut t, wurm);
        }
        t.resolve_all();
        let nothing = yeti_gone || target_gone;
        if !target_gone {
            assert_eq!(damage(&t, wurm), if nothing { 0 } else { 3 });
        }
        if !yeti_gone {
            assert_eq!(t.on_battlefield(y), nothing, "Wurm deals 6 back");
        }
    }
}

// --- Ravenous Gigantotherium and Living Inferno --------------------------------------------------

/// P0's Ravenous Gigantotherium enters with `n` +1/+1 counters; its trigger targets
/// `targets` with the damage divided as `split`. Returns the game with the trigger on the
/// stack and the Gigantotherium.
fn gigantotherium(t: &mut TestGame, n: u32, targets: &[Entity], split: &[i64]) -> ObjectId {
    supported("Ravenous Gigantotherium");
    t.answer_targets(P0, targets);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(split.to_vec()));
    let g = enter_with(t, P0, "Ravenous Gigantotherium", n, false);
    assert_eq!(stack(t), 1);
    g
}

#[test]
fn ravenous_gigantotherium_keeps_its_division_for_remaining_targets() {
    cr!("608.2b", "601.2d", "603.3d");
    ruling!(
        "Ravenous Gigantotherium",
        "If some of the targets become illegal for Ravenous Gigantotherium's ability, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let air = t.battlefield(P1, "Air Elemental");
    let g = gigantotherium(&mut t, 3, &[obj(wurm), obj(air)], &[2, 4]);
    exile(&mut t, air);
    t.resolve_all();
    assert_eq!(damage(&t, wurm), 2);
    // The Wurm deals 6 back to the 6/6.
    assert!(!t.on_battlefield(g));
}

#[test]
fn ravenous_gigantotherium_locks_x_as_it_is_put_on_the_stack() {
    cr!("601.2d", "603.3d", "613.4c");
    ruling!(
        "Ravenous Gigantotherium",
        "The value of X is locked in as you divide the damage while putting Ravenous Gigantotherium's ability onto the stack. Changing Ravenous Gigantotherium's power after that time won't change how much damage is dealt."
    );
    ruling!(
        "Ravenous Gigantotherium",
        "Static abilities, such as that of Glorious Anthem, modify Ravenous Gigantotherium's power before its triggered ability is put onto the stack. Spells, activated abilities, and triggered abilities can't raise its power in time to change the value of X."
    );
    // Glorious Anthem: a 4/4 deals 4 to an indestructible Darksteel Myr; Giant Growth
    // afterwards doesn't change it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let g = gigantotherium(&mut t, 0, &[obj(myr)], &[4]);
    giant_growth(&mut t, g);
    t.resolve_all();
    assert_eq!(damage(&t, myr), 4);
}

#[test]
fn ravenous_gigantotherium_targets_deal_damage_back_before_dying() {
    cr!("608.2", "704.3", "120.3");
    ruling!(
        "Ravenous Gigantotherium",
        "The creatures targeted by Ravenous Gigantotherium's ability deal damage back to Ravenous Gigantotherium before dying if they're dealt lethal damage."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let g = gigantotherium(&mut t, 3, &[obj(giant)], &[6]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(damage(&t, g), 3);
}

#[test]
fn living_inferno_locks_its_damage_but_not_the_damage_dealt_back() {
    cr!("601.2d", "602.2b", "608.2h");
    ruling!(
        "Living Inferno",
        "The amount of damage Living Inferno will deal is locked in as you activate its ability, since you're dividing damage. If Living Inferno's power changes after its ability is activated but before it resolves (via Giant Growth, for example), the amount of damage Living Inferno deals won't change. The same is *not* true for the amount of damage dealt to Living Inferno by the other creatures, which is determined when the ability resolves."
    );
    supported("Living Inferno");
    let mut t = TestGame::new(2);
    let li = t.battlefield(P0, "Living Inferno");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![8]));
    t.activate(P0, li, 0, &[obj(myr)]).unwrap();
    giant_growth(&mut t, li);
    giant_growth(&mut t, myr);
    t.resolve_all();
    assert_eq!(damage(&t, myr), 8);
    assert_eq!(damage(&t, li), 3, "the pumped Myr deals 3 back");
}
