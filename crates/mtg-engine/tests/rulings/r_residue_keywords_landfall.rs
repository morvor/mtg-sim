//! Item `rulings-residue-keywords` — landfall interactions (CR 207.2c, 603.2): Skyclave
//! Shade's permission and new objects (400.7, 601.3), Admonition Angel's linked abilities
//! and leaves-the-battlefield order (607.2a, 603.6c), landfall abilities that add mana
//! still using the stack (605.1a), Eternity Vessel's life total (119.5, 810.9), Moraug's
//! additional combat (500.8), and Springheart Nantuko's optional payment.

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether P0 could cast the card now (any way).
fn castable(t: &mut TestGame, card: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// P0 plays a Forest from hand (a land enters under P0's control).
fn play_forest(t: &mut TestGame) {
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).unwrap();
}

/// Skyclave Shade in P0's graveyard during P0's precombat main phase, with mana for it
/// kicked, after a land entered and the landfall ability resolved (P0 chose to allow it).
fn shade() -> (TestGame, ObjectId) {
    supported("Skyclave Shade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let shade = t.graveyard(P0, "Skyclave Shade");
    t.answer_yes(P0, true);
    play_forest(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    (t, shade)
}

#[test]
fn skyclave_shade_cast_from_the_graveyard_follows_timing_and_may_be_kicked() {
    cr!("601.3", "307.1", "702.33a", "117.1a");
    ruling!(
        "Skyclave Shade",
        "You must follow the normal timing permissions and restrictions for Skyclave Shade when casting it with the permission of its landfall ability. You must pay its mana cost (or, if another effect allows, an alternative cost). You may pay its kicker cost when casting it this way."
    );
    let (mut t, shade) = shade();
    // Not during combat (it's a creature card without flash) ...
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, shade));
    // ... but in a main phase with an empty stack, kicked, paying its costs.
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, shade));
    let spell = t.cast(P0, shade).kicked(true).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
    t.resolve_all();
    let s = t.named_on_battlefield("Skyclave Shade")[0];
    assert_eq!(p1(&t, s), 2);
}

#[test]
fn skyclave_shade_is_a_new_object_after_it_was_cast() {
    cr!("400.7", "601.3");
    ruling!(
        "Skyclave Shade",
        "After you cast Skyclave Shade from your graveyard, it's considered a new object, and the permission from its landfall ability won't apply to it again even if it returns to your graveyard later in the turn. You'll need to have another land enter the battlefield under your control after it has returned to your graveyard to cast it again."
    );
    let (mut t, shade) = shade();
    t.cast(P0, shade).kicked(false).go();
    t.resolve_all();
    let s = t.named_on_battlefield("Skyclave Shade")[0];
    destroy(&mut t, s);
    let card = t.g.current(s);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.lands(P0, "Swamp", 2);
    assert!(!castable(&mut t, card));
    // Another land enters: it can be cast again.
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert!(castable(&mut t, card));
}

#[test]
fn skyclave_shade_must_be_in_the_graveyard_as_the_land_enters() {
    cr!("603.4", "603.10");
    ruling!(
        "Skyclave Shade",
        "Skyclave Shade's landfall ability triggers only if it's in your graveyard immediately after the land enters the battlefield."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let shade = t.hand(P0, "Skyclave Shade");
    play_forest(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Discarded afterward, it can't be cast from the graveyard.
    move_to(&mut t, shade, Zone::Graveyard(P0));
    assert!(!castable(&mut t, shade));
}

/// P0's Admonition Angel; P0 answers yes to the "may" and the target for the next
/// landfall ability.
fn angel_exiles(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P0, true);
}

#[test]
fn admonition_angel_returns_only_cards_its_landfall_ability_exiled() {
    cr!("607.1", "607.2a", "610.3");
    ruling!(
        "Admonition Angel",
        "Admonition Angel's landfall ability and its last ability are linked. The last ability refers only to cards exiled with its landfall ability."
    );
    supported("Admonition Angel");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Admonition Angel");
    let theirs = t.battlefield(P1, "Admonition Angel");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P0, "Hill Giant");
    // P0's Angel exiles P1's Bears; P1's Angel exiles P0's Giant.
    angel_exiles(&mut t, x);
    t.enter(P0, "Forest");
    t.resolve_all();
    t.answer_targets(P1, &[obj(y)]);
    t.answer_yes(P1, true);
    t.enter(P1, "Forest");
    t.resolve_all();
    assert_eq!(t.zone(x), Zone::Exile);
    assert_eq!(t.zone(y), Zone::Exile);
    // P0's Angel leaves: only the Bears return.
    move_to(&mut t, mine, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(x), Zone::Battlefield);
    assert_eq!(t.zone(y), Zone::Exile);
    assert!(t.on_battlefield(theirs));
}

