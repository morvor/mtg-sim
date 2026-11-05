//! Rulings batch P178 — leaves-the-battlefield and counter rulings (Hei Bai, Angel of
//! Fury), sacrifice triggers (Korvold), characteristic changes (Sludge Monster), "enters
//! or attacks" triggers (Agent Frank Horrigan, Soul of Windgrace, Glint-Sleeve Siphoner,
//! Naomi, Bloodhall Priest, Sanctuary Warden), threshold (Nut Collector, Deep-Sea
//! Terror, Swirling Sandstorm, Krosan Restorer), The Wise Mothman, Harvestrite Host,
//! Nezahal's return and The Raven Man.

use crate::r_p178_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(obj(id), kind, n, None);
    t.g.recompute();
}

/// P0's Hei Bai (with the given counters) dies; its trigger targets `target`.
fn hei_bai_dies(t: &mut TestGame, counters: &[(&str, u32)], target: ObjectId) {
    supported("Hei Bai, Spirit of Balance");
    let hb = t.battlefield(P0, "Hei Bai, Spirit of Balance");
    for (k, n) in counters {
        add(t, hb, k, *n);
    }
    t.settle();
    t.answer_targets(P0, &[obj(target)]);
    destroy(t, hb);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hei Bai, Spirit of Balance"));
}

#[test]
fn hei_bai_puts_the_same_number_of_each_kind_of_counter() {
    cr!("603.10a", "122.1");
    ruling!(
        "Hei Bai, Spirit of Balance",
        "Hei Bai's last ability doesn't cause you to move counters from Hei Bai onto the target creature. Rather, you put the same number of each kind of counter Hei Bai had when he left the battlefield onto the target creature."
    );
    ruling!(
        "Hei Bai, Spirit of Balance",
        "Hei Bai's last ability puts all counters that were on Hei Bai onto the target creature, not just his +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    hei_bai_dies(&mut t, &[("+1/+1", 2), ("flying", 1)], wurm);
    assert_eq!(t.counters(wurm, "+1/+1"), 2);
    assert_eq!(t.counters(wurm, "flying"), 1);
    assert!(has_kw(&t, wurm, KeywordKind::Flying));
}

#[test]
fn hei_bai_minus_counters_can_kill_the_recipient() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Hei Bai, Spirit of Balance",
        "If Hei Bai has -1/-1 counters on him when he leaves the battlefield, that ability will include those as well. This may result in the recipient of the counters dying."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    hei_bai_dies(&mut t, &[("-1/-1", 2)], bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn hei_bai_killed_by_minus_counters_sees_both_kinds() {
    cr!("704.3", "704.5q", "603.10a");
    ruling!(
        "Hei Bai, Spirit of Balance",
        "If enough -1/-1 counters are put on Hei Bai at the same time to make his toughness 0 or less, his last ability will see all of the +1/+1 counters he had when he left the battlefield as well as the -1/-1 counters he had"
    );
    supported("Hardened Scales");
    let mut t = TestGame::new(2);
    // Hardened Scales: one more +1/+1 counter is put on the Wurm, so how many of each
    // kind it receives shows.
    let wurm = t.battlefield(P0, "Craw Wurm");
    let hb = t.battlefield(P0, "Hei Bai, Spirit of Balance");
    add(&mut t, hb, "+1/+1", 2);
    assert_eq!(t.pt(hb), (5, 5));
    t.battlefield(P0, "Hardened Scales");
    t.answer_targets(P0, &[obj(wurm)]);
    // Five -1/-1 counters at once: 0/0. It dies and the counters would annihilate as the
    // same state-based actions are performed.
    add(&mut t, hb, "-1/-1", 5);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hei Bai, Spirit of Balance"));
    // The Wurm got 2 (+1 from the Scales) +1/+1 and 5 -1/-1 counters: net -2/-2.
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn hei_bai_and_buzzard_wasp_colony_both_put_counters() {
    cr!("603.10a", "603.2");
    ruling!(
        "Hei Bai, Spirit of Balance",
        "In some cases, you may end up putting the appropriate counters on more than one permanent."
    );
    supported("Buzzard-Wasp Colony");
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Buzzard-Wasp Colony");
    let wurm = t.battlefield(P0, "Craw Wurm");
    hei_bai_dies(&mut t, &[("+1/+1", 2)], wurm);
    assert_eq!(t.counters(wurm, "+1/+1"), 2);
    assert_eq!(t.counters(colony, "+1/+1"), 2);
    // Targeting the Colony: both abilities put counters on it.
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Buzzard-Wasp Colony");
    hei_bai_dies(&mut t, &[("+1/+1", 2)], colony);
    assert_eq!(t.counters(colony, "+1/+1"), 4);
}

#[test]
fn angel_of_fury_looks_back_at_its_ability() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Angel of Fury",
        "If Angel of Fury lost its triggered ability while on the battlefield (due to Snakeform, for example) and then was destroyed, the ability would not trigger."
    );
    supported("Angel of Fury");
    // Lost the ability on the battlefield: no trigger.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Angel of Fury");
    cast_resolve(&mut t, P1, "Turn to Frog", &[obj(angel)]);
    t.answer_yes(P0, true);
    destroy(&mut t, angel);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Angel of Fury"));
    // Loses it only in the graveyard (Yixlid Jailer): it still triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let angel = t.battlefield(P0, "Angel of Fury");
    t.answer_yes(P0, true);
    destroy(&mut t, angel);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.zone(angel), Zone::Library(P0), "shuffled in");
}

