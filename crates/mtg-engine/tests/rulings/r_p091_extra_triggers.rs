//! Rulings batch P091 — "that ability triggers an additional time" for lands (Ancient
//! Greenwarden) and lands or Birds (Traveling Chocobo).

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn greenwarden_land_entering_triggers_twice_and_more_with_more_greenwardens() {
    cr!("603.2d", "603.2");
    ruling!(
        "Ancient Greenwarden",
        "If you control two Ancient Greenwardens, a land entering the battlefield causes abilities to trigger three times, not four."
    );
    ruling!(
        "Ancient Greenwarden",
        "Ancient Greenwarden's effect doesn't copy the triggered ability; it causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    ruling!(
        "Traveling Chocobo",
        "If you control two Traveling Chocobos, a land or Bird you control entering causes abilities to trigger three times, not four."
    );
    ruling!(
        "Traveling Chocobo",
        "Traveling Chocobo's ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    supported("Ancient Greenwarden");
    supported("Sporemound");
    supported("Grappling Kraken");
    for doubler in ["Ancient Greenwarden", "Traveling Chocobo"] {
        // One doubler: twice. Two: three times, not four.
        for (n, expected) in [(0usize, 1usize), (1, 2), (2, 3)] {
            let mut t = TestGame::new(2);
            for _ in 0..n {
                t.battlefield(P0, doubler);
            }
            let mound = t.battlefield(P0, "Sporemound");
            play_from_hand(&mut t, P0, "Forest");
            assert_eq!(triggers_of(&t, mound), expected, "{doubler} x{n}");
            t.resolve_all();
            assert_eq!(tokens(&t, P0).len(), expected, "{doubler} x{n}");
        }
        // Targets are chosen separately for each instance: two different creatures.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let kraken = t.battlefield(P0, "Grappling Kraken");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[Entity::Object(a)]);
        t.answer_targets(P0, &[Entity::Object(b)]);
        play_from_hand(&mut t, P0, "Island");
        assert_eq!(triggers_of(&t, kraken), 2, "{doubler}");
        t.resolve_all();
        for c in [a, b] {
            assert!(t.obj_now(c).tapped, "{doubler}");
            assert_eq!(t.counters(c, "stun"), 1, "{doubler}");
        }
    }
}

#[test]
fn greenwarden_resolution_choices_are_made_for_each_instance() {
    cr!("603.2d", "608.2d");
    ruling!(
        "Ancient Greenwarden",
        "Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    ruling!(
        "Traveling Chocobo",
        "Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Seer's Sundial");
    for doubler in ["Ancient Greenwarden", "Traveling Chocobo"] {
        // Seer's Sundial: "you may pay {2}. If you do, draw a card." Pay once, decline once.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let dial = t.battlefield(P0, "Seer's Sundial");
        t.lands(P0, "Wastes", 2);
        play_from_hand(&mut t, P0, "Forest");
        assert_eq!(triggers_of(&t, dial), 2, "{doubler}");
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 1, "{doubler}");
        t.answer_yes(P0, false);
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 1, "{doubler}");
        assert_eq!(t.stack_len(), 0);
    }
}

#[test]
fn greenwarden_affects_the_lands_own_etb_and_triggers_that_dont_mention_lands() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Ancient Greenwarden",
        "Ancient Greenwarden's third ability affects a land's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that land enters the battlefield"
    );
    ruling!(
        "Ancient Greenwarden",
        "The trigger event doesn't have to specifically refer to \"lands.\" For example, an ability that triggers \"whenever a creature enters the battlefield under your control\" would trigger twice if the entering land is also a creature."
    );
    ruling!(
        "Traveling Chocobo",
        "Traveling Chocobo's last ability affects a permanent's own \"enters\" triggered abilities (as long as it's a land or Bird you control) as well as other triggered abilities that trigger when a land or Bird you control enters the battlefield."
    );
    ruling!(
        "Sporemound",
        "Sporemound's ability triggers whenever you play a land and also whenever a land enters the battlefield under your control for some other reason"
    );
    supported("Glimmerpost");
    supported("Soul Warden");
    for doubler in ["Ancient Greenwarden", "Traveling Chocobo"] {
        // Glimmerpost's own ETB trigger triggers twice.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let post = play_from_hand(&mut t, P0, "Glimmerpost");
        assert_eq!(triggers_of(&t, post), 2, "{doubler}");
        t.resolve_all();
        assert_eq!(t.life(P0), 22, "{doubler}");
        // Dryad Arbor (a land creature) put onto the battlefield: Soul Warden's "another
        // creature enters" triggers twice; Sporemound's landfall too (not played).
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let warden = t.battlefield(P0, "Soul Warden");
        let mound = t.battlefield(P0, "Sporemound");
        put_onto_battlefield(&mut t, P0, "Dryad Arbor");
        assert_eq!(triggers_of(&t, warden), 2, "{doubler}");
        assert_eq!(triggers_of(&t, mound), 2, "{doubler}");
    }
    // Without a doubler, Sporemound triggers once for a land put onto the battlefield.
    let mut t = TestGame::new(2);
    let mound = t.battlefield(P0, "Sporemound");
    put_onto_battlefield(&mut t, P0, "Forest");
    assert_eq!(triggers_of(&t, mound), 1);
}

