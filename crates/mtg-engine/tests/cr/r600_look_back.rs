//! CR 603.9 and 603.10b–f: abilities that trigger on a player losing the game, and
//! triggered abilities that look back in time.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn loss_watcher(name: &str) -> CardDef {
    // "Whenever a player loses the game, you gain 2 life."
    CB::new(name)
        .enchantment()
        .ability(trig(TriggerCond::PlayerLoses, Body::effect(gain(2))))
        .build()
}

#[test]
fn loses_the_game_triggers_on_any_loss_or_concession() {
    cr!("603.9");
    let mut t = TestGame::new(3);
    t.custom(P0, loss_watcher("Loss Watcher"), Zone::Battlefield);
    // A state-based loss (0 life).
    t.g.players[P2.idx()].life = 0;
    t.settle();
    assert!(t.has_lost(P2));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Conceding.
    let mut t = TestGame::new(3);
    t.custom(P0, loss_watcher("Loss Watcher"), Zone::Battlefield);
    t.g.perform_action(P1, Action::Concede).unwrap();
    assert!(t.has_lost(P1));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn loses_the_game_abilities_look_back_before_the_players_objects_leave() {
    cr!("603.10", "603.10f");
    let mut t = TestGame::new(3);
    // P0 controls an enchantment owned by P2. When P2 loses, it leaves the game with
    // them (CR 800.4a), but the ability looks back in time and still triggers.
    let w = t.custom(P2, loss_watcher("Borrowed Watcher"), Zone::Battlefield);
    t.g.objects[w.0 as usize].base_controller = P0;
    t.g.recompute();
    t.g.player_loses(P2);
    assert!(!t.is_live(w));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn phasing_out_abilities_look_back_in_time() {
    cr!("603.10", "603.10b");
    // "When this creature phases out, you gain 3 life." A phased-out permanent is treated
    // as though it doesn't exist, but the ability looks back to when it was phased in.
    let def = CB::new("Fading Sentinel")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::PhasesOut(Filter::Source),
            Body::effect(gain(3)),
        ))
        .build();
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Battlefield);
    let phase = CB::new("Phase Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::PhaseOut {
                what: Sel::Target(0),
            },
        ))
        .build();
    let p = t.custom(P0, phase, Zone::Hand(P0));
    t.cast(P0, p).target(s).go();
    t.resolve_all();
    assert!(t.obj(s).phased_out);
    assert_eq!(t.life(P0), 23);
    // An Aura phasing out indirectly with the creature it's attached to also sees it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = CB::new("Fading Charm")
        .enchantment()
        .subtypes(&["Aura"])
        .ability(AbilityDef::new(
            AbilityKind::Keyword(keywords::Keyword::with_filter(
                keywords::KeywordKind::Enchant,
                Filter::creature(),
            )),
            "Enchant creature",
        ))
        .ability(trig(
            TriggerCond::PhasesOut(Filter::Source),
            Body::effect(gain(4)),
        ))
        .build();
    let a = t.custom(P0, aura, Zone::Battlefield);
    t.g.objects[a.0 as usize].attached_to = Some(Entity::Object(bears));
    t.g.recompute();
    let p = t.custom(
        P0,
        CB::new("Phase Test")
            .instant()
            .cost("{0}")
            .spell(Body::simple(
                vec![target_creature()],
                Effect::PhaseOut {
                    what: Sel::Target(0),
                },
            ))
            .build(),
        Zone::Hand(P0),
    );
    t.cast(P0, p).target(bears).go();
    t.resolve_all();
    assert!(t.obj(a).phased_out_indirectly);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn becoming_unattached_looks_back_in_time() {
    cr!("603.10", "603.10c");
    // Grafted Wargear: "Whenever Grafted Wargear becomes unattached from a permanent,
    // sacrifice that permanent."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ape = t.battlefield(P0, "Craw Wurm");
    let wg = t.battlefield(P0, "Grafted Wargear");
    t.activate(P0, wg, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(wg).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (5, 4));
    // Moving it to another creature: the first one is sacrificed.
    t.activate(P0, wg, 0, &[Entity::Object(ape)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(ape));
    // The Equipment leaving the battlefield: it's no longer on the battlefield when it
    // becomes unattached, but the ability looks back and still triggers.
    t.g.destroy(wg, None);
    t.settle();
    assert!(!t.is_live(wg));
    t.resolve_all();
    assert!(!t.on_battlefield(ape));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
}

#[test]
fn losing_control_looks_back_in_time() {
    cr!("603.10", "603.10d");
    // Khârn the Betrayer: "When you lose control of Khârn the Betrayer, draw two cards."
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Khârn the Betrayer");
    let steal = CB::new("Steal Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::GainControl {
                what: Sel::Target(0),
                who: PlayerRef::You,
                duration: Duration::Permanent,
            },
        ))
        .build();
    let s = t.custom(P1, steal, Zone::Hand(P1));
    let before0 = t.hand_size(P0);
    let before1 = t.hand_size(P1);
    t.cast(P1, s).target(k).go();
    t.resolve_all();
    assert_eq!(t.obj(k).controller, P1);
    // The ability is controlled by the player who lost control (the controller at the
    // time it triggered, looking back), not the new controller.
    assert_eq!(t.hand_size(P0), before0 + 2);
    assert_eq!(t.hand_size(P1), before1 - 1);
}

#[test]
fn spell_countered_abilities_look_back_in_time() {
    cr!("603.10", "603.10e");
    // A spell with "When this spell is countered, you gain 5 life." It's in the graveyard
    // when it's countered, but the ability looks back to when it was on the stack.
    let mut stubborn = TriggeredAbility::new(
        TriggerCond::SpellCountered(Filter::Source),
        Body::effect(gain(5)),
    );
    stubborn.zone = FunctionZone::Stack;
    let def = CB::new("Stubborn Bolt")
        .instant()
        .cost("{0}")
        .ability(trig_from(stubborn))
        .spell(Body::effect(draw(1)))
        .build();
    let mut t = TestGame::new(2);
    let b = t.custom(P0, def, Zone::Hand(P0));
    let spell = t.cast(P0, b).go();
    let counter = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Stubborn Bolt"));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // Resolving normally doesn't trigger it.
    let def = CB::new("Stubborn Bolt")
        .instant()
        .cost("{0}")
        .ability(trig_from({
            let mut s = TriggeredAbility::new(
                TriggerCond::SpellCountered(Filter::Source),
                Body::effect(gain(5)),
            );
            s.zone = FunctionZone::Stack;
            s
        }))
        .spell(Body::effect(draw(1)))
        .build();
    let mut t = TestGame::new(2);
    let b = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