#[test]
fn admonition_angel_leaving_with_landfall_on_the_stack_exiles_forever() {
    cr!("603.6c", "603.3b", "610.3c", "607.2a");
    ruling!(
        "Admonition Angel",
        "If Admonition Angel's landfall ability triggers, but the Angel leaves the battlefield before it resolves, its last ability will trigger and resolve first. All previously exiled cards will be returned to the battlefield. Then the landfall ability will resolve and exile the targeted permanent forever."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Admonition Angel");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Hill Giant");
    angel_exiles(&mut t, x);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.zone(x), Zone::Exile);
    angel_exiles(&mut t, y);
    t.enter(P0, "Forest");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the Angel leaves: its last ability goes on the stack above.
    move_to(&mut t, angel, Zone::Hand(P0));
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(x), Zone::Battlefield);
    assert_eq!(t.zone(y), Zone::Battlefield);
    t.resolve_all();
    assert_eq!(t.zone(y), Zone::Exile);
    // Forever: the Angel coming back and leaving again doesn't return it.
    let a = t.g.current(angel);
    t.lands(P0, "Plains", 6);
    t.cast(P0, a).go();
    t.resolve_all();
    let a = t.named_on_battlefield("Admonition Angel")[0];
    move_to(&mut t, a, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(y), Zone::Exile);
}

#[test]
fn nissa_and_omnath_landfall_abilities_that_add_mana_use_the_stack() {
    cr!("605.1a", "605.5a", "603.3", "117.3b");
    ruling!(
        "Nissa, Resurgent Animist",
        "Nissa's landfall ability isn't a mana ability, even though it causes you to add mana. It uses the stack and can be responded to."
    );
    ruling!(
        "Omnath, Locus of Creation",
        "Omnath's landfall ability always uses the stack and players may respond to it. It isn't a mana ability because the event that causes it to trigger isn't a mana ability, even if it's the second time the ability has resolved in a turn."
    );
    supported("Nissa, Resurgent Animist");
    supported("Omnath, Locus of Creation");
    for name in ["Nissa, Resurgent Animist", "Omnath, Locus of Creation"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        for i in 0..2 {
            t.enter(P0, "Forest");
            t.settle();
            // On the stack, not resolved: no mana yet, and P1 gets priority to respond.
            assert_eq!(t.stack_len(), 1, "{name} #{i}");
            let before = crate::r_p119_common::pool_total(&t, P0);
            let from = t.asked().len();
            t.g.turn.priority = Some(P0);
            t.g.run_until(100, |g| g.stack.is_empty());
            assert!(asked_since(&t, from)
                .iter()
                .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. })));
            // (Omnath's first resolution gains life; its second adds {R}{G}{W}{U}.)
            if name.starts_with("Omnath") && i == 0 {
                assert_eq!(t.life(P0), 24);
            } else {
                assert!(
                    crate::r_p119_common::pool_total(&t, P0) > before,
                    "{name} #{i}"
                );
            }
        }
    }
}

