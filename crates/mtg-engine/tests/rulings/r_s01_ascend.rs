//! Rulings batch S01 — ascend (CR 702.131): with ten or more permanents, you get the
//! city's blessing for the rest of the game.

use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn blessed(t: &TestGame, p: PlayerId) -> bool {
    t.g.player(p).has_citys_blessing
}

fn permanents(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents().filter(|o| o.controller == p).count()
}

#[test]
fn tokens_and_lands_are_permanents_spells_and_emblems_arent() {
    cr!("702.131b", "110.1", "114.1");
    ruling!(
        "Wayward Swordtooth",
        "A permanent is any object on the battlefield, including tokens and lands. Spells and emblems aren't permanents."
    );
    supported("Wayward Swordtooth");
    let mut t = TestGame::new(2);
    // Wayward Swordtooth, six lands and two Soldier tokens: nine permanents.
    t.battlefield(P0, "Wayward Swordtooth");
    t.lands(P0, "Plains", 4);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    t.lands(P0, "Forest", 2);
    // An emblem in the command zone, and a creature spell on the stack.
    mtg_engine::tokens::create_emblem(&mut t.g, P0, vec![], None);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(permanents(&t, P0), 9);
    assert!(!blessed(&t, P0));
    // The creature spell resolves: ten permanents.
    t.resolve_all();
    assert!(blessed(&t, P0));
}

#[test]
fn a_spell_with_ascend_gives_no_blessing_until_it_resolves() {
    cr!("702.131b", "608.3");
    ruling!(
        "Wayward Swordtooth",
        "If you cast a spell with ascend, you don't get the city's blessing until it resolves. Players may respond to that spell by trying to change whether you get the city's blessing."
    );
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 8);
        let sword = t.hand(P0, "Wayward Swordtooth");
        t.cast(P0, sword).go();
        t.settle();
        // Nine permanents and a spell with ascend on the stack.
        assert!(!blessed(&t, P0));
        if respond {
            t.lands(P1, "Mountain", 1);
            let bolt = t.hand(P1, "Lightning Bolt");
            t.cast(P1, bolt).target(bears).go();
        }
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Wayward Swordtooth").len(), 1);
        assert_eq!(blessed(&t, P0), !respond);
    }
}

#[test]
fn the_blessing_from_a_played_tenth_land_comes_before_anyone_can_respond() {
    cr!("702.131b", "305.1", "117.1c");
    ruling!(
        "Wayward Swordtooth",
        "Ascend on a permanent isn't a triggered ability and doesn't use the stack. Players can respond to a spell that will give you your tenth permanent, but they can't respond to getting the city's blessing once you control that tenth permanent."
    );
    ruling!(
        "Slippery Scoundrel",
        "Ascend on a permanent isn’t a triggered ability and doesn’t use the stack. Players can respond to a spell that will give you your tenth permanent, but they can’t respond to getting the city’s blessing once you control that tenth permanent."
    );
    ruling!(
        "Illustrious Wanderglyph",
        "Ascend on a permanent isn't a triggered ability and doesn't use the stack. Players can respond to a spell that will give you your tenth permanent, but they can't respond to you getting the city's blessing once you control that tenth permanent."
    );
    ruling!(
        "Detective of the Month",
        "Ascend on a permanent isn’t a triggered ability and doesn’t use the stack. Players can respond to a spell that will give you your tenth permanent, but they can’t respond to you getting the city’s blessing once you control that tenth permanent."
    );
    for name in [
        "Wayward Swordtooth",
        "Slippery Scoundrel",
        "Illustrious Wanderglyph",
        "Detective of the Month",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.lands(P0, "Island", 8);
        let land = t.hand(P0, "Island");
        // At each priority: (blessed, objects on the stack).
        let seen = watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::Priority { .. }),
            |g| (g.player(PlayerId(0)).has_citys_blessing, g.stack.len()),
        );
        let seen1 = watch(
            &mut t,
            P1,
            |d| matches!(d, Decision::Priority { .. }),
            |g| g.player(PlayerId(0)).has_citys_blessing,
        );
        t.answer(
            P0,
            DecisionKind::Priority,
            Answer::Action(Action::PlayLand { card: land }),
        );
        let ok = t.g.run_until(100, |g| g.turn.priority == Some(PlayerId(1)));
        assert!(ok);
        let seen = seen.lock().unwrap().clone();
        // Before playing the land; then right after it, with nothing on the stack.
        assert_eq!(seen[0], (false, 0), "{name}");
        assert!(seen.len() > 1, "{name}");
        assert!(seen[1..].iter().all(|s| *s == (true, 0)), "{name}: {seen:?}");
        assert!(seen1.lock().unwrap().iter().all(|b| *b), "{name}");
    }
}

