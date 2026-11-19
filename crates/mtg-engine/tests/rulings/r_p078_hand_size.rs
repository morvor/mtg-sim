//! Rulings batch P078 — hand size: maximum hand size effects in timestamp order, and
//! abilities that count the cards in a hand (when they check, and what they lock in).

use crate::r_p078_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s25_common::cast_new;
use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Maximum hand size in timestamp order (CR 402.2, 613.7)
// ---------------------------------------------------------------------------------

/// The maximum hand size of `of` after the permanents enter in order, each (controller,
/// name); a Cursed Rack chooses its controller's opponent (the only one).
fn max_hand_after(order: &[(PlayerId, &str)], of: PlayerId) -> Option<i32> {
    let mut t = TestGame::new(2);
    for (p, name) in order {
        supported(name);
        t.enter(*p, name);
        t.settle();
    }
    t.g.recompute();
    t.g.player(of).max_hand_size
}

/// The cards `p` keeps through their cleanup step with ten cards in hand.
fn kept_through_cleanup(order: &[(PlayerId, &str)], p: PlayerId) -> usize {
    let mut t = TestGame::new(2);
    for (q, name) in order {
        t.enter(*q, name);
        t.settle();
    }
    fill_hand(&mut t, p, 10);
    t.set_step(p, Step::End);
    let next = PlayerId((p.0 + 1) % 2);
    t.advance_to(next, Step::Upkeep);
    t.hand_size(p)
}

