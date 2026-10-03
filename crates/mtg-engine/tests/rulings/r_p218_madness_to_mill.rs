//! Rulings batch P218 — madness (CR 702.35), magecraft (an ability word, CR 207.2c),
//! melee (CR 702.121), menace (CR 702.111), metalcraft (CR 207.2c) and mill (CR 701.17).

use crate::r_s01_common::{attack_with, stack_library, supported, triggers_on_stack, watch};
use crate::r_s02_common::{can_cast as can_cast_now, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::activate_containing;
use crate::r_s09_common::declare;
use crate::r_s11_common::triggered_from;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` discards `card` (as an effect would) and the triggers are put on the stack.
fn discard(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    t.g.discard(p, card, None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn grave_scrabbler_madness_cost_paid_even_if_reduced() {
    cr!("702.35a", "601.2f", "702.35c");
    ruling!(
        "Grave Scrabbler",
        "Grave Scrabbler's madness cost was paid if you cast it using its madness ability, even if an effect changed the amount of mana you paid for that cost."
    );
    supported("Grave Scrabbler");
    supported("Undead Warchief");
    // Undead Warchief: "Zombie spells you control cost {1} less to cast." Grave Scrabbler's
    // madness cost {1}{B} becomes {B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Undead Warchief");
    t.lands(P0, "Swamp", 1);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let scrabbler = t.hand(P0, "Grave Scrabbler");
    discard(&mut t, P0, scrabbler);
    assert!(t.in_exile("Grave Scrabbler"));
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.g.find_in_zone(Zone::Stack, "Grave Scrabbler").len(), 1);
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grave Scrabbler").len(), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn anjes_ravager_madness_choice_comes_after_drawing() {
    cr!("702.35a", "603.12", "608.2c");
    ruling!(
        "Anje's Ravager",
        "If any of the cards you discard have madness, you'll choose whether to cast them after you've drawn three cards."
    );
    supported("Anje's Ravager");
    supported("Fiery Temper");
    let mut t = TestGame::new(2);
    let ravager = t.battlefield(P0, "Anje's Ravager");
    t.hand(P0, "Fiery Temper");
    t.hand(P0, "Forest");
    stack_library(&mut t, P0, &["Island", "Island", "Island"]);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(P0).hand.len(),
    );
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(ravager, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![3]);
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn emrakul_discarded_goes_to_exile_then_cast_or_graveyard() {
    cr!("702.35a", "702.35b");
    ruling!(
        "Emrakul, the World Anew",
        "If you discard a card with madness, you discard it into exile instead of into your graveyard. When you do, you can either cast it from exile for its madness cost or put it into your graveyard."
    );
    // Declining: it goes to the graveyard.
    let mut t = TestGame::new(2);
    let emrakul = t.hand(P0, "Emrakul, the World Anew");
    discard(&mut t, P0, emrakul);
    assert!(t.in_exile("Emrakul, the World Anew"));
    assert_eq!(triggers_on_stack(&t, "Madness"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Emrakul, the World Anew"));
    // Casting it for six {C}.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::C, 6);
    let emrakul = t.hand(P0, "Emrakul, the World Anew");
    discard(&mut t, P0, emrakul);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let on_stack = t.g.find_in_zone(Zone::Stack, "Emrakul, the World Anew");
    assert_eq!(on_stack.len(), 1);
    assert_eq!(
        t.g.obj(on_stack[0]).stack.as_ref().map(|s| s.cast.method.clone()),
        Some(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Madness))
    );
}

#[test]
fn extus_magecraft_must_target_a_nonlegendary_creature_card() {
    cr!("603.3d", "115.1", "601.2c");
    ruling!(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        "Returning a nonlegendary creature card is not optional. If there is at least one in your graveyard, you must target it with the magecraft ability."
    );
    // P0 doesn't name a target: the only legal one is chosen anyway.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Extus, Oriq Overlord // Awaken the Blood Avatar");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
}

#[test]
fn veyran_doubles_each_magecraft_trigger_including_its_own() {
    cr!("603.2d", "603.2");
    ruling!(
        "Veyran, Voice of Duality",
        "Veyran, Voice of Duality's last ability causes each of your magecraft abilities, including Veyran's own, to trigger an additional time."
    );
    supported("Veyran, Voice of Duality");
    supported("Archmage Emeritus");
    let mut t = TestGame::new(2);
    let veyran = t.battlefield(P0, "Veyran, Voice of Duality");
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    t.lands(P0, "Mountain", 1);
    let hand = t.hand_size(P0);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(triggered_from(&t, veyran), 2);
    assert_eq!(triggered_from(&t, emeritus), 2);
    assert_eq!(t.pt(veyran), (4, 4));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn tifa_melee_counts_only_opponents_attacked() {
    cr!("702.121a", "702.121b");
    ruling!(
        "Tifa, Martial Artist",
        "Melee will trigger if the creature with melee attacks a planeswalker or battle. However, the effect counts only opponents (and not planeswalkers or battles) that you attacked with a creature when determining the bonus."
    );
    // Tifa attacks P1's planeswalker: melee triggers, but no opponent was attacked.
    let mut t = TestGame::new(2);
    let tifa = t.battlefield(P0, "Tifa, Martial Artist");
    let jace = t.battlefield(P1, "Jace Beleren");
    declare(&mut t, P0, &[(tifa, Entity::Object(jace))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Melee"), 1);
    t.resolve_all();
    assert_eq!(t.pt(tifa), (4, 4));
    // Another creature attacking P1 makes it count P1.
    let mut t = TestGame::new(2);
    let tifa = t.battlefield(P0, "Tifa, Martial Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    declare(
        &mut t,
        P0,
        &[(tifa, Entity::Object(jace)), (bears, Entity::Player(P1))],
    );
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(tifa), (5, 5));
}

#[test]
fn custodi_soulcaller_counts_players_attacked_even_if_gone() {
    cr!("702.121b", "800.4a", "608.2h");
    ruling!(
        "Custodi Soulcaller",
        "The value of X in the last ability is calculated in a similar fashion to how melee bonuses are calculated. It doesn’t matter if the creatures are still attacking or on the battlefield. It also doesn’t matter if the player you attacked is still in the game."
    );
    supported("Custodi Soulcaller");
    // P0 attacks P1 with the Soulcaller and P2 with the Bears; then the Bears die and P2
    // leaves the game before the triggers resolve. X is still 2.
    let mut t = TestGame::new(3);
    let soulcaller = t.battlefield(P0, "Custodi Soulcaller");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let returned = t.graveyard(P0, "Runeclaw Bear");
    t.answer_targets(P0, &[Entity::Object(returned)]);
    declare(
        &mut t,
        P0,
        &[
            (soulcaller, Entity::Player(P1)),
            (bears, Entity::Player(P2)),
        ],
    );
    t.settle();
    destroy(&mut t, bears);
    t.g.player_loses(P2);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Runeclaw Bear").len(), 1);
    assert_eq!(t.pt(soulcaller), (3, 4));
}

#[test]
fn wulfgar_makes_melee_trigger_an_additional_time() {
    cr!("702.121a", "603.2d");
    ruling!(
        "Wulfgar of Icewind Dale",
        "Wulfgar's last ability will cause its melee ability to trigger an additional time."
    );
    supported("Wulfgar of Icewind Dale");
    let mut t = TestGame::new(2);
    let wulfgar = t.battlefield(P0, "Wulfgar of Icewind Dale");
    declare(&mut t, P0, &[(wulfgar, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Melee"), 2);
    t.resolve_all();
    assert_eq!(t.pt(wulfgar), (6, 6));
}

#[test]
fn labyrinth_raptor_abilities_apply_to_itself_while_it_has_menace() {
    cr!("702.111a", "509.1h");
    ruling!(
        "Labyrinth Raptor",
        "Labyrinth Raptor’s second and third abilities apply to itself as long as it still has menace."
    );
    supported("Labyrinth Raptor");
    // Third ability: the Raptor (menace) gets +1/+0; the Bears (no menace) don't.
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Labyrinth Raptor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    activate_containing(&mut t, P0, raptor, "get +1/+0").expect("activated");
    t.resolve_all();
    assert_eq!(t.pt(raptor), (3, 2));
    assert_eq!(t.pt(bears), (2, 2));
    // Second ability: blocked by two creatures, P1 sacrifices one of them.
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Gray Ogre");
    attack_with(&mut t, &[(raptor, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(b1, raptor), (b2, raptor)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggered_from(&t, raptor), 1);
    t.resolve_all();
    assert_eq!(
        [b1, b2].iter().filter(|b| t.on_battlefield(**b)).count(),
        1
    );
}

#[test]
fn argent_sphinx_activated_in_an_end_step_returns_in_the_next_turns() {
    cr!("603.7a", "603.7b", "513.1");
    ruling!(
        "Argent Sphinx",
        "If you activate Argent Sphinx’s metalcraft ability during a turn’s end step, Argent Sphinx will return to the battlefield at the beginning of the following turn’s end step."
    );
    supported("Argent Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Argent Sphinx");
    t.lands(P0, "Ornithopter", 3);
    t.lands(P0, "Island", 1);
    t.set_step(P0, Step::End);
    activate_containing(&mut t, P0, sphinx, "Exile").expect("activated");
    t.resolve_all();
    assert!(t.in_exile("Argent Sphinx"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Argent Sphinx"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Argent Sphinx").len(), 1);
}

#[test]
fn argent_sphinx_returns_regardless_of_artifacts() {
    cr!("603.7a", "602.5b");
    ruling!(
        "Argent Sphinx",
        "If you activate Argent Sphinx’s metalcraft ability, Argent Sphinx will return to the battlefield at the beginning of the next end step no matter how many artifacts you control at that time."
    );
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Argent Sphinx");
    let thopters = t.lands(P0, "Ornithopter", 3);
    t.lands(P0, "Island", 1);
    activate_containing(&mut t, P0, sphinx, "Exile").expect("activated");
    t.resolve_all();
    assert!(t.in_exile("Argent Sphinx"));
    for th in thopters {
        destroy(&mut t, th);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Argent Sphinx").len(), 1);
}

#[test]
fn blade_tribe_berserkers_keeps_its_bonus_without_the_artifacts() {
    cr!("611.2c", "603.4");
    ruling!(
        "Blade-Tribe Berserkers",
        "Once the metalcraft ability resolves, Blade-Tribe Berserkers retains its bonuses for the rest of the turn, even if you cease to control three or more artifacts for some reason."
    );
    supported("Blade-Tribe Berserkers");
    let mut t = TestGame::new(2);
    let thopters = t.lands(P0, "Ornithopter", 3);
    let berserkers = enter(&mut t, P0, "Blade-Tribe Berserkers");
    t.resolve_all();
    assert_eq!(t.pt(berserkers), (6, 6));
    for th in thopters {
        destroy(&mut t, th);
    }
    assert_eq!(t.pt(berserkers), (6, 6));
    assert!(t
        .obj_now(berserkers)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn stoic_rebuttal_cost_reduction_works_on_the_stack() {
    cr!("601.2f", "113.6a");
    ruling!(
        "Stoic Rebuttal",
        "Stoic Rebuttal’s metalcraft ability functions while Stoic Rebuttal is on the stack."
    );
    supported("Stoic Rebuttal");
    // With three artifacts, {U}{U} is enough to cast it.
    for (artifacts, ok) in [(3, true), (2, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Ornithopter", artifacts);
        t.lands(P0, "Island", 2);
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.cast(P1, bolt).target(P0).go();
        let rebuttal = t.hand(P0, "Stoic Rebuttal");
        assert_eq!(can_cast_now(&mut t, P0, rebuttal, CastMethod::Normal), ok);
        if ok {
            let bolt = t.g.stack[0];
            t.cast(P0, rebuttal).target(bolt).go();
            t.resolve_all();
            assert_eq!(t.life(P0), 20);
            assert!(t.in_graveyard(P1, "Lightning Bolt"));
        }
    }
}

#[test]
fn molten_psyche_counts_every_card_drawn_this_turn() {
    cr!("121.1", "120.3");
    ruling!(
        "Molten Psyche",
        "The metalcraft effect counts all cards each opponent drew for any reason during that turn, not just the cards those opponents drew due to Molten Psyche’s first effect."
    );
    supported("Molten Psyche");
    let mut t = TestGame::new(2);
    t.lands(P0, "Ornithopter", 3);
    t.lands(P0, "Mountain", 3);
    // P1 drew two cards earlier this turn, then has one more card in hand.
    t.g.draw_cards(P1, 2);
    t.hand(P1, "Forest");
    let hand = t.hand_size(P1);
    let psyche = t.hand(P0, "Molten Psyche");
    t.cast(P0, psyche).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(t.life(P1), 20 - (2 + hand as i32));
}

#[test]
fn free_the_fae_adventurer_cards_are_creature_cards_in_the_graveyard() {
    cr!("715.4", "701.17a");
    ruling!(
        "Picklock Prankster // Free the Fae",
        "Adventurer cards aren't instant or sorcery cards while they're in your graveyard. You can't use Free the Fae to put an adventurer card milled with it into your hand unless that card is a Faerie."
    );
    supported("Picklock Prankster // Free the Fae");
    // (milled cards, which one P0 may put into their hand)
    for (milled, expected) in [
        (
            ["Bonecrusher Giant // Stomp", "Grizzly Bears", "Forest", "Forest"],
            None,
        ),
        (
            [
                "Bonecrusher Giant // Stomp",
                "Picklock Prankster // Free the Fae",
                "Forest",
                "Forest",
            ],
            Some("Picklock Prankster"),
        ),
    ] {
        let mut t = TestGame::new(2);
        let ids = stack_library(&mut t, P0, &milled);
        t.lands(P0, "Island", 2);
        let prankster = t.hand(P0, "Picklock Prankster // Free the Fae");
        t.cast(P0, prankster).method(CastMethod::Half(1)).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Bonecrusher Giant"));
        match expected {
            None => assert!(ids.iter().all(|id| t.zone(*id) == Zone::Graveyard(P0))),
            Some(name) => assert!(t.in_hand(P0, name)),
        }
    }
}

#[test]
fn colossal_grave_reaver_mills_at_once_and_triggers_once() {
    cr!("701.17a", "603.2c");
    ruling!(
        "Colossal Grave-Reaver",
        "All three of the milled cards are put into your graveyard at the same time. If there is more than one creature card among those cards, the last ability triggers only once."
    );
    supported("Colossal Grave-Reaver");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Forest"]);
    let reaver = enter(&mut t, P0, "Colossal Grave-Reaver");
    t.resolve_all();
    assert_eq!(triggered_from(&t, reaver), 2, "the mill and one put-onto trigger");
    let back = t.named_on_battlefield("Grizzly Bears").len() + t.named_on_battlefield("Hill Giant").len();
    assert_eq!(back, 1);
}

#[test]
fn locke_creates_one_treasure_however_many_lands_are_milled() {
    cr!("701.17a", "111.10a");
    ruling!(
        "Locke, Treasure Hunter",
        "As long as one or more lands were milled this way, you'll create a Treasure token. Additional land cards milled beyond the first won't cause you to create additional Treasures."
    );
    supported("Locke, Treasure Hunter");
    for (tops, treasures) in [(["Forest", "Island"], 1), (["Forest", "Grizzly Bears"], 1), (["Grizzly Bears", "Hill Giant"], 0)] {
        let mut t = TestGame::new(2);
        let locke = t.battlefield(P0, "Locke, Treasure Hunter");
        t.library_top(P0, tops[0]);
        t.library_top(P1, tops[1]);
        attack_with(&mut t, &[(locke, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Treasure").len(), treasures, "{tops:?}");
    }
}
