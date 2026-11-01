//! Rulings batch P150 — Royal Assassin's target, Moderation's spell limit, Painter's
//! Servant's color-adding effect and Blood Sun's ability loss.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s17_common::color_set;
use crate::r_s21_common::castable;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Royal Assassin: "{T}: Destroy target tapped creature."
// ---------------------------------------------------------------------------------------

#[test]
fn royal_assassin_cant_target_itself() {
    cr!("602.2b", "601.2c", "601.2g");
    ruling!(
        "Royal Assassin",
        "Can't target itself because you choose the target before you tap him."
    );
    supported("Royal Assassin");
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Royal Assassin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let from = t.asked().len();
    t.activate(P0, assassin, 0, &[Entity::Object(bears)])
        .unwrap();
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![Entity::Object(bears)]]);
    assert!(t.obj_now(assassin).tapped);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn royal_assassin_ability_does_nothing_if_the_target_untapped() {
    cr!("608.2b");
    ruling!(
        "Royal Assassin",
        "If the creature is no longer tapped when the Assassin's ability resolves, then the ability doesn't resolve."
    );
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Royal Assassin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.activate(P0, assassin, 0, &[Entity::Object(bears)])
        .unwrap();
    t.g.untap(bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Moderation: "You can't cast more than one spell each turn. Whenever you cast a spell,
// draw a card."
// ---------------------------------------------------------------------------------------

#[test]
fn moderation_counts_itself_but_not_spells_before_it_was_controlled() {
    cr!("101.2", "601.2");
    ruling!(
        "Moderation",
        "If you cast Moderation, you've already cast at least one spell that turn (Moderation itself) so you won't be able to cast any more spells until the next turn. If Moderation comes under your control on a turn you haven't cast any spells, you'll be able to cast one spell that turn."
    );
    ruling!(
        "Moderation",
        "Moderation's triggered ability works only while it's on the battlefield, so it doesn't let you draw a card when you cast it."
    );
    supported("Moderation");
    // Cast: no card drawn for Moderation itself, and no second spell.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_and_resolve(&mut t, P0, "Moderation", &[]);
    assert_eq!(t.hand_size(P0), 0);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    assert!(!castable(&mut t, P0, shock));
    // Put onto the battlefield: one spell (which draws a card), then no more.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Moderation");
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P0, "Shock");
    let second = t.hand(P0, "Shock");
    assert!(castable(&mut t, P0, shock));
    t.cast(P0, shock).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(!castable(&mut t, P0, second));
    // The next turn, one again.
    t.advance_to(P1, Step::Upkeep);
    assert!(castable(&mut t, P0, second));
}

#[test]
fn moderation_stops_a_suspended_card_from_being_cast() {
    cr!("101.2", "702.62a");
    ruling!(
        "Moderation",
        "If an effect says you can't cast spells, and another effect instructs you to cast a spell, such as that of the ability that triggers when you remove the last time counter from a card with suspend, you won't be able to cast that spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moderation");
    // Rift Bolt in exile, suspended with one time counter.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::Suspend {
            card: bolt,
        }),
    )
    .unwrap();
    let exiled = t.g.current(bolt);
    // P0's upkeep: the last counter is removed; in response to the cast trigger, P0 casts
    // a spell, the one spell allowed this turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert_eq!(t.counters(exiled, "time"), 0);
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast_with(P0, shock, &[Entity::Player(P1)]).unwrap();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "only Shock dealt damage");
    assert_eq!(t.zone(exiled), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Painter's Servant: "As this creature enters, choose a color. All cards that aren't on the
// battlefield, spells, and permanents are the chosen color in addition to their other
// colors."
// ---------------------------------------------------------------------------------------

fn servant(t: &mut TestGame, p: PlayerId, color: Color) -> ObjectId {
    let i = Color::ALL.iter().position(|c| *c == color).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
    let s = t.enter(p, "Painter's Servant");
    t.g.recompute();
    s
}

fn colors(t: &mut TestGame, id: ObjectId) -> ColorSet {
    t.g.recompute();
    t.obj_now(id).chars.colors
}

#[test]
fn painters_servant_adds_its_color_cumulatively_everywhere() {
    cr!("105.2", "105.3", "613.1e");
    ruling!(
        "Painter's Servant",
        "This ability doesn't overwrite any previous colors. Rather, it adds another color."
    );
    ruling!(
        "Painter's Servant",
        "The effects of multiple Painter's Servants are cumulative."
    );
    ruling!(
        "Painter's Servant",
        "If something affected by Painter's Servant is normally colorless, it will simply be the new color. It won't be both the new color and colorless."
    );
    ruling!(
        "Painter's Servant",
        "This ability affects every card in every game zone, all tokens on the battlefield, and all spell copies on the stack, regardless of who controls or owns them."
    );
    supported("Painter's Servant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let hand = t.hand(P1, "Lightning Bolt");
    let lib = t.library_top(P1, "Llanowar Elves");
    let exiled = t.exile(P1, "Hill Giant");
    let gy = t.graveyard(P0, "Counterspell");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    servant(&mut t, P0, Color::Blue);
    servant(&mut t, P0, Color::Red);
    let blue_red = color_set(&[Color::Blue, Color::Red]);
    assert_eq!(
        colors(&mut t, bears),
        color_set(&[Color::Blue, Color::Red, Color::Green])
    );
    assert_eq!(colors(&mut t, thopter), blue_red);
    assert_eq!(colors(&mut t, token), blue_red);
    assert_eq!(colors(&mut t, hand), blue_red);
    assert_eq!(
        colors(&mut t, lib),
        color_set(&[Color::Blue, Color::Red, Color::Green])
    );
    assert_eq!(colors(&mut t, exiled), blue_red);
    assert_eq!(colors(&mut t, gy), blue_red);
    // A spell an opponent controls and a copy of it on the stack.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast_with(P0, twincast, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    let copy = *crate::r_s25_common::spell_copies(&t)
        .first()
        .expect("a copy");
    assert_eq!(colors(&mut t, spell), blue_red);
    assert_eq!(colors(&mut t, copy), blue_red);
}

#[test]
fn painters_servant_is_overwritten_by_a_later_color_change() {
    cr!("613.1e", "613.7");
    ruling!(
        "Painter's Servant",
        "While Painter's Servant is on the battlefield, an effect that changes an object's colors will overwrite Painter's Servant's effect. For example, casting Cerulean Wisps on a creature will turn it blue, regardless of the color chosen for Painter's Servant."
    );
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    servant(&mut t, P0, Color::Red);
    cast_and_resolve(&mut t, P0, "Cerulean Wisps", &[Entity::Object(bears)]);
    assert_eq!(colors(&mut t, bears), ColorSet::single(Color::Blue));
}

#[test]
fn painters_servant_colors_objects_as_they_enter_and_as_they_are_cast() {
    cr!("400.7", "603.2", "603.6a");
    ruling!(
        "Painter's Servant",
        "Each card becomes a new object as it changes zones, so this effect will apply to it from scratch in the new zone."
    );
    supported("Court Street Denizen");
    supported("Kraken's Eye");
    let mut t = TestGame::new(2);
    let opp = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Court Street Denizen");
    t.battlefield(P0, "Kraken's Eye");
    servant(&mut t, P0, Color::White);
    // Not white otherwise; the Denizen's trigger sees it white as it enters.
    t.answer_targets(P0, &[Entity::Object(opp)]);
    let elves = t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    assert!(colors(&mut t, elves).contains(Color::White));
    assert!(t.obj_now(opp).tapped, "Court Street Denizen triggered");
    // A blue Servant: Kraken's Eye triggers for a red spell.
    servant(&mut t, P0, Color::Blue);
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 - 3 + 1);
}

#[test]
fn painters_servant_color_applies_to_replacement_effects_on_entering_permanents() {
    cr!("614.12", "613.1e");
    ruling!(
        "Painter's Servant",
        "replacement effects that care about the color of permanents entering the battlefield"
    );
    supported("Curator Beastie");
    // Curator Beastie: "Colorless creatures you control enter with two additional +1/+1
    // counters on them." A colorless Ornithopter gets them, unless a Painter's Servant
    // makes it red as it enters.
    for with_servant in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Curator Beastie");
        if with_servant {
            servant(&mut t, P0, Color::Red);
        }
        let thopter = t.enter(P0, "Ornithopter");
        let expected = if with_servant { 0 } else { 2 };
        assert_eq!(
            t.counters(thopter, "+1/+1"),
            expected,
            "servant: {with_servant}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Blood Sun: "All lands lose all abilities except mana abilities."
// ---------------------------------------------------------------------------------------

fn activated_abilities(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

fn flying(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying)
}

#[test]
fn blood_sun_doesnt_remove_abilities_lands_gain_later() {
    cr!("613.1f", "613.7a");
    ruling!(
        "Blood Sun",
        "If a land gains an ability after Blood Sun has entered the battlefield, it keeps that ability."
    );
    supported("Blood Sun");
    supported("Hydroform");
    // Hydroform: "Target land becomes a 3/3 Elemental creature with flying until end of
    // turn. It's still a land."
    for blood_sun_first in [true, false] {
        let mut t = TestGame::new(2);
        let forest = t.battlefield(P0, "Forest");
        if blood_sun_first {
            t.battlefield(P0, "Blood Sun");
        }
        cast_and_resolve(&mut t, P0, "Hydroform", &[Entity::Object(forest)]);
        if !blood_sun_first {
            t.battlefield(P0, "Blood Sun");
        }
        assert_eq!(flying(&mut t, forest), blood_sun_first);
        assert_eq!(t.pt(forest), (3, 3));
    }
}

#[test]
fn blood_sun_lets_type_changing_land_abilities_apply_but_stops_granting_ones() {
    cr!("613.1d", "613.1f", "613.8");
    ruling!(
        "Blood Sun",
        "If a land has an ability that continuously changes the types of other lands (such as Urborg, Tomb of Yawgmoth does), that ability will apply before Blood Sun removes that land’s abilities. If a land has an ability that grants abilities to other objects, Blood Sun will stop it from doing so."
    );
    supported("Urborg, Tomb of Yawgmoth");
    supported("The World Tree");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Sun");
    t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    let mountain = t.battlefield(P1, "Mountain");
    t.g.recompute();
    assert!(t.obj_now(mountain).chars.has_subtype("Swamp"));
    // The World Tree: with six lands, "lands you control have '{T}: Add one mana of any
    // color.'" — Blood Sun removes that static ability, so the Mountain doesn't get it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The World Tree");
    let lands = t.lands(P0, "Mountain", 5);
    let with_tree = activated_abilities(&mut t, lands[0]);
    t.battlefield(P0, "Blood Sun");
    assert_eq!(activated_abilities(&mut t, lands[0]), with_tree - 1);
}
