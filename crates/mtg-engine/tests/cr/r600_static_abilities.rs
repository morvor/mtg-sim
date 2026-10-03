//! CR 604: handling static abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn lose_all_abilities_enchantments() -> CardDef {
    CB::new("Enchantment Silencer")
        .artifact()
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::Type(CardType::Enchantment),
            mods: vec![Modification::RemoveAllAbilities],
        }))
        .build()
}

#[test]
fn static_abilities_are_simply_true_while_their_source_is_on_the_battlefield() {
    cr!("604.1", "604.2");
    // Glorious Anthem: "Creatures you control get +1/+1."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let opp = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    // No activation or trigger: it just applies.
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(opp), (2, 2));
    assert_eq!(t.stack_len(), 0);
    // A creature entering later is affected as soon as it's there.
    let later = t.enter(P0, "Craw Wurm");
    t.g.recompute();
    assert_eq!(t.pt(later), (7, 5));
    // It stops applying if the permanent loses the ability...
    let silencer = t.custom(P1, lose_all_abilities_enchantments(), Zone::Battlefield);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    t.g.destroy(silencer, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // ... or leaves the battlefield.
    t.g.destroy(anthem, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn static_abilities_can_create_replacement_and_prevention_effects() {
    cr!("604.2");
    // "Creatures your opponents control enter tapped." (a replacement effect) and
    // "Prevent all damage that would be dealt to you." (a prevention effect).
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        CB::new("Tapped Arrivals")
            .enchantment()
            .ability(stat(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::creature().opp_controls()),
                action: ReplacementAction::EnterTapped,
                self_replacement: false,
                optional: false,
            })))
            .build(),
        Zone::Battlefield,
    );
    let c = t.enter(P1, "Grizzly Bears");
    assert!(t.obj(c).tapped);
    let mine = t.enter(P0, "Grizzly Bears");
    assert!(!t.obj(mine).tapped);
}

#[test]
fn characteristic_defining_abilities_function_in_all_zones() {
    cr!("604.3");
    // Nightmare: "Nightmare's power and toughness are each equal to the number of Swamps
    // you control."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let in_hand = t.hand(P0, "Nightmare");
    let in_gy = t.graveyard(P0, "Nightmare");
    let in_exile = t.exile(P0, "Nightmare");
    let in_library = t.library_top(P0, "Nightmare");
    t.g.recompute();
    for id in [in_hand, in_gy, in_exile, in_library] {
        assert_eq!(t.pt(id), (3, 3), "{:?}", t.obj(id).zone);
    }
    // Transguild Courier: "Transguild Courier is all colors."
    let c = t.hand(P0, "Transguild Courier");
    let l = t.library_top(P0, "Transguild Courier");
    t.g.recompute();
    assert_eq!(t.obj(c).chars.colors, ColorSet::ALL);
    assert_eq!(t.obj(l).chars.colors, ColorSet::ALL);
    // On the stack too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let n = t.hand(P0, "Nightmare");
    t.cast(P0, n).go();
    t.g.recompute();
    assert_eq!(t.pt(n), (6, 6));
}