#[test]
fn angel_of_fury_stays_where_it_went_or_never_triggers() {
    cr!("603.6c", "614.6", "400.7");
    ruling!(
        "Angel of Fury",
        "If Angel of Fury is removed from the graveyard after the ability triggers but before it resolves, it will remain in its zone when its owner shuffles their library. Similarly, if a replacement effect has it move to a different zone instead of being put into the graveyard, the ability won't trigger at all."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Angel of Fury");
    t.answer_yes(P0, true);
    destroy(&mut t, angel);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.current(angel);
    t.g.exile_object(card, None);
    t.resolve_all();
    assert_eq!(t.zone(angel), Zone::Exile, "remains in exile");
    // Rest in Peace: exiled instead, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let angel = t.battlefield(P0, "Angel of Fury");
    destroy(&mut t, angel);
    assert_eq!(t.zone(angel), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn korvold_sacrificing_itself_draws_but_puts_no_counter() {
    cr!("603.10a", "701.21a");
    ruling!(
        "Korvold, Fae-Cursed King",
        "If you sacrifice Korvold, its last ability triggers. You won't put a +1/+1 counter on anything, but you'll draw a card."
    );
    supported("Korvold, Fae-Cursed King");
    supported("Viscera Seer");
    let mut t = TestGame::new(2);
    let korvold = t.battlefield(P0, "Korvold, Fae-Cursed King");
    let seer = t.battlefield(P0, "Viscera Seer");
    t.answer_choose(P0, &[obj(korvold)]);
    t.activate(P0, seer, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Korvold, Fae-Cursed King"));
    let h = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1);
    assert_eq!(t.counters(seer, "+1/+1"), 0);
}

#[test]
fn korvold_trigger_from_a_sacrifice_cost_resolves_first() {
    cr!("603.3", "602.2b");
    ruling!(
        "Korvold, Fae-Cursed King",
        "If you sacrifice a permanent as part of a spell or activated ability's cost, Korvold's last ability will resolve before that spell or ability."
    );
    let mut t = TestGame::new(2);
    let korvold = t.battlefield(P0, "Korvold, Fae-Cursed King");
    let seer = t.battlefield(P0, "Viscera Seer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, seer, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_from(&t, korvold), 1);
    t.resolve();
    assert_eq!(t.counters(korvold, "+1/+1"), 1);
    assert_eq!(t.stack_len(), 1, "the Seer's ability is still waiting");
}

#[test]
fn korvold_attacks_with_nothing_to_sacrifice_and_has_no_sacrifice_ability() {
    cr!("701.21a", "602.1");
    ruling!(
        "Korvold, Fae-Cursed King",
        "If you somehow control no other permanents, you can attack with Korvold. You won't sacrifice anything, and there's no penalty for being unable to do so."
    );
    ruling!(
        "Korvold, Fae-Cursed King",
        "Korvold's last ability is a triggered ability, not an activated ability. It doesn't allow you to sacrifice a permanent whenever you want"
    );
    let mut t = TestGame::new(2);
    let korvold = t.battlefield(P0, "Korvold, Fae-Cursed King");
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, korvold));
    t.attack(&[(korvold, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(korvold));
    assert_eq!(t.counters(korvold, "+1/+1"), 0);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sludge_monster_affects_any_slimed_non_horror_until_it_becomes_a_horror() {
    cr!("613.4b", "613.1f", "122.1");
    ruling!(
        "Sludge Monster",
        "Sludge Monster's last ability affects all non-Horror creatures with slime counters on them, not just creatures that had slime counters put on them with Sludge Monster's first ability."
    );
    ruling!(
        "Sludge Monster",
        "If a non-Horror creature with a slime counter on it somehow becomes a Horror, the effect making it 2/2 and making it lose its abilities ends."
    );
    supported("Sludge Monster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sludge Monster");
    let angel = t.battlefield(P1, "Serra Angel");
    // A slime counter from another source.
    add(&mut t, angel, "slime", 1);
    add(&mut t, angel, "+1/+1", 1);
    assert_eq!(t.pt(angel), (3, 3));
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    // It becomes a Horror (every creature type): 4/4 flying again, plus its counter.
    t.battlefield(P1, "Maskwood Nexus");
    t.g.recompute();
    assert_eq!(t.pt(angel), (5, 5));
    assert!(has_kw(&t, angel, KeywordKind::Flying));
}

#[test]
fn agent_frank_horrigan_indestructible_only_if_declared_as_attacker() {
    cr!("508.4", "508.1");
    ruling!(
        "Agent Frank Horrigan",
        "If Agent Frank Horrigan enters the battlefield attacking, it doesn’t have indestructible since it was never declared as an attacker."
    );
    ruling!(
        "Agent Frank Horrigan",
        "The ability that causes Agent Frank Horrigan to have indestructible starts to apply as soon as it’s declared as an attacker. It applies for the remainder of the turn. It doesn’t matter what happens to the player or permanent it attacked after that point."
    );
    supported("Agent Frank Horrigan");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::DeclareAttackers);
    let frank = crate::r_p079_common::enter_attacking(
        &mut t,
        P0,
        "Agent Frank Horrigan",
        Entity::Player(P1),
    );
    assert!(!has_kw(&t, frank, KeywordKind::Indestructible));
    // Declared attacking a planeswalker, which then leaves.
    let mut t = TestGame::new(2);
    let frank = t.battlefield(P0, "Agent Frank Horrigan");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(frank, obj(jace))]);
    assert!(has_kw(&t, frank, KeywordKind::Indestructible));
    destroy(&mut t, jace);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(has_kw(&t, frank, KeywordKind::Indestructible));
    destroy(&mut t, frank);
    assert!(t.on_battlefield(frank));
}

#[test]
fn agent_frank_horrigan_proliferates_twice_in_one_resolution() {
    cr!("701.34a", "608.2c");
    ruling!(
        "Agent Frank Horrigan",
        "While proliferating twice, players can’t respond between proliferating the first time and proliferating the second time."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bears, "+1/+1", 1);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.enter(P0, "Agent Frank Horrigan");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

#[test]
fn soul_of_windgrace_rulings() {
    cr!("508.4", "608.2c", "602.1");
    ruling!(
        "Soul of Windgrace",
        "If Soul of Windgrace enters the battlefield attacking, its first ability triggers only once that combat."
    );
    ruling!(
        "Soul of Windgrace",
        "You choose the land card as the triggered ability resolves."
    );
    ruling!(
        "Soul of Windgrace",
        "You may activate Soul of Windgrace's last ability even if it's already tapped. It will still gain indestructible."
    );
    supported("Soul of Windgrace");
    // Enters attacking: one trigger (it entered; it wasn't declared as an attacker).
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.set_step(P0, Step::DeclareAttackers);
    let soul =
        crate::r_p079_common::enter_attacking(&mut t, P0, "Soul of Windgrace", Entity::Player(P1));
    assert_eq!(triggers_from(&t, soul), 1);
    // The land is chosen on resolution: one put into the graveyard after it triggered.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let soul = t.enter(P0, "Soul of Windgrace");
    t.settle();
    assert_eq!(triggers_from(&t, soul), 1);
    let forest = t.graveyard(P1, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert_eq!(controller(&mut t, forest), P0);
    // Tapped: the last ability still gives it indestructible.
    let mut t = TestGame::new(2);
    let soul = t.battlefield(P0, "Soul of Windgrace");
    t.g.tap(soul);
    t.hand(P0, "Forest");
    t.lands(P0, "Swamp", 3);
    activate_containing(&mut t, P0, soul, "indestructible").unwrap();
    t.resolve_all();
    assert!(has_kw(&t, soul, KeywordKind::Indestructible));
    assert!(tapped(&t, soul));
}

#[test]
fn glint_sleeve_siphoner_triggers_on_entering_and_on_attacking() {
    cr!("603.2");
    ruling!(
        "Glint-Sleeve Siphoner",
        "The first triggered ability triggers both when Glint-Sleeve Siphoner enters the battlefield and whenever it attacks. You don't have to choose only one."
    );
    supported("Glint-Sleeve Siphoner");
    let mut t = TestGame::new(2);
    let gs = t.enter(P0, "Glint-Sleeve Siphoner");
    t.resolve_all();
    assert_eq!(crate::r_s28_common::energy(&t, P0), 1);
    let gs = t.g.current(gs);
    t.g.objects[gs.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(gs, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(crate::r_s28_common::energy(&t, P0), 2);
}

#[test]
fn naomi_and_bloodhall_priest_check_their_condition_twice() {
    cr!("603.4");
    ruling!(
        "Naomi, Pillar of Order",
        "If you don't control both an artifact and an enchantment at the moment Naomi, Pillar of Order enters the battlefield or attacks, its ability won't trigger at all. If the ability triggers, it will check again as it tries to resolve."
    );
    ruling!(
        "Bloodhall Priest",
        "If you have a card in hand at the moment the trigger condition occurs, Bloodhall Priest’s ability won’t trigger"
    );
    supported("Naomi, Pillar of Order");
    supported("Bloodhall Priest");
    // Naomi: no enchantment, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sol Ring");
    let naomi = t.enter(P0, "Naomi, Pillar of Order");
    t.settle();
    assert_eq!(triggers_from(&t, naomi), 0);
    // Both: it triggers; the artifact leaves before it resolves: no token.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Glorious Anthem");
    let naomi = t.enter(P0, "Naomi, Pillar of Order");
    t.settle();
    assert_eq!(triggers_from(&t, naomi), 1);
    destroy(&mut t, ring);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 0);
    // Bloodhall Priest: a card in hand, no trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    let priest = t.enter(P0, "Bloodhall Priest");
    t.settle();
    assert_eq!(triggers_from(&t, priest), 0);
    // Empty hand: it triggers; a card in hand on resolution: no damage.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let priest = t.enter(P0, "Bloodhall Priest");
    t.settle();
    assert_eq!(triggers_from(&t, priest), 1);
    t.hand(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sanctuary_warden_removes_any_kind_of_counter() {
    cr!("122.1");
    ruling!(
        "Sanctuary Warden",
        "Sanctuary Warden's triggered ability allows you to remove any kind of counter, not just a shield counter."
    );
    supported("Sanctuary Warden");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bears, "+1/+1", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let warden = t.enter(P0, "Sanctuary Warden");
    let h = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(warden, "shield"), 2);
    assert_eq!(t.hand_size(P0), h + 1);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn the_wise_mothman_triggers_once_when_several_players_mill() {
    cr!("603.2c", "701.17a");
    ruling!(
        "The Wise Mothman",
        "If more than one player is instructed to mill at the same time (by the activated ability of Raul, Trouble Shooter, for example), as long as at least one nonland card is milled, The Wise Mothman’s last ability will trigger just once."
    );
    supported("The Wise Mothman");
    supported("Raul, Trouble Shooter");
    let mut t = TestGame::new(2);
    let moth = t.battlefield(P0, "The Wise Mothman");
    let raul = t.battlefield(P0, "Raul, Trouble Shooter");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    activate_containing(&mut t, P0, raul, "mills").unwrap();
    t.resolve();
    assert_eq!(triggers_from(&t, moth), 1);
}

#[test]
fn harvestrite_host_a_countered_resolution_doesnt_count() {
    cr!("608.2b");
    ruling!(
        "Harvestrite Host",
        "If the target creature is an illegal target as Harvestrite Host’s ability tries to resolve, it won’t resolve and none of its effects will happen. It won’t count toward the number of times the ability has resolved this turn."
    );
    supported("Harvestrite Host");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let host = t.enter(P0, "Harvestrite Host");
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    // A second Host enters: the first Host's ability triggers again (its first
    // resolution this turn, as the fizzled one doesn't count), and so does the new
    // Host's own (its first). Neither draws.
    let h = t.hand_size(P0);
    t.answer_targets(P0, &[obj(host)]);
    t.answer_targets(P0, &[obj(host)]);
    let host2 = t.enter(P0, "Harvestrite Host");
    t.settle();
    assert_eq!(triggers_from(&t, host) + triggers_from(&t, host2), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h, "no ability has resolved twice yet");
    assert_eq!(t.pt(host), (5, 3), "both resolved, each giving +1/+0");
    // A third Host: now the first Host's ability resolves for the second time (draw),
    // the second Host's ability for the second time (draw), the third's for the first.
    t.answer_targets(P0, &[obj(host)]);
    t.answer_targets(P0, &[obj(host)]);
    t.answer_targets(P0, &[obj(host)]);
    t.enter(P0, "Harvestrite Host");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 2);
}

#[test]
fn nut_collector_squirrel_damage_becomes_lethal_without_threshold() {
    cr!("704.5g", "613.4c");
    ruling!(
        "Nut Collector",
        "nonlethal damage dealt to Squirrels may become lethal if the number of cards in your graveyard falls below seven."
    );
    supported("Nut Collector");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    t.battlefield(P0, "Nut Collector");
    let squirrel = crate::r_s02_common::create_token(&mut t, P0, "Squirrel");
    assert_eq!(t.pt(squirrel), (3, 3));
    crate::r_s06_common::damage(&mut t, squirrel, 2, squirrel);
    assert!(t.on_battlefield(squirrel));
    let card = t.g.player(P0).graveyard[0];
    t.g.exile_object(card, None);
    t.settle();
    assert!(!t.on_battlefield(squirrel), "2 damage on a 1/1");
}

#[test]
fn deep_sea_terror_checks_only_as_attackers_are_declared() {
    cr!("508.1c");
    ruling!(
        "Deep-Sea Terror",
        "Deep-Sea Terror’s ability only checks as attackers are declared."
    );
    supported("Deep-Sea Terror");
    let mut t = TestGame::new(2);
    let terror = t.battlefield(P0, "Deep-Sea Terror");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, terror), "six or fewer cards");
    fill_graveyard(&mut t, P0, 7);
    assert!(can_attack(&mut t, terror));
    attack_with(&mut t, &[(terror, Entity::Player(P1))]);
    for _ in 0..7 {
        let card = t.g.player(P0).graveyard[0];
        t.g.exile_object(card, None);
    }
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14, "still attacking");
}

#[test]
fn swirling_sandstorm_needs_threshold_as_it_resolves() {
    cr!("608.2c");
    ruling!(
        "Swirling Sandstorm",
        "This card does nothing if you don’t have seven cards in your graveyard when it resolves."
    );
    supported("Swirling Sandstorm");
    for (cards, dies) in [(6, false), (7, true)] {
        let mut t = TestGame::new(2);
        fill_graveyard(&mut t, P0, cards);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_resolve(&mut t, P0, "Swirling Sandstorm", &[]);
        assert_eq!(!t.on_battlefield(bears), dies, "{cards} cards");
    }
}

#[test]
fn krosan_restorer_abilities_use_the_stack() {
    cr!("605.1a", "602.2");
    ruling!(
        "Krosan Restorer",
        "Krosan Restorer’s activated abilities aren’t mana abilities. They each use the stack and can be responded to."
    );
    supported("Krosan Restorer");
    let mut t = TestGame::new(2);
    let kr = t.battlefield(P0, "Krosan Restorer");
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    t.activate(P0, kr, 0, &[obj(forest)]).unwrap();
    assert_eq!(t.stack_len(), 1, "on the stack");
    assert!(tapped(&t, forest));
    t.resolve_all();
    assert!(!tapped(&t, forest));
}

#[test]
fn nezahal_returns_as_a_new_object() {
    cr!("400.7", "603.7a");
    ruling!(
        "Nezahal, Primal Tide",
        "After Nezahal returns to the battlefield, it will be a new object with no connection to the creature that was exiled."
    );
    let mut t = TestGame::new(2);
    let nez = t.battlefield(P0, "Nezahal, Primal Tide");
    add(&mut t, nez, "+1/+1", 1);
    let aura = crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", obj(nez));
    crate::r_p179_common::fill_hand(&mut t, P0, 3);
    activate_containing(&mut t, P0, nez, "Exile").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(nez), Zone::Exile);
    assert!(!t.on_battlefield(aura));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(nez));
    assert!(tapped(&t, nez));
    assert_eq!(t.counters(nez, "+1/+1"), 0);
    assert_eq!(t.pt(nez), (7, 7));
}

#[test]
fn the_raven_man_triggers_once_per_end_step() {
    cr!("603.4", "603.2");
    ruling!(
        "The Raven Man",
        "It only triggers once, not once for each card discarded or once for each player that discarded a card. It will trigger even if its controller is the one that discarded a card this turn."
    );
    supported("The Raven Man");
    let mut t = TestGame::new(2);
    let raven = t.battlefield(P0, "The Raven Man");
    for p in [P0, P0, P1] {
        let c = t.hand(p, "Forest");
        t.g.discard(p, c, None);
    }
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, raven), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Only P0 discarded: still triggers.
    let mut t = TestGame::new(2);
    let raven = t.battlefield(P0, "The Raven Man");
    let c = t.hand(P0, "Forest");
    t.g.discard(P0, c, None);
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, raven), 1);
}