#[test]
fn greenwarden_entering_with_a_land_counts_and_chocobo_counts_itself() {
    cr!("603.2d", "603.6a");
    ruling!(
        "Ancient Greenwarden",
        "If a land entering the battlefield at the same time as Ancient Greenwarden (including Ancient Greenwarden itself if an effect causes it to be a land) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    ruling!(
        "Traveling Chocobo",
        "If a land or Bird you control entering at the same time as Traveling Chocobo (including Traveling Chocobo itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    // Traveling Chocobo itself (a Bird) entering: Soul Warden triggers twice.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    put_onto_battlefield(&mut t, P0, "Traveling Chocobo");
    assert_eq!(triggers_of(&t, warden), 2);
    // A non-Bird creature: once.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    put_onto_battlefield(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_of(&t, warden), 1);
    // Ancient Greenwarden and a Forest entering together: Sporemound triggers twice.
    let mut t = TestGame::new(2);
    let mound = t.battlefield(P0, "Sporemound");
    let gw = t.hand(P0, "Ancient Greenwarden");
    let forest = t.hand(P0, "Forest");
    crate::r_s31_common::put_together(&mut t, &[gw, forest]);
    assert_eq!(triggers_of(&t, mound), 2);
}

#[test]
fn greenwarden_looks_at_permanents_as_they_exist() {
    cr!("603.2d", "613.1");
    ruling!(
        "Ancient Greenwarden",
        "Look at each permanent as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times. For example, if you control Ashaya, Soul of the Wild, a nontoken creature entering the battlefield will cause any appropriate abilities to trigger an additional time."
    );
    supported("Ashaya, Soul of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Greenwarden");
    t.battlefield(P0, "Ashaya, Soul of the Wild");
    let warden = t.battlefield(P0, "Soul Warden");
    put_onto_battlefield(&mut t, P0, "Grizzly Bears");
    assert_eq!(triggers_of(&t, warden), 2);
    // A creature token isn't a land under Ashaya: once.
    t.resolve_all();
    let before = t.life(P0);
    crate::r_s02_common::create_token(&mut t, P0, "Beast");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_of(&t, warden), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), before + 1);
}

#[test]
fn doublers_dont_affect_replacement_effects_or_play_triggers() {
    cr!("603.2d", "614.1c", "614.12");
    ruling!(
        "Ancient Greenwarden",
        "Replacement effects are unaffected by Ancient Greenwarden's third ability. This includes any abilities that apply \"as [this land] enters the battlefield\" and any ability that says the land enters the battlefield with counters."
    );
    ruling!(
        "Traveling Chocobo",
        "Replacement effects are unaffected by Traveling Chocobo's ability."
    );
    ruling!(
        "Traveling Chocobo",
        "Abilities that apply \"as [this permanent] enters,\" such as choosing a color with Crossroads Village, are also unaffected."
    );
    ruling!(
        "Ancient Greenwarden",
        "An ability that triggers whenever you play a land won't trigger an additional time."
    );
    supported("Gemstone Mine");
    supported("Crossroads Village");
    supported("Null Profusion");
    for doubler in ["Ancient Greenwarden", "Traveling Chocobo"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let mine = play_from_hand(&mut t, P0, "Gemstone Mine");
        assert_eq!(t.counters(mine, "mining"), 3, "{doubler}");
        // Crossroads Village: one color choice.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let from = t.asked().len();
        let village = put_onto_battlefield(&mut t, P0, "Crossroads Village");
        let colors = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| format!("{d:?}").contains("Choose a color"))
            .count();
        assert_eq!(colors, 1, "{doubler}");
        assert!(t.obj_now(village).choices.color.is_some(), "{doubler}");
    }
    // Null Profusion: "Whenever you play a card" — playing a land triggers it once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Greenwarden");
    let np = t.battlefield(P0, "Null Profusion");
    play_from_hand(&mut t, P0, "Forest");
    assert_eq!(triggers_of(&t, np), 1);
}

#[test]
fn chocobo_doesnt_affect_a_teammates_permanents() {
    cr!("603.2d", "810.2");
    ruling!(
        "Traveling Chocobo",
        "In a Two-Headed Giant game, Traveling Chocobo's second ability does not affect triggered abilities of permanents your teammate controls."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Traveling Chocobo");
    let mine = t.battlefield(P0, "Soul Warden");
    let mate = t.battlefield(P2, "Soul Warden");
    assert!(t.g.teammates(P0).contains(&P2));
    put_onto_battlefield(&mut t, P0, "Birds of Paradise");
    assert_eq!(triggers_of(&t, mine), 2);
    assert_eq!(triggers_of(&t, mate), 1);
}
