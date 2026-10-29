//! Rulings batch P183 — typal spells and abilities: target legality (CR 115, 608.2b),
//! conditions checked on resolution (CR 608.2h), replacement effects on dying and on
//! token creation (CR 614.1a, 614.1c), cast triggers and copies (CR 707.10), playing a
//! card during resolution (CR 608.2g, 601.2), creature type choices (CR 205.3m), and
//! end-step conditions (CR 603.4).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p189_common::triggers_of;
use crate::r_p194_common::end_step;
use crate::r_s01_common::{asked_since, give_mana_for, supported, tokens, with_subtype};
use crate::r_s02_common::{can_activate, can_cast};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::give_control;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::{cast_new, spell_copies};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of the target choices asked of `p` since `from`.
fn target_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    crate::r_s02_common::target_candidates(t, p, from)
}

#[test]
fn power_word_kill_cant_target_a_creature_with_any_of_the_four_types() {
    cr!("115.1a", "601.2c");
    ruling!(
        "Power Word Kill",
        "A creature is a legal target for Power Word Kill if it doesn't have any of these four creature types. If it has one of these four creature types, it's not a legal target, even if it also has a creature type that's not among these four."
    );
    supported("Power Word Kill");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Demon Wall: a Demon and a Wall.
    let wall = t.battlefield(P1, "Demon Wall");
    let angel = t.battlefield(P1, "Serra Angel");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Power Word Kill", &[obj(bears)]);
    let c = &target_candidates(&t, P0, from)[0];
    assert!(c.contains(&obj(bears)));
    assert!(!c.contains(&obj(wall)) && !c.contains(&obj(angel)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn shoot_the_sheriff_cant_target_a_mercenary() {
    cr!("115.1a", "700.12");
    ruling!(
        "Shoot the Sheriff",
        "The only deputy in the Outlaws of Thunder Junction release is a Mercenary, so no, you cannot shoot the deputy."
    );
    supported("Shoot the Sheriff");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let merc = t.battlefield(P1, "Misshapen Fiend");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Shoot the Sheriff", &[obj(bears)]);
    let c = &target_candidates(&t, P0, from)[0];
    assert!(c.contains(&obj(bears)) && !c.contains(&obj(merc)));
}

#[test]
fn kalitas_exiles_nontoken_creatures_so_dies_triggers_see_only_tokens() {
    cr!("614.1a", "603.6c", "700.4");
    ruling!(
        "Kalitas, Traitor of Ghet",
        "Abilities that trigger whenever a creature an opponent controls dies won't trigger unless that creature is a token (see below)."
    );
    supported("Kalitas, Traitor of Ghet");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(triggers_of(&t, artist), 0);
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    // A token dies: Blood Artist triggers.
    let token = with_subtype(&t, P0, "Zombie")[0];
    give_control(&mut t, token, P1);
    t.answer_targets(P0, &[pl(P1)]);
    kill(&mut t, token);
    t.resolve_all();
    assert_eq!(triggers_of(&t, artist), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn kalitas_cant_sacrifice_itself() {
    cr!("118.3");
    ruling!(
        "Kalitas, Traitor of Ghet",
        "You can't sacrifice Kalitas to activate the last ability, even if it's somehow become a Zombie."
    );
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Zombie");
    t.enter(P0, "Arcane Adaptation");
    let kalitas = t.battlefield(P0, "Kalitas, Traitor of Ghet");
    t.lands(P0, "Swamp", 3);
    assert!(t.obj_now(kalitas).chars.has_subtype("Zombie"));
    assert!(!can_activate(&mut t, P0, kalitas));
    // With another Zombie (Walking Corpse) it can, sacrificing only that one.
    let corpse = t.battlefield(P0, "Walking Corpse");
    assert!(can_activate(&mut t, P0, kalitas));
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(corpse)]);
    t.activate(P0, kalitas, 0, &[]).unwrap();
    for (_, d) in asked_since(&t, from) {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&obj(kalitas)));
        }
    }
    t.resolve_all();
    assert_eq!(t.counters(kalitas, counters::PLUS1), 2);
}

