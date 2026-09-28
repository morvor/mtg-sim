//! Rulings batch S14 — reconfigure (CR 702.151): two activated abilities, "[Cost]: Attach
//! this permanent to another target creature you control. Activate only as a sorcery"
//! and "[Cost]: Unattach this permanent. Activate only if this permanent is attached to a
//! creature and only as a sorcery." Neither is an equip ability (CR 702.6).

use crate::r_s01_common::*;
use crate::r_s04_common::{ability_targets, add_mana};
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The uids of the activated abilities of `source` whose text is `text`.
fn abilities_named(t: &mut TestGame, source: ObjectId, text: &str) -> Vec<u64> {
    t.g.recompute();
    t.obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text == text)
        .map(|a| a.uid)
        .collect()
}

fn activate(t: &mut TestGame, source: ObjectId, uid: u64, target: Option<ObjectId>) -> bool {
    if let Some(x) = target {
        t.answer_targets(P0, &[Entity::Object(x)]);
    }
    t.g.turn.priority = Some(P0);
    let r = t.g.activate_ability(P0, source, uid);
    t.g.flush_events();
    t.clear_answers();
    r.is_ok()
}

#[test]
fn reconfigure_is_an_attach_ability_and_an_unattach_ability() {
    cr!("702.151a", "702.151b", "602.5d");
    ruling!(
        "Acquisition Octopus",
        "Reconfigure represents two activated abilities. Reconfigure [cost] means \"[Cost]: Attach this permanent to another target creature you control. Activate only as a sorcery,\" and \"[Cost]: Unattach this permanent. Activate only if this permanent is attached to a creature and only as a sorcery.\""
    );
    supported("Acquisition Octopus");
    // Acquisition Octopus: 2/2 artifact creature — Equipment Octopus, reconfigure {2}.
    let mut t = TestGame::new(2);
    let octopus = t.battlefield(P0, "Acquisition Octopus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let uids = abilities_named(&mut t, octopus, "Reconfigure");
    assert_eq!(uids.len(), 2);
    let (attach, unattach) = (uids[0], uids[1]);
    // "Another target creature you control": not itself, not an opponent's creature.
    let targets = ability_targets(&mut t, octopus, 0);
    assert_eq!(targets, vec![Entity::Object(bears)]);
    assert!(!targets.contains(&Entity::Object(theirs)));
    // Unattaching isn't possible while it isn't attached.
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(!activate(&mut t, octopus, unattach, None));
    // Not at instant speed: not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!activate(&mut t, octopus, attach, Some(bears)));
    t.set_step(P0, Step::PostcombatMain);
    assert!(activate(&mut t, octopus, attach, Some(bears)));
    t.resolve_all();
    assert_eq!(t.obj(octopus).attached_to, Some(Entity::Object(bears)));
    assert!(!t.obj(octopus).is_creature());
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(activate(&mut t, octopus, unattach, None));
    t.resolve_all();
    assert_eq!(t.obj(octopus).attached_to, None);
    assert!(t.obj(octopus).is_creature());
}

#[test]
fn reconfigure_isnt_an_equip_ability() {
    cr!("702.151a", "702.6a");
    ruling!(
        "Acquisition Octopus",
        "Although it causes an Equipment to become attached to a creature, reconfigure is not an \"equip ability\" for the purpose of cards like Fighter Class and Leonin Shikari."
    );
    supported("Auriok Steelshaper");
    // Auriok Steelshaper: "Equip costs you pay cost {1} less." Bone Saw's equip {1} costs
    // nothing; Acquisition Octopus's reconfigure {2} still costs {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Auriok Steelshaper");
    let octopus = t.battlefield(P0, "Acquisition Octopus");
    let saw = t.battlefield(P0, "Bone Saw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let equip = abilities_named(&mut t, saw, "Equip");
    assert_eq!(equip.len(), 1, "Bone Saw's equip ability");
    assert!(activate(&mut t, saw, equip[0], Some(bears)));
    t.resolve_all();
    assert_eq!(t.obj(saw).attached_to, Some(Entity::Object(bears)));
    let attach = abilities_named(&mut t, octopus, "Reconfigure")[0];
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(!activate(&mut t, octopus, attach, Some(bears)));
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(activate(&mut t, octopus, attach, Some(bears)));
    t.resolve_all();
    assert_eq!(t.obj(octopus).attached_to, Some(Entity::Object(bears)));
}
