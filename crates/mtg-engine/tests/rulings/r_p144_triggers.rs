//! Rulings batch P144 — repeatable impulse draw: when the abilities trigger, how often,
//! and what they look at (CR 603.2, 603.4, 603.10, 120.3, 510.2).

use crate::r_p107_common::{act, mana};
use crate::r_p119_common::counter_spell;
use crate::r_p144_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s02_common::{can_cast, create_token, destroy};
use crate::r_s06_common::attach_new;
use crate::r_s09_common::declare;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn exiled(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Exile
}

/// P0 attacks P1 with `attackers` (no blocks); the combat runs to its end.
fn attack_through(t: &mut TestGame, attackers: &[ObjectId]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, P0, &decl);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

/// P0 attacks with `decl`; the attack triggers resolve.
fn attack_only(t: &mut TestGame, decl: &[(ObjectId, Entity)]) {
    declare(t, P0, decl);
    t.resolve_all();
}

#[test]
fn dream_pillager_can_exile_the_whole_library_without_losing() {
    cr!("704.5b", "121.4");
    supported("Dream Pillager");
    ruling!(
        "Dream Pillager",
        "Exiling all the cards from your library this way won't cause you to lose the game. You'll lose only if you attempt to draw a card from an empty library."
    );
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].library.clear();
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island"]);
    let d = t.battlefield(P0, "Dream Pillager");
    attack_through(&mut t, &[d]);
    assert_eq!(t.life(P1), 16);
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    assert_eq!(t.library_size(P0), 0);
    t.settle();
    assert!(!t.has_lost(P0));

    ruling!(
        "Dream Pillager",
        "If you cast an instant or sorcery card this way, it will go to your graveyard after it resolves, fails to resolve, or is countered. It won't return to exile."
    );
    to_main(&mut t);
    mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.g.current(lib[0]);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 13);
    // A countered one goes to the graveyard too.
    let mut t2 = TestGame::new(2);
    let lib2 = on_top(&mut t2, P0, &["Lightning Bolt", "Island"]);
    let d2 = t2.battlefield(P0, "Dream Pillager");
    attack_through(&mut t2, &[d2]);
    to_main(&mut t2);
    mana(&mut t2, P0, ManaType::R, 1);
    let bolt2 = t2.g.current(lib2[0]);
    let spell = t2.cast(P0, bolt2).target(P1).go();
    counter_spell(&mut t2, spell);
    assert_eq!(t2.zone(lib2[0]), Zone::Graveyard(P0));

    // Drawing from the empty library is what loses.
    let _ = t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn fireglass_mentor_cares_whether_life_was_lost_not_the_net_change() {
    cr!("119.3", "603.4");
    supported("Fireglass Mentor");
    ruling!(
        "Fireglass Mentor",
        "Fireglass Mentor’s ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    for lose in [true, false] {
        let mut t = TestGame::new(2);
        let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island", "Wastes"]);
        t.battlefield(P0, "Fireglass Mentor");
        t.g.gain_life(P1, 2);
        if lose {
            t.g.lose_life(P1, 1);
        }
        t.g.flush_events();
        t.answer_choose_moved(P0, &[obj(lib[0])]);
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        assert_eq!(t.life(P1), if lose { 21 } else { 22 });
        assert_eq!(
            exiled(&t, lib[0]) && exiled(&t, lib[1]),
            lose,
            "lost: {lose}"
        );
    }
}

#[test]
fn furious_rise_checks_its_condition_twice_and_exiles_one_card() {
    cr!("603.4");
    supported("Furious Rise");
    ruling!(
        "Furious Rise",
        "Furious Rise's ability has you exile only one card, no matter how many creatures with power 4 or greater you control beyond the first."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island", "Wastes"]);
    t.battlefield(P0, "Furious Rise");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Air Elemental");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert!(!exiled(&t, lib[1]));

    ruling!(
        "Furious Rise",
        "If you don't control a creature with power 4 or greater as your end step begins, Furious Rise's ability doesn't trigger. If you don't control one as the ability resolves, it has no effect. It doesn't have to be the same creature at both times, however."
    );
    // None as the end step begins: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furious Rise");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // One then, gone as it resolves: no effect.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island"]);
    t.battlefield(P0, "Furious Rise");
    let w = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, w);
    t.resolve_all();
    assert!(!exiled(&t, lib[0]));
    // A different one as it resolves: it exiles.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island"]);
    t.battlefield(P0, "Furious Rise");
    let w = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::End);
    t.settle();
    destroy(&mut t, w);
    t.battlefield(P0, "Air Elemental");
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
}