#[test]
fn brightcap_badger_is_a_creature_card_outside_the_stack() {
    cr!("715.4", "202.3");
    ruling!(
        "Brightcap Badger // Fungus Frolic",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it's in your graveyard, Brightcap Badger is a green creature card whose mana value is 4."
    );
    supported("Brightcap Badger // Fungus Frolic");
    let mut t = TestGame::new(2);
    let b = t.graveyard(P0, "Brightcap Badger // Fungus Frolic");
    let o = t.obj_now(b);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Instant));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert_eq!(o.chars.mana_value(), 4);
    // Cast normally, it's a creature spell.
    let mut t = TestGame::new(2);
    let b = t.hand(P0, "Brightcap Badger // Fungus Frolic");
    t.lands(P0, "Forest", 4);
    let s = t.cast(P0, b).go();
    assert!(t.g.obj(s).is(CardType::Creature));
}

#[test]
fn johann_can_cast_only_the_adventure_of_brightcap_badger() {
    cr!("715.3a", "601.3e");
    ruling!(
        "Brightcap Badger // Fungus Frolic",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if you control Johann, Apprentice Sorcerer (\"Once each turn, you may cast an instant or sorcery spell from the top of your library\") and Brightcap Badger is on top of your library, you can cast Fungus Frolic, but not Brightcap Badger."
    );
    supported("Johann, Apprentice Sorcerer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Johann, Apprentice Sorcerer");
    t.lands(P0, "Forest", 4);
    let b = t.library_top(P0, "Brightcap Badger // Fungus Frolic");
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let methods: Vec<CastMethod> = t
        .g
        .legal_actions(P0)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card, method } if card == b => Some(method),
            _ => None,
        })
        .collect();
    assert!(!methods.is_empty());
    assert!(methods.iter().all(|m| matches!(m, CastMethod::Half(_))));
    t.cast(P0, b).method(methods[0].clone()).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Saproling").len(), 2);
}

#[test]
fn splash_portal_and_a_token_that_was_exiled() {
    cr!("111.7", "704.5d");
    ruling!(
        "Splash Portal",
        "If a token is exiled this way, it will cease to exist and won’t return to the battlefield. You won’t draw a card even if it was a Bird, Frog, Otter, or Rat."
    );
    supported("Splash Portal");
    supported("Battle Screech");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Battle Screech", &[]);
    t.resolve_all();
    let birds = with_subtype(&t, P0, "Bird");
    assert_eq!(birds.len(), 2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Splash Portal", &[obj(birds[0])]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 1);
    assert!(!t.g.permanents().any(|o| o.id == t.g.current(birds[0])));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn splash_portal_checks_the_creature_types_of_the_returned_permanent() {
    cr!("400.7", "707.9");
    ruling!(
        "Splash Portal",
        "In the case where the exiled permanent returns with different creature types than it had when it left the battlefield (for example, if the target creature was copying something else when Splash Portal began to resolve), Splash Portal will check its creature types as it exists after returning to the battlefield. If it’s a Bird, Frog, Otter, or Rat at that point, you’ll draw a card."
    );
    supported("Clone");
    // (what Clone copies at first, what it copies when it returns, whether P0 draws)
    for (before, after, draws) in [
        ("Grizzly Bears", "Storm Crow", true),
        ("Storm Crow", "Grizzly Bears", false),
    ] {
        let mut t = TestGame::new(2);
        let first = t.battlefield(P1, before);
        let second = t.battlefield(P1, after);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(first)]);
        let clone = t.enter(P0, "Clone");
        t.resolve_all();
        assert_eq!(t.obj_now(clone).chars.name, before);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(second)]);
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Splash Portal", &[obj(clone)]);
        t.resolve_all();
        assert_eq!(t.obj_now(clone).chars.name, after);
        assert_eq!(t.hand_size(P0), hand + draws as usize, "{before} -> {after}");
    }
}

