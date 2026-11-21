//! Rulings batch S01 — airbend (CR 701.65): exile the objects; for each card exiled this
//! way, its owner may cast it for {2} rather than its mana cost while it remains exiled.

use crate::r_s01_common::*;
use mtg_engine::kwa::bending::AIRBEND_METHOD;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const AIRBEND: CastMethod = CastMethod::Alternative(AIRBEND_METHOD);

/// P0 casts Airbending Lesson ("Airbend target nonland permanent. Draw a card.") on
/// `target` and resolves it. Returns the airbent object (in exile, if it's a card).
fn airbending_lesson(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Airbending Lesson");
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Airbending Lesson");
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    t.g.current(target)
}

#[test]
fn x_is_zero_when_casting_an_airbent_card() {
    cr!("701.65a", "107.3b");
    ruling!(
        "Airbending Lesson",
        "If the spell you cast has {X} in its mana cost, you must choose 0 as the value of X."
    );
    supported("Endless One");
    let mut t = TestGame::new(2);
    // "This creature enters with X +1/+1 counters on it."
    let one = t.battlefield(P1, "Endless One");
    t.g.add_counters(Entity::Object(one), "+1/+1", 2, None);
    let exiled = airbending_lesson(&mut t, one);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 5);
    let spell = t.cast(P1, exiled).method(AIRBEND).x(3).go();
    assert_eq!(t.g.obj(spell).stack.as_ref().unwrap().cast.x.unwrap_or(0), 0);
    assert_eq!(tapped_lands(&t, P1), 2);
    t.resolve_all();
    // It entered with no counters and died as a 0/0.
    assert!(t.named_on_battlefield("Endless One").is_empty());
    assert!(t.in_graveyard(P1, "Endless One"));
}

#[test]
fn airbent_tokens_cease_to_exist() {
    cr!("701.65a", "704.5d");
    ruling!(
        "Airbending Lesson",
        "Tokens exiled this way will cease to exist and cannot be cast."
    );
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.lands(P1, "Plains", 2);
    let alarm = t.hand(P1, "Raise the Alarm");
    t.cast(P1, alarm).go();
    t.resolve_all();
    let soldier = tokens(&t, P1)[0];
    t.set_step(P0, Step::PrecombatMain);
    let gone = airbending_lesson(&mut t, soldier);
    assert!(!t.on_battlefield(gone));
    assert_eq!(tokens(&t, P1).len(), 1);
    // Nothing is left in exile to cast.
    assert!(t.g.exile.iter().all(|o| !t.g.obj(*o).is_token()));
    assert!(t.g.kwa.airbent.is_empty());
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn airbent_cards_are_cast_for_an_alternative_cost_with_additional_costs() {
    cr!("701.65a", "118.9a", "601.2b");
    ruling!(
        "Airbending Lesson",
        "Since you are using an alternative cost to cast a spell from exile this way, you can't pay any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Kavu Titan");
    supported("Mulldrifter");
    // Kicker: "If this creature was kicked, it enters with three +1/+1 counters on it and
    // with trample."
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P1, "Kavu Titan");
    let exiled = airbending_lesson(&mut t, titan);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 5);
    t.cast(P1, exiled).method(AIRBEND).kicked(true).go();
    assert_eq!(tapped_lands(&t, P1), 5);
    t.resolve_all();
    let titan = t.named_on_battlefield("Kavu Titan")[0];
    assert_eq!(t.pt(titan), (5, 5));
    // Evoke is another alternative cost: casting from exile is offered only for {2}.
    let mut t = TestGame::new(2);
    let drifter = t.battlefield(P1, "Mulldrifter");
    let exiled = airbending_lesson(&mut t, drifter);
    t.set_step(P1, Step::PrecombatMain);
    let options = t.g.cast_options(P1, exiled);
    assert!(!options.is_empty());
    assert!(options.iter().all(|o| o.method == AIRBEND));
}

#[test]
fn an_airbent_spell_with_a_mandatory_additional_cost_needs_it_paid() {
    cr!("701.65a", "601.2h");
    ruling!(
        "Aang, Swift Savior // Aang and La, Ocean's Fury",
        "If the card has any mandatory additional costs, you must pay those."
    );
    supported("Village Rites");
    supported("Aang, Swift Savior // Aang and La, Ocean's Fury");
    let mut t = TestGame::new(2);
    // P1 casts Village Rites ("As an additional cost to cast this spell, sacrifice a
    // creature. Draw two cards."); P0 flashes in Aang and airbends the spell.
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    let rites = t.hand(P1, "Village Rites");
    t.answer_choose(P1, &[Entity::Object(bears)]);
    let spell = t.cast(P1, rites).go();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let aang = t.hand(P0, "Aang, Swift Savior");
    t.cast(P0, aang).go();
    // Aang resolves; its enters trigger targets the spell and resolves.
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.resolve();
    t.resolve();
    let exiled = t.g.current(spell);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.hand_size(P1), 0);
    // Casting it from exile for {2} still requires sacrificing a creature.
    t.lands(P1, "Swamp", 4);
    assert!(t.cast(P1, exiled).method(AIRBEND).try_go().is_err());
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_choose(P1, &[Entity::Object(ogre)]);
    t.cast(P1, exiled).method(AIRBEND).go();
    assert!(!t.on_battlefield(ogre));
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn airbent_lands_cant_be_played_from_exile() {
    cr!("701.65a", "305.1", "305.9");
    ruling!(
        "Airbender Ascension",
        "Lands exiled this way cannot be played from exile."
    );
    supported("Airbender Ascension");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    // "When this enchantment enters, airbend up to one target creature."
    t.lands(P0, "Plains", 2);
    let asc = t.hand(P0, "Airbender Ascension");
    t.cast(P0, asc).go();
    t.answer_targets(P0, &[Entity::Object(arbor)]);
    t.resolve_all();
    let exiled = t.g.current(arbor);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    assert!(t.g.cast_options(P1, exiled).is_empty());
    assert!(t.play_land(P1, exiled).is_err());
    assert_eq!(t.zone(exiled), Zone::Exile);
}

#[test]
fn an_airbent_earthbent_land_returns_tapped() {
    cr!("701.65a", "701.66a");
    ruling!(
        "Airbender Ascension",
        "(Lands that were animated by earthbend will be returned to the battlefield tapped when they're exiled.)"
    );
    supported("Earthbending Lesson");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Wastes");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 4);
    // "Earthbend 4."
    let lesson = t.hand(P1, "Earthbending Lesson");
    t.cast(P1, lesson).target(land).go();
    t.resolve_all();
    assert_eq!(t.pt(land), (4, 4));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    let asc = t.hand(P0, "Airbender Ascension");
    t.cast(P0, asc).go();
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.resolve_all();
    let back = t.g.current(land);
    assert!(t.on_battlefield(back));
    assert!(t.obj(back).tapped);
    assert_eq!(t.obj(back).controller, P1);
    assert!(t.g.exile.is_empty());
}