#[test]
fn hand_size_reductions_apply_after_a_set_size_in_timestamp_order() {
    cr!("402.2", "514.1", "613.11", "613.7", "613.7d");
    ruling!(
        "Thought Devourer",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you are affected by Cursed Rack (an artifact that sets a player's maximum hand size to four) and then put Thought Devourer onto the battlefield, your maximum hand size would be zero. However, if those permanents entered in the opposite order, your maximum hand size would be four."
    );
    ruling!(
        "Thought Eater",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you are affected by Cursed Rack (an artifact that sets a player's maximum hand size to four) and then put Thought Eater onto the battlefield, your maximum hand size would be one. However, if those permanents entered in the opposite order, your maximum hand size would be four."
    );
    ruling!(
        "Thought Nibbler",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you are affected by Cursed Rack (an artifact that sets a player's maximum hand size to four) and then put Thought Nibbler onto the battlefield, your maximum hand size would be two. However, if those permanents entered in the opposite order, your maximum hand size would be four."
    );
    ruling!(
        "Gnat Miser",
        "If multiple effects modify a player's hand size, apply them in timestamp order. For example, if you put Cursed Rack (an artifact that sets a chosen player's maximum hand size to four) onto the battlefield choosing your opponent and then put Gnat Miser onto the battlefield, your opponent will have a maximum hand size of three. However, if those permanents entered in the opposite order, your opponent's maximum hand size would be four."
    );
    ruling!(
        "Locust Miser",
        "If multiple effects modify a player's hand size, apply them in timestamp order. For example, if you put Cursed Rack (an artifact that sets a chosen player's maximum hand size to four) onto the battlefield choosing your opponent and then put Locust Miser onto the battlefield, your opponent will have a maximum hand size of two. However, if those permanents entered in the opposite order, your opponent's maximum hand size would be four."
    );
    // (card, its controller, Cursed Rack's controller, the affected player, after, before)
    let cases: [(&str, PlayerId, PlayerId, PlayerId, i32, i32); 5] = [
        ("Thought Devourer", P0, P1, P0, 0, 4),
        ("Thought Eater", P0, P1, P0, 1, 4),
        ("Thought Nibbler", P0, P1, P0, 2, 4),
        ("Gnat Miser", P0, P0, P1, 3, 4),
        ("Locust Miser", P0, P0, P1, 2, 4),
    ];
    for (name, owner, rack, who, rack_first, card_first) in cases {
        assert_eq!(
            max_hand_after(&[(rack, "Cursed Rack"), (owner, name)], who),
            Some(rack_first),
            "Cursed Rack, then {name}"
        );
        assert_eq!(
            max_hand_after(&[(owner, name), (rack, "Cursed Rack")], who),
            Some(card_first),
            "{name}, then Cursed Rack"
        );
    }
    // And in the cleanup step: Cursed Rack then Thought Devourer discards everything.
    assert_eq!(
        kept_through_cleanup(&[(P1, "Cursed Rack"), (P0, "Thought Devourer")], P0),
        0
    );
}

#[test]
fn no_maximum_hand_size_and_a_set_size_apply_in_timestamp_order() {
    cr!("402.2", "514.1", "613.11", "613.7", "613.7d");
    ruling!(
        "Graceful Adept",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then put Graceful Adept onto the battlefield, you'll have no maximum hand size. However, if those permanents entered in the opposite order, your maximum hand size would be two."
    );
    ruling!(
        "Reliquary Tower",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then put Reliquary Tower onto the battlefield, you'll have no maximum hand size. However, if those permanents enter in the opposite order, your maximum hand size would be two."
    );
    ruling!(
        "Venser's Journal",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then put Venser's Journal onto the battlefield, you'll have no maximum hand size. However, if those permanents enter in the opposite order, your maximum hand size would be two."
    );
    ruling!(
        "Doctor Octopus, Master Planner",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Spellbook (an artifact that says you have no maximum hand size) onto the battlefield and then put Doctor Octopus, Master Planner onto the battlefield, your maximum hand size will be eight. However, if those permanents entered in the opposite order, you would have no maximum hand size."
    );
    for name in ["Graceful Adept", "Reliquary Tower", "Venser's Journal"] {
        assert_eq!(
            max_hand_after(&[(P0, "Null Profusion"), (P0, name)], P0),
            None,
            "Null Profusion, then {name}"
        );
        assert_eq!(
            max_hand_after(&[(P0, name), (P0, "Null Profusion")], P0),
            Some(2),
            "{name}, then Null Profusion"
        );
    }
    let doc = "Doctor Octopus, Master Planner";
    assert_eq!(max_hand_after(&[(P0, "Spellbook"), (P0, doc)], P0), Some(8));
    assert_eq!(max_hand_after(&[(P0, doc), (P0, "Spellbook")], P0), None);
    assert_eq!(
        kept_through_cleanup(&[(P0, "Null Profusion"), (P0, "Reliquary Tower")], P0),
        10
    );
    assert_eq!(
        kept_through_cleanup(&[(P0, "Reliquary Tower"), (P0, "Null Profusion")], P0),
        2
    );
}

// ---------------------------------------------------------------------------------
// "Draw cards equal to the difference" (Damia, Doctor Octopus, Krang)
// ---------------------------------------------------------------------------------

#[test]
fn damia_checks_the_hand_as_the_upkeep_begins_and_can_be_responded_to() {
    cr!("603.4", "503.1a", "117.3a", "117.3b");
    ruling!(
        "Damia, Sage of Stone",
        "The number of cards in your hand is checked at the beginning of your upkeep before you have the opportunity to cast spells and activate abilities. If you have seven or more cards in your hand at that time, Damia’s last ability won’t trigger."
    );
    ruling!(
        "Damia, Sage of Stone",
        "After the ability has triggered but before it resolves, you have the opportunity to cast spells and activate abilities."
    );
    supported("Damia, Sage of Stone");
    // Seven cards: no trigger, even with an instant that could be cast in the upkeep.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Damia, Sage of Stone");
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Lightning Bolt");
    fill_hand(&mut t, P0, 6);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Six cards: it triggers, and P0 can cast Lightning Bolt in response; then draws
    // two (from five to seven).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Damia, Sage of Stone");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    fill_hand(&mut t, P0, 5);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 1, "Damia's ability is still waiting");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn draw_the_difference_with_thought_reflection_draws_twice_as_many() {
    cr!("121.6", "614.11");
    ruling!(
        "Damia, Sage of Stone",
        "Effects can modify the number of cards you’ll draw. For example, if you have four cards in your hand and control Thought Reflection (“If you would draw a card, draw two cards instead.”), you’ll draw six cards."
    );
    ruling!(
        "Doctor Octopus, Master Planner",
        "The number of cards you ultimately draw may be affected by replacement effects. For example, if you have six cards in hand as Doctor Octopus's last ability resolves, and you also control Thought Reflection (an enchantment that says \"If you would draw a card, draw two cards instead\"), you'll end up drawing four cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Damia, Sage of Stone");
    t.battlefield(P0, "Thought Reflection");
    fill_hand(&mut t, P0, 4);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 10);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doctor Octopus, Master Planner");
    t.battlefield(P0, "Thought Reflection");
    fill_hand(&mut t, P0, 6);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 10);
}

#[test]
fn krang_checks_the_hand_as_it_enters_and_again_as_it_resolves() {
    cr!("603.4");
    ruling!(
        "Krang, Master Mind",
        "If you have four or more cards in hand when Krang enters, his second ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you have four or more card in hand at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Krang, Master Mind");
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, 4);
    t.enter(P0, "Krang, Master Mind");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, 1);
    t.enter(P0, "Krang, Master Mind");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    // Unchanged: it draws the difference.
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, 1);
    t.enter(P0, "Krang, Master Mind");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