#[test]
fn enigma_thief_returns_the_targets_still_legal() {
    cr!("608.2b", "115.1a");
    ruling!(
        "Enigma Thief",
        "If a nonland permanent changes control while it's the target of Enigma Thief's ability, that permanent won't be a legal target as the ability resolves. Any remaining legal targets will be returned to their owners' hand."
    );
    supported("Enigma Thief");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Enigma Thief");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1's Bears come under P2's control in response.
    give_control(&mut t, bears, P2);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P2, "Hill Giant"));
}

#[test]
fn unagis_spray_checks_its_target_and_your_creatures_on_resolution() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Unagi's Spray",
        "If the targeted creature becomes an illegal target before Unagi's Spray resolves (perhaps because it left the battlefield), the spell will not resolve."
    );
    ruling!(
        "Unagi's Spray",
        "Unagi's Spray checks if you control a Fish, Octopus, Otter, Seal, Serpent, or Whale when it resolves, not when it is cast."
    );
    supported("Unagi's Spray");
    // (Otter when cast, Otter when it resolves, the target leaves, cards drawn)
    let cases = [
        (true, true, true, 0),
        (false, true, false, 1),
        (true, false, false, 0),
        (true, true, false, 1),
    ];
    for (at_cast, at_resolution, target_leaves, draws) in cases {
        let mut t = TestGame::new(2);
        let otter = at_cast.then(|| t.battlefield(P0, "Valley Floodcaller"));
        let bears = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Unagi's Spray", &[obj(bears)]);
        match (otter, at_resolution) {
            (Some(o), false) => kill(&mut t, o),
            (None, true) => {
                t.battlefield(P0, "Valley Floodcaller");
            }
            _ => {}
        }
        if target_leaves {
            kill(&mut t, bears);
        }
        t.resolve_all();
        let case = (at_cast, at_resolution, target_leaves);
        assert_eq!(t.hand_size(P0), hand + draws, "{case:?}");
        if !target_leaves {
            assert_eq!(t.pt(bears), (-2, 2), "{case:?}");
        }
    }
}

#[test]
fn fishers_talent_replaces_the_fish_but_keeps_the_rest_of_the_instruction() {
    cr!("614.1a", "111.1");
    ruling!(
        "Fisher's Talent",
        "The token's characteristics are entirely replaced by either 3/3 blue Shark creature token or 8/8 blue Octopus creature token, as appropriate. They don't have any other abilities the tokens would have been created with. Anything else specified in the effect creating the tokens (such as tapped, attacking, \"That token gains haste,\" or \"Exile that token at end of combat\") still applies."
    );
    supported("Fisher's Talent");
    supported("Into the Flood Maw");
    for level in [2, 3] {
        let mut t = TestGame::new(2);
        let talent = t.battlefield(P1, "Fisher's Talent");
        let mut ctx = mtg_engine::eval::Ctx::new(Some(talent), P1);
        for l in 2..=level {
            t.g.exec(&Effect::SetClassLevel { level: l }, &mut ctx);
        }
        t.g.recompute();
        let bears = t.battlefield(P1, "Grizzly Bears");
        // P0 casts Into the Flood Maw promising the gift (a tapped Fish) to P1.
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        cast_new(&mut t, P0, "Into the Flood Maw", &[obj(bears)]);
        t.resolve_all();
        let toks = tokens(&t, P1);
        assert_eq!(toks.len(), 1, "level {level}");
        let o = t.obj_now(toks[0]);
        let (name, pt) = if level == 2 {
            ("Shark", (3, 3))
        } else {
            ("Octopus", (8, 8))
        };
        assert!(o.chars.has_subtype(name), "level {level}");
        assert!(!o.chars.has_subtype("Fish"));
        assert_eq!(t.pt(toks[0]), pt);
        assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
        assert!(o.chars.abilities.is_empty());
        assert!(o.tapped, "still tapped");
    }
}

