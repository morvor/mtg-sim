//! CR 702.44 Sunburst.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::*;
use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;
use smol_str::SmolStr;

fn p1p1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

fn charge(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::CHARGE)
}

/// Casts `name` (a permanent spell) for P0, resolves it, and returns the permanent.
fn cast_permanent(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.hand(P0, name);
    t.cast(P0, c).go();
    t.resolve_all();
    *t.named_on_battlefield(name).last().expect("not on the battlefield")
}

#[test]
fn a_creature_enters_with_a_counter_for_each_color_spent() {
    cr!("702.44", "702.44a");
    assert_supported("Suntouched Myr");
    let mut t = TestGame::new(2);
    // Suntouched Myr {3}: paid with {W}{U}{B}.
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let myr = cast_permanent(&mut t, "Suntouched Myr");
    assert_eq!(p1p1(&t, myr), 3);
    assert_eq!(charge(&t, myr), 0);
    assert_eq!(t.pt(myr), (3, 3));
}

#[test]
fn colors_count_once_and_colorless_mana_doesnt_count() {
    cr!("702.44a");
    ruling!(
        "Engineered Explosives",
        "Colorless mana won't give Engineered Explosives another charge counter. Colorless is not a color."
    );
    let mut t = TestGame::new(2);
    // {3} paid with {G}{G}{C}: one color.
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 1);
    let myr = cast_permanent(&mut t, "Suntouched Myr");
    assert_eq!(p1p1(&t, myr), 1);
}

#[test]
fn a_noncreature_enters_with_charge_counters() {
    cr!("702.44a");
    assert_supported("Pentad Prism");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let prism = cast_permanent(&mut t, "Pentad Prism");
    assert_eq!(charge(&t, prism), 2);
    assert_eq!(p1p1(&t, prism), 0);
}

#[test]
fn type_changing_effects_are_ignored() {
    cr!("702.44a");
    // An enchantment that makes each noncreature artifact a 2/2 artifact creature: Pentad
    // Prism enters as a creature because of it, but sunburst ignores that.
    let animator = CardDef::custom(Characteristics {
        name: SmolStr::new("Artifact Animator"),
        card_types: [CardType::Enchantment].into_iter().collect(),
        abilities: vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::And(vec![
                    Filter::Type(CardType::Artifact),
                    Filter::not(Filter::Type(CardType::Creature)),
                ]),
                mods: vec![
                    Modification::AddTypes(vec![CardType::Creature]),
                    Modification::SetPT(Some(Value::c(2)), Some(Value::c(2))),
                ],
            })),
            "Each noncreature artifact is a 2/2 artifact creature.",
        )],
        ..Default::default()
    });
    let mut t = TestGame::new(2);
    t.custom(P0, animator, Zone::Battlefield);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let prism = cast_permanent(&mut t, "Pentad Prism");
    assert!(t.obj_now(prism).is_creature());
    assert_eq!(charge(&t, prism), 2);
    assert_eq!(p1p1(&t, prism), 0);
    assert_eq!(t.pt(prism), (2, 2));
}

