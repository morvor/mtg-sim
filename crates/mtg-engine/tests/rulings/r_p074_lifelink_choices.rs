//! Rulings batch P074 — choices and conditions around lifelink grants: "if you control an
//! artifact and an enchantment" (one permanent can be both; checked on resolution),
//! choices made on resolution (Talus Paladin, Rattleback Apothecary), Bard the Bowman's
//! second-card-drawn trigger, Radiant Scrollwielder's exiled card, The Wind Crystal's
//! cost reduction, and Nethroi's total-power targets.

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::can_cast;
use crate::r_s03_common::respond;
use crate::r_s04_common::{add_mana, next_upkeep};
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::has_kw;
use crate::r_s08_common::mana_value;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const LIFELINK: KeywordKind = KeywordKind::Lifelink;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn an_artifact_enchantment_is_both_the_artifact_and_the_enchantment() {
    cr!("205.2a", "603.4", "700.2");
    ruling!(
        "Kami of Terrible Secrets",
        "The artifact and the enchantment may be the same permanent."
    );
    ruling!(
        "Naomi, Pillar of Order",
        "The artifact and the enchantment may be the same permanent."
    );
    ruling!(
        "Soul Transfer",
        "The artifact and the enchantment may be the same permanent."
    );
    ruling!(
        "When We Were Young",
        "The artifact and the enchantment may be the same permanent."
    );
    for name in [
        "Naomi, Pillar of Order",
        "Soul Transfer",
        "When We Were Young",
        "Bow of Nylea",
    ] {
        supported(name);
    }
    // Bow of Nylea is an artifact enchantment. Kami: draw a card and gain 1 life.
    for with_bow in [true, false] {
        let mut t = TestGame::new(2);
        if with_bow {
            t.battlefield(P0, "Bow of Nylea");
        }
        let hand = t.hand_size(P0);
        enter(&mut t, P0, "Kami of Terrible Secrets");
        t.resolve_all();
        let n = with_bow as usize;
        assert_eq!(t.hand_size(P0), hand + n, "Kami, bow {with_bow}");
        assert_eq!(t.life(P0), 20 + n as i32);
        // Naomi: a Samurai token.
        let mut t = TestGame::new(2);
        if with_bow {
            t.battlefield(P0, "Bow of Nylea");
        }
        enter(&mut t, P0, "Naomi, Pillar of Order");
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Samurai").len(), n, "Naomi");
        // When We Were Young: +2/+2 and lifelink.
        let mut t = TestGame::new(2);
        if with_bow {
            t.battlefield(P0, "Bow of Nylea");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        give_mana_for(&mut t, P0, "When We Were Young");
        let w = t.hand(P0, "When We Were Young");
        t.cast(P0, w).targets(&[obj(bears)]).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (4, 4));
        assert_eq!(has_kw(&t, bears, LIFELINK), with_bow, "WWWY");
    }
    // Soul Transfer: both modes with only Bow of Nylea.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bow of Nylea");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let dead = t.graveyard(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Soul Transfer");
    let st = t.hand(P0, "Soul Transfer");
    t.cast(P0, st)
        .modes(&[0, 1])
        .targets(&[obj(elves)])
        .targets(&[obj(dead)])
        .go();
    t.resolve_all();
    assert!(t.in_exile("Llanowar Elves"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn when_we_were_young_checks_on_resolution_and_affects_only_its_targets() {
    cr!("608.2c", "608.2b");
    ruling!(
        "When We Were Young",
        "Whether you control an artifact and an enchantment is checked as When We Were Young resolves."
    );
    ruling!(
        "When We Were Young",
        "“Those creatures” refers to the target creatures, not the artifact and the enchantment."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "When We Were Young");
    let w = t.hand(P0, "When We Were Young");
    t.cast(P0, w).targets(&[obj(bears)]).go();
    // After casting, before it resolves: an artifact creature and an enchantment arrive.
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    t.resolve_all();
    assert!(has_kw(&t, bears, LIFELINK));
    assert!(!has_kw(&t, thopter, LIFELINK), "the artifact isn't one of those creatures");
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn talus_paladin_choices_are_made_separately() {
    cr!("603.5", "608.2d");
    ruling!(
        "Talus Paladin",
        "The choices you make as Talus Paladin’s ability resolves are made individually. First you decide whether to have Allies you control gain lifelink until end of turn. Then, regardless of that decision, you decide whether to put a +1/+1 counter on Talus Paladin."
    );
    supported("Talus Paladin");
    for (lifelink, counter) in [(false, true), (true, false)] {
        let mut t = TestGame::new(2);
        let paladin = t.battlefield(P0, "Talus Paladin");
        let from = t.asked().len();
        t.answer_yes(P0, lifelink).answer_yes(P0, counter);
        enter(&mut t, P0, "Talus Paladin");
        t.resolve_all();
        let yes_no = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count();
        // Both Paladins trigger (each for the entering Ally); two questions each.
        assert_eq!(yes_no, 4);
        assert_eq!(has_kw(&t, paladin, LIFELINK), lifelink);
        assert!(t.counters(paladin, "+1/+1") <= 1);
        assert_eq!(t.counters(paladin, "+1/+1") == 1, counter);
    }
}

fn pick_menace_or_lifelink(want: &'static str) -> fn(&mtg_engine::game::Game, &Decision) -> Option<Answer> {
    fn lifelink(_: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        pick(d, "lifelink")
    }
    fn menace(_: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        pick(d, "menace")
    }
    fn pick(d: &Decision, want: &str) -> Option<Answer> {
        match d {
            Decision::ChooseOption { options, .. } => options
                .iter()
                .position(|o| o.to_lowercase().contains(want))
                .map(Answer::Index),
            _ => None,
        }
    }
    if want == "lifelink" {
        lifelink
    } else {
        menace
    }
}

#[test]
fn rattleback_apothecary_chooses_the_ability_on_resolution() {
    cr!("608.2d", "700.13");
    ruling!(
        "Rattleback Apothecary",
        "You choose which ability the target creature gets as Rattleback Apothecary’s triggered ability resolves."
    );
    supported("Rattleback Apothecary");
    for want in ["lifelink", "menace"] {
        let mut t = TestGame::new(2);
        let rb = t.battlefield(P0, "Rattleback Apothecary");
        respond(&mut t, P0, pick_menace_or_lifelink(want));
        give_mana_for(&mut t, P0, "Lightning Bolt");
        let bolt = t.hand(P0, "Lightning Bolt");
        t.answer_targets(P0, &[obj(rb)]);
        t.cast(P0, bolt).target(pl(P1)).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "the crime trigger");
        let asked_options = |t: &TestGame| {
            t.asked()
                .iter()
                .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
                .count()
        };
        assert_eq!(asked_options(&t), 0, "not chosen yet");
        t.resolve();
        assert_eq!(asked_options(&t), 1);
        assert_eq!(has_kw(&t, rb, LIFELINK), want == "lifelink");
        assert_eq!(has_kw(&t, rb, KeywordKind::Menace), want == "menace");
    }
}

fn target_soldier(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseTargets { candidates, .. } => candidates
            .iter()
            .find(|e| match e {
                Entity::Object(o) => g.obj(*o).chars.name.contains("Soldier"),
                _ => false,
            })
            .map(|e| Answer::Entities(vec![*e])),
        _ => None,
    }
}

#[test]
fn bard_the_bowman_second_card_while_recruiting_can_target_the_soldier() {
    cr!("701.70a", "603.3d", "121.2");
    ruling!(
        "Bard the Bowman",
        "If you draw your second card in a turn while recruiting, the Human Soldier you create can be chosen as the target of Bard the Bowman's ability."
    );
    supported("Bard the Bowman");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bard the Bowman");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    respond(&mut t, P0, target_soldier);
    // Patient Instructor: "When this creature enters, recruit." The drawn and discarded
    // card is a nonland filler.
    enter(&mut t, P0, "Patient Instructor");
    t.resolve_all();
    let soldiers = tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 1);
    assert_eq!(t.counters(soldiers[0], "+1/+1"), 1);
    assert!(has_kw(&t, soldiers[0], LIFELINK));
}

#[test]
fn bard_the_bowman_triggers_only_on_the_second_card() {
    cr!("603.2", "121.2");
    ruling!(
        "Bard the Bowman",
        "The triggered ability can trigger only once each turn. It doesn't matter whether Bard the Bowman was on the battlefield when the first card was drawn. If he's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn."
    );
    // Bard arrives after the first draw: the second triggers, the third doesn't.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    let bard = t.battlefield(P0, "Bard the Bowman");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.answer_targets(P0, &[obj(bard)]);
    t.resolve_all();
    assert_eq!(t.counters(bard, "+1/+1"), 1);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Bard arrives after the second draw: nothing that turn.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    let bard = t.battlefield(P0, "Bard the Bowman");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(bard, "+1/+1"), 0);
}

