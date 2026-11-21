//! Rulings batch P164 — Aura synergies: moving Auras (Aura Finesse, Enchantment
//! Alteration), "whenever an Aura becomes attached" (Siona), "whenever an enchantment
//! enters" (Ajani's Chosen), Auras created attached (Preston Garvey's Settlement), umbra
//! armor granted by Umbra Mystic, "if ~ is enchanted" (Dreampod Druid), Eidolon of
//! Countless Battles, Hateful Eidolon, Nyx Herald and Calix.

use crate::r_p164_common::*;
use crate::r_s01_common::custom_card;
use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Aura Finesse: attach `aura` to `creature`.
fn finesse(t: &mut TestGame, p: PlayerId, aura: ObjectId, creature: ObjectId) -> ObjectId {
    t.lands(p, "Island", 1);
    let f = t.hand(p, "Aura Finesse");
    t.cast(p, f).target(aura).target(creature).go()
}

/// The number of Human Soldier tokens `p` controls (Siona's).
fn soldiers(t: &TestGame, p: PlayerId) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.has_subtype("Soldier"))
        .count()
}

/// `p` casts the real Aura `name` on `target` (with plenty of lands of `land`) and
/// resolves everything.
fn cast_aura(t: &mut TestGame, p: PlayerId, name: &str, land: &str, target: ObjectId) {
    t.lands(p, land, 4);
    let a = t.hand(p, name);
    t.cast(p, a).target(target).go();
    t.resolve_all();
}

#[test]
fn aura_finesse_with_one_illegal_target_still_draws() {
    cr!("608.2b", "701.3b");
    ruling!(
        "Aura Finesse",
        "As Aura Finesse resolves, if either target is illegal, the spell resolves but the Aura doesn’t move. You still draw a card. If both targets are illegal, Aura Finesse doesn’t resolve and you don’t draw a card."
    );
    supported("Aura Finesse");
    for (bounce_aura, bounce_creature) in [(false, true), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let elves = t.battlefield(P0, "Llanowar Elves");
        let aura = attach_new(&mut t, P0, "Holy Strength", bears);
        finesse(&mut t, P0, aura, elves);
        if bounce_aura {
            bounce(&mut t, aura);
        }
        if bounce_creature {
            bounce(&mut t, elves);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        let both = bounce_aura && bounce_creature;
        assert_eq!(t.hand_size(P0), hand + usize::from(!both));
        if !bounce_aura {
            assert_eq!(attached_to(&t, aura), Some(o(bears)));
        }
    }
}

#[test]
fn aura_finesse_targets_any_aura_you_control_and_any_creature() {
    cr!("115.1", "608.2b", "303.4e", "303.4j");
    ruling!(
        "Aura Finesse",
        "If you target an Aura that’s attached to a creature, you may also target the creature it’s attached to. Aura Finesse won’t cause the Aura to move, but you’ll still draw a card when it resolves."
    );
    ruling!(
        "Aura Finesse",
        "You may target any Aura you control, and you may target any creature (including one you don’t control). Whether the Aura could legally enchant the creature is irrelevant as you’re choosing the targets."
    );
    ruling!(
        "Aura Finesse",
        "You still control the Aura, even if you attach it to a creature you don’t control."
    );
    // The creature it already enchants: it stays, and a card is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    finesse(&mut t, P0, aura, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(attached_to(&t, aura), Some(o(bears)));
    // An opponent's creature: P0 still controls the Aura.
    let giant = t.battlefield(P1, "Hill Giant");
    finesse(&mut t, P0, aura, giant);
    t.resolve_all();
    assert_eq!(attached_to(&t, aura), Some(o(giant)));
    assert_eq!(t.obj_now(aura).controller, P0);
    assert_eq!(t.obj_now(giant).controller, P1);
    assert_eq!(t.pt(giant), (4, 5));
    // Wild Growth (enchant land) may be targeted with a creature; it doesn't move, and
    // the card is still drawn.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let growth = attach_new(&mut t, P0, "Wild Growth", forest);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let f = finesse(&mut t, P0, growth, bears);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, f),
        vec![o(growth), o(bears)]
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(attached_to(&t, growth), Some(o(forest)));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn enchantment_alteration_new_target_changes_only_the_aura() {
    cr!("115.7", "608.2c");
    ruling!(
        "Enchantment Alteration",
        "If you change this card’s target, you can change which Aura is targeted, but you can’t choose where that Aura will be moved."
    );
    supported("Enchantment Alteration");
    supported("Redirect");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P0, "Hill Giant");
    let holy = attach_new(&mut t, P0, "Holy Strength", bears);
    let unholy = attach_new(&mut t, P0, "Unholy Strength", elves);
    t.lands(P0, "Island", 1);
    let ea = t.hand(P0, "Enchantment Alteration");
    let spell = t.cast(P0, ea).target(holy).go();
    // P1 redirects it to Unholy Strength; the only thing P1 chooses is the target.
    t.lands(P1, "Island", 2);
    let redirect = t.hand(P1, "Redirect");
    t.cast(P1, redirect).target(spell).go();
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[o(unholy)]);
    t.resolve();
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![o(unholy)]);
    // As it resolves, P0 chooses where it goes: the Hill Giant.
    let from = t.asked().len();
    t.answer_choose(P0, &[o(giant)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, unholy), Some(o(giant)));
    assert_eq!(attached_to(&t, holy), Some(o(bears)));
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0]);
}

