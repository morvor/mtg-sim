//! CR 306: planeswalkers — casting and resolving, planeswalker types, the legend rule
//! instead of a uniqueness rule, loyalty, loyalty abilities, attacking planeswalkers, and
//! damage to them.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn planeswalker_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("306.1");
    check_sorcery_timing("Jace Beleren", "{1}{U}{U}");
}

#[test]
fn a_planeswalker_spell_resolves_onto_the_battlefield_under_its_controllers_control() {
    cr!("306.2", "306.5b");
    let mut t = TestGame::new(2);
    let jace = cast_others_card(&mut t, P0, P1, "Jace Beleren", "{1}{U}{U}", &[]);
    assert!(t.on_battlefield(jace));
    assert_eq!((t.obj(jace).controller, t.obj(jace).owner), (P0, P1));
    assert_eq!(t.obj(jace).loyalty(), 3);
}

#[test]
fn planeswalker_subtypes_are_planeswalker_types() {
    cr!("306.3");
    assert_eq!(subtypes_of("Jace Beleren"), vec!["Jace"]);
    assert_eq!(subtypes_of("Garruk Wildspeaker"), vec!["Garruk"]);
    for s in ["Jace", "Garruk", "Chandra", "Ajani"] {
        assert_eq!(subtype_kind(s), Some(SubtypeKind::Planeswalker), "{s}");
    }
}

#[test]
fn planeswalkers_of_the_same_type_coexist_but_the_legend_rule_applies() {
    cr!("306.4");
    let mut t = TestGame::new(2);
    // Two different Jaces: no uniqueness rule.
    let a = t.battlefield(P0, "Jace Beleren");
    let b = t.battlefield(P0, "Jace, the Mind Sculptor");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // Two Jace Beleren: they're legendary (Oracle errata), so the legend rule applies.
    assert!(t.obj(a).chars.supertypes.contains(Supertype::Legendary));
    let c = t.battlefield(P0, "Jace Beleren");
    t.settle();
    assert_eq!(t.named_on_battlefield("Jace Beleren").len(), 1);
    assert!(t.in_graveyard(P0, "Jace Beleren"));
    assert!(t.on_battlefield(b));
    let _ = c;
}

#[test]
fn only_planeswalkers_have_loyalty() {
    cr!("306.5");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.obj(bears).chars.loyalty, None);
    assert_eq!(t.obj(jace).chars.loyalty, Some(3));
    // A planeswalker that stops being a planeswalker has no loyalty: it isn't put into
    // the graveyard with no loyalty counters.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(jace)],
    );
    assert_eq!(t.obj(jace).chars.loyalty, None);
    t.g.remove_counters(Entity::Object(jace), counters::LOYALTY, 3);
    t.settle();
    assert!(t.on_battlefield(jace));
}

#[test]
fn a_planeswalker_cards_loyalty_off_the_battlefield_is_its_printed_number() {
    cr!("306.5a");
    let mut t = TestGame::new(2);
    let jace = t.hand(P0, "Jace Beleren");
    let garruk = t.graveyard(P0, "Garruk Wildspeaker");
    assert_eq!(t.obj(jace).chars.loyalty, Some(3));
    assert_eq!(t.obj(garruk).chars.loyalty, Some(3));
}

#[test]
fn a_planeswalkers_loyalty_on_the_battlefield_is_its_loyalty_counters() {
    cr!("306.5c");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    assert_eq!(t.obj(jace).loyalty(), 3);
    t.g.add_counters(Entity::Object(jace), counters::LOYALTY, 2, None);
    assert_eq!(t.obj(jace).loyalty(), 5);
    t.g.remove_counters(Entity::Object(jace), counters::LOYALTY, 4);
    assert_eq!(t.obj(jace).loyalty(), 1);
}

#[test]
fn loyalty_abilities_are_activated_as_a_sorcery_once_per_turn() {
    cr!("306.5d");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    // Not during the opponent's turn, nor with a nonempty stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    hold_stack(&mut t, P0);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    t.resolve_all();
    // +2: Each player draws a card.
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj(jace).loyalty(), 5);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1 + 1));
    // Only one loyalty ability of that permanent each turn.
    assert!(t.activate(P0, jace, 1, &[Entity::Player(P0)]).is_err());
    // Next turn it can be activated again.
    t.advance_to(P0, Step::Upkeep);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, jace, 1, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.obj(jace).loyalty(), 4);
}

#[test]
fn planeswalkers_can_be_attacked_and_damage_removes_loyalty_counters() {
    cr!("306.6", "306.8");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.add_counters(Entity::Object(jace), counters::LOYALTY, 2, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Object(jace))], &[]);
    assert_eq!(t.obj(jace).loyalty(), 3);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj(jace).damage, 0, "no damage is marked on it");
    // Noncombat damage too.
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(jace).go();
    t.resolve();
    assert_eq!(t.obj(jace).loyalty(), 1);
}

#[test]
fn damage_to_a_player_isnt_redirected_to_their_planeswalker() {
    cr!("306.7");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let asked = t.asked().len();
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj(jace).loyalty(), 3);
    // No choice to redirect the damage was offered.
    assert!(t.asked()[asked..].iter().all(|(_, d)| !matches!(
        d,
        mtg_engine::decision::Decision::ChooseEntities { .. }
            | mtg_engine::decision::Decision::YesNo { .. }
    )));
    // Damage is dealt to a planeswalker directly by targeting it.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(jace).go();
    t.resolve();
    assert!(!t.on_battlefield(jace));
}

#[test]
fn a_planeswalker_with_zero_loyalty_is_put_into_its_owners_graveyard() {
    cr!("306.9");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    deal_damage(&mut t, P0, Entity::Object(jace), 2);
    t.settle();
    assert!(t.on_battlefield(jace));
    // -1: Target player draws a card — using the last loyalty counter.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.g.remove_counters(Entity::Object(jace), counters::LOYALTY, 2);
    t.activate(P0, jace, 1, &[Entity::Player(P0)]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Jace Beleren"));
    // Its ability still resolves.
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    let _ = Zone::Battlefield;
}
