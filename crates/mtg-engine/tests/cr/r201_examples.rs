//! The worked examples of CR 201 (names), played out as tests. The CR 201.2b and 201.5a
//! examples are cited by the tests in `r201_names.rs` and
//! `cards/grant_granter_tokens.rs`.

use crate::r200_common::name_card;
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The activated abilities' texts of `id`.
fn activated(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect()
}

/// The index of `id`'s activated ability whose text contains `needle`.
fn index_of(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    activated(t, id)
        .iter()
        .position(|a| a.contains(needle))
        .unwrap_or_else(|| panic!("no ability with {needle:?}: {:?}", activated(t, id)))
}

fn cards_named(t: &TestGame, zone: Zone, name: &str) -> usize {
    t.g.objects
        .iter()
        .filter(|o| t.g.is_live(o.id) && o.zone == zone && o.chars.name == name)
        .count()
}

/// P0 casts Dispossess ("Choose an artifact card name. Search target opponent's
/// graveyard, hand, and library for any number of cards with the chosen name and exile
/// them. Then that player shuffles.") naming `name`, with a Mycosynth Lattice making an
/// Island on the battlefield an artifact.
fn dispossess_naming(name: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Mycosynth Lattice");
    let island = t.battlefield(P1, "Island");
    t.g.recompute();
    assert!(t.obj_now(island).chars.is(CardType::Artifact));
    t.hand(P1, "Black Lotus");
    t.graveyard(P1, "Black Lotus");
    t.hand(P1, "Island");
    t.graveyard(P1, "Island");
    t.lands(P0, "Swamp", 3);
    let d = t.hand(P0, "Dispossess");
    let spell = t.cast(P0, d).target(P1).go();
    name_card(&mut t, P0, name);
    t.resolve();
    (t, spell)
}

#[test]
fn an_artifact_card_name_is_any_artifact_cards_name_but_not_an_artifact_lands() {
    cr!("201.4", "201.4a");
    cr_example!("201.4a", "Dispossess");
    // Black Lotus (legal in no format but Vintage) is an artifact card name.
    let (t, _) = dispossess_naming("Black Lotus");
    assert_eq!(cards_named(&t, Zone::Exile, "Black Lotus"), 2);
    assert_eq!(cards_named(&t, Zone::Hand(P1), "Black Lotus"), 0, "{}", t.dump_log());
    // Island isn't, though an Island on the battlefield is an artifact: nothing named
    // Island is exiled.
    let (t, _) = dispossess_naming("Island");
    assert_eq!(cards_named(&t, Zone::Exile, "Island"), 0);
    assert_eq!(cards_named(&t, Zone::Hand(P1), "Island"), 1);
    assert_eq!(cards_named(&t, Zone::Graveyard(P1), "Island"), 1);
}

#[test]
fn a_gained_regenerate_ability_naming_skithiryx_regenerates_quicksilver_elemental() {
    cr!("201.5b");
    cr_example!("201.5b", "Quicksilver Elemental");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let q = t.battlefield(P0, "Quicksilver Elemental");
    let sk = t.battlefield(P1, "Skithiryx, the Blight Dragon");
    // "{U}: This creature gains all activated abilities of target creature until end of
    // turn."
    t.lands(P0, "Island", 1);
    let gain = index_of(&t, q, "{U}");
    t.activate(P0, q, gain, &[Entity::Object(sk)]).unwrap();
    t.resolve();
    t.g.recompute();
    // "{B}{B}: Regenerate Skithiryx." (paid with blue mana).
    t.lands(P0, "Island", 2);
    let regen = index_of(&t, q, "Regenerate");
    t.activate(P0, q, regen, &[]).unwrap();
    t.resolve();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: false,
        },
        &[],
    );
    t.settle();
    // Quicksilver Elemental regenerated; Skithiryx didn't.
    assert!(t.g.is_live(q), "{}", t.dump_log());
    assert!(t.obj_now(q).tapped);
    assert!(!t.g.is_live(sk));
    assert!(t.in_graveyard(P1, "Skithiryx, the Blight Dragon"));
}

#[test]
fn glacial_ray_spliced_onto_kodamas_reach_makes_kodamas_reach_deal_the_damage() {
    cr!("201.5b", "702.47a");
    cr_example!("201.5b", "Kodama's Reach");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 2);
    let reach = t.hand(P0, "Kodama's Reach");
    let ray = t.hand(P0, "Glacial Ray");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P0, reach).target(bear).go();
    assert!(t.in_hand(P0, "Glacial Ray"));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let dealt: Vec<(ObjectId, ObjectId)> = t.g.history.damage_by_source.iter().copied().collect();
    assert!(
        dealt.iter().any(|(s, d)| *s == spell && *d == bear),
        "Kodama's Reach dealt the damage: {dealt:?}"
    );
    assert!(!dealt.iter().any(|(s, _)| *s == ray));
    // It's still Kodama's Reach: a land was put onto the battlefield.
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.name == "Forest")
            .count(),
        4
    );
}

#[test]
fn the_ever_changing_dane_as_runeclaw_bear_keeps_a_copy_ability_naming_itself() {
    cr!("201.5b", "707.9b");
    cr_example!("201.5b", "The Ever-Changing 'Dane says");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    let bear = t.battlefield(P0, "Runeclaw Bear");
    let giant = t.battlefield(P0, "Hill Giant");
    // Another Runeclaw Bear: the ability refers only to the 'Dane.
    let other_bear = t.battlefield(P1, "Runeclaw Bear");
    t.lands(P0, "Wastes", 2);
    let ab = index_of(&t, dane, "Sacrifice another creature");
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.activate(P0, dane, ab, &[]).unwrap();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.obj_now(dane).chars.name, "Runeclaw Bear");
    assert_eq!(t.pt(dane), (2, 2));
    // Its ability: "{1}, Sacrifice another creature: Runeclaw Bear becomes a copy of the
    // sacrificed creature, except it has this ability."
    let ab = index_of(&t, dane, "Sacrifice another creature");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, dane, ab, &[]).unwrap();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.obj_now(dane).chars.name, "Hill Giant");
    assert_eq!(t.pt(dane), (3, 3));
    assert!(activated(&t, dane)
        .iter()
        .any(|a| a.contains("Sacrifice another creature")));
    // The other Runeclaw Bear is unaffected.
    assert_eq!(t.obj_now(other_bear).chars.name, "Runeclaw Bear");
    assert_eq!(t.pt(other_bear), (2, 2));
    assert!(activated(&t, other_bear).is_empty());
}