#[test]
fn thurids_copy_resolves_first_even_if_the_spell_is_countered() {
    cr!("707.10", "603.3");
    ruling!(
        "Thurid, Mare of Destiny",
        "Thurid's triggered ability and the copy it creates will resolve before the spell that caused it to trigger. They resolve even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Thurid, Mare of Destiny");
    supported("Mesa Pegasus");
    for counter_before_trigger in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Thurid, Mare of Destiny");
        let spell = cast_new(&mut t, P0, "Mesa Pegasus", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if !counter_before_trigger {
            t.resolve();
            assert_eq!(spell_copies(&t).len(), 1);
        }
        cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
        t.resolve();
        assert!(t.in_graveyard(P0, "Mesa Pegasus"));
        t.resolve_all();
        let pegasi: Vec<ObjectId> = t.named_on_battlefield("Mesa Pegasus");
        assert_eq!(pegasi.len(), 1, "countered before the trigger: {counter_before_trigger}");
        assert!(t.obj_now(pegasi[0]).is_token());
    }
}

#[test]
fn thurid_copies_only_cast_creature_spells() {
    cr!("603.2", "205.3m", "308.1");
    ruling!(
        "Thurid, Mare of Destiny",
        "Thurid's triggered ability doesn't trigger if a Pegasus, Unicorn, or Horse creature is put onto the battlefield without being cast."
    );
    ruling!(
        "Thurid, Mare of Destiny",
        "Thurid's triggered ability will copy only creature spells, not noncreature kindred spells, even if those spells have the Pegasus, Unicorn, or Horse types."
    );
    supported("Crib Swap");
    let mut t = TestGame::new(2);
    let thurid = t.battlefield(P0, "Thurid, Mare of Destiny");
    t.enter(P0, "Mesa Pegasus");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_of(&t, thurid), 0);
    // Crib Swap: a kindred instant with changeling (a Pegasus, a Unicorn, and a Horse).
    let bears = t.battlefield(P1, "Grizzly Bears");
    let swap = cast_new(&mut t, P0, "Crib Swap", &[obj(bears)]);
    assert!(t.g.obj(swap).chars.has_subtype("Horse"));
    t.settle();
    assert_eq!(triggers_of(&t, thurid), 0);
    assert!(spell_copies(&t).is_empty());
}

#[test]
fn reflections_of_littjara_copies_a_kindred_spell_of_the_chosen_type() {
    cr!("205.3m", "707.10");
    ruling!(
        "Reflections of Littjara",
        "If a spell has the kindred card type and the chosen creature type, Reflections of Littjara will copy it."
    );
    supported("Reflections of Littjara");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Reflections of Littjara");
    let a = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Crib Swap", &[obj(a)]);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    // The copy exiled the Bears (one Shapeshifter token); the original then had no
    // legal target.
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(with_subtype(&t, P1, "Shapeshifter").len(), 1);
}

#[test]
fn valley_floodcaller_doesnt_affect_sorcery_speed_abilities() {
    cr!("702.8a", "602.5d");
    ruling!(
        "Valley Floodcaller",
        "Valley Floodcaller’s second ability has no effect on abilities that you can activate “any time you could cast a sorcery.”"
    );
    supported("Valley Floodcaller");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valley Floodcaller");
    t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.set_step(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, div, CastMethod::Normal));
    assert!(!can_activate(&mut t, P0, sword));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_activate(&mut t, P0, sword));
}

