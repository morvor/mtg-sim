//! Rulings batch P076 — spells and lands that give trample, vigilance, or "can't be
//! blocked": targets that become illegal (CR 608.2b), prevented damage, divided counters,
//! and Glade of the Pump Spells' land-play cost.

use crate::r_p076_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::has_kw;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts the cantrip `name` targeting a Hill Giant P0 controls, the Giant is destroyed
/// in response, and the spell doesn't resolve: P0 draws no card (CR 608.2b).
fn fizzle_no_draw(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, name, &[Entity::Object(giant)]);
    let hand = hand_count(&t, P0);
    crate::r_s02_common::destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(hand_count(&t, P0), hand, "{name}: drew a card");
    assert!(t.in_graveyard(P0, name));
}

#[test]
fn charge_through_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Charge Through",
        "If the target becomes illegal before Charge Through resolves (perhaps because an opponent destroyed it), you won’t draw a card."
    );
    fizzle_no_draw("Charge Through");
}

#[test]
fn impolite_entrance_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Impolite Entrance",
        "If the target creature is an illegal target as Impolite Entrance tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    fizzle_no_draw("Impolite Entrance");
}

#[test]
fn rile_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Rile",
        "If the target creature is an illegal target by the time Rile resolves, the entire spell doesn't resolve. You won't draw a card."
    );
    fizzle_no_draw("Rile");
}

#[test]
fn enter_the_enigma_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Enter the Enigma",
        "If the target creature is an illegal target when Enter the Enigma tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    fizzle_no_draw("Enter the Enigma");
}

#[test]
fn rile_gives_trample_even_if_its_damage_is_prevented() {
    cr!("615.1", "608.2c");
    ruling!(
        "Rile",
        "If the damage that would be dealt by Rile is prevented, the creature still gains trample until end of turn."
    );
    supported("Rile");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    // Mending Hands: "Prevent the next 4 damage that would be dealt to any target this turn."
    cast_new(&mut t, P0, "Mending Hands", &[Entity::Object(giant)]);
    t.resolve_all();
    let hand = hand_count(&t, P0);
    cast_new(&mut t, P0, "Rile", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(has_kw(&t, giant, KeywordKind::Trample));
    assert_eq!(hand_count(&t, P0), hand + 1, "Rile still draws a card");
}

#[test]
fn crash_through_with_no_creatures_just_draws() {
    cr!("608.2c");
    ruling!(
        "Crash Through",
        "You may cast Crash Through even if you control no creatures. If you control no creatures as the spell resolves, you'll just draw a card."
    );
    supported("Crash Through");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Crash Through", &[]);
    let hand = hand_count(&t, P0);
    t.resolve_all();
    assert_eq!(hand_count(&t, P0), hand + 1);
}

#[test]
fn invigorated_rampage_second_mode_with_one_illegal_target() {
    cr!("608.2b", "700.2");
    ruling!(
        "Invigorated Rampage",
        "If you choose Invigorated Rampage's second mode and one target becomes an illegal target, the remaining target gets +2/+0 and gains trample. It doesn't get +4/+0."
    );
    supported("Invigorated Rampage");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Invigorated Rampage");
    let card = t.hand(P0, "Invigorated Rampage");
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    t.cast(P0, card).go();
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 3));
    assert!(has_kw(&t, giant, KeywordKind::Trample));
}

#[test]
fn natures_way_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Nature's Way",
        "If the creature you don't control is an illegal target as Nature's Way tries to resolve, the creature you control will still gain vigilance and trample."
    );
    ruling!(
        "Nature's Way",
        "If either target is an illegal target as Nature's Way resolves, no damage will be dealt."
    );
    supported("Nature's Way");
    // The creature you don't control is gone: yours still gains vigilance and trample.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Nature's Way", &[Entity::Object(giant), Entity::Object(bears)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert!(has_kw(&t, giant, KeywordKind::Vigilance));
    assert!(has_kw(&t, giant, KeywordKind::Trample));
    // Your creature is gone: no damage is dealt to theirs.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Nature's Way", &[Entity::Object(giant), Entity::Object(bears)]);
    // Bounce your creature in response (it's an illegal target now).
    let g = t.g.current(giant);
    t.g.move_object(
        g,
        mtg_engine::object::Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn storm_the_seedcore_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Storm the Seedcore",
        "If all of Storm the Seedcore’s targets are illegal at the time the spell tries to resolve, it won’t resolve and none of its effects will happen. Creatures you control won’t gain vigilance and trample."
    );
    supported("Storm the Seedcore");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::divide(&mut t, P0, &[4]);
    cast_new(&mut t, P0, "Storm the Seedcore", &[Entity::Object(bears)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert!(!has_kw(&t, giant, KeywordKind::Vigilance));
    assert!(!has_kw(&t, giant, KeywordKind::Trample));
    assert_eq!(t.counters(giant, "+1/+1"), 0);
}

#[test]
fn storm_the_seedcore_keeps_the_division_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Storm the Seedcore",
        "If some of the creatures are illegal targets as Storm the Seedcore tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost. They won’t be put instead on a legal target."
    );
    supported("Storm the Seedcore");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::divide(&mut t, P0, &[1, 3]);
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    cast_new(&mut t, P0, "Storm the Seedcore", &[]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    assert!(has_kw(&t, giant, KeywordKind::Vigilance));
    assert!(has_kw(&t, giant, KeywordKind::Trample));
}

