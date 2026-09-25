//! CR 304: instants — casting any time with priority, resolving, spell types, never
//! entering the battlefield, and "any time they could cast an instant".

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_instant_can_be_cast_any_time_its_caster_has_priority() {
    cr!("304.1");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    // During the opponent's turn, in combat, with a spell on the stack.
    t.set_step(P1, Step::DeclareBlockers);
    hold_stack(&mut t, P1);
    assert!(can_cast(&mut t, P0, bolt));
    let spell = t.cast(P0, bolt).target(P1).go();
    // Casting it uses the stack: it's on top of the other spell until it resolves.
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_resolving_instant_does_what_it_says_then_goes_to_its_owners_graveyard() {
    cr!("304.2");
    let mut t = TestGame::new(2);
    // P0 casts a Lightning Bolt P1 owns.
    cast_others_card(
        &mut t,
        P0,
        P1,
        "Lightning Bolt",
        "{R}",
        &[Entity::Player(P1)],
    );
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn instant_subtypes_are_spell_types_shared_with_sorceries() {
    cr!("304.3", "307.3");
    // "Instant — Arcane" and "Sorcery — Arcane": the same spell type.
    assert_eq!(subtypes_of("Glacial Ray"), vec!["Arcane"]);
    assert_eq!(subtypes_of("Kodama's Reach"), vec!["Arcane"]);
    for s in ["Arcane", "Lesson", "Adventure", "Trap", "Omen"] {
        assert_eq!(subtype_kind(s), Some(SubtypeKind::Spell), "{s}");
    }
    // Each word after the dash is a separate subtype.
    let tl = TypeLine::parse("Instant — Arcane Lesson");
    assert_eq!(tl.subtypes, vec!["Arcane", "Lesson"]);
    // A filter for Arcane spells matches the instant and the sorcery alike.
    let mut t = TestGame::new(2);
    let ray = t.hand(P0, "Glacial Ray");
    let reach = t.hand(P0, "Kodama's Reach");
    let arcane = Filter::Subtype("Arcane".into());
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(ray, &arcane, &ctx));
    assert!(t.g.matches(reach, &arcane, &ctx));
}

#[test]
fn an_instant_cant_enter_the_battlefield() {
    cr!("304.4");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(bolt)],
    );
    // It remains in its previous zone.
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.g.current(bolt), bolt);
    assert!(t.named_on_battlefield("Lightning Bolt").is_empty());
}

#[test]
fn any_time_they_could_cast_an_instant_means_whenever_they_have_priority() {
    cr!("304.5");
    // Winged Coatl has flash: "You may cast this spell any time you could cast an instant."
    // An effect that stops P0 from casting instant spells doesn't stop that.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        CB::new("Instant Ban")
            .enchantment()
            .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
                who: PlayerFilter::Opponent,
                what: Filter::Type(CardType::Instant),
            })))
            .build(),
        Zone::Battlefield,
    );
    let coatl = t.hand(P0, "Winged Coatl");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana_for(&mut t, P0, "{1}{G}{U}{R}");
    t.set_step(P1, Step::End);
    hold_stack(&mut t, P1);
    assert!(!can_cast(&mut t, P0, bolt), "no instant spells");
    assert!(can_cast(&mut t, P0, coatl));
    t.cast(P0, coatl).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Winged Coatl").len(), 1);
}