#[test]
fn celestial_gatekeeper_can_target_itself_but_is_exiled_first() {
    cr!("608.2b", "400.7");
    ruling!(
        "Celestial Gatekeeper",
        "You can choose to target this card as one of the two Bird and/or Cleric cards. If you do, it gets exiled and does not get returned to the battlefield."
    );
    supported("Celestial Gatekeeper");
    let mut t = TestGame::new(2);
    let gk = t.battlefield(P0, "Celestial Gatekeeper");
    let cleric = t.graveyard(P0, "Bishop of Wings");
    // Crib Swap (a kindred instant with changeling) is a Bird card but not a permanent
    // card.
    let swap = t.graveyard(P0, "Crib Swap");
    t.g.destroy(gk, None);
    // Its trigger isn't on the stack yet; it targets the card in the graveyard and the
    // Bishop.
    let gk_card = t.g.current(gk);
    assert_eq!(t.g.obj(gk_card).zone, Zone::Graveyard(P0));
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(gk_card), obj(cleric)]);
    t.g.flush_events();
    t.settle();
    let c = &target_candidates(&t, P0, from);
    assert!(c.iter().any(|v| v.contains(&obj(gk_card))));
    assert!(c.iter().all(|v| !v.contains(&obj(swap))));
    t.resolve_all();
    assert_eq!(t.zone(gk), Zone::Exile);
    assert!(t.on_battlefield(cleric));
}

#[test]
fn leaf_crowned_elder_plays_the_card_from_the_library_during_resolution() {
    cr!("608.2g", "601.2b", "601.2c", "118.9a", "107.3b");
    ruling!(
        "Leaf-Crowned Elder",
        "You play the revealed card as part of the resolution of this ability. It’s played from your library, not your hand. You choose modes, pay additional costs, choose targets, etc. for the card as normal when playing it. Any X in the mana cost will be 0. Alternative costs can’t be paid."
    );
    supported("Leaf-Crowned Elder");
    supported("Rootgrapple");
    supported("Kurbis, Harvest Celebrant");
    supported("Inner-Flame Acolyte");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leaf-Crowned Elder");
    let stone = t.battlefield(P1, "Mind Stone");
    t.library_top(P0, "Rootgrapple");
    // Look? reveal? play it?
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[obj(stone)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(stone), Zone::Graveyard(P1));
    assert!(t.in_graveyard(P0, "Rootgrapple"));
    // Kurbis ({X}{G}{G}): X is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leaf-Crowned Elder");
    let k = t.library_top(P0, "Kurbis, Harvest Celebrant");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    next_upkeep(&mut t, P0);
    t.resolve_all();
    // Played without a choice of X (X is 0, and no mana was spent): it entered with no
    // counters and died as a 0/0.
    assert!(t.in_graveyard(P0, "Kurbis, Harvest Celebrant"));
    assert!(t.library_size(P0) > 0 && t.zone(k) == Zone::Graveyard(P0));
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseX { .. })));
    // Inner-Flame Acolyte: no evoke (it isn't sacrificed).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leaf-Crowned Elder");
    let a = t.library_top(P0, "Inner-Flame Acolyte");
    let from = t.asked().len();
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(a)));
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseCastingMethod { .. })));
}

#[test]
fn honor_the_god_pharaoh_requires_discarding_exactly_one_card() {
    cr!("601.2b", "601.2h");
    ruling!(
        "Honor the God-Pharaoh",
        "You must discard exactly one card to cast Honor the God-Pharaoh; you can't cast it without discarding a card, and you can't discard additional cards."
    );
    supported("Honor the God-Pharaoh");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Honor the God-Pharaoh");
    let h = t.hand(P0, "Honor the God-Pharaoh");
    assert!(!can_cast(&mut t, P0, h, CastMethod::Normal));
    let a = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    assert!(can_cast(&mut t, P0, h, CastMethod::Normal));
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(a)]);
    t.cast(P0, h).go();
    let discard: Vec<(u32, u32)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { min, max, .. } => Some((min, max)),
            _ => None,
        })
        .collect();
    assert_eq!(discard, vec![(1, 1)]);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn defy_death_checks_for_an_angel_with_no_chance_to_act() {
    cr!("608.2c", "117.1");
    ruling!(
        "Defy Death",
        "No player can cast spells or activate abilities between returning the creature card to the battlefield and checking whether it’s an Angel."
    );
    supported("Defy Death");
    let mut t = TestGame::new(2);
    let angel = t.graveyard(P0, "Serra Angel");
    cast_new(&mut t, P0, "Defy Death", &[obj(angel)]);
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    assert!(asked_since(&t, from).is_empty(), "no decision during the resolution");
    assert_eq!(t.counters(angel, counters::PLUS1), 2);
}

