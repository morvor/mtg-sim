//! CR 702.151 Reconfigure.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::run_effect;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The two activated abilities reconfigure stands for: (attach, unattach).
fn reconfigure_uids(t: &mut TestGame, eq: ObjectId) -> (u64, u64) {
    t.g.recompute();
    let v: Vec<u64> = t
        .obj(eq)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text == "Reconfigure")
        .map(|a| a.uid)
        .collect();
    assert_eq!(v.len(), 2);
    (v[0], v[1])
}

fn attach_with_reconfigure(t: &mut TestGame, eq: ObjectId, to: ObjectId) {
    let (attach, _) = reconfigure_uids(t, eq);
    add_mana(t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[Entity::Object(to)]);
    activate_uid(t, P0, eq, attach).expect("reconfigure");
    t.resolve_all();
}

#[test]
fn reconfigure_attaches_it_to_another_creature_and_it_stops_being_a_creature() {
    cr!("702.151", "702.151a", "702.151b", "301.5c");
    assert_supported("Lizard Blades");
    ruling!(
        "Bronzeplate Boar",
        "Attaching an Equipment with reconfigure to a creature causes that Equipment to stop being a creature until it becomes unattached. It also loses any creature subtypes it had."
    );
    ruling!(
        "Bronzeplate Boar",
        "Reconfigure represents two activated abilities."
    );
    let mut t = TestGame::new(2);
    // Lizard Blades: 1/1 artifact creature — Equipment Lizard, double strike, "Equipped
    // creature has double strike." Reconfigure {2}.
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj(blades).is_creature());
    attach_with_reconfigure(&mut t, blades, bears);
    assert_eq!(t.obj(blades).attached_to, Some(Entity::Object(bears)));
    let o = t.obj(blades);
    assert!(!o.is_creature());
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Equipment"));
    assert!(!o.chars.has_subtype("Lizard"));
    assert!(t
        .obj(bears)
        .chars
        .has_keyword(KeywordKind::DoubleStrike));
    // Unattached, it's a creature again.
    let (_, unattach) = reconfigure_uids(&mut t, blades);
    add_mana(&mut t, P0, ManaType::C, 2);
    activate_uid(&mut t, P0, blades, unattach).expect("unattach");
    t.resolve_all();
    assert_eq!(t.obj(blades).attached_to, None);
    assert!(t.obj(blades).is_creature());
    assert!(t.obj(blades).chars.has_subtype("Lizard"));
    assert!(!t
        .obj(bears)
        .chars
        .has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn reconfigure_targets_another_creature_you_control_as_a_sorcery() {
    cr!("702.151a");
    ruling!(
        "Bronzeplate Boar",
        "An Equipment creature can never become attached to itself. If an effect tries to do this, nothing happens."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let (attach, unattach) = reconfigure_uids(&mut t, blades);
    add_mana(&mut t, P0, ManaType::C, 2);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(!activatable(&mut t, P0, blades, attach));
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(activatable(&mut t, P0, blades, attach));
    // Not attached: the unattach ability can't be activated.
    assert!(!activatable(&mut t, P0, blades, unattach));
    activate_uid(&mut t, P0, blades, attach).unwrap();
    let candidates = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap();
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(!candidates.contains(&Entity::Object(blades)));
    assert!(!candidates.contains(&Entity::Object(theirs)));
    // An effect trying to attach it to itself does nothing.
    t.resolve_all();
    run_effect(
        &mut t,
        Some(blades),
        P0,
        Effect::Attach {
            what: Sel::This,
            to: Sel::This,
        },
        &[],
    );
    assert_eq!(t.obj(blades).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn attached_by_another_effect_it_also_stops_being_a_creature() {
    cr!("702.151b");
    ruling!(
        "Bronzeplate Boar",
        "An Equipment creature with reconfigure can be attached to creatures by effects other than its reconfigure ability, such as the activated ability of Brass Squire."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Attach target Equipment you control to target creature you control."
    run_effect(
        &mut t,
        Some(blades),
        P0,
        Effect::Attach {
            what: Sel::This,
            to: Sel::Target(0),
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj(blades).attached_to, Some(Entity::Object(bears)));
    assert!(!t.obj(blades).is_creature());
    // The equipped creature leaving makes it unattached: it's a creature again.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(bears)],
    );
    t.settle();
    assert_eq!(t.obj(blades).attached_to, None);
    assert!(t.obj(blades).is_creature());
}

#[test]
fn it_isnt_a_creature_while_attached_even_if_it_loses_its_abilities() {
    cr!("702.151b");
    ruling!(
        "Bronzeplate Boar",
        "If an Equipment with reconfigure somehow loses its abilities while it is attached, the effect causing it to not be a creature continues to apply until it becomes unattached."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_with_reconfigure(&mut t, blades, bears);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(blades)],
    );
    t.settle();
    assert!(!t.obj(blades).chars.has_keyword(KeywordKind::Reconfigure));
    assert!(!t.obj(blades).is_creature());
    assert_eq!(t.obj(blades).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn if_it_is_still_a_creature_after_it_becomes_attached_it_becomes_unattached() {
    cr!("702.151b", "704.5p");
    ruling!(
        "Bronzeplate Boar",
        "If a permanent with reconfigure is somehow still a creature after it becomes attached (perhaps due to an effect like that of March of the Machines), it immediately becomes unattached from the equipped creature."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_with_reconfigure(&mut t, blades, bears);
    // Mind Transfer Protocol: "Until end of turn, target artifact or creature becomes an
    // artifact creature with base power and toughness 4/5."
    let spell = t.hand(P0, "Mind Transfer Protocol");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, spell).target(blades).go();
    t.resolve_all();
    assert!(t.obj(blades).is_creature());
    assert_eq!(t.obj(blades).attached_to, None);
    assert_eq!(t.pt(blades), (4, 5));
}

#[test]
fn an_equipment_isnt_tapped_or_untapped_with_the_creature() {
    cr!("702.151a");
    ruling!(
        "Bronzeplate Boar",
        "An Equipment doesn't become tapped when the permanent it's attached to becomes tapped."
    );
    ruling!(
        "Bronzeplate Boar",
        "Similarly, if an Equipment is tapped, its reconfigure abilities may still be activated and it may still become attached to creatures. Becoming attached doesn't untap it."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[blades.0 as usize].tapped = true;
    attach_with_reconfigure(&mut t, blades, bears);
    assert!(t.obj(blades).tapped);
    t.g.objects[blades.0 as usize].tapped = false;
    // The Bears attack with double strike; the Equipment stays untapped.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(t.obj(bears).tapped);
    assert!(!t.obj(blades).tapped);
    // Unattached after combat, it could block during the opponent's turn.
    t.advance_to(P0, Step::PostcombatMain);
    let (_, unattach) = reconfigure_uids(&mut t, blades);
    add_mana(&mut t, P0, ManaType::C, 2);
    activate_uid(&mut t, P0, blades, unattach).unwrap();
    t.resolve_all();
    assert!(t.obj(blades).is_creature());
    assert!(!t.obj(blades).tapped);
}

#[test]
fn auras_that_can_enchant_only_creatures_fall_off_as_it_stops_being_a_creature() {
    cr!("702.151b", "704.5m");
    ruling!(
        "Bronzeplate Boar",
        "As soon as an Equipment creature with reconfigure stops being a creature, any Equipment and Auras with enchant creature abilities become unattached."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Holy Strength ("Enchant creature. Enchanted creature gets +1/+2.") on the Blades.
    let aura = t.hand(P0, "Holy Strength");
    add_mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, aura).target(blades).go();
    t.resolve_all();
    assert_eq!(t.pt(blades), (2, 3));
    attach_with_reconfigure(&mut t, blades, bears);
    assert!(!t.obj(blades).is_creature());
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn reconfigure_isnt_an_equip_ability() {
    cr!("702.151a");
    ruling!(
        "Bronzeplate Boar",
        "Although it causes an Equipment to become attached to a creature, reconfigure is not an “equip ability”"
    );
    let mut t = TestGame::new(2);
    // Bureau Headmaster: "Equip abilities you activate cost {1} less to activate."
    t.battlefield(P0, "Bureau Headmaster");
    let blades = t.battlefield(P0, "Lizard Blades");
    t.battlefield(P0, "Grizzly Bears");
    let (attach, _) = reconfigure_uids(&mut t, blades);
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(!activatable(&mut t, P0, blades, attach));
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(activatable(&mut t, P0, blades, attach));
}