// ---------------------------------------------------------------------------------
// "-1/-1 for each card in your hand" (Grim Strider, Stingerback Terror)
// ---------------------------------------------------------------------------------

#[test]
fn damage_from_combat_can_become_lethal_after_drawing() {
    cr!("704.5g", "120.6", "510.2");
    ruling!(
        "Grim Strider",
        "Because damage remains marked on a creature until it’s removed as the turn ends, the damage a Grim Strider takes during combat may become lethal if you draw cards later in the turn."
    );
    ruling!(
        "Stingerback Terror",
        "Because damage remains marked on a creature until it's removed as the turn ends, the damage Stingerback Terror takes during combat may become lethal if you draw cards later in the turn."
    );
    supported("Grim Strider");
    supported("Stingerback Terror");
    // P0's creature, a 4/4 with the cards in hand, blocks P1's Hill Giant (3/3) and takes
    // 3 damage. After combat, P0 draws a card: 3 toughness, so the 3 damage is lethal.
    for (name, cards) in [("Grim Strider", 2), ("Stingerback Terror", 3)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        fill_hand(&mut t, P0, cards);
        assert_eq!(t.pt(c), (4, 4), "{name}");
        t.set_step(P1, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        t.attack(&[(giant, pl(P0))], &[(c, giant)]);
        assert!(t.on_battlefield(c), "{name} survives the combat damage");
        assert_eq!(t.obj_now(c).damage, 3, "{name}");
        assert!(t.in_graveyard(P1, "Hill Giant"));
        t.advance_to(P1, Step::PostcombatMain);
        assert!(
            t.on_battlefield(c),
            "{name}: the damage is still marked, not lethal"
        );
        t.g.draw_cards(P0, 1);
        t.settle();
        assert!(
            !t.on_battlefield(c),
            "{name}: the damage marked in combat is now lethal"
        );
    }
}

#[test]
fn drawing_then_putting_back_doesnt_kill_it_midway() {
    cr!("704.3", "608.2", "117.5");
    ruling!(
        "Grim Strider",
        "If a spell or ability causes you to draw cards and then has you remove some from hand (such as by discarding them or putting them on top of your library), Grim Strider may briefly have 0 toughness or less. As long as its toughness is 1 or greater again after the spell or ability is finished resolving, it will remain on the battlefield."
    );
    ruling!(
        "Stingerback Terror",
        "If a spell or ability causes you to draw cards and then has you remove some from hand (such as by discarding them or putting them on top of your library), Stingerback Terror may briefly have 0 toughness or less. As long as its toughness is 1 or greater again after the spell or ability is finished resolving, it will remain on the battlefield."
    );
    // Brainstorm: draw three, put two back. Grim Strider (6/6) with three other cards:
    // six cards (0 toughness) midway, four (2 toughness) after. Stingerback Terror (7/7)
    // with four: seven midway, five after.
    for (name, others) in [("Grim Strider", 3), ("Stingerback Terror", 4)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        fill_hand(&mut t, P0, others);
        let seen = crate::r_s01_common::watch(
            &mut t,
            P0,
            |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
            |g| g.player(PlayerId(0)).hand.len(),
        );
        cast_new(&mut t, P0, "Brainstorm", &[]);
        t.resolve_all();
        assert!(
            seen_values(&seen).iter().any(|n| *n >= others + 3),
            "{name}"
        );
        assert!(t.on_battlefield(c), "{name} survives");
        assert_eq!(t.pt(c).1, 2, "{name}");
    }
}

// ---------------------------------------------------------------------------------
// Descendant of Kiyomaro: "As long as you have more cards in hand than each opponent,
// this creature gets +1/+2 and has 'Whenever this creature deals combat damage, you gain
// 3 life.'"
// ---------------------------------------------------------------------------------

#[test]
fn descendant_of_kiyomaro_needs_more_cards_than_every_opponent() {
    cr!("611.3a", "604.2");
    ruling!(
        "Descendant of Kiyomaro",
        "If any opponent has more cards or as many cards in hand as Descendant of Kiyomaro’s controller, Descendant of Kiyomaro doesn’t get any bonus."
    );
    supported("Descendant of Kiyomaro");
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Descendant of Kiyomaro");
    fill_hand(&mut t, P0, 3);
    fill_hand(&mut t, P1, 2);
    fill_hand(&mut t, P2, 3);
    assert_eq!(t.pt(d), (2, 3), "P2 has as many");
    t.hand(P2, "Grizzly Bears");
    assert_eq!(t.pt(d), (2, 3), "P2 has more");
    fill_hand(&mut t, P0, 2);
    assert_eq!(t.pt(d), (3, 5), "more than each opponent");
}

#[test]
fn descendant_of_kiyomaros_trigger_resolves_after_it_loses_the_ability() {
    cr!("113.7a", "611.3a");
    ruling!(
        "Descendant of Kiyomaro",
        "Changing the number of cards in any player’s hand can cause Descendant of Kiyomaro to lose the life-gain triggered ability. Once an ability triggers, however, it exists independently of its source."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Descendant of Kiyomaro");
    fill_hand(&mut t, P0, 2);
    attack_with(&mut t, &[(d, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 1);
    // Before it resolves, P1 gets more cards: Descendant loses the ability.
    fill_hand(&mut t, P1, 3);
    assert_eq!(t.pt(d), (2, 3));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Without the bonus, combat damage doesn't trigger.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Descendant of Kiyomaro");
    t.attack(&[(d, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------
// Counting on resolution
// ---------------------------------------------------------------------------------

#[test]
fn upkeep_triggers_count_the_hands_as_they_resolve() {
    cr!("608.2h", "603.2");
    ruling!("Bulwark", "Counts cards during resolution.");
    ruling!(
        "Price of Knowledge",
        "Count the number of cards in that opponent’s hand when the ability resolves to determine how much damage is dealt."
    );
    ruling!(
        "Skullcage",
        "Skullcage doesn’t check hand size when it triggers, only when it resolves."
    );
    ruling!(
        "Wolfcaller's Howl",
        "Use the number of opponents who have four or more cards in hand as the ability resolves to determine the value of X."
    );
    for n in [
        "Bulwark",
        "Price of Knowledge",
        "Skullcage",
        "Wolfcaller's Howl",
    ] {
        supported(n);
    }
    // Bulwark: five cards against one, then three in response: 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bulwark");
    fill_hand(&mut t, P0, 5);
    fill_hand(&mut t, P1, 1);
    to_upkeep(&mut t, P0);
    t.answer_targets(P0, &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Price of Knowledge: P1's hand grows from 1 to 4 before it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Price of Knowledge");
    fill_hand(&mut t, P1, 1);
    to_upkeep(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P1, 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Skullcage: three cards as it triggers (it still triggers), five as it resolves: 2
    // damage; and five as it triggers, three as it resolves: none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skullcage");
    fill_hand(&mut t, P1, 3);
    to_upkeep(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skullcage");
    fill_hand(&mut t, P1, 5);
    to_upkeep(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.g.player(P1).hand.clone();
    for c in hand.into_iter().take(2) {
        crate::r_s05_common::move_to(&mut t, c, mtg_engine::object::Zone::Graveyard(P1));
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Wolfcaller's Howl: P1 has four cards, P2 three; P2 gets a fourth in response.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Wolfcaller's Howl");
    fill_hand(&mut t, P1, 4);
    fill_hand(&mut t, P2, 3);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P2, 1);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
}

#[test]
fn ivory_tower_does_nothing_with_four_or_fewer_cards() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Ivory Tower",
        "If you have four or fewer cards in your hand when Ivory Tower's ability resolves, the ability has no effect."
    );
    supported("Ivory Tower");
    for (cards, life) in [(0, 20), (4, 20), (6, 22)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ivory Tower");
        fill_hand(&mut t, P0, cards);
        to_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.life(P0), life, "{cards} cards");
    }
}

#[test]
fn triskaidekaphile_needs_exactly_thirteen_as_the_upkeep_begins() {
    cr!("603.4", "104.2b");
    ruling!(
        "Triskaidekaphile",
        "Triskaidekaphile's triggered ability will trigger only if you have exactly thirteen cards in your hand as your upkeep starts. If you have fewer cards in your hand, you won't be able to draw cards during your upkeep in time to cause the ability to trigger."
    );
    supported("Triskaidekaphile");
    for (cards, wins) in [(12, false), (13, true), (14, false)] {
        let mut t = TestGame::new(2);
        let tris = t.battlefield(P0, "Triskaidekaphile");
        fill_hand(&mut t, P0, cards);
        t.lands(P0, "Island", 4);
        to_upkeep(&mut t, P0);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(wins), "{cards} cards");
        if !wins {
            // Drawing to thirteen during the upkeep is too late.
            t.activate(P0, tris, 0, &[]).unwrap();
            t.resolve_all();
            t.settle();
            assert_eq!(t.g.result, None, "{cards} cards");
        } else {
            t.resolve_all();
            assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
        }
    }
}

#[test]
fn vicious_shadows_counts_the_targets_hand_on_resolution() {
    cr!("608.2h", "115.1");
    ruling!(
        "Vicious Shadows",
        "The number of cards in the targeted opponent’s hand is checked only when the ability resolves."
    );
    ruling!(
        "Vicious Shadows",
        "The player you target doesn’t have to be the controller of the creature that was put into a graveyard."
    );
    supported("Vicious Shadows");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Vicious Shadows");
    let bears = t.battlefield(P2, "Grizzly Bears");
    fill_hand(&mut t, P1, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    fill_hand(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 20);
}

// ---------------------------------------------------------------------------------
// Values locked in on resolution
// ---------------------------------------------------------------------------------

#[test]
fn hand_counts_are_locked_in_as_the_spell_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Warped Physique",
        "Once Warped Physique resolves, the effect won’t change if the number of cards in your hand changes."
    );
    ruling!(
        "Warped Physique",
        "Use the number of cards in your hand as Warped Physique resolves to determine the value of X. Warped Physique won’t be in your hand at that time."
    );
    ruling!(
        "Nightmarish End",
        "The value of X is calculated as the ability resolves. The effect won't change later in the turn, even if the number of cards in your hand does."
    );
    ruling!("Flunk", "The value of X is determined as Flunk resolves.");
    for n in ["Warped Physique", "Nightmarish End", "Flunk"] {
        supported(n);
    }
    // Warped Physique: two other cards in hand; a third arrives before it resolves.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_hand(&mut t, P0, 2);
    cast_new(&mut t, P0, "Warped Physique", &[obj(wurm)]);
    fill_hand(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (9, 1));
    fill_hand(&mut t, P0, 2);
    assert_eq!(t.pt(wurm), (9, 1));
    // Nightmarish End: two cards in hand as it resolves.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_hand(&mut t, P0, 1);
    cast_new(&mut t, P0, "Nightmarish End", &[obj(wurm)]);
    fill_hand(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    fill_hand(&mut t, P0, 1);
    assert_eq!(t.pt(wurm), (4, 2));
    // Flunk: P1 has three cards as it's cast, five as it resolves: -2/-2.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_hand(&mut t, P1, 3);
    cast_new(&mut t, P0, "Flunk", &[obj(wurm)]);
    fill_hand(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    fill_hand(&mut t, P1, 2);
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn flunk_never_gives_a_bonus() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Flunk",
        "If the target creature’s controller has seven or more cards in hand when Flunk resolves, the creature gets -0/-0. This effect will never increase the creature’s power and toughness."
    );
    for cards in [7, 9] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        fill_hand(&mut t, P1, cards);
        cast_new(&mut t, P0, "Flunk", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (2, 2), "{cards} cards");
    }
}

#[test]
fn master_the_way_counts_after_drawing_and_does_nothing_without_its_target() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Master the Way",
        "Use the number of cards in your hand after you draw the card to determine how much damage Master the Way deals to the target permanent or player. (Remember that Master the Way is not in your hand; it’s on the stack.)"
    );
    ruling!(
        "Master the Way",
        "If the permanent or player is an illegal target as Master the Way tries to resolve, Master the Way won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    supported("Master the Way");
    let mut t = TestGame::new(2);
    fill_hand(&mut t, P0, 2);
    cast_new(&mut t, P0, "Master the Way", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P1), 17);
    // The target is gone: no draw.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_hand(&mut t, P0, 2);
    cast_new(&mut t, P0, "Master the Way", &[obj(wurm)]);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P0, "Master the Way"));
}

#[test]
fn rals_staticaster_checks_ral_twice_and_locks_in_its_bonus() {
    cr!("603.4", "608.2h", "611.2c");
    ruling!(
        "Ral's Staticaster",
        "The number of cards in your hand is checked only as the triggered ability of Ral’s Staticaster resolves. If that number changes later in the turn, its power is unaffected."
    );
    ruling!(
        "Ral's Staticaster",
        "Whether you control a Ral planeswalker is checked as the ability triggers and as it resolves. After it has resolved while you controlled a Ral planeswalker, Ral’s Staticaster keeps the bonus for the rest of the turn even if you lose your planeswalker."
    );
    supported("Ral's Staticaster");
    supported("Ral, Izzet Viceroy");
    // No Ral as it attacks: no trigger.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Ral's Staticaster");
    fill_hand(&mut t, P0, 2);
    attack_with(&mut t, &[(s, pl(P1))]);
    assert_eq!(t.stack_len(), 0);
    // Ral leaves before it resolves: no bonus.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Ral's Staticaster");
    let ral = t.battlefield(P0, "Ral, Izzet Viceroy");
    fill_hand(&mut t, P0, 2);
    attack_with(&mut t, &[(s, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ral);
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 3));
    // Ral stays: +1/+0 per card as it resolves (three), kept after the hand changes and
    // after Ral leaves.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Ral's Staticaster");
    let ral = t.battlefield(P0, "Ral, Izzet Viceroy");
    fill_hand(&mut t, P0, 2);
    attack_with(&mut t, &[(s, pl(P1))]);
    fill_hand(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(s), (6, 3));
    fill_hand(&mut t, P0, 2);
    destroy(&mut t, ral);
    assert_eq!(t.pt(s), (6, 3));
}