#[test]
fn advocate_of_the_beast_without_a_beast_does_nothing() {
    cr!("603.3d");
    ruling!(
        "Advocate of the Beast",
        "If you don't control any Beast creatures as your end step begins, the ability will be removed from the stack and will have no effect."
    );
    supported("Advocate of the Beast");
    let mut t = TestGame::new(2);
    let adv = t.battlefield(P0, "Advocate of the Beast");
    t.battlefield(P1, "Coeurl");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(triggers_of(&t, adv), 0);
    // With a Beast: a counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Advocate of the Beast");
    let c = t.battlefield(P0, "Coeurl");
    end_step(&mut t);
    assert_eq!(t.counters(c, counters::PLUS1), 1);
}

#[test]
fn death_priest_checks_deaths_this_turn_once_as_the_end_step_begins() {
    cr!("603.4", "513.1");
    ruling!(
        "Death-Priest of Myrkul",
        "Death-Priest of Myrkul doesn't need to have been on the battlefield when the creature died."
    );
    ruling!(
        "Death-Priest of Myrkul",
        "Death-Priest of Myrkul's last ability will trigger only once during your end step, no matter how many creatures died this turn. However, if no creatures have died so far this turn as the end step begins, the ability won't trigger at all. It's not possible to cause a creature to die during the end step in time to have the ability trigger."
    );
    supported("Death-Priest of Myrkul");
    // Two creatures died before Death-Priest arrived: one trigger, one Skeleton.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    kill(&mut t, a);
    kill(&mut t, b);
    let dp = t.battlefield(P0, "Death-Priest of Myrkul");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    end_step(&mut t);
    assert_eq!(triggers_of(&t, dp), 1);
    assert_eq!(with_subtype(&t, P0, "Skeleton").len(), 1);
    // Nothing died: no trigger, even if a creature dies during the end step.
    let mut t = TestGame::new(2);
    let dp = t.battlefield(P0, "Death-Priest of Myrkul");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(triggers_of(&t, dp), 0);
}

#[test]
fn patriarchs_bidding_types_can_repeat_and_need_no_cards() {
    cr!("608.2d", "205.3m", "101.4");
    ruling!(
        "Patriarch's Bidding",
        "Any player may choose a creature type that's already been chosen. Doing so won't affect which creature cards are returned to the battlefield."
    );
    ruling!(
        "Patriarch's Bidding",
        "Each player may choose any creature type, even if they don't have any creature cards of that type in their graveyard."
    );
    supported("Patriarch's Bidding");
    let run = |p0: &str, p1: &str| {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Llanowar Elves");
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P1, "Elvish Mystic");
        t.graveyard(P1, "Hill Giant");
        choose_creature_type(&mut t, P0, p0);
        choose_creature_type(&mut t, P1, p1);
        let from = t.asked().len();
        cast_new(&mut t, P0, "Patriarch's Bidding", &[]);
        t.resolve_all();
        let asked: Vec<(PlayerId, usize)> = asked_since(&t, from)
            .into_iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseOption { prompt, options, .. }
                    if prompt == "Choose a creature type" =>
                {
                    Some((p, options.len()))
                }
                _ => None,
            })
            .collect();
        let mut back: Vec<String> = t
            .g
            .permanents()
            .filter(|o| o.is(CardType::Creature))
            .map(|o| o.chars.name.to_string())
            .collect();
        back.sort();
        (asked, back)
    };
    // Both choose Elf: both Elves return, as if only one had.
    let (asked, back) = run("Elf", "Elf");
    assert_eq!(asked.iter().map(|a| a.0).collect::<Vec<_>>(), vec![P0, P1]);
    assert_eq!(back, vec!["Elvish Mystic", "Llanowar Elves"]);
    // P1 chooses Sliver (no Slivers anywhere): offered every creature type.
    let (asked, back) = run("Bear", "Sliver");
    assert_eq!(asked[1].1, subtype_lists().creature.len());
    assert_eq!(back, vec!["Grizzly Bears"]);
}

