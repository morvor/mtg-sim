//! CR 701.65: airbend; CR 701.66: earthbend; CR 701.67: waterbend.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{Effect, KeywordAction, PlayerRef, Sel, Value};
use mtg_engine::kwa::bending::{AIRBEND_METHOD, AIRBENT_EVENT, EARTHBENT_EVENT, WATERBENT_EVENT};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn airbend(t: &mut TestGame, p: PlayerId, what: &[ObjectId]) {
    let targets: Vec<Entity> = what.iter().map(|o| Entity::Object(*o)).collect();
    run(t, p, None, ka(KeywordAction::Airbend, Sel::Target(0), 1), &targets);
}

/// A permanent with "Whenever you [action], you gain 1 life."
fn gain_on(t: &mut TestGame, p: PlayerId, action: &str) {
    t.custom(
        p,
        text_card(
            "Bending Observer",
            "Enchantment",
            "{1}",
            None,
            &format!("Whenever you {action}, you gain 1 life."),
        ),
        Zone::Battlefield,
    );
}

#[test]
fn airbent_cards_are_exiled_and_their_owners_may_cast_them_for_two() {
    cr!("701.65a");
    ruling!(
        "Airbending Lesson",
        "For each card exiled this way, for as long as it's exiled, its owner may cast it from exile by paying {2} rather than paying its mana cost."
    );
    supported("Airbending Lesson");
    // "Airbend target nonland permanent. Draw a card."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Airbending Lesson");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let exiled = t.g.current(giant);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Not its controller who airbent it: its owner casts it, for {2}.
    t.g.turn.priority = Some(P0);
    assert!(t
        .g
        .cast_spell(P0, exiled, CastMethod::Alternative(AIRBEND_METHOD))
        .is_err());
    t.set_step(P1, Step::PrecombatMain);
    let lands = t.lands(P1, "Mountain", 2);
    t.cast(P1, exiled)
        .method(CastMethod::Alternative(AIRBEND_METHOD))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    // A card that left exile and came back is a new object: it can't be cast this way.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    airbend(&mut t, P0, &[bears]);
    let exiled = t.g.current(bears);
    let options = t.g.cast_options(P1, exiled);
    assert!(options
        .iter()
        .any(|o| o.method == CastMethod::Alternative(AIRBEND_METHOD)));
    t.g.move_object(exiled, Zone::Graveyard(P1), mtg_engine::events::MoveCause::Effect, None);
    let back = t.g.current(exiled);
    t.g.exile_object(back, None);
    let again = t.g.current(back);
    assert_eq!(t.zone(again), Zone::Exile);
    assert!(t.g.cast_options(P1, again).is_empty());
}

#[test]
fn airbending_spells_and_tokens() {
    cr!("701.65a");
    ruling!(
        "Airbender Ascension",
        "If a token is exiled by airbend or Airbender Ascension's last ability, it will cease to exist and can't be cast or return to the battlefield."
    );
    // A spell: exiled from the stack, it doesn't resolve; its owner may cast it for {2}.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let spell = t.cast(P1, bolt).go();
    airbend(&mut t, P0, &[spell]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let exiled = t.g.current(spell);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.lands(P1, "Mountain", 2);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.cast(P1, exiled)
        .method(CastMethod::Alternative(AIRBEND_METHOD))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // A token ceases to exist.
    let mut t = TestGame::new(2);
    run(
        &mut t,
        P1,
        None,
        Effect::CreateToken {
            spec: mtg_engine::tokens::predefined("Treasure").expect("Treasure"),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &[],
    );
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token())
        .map(|o| o.id)
        .expect("token");
    airbend(&mut t, P0, &[token]);
    t.settle();
    assert!(t.g.permanents().all(|o| !o.is_token()));
    assert!(t
        .g
        .exile
        .iter()
        .all(|o| t.g.cast_options(P1, *o).is_empty()));
    // It was exiled, though: P0 airbent.
    assert_eq!(custom_events(&t, AIRBENT_EVENT).len(), 1);
}

#[test]
fn whenever_you_airbend_triggers_once_one_or_more_objects_are_exiled() {
    cr!("701.65b");
    supported("Whirlwind Technique");
    // "Target player draws two cards, then discards a card. Airbend up to two target
    // creatures."
    let mut t = TestGame::new(2);
    gain_on(&mut t, P0, "airbend");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 6);
    let spell = t.hand(P0, "Whirlwind Technique");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(b)), Zone::Exile);
    assert_eq!(t.life(P0), 21);
    // Nothing exiled: no trigger.
    let mut t = TestGame::new(2);
    gain_on(&mut t, P0, "airbend");
    t.lands(P0, "Island", 6);
    let spell = t.hand(P0, "Whirlwind Technique");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(custom_events(&t, AIRBENT_EVENT).is_empty());
}

/// Casts Earthbending Lesson ("Earthbend 4.") targeting `land`.
fn earthbending_lesson(t: &mut TestGame, land: ObjectId) {
    supported("Earthbending Lesson");
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Earthbending Lesson");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.cast(P0, spell).go();
    t.resolve_all();
}

