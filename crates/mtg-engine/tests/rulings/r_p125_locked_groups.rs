//! Rulings batch P125 — "Creatures you control gain indestructible until end of turn" and
//! the like from a resolving spell or ability: the affected set is determined as it
//! resolves (CR 611.2c), so permanents that come under your control later aren't
//! affected. Regeneration shields for "each creature you control" (CR 701.19). Full party
//! (CR 700.8).

use crate::r_p125_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A creature entering under P0's control after the effect resolved.
fn newcomer(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Grizzly Bears")
}

#[test]
fn rootborn_defenses_affects_the_populated_token_but_not_later_creatures() {
    cr!("611.2c", "701.36a");
    ruling!(
        "Rootborn Defenses",
        "Rootborn Defenses affects only creatures you control after populating at the time it resolves."
    );
    supported("Rootborn Defenses");
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P0, "Soldier");
    cast_new(&mut t, P0, "Rootborn Defenses", &[]);
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 2, "a token was created by populating");
    assert!(toks.iter().all(|id| indestructible(&t, *id)));
    assert!(indestructible(&t, soldier));
    let later = newcomer(&mut t);
    assert!(!indestructible(&t, later));
}

#[test]
fn squad_commander_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "700.8");
    ruling!(
        "Squad Commander",
        "Squad Commander's last ability affects only creatures you control at the time it resolves."
    );
    supported("Squad Commander");
    let mut t = TestGame::new(2);
    let (commander, _) = full_party(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(indestructible(&t, commander));
    assert_eq!(t.pt(commander), (4, 3));
    let later = newcomer(&mut t);
    assert!(!indestructible(&t, later));
    assert_eq!(t.pt(later), (2, 2));
}

/// P0's full party: Squad Commander (Warrior), Archpriest of Iona (Cleric), Nimble
/// Trapfinder (Rogue) and Prodigal Sorcerer (Wizard), in P0's precombat main phase.
/// Returns Squad Commander and Archpriest of Iona.
fn full_party(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let commander = t.battlefield(P0, "Squad Commander");
    let archpriest = t.battlefield(P0, "Archpriest of Iona");
    t.battlefield(P0, "Nimble Trapfinder");
    t.battlefield(P0, "Prodigal Sorcerer");
    t.answer_targets(P0, &[obj(archpriest)]);
    (commander, archpriest)
}