#[test]
fn what_is_and_isnt_a_characteristic_defining_ability() {
    cr!("604.3a");
    let is_cda = |def: &CardDef| {
        abilities(def)
            .iter()
            .any(|a| matches!(&a.kind, AbilityKind::Static(s) if s.is_cda))
    };
    // (1)+(2): defines its own power/toughness or colors, printed on the card.
    assert!(is_cda(&card("Nightmare")));
    assert!(is_cda(&card("Transguild Courier")));
    // (3): it directly affects other objects' characteristics.
    let other = compile_def(
        "Painter Test",
        "Creature — Human",
        "{1}",
        "Creatures you control are black.",
    );
    assert!(!is_cda(&other));
    // (5): it sets the values only if a condition is met.
    let conditional = compile_def(
        "Conditional Test",
        "Creature — Human",
        "{1}",
        "As long as you control a Swamp, Conditional Test gets +2/+2.",
    );
    assert!(!is_cda(&conditional));
    // A CDA applies in layer 7a, before effects that set base power and toughness (7b)
    // regardless of timestamps, and before modifications (7c).
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let n = t.battlefield(P0, "Nightmare");
    let shrink = CB::new("Shrink Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))],
                duration: Duration::EndOfTurn,
            },
        ))
        .build();
    let s = t.custom(P0, shrink, Zone::Hand(P0));
    t.cast(P0, s).target(n).go();
    t.resolve_all();
    assert_eq!(t.pt(n), (1, 1));
    // A land entering later doesn't make the CDA win over the setting effect.
    t.enter(P0, "Swamp");
    t.g.recompute();
    assert_eq!(t.pt(n), (1, 1));
    t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(n), (2, 2));
}

#[test]
fn equipment_modifies_the_equipped_creature_without_targeting_it() {
    cr!("604.4");
    // Bonesplitter: "Equipped creature gets +2/+0."
    let mut t = TestGame::new(2);
    let shrouded = t.custom(
        P0,
        CB::new("Shrouded Beast")
            .creature(2, 2)
            .keyword(KeywordKind::Shroud)
            .build(),
        Zone::Battlefield,
    );
    let other = t.battlefield(P0, "Grizzly Bears");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.g.attach(eq, Entity::Object(shrouded));
    t.g.recompute();
    // It can't be targeted, but the static ability doesn't target.
    assert_eq!(t.pt(shrouded), (4, 2));
    // Moved: it stops applying to the original and applies to the new creature.
    t.lands(P0, "Plains", 1);
    t.activate(P0, eq, 0, &[Entity::Object(other)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(shrouded), (2, 2));
    assert_eq!(t.pt(other), (4, 2));
}

#[test]
fn some_static_abilities_apply_while_a_spell_is_on_the_stack() {
    cr!("604.5");
    // Carnage Tyrant: "This spell can't be countered."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let ct = t.hand(P0, "Carnage Tyrant");
    let spell = t.cast(P0, ct).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Carnage Tyrant").len(), 1);
    assert!(t.in_graveyard(P1, "Counterspell"));
}

#[test]
fn some_static_abilities_apply_in_zones_a_card_could_be_cast_from() {
    cr!("604.6");
    // Gravecrawler: "You may cast Gravecrawler from your graveyard as long as you control
    // a Zombie."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let g = t.graveyard(P0, "Gravecrawler");
    assert!(t.cast(P0, g).try_go().is_err());
    t.battlefield(P0, "Walking Corpse");
    let spell = t.cast(P0, g).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gravecrawler").len(), 1);
    // "Cast this spell only during combat."
    let def = compile_def(
        "Combat Trick Test",
        "Instant",
        "{0}",
        "Cast this spell only during combat.\nDraw a card.",
    );
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def, Zone::Hand(P0));
    assert!(t.cast(P0, c).try_go().is_err());
    t.advance_to(P0, Step::BeginningOfCombat);
    let hand = t.hand_size(P0);
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn static_abilities_dont_use_last_known_information() {
    cr!("604.7");
    // Goblin King: "Other Goblins get +1/+1 and have mountainwalk." A 1/1 Goblin with
    // 1 damage survives while it's 2/2, but not once Goblin King is gone: the ability
    // doesn't keep applying based on Goblin King's last known information.
    let mut t = TestGame::new(2);
    let king = t.battlefield(P0, "Goblin King");
    let g = t.battlefield(P0, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(g), (2, 2));
    t.g.deal_damage(king, Entity::Object(g), 1, false);
    t.settle();
    assert!(t.on_battlefield(g));
    t.g.move_object(king, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.settle();
    assert!(!t.on_battlefield(g));
    assert!(t.in_graveyard(P0, "Raging Goblin"));
}