#[test]
fn siona_triggers_whenever_an_aura_becomes_attached() {
    cr!("603.2", "603.2e", "701.3a", "701.3b");
    ruling!(
        "Siona, Captain of the Pyleas",
        "Siona's last ability triggers both whenever an Aura enters the battlefield attached to a creature you control and whenever an Aura on the battlefield attached to a different object becomes attached to a creature you control. It doesn't trigger if an effect tries to attach an Aura to the creature it already enchants."
    );
    supported("Siona, Captain of the Pyleas");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Siona, Captain of the Pyleas");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    // Enters attached.
    cast_aura(&mut t, P0, "Holy Strength", "Plains", bears);
    assert_eq!(soldiers(&t, P0), 1);
    let holy = t.named_on_battlefield("Holy Strength")[0];
    // Moves to another creature.
    finesse(&mut t, P0, holy, elves);
    t.resolve_all();
    assert_eq!(attached_to(&t, holy), Some(o(elves)));
    assert_eq!(soldiers(&t, P0), 2);
    // "Attached" to the creature it already enchants: no trigger.
    finesse(&mut t, P0, holy, elves);
    t.resolve_all();
    assert_eq!(soldiers(&t, P0), 2);
}

#[test]
fn siona_trigger_follows_control_and_abilities_after_the_attachment() {
    cr!("603.2", "603.3a", "613.1b", "613.1f");
    ruling!(
        "Siona, Captain of the Pyleas",
        "If an Aura becomes attached to Siona that causes another player to gain control of it, that player will control the triggered ability that creates a Soldier."
    );
    ruling!(
        "Siona, Captain of the Pyleas",
        "If an Aura becomes attached to Siona that causes it to lose all abilities, Siona's last ability won't trigger."
    );
    supported("Control Magic");
    supported("Deep Freeze");
    // P1's Control Magic on P0's Siona: P1 gets the Soldier.
    let mut t = TestGame::new(2);
    let siona = t.battlefield(P0, "Siona, Captain of the Pyleas");
    t.set_step(P1, Step::PrecombatMain);
    cast_aura(&mut t, P1, "Control Magic", "Island", siona);
    assert_eq!(t.obj_now(siona).controller, P1);
    assert_eq!(soldiers(&t, P1), 1);
    assert_eq!(soldiers(&t, P0), 0);
    // P0's Deep Freeze on Siona: no trigger.
    let mut t = TestGame::new(2);
    let siona = t.battlefield(P0, "Siona, Captain of the Pyleas");
    cast_aura(&mut t, P0, "Deep Freeze", "Island", siona);
    assert_eq!(soldiers(&t, P0), 0);
}