#[test]
fn radiant_scrollwielder_card_still_needs_its_mana_cost_and_timing() {
    cr!("601.2f", "307.1", "117.1a");
    ruling!(
        "Radiant Scrollwielder",
        "You still pay the mana cost when casting the exiled card, and you must follow all timing rules for it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Radiant Scrollwielder");
    let card = t.graveyard(P0, "Lava Spike");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Exile);
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal), "a sorcery in upkeep");
    t.g.players[P0.idx()].mana_pool.empty();
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal), "no mana");
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23, "lifelink");
    assert!(t.in_exile("Lava Spike"));
}

#[test]
fn the_wind_crystal_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "The Wind Crystal",
        "The cost reduction applies only to generic mana in the total cost of white spells you cast."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Wind Crystal");
    let stp = t.hand(P0, "Swords to Plowshares");
    assert!(!can_cast(&mut t, P0, stp, CastMethod::Normal));
    let raise = t.hand(P0, "Raise the Alarm");
    t.lands(P0, "Plains", 1);
    assert!(can_cast(&mut t, P0, raise, CastMethod::Normal), "{{1}}{{W}} for {{W}}");
    t.cast(P0, raise).go();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Soldier").len(), 2);
    // A red spell isn't reduced.
    let shock = t.hand(P0, "Lava Axe");
    t.lands(P0, "Mountain", 4);
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
}