#[test]
fn door_of_destinies_any_creature_type_of_a_multitype_creature_counts() {
    cr!("205.3m", "205.3a");
    ruling!(
        "Door of Destinies",
        "Creature types, such as Human or Sliver, appear after the dash on the type line of creatures. Notably, artifact is not a creature type. A creature may have more than one creature type, such as Goblin Warrior. Such a creature would benefit from Door of Destinies if the chosen creature type was Goblin or Warrior."
    );
    supported("Door of Destinies");
    assert!(!subtype_lists().creature.iter().any(|c| c == "Artifact"));
    for ty in ["Goblin", "Warrior"] {
        let mut t = TestGame::new(2);
        choose_creature_type(&mut t, P0, ty);
        let door = t.enter(P0, "Door of Destinies");
        t.g.add_counters(obj(door), counters::CHARGE, 2, None);
        t.g.recompute();
        let piker = t.battlefield(P0, "Goblin Piker");
        assert_eq!(t.pt(piker), (2 + 2, 1 + 2), "{ty}");
    }
}

#[test]
fn door_of_destinies_counter_arrives_before_the_creature() {
    cr!("603.3", "608.3");
    ruling!(
        "Door of Destinies",
        "If you cast a creature spell of the chosen type, Door of Destinies will get a charge counter before the creature enters. The creature will enter with the additional boost to its power and toughness."
    );
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    let door = t.enter(P0, "Door of Destinies");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    t.resolve();
    assert_eq!(t.counters(door, counters::CHARGE), 1);
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve();
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b));
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn kaheeras_condition_allows_mixed_and_extra_types_and_ignores_noncreatures() {
    cr!("702.139a");
    ruling!(
        "Kaheera, the Orphanguard",
        "Creature cards in your deck may be a mix of the five creature types that Kaheera cares about. They may also have additional types, as long as each creature card is at least one of those five. Noncreature cards may have any subtypes."
    );
    use std::sync::Arc;
    let copies = |n: &str, k: usize| -> Vec<Arc<mtg_engine::card::CardDef>> {
        (0..k).map(|_| card(n)).collect()
    };
    let deck = |extra: Option<&str>| {
        // Cats, Cat Beasts, Elemental Warriors, Equipment and Auras, and lands.
        let mut d = copies("Savannah Lions", 9);
        d.extend(copies("Coeurl", 9));
        d.extend(copies("Brighthearth Banneret", 9));
        d.extend(copies("Bonesplitter", 5));
        d.extend(copies("Pacifism", 4));
        d.extend(copies("Plains", 24));
        if let Some(e) = extra {
            d.pop();
            d.push(card(e));
        }
        d
    };
    for (extra, ok) in [(None, true), (Some("Grizzly Bears"), false)] {
        let mut t = crate::r_s12_common::pregame(
            mtg_engine::game::GameConfig {
                starting_player: Some(P0),
                ..Default::default()
            },
            vec![deck(extra), copies("Plains", 60)],
        );
        let side = t.g.add_to_sideboard(P0, vec![card("Kaheera, the Orphanguard")]);
        t.answer_choose(P0, &[obj(side[0])]);
        t.g.start();
        assert_eq!(t.g.companion_of(P0) == Some(side[0]), ok, "{extra:?}");
    }
}