#[test]
fn siona_looks_at_the_creature_after_the_aura_is_attached() {
    cr!("603.2", "603.10", "613.1d");
    ruling!(
        "Siona, Captain of the Pyleas",
        "If an Aura becomes attached to a noncreature permanent you control that causes it to become a creature, Siona's last ability triggers. If an Aura becomes attached to a creature you don't control that causes it to become a creature you do control, Siona's last ability triggers. If an Aura becomes attached to a creature you control that causes it to no longer be a creature or to no longer be under your control, Siona's last ability doesn't trigger."
    );
    supported("Animate Artifact");
    supported("Imprisoned in the Moon");
    // Animate Artifact on P0's Mind Stone: a creature now, so it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Siona, Captain of the Pyleas");
    let stone = t.battlefield(P0, "Mind Stone");
    cast_aura(&mut t, P0, "Animate Artifact", "Island", stone);
    assert!(t.obj_now(stone).is_creature());
    assert_eq!(soldiers(&t, P0), 1);
    // Control Magic on P1's Bears: P0's creature now, so it triggers.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_aura(&mut t, P0, "Control Magic", "Island", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(soldiers(&t, P0), 2);
    // Imprisoned in the Moon on P0's Elves: no longer a creature, so no trigger.
    let elves = t.battlefield(P0, "Llanowar Elves");
    cast_aura(&mut t, P0, "Imprisoned in the Moon", "Island", elves);
    assert!(!t.obj_now(elves).is_creature());
    assert_eq!(soldiers(&t, P0), 2);
    // Fealty to the Realm ("The monarch controls enchanted creature.") on P0's own Hill
    // Giant while P1 is the monarch: it enters attached to a creature P1 controls, so no
    // trigger (its "you become the monarch" gives the Giant back afterwards).
    supported("Fealty to the Realm");
    let giant = t.battlefield(P0, "Hill Giant");
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.lands(P0, "Island", 5);
    let fealty = t.hand(P0, "Fealty to the Realm");
    t.cast(P0, fealty).target(giant).go();
    t.resolve();
    assert_eq!(t.obj_now(giant).controller, P1);
    t.resolve_all();
    assert_eq!(soldiers(&t, P0), 2);
    assert_eq!(soldiers(&t, P1), 0);
}

#[test]
fn ajanis_chosen_triggers_for_an_aura_on_an_opponents_permanent() {
    cr!("303.4e", "603.2", "603.6a");
    ruling!(
        "Ajani's Chosen",
        "If you cast an Aura targeting an opponent or a permanent controlled by an opponent, you control that Aura when it enters the battlefield. The ability of Ajani's Chosen will trigger."
    );
    supported("Ajani's Chosen");
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Ajani's Chosen");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    cast_aura(&mut t, P0, "Pacifism", "Plains", giant);
    assert_eq!(triggers_of(&t, chosen), 1);
    let pacifism = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(t.obj_now(pacifism).controller, P0);
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(attached_to(&t, pacifism), Some(o(giant)));
}

