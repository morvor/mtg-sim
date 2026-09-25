//! CR 603.6: zone-change triggers (enters- and leaves-the-battlefield abilities).

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn put_onto_battlefield(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    let id =
        t.g.create_card_object(std::sync::Arc::new(def), p, Zone::Nowhere);
    t.g.move_object_ev(MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    })
    .unwrap()
}

fn destroy(t: &mut TestGame, id: ObjectId) {
    t.g.destroy(id, None);
}

#[test]
fn zone_change_triggers_look_for_the_object_in_the_zone_it_moved_to() {
    cr!("603.6");
    // "When this creature dies, return it to its owner's hand."
    let phoenix = || {
        CB::new("Returning Beast")
            .creature(2, 2)
            .ability(trig(
                TriggerCond::Dies(Filter::Source),
                Body::effect(Effect::Move {
                    what: Sel::TriggerObject,
                    to: Destination::zone(ZoneKind::Hand),
                }),
            ))
            .build()
    };
    let mut t = TestGame::new(2);
    let b = t.custom(P0, phoenix(), Zone::Battlefield);
    destroy(&mut t, b);
    t.settle();
    t.resolve();
    assert!(t.in_hand(P0, "Returning Beast"));
    // If the card leaves the graveyard before the ability resolves, it can't be found,
    // even if it comes back to the graveyard again.
    let mut t = TestGame::new(2);
    let b = t.custom(P0, phoenix(), Zone::Battlefield);
    destroy(&mut t, b);
    t.settle();
    let in_gy = t.g.current(b);
    let exiled = t.g.exile_object(in_gy, None).unwrap();
    t.g.move_object(exiled, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.resolve();
    assert!(!t.in_hand(P0, "Returning Beast"));
    assert!(t.in_graveyard(P0, "Returning Beast"));
}

#[test]
fn a_leaves_the_battlefield_ability_checks_only_the_first_zone_it_went_to() {
    cr!("603.6", "603.6c");
    // "When this leaves the battlefield, exile it."
    let def = CB::new("Exiled Wanderer")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::LeavesBattlefield(Filter::Source),
            Body::effect(Effect::Exile {
                what: Sel::TriggerObject,
                face_down: false,
                link: false,
            }),
        ))
        .build();
    let mut t = TestGame::new(2);
    let w = t.custom(P0, def.clone(), Zone::Battlefield);
    // Went to the graveyard: found there and exiled.
    destroy(&mut t, w);
    t.resolve_all();
    assert!(t.in_exile("Exiled Wanderer"));
    // Went to a zone hidden from the ability's controller (the owner's hand): it can't be
    // found there.
    let w = t.custom(P1, def.clone(), Zone::Battlefield);
    t.g.objects[w.0 as usize].base_controller = P0;
    t.g.recompute();
    t.g.move_object(w, Zone::Hand(P1), MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_hand(P1, "Exiled Wanderer"));
    // Likewise a library.
    let w = t.custom(P0, def, Zone::Battlefield);
    t.g.move_object(w, Zone::Library(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(w)).zone, Zone::Library(P0));
}