#[test]
fn a_full_party_is_four_creatures_in_your_party() {
    cr!("700.8", "700.8a", "603.4");
    ruling!(
        "Squad Commander",
        "Some cards refer to you having a “full party.” This is true if the number of creatures in your party is four."
    );
    ruling!(
        "Archpriest of Iona",
        "Some cards refer to you having a “full party.” This is true if the number of creatures in your party is four."
    );
    ruling!(
        "Nimble Trapfinder",
        "Some cards refer to you having a “full party.” This is true if the number of creatures in your party is four."
    );
    supported("Archpriest of Iona");
    supported("Nimble Trapfinder");
    // Four: each "if you have a full party" ability triggers.
    let mut t = TestGame::new(2);
    let (commander, archpriest) = full_party(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert!(indestructible(&t, commander));
    // Archpriest: power 4 (its party) +1 from its own trigger, and flying.
    assert_eq!(t.pt(archpriest), (6, 3));
    assert!(has(&t, archpriest, mtg_engine::keywords::KeywordKind::Flying));

    // Three (no Wizard): none of them trigger.
    let mut t = TestGame::new(2);
    let commander = t.battlefield(P0, "Squad Commander");
    let archpriest = t.battlefield(P0, "Archpriest of Iona");
    t.battlefield(P0, "Nimble Trapfinder");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(!indestructible(&t, commander));
    assert_eq!(t.pt(archpriest), (3, 2));
}

#[test]
fn sylvan_awakening_affects_only_lands_controlled_as_it_resolves_and_doesnt_untap() {
    cr!("611.2c");
    ruling!(
        "Sylvan Awakening",
        "Sylvan Awakening affects only lands you control at the time it resolves."
    );
    ruling!(
        "Sylvan Awakening",
        "Sylvan Awakening doesn’t untap any of the lands that become creatures."
    );
    supported("Sylvan Awakening");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    cast_new(&mut t, P0, "Sylvan Awakening", &[]);
    t.resolve_all();
    let paid: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.tapped)
        .map(|o| o.id)
        .collect();
    assert_eq!(paid.len(), 3);
    for l in paid.iter().chain([&forest]) {
        assert!(t.obj_now(*l).is(mtg_engine::types::CardType::Creature));
        assert_eq!(t.pt(*l), (2, 2));
        assert!(indestructible(&t, *l));
    }
    assert!(paid.iter().all(|l| t.obj_now(*l).tapped), "they stay tapped");
    let later = t.battlefield(P0, "Plains");
    assert!(!t.obj_now(later).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn sylvan_awakening_ends_as_your_next_turn_begins() {
    cr!("611.2b", "502.1");
    ruling!(
        "Sylvan Awakening",
        "The lands affected by Sylvan Awakening stop being creatures as your next untap step begins, before you untap your permanents."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    cast_new(&mut t, P0, "Sylvan Awakening", &[]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn smugglers_surprise_last_mode_affects_creatures_as_it_resolves() {
    cr!("611.2c", "702.172a");
    ruling!(
        "Smuggler's Surprise",
        "The effect of Smuggler's Surprise's last mode affects only creatures you control at the time it resolves."
    );
    supported("Smuggler's Surprise");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let spell = t.hand(P0, "Smuggler's Surprise");
    t.cast(P0, spell).modes(&[2]).go();
    t.resolve_all();
    assert!(indestructible(&t, wurm) && hexproof(&t, wurm));
    assert!(!indestructible(&t, bears), "power less than 4");
    let later = t.battlefield(P0, "Craw Wurm");
    assert!(!indestructible(&t, later));

    // With the second mode too, creatures put onto the battlefield that way with power 4
    // or greater are affected.
    let mut t = TestGame::new(2);
    let wurm = t.hand(P0, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 5);
    let spell = t.hand(P0, "Smuggler's Surprise");
    t.answer_choose(P0, &[obj(wurm)]);
    t.cast(P0, spell).modes(&[1, 2]).go();
    t.resolve_all();
    let wurm = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurm.len(), 1);
    assert!(indestructible(&t, wurm[0]) && hexproof(&t, wurm[0]));
}

#[test]
fn bleeding_effect_is_determined_as_it_resolves() {
    cr!("611.2c", "603.2");
    ruling!(
        "Bleeding Effect",
        "The set of creatures affected by Bleeding Effect’s ability and how they are affected is determined as the ability resolves."
    );
    supported("Bleeding Effect");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bleeding Effect");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Birds of Paradise");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    use mtg_engine::keywords::KeywordKind::*;
    assert!(has(&t, bears, Flying));
    assert!(!has(&t, bears, Trample));
    // A creature card with trample enters the graveyard, the flier leaves: nothing changes.
    t.graveyard(P0, "Colossal Dreadmaw");
    let birds = t.g.players[0].graveyard.iter().copied().find(|id| t.g.obj(*id).chars.name == "Birds of Paradise").unwrap();
    t.g.exile_object(birds, None);
    t.g.recompute();
    assert!(has(&t, bears, Flying));
    assert!(!has(&t, bears, Trample));
    let later = newcomer(&mut t);
    assert!(!has(&t, later, Flying));
}

#[test]
fn jirina_affects_only_humans_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Jirina, Dauntless General",
        "The set of creatures affected by Jirina's last ability is determined as the ability resolves."
    );
    supported("Jirina, Dauntless General");
    let mut t = TestGame::new(2);
    let jirina = t.battlefield(P0, "Jirina, Dauntless General");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, jirina, 0, &[]).expect("activate");
    t.resolve_all();
    assert!(indestructible(&t, sorcerer) && hexproof(&t, sorcerer));
    assert!(!indestructible(&t, bears));
    let later = t.battlefield(P0, "Savannah Lions");
    let later_human = t.battlefield(P0, "Prodigal Sorcerer");
    assert!(!indestructible(&t, later) && !indestructible(&t, later_human));
}

#[test]
fn selfless_spirit_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Selfless Spirit",
        "The set of creatures affected by Selfless Spirit's last ability is determined as the ability resolves."
    );
    supported("Selfless Spirit");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Selfless Spirit");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, spirit, 0, &[]).expect("activate");
    t.resolve_all();
    assert!(indestructible(&t, bears));
    let later = newcomer(&mut t);
    assert!(!indestructible(&t, later));
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
}

#[test]
fn heroic_intervention_affects_only_permanents_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Heroic Intervention",
        "The set of permanents affected by Heroic Intervention is determined as the spell resolves."
    );
    supported("Heroic Intervention");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let relic = t.battlefield(P0, "Darksteel Relic");
    cast_new(&mut t, P0, "Heroic Intervention", &[]);
    t.resolve_all();
    assert!(indestructible(&t, bears) && hexproof(&t, bears));
    assert!(hexproof(&t, relic));
    let later = newcomer(&mut t);
    assert!(!hexproof(&t, later));
}