#[test]
fn ajanis_chosen_the_aura_needs_a_target_first_and_both_must_remain() {
    cr!("303.4a", "601.2c", "608.2b", "701.3b", "614.1a", "614.6");
    ruling!(
        "Ajani's Chosen",
        "The ability doesn't trigger until the enchantment enters the battlefield. This means that, for example, you can't cast an Aura with enchant creature if there are no legal targets available, hoping to enchant the Cat token. Ajani's Chosen itself, however, may be a legal target for the Aura."
    );
    ruling!(
        "Ajani's Chosen",
        "You may attach the Aura to the Cat token only if the Aura and the token are both on the battlefield when the triggered ability resolves. If the Aura leaves the battlefield before the ability resolves, it won't become attached to the token. If the token leaves the battlefield before the ability resolves, the Aura won't become attached to that token."
    );
    supported("Lightning Greaves");
    supported("Hallowed Moonlight");
    // Ajani's Chosen with shroud is no legal target: Holy Strength can't be cast.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Ajani's Chosen");
    attach_new(&mut t, P0, "Lightning Greaves", chosen);
    t.lands(P0, "Plains", 1);
    let holy = t.hand(P0, "Holy Strength");
    assert!(t.cast(P0, holy).try_go().is_err());
    // Without the Greaves, Ajani's Chosen is a legal target; the Aura then moves to the
    // Cat.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Ajani's Chosen");
    t.answer_yes(P0, true);
    cast_aura(&mut t, P0, "Holy Strength", "Plains", chosen);
    let cat = tokens(&t, P0)[0];
    let holy = t.named_on_battlefield("Holy Strength")[0];
    assert_eq!(attached_to(&t, holy), Some(o(cat)));
    assert_eq!(t.pt(cat), (3, 4));
    // The Aura leaves before the trigger resolves: a Cat, but nothing attached.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Ajani's Chosen");
    t.lands(P0, "Plains", 1);
    let holy = t.hand(P0, "Holy Strength");
    t.cast(P0, holy).target(chosen).go();
    t.resolve();
    assert_eq!(triggers_of(&t, chosen), 1);
    let holy = t.named_on_battlefield("Holy Strength")[0];
    bounce(&mut t, holy);
    t.answer_yes(P0, true);
    t.resolve_all();
    let cat = tokens(&t, P0)[0];
    assert_eq!(t.pt(cat), (2, 2));
    assert!(t.in_hand(P0, "Holy Strength"));
    // The token: the ability creates it and attaches the Aura in one resolution, during
    // which no player acts and no state-based action is checked, so it can't leave in
    // between; it can only fail to be on the battlefield at all. Under Hallowed Moonlight
    // ("if a creature would enter and it wasn't cast, exile it instead") the Cat is
    // exiled instead of entering, and the Aura stays on Ajani's Chosen.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Ajani's Chosen");
    t.lands(P0, "Plains", 3);
    let moonlight = t.hand(P0, "Hallowed Moonlight");
    t.cast(P0, moonlight).go();
    t.resolve_all();
    let holy = t.hand(P0, "Holy Strength");
    t.cast(P0, holy).target(chosen).go();
    t.resolve();
    assert_eq!(triggers_of(&t, chosen), 1);
    let holy = t.named_on_battlefield("Holy Strength")[0];
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(attached_to(&t, holy), Some(o(chosen)));
    assert_eq!(t.pt(chosen), (4, 5));
}

#[test]
fn preston_garvey_creates_no_settlement_without_a_land_it_can_enchant() {
    cr!("608.2b", "303.4i", "702.16c");
    ruling!(
        "Preston Garvey, Minuteman",
        "If the target land is an illegal target as Preston Garvey's first ability tries to resolve, it won't resolve and none of its effects happen. The Settlement token won't be created."
    );
    ruling!(
        "Preston Garvey, Minuteman",
        "In the rare case that the target land is a legal target as Preston Garvey's first ability resolves but the target land can't legally be enchanted by the Settlement token, the token won't be created at all."
    );
    supported("Preston Garvey, Minuteman");
    let settlements = |t: &TestGame| t.named_on_battlefield("Settlement").len();
    // The usual case: a Settlement on the Forest.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Preston Garvey, Minuteman");
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[o(forest)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(settlements(&t), 1);
    // The Forest leaves first: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Preston Garvey, Minuteman");
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[o(forest)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    bounce(&mut t, forest);
    t.resolve_all();
    assert_eq!(settlements(&t), 0);
    assert!(tokens(&t, P0).is_empty());
    // A land with protection from enchantments is a legal target (Preston is a creature)
    // but can't be enchanted: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Preston Garvey, Minuteman");
    let warded = t.custom(
        P0,
        custom_card(
            "Warded Field",
            "Land",
            "",
            None,
            "Protection from enchantments\n{T}: Add {C}.",
        ),
        mtg_engine::object::Zone::Battlefield,
    );
    t.answer_targets(P0, &[o(warded)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_of(&t, t.named_on_battlefield("Preston Garvey, Minuteman")[0]), 1);
    t.resolve_all();
    assert!(t.on_battlefield(warded));
    assert_eq!(settlements(&t), 0);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn umbra_mystic_umbra_armor_ignores_who_controls_the_aura() {
    cr!("702.89a", "616.1");
    ruling!(
        "Umbra Mystic",
        "If a permanent you control would be destroyed, and it's enchanted by an Aura with umbra armor, it doesn't matter who controls that Aura. The umbra armor effect is mandatory. If that permanent is enchanted by multiple Auras, you choose which one is destroyed, regardless of who controls them."
    );
    supported("Umbra Mystic");
    // P0's choice decides which Aura is destroyed, whoever controls it: the two answers
    // destroy different Auras.
    let mut kept_mine = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Umbra Mystic");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let theirs = attach_new(&mut t, P1, "Holy Strength", bears);
        let mine = attach_new(&mut t, P0, "Unholy Strength", bears);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        crate::r_s02_common::destroy(&mut t, bears);
        assert!(t.on_battlefield(bears));
        let asked: Vec<PlayerId> = t
            .asked()
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(asked, vec![P0]);
        assert_ne!(t.on_battlefield(theirs), t.on_battlefield(mine));
        kept_mine.push(t.on_battlefield(mine));
    }
    assert_ne!(kept_mine[0], kept_mine[1]);
    // Only one Aura, controlled by the opponent: destroyed with no choice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Umbra Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = attach_new(&mut t, P1, "Holy Strength", bears);
    crate::r_s02_common::destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(theirs));
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseReplacement { .. })));
}