#[test]
fn prophetic_flamespeaker_triggers_on_each_combat_damage() {
    cr!("510.4", "702.4b", "603.2c");
    supported("Prophetic Flamespeaker");
    ruling!(
        "Prophetic Flamespeaker",
        "If Prophetic Flamespeaker deals combat damage to a player multiple times in a turn (if it wasn't blocked, for example), its triggered ability will trigger each time."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island", "Wastes"]);
    let f = t.battlefield(P0, "Prophetic Flamespeaker");
    attack_through(&mut t, &[f]);
    assert_eq!(t.life(P1), 18);
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    assert!(!exiled(&t, lib[2]));
}

#[test]
fn syr_carah_triggers_for_each_player_even_as_she_dies() {
    cr!("603.2c", "704.5g", "120.3");
    supported("Syr Carah, the Bold");
    ruling!(
        "Syr Carah, the Bold",
        "If Syr Carah or an instant or sorcery spell you control deals damage to two or more players at the same time, Syr Carah's first ability triggers for each of those players."
    );
    ruling!(
        "Syr Carah, the Bold",
        "Syr Carah's first ability triggers even if Syr Carah is dealt lethal damage at the same time that it or an instant or sorcery spell you control deals damage to a player."
    );
    // Earthquake (X=3) deals 3 damage to each player (three of them) and to Syr Carah
    // (3/3): she dies, and her ability triggers three times.
    let mut t = TestGame::new(3);
    let lib = on_top(&mut t, P0, &["Island", "Swamp", "Forest", "Wastes"]);
    let s = t.battlefield(P0, "Syr Carah, the Bold");
    let eq = t.hand(P0, "Earthquake");
    mana(&mut t, P0, ManaType::R, 4);
    t.cast(P0, eq).x(3).go();
    t.resolve_all();
    assert!(!t.on_battlefield(s));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (17, 17, 17));
    assert!(lib[..3].iter().all(|c| exiled(&t, *c)));
    assert!(!exiled(&t, lib[3]));
}