#[test]
fn nissa_after_the_second_time_just_adds_mana() {
    cr!("603.2", "106.1");
    ruling!(
        "Nissa, Resurgent Animist",
        "Each time the landfall ability resolves in a turn after the second time, you'll just add mana."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nissa, Resurgent Animist");
    stack_library(
        &mut t,
        P0,
        &["Llanowar Elves", "Elvish Mystic", "Elvish Visionary"],
    );
    let hand = t.hand_size(P0);
    let mut revealed = vec![];
    for _ in 0..3 {
        t.enter(P0, "Forest");
        t.resolve_all();
        revealed.push(t.hand_size(P0) - hand);
    }
    // Only the second resolution revealed (an Elf put into hand).
    assert_eq!(revealed, vec![0, 1, 1]);
    assert_eq!(crate::r_p119_common::pool_total(&t, P0), 3);
}

/// P0's Eternity Vessel entered with `life` P0's life total (as charge counters).
fn vessel(t: &mut TestGame, life: i32) -> ObjectId {
    supported("Eternity Vessel");
    t.g.players[0].life = life;
    let v = t.enter(P0, "Eternity Vessel");
    assert_eq!(t.counters(v, "charge"), life.max(0) as u32);
    v
}

#[test]
fn eternity_vessel_counts_its_charge_counters_as_landfall_resolves() {
    cr!("119.5", "122.1");
    ruling!(
        "Eternity Vessel",
        "Eternity Vessel's landfall ability cares only about how many charge counters are on it, not what its controller's life total was as it entered. The number of charge counters may be changed by other spells and abilities."
    );
    let mut t = TestGame::new(2);
    let v = vessel(&mut t, 20);
    t.g.players[0].life = 15;
    crate::r_p108_common::put_counters(&mut t, v, "charge", 3);
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn eternity_vessel_life_total_change_is_a_gain_or_a_loss() {
    cr!("119.5", "119.3");
    ruling!(
        "Eternity Vessel",
        "For your life total to become the number of charge counters on Eternity Vessel, you actually gain or lose the necessary amount of life. For example, if your life total is 4 when Eternity Vessel's landfall ability resolves and Eternity Vessel has 12 charge counters on it, the ability will cause you to gain 8 life. Other cards that interact with life gain or life loss will interact with this effect accordingly."
    );
    let mut t = TestGame::new(2);
    vessel(&mut t, 12);
    t.g.players[0].life = 4;
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    // Boon Reflection: "If you would gain life, you gain twice that much life instead."
    t.battlefield(P0, "Boon Reflection");
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    // A gain of 8, doubled; and Pridemate's "whenever you gain life" triggered.
    assert_eq!(t.life(P0), 20);
    assert_eq!(p1(&t, pridemate), 1);
}

#[test]
fn eternity_vessel_in_two_headed_giant() {
    cr!("810.9", "119.5", "810.9a");
    ruling!(
        "Eternity Vessel",
        "In a Two-Headed Giant game, your life total is the same as your team's life total, so Eternity Vessel enters with a number of charge counters on it equal to your team's life total. When its landfall ability resolves, it causes its controller to lose or gain the appropriate amount of life, and the team's life total is adjusted by the amount of life the player gains or loses as a result of this ability. Suppose Eternity Vessel has 15 charge counters on it when its landfall ability resolves, and its controller's team has 11 life. That player's life total is 11, which becomes 15, for a net gain of 4 life. The team's life total becomes 15."
    );
    supported("Eternity Vessel");
    let mut t = two_headed_giant();
    let team = t.life(P0);
    let v = t.enter(P0, "Eternity Vessel");
    assert_eq!(t.counters(v, "charge"), team as u32);
    // 15 counters; the team has 11 life.
    let c = t.g.current(v);
    t.g.objects[c.0 as usize]
        .counters
        .insert("charge".into(), 15);
    t.g.lose_life(P1, (team - 11) as u32);
    t.settle();
    assert_eq!(t.life(P0), 11);
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn moraug_landfall_in_the_first_main_phase_adds_a_combat_before_the_regular_one() {
    cr!("500.8", "505.1a", "506.1");
    ruling!(
        "Moraug, Fury of Akoum",
        "If the landfall ability resolves during your precombat main phase, the additional combat phase will happen before your regular combat phase. You'll untap creatures you control at the beginning of the additional combat but not at the beginning of your regular combat."
    );
    supported("Moraug, Fury of Akoum");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moraug, Fury of Akoum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    play_forest(&mut t);
    t.resolve_all();
    // The additional combat: Bears untap at its beginning, then attack.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!tapped(&t, bears));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17); // 2 + 1 for having attacked once this turn
                                // Then the regular combat, with no main phase between and no untap.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.g.turn.combat_phases, 2);
    assert_eq!(t.g.turn.main_phases, 1);
    assert!(tapped(&t, bears));
}

#[test]
fn springheart_nantuko_makes_an_insect_whenever_no_copy_was_made() {
    cr!("603.5", "702.103b");
    ruling!(
        "Springheart Nantuko",
        "If you don't pay {1}{G}, either because you simply chose not to or because Springheart Nantuko isn't attached to a creature you control, you'll still create a 1/1 green Insect creature token when Springheart Nantuko's landfall ability resolves."
    );
    supported("Springheart Nantuko");
    // Attached (bestowed) to P0's Bears; P0 declines to pay.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let n = t.hand(P0, "Springheart Nantuko");
    t.cast(P0, n)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve();
    t.answer_yes(P0, false);
    play_forest(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(tokens_of(&t, P0, "Insect"), 1);
    // A creature, not attached: no payment, an Insect.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Springheart Nantuko");
    t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    play_forest(&mut t);
    t.resolve_all();
    assert!(!was_asked(&t, from, |d| matches!(
        d,
        Decision::YesNo { .. }
    )));
    assert_eq!(tokens_of(&t, P0, "Insect"), 1);
}