#[test]
fn umbra_mystic_on_an_umbra_is_redundant() {
    cr!("702.89a", "616.1f");
    ruling!(
        "Umbra Mystic",
        "Multiple instances of umbra armor on the same Aura are redundant."
    );
    supported("Hyena Umbra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Umbra Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = attach_new(&mut t, P0, "Hyena Umbra", bears);
    let other = attach_new(&mut t, P0, "Holy Strength", bears);
    // Hyena Umbra is chosen: it's destroyed once (one Aura), the creature survives.
    t.answer(P0, DecisionKind::Replacement, Answer::Index(0));
    crate::r_s02_common::destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    let gone = [umbra, other]
        .iter()
        .filter(|a| !t.on_battlefield(**a))
        .count();
    assert_eq!(gone, 1);
}

#[test]
fn umbra_mystic_umbra_armor_chains_through_an_aura_on_an_aura() {
    cr!("702.89a", "616.1f", "614.6");
    ruling!(
        "Umbra Mystic",
        "Say you control a permanent that's enchanted by an Aura you control, and that Aura is itself enchanted by an Aura. If the permanent would be destroyed, instead the first Aura is destroyed... but since that Aura would be destroyed, instead the second Aura is destroyed."
    );
    supported("Feedback");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Umbra Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let holy = attach_new(&mut t, P0, "Holy Strength", bears);
    // Feedback: "Enchant enchantment".
    let feedback = attach_new(&mut t, P0, "Feedback", holy);
    crate::r_s02_common::destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(holy));
    assert!(!t.on_battlefield(feedback));
    assert!(t.in_graveyard(P0, "Feedback"));
}

#[test]
fn dreampod_druid_checks_enchanted_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Dreampod Druid",
        "Dreampod Druid’s ability checks to see if it’s enchanted at the beginning of each upkeep. If it’s not, the ability won’t trigger. If the ability triggers but Dreampod Druid isn’t enchanted when that ability resolves, the ability will do nothing."
    );
    supported("Dreampod Druid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Dreampod Druid");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_of(&t, druid), 0);
    // Enchanted at P0's upkeep: it triggers; the Aura leaves before it resolves.
    let aura = attach_new(&mut t, P0, "Holy Strength", druid);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_of(&t, druid), 1);
    bounce(&mut t, aura);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Still enchanted: a Saproling (at each upkeep, the opponent's too).
    attach_new(&mut t, P0, "Holy Strength", druid);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn eidolon_of_countless_battles_is_0_0_off_the_battlefield() {
    cr!("113.6", "611.3a", "604.1");
    ruling!(
        "Eidolon of Countless Battles",
        "Eidolon of Countless Battles's last ability functions only on the battlefield. In other zones, it is a 0/0 enchantment creature card."
    );
    supported("Eidolon of Countless Battles");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    let in_hand = t.hand(P0, "Eidolon of Countless Battles");
    let in_yard = t.graveyard(P0, "Eidolon of Countless Battles");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (0, 0));
    assert_eq!(t.pt(in_yard), (0, 0));
    // On the battlefield: two creatures and one Aura.
    let e = t.battlefield(P0, "Eidolon of Countless Battles");
    assert_eq!(t.pt(e), (3, 3));
}