#[test]
fn selfless_glyphweaver_only_from_the_battlefield_and_only_current_creatures() {
    cr!("611.2c", "602.1");
    ruling!(
        "Selfless Glyphweaver // Deadly Vanity",
        "You may activate Selfless Glyphweaver’s ability only if Selfless Glyphweaver is on the battlefield."
    );
    supported("Selfless Glyphweaver // Deadly Vanity");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Selfless Glyphweaver // Deadly Vanity");
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, in_hand));
    let weaver = t.battlefield(P0, "Selfless Glyphweaver // Deadly Vanity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, weaver, 0, &[]).expect("activate");
    assert!(t.in_exile("Selfless Glyphweaver"));
    t.resolve_all();
    assert!(indestructible(&t, bears));
    let later = newcomer(&mut t);
    assert!(!indestructible(&t, later));
}

// --- Regeneration --------------------------------------------------------------------------

#[test]
fn loxodon_hierarch_sets_up_a_shield_for_each_creature() {
    cr!("701.19a", "701.19b");
    ruling!(
        "Loxodon Hierarch",
        "The second ability sets up individual regeneration shields for each creature you control."
    );
    supported("Loxodon Hierarch");
    let mut t = TestGame::new(2);
    let hierarch = t.battlefield(P0, "Loxodon Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.activate(P0, hierarch, 0, &[]).expect("activate");
    t.resolve_all();
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears) && t.obj_now(bears).tapped, "regenerated");
    // The Bears' shield is used up; the Giant's isn't.
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
    destroy(&mut t, giant);
    assert!(t.on_battlefield(giant));
    // The shields wear off when the turn ends.
    t.advance_to(P1, Step::Upkeep);
    destroy(&mut t, giant);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn wrap_in_vigor_shields_creatures_controlled_as_it_resolves() {
    cr!("701.19a", "611.2c");
    ruling!(
        "Wrap in Vigor",
        "Wrap in Vigor sets up a separate regeneration shield on each creature you control at the time it resolves."
    );
    supported("Wrap in Vigor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Wrap in Vigor", &[]);
    t.resolve_all();
    let later = newcomer(&mut t);
    destroy(&mut t, bears);
    destroy(&mut t, giant);
    destroy(&mut t, later);
    assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
    assert!(!t.on_battlefield(later));
}

#[test]
fn asceticism_can_regenerate_any_creature() {
    cr!("701.19a", "115.1");
    ruling!(
        "Asceticism",
        "You may target any creature with the regeneration ability, not just one you control."
    );
    supported("Asceticism");
    let mut t = TestGame::new(2);
    let asceticism = t.battlefield(P0, "Asceticism");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Forest", 2);
    t.activate(P0, asceticism, 0, &[obj(ogre)]).expect("activate");
    t.resolve_all();
    destroy(&mut t, ogre);
    assert!(t.on_battlefield(ogre));
}

#[test]
fn resistance_reunited_equipped_set_is_fixed_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Resistance Reunited",
        "The set of creatures that gains indestructible is determined as Resistance Reunited resolves. After that point, an equipped creature that gained indestructible won't lose indestructible if it becomes unequipped."
    );
    supported("Resistance Reunited");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Gray Ogre");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.g.attach(blade, obj(bears));
    let their_blade = t.battlefield(P1, "Bonesplitter");
    t.g.attach(their_blade, obj(theirs));
    t.g.recompute();
    cast_new(&mut t, P0, "Resistance Reunited", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    assert!(indestructible(&t, bears));
    assert!(!indestructible(&t, giant), "not equipped");
    assert!(!indestructible(&t, theirs), "not P0's");
    // The Bonesplitter moves to the Giant.
    t.g.attach(blade, obj(giant));
    t.g.recompute();
    assert!(indestructible(&t, bears), "keeps it unequipped");
    assert!(!indestructible(&t, giant), "doesn't gain it equipped");
}

#[test]
fn paladin_danse_affects_artifact_or_human_creatures_as_it_resolves() {
    cr!("611.2c", "602.2");
    ruling!(
        "Paladin Danse, Steel Maverick",
        "The set of creatures affected by Paladin Danse's last ability is determined as the ability resolves."
    );
    supported("Paladin Danse, Steel Maverick");
    let mut t = TestGame::new(2);
    let danse = t.battlefield(P0, "Paladin Danse, Steel Maverick");
    let thopter = t.battlefield(P0, "Ornithopter");
    let human = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, danse, 0, &[]).expect("activate");
    assert!(t.in_exile("Paladin Danse, Steel Maverick"));
    t.resolve_all();
    assert!(indestructible(&t, thopter), "an artifact creature");
    assert!(indestructible(&t, human), "a Human creature");
    assert!(!indestructible(&t, bears), "neither");
    let later_thopter = t.battlefield(P0, "Ornithopter");
    let later_human = t.battlefield(P0, "Prodigal Sorcerer");
    assert!(!indestructible(&t, later_thopter));
    assert!(!indestructible(&t, later_human));
}