#[test]
fn only_a_resolving_spell_gets_sunburst_counters() {
    cr!("702.44b");
    let mut t = TestGame::new(2);
    // Put onto the battlefield from the graveyard: no mana was spent to cast it.
    let card = t.graveyard(P0, "Pentad Prism");
    t.g.move_object(card, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    let prism = t.named_on_battlefield("Pentad Prism")[0];
    assert_eq!(charge(&t, prism), 0);
    // Cast with colorless mana only: no colored mana was spent.
    t.lands(P0, "Wastes", 2);
    let prism2 = cast_permanent(&mut t, "Pentad Prism");
    assert_ne!(prism, prism2);
    assert_eq!(charge(&t, prism2), 0);
}

#[test]
fn x_lets_more_colors_be_spent() {
    cr!("702.44b");
    ruling!(
        "Engineered Explosives",
        "The value chosen for X doesn't directly affect the number of charge counters Engineered Explosives enters the battlefield with, but it does let you pay more mana and thus spend more colors of mana to cast it."
    );
    assert_supported("Engineered Explosives");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let ee = t.hand(P0, "Engineered Explosives");
    t.cast(P0, ee).x(3).go();
    t.resolve_all();
    let ee = t.named_on_battlefield("Engineered Explosives")[0];
    assert_eq!(charge(&t, ee), 2);
}

#[test]
fn mana_spent_on_additional_costs_counts() {
    cr!("702.44b");
    // A kicker cost paid with {R} adds red to the colors spent.
    let def = with_cost(
        custom_card(
            "Kicked Sunburst Golem",
            "Artifact Creature — Golem",
            Some((0, 0)),
            "Kicker {R}\nSunburst",
        ),
        "{1}",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Mountain", 1);
    let g = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, g).kicked(true).go();
    t.resolve_all();
    let g = t.named_on_battlefield("Kicked Sunburst Golem")[0];
    assert_eq!(p1p1(&t, g), 1);
}

#[test]
fn modular_sunburst_sets_the_number_of_counters() {
    cr!("702.44c", "702.43a");
    ruling!(
        "Arcbound Wanderer",
        "The sunburst ability sets the number for the modular ability."
    );
    ruling!(
        "Arcbound Wanderer",
        "the number of counters on Arcbound Wanderer at the time it left the battlefield is what's relevant"
    );
    assert_supported("Arcbound Wanderer");
    let mut t = TestGame::new(2);
    // Arcbound Wanderer {6} paid with {W}{U}{B}{R} and two colorless.
    for land in ["Plains", "Island", "Swamp", "Mountain"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 2);
    let w = cast_permanent(&mut t, "Arcbound Wanderer");
    assert_eq!(p1p1(&t, w), 4);
    let worker = t.enter(P0, "Arcbound Worker");
    t.g.add_counters(Entity::Object(w), counters::PLUS1, 1, None);
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.answer_yes(P0, true);
    t.g.destroy(w, None);
    t.resolve_all();
    assert_eq!(p1p1(&t, worker), 1 + 5);
}

#[test]
fn modular_sunburst_gives_plus_one_counters_even_to_a_noncreature() {
    cr!("702.44c");
    let def = with_cost(
        custom_card("Sunburst Relic", "Artifact", None, "Modular—Sunburst"),
        "{2}",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let r = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, r).go();
    t.resolve_all();
    let r = t.named_on_battlefield("Sunburst Relic")[0];
    assert_eq!(p1p1(&t, r), 2);
    assert_eq!(charge(&t, r), 0);
}

#[test]
fn two_solar_arrays_give_the_next_artifact_spell_two_instances() {
    cr!("702.44d");
    ruling!(
        "Solar Array",
        "If a spell has multiple instances of sunburst, each one applies. For example, if you activate the abilities of two Solar Arrays and then cast Selfcraft Mechan (an artifact creature), it will have two instances of sunburst and will thus enter with two +1/+1 counters on it for each color of mana spent to cast it."
    );
    assert_supported("Solar Array");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Solar Array");
    let a2 = t.battlefield(P0, "Solar Array");
    // "{T}: Add one mana of any color. When you next cast an artifact spell this turn,
    // that spell gains sunburst."
    t.activate(P0, a1, 0, &[]).unwrap();
    t.activate(P0, a2, 0, &[]).unwrap();
    let mut colors: Vec<mana::ManaType> = t
        .g
        .player(P0)
        .mana_pool
        .mana
        .iter()
        .map(|m| m.ty)
        .collect();
    assert_eq!(colors.len(), 2);
    colors.push(mana::ManaType::U);
    colors.sort();
    colors.dedup();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    // Selfcraft Mechan {3}{U}.
    let mechan = t.hand(P0, "Selfcraft Mechan");
    t.cast(P0, mechan).go();
    // (Its own enters ability: don't sacrifice an artifact.)
    t.answer_yes(P0, false);
    t.resolve_all();
    let m = t.named_on_battlefield("Selfcraft Mechan")[0];
    assert_eq!(p1p1(&t, m), 2 * colors.len() as u32);
}

#[test]
fn each_instance_of_sunburst_works_separately() {
    cr!("702.44d");
    let def = with_cost(
        custom_card(
            "Double Sunburst Myr",
            "Artifact Creature — Myr",
            Some((0, 0)),
            "Sunburst\nSunburst",
        ),
        "{2}",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let m = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, m).go();
    t.resolve_all();
    let m = t.named_on_battlefield("Double Sunburst Myr")[0];
    assert_eq!(p1p1(&t, m), 4);
}

/// The distinct colors of mana spent to cast `spell`.
fn colors_spent(t: &TestGame, spell: ObjectId) -> u32 {
    let mut colors: Vec<types::Color> = t
        .obj_now(spell)
        .stack
        .as_ref()
        .expect("a spell")
        .cast
        .mana_spent
        .iter()
        .filter_map(|m| m.color())
        .collect();
    colors.sort();
    colors.dedup();
    colors.len() as u32
}

#[test]
fn solar_arrays_mana_spent_on_an_artifact_spell_gives_it_sunburst() {
    cr!("702.44a", "603.7");
    ruling!(
        "Solar Array",
        "Sunburst checks what mana was actually spent to cast the spell."
    );
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Solar Array");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    // Selfcraft Mechan {3}{U}: Solar Array is tapped for mana while its costs are paid.
    // The spell is the next artifact spell cast after the ability resolved, so its
    // delayed triggered ability triggers.
    let mechan = t.hand(P0, "Selfcraft Mechan");
    let spell = t.cast(P0, mechan).go();
    assert!(t.obj_now(array).tapped);
    let n = colors_spent(&t, spell);
    assert!(n >= 2);
    t.answer_yes(P0, false);
    t.resolve_all();
    let m = t.named_on_battlefield("Selfcraft Mechan")[0];
    assert_eq!(p1p1(&t, m), n);
}

#[test]
fn solar_arrays_delayed_trigger_lasts_only_this_turn() {
    cr!("702.44a", "603.7b");
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Solar Array");
    t.activate(P0, array, 0, &[]).unwrap();
    // The mana empties; on P0's next turn, the next artifact spell doesn't gain sunburst.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let myr = t.hand(P0, "Suntouched Myr");
    // Suntouched Myr has its own sunburst: one instance, two colors.
    let spell = t.cast(P0, myr).go();
    assert_eq!(colors_spent(&t, spell), 2);
    t.resolve_all();
    let m = t.named_on_battlefield("Suntouched Myr")[0];
    assert_eq!(p1p1(&t, m), 2);
}