#[test]
fn hateful_eidolon_dying_with_enchanted_creatures_sees_each_of_them() {
    cr!("603.10a", "603.2");
    ruling!(
        "Hateful Eidolon",
        "If Hateful Eidolon dies at the same time as one or more enchanted creatures, its last ability triggers for each of those creatures. This includes Hateful Eidolon itself if it's enchanted."
    );
    supported("Hateful Eidolon");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    let eidolon = t.battlefield(P0, "Hateful Eidolon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // The Bears: two of P0's Auras and one of P1's. The Eidolon: one of P0's. P1's Giant:
    // one of P0's.
    attach_new(&mut t, P0, "Holy Strength", bears);
    attach_new(&mut t, P0, "Unholy Strength", bears);
    attach_new(&mut t, P1, "Holy Strength", bears);
    attach_new(&mut t, P0, "Holy Strength", eidolon);
    attach_new(&mut t, P0, "Unholy Strength", giant);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // One trigger per enchanted creature that died (the unenchanted Bears isn't one).
    assert_eq!(triggers_of(&t, eidolon), 3);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn nyx_herald_bonus_stays_after_the_aura_leaves() {
    cr!("611.2c", "613.1a", "613.4c");
    ruling!(
        "Nyx Herald",
        "Once Nyx Herald’s ability resolves, the creature keeps +1/+1 and trample even if it stops being enchanted later in the turn (or, in some unusual cases, stops being an enchantment)."
    );
    supported("Nyx Herald");
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nyx Herald");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    t.answer_targets(P0, &[o(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 5));
    bounce(&mut t, aura);
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    // An enchantment creature (Nyx Herald itself) that stops being an enchantment:
    // Cytoshape makes it a copy of Grizzly Bears, a 2/2 creature that isn't one. It's a
    // 3/3 with trample.
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Nyx Herald");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(herald)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(herald), (3, 4));
    t.lands(P0, "Tropical Island", 3);
    t.answer_choose(P0, &[o(bears)]);
    let cyto = t.hand(P0, "Cytoshape");
    t.cast(P0, cyto).target(herald).go();
    t.resolve_all();
    let chars = &t.obj_now(herald).chars;
    assert_eq!(chars.name, "Grizzly Bears");
    assert!(!chars.is(mtg_engine::types::CardType::Enchantment));
    assert_eq!(t.pt(herald), (3, 3));
    assert!(has_kw(&t, herald, KeywordKind::Trample));
}

#[test]
fn calix_creates_one_token_a_turn_and_extra_instances_do_nothing() {
    cr!("603.2h", "510.4");
    ruling!(
        "Calix, Guided by Fate",
        "Once you choose to create a token using the last ability, that ability won't trigger again that turn. Additional instances of the ability that are already on the stack will resolve but will have no effect."
    );
    supported("Calix, Guided by Fate");
    supported("Fencing Ace");
    // An enchanted Fencing Ace (double strike) deals combat damage to P1 twice. A token
    // made in the first-strike damage step: no trigger in the regular one. No token
    // made the first time: it triggers again, and a token can be made then. (The token
    // entering triggers Calix's constellation ability, which targets Calix.)
    for first in [true, false] {
        let mut t = TestGame::new(2);
        let calix = t.battlefield(P0, "Calix, Guided by Fate");
        let ace = t.battlefield(P0, "Fencing Ace");
        attach_new(&mut t, P0, "Holy Strength", ace);
        let noble = t.battlefield(P0, "Noble Stand");
        t.answer_targets(P0, &[o(calix)]);
        t.answer_yes(P0, first);
        if first {
            t.answer_choose(P0, &[o(noble)]);
        } else {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[o(noble)]);
        }
        attack_with(&mut t, &[(ace, pl(P1))]);
        t.advance_to(P0, Step::FirstStrikeDamage);
        t.settle();
        assert_eq!(crate::r_p189_common::stack_triggers_from(&t, calix).len(), 1);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Noble Stand").len(), 1 + usize::from(first));
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P1), 16, "two hits");
        assert_eq!(
            crate::r_p189_common::stack_triggers_from(&t, calix).len(),
            usize::from(!first)
        );
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Noble Stand").len(), 2);
    }
    // Two instances on the stack at once: the second does nothing.
    let mut t = TestGame::new(2);
    let calix = t.battlefield(P0, "Calix, Guided by Fate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    t.battlefield(P0, "Noble Stand");
    attack_with(&mut t, &[(calix, pl(P1)), (bears, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_of(&t, calix), 2);
    let noble = t.named_on_battlefield("Noble Stand")[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(noble)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(noble)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Noble Stand").len(), 2);
}
