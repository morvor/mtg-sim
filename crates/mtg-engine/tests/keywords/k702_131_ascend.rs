//! CR 702.131 Ascend.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn blessed(t: &TestGame, p: PlayerId) -> bool {
    t.g.player(p).has_citys_blessing
}

#[test]
fn ascend_on_a_sorcery_gives_the_blessing_as_it_resolves_before_its_other_effects() {
    cr!("702.131", "702.131a");
    assert_supported_card("Secrets of the Golden City");
    ruling!(
        "Secrets of the Golden City",
        "If you cast a spell with ascend, you don’t get the city’s blessing until it resolves."
    );
    let mut t = TestGame::new(2);
    // Secrets of the Golden City: {1}{U}{U} sorcery, ascend, "Draw two cards. If you have
    // the city's blessing, draw three cards instead."
    t.lands(P0, "Island", 3);
    battlefield_n(&mut t, P0, "Plains", 7);
    let s = t.hand(P0, "Secrets of the Golden City");
    t.cast(P0, s).go();
    assert!(!blessed(&t, P0));
    t.resolve_all();
    assert!(blessed(&t, P0));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn ascend_on_a_spell_does_nothing_with_fewer_than_ten_permanents() {
    cr!("702.131a");
    ruling!(
        "Secrets of the Golden City",
        "A permanent is any object on the battlefield, including tokens and lands. Spells and emblems aren’t permanents."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    battlefield_n(&mut t, P0, "Plains", 6);
    // Opponents' permanents don't count.
    battlefield_n(&mut t, P1, "Plains", 5);
    let s = t.hand(P0, "Secrets of the Golden City");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!blessed(&t, P0));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn ascend_on_a_permanent_gives_the_blessing_any_time_for_the_rest_of_the_game() {
    cr!("702.131b");
    assert_supported_card("Skymarcher Aspirant");
    ruling!(
        "Skymarcher Aspirant",
        "Once you have the city's blessing, you have it for the rest of the game, even if you lose control of some or all of your permanents."
    );
    let mut t = TestGame::new(2);
    // Skymarcher Aspirant: 2/1, ascend, "This creature has flying as long as you have the
    // city's blessing."
    let a = t.battlefield(P0, "Skymarcher Aspirant");
    let lands = battlefield_n(&mut t, P0, "Plains", 8);
    t.settle();
    assert!(!blessed(&t, P0));
    assert!(!has(&t, a, KeywordKind::Flying));
    let tenth = t.battlefield(P0, "Plains");
    t.settle();
    assert!(blessed(&t, P0));
    assert!(has(&t, a, KeywordKind::Flying));
    // It stays, even without ten permanents or the permanent with ascend.
    for l in lands.iter().chain([tenth].iter()) {
        t.g.move_object(*l, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.settle();
    assert!(blessed(&t, P0));
    assert!(has(&t, a, KeywordKind::Flying));
}

#[test]
fn ascend_works_only_on_the_battlefield_for_its_controller() {
    cr!("702.131b");
    ruling!(
        "Skymarcher Aspirant",
        "If you control ten permanents but don't control a permanent or resolving spell with ascend, you don't get the city's blessing."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Skymarcher Aspirant");
    t.graveyard(P0, "Skymarcher Aspirant");
    battlefield_n(&mut t, P0, "Plains", 10);
    // An opponent's permanent with ascend gives only its controller the blessing.
    t.battlefield(P1, "Skymarcher Aspirant");
    t.settle();
    assert!(!blessed(&t, P0));
    assert!(!blessed(&t, P1));
}

#[test]
fn any_number_of_players_may_have_the_citys_blessing() {
    cr!("702.131c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skymarcher Aspirant");
    battlefield_n(&mut t, P0, "Plains", 9);
    t.battlefield(P1, "Dusk Charger");
    battlefield_n(&mut t, P1, "Swamp", 9);
    t.settle();
    assert!(blessed(&t, P0));
    assert!(blessed(&t, P1));
}

#[test]
fn continuous_effects_are_reapplied_before_triggers_are_checked() {
    cr!("702.131d");
    ruling!(
        "Skymarcher Aspirant",
        "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the \"Legend Rule\" or due to being a creature with 0 toughness), you get the city's blessing before it leaves the battlefield."
    );
    let watcher = custom_card(
        "Blessing Watcher",
        "Creature — Human",
        Some((1, 1)),
        "Ascend\nWhenever another creature you control enters, if you have the city's blessing, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let charger = t.battlefield(P0, "Dusk Charger");
    battlefield_n(&mut t, P0, "Plains", 7);
    t.settle();
    assert!(!blessed(&t, P0));
    assert_eq!(t.pt(charger), (3, 3));
    // The tenth permanent: a creature that dies to state-based actions at once. The
    // blessing comes first, and the trigger sees it.
    let def = custom_card("Doomed Wisp", "Creature — Spirit", Some((0, 0)), "");
    let wisp = t.custom(P0, def, Zone::Hand(P0));
    t.g.move_object(
        wisp,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.g.recompute();
    // Dusk Charger: "This creature gets +2/+2 as long as you have the city's blessing."
    assert_eq!(t.pt(charger), (5, 5));
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Doomed Wisp"));
    assert!(blessed(&t, P0));
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.pt(charger), (5, 5));
}

#[test]
fn if_you_have_the_citys_blessing_instead() {
    cr!("702.131b");
    assert_supported_card("Kumena's Awakening");
    // Kumena's Awakening: ascend; "At the beginning of your upkeep, each player draws a
    // card. If you have the city's blessing, instead only you draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kumena's Awakening");
    battlefield_n(&mut t, P0, "Island", 9);
    t.settle();
    assert!(blessed(&t, P0));
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    // P1 drew only for their own turn.
    assert_eq!(t.hand_size(P1), h1 + 1);
}

#[test]
fn a_spell_with_ascend_checks_as_it_resolves() {
    cr!("702.131a");
    assert_supported_card("Golden Demise");
    ruling!(
        "Secrets of the Golden City",
        "For example, if you control ten permanents, lose control of one, then cast Golden Demise, you won’t have the city’s blessing and the spell will affect creatures you control."
    );
    // Golden Demise: {1}{B}{B} sorcery, ascend, "All creatures get -2/-2 until end of
    // turn. If you have the city's blessing, instead only creatures your opponents control
    // get -2/-2 until end of turn."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    battlefield_n(&mut t, P0, "Plains", 6);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let gd = t.hand(P0, "Golden Demise");
    t.cast(P0, gd).go();
    t.resolve_all();
    assert!(blessed(&t, P0));
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    // Ten permanents, but P0 loses control of one: no blessing, and it affects P0's
    // creatures too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let plains = battlefield_n(&mut t, P0, "Plains", 6);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    run(
        &mut t,
        P1,
        mtg_engine::ability::Effect::GainControl {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::Objects(vec![
                plains[0],
            ])),
            who: mtg_engine::ability::PlayerRef::You,
            duration: mtg_engine::ability::Duration::Permanent,
        },
    );
    assert_eq!(t.obj_now(plains[0]).controller, P1);
    let gd = t.hand(P0, "Golden Demise");
    t.cast(P0, gd).go();
    t.resolve_all();
    assert!(!blessed(&t, P0));
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn enters_triggers_see_the_characteristics_the_blessing_gives() {
    cr!("702.131b", "702.131d");
    assert_supported_card("Snubhorn Sentry");
    ruling!(
        "Skymarcher Aspirant",
        "use the entering permanent's characteristics after you have the city's blessing to determine whether those abilities trigger"
    );
    // Snubhorn Sentry: 0/3, ascend, "This creature gets +3/+0 as long as you have the
    // city's blessing." Elemental Bond: "Whenever a creature you control with power 3 or
    // greater enters, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    battlefield_n(&mut t, P0, "Plains", 8);
    t.lands(P0, "Plains", 1);
    let sentry = t.hand(P0, "Snubhorn Sentry");
    t.settle();
    assert!(!blessed(&t, P0));
    let hand = t.hand_size(P0);
    t.cast(P0, sentry).go();
    t.resolve_all();
    assert!(blessed(&t, P0));
    assert_eq!(t.pt(sentry), (3, 3));
    // Drew for Elemental Bond (the Sentry left the hand).
    assert_eq!(t.hand_size(P0), hand);
}