#[test]
fn earthbend_animates_a_land_and_returns_it_when_it_dies_or_is_exiled() {
    cr!("701.66a");
    ruling!(
        "Earthbending Lesson",
        "If a land was animated by earthbend and would go to any zone other than the graveyard or exile, it will not be returned to the battlefield by the delayed triggered ability created by earthbend."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    earthbending_lesson(&mut t, land);
    let o = t.obj_now(land);
    assert!(o.is_creature() && o.is(CardType::Land));
    assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert_eq!(t.counters(land, counters::PLUS1), 4);
    assert_eq!(t.pt(land), (4, 4));
    // It dies: it returns to the battlefield tapped (a new object, a land again).
    t.g.destroy(land, None);
    t.resolve_all();
    let back = t.g.current(land);
    assert_eq!(t.zone(back), Zone::Battlefield);
    assert!(t.obj_now(back).tapped);
    assert!(!t.obj_now(back).is_creature());
    assert_eq!(t.obj_now(back).controller, P0);
    // Put into exile (by airbending it): it returns too.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    earthbending_lesson(&mut t, land);
    airbend(&mut t, P1, &[land]);
    t.resolve_all();
    let back = t.g.current(land);
    assert_eq!(t.zone(back), Zone::Battlefield);
    assert!(t.obj_now(back).tapped);
    // Returned to its owner's hand: it stays there.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    earthbending_lesson(&mut t, land);
    t.g.move_object(land, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(land)), Zone::Hand(P0));
}

#[test]
fn earthbending_a_land_thats_already_a_creature() {
    cr!("701.66a");
    ruling!(
        "Earthbending Lesson",
        "You may target a land that is already a creature, perhaps because of a previous earthbend ability. The land will get the +1/+1 counters, gain haste, and have its base power and toughness set to 0/0."
    );
    supported("Badgermole Cub");
    // Badgermole Cub: "When this creature enters, earthbend 1."
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.enter(P0, "Badgermole Cub");
    t.resolve_all();
    assert_eq!(t.pt(land), (1, 1));
    earthbending_lesson(&mut t, land);
    assert_eq!(t.pt(land), (5, 5));
}

#[test]
fn whenever_you_earthbend() {
    cr!("701.66b");
    let mut t = TestGame::new(2);
    gain_on(&mut t, P0, "earthbend");
    let land = t.battlefield(P0, "Wastes");
    earthbending_lesson(&mut t, land);
    assert_eq!(t.life(P0), 21);
    assert_eq!(
        custom_events(&t, EARTHBENT_EVENT),
        vec![(Some(P0), Some(land), 4)]
    );
    // The delayed triggered ability exists.
    assert!(!t.g.delayed_triggers.is_empty());
}

#[test]
fn waterbend_taps_artifacts_and_creatures_for_generic_mana() {
    cr!("701.67a");
    ruling!(
        "Geyser Leaper",
        "\"Waterbend [cost]\" means \"Pay [cost]. For each generic mana in that cost, you may tap an untapped artifact or creature you control rather than pay that mana.\""
    );
    supported("Geyser Leaper");
    // "Waterbend {4}: Draw a card, then discard a card."
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Geyser Leaper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Ornithopter");
    let islands = t.lands(P0, "Island", 3);
    choose(&mut t, P0, &[bears, rock]);
    t.activate(P0, leaper, 0, &[]).expect("activate");
    assert!(t.obj_now(bears).tapped && t.obj_now(rock).tapped);
    assert_eq!(islands.iter().filter(|i| t.obj_now(**i).tapped).count(), 2);
    assert!(!t.obj_now(leaper).tapped);
    // Not enough artifacts, creatures, and mana: it can't be activated.
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Geyser Leaper");
    t.lands(P0, "Island", 2);
    assert!(t.activate(P0, leaper, 0, &[]).is_err());
    // With the creature itself, it can: Geyser Leaper taps to help pay.
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Geyser Leaper");
    t.lands(P0, "Island", 3);
    t.activate(P0, leaper, 0, &[]).expect("activate");
    assert!(t.obj_now(leaper).tapped);
}

#[test]
fn only_the_waterbend_cost_can_be_paid_by_tapping() {
    cr!("701.67b");
    ruling!(
        "Water Whip",
        "you may tap an untapped artifact or creature you control rather than pay one generic mana in that total cost any number of times up to a maximum of the amount of generic mana in the waterbend component of that total cost."
    );
    supported("Water Whip");
    // Water Whip: {U}{U}, "As an additional cost to cast this spell, waterbend {5}."
    let whip = |creatures: usize, islands: usize| -> Result<ObjectId, mtg_engine::casting::Illegal> {
        let mut t = TestGame::new(2);
        for _ in 0..creatures {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.lands(P0, "Island", islands);
        let spell = t.hand(P0, "Water Whip");
        t.answer_targets(P0, &[]);
        t.cast(P0, spell).try_go()
    };
    assert!(whip(5, 2).is_ok());
    assert!(whip(4, 3).is_ok());
    // Six creatures can't pay the {U}{U}.
    assert!(whip(6, 1).is_err());
    assert!(whip(4, 2).is_err());
}

#[test]
fn whenever_you_waterbend_triggers_however_the_cost_was_paid() {
    cr!("701.67c");
    ruling!(
        "Avatar Aang // Aang, Master of Elements",
        "An ability that triggers \"whenever you waterbend\" triggers whenever you pay a waterbend cost, regardless of how you paid that cost. You do not have to tap artifacts or creatures to help pay the cost for the ability to trigger."
    );
    let mut t = TestGame::new(2);
    gain_on(&mut t, P0, "waterbend");
    let leaper = t.battlefield(P0, "Geyser Leaper");
    t.lands(P0, "Island", 4);
    // Paid with mana only.
    t.answer_choose(P0, &[]);
    t.activate(P0, leaper, 0, &[]).expect("activate");
    assert!(!t.obj_now(leaper).tapped);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(
        custom_events(&t, WATERBENT_EVENT),
        vec![(Some(P0), None, 4)]
    );
}