#[test]
fn outpost_siege_dragons_sees_a_manifested_noncreature_leave() {
    cr!("701.40a", "708.2", "603.6c");
    supported("Outpost Siege");
    ruling!(
        "Outpost Siege",
        "If a noncreature card is manifested and then leaves the battlefield while face down, the \"Dragons\" ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.enter(P0, "Outpost Siege");
    t.settle();
    let lib = on_top(&mut t, P0, &["Lava Spike", "Wastes"]);
    let summons = t.hand(P0, "Soul Summons");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, summons).go();
    t.resolve_all();
    let fd = t.g.current(lib[0]);
    assert!(t.on_battlefield(fd) && t.obj_now(fd).face_down);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, fd);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn laelia_counts_each_exile_and_any_player() {
    cr!("702.85a", "603.2c", "702.66a");
    supported("Laelia, the Blade Reforged");
    ruling!(
        "Laelia, the Blade Reforged",
        "If a player is instructed to exile cards from their library \"until\" a certain event occurs (for example, because of the triggered ability of a spell with cascade or discover), that player exiles those cards one at a time. Laelia's last ability will trigger that many times."
    );
    // Cascade from Bloodbraid Elf exiles Island, Island, Memnite: three triggers.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Laelia, the Blade Reforged");
    on_top(&mut t, P0, &["Island", "Island", "Memnite", "Wastes"]);
    let elf = t.hand(P0, "Bloodbraid Elf");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::G, 2);
    t.answer_yes(P0, false);
    t.cast(P0, elf).go();
    t.resolve_all();
    assert_eq!(t.counters(l, counters::PLUS1), 3);
    // Discover (Geological Appraiser, discover 3): Island, Island, Memnite.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Laelia, the Blade Reforged");
    on_top(&mut t, P0, &["Island", "Island", "Memnite", "Wastes"]);
    let ga = t.hand(P0, "Geological Appraiser");
    mana(&mut t, P0, ManaType::R, 4);
    t.answer_yes(P0, false);
    t.cast(P0, ga).go();
    t.resolve_all();
    assert_eq!(t.counters(l, counters::PLUS1), 3);
    assert!(t.in_hand(P0, "Memnite"));
    // A spell's "exile cards from the top of your library until you exile a nonland
    // card" (Solstice Revelations): Island, Island, Lava Spike.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Laelia, the Blade Reforged");
    let lib = on_top(&mut t, P0, &["Island", "Island", "Lava Spike", "Wastes"]);
    let sr = t.hand(P0, "Solstice Revelations");
    mana(&mut t, P0, ManaType::R, 3);
    t.cast(P0, sr).go();
    t.resolve_all();
    // (Not cast, the nonland card goes to P0's hand.)
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    assert_eq!(t.zone(lib[2]), Zone::Hand(P0));
    assert!(!exiled(&t, lib[3]));
    assert_eq!(t.counters(l, counters::PLUS1), 3);

    ruling!(
        "Laelia, the Blade Reforged",
        "Laelia's last triggered ability doesn't care which player is exiling cards from the library or graveyard. Cards put into exile from your library or graveyard for any reason, such as the delve ability, cause the ability to trigger."
    );
    // P1's Scavenging Ooze exiles a card from P0's graveyard.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Laelia, the Blade Reforged");
    let g = t.graveyard(P0, "Grizzly Bears");
    let ooze = t.battlefield(P1, "Scavenging Ooze");
    mana(&mut t, P1, ManaType::G, 1);
    act(&mut t, P1, ooze, "Exile target card", &[obj(g)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, g));
    assert_eq!(t.counters(l, counters::PLUS1), 1);
    // Delve: Treasure Cruise exiling three cards from P0's graveyard (one trigger: "one or
    // more cards").
    let gy: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    let cruise = t.hand(P0, "Treasure Cruise");
    mana(&mut t, P0, ManaType::U, 5);
    t.answer_choose(P0, &[obj(gy[0]), obj(gy[1]), obj(gy[2])]);
    t.cast(P0, cruise).go();
    t.resolve_all();
    assert!(gy.iter().all(|c| exiled(&t, *c)));
    assert_eq!(t.counters(l, counters::PLUS1), 2);
}

#[test]
fn tectonic_giant_triggers_once_per_spell_and_chooses_on_resolution() {
    cr!("603.2", "115.3", "608.2c");
    supported("Tectonic Giant");
    ruling!(
        "Tectonic Giant",
        "If a spell targets Tectonic Giant more than once, its ability triggers only once."
    );
    // P1's Seeds of Strength targets the Giant three times.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lightning Bolt", "Island", "Wastes"]);
    let g = t.battlefield(P0, "Tectonic Giant");
    t.set_step(P1, Step::PrecombatMain);
    let seeds = t.hand(P1, "Seeds of Strength");
    mana(&mut t, P1, ManaType::G, 1);
    mana(&mut t, P1, ManaType::W, 1);
    // (The trigger's mode is chosen as it's put on the stack: the second one.)
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P1, seeds).targets(&[obj(g), obj(g), obj(g)]).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);

    ruling!(
        "Tectonic Giant",
        "You choose which of the exiled cards you'll be able to play while the ability is resolving. You can't wait to pick one later. The other card remains in exile."
    );
    ruling!(
        "Tectonic Giant",
        "If you don't play the exiled card that you chose, it remains in exile with the card you didn't choose."
    );
    // The choice of card is asked during the resolution.
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[obj(lib[1])]);
    t.resolve();
    let asked: Vec<Decision> = t.asked()[from..].iter().map(|(_, d)| d.clone()).collect();
    assert!(asked
        .iter()
        .any(|d| matches!(d, Decision::ChooseEntities { .. })));
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    t.resolve_all();
    assert_eq!(t.pt(g), (6, 7));
    // P0's turn: only the chosen card (Island) is playable.
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, lib[1]));
    // After P0's next turn ends, neither: both remain in exile.
    t.advance_to(P1, Step::Upkeep);
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
}

