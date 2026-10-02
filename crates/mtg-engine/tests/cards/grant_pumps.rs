//! Pump and type-change phrasing (`src/oracle/patterns/grant_grammar.rs`,
//! `pump_combat_tricks.rs`, `control_exile_battlefield.rs`): a leading duration over
//! several clauses with their own subjects, "gets an additional +N/+N" (CR 613.4c), "has
//! base toughness N" (CR 613.4b), "becomes a Treasure artifact with \"...\" and loses
//! all other card types and abilities" and a card returned as "a Treasure artifact with
//! \"...\"" or "a Skeleton ... and has no abilities" (CR 205.1a, 613.1d, 613.1f).

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn arm_the_cathars_three_targets_three_bonuses() {
    cr!("611.2a", "115.3");
    assert_supported(&["Arm the Cathars"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Arm the Cathars");
    t.cast(P0, s)
        .target(a)
        .target(b)
        .target(c)
        .go();
    t.resolve();
    assert_eq!(t.pt(a), (5, 5));
    assert_eq!(t.pt(b), (4, 4));
    assert_eq!(t.pt(c), (3, 3));
    assert!(t.obj_now(c).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn legion_leadership_doubles_power_and_grants_first_strike() {
    cr!("611.2a", "701.10d");
    assert_supported(&["Legion Leadership // Legion Stronghold"]);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Legion Leadership // Legion Stronghold");
    t.cast(P0, s).target(g).go();
    t.resolve();
    assert_eq!(t.pt(g), (6, 3));
    assert!(t.obj_now(g).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn chariot_of_the_sun_sets_only_base_toughness() {
    cr!("613.4b");
    assert_supported(&["Chariot of the Sun", "Take to the Streets"]);
    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Chariot of the Sun");
    let g = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(g), "+1/+1", 1, None);
    t.lands(P0, "Plains", 2);
    t.activate(P0, ch, 0, &[Entity::Object(g)]).unwrap();
    t.resolve();
    // Base 3/1, plus the counter.
    assert_eq!(t.pt(g), (4, 2));
    assert!(t.obj_now(g).has_keyword(KeywordKind::Flying));
}

#[test]
fn take_to_the_streets_citizens_get_an_additional_bonus() {
    cr!("613.4c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let citizen = t.custom(P0, citizen_def(), mtg_engine::object::Zone::Battlefield);
    t.lands(P0, "Forest", 5);
    let s = t.hand(P0, "Take to the Streets");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(citizen), (4, 4));
    assert!(t.obj_now(citizen).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
}

fn citizen_def() -> CardDef {
    let mut c = mtg_engine::object::Characteristics::default();
    c.name = "Citizen".into();
    c.card_types.insert(CardType::Creature);
    c.subtypes.push("Citizen".into());
    c.power = Some(1);
    c.toughness = Some(1);
    CardDef::custom(c)
}

#[test]
fn xu_ifit_returned_creature_is_a_skeleton_without_abilities() {
    cr!("614.1c", "613.1f");
    ruling!(
        "Xu-Ifit, Osteoharmonist",
        "it will lose that ability before it can trigger"
    );
    assert_supported(&["Xu-Ifit, Osteoharmonist"]);
    let mut t = TestGame::new(2);
    let xu = t.battlefield(P0, "Xu-Ifit, Osteoharmonist");
    let gy = t.graveyard(P0, "Mulldrifter");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.activate(P0, xu, 0, &[Entity::Object(gy)]).unwrap();
    t.resolve_all();
    let m = t.named_on_battlefield("Mulldrifter")[0];
    let o = t.obj_now(m);
    assert!(o.chars.has_subtype("Skeleton") && o.chars.has_subtype("Elemental"));
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.hand_size(P0), hand, "its enters trigger never existed");
}

#[test]
fn vraska_the_silencer_returns_the_card_as_a_treasure() {
    cr!("614.1c", "205.1a", "111.10a");
    ruling!(
        "Vraska, the Silencer",
        "It will retain its supertypes as well as its abilities"
    );
    assert_supported(&["Vraska, the Silencer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vraska, the Silencer");
    t.lands(P0, "Swamp", 1);
    let theirs = t.battlefield(P1, "Mulldrifter");
    t.answer(P0, DecisionKind::YesNo, mtg_engine::decision::Answer::Bool(true));
    t.g.destroy_all(vec![theirs], None, false);
    t.settle();
    t.resolve_all();
    let tr = t.named_on_battlefield("Mulldrifter")[0];
    let o = t.obj_now(tr);
    assert_eq!(o.controller, P0);
    assert!(o.tapped);
    assert!(o.chars.card_types.contains(CardType::Artifact));
    assert!(!o.chars.card_types.contains(CardType::Creature));
    assert!(o.chars.has_subtype("Treasure") && !o.chars.has_subtype("Elemental"));
    // It keeps its other abilities and gains the Treasure's.
    assert!(o.has_keyword(KeywordKind::Flying));
    let mana = o
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .count();
    assert_eq!(mana, 1);
}