#[test]
fn the_wind_crystal_reduction_applies_to_the_total_cost_not_mana_value() {
    cr!("601.2f", "202.3", "702.33d");
    ruling!(
        "The Wind Crystal",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of The Wind Crystal's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Benalish Lancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Wind Crystal");
    // Benalish Lancer {2}{W}, kicker {2}{W}: kicked, {4}{W}{W} - {1} = five mana.
    let lands = t.lands(P0, "Plains", 5);
    let lancer = t.hand(P0, "Benalish Lancer");
    let spell = t.cast(P0, lancer).kicked(true).go();
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    assert_eq!(mana_value(&t, spell), 3);
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(lancer), "+1/+1"), 2);
}

/// Casts Nethroi for its mutate cost onto `target` (on top), with the mutate trigger's
/// targets `returns`, and resolves everything.
fn mutate_nethroi(t: &mut TestGame, target: ObjectId, returns: &[Entity]) {
    let n = t.hand(P0, "Nethroi, Apex of Death");
    add_mana(t, P0, ManaType::W, 1);
    add_mana(t, P0, ManaType::B, 2);
    add_mana(t, P0, ManaType::C, 4);
    t.cast(P0, n)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(target)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_targets(P0, returns);
    t.resolve_all();
}

#[test]
fn nethroi_returns_cards_with_total_power_ten_or_less() {
    cr!("115.1", "601.2c", "702.140d");
    ruling!(
        "Nethroi, Apex of Death",
        "Nethroi's triggered ability can return any combination of creatures whose powers total 10 or less."
    );
    ruling!(
        "Nethroi, Apex of Death",
        "Use the power of the creature cards as they exist in your graveyard to determine whether they can be the targets of Nethroi's triggered ability."
    );
    supported("Nethroi, Apex of Death");
    let mut t = TestGame::new(2);
    let host = t.battlefield(P0, "Grizzly Bears");
    // Craw Wurm (6) + two Grizzly Bears (2 each) + Ornithopter (0) + Clone (0 in the
    // graveyard, though it would enter as a copy of something bigger).
    let wurm = t.graveyard(P0, "Craw Wurm");
    t.graveyard(P0, "Craw Wurm");
    let b1 = t.graveyard(P0, "Grizzly Bears");
    let b2 = t.graveyard(P0, "Grizzly Bears");
    let thopter = t.graveyard(P0, "Ornithopter");
    let clone = t.graveyard(P0, "Clone");
    let cards = [wurm, b1, b2, thopter, clone].map(obj);
    mutate_nethroi(&mut t, host, &cards);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Craw Wurm"), "the other Wurm wasn't chosen");
    assert!(!t.in_graveyard(P0, "Clone"), "Clone was a legal target");
}

#[test]
fn nethroi_targets_over_ten_power_on_resolution_are_all_illegal() {
    cr!("608.2b", "115.7", "604.3");
    ruling!(
        "Nethroi, Apex of Death",
        "If the power of the target creature cards change while in your graveyard, most likely because one has a characteristic-defining ability that defines a * in its power, the total power of the creatures may become greater than 10. If this happens, the entire selection of targets is illegal and nothing is returned to the battlefield."
    );
    // Tarmogoyf in the graveyard with only creature cards around: 1/2 counting itself.
    let mut t = TestGame::new(2);
    let host = t.battlefield(P0, "Grizzly Bears");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    let wurm = t.graveyard(P0, "Craw Wurm");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let bear2 = t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    // Goyf (1) + Wurm (6) + Elves (1) + Bears (2) = 10.
    assert_eq!(t.pt(goyf).0, 1);
    let n = t.hand(P0, "Nethroi, Apex of Death");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, n)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(host)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_targets(P0, &[obj(goyf), obj(wurm), obj(elves), obj(bear2)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the mutate trigger");
    // In response, a sorcery and an instant go to graveyards: Tarmogoyf becomes 3/4.
    t.graveyard(P1, "Lava Spike");
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(goyf).0, 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tarmogoyf"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}