#[test]
fn ob_nixilis_looks_at_the_total_combat_damage() {
    cr!("510.2", "119.3", "603.2c");
    supported("Ob Nixilis, Captive Kingpin");
    ruling!(
        "Ob Nixilis, Captive Kingpin",
        "Ob Nixilis's last ability looks at all combat damage dealt to an opponent in total to determine whether it triggers. For example, if three 1/1 creatures deal combat damage to an opponent, that player loses 3 life. Ob Nixilis's ability won't trigger."
    );
    for n in [3usize, 1] {
        let mut t = TestGame::new(2);
        let lib = on_top(&mut t, P0, &["Lightning Bolt", "Wastes"]);
        let o = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
        let atk: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P0, "Memnite")).collect();
        attack_through(&mut t, &atk);
        assert_eq!(t.life(P1), 20 - n as i32);
        let triggered = n == 1;
        assert_eq!(exiled(&t, lib[0]), triggered, "{n} attackers");
        assert_eq!(t.counters(o, counters::PLUS1), triggered as u32);
    }
}

#[test]
fn flame_channeler_ignores_damage_dealt_by_fighting_creatures() {
    cr!("701.14b", "120.1");
    supported("Flame Channeler // Embodiment of Flame");
    ruling!(
        "Flame Channeler // Embodiment of Flame",
        "Some spells cause other objects to deal damage, such as spells that have two creatures fight. In those cases, it's those other objects that are dealing damage, not the spell itself, and this will not cause Flame Channeler's abilities to trigger."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Flame Channeler");
    let b = t.battlefield(P0, "Craw Wurm");
    let e = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Prey Upon");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, prey).targets(&[obj(b), obj(e)]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(e));
    assert_eq!(t.obj_now(f).chars.name.as_str(), "Flame Channeler");
}

#[test]
fn spinneret_and_spiderling_is_one_spider_and_need_not_attack() {
    cr!("205.3", "508.1");
    supported("Spinneret and Spiderling");
    ruling!(
        "Spinneret and Spiderling",
        "Spinneret and Spiderling are one Spider under the game rules."
    );
    ruling!(
        "Spinneret and Spiderling",
        "Spinneret and Spiderling doesn't have to be one of the two or more Spiders you attack with in order to cause their first ability to trigger."
    );
    // Attacking alone: one Spider, no trigger. With one other Spider: two. Two other
    // Spiders without it: it triggers too.
    let cases: [(bool, usize, u32); 3] = [(true, 0, 0), (true, 1, 1), (false, 2, 1)];
    for (with_it, others, expected) in cases {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Spinneret and Spiderling");
        let mut atk: Vec<ObjectId> = (0..others)
            .map(|_| t.battlefield(P0, "Giant Spider"))
            .collect();
        if with_it {
            atk.push(s);
        }
        let decl: Vec<(ObjectId, Entity)> = atk.iter().map(|a| (*a, Entity::Player(P1))).collect();
        attack_only(&mut t, &decl);
        assert_eq!(
            t.counters(s, counters::PLUS1),
            expected,
            "with it: {with_it}, others: {others}"
        );
    }
}

#[test]
fn dragonhawk_counts_once_and_only_its_own_cards() {
    cr!("603.7c", "608.2h");
    supported("Dragonhawk, Fate's Tempest");
    ruling!(
        "Dragonhawk, Fate's Tempest",
        "The value of X is calculated only once, as Dragonhawk's last ability resolves."
    );
    ruling!(
        "Dragonhawk, Fate's Tempest",
        "The delayed triggered ability created by Dragonhawk's last ability counts only cards that were exiled by the ability that created it. It doesn't count cards exiled by previous instances of Dragonhawk's last ability or by the abilities of other Dragonhawks you may have controlled previously."
    );
    // Dragonhawk (5/5) and Craw Wurm: it enters and attacks. ETB: X = 2. Before the
    // attack trigger resolves, the Wurm dies: X = 1 for it, and stays 1 after another
    // power-4 creature arrives.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Shock", "Lava Spike", "Wastes", "Wastes"],
    );
    let w = t.battlefield(P0, "Craw Wurm");
    let d = t.enter(P0, "Dragonhawk, Fate's Tempest");
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]) && !exiled(&t, lib[2]));
    let dn = t.g.current(d);
    t.g.objects[dn.0 as usize].summoning_sick = false;
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.settle();
    destroy(&mut t, w);
    t.resolve_all();
    assert!(exiled(&t, lib[2]) && !exiled(&t, lib[3]));
    t.battlefield(P0, "Air Elemental");
    t.g.recompute();
    assert!(!exiled(&t, lib[3]));
    // Two delayed triggers at P0's end step: the first counts its two cards, the second
    // its one. Bolt (one of the first two) is cast at P1 first: 3, then 2*1 + 2*1.
    to_main(&mut t);
    mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.g.current(lib[0]);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 17 - 2 - 2);
}