#[test]
fn the_blessing_stays_for_the_rest_of_the_game() {
    cr!("702.131a", "702.131b");
    ruling!(
        "Secrets of the Golden City",
        "Once you have the city’s blessing, you have it for the rest of the game, even if you lose control of some or all of your permanents. The city’s blessing isn’t a permanent itself and can’t be removed by any effect."
    );
    ruling!(
        "Andúril, Narsil Reforged",
        "Once you have the city's blessing, you have it for the rest of the game, even if you lose control of some or all your permanents."
    );
    supported("Secrets of the Golden City");
    supported("Andúril, Narsil Reforged");
    // "Draw two cards. If you have the city's blessing, draw three cards instead."
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Island", 10);
    let s = t.hand(P0, "Secrets of the Golden City");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(blessed(&t, P0));
    assert_eq!(t.hand_size(P0), 3);
    // All of those permanents leave; a player gaining control of the rest doesn't matter.
    for l in &lands[3..] {
        t.g.move_object(*l, Zone::Exile, MoveCause::Effect, None);
    }
    t.settle();
    assert!(blessed(&t, P0));
    t.lands(P0, "Island", 3);
    let s = t.hand(P0, "Secrets of the Golden City");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 6);
    // A permanent with ascend: Andúril.
    let mut t = TestGame::new(2);
    let anduril = t.battlefield(P0, "Andúril, Narsil Reforged");
    let lands = t.lands(P0, "Plains", 9);
    t.settle();
    assert!(blessed(&t, P0));
    t.g.move_object(anduril, Zone::Exile, MoveCause::Effect, None);
    for l in lands {
        t.g.move_object(l, Zone::Graveyard(P0), MoveCause::Effect, None);
    }
    t.settle();
    assert_eq!(permanents(&t, P0), 0);
    assert!(blessed(&t, P0));
}

#[test]
fn a_tenth_permanent_that_leaves_at_once_still_gives_the_blessing() {
    cr!("702.131b", "704.5j");
    ruling!(
        "Slippery Scoundrel",
        "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the “Legend Rule” or due to being a creature with 0 toughness), you get the city’s blessing before it leaves the battlefield."
    );
    ruling!(
        "Illustrious Wanderglyph",
        "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the \"legend rule\" or due to being a creature with 0 toughness), you get the city's blessing before it leaves the battlefield."
    );
    ruling!(
        "Detective of the Month",
        "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the “legend rule” or due to being a creature with 0 toughness), you get the city’s blessing before it leaves the battlefield."
    );
    for name in [
        "Slippery Scoundrel",
        "Illustrious Wanderglyph",
        "Detective of the Month",
    ] {
        // Nine permanents; the tenth is a second Isamaru.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.lands(P0, "Plains", 7);
        t.settle();
        assert!(!blessed(&t, P0), "{name}");
        let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
        t.cast(P0, isamaru).go();
        t.resolve_all();
        // The legend rule put one of them into the graveyard, leaving nine permanents.
        assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
        assert_eq!(permanents(&t, P0), 9);
        assert!(blessed(&t, P0), "{name}");
    }
}