#[test]
fn vigorspore_wurm_with_no_creature_cards_just_gives_vigilance() {
    cr!("603.3");
    ruling!(
        "Vigorspore Wurm",
        "If there are no creature cards in your graveyard, the target creature just gains vigilance until end of turn."
    );
    supported("Vigorspore Wurm");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Vigorspore Wurm");
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 3));
    assert!(has_kw(&t, giant, KeywordKind::Vigilance));
}

#[test]
fn vigorspore_wurm_with_menace_cant_be_blocked() {
    cr!("702.111b", "509.1b");
    ruling!(
        "Vigorspore Wurm",
        "If Vigorspore Wurm gains menace, it can't be blocked at all."
    );
    supported("Vigorspore Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Vigorspore Wurm");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        wurm,
        vec![mtg_engine::ability::Modification::AddKeyword(
            mtg_engine::keywords::Keyword::new(KeywordKind::Menace),
        )],
    );
    crate::r_s01_common::attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    use crate::r_s21_common::legal_blocks;
    assert!(!legal_blocks(&mut t, P1, &[(a, wurm)]));
    assert!(!legal_blocks(&mut t, P1, &[(a, wurm), (b, wurm)]));
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn glade_of_the_pump_spells_needs_a_land_drop() {
    cr!("305.2", "116.2a");
    ruling!(
        "Glade of the Pump Spells",
        "Paying {2}{G} won’t let you play Glade of the Pump Spells if you don’t have any land drops remaining that turn."
    );
    ruling!(
        "Glade of the Pump Spells",
        "You still follow all of the normal timing rules for playing a land as a special action when you play Glade of the Pump Spells. You just have to pay {2}{G} in addition to all of the other requirements as you do so."
    );
    supported("Glade of the Pump Spells");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Without {2}{G} it can't be played; with it, it's played as the land drop.
    let glade = t.hand(P0, "Glade of the Pump Spells");
    assert!(t.play_land(P0, glade).is_err());
    assert_eq!(t.g.player(P0).lands_played_this_turn, 0);
    mana(&mut t, P0, ManaType::G, 3);
    t.play_land(P0, glade).expect("first Glade");
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "the {{2}}{{G}} was paid");
    // A second one can't be played with no land drop left, even paying {2}{G}.
    mana(&mut t, P0, ManaType::G, 3);
    let glade2 = t.hand(P0, "Glade of the Pump Spells");
    assert!(t.play_land(P0, glade2).is_err());
    // Timing rules: not during the opponent's turn, even paying {2}{G}.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::G, 3);
    let glade = t.hand(P0, "Glade of the Pump Spells");
    assert!(t.play_land(P0, glade).is_err());
    // Nor while the stack isn't empty.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Crash Through", &[]);
    mana(&mut t, P0, ManaType::G, 3);
    let glade = t.hand(P0, "Glade of the Pump Spells");
    assert!(t.play_land(P0, glade).is_err());
}

#[test]
fn glade_of_the_pump_spells_put_onto_the_battlefield_costs_nothing() {
    cr!("305.4");
    ruling!(
        "Glade of the Pump Spells",
        "You don’t have to pay the mana cost if a spell or ability lets you put Glade of the Pump Spells onto the battlefield, such as with the activated ability of Elvish Reclaimer."
    );
    supported("Glade of the Pump Spells");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P0, "Glade of the Pump Spells");
    // Growth Spiral: "Draw a card. You may put a land card from your hand onto the
    // battlefield." (Paid with mana in the pool; nothing else is left to pay with.)
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::U, 1);
    let spiral = t.hand(P0, "Growth Spiral");
    crate::r_s22_common::choose_named_when_offered(&mut t, P0, "Glade of the Pump Spells");
    t.answer_yes(P0, true);
    t.cast(P0, spiral).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Glade of the Pump Spells").len(), 1);
}