#[test]
fn sword_of_forge_and_frontier_extra_land_plays_add_up() {
    cr!("305.2", "305.2a", "702.4b");
    supported("Sword of Forge and Frontier");
    ruling!(
        "Sword of Forge and Frontier",
        "The effect that allows you to play an additional land that turn is cumulative with other effects that do so, including itself. If the second ability triggers multiple times in the same turn, perhaps because there were multiple combats, you will be able to play that many additional lands."
    );
    // A double striker (Prophetic Flamespeaker) carrying the Sword deals combat damage
    // twice: two triggers, two additional land plays (three lands this turn).
    let mut t = TestGame::new(2);
    on_top(
        &mut t,
        P0,
        &[
            "Wastes", "Wastes", "Wastes", "Wastes", "Wastes", "Wastes", "Wastes", "Wastes",
        ],
    );
    let f = t.battlefield(P0, "Prophetic Flamespeaker");
    attach_new(&mut t, P0, "Sword of Forge and Frontier", obj(f));
    attack_through(&mut t, &[f]);
    to_main(&mut t);
    for i in 0..3 {
        let land = t.hand(P0, "Island");
        assert!(t.play_land(P0, land).is_ok(), "land {i}");
    }
    let land = t.hand(P0, "Island");
    assert!(t.play_land(P0, land).is_err());
}