#[test]
fn an_intervening_if_blessing_trigger_needs_the_blessing_already() {
    cr!("603.4", "702.131b");
    ruling!(
        "Resplendent Griffin",
        "You must already have the city’s blessing in order for these abilities to trigger; otherwise they do nothing. In other words, there’s no way to have the ability trigger if you don’t have the city’s blessing, even if you intend to get it in response to the triggered ability."
    );
    supported("Resplendent Griffin");
    let mut t = TestGame::new(2);
    // "Whenever this creature attacks, if you have the city's blessing, put a +1/+1
    // counter on it." Nine permanents.
    let griffin = t.battlefield(P0, "Resplendent Griffin");
    t.lands(P0, "Plains", 8);
    attack_with(&mut t, &[(griffin, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "if you have the city's blessing"), 0);
    // The tenth permanent arrives before blockers: the blessing comes too late.
    t.lands(P0, "Plains", 1);
    t.settle();
    assert!(blessed(&t, P0));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.counters(griffin, "+1/+1"), 0);
    assert_eq!(t.life(P1), 18);
    // Attacking with the blessing, it triggers.
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(griffin, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "if you have the city's blessing"), 1);
    t.resolve_all();
    assert_eq!(t.counters(griffin, "+1/+1"), 1);
}

#[test]
fn a_blessing_check_without_intervening_if_happens_on_resolution() {
    cr!("603.4", "608.2c");
    ruling!(
        "Kumena's Awakening",
        "Some cards have triggered abilities that check if you have the city’s blessing, but don’t use an intervening “if” clause. These abilities trigger regardless of whether you have the city’s blessing and check whether you do only as they resolve."
    );
    supported("Kumena's Awakening");
    for get_it in [false, true] {
        let mut t = TestGame::new(2);
        // "At the beginning of your upkeep, each player draws a card. If you have the
        // city's blessing, instead only you draw a card." Nine permanents.
        t.battlefield(P0, "Kumena's Awakening");
        t.lands(P0, "Island", 8);
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "each player draws a card"), 1);
        if get_it {
            // In response, the tenth permanent: the blessing counts on resolution.
            t.lands(P0, "Island", 1);
            t.settle();
            assert!(blessed(&t, P0));
        }
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        t.resolve_all();
        assert_eq!(t.hand_size(P0), h0 + 1);
        assert_eq!(t.hand_size(P1), h1 + usize::from(!get_it));
    }
}

#[test]
fn ten_permanents_without_ascend_give_no_blessing() {
    cr!("702.131a", "702.131b");
    ruling!(
        "Golden Demise",
        "If you control ten permanents but don’t control a permanent or resolving spell with ascend, you don’t get the city’s blessing. For example, if you control ten permanents, lose control of one, then cast Golden Demise, you won’t have the city’s blessing and the spell will affect creatures you control."
    );
    supported("Golden Demise");
    for lose_one in [false, true] {
        let mut t = TestGame::new(2);
        // "All creatures get -2/-2 until end of turn. If you have the city's blessing,
        // instead only creatures your opponents control get -2/-2 until end of turn."
        let mine = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 8);
        let theirs = t.battlefield(P1, "Hill Giant");
        t.settle();
        assert!(!blessed(&t, P0));
        if lose_one {
            t.lands(P1, "Mountain", 1);
            let bolt = t.hand(P1, "Lightning Bolt");
            t.cast(P1, bolt).target(bears).go();
            t.resolve_all();
        }
        let demise = t.hand(P0, "Golden Demise");
        t.cast(P0, demise).go();
        t.resolve_all();
        assert_eq!(blessed(&t, P0), !lose_one);
        assert_eq!(t.pt(theirs), (1, 1));
        assert_eq!(t.pt(mine), if lose_one { (1, 1) } else { (3, 3) });
    }
}