#[test]
fn every_permanent_is_checked_including_those_entering_simultaneously() {
    cr!("603.6a");
    let mut t = TestGame::new(2);
    // Two tokens enter at once, each with "Whenever another creature enters, you gain 1 life."
    let spec = TokenSpec {
        name: "Greeter".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![trig(
            TriggerCond::EntersBattlefield(Filter::creature().other()),
            Body::effect(gain(1)),
        )],
        scryfall_name: None,
    };
    let maker = CB::new("Greeter Maker")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::CreateToken {
            spec,
            count: Value::c(2),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        }))
        .build();
    let m = t.custom(P0, maker, Zone::Hand(P0));
    t.cast(P0, m).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Greeter").len(), 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn continuous_effects_apply_the_moment_a_permanent_enters() {
    cr!("603.6b", "603.10");
    // "All lands are creatures": a land entering is a creature entering.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        CB::new("Animator")
            .enchantment()
            .ability(stat(StaticEffect::Continuous {
                affected: Filter::Type(CardType::Land),
                mods: vec![
                    Modification::AddTypes(vec![CardType::Creature]),
                    Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
                ],
            }))
            .build(),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        CB::new("Creature Watcher")
            .enchantment()
            .ability(trig(
                TriggerCond::EntersBattlefield(Filter::creature()),
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // "All creatures lose all abilities": an entering creature's ETB ability never exists.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        CB::new("Humility Test")
            .enchantment()
            .ability(stat(StaticEffect::Continuous {
                affected: Filter::creature(),
                mods: vec![Modification::RemoveAllAbilities],
            }))
            .build(),
        Zone::Battlefield,
    );
    let etb = CB::new("ETB Gainer")
        .creature(1, 1)
        .ability(trig(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(gain(5)),
        ))
        .build();
    put_onto_battlefield(&mut t, P0, etb);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn from_anywhere_triggers_are_not_leaves_the_battlefield_abilities() {
    cr!("603.6c", "603.10a");
    // A creature with "Whenever a creature card is put into a graveyard from anywhere,
    // you gain 1 life" dies along with another creature: it doesn't look back in time,
    // so it doesn't see either event.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        CB::new("Anywhere Watcher")
            .creature(1, 1)
            .ability(trig(
                TriggerCond::ZoneChange {
                    filter: Filter::creature(),
                    from: None,
                    to: Some(ZoneKind::Graveyard),
                },
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    // A leaves-the-battlefield watcher dying in the same event does see both.
    t.custom(
        P1,
        CB::new("Death Watcher")
            .creature(1, 1)
            .ability(trig(
                TriggerCond::Dies(Filter::creature()),
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    t.battlefield(P0, "Grizzly Bears");
    let wrath = CB::new("Wrath Test")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: true,
        }))
        .build();
    let w = t.custom(P0, wrath, Zone::Hand(P0));
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 23);
}

#[test]
fn a_permanent_leaving_the_game_with_its_owner_triggers_leaves_the_battlefield_abilities() {
    cr!("603.6c");
    let mut t = TestGame::new(3);
    // P0 controls a creature owned by P2 with "When this leaves the battlefield, you gain
    // 3 life."
    let leaver = CB::new("Borrowed Leaver")
        .creature(1, 1)
        .ability(trig(
            TriggerCond::LeavesBattlefield(Filter::Source),
            Body::effect(gain(3)),
        ))
        .build();
    let l = t.custom(P2, leaver, Zone::Battlefield);
    t.g.objects[l.0 as usize].base_controller = P0;
    t.g.recompute();
    assert_eq!(t.obj(l).controller, P0);
    t.g.player_loses(P2);
    t.resolve_all();
    assert!(!t.is_live(l));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn an_auras_trigger_on_the_enchanted_permanent_leaving_can_find_both_cards() {
    cr!("603.6e");
    let mut t = TestGame::new(2);
    // "Enchant creature. When enchanted creature dies, return that card to the battlefield
    // under your control, then return this Aura to its owner's hand."
    let aura = CB::new("Undying Bond")
        .enchantment()
        .subtypes(&["Aura"])
        .ability(AbilityDef::new(
            AbilityKind::Keyword(mtg_engine::keywords::Keyword::with_filter(
                mtg_engine::keywords::KeywordKind::Enchant,
                Filter::creature(),
            )),
            "Enchant creature",
        ))
        .ability(trig(
            TriggerCond::Dies(Filter::AttachedToSource),
            Body::effect(Effect::Seq(vec![
                Effect::Move {
                    what: Sel::TriggerObject,
                    to: Destination::battlefield().under_your_control(),
                },
                Effect::Move {
                    what: Sel::This,
                    to: Destination::zone(ZoneKind::Hand),
                },
            ])),
        ))
        .build();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.custom(P0, aura, Zone::Battlefield);
    t.g.objects[a.0 as usize].attached_to = Some(Entity::Object(bears));
    t.g.recompute();
    destroy(&mut t, bears);
    // State-based actions put the Aura into the graveyard before the trigger resolves.
    t.settle();
    assert!(t.in_graveyard(P0, "Undying Bond"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_hand(P0, "Undying Bond"));
}