#[test]
fn oracles_vault_gets_a_brick_counter_even_if_the_card_isnt_played() {
    cr!("608.2c");
    supported("Oracle's Vault");
    ruling!(
        "Oracle's Vault",
        "The first ability of Oracle's Vault puts a brick counter on it even if you can't or don't play the card."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes"]);
    t.g.players[P0.idx()].lands_played_this_turn = 1;
    let v = t.battlefield(P0, "Oracle's Vault");
    mana(&mut t, P0, ManaType::C, 2);
    act(&mut t, P0, v, "brick counter", &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert_eq!(t.counters(v, "brick"), 1);
}

#[test]
fn neyali_triggers_per_player_attacked_with_tokens() {
    cr!("603.2c", "506.2", "508.1b");
    supported("Neyali, Suns' Vanguard");
    ruling!(
        "Neyali, Suns' Vanguard",
        "The second ability of Neyali, Suns' Vanguard triggers for each player you are attacking with one or more tokens. It does not trigger if you are attacking a planeswalker a player controls, but not that player."
    );
    // Four players: tokens attack P1 (two of them) and P2; a token attacks P3's
    // planeswalker. Two triggers.
    let mut t = TestGame::new(4);
    let lib = on_top(&mut t, P0, &["Island", "Swamp", "Forest", "Wastes"]);
    t.battlefield(P0, "Neyali, Suns' Vanguard");
    let pw = t.battlefield(P3, "Jace Beleren");
    let tok: Vec<ObjectId> = (0..4)
        .map(|_| create_token(&mut t, P0, "Soldier"))
        .collect();
    attack_only(
        &mut t,
        &[
            (tok[0], Entity::Player(P1)),
            (tok[1], Entity::Player(P1)),
            (tok[2], Entity::Player(P2)),
            (tok[3], obj(pw)),
        ],
    );
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    assert!(!exiled(&t, lib[2]));
}

#[test]
fn vance_s_blasting_cannons_third_spell_trigger() {
    cr!("603.2", "603.3", "601.2i");
    supported("Vance's Blasting Cannons // Spitfire Bastion");
    ruling!(
        "Vance's Blasting Cannons // Spitfire Bastion",
        "The second ability of Vance's Blasting Cannons counts all the spells you've cast, including Vance's Blasting Cannons itself if you cast it this turn. The ability won't trigger unless Vance's Blasting Cannons is on the battlefield as you cast your third spell of the turn."
    );
    // Cast Ornithopter, then the Cannons (second spell), then a third spell: it triggers.
    let mut t = TestGame::new(2);
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    let v = t.hand(P0, "Vance's Blasting Cannons");
    mana(&mut t, P0, ManaType::R, 4);
    t.cast(P0, v).go();
    t.resolve_all();
    let v = t.g.current(v);
    assert!(t.on_battlefield(v));
    t.answer_yes(P0, true);
    let o2 = t.hand(P0, "Ornithopter");
    t.cast(P0, o2).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Spitfire Bastion");
    // The Cannons entering after the third spell: no trigger for that spell (or a later
    // fourth one).
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    let v = t.battlefield(P0, "Vance's Blasting Cannons");
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Vance's Blasting Cannons");

    ruling!(
        "Vance's Blasting Cannons // Spitfire Bastion",
        "The second ability of Vance's Blasting Cannons resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vance's Blasting Cannons");
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    let o = t.hand(P0, "Ornithopter");
    let spell = t.cast(P0, o).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter_spell(&mut t, spell);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Spitfire Bastion");

    ruling!(
        "Vance's Blasting Cannons // Spitfire Bastion",
        "The card exiled by the first ability of Vance's Blasting Cannons is exiled face up."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Lava Spike", "Wastes"]);
    t.battlefield(P0, "Vance's Blasting Cannons");
    next_upkeep(&mut t);
    assert!(exiled(&t, lib[0]) && !t.obj_now(lib[0]).face_down);
}

#[test]
fn unstable_amulet_counts_spells_cast_from_anywhere_but_the_hand() {
    cr!("601.2a", "400.11b");
    supported("Unstable Amulet");
    ruling!(
        "Unstable Amulet",
        "Unstable Amulet's second ability counts any spells cast from zones other than your hand. These are usually spells cast from exile, the graveyard, or the command zone. It also counts spells cast from outside the game, such as spells cast with Wish or Garth One-Eye's ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unstable Amulet");
    // From the hand: nothing.
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // From exile (Amulet's own permission), the graveyard (flashback), the command zone
    // (a commander), and outside the game: 1 damage each.
    let lib = on_top(&mut t, P0, &["Ornithopter", "Wastes"]);
    t.g.add_counters(Entity::Player(P0), counters::ENERGY, 2, None);
    let a = t.named_on_battlefield("Unstable Amulet")[0];
    act(&mut t, P0, a, "Exile the top", &[]).unwrap();
    t.resolve_all();
    let c = t.g.current(lib[0]);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    let fb = t.graveyard(P0, "Think Twice");
    mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, fb)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let cmd = t.command(P0, "Ornithopter");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.players[P0.idx()]
        .commander_names
        .push("Ornithopter".into());
    t.g.dirty = true;
    t.g.recompute();
    t.cast(P0, cmd).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Wish (from the hand: nothing), then a card from outside the game.
    let side = t.g.add_to_sideboard(P0, vec![card("Ornithopter")]);
    let wish = t.hand(P0, "Wish");
    mana(&mut t, P0, ManaType::R, 3);
    t.cast(P0, wish).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.cast(P0, side[0]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
