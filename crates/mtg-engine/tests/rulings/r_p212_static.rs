//! Rulings batch P212 — Equipment (CR 301.5, 702.6) bonuses and abilities: layer-6
//! timestamps for "loses flying" (CR 613.7a, 613.7e), bonuses that keep updating
//! (CR 611.3a), abilities granted to the equipped creature (CR 113.7, 602.2) and whose
//! costs only the creature's controller can pay (CR 701.21a), and protection, hexproof
//! and ward granted by Equipment (CR 702.11, 702.16, 702.21).

use crate::r_p210_common::deal;
use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{ability_targets, spell_targets};
use crate::r_s06_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn custom(t: &mut TestGame, p: PlayerId, type_line: &str, pt: (i32, i32), text: &str) -> ObjectId {
    let def = custom_card("Custom", type_line, "{1}", Some(pt), text);
    t.custom(p, def, Zone::Battlefield)
}

// ---------------------------------------------------------------------------------------
// "Loses flying" (Colossus Hammer, Starforged Sword, Magebane Armor)
// ---------------------------------------------------------------------------------------

#[test]
fn loses_flying_beats_the_creatures_own_as_long_as_flying() {
    cr!("613.1f", "613.7a", "613.7e");
    ruling!(
        "Starforged Sword",
        "If an ability of the equipped creature states that it has flying \"as long as\" a condition is true, that condition becoming true won't cause the creature to gain flying."
    );
    ruling!(
        "Colossus Hammer",
        "If an ability of the equipped creature states that it has flying \"as long as\" a particular condition is true, that condition becoming true will not cause that creature to gain flying."
    );
    supported("Starforged Sword");
    supported("Colossus Hammer");
    for eq in [None, Some("Starforged Sword"), Some("Colossus Hammer")] {
        let mut t = TestGame::new(2);
        let c = custom(
            &mut t,
            P0,
            "Creature — Bird",
            (1, 1),
            "This creature has flying as long as it's attacking.",
        );
        if let Some(eq) = eq {
            attach_new(&mut t, P0, eq, c);
        }
        assert!(!has_kw(&t, c, KeywordKind::Flying));
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        t.g.recompute();
        assert_eq!(has_kw(&t, c, KeywordKind::Flying), eq.is_none(), "{eq:?}");
    }
}

#[test]
fn flying_gained_after_the_equipment_attached_works() {
    cr!("613.1f", "613.7b", "613.7d");
    ruling!(
        "Colossus Hammer",
        "If the equipped creature gains flying after Colossus Hammer became attached to it, it will keep flying."
    );
    ruling!(
        "Starforged Sword",
        "If the equipped creature gains flying after Starforged Sword became attached to it, it will have flying."
    );
    ruling!(
        "Magebane Armor",
        "Magebane Armor causes the equipped creature to lose flying at the time it became equipped. Any ability that grants that creature flying that starts to work later (such as a Jump that resolves later, or a Levitation that comes under your control later) will work normally."
    );
    supported("Magebane Armor");
    supported("Jump");
    supported("Levitation");
    for eq in ["Colossus Hammer", "Starforged Sword", "Magebane Armor"] {
        // A Jump that resolves later.
        let mut t = TestGame::new(2);
        let drake = t.battlefield(P0, "Wind Drake");
        attach_new(&mut t, P0, eq, drake);
        assert!(!has_kw(&t, drake, KeywordKind::Flying), "{eq}");
        crate::r_p209_common::cast_spell(&mut t, P0, "Jump", &[Entity::Object(drake)]);
        t.resolve_all();
        assert!(has_kw(&t, drake, KeywordKind::Flying), "{eq}");
        // A Levitation that enters later.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, eq, bears);
        t.battlefield(P0, "Levitation");
        t.g.recompute();
        assert!(has_kw(&t, bears, KeywordKind::Flying), "{eq}");
        // A Levitation already there: the Equipment's later timestamp wins.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Levitation");
        attach_new(&mut t, P0, eq, bears);
        assert!(!has_kw(&t, bears, KeywordKind::Flying), "{eq}");
    }
}

#[test]
fn magebane_armor_on_a_creature_without_flying() {
    cr!("301.5", "613.1f");
    ruling!(
        "Magebane Armor",
        "Magebane Armor can equip a creature that doesn’t have flying. The “lose flying” effect just won’t do anything in that case."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let armor = t.battlefield(P0, "Magebane Armor");
    t.lands(P0, "Wastes", 2);
    crate::r_p213_common::equip(&mut t, P0, armor, bears).unwrap();
    t.resolve_all();
    assert!(on(&t, armor, bears));
    assert_eq!(t.pt(bears), (4, 6));
}

// ---------------------------------------------------------------------------------------
// Bonuses
// ---------------------------------------------------------------------------------------

#[test]
fn equipment_bonuses_that_count_keep_updating() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Nettlecyst",
        "If a permanent you control is both an artifact and an enchantment, count it only once when determining the bonus from an equipped Nettlecyst."
    );
    ruling!(
        "Civic Saber",
        "If the equipped creature becomes a different number of colors, the bonus will change accordingly."
    );
    ruling!(
        "Warmonger's Chariot",
        "If the equipped creature doesn’t have defender, the second ability simply doesn’t affect it."
    );
    for c in ["Nettlecyst", "Civic Saber", "Warmonger's Chariot"] {
        supported(c);
    }
    // Nettlecyst: itself (an artifact), then an artifact enchantment counts once.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Nettlecyst", bears);
    assert_eq!(t.pt(bears), (3, 3));
    let ae = custom_card("Relic", "Artifact Enchantment", "{1}", None, "");
    t.custom(P0, ae, Zone::Battlefield);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    // Civic Saber: one color, then three.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Civic Saber", bears);
    assert_eq!(t.pt(bears), (3, 2));
    let wub = crate::r_s17_common::color_set(&[Color::White, Color::Blue, Color::Black]);
    modify_until_eot(&mut t, bears, vec![Modification::SetColors(wub)]);
    assert_eq!(t.pt(bears), (5, 2));
    modify_until_eot(&mut t, bears, vec![Modification::SetColors(ColorSet::NONE)]);
    assert_eq!(t.pt(bears), (2, 2));
    // Warmonger's Chariot: a creature without defender just gets +2/+2 and attacks as
    // usual; one with defender can attack.
    for name in ["Grizzly Bears", "Wall of Stone"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        attach_new(&mut t, P0, "Warmonger's Chariot", c);
        let base = if name == "Grizzly Bears" { (4, 4) } else { (2, 10) };
        assert_eq!(t.pt(c), base);
        crate::r_s09_common::to_combat(&mut t, P0);
        assert!(crate::r_s02_common::can_attack(&mut t, c), "{name}");
    }
}

#[test]
fn pact_weapon_replaced_draw_gives_no_bonus_and_no_life_loss() {
    cr!("614.11", "121.6");
    ruling!(
        "Pact Weapon",
        "If the card draw is replaced with another effect, the equipped creature won't get any bonus to its power and toughness and you won't lose any life. This is true even if the card draw is replaced by one or more card draws."
    );
    supported("Pact Weapon");
    supported("Abundance");
    for replaced in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Pact Weapon", bears);
        if replaced {
            t.battlefield(P0, "Abundance");
            t.answer_yes(P0, true);
        }
        stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm", "Forest"]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.resolve_all();
        if replaced {
            assert_eq!(t.pt(bears), (2, 2));
            assert_eq!(t.life(P0), 20);
        } else {
            assert!(t.in_hand(P0, "Hill Giant"));
            assert_eq!(t.pt(bears), (6, 6));
            assert_eq!(t.life(P0), 16);
        }
    }
}

#[test]
fn dragon_throne_negative_power_counts_as_zero() {
    cr!("107.1b");
    ruling!(
        "Dragon Throne of Tarkir",
        "If the equipped creature's power is negative while resolving the activated ability it gains, X is considered to be 0."
    );
    supported("Dragon Throne of Tarkir");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Dragon Throne of Tarkir", bears);
    let lion = t.battlefield(P0, "Silvercoat Lion");
    boost(&mut t, bears, -4, 0);
    assert_eq!(t.pt(bears).0, -2);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, bears, "trample").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(lion), (2, 2));
    assert!(has_kw(&t, lion, KeywordKind::Trample));
}

#[test]
fn skyclave_pick_axe_with_no_equipped_creature_gives_nothing() {
    cr!("608.2b", "301.5a");
    ruling!(
        "Skyclave Pick-Axe",
        "If there's no equipped creature as Skyclave Pick-Axe's landfall ability resolves, no creature gets +2/+2."
    );
    supported("Skyclave Pick-Axe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    attach_new(&mut t, P0, "Skyclave Pick-Axe", bears);
    crate::r_s05_common::enter(&mut t, P0, "Forest");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(lion), (2, 2));
}

// ---------------------------------------------------------------------------------------
// Abilities granted to the equipped creature
// ---------------------------------------------------------------------------------------

#[test]
fn granted_abilities_are_the_creatures() {
    cr!("113.7", "602.2", "702.16b");
    ruling!(
        "Leonin Bola",
        "Leonin Bola grants an ability to the equipped creature. The ability can target a creature with protection from artifacts (unless the equipped creature is an artifact creature)."
    );
    ruling!("Surestrike Trident", "Surestrike Trident grants an ability to the equipped creature.");
    supported("Leonin Bola");
    supported("Surestrike Trident");
    // Leonin Bola: a nonartifact creature's ability can target Tel-Jilad Chosen; an
    // artifact creature's can't.
    for (name, can) in [("Grizzly Bears", true), ("Ornithopter", false)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        attach_new(&mut t, P0, "Leonin Bola", c);
        let chosen = t.battlefield(P1, "Tel-Jilad Chosen");
        let i = granted(&t, c, "Unattach");
        let targets = ability_targets(&mut t, c, i);
        assert_eq!(targets.contains(&Entity::Object(chosen)), can, "{name}");
    }
    // Surestrike Trident: Child of Night (lifelink) deals the damage: P0 gains life.
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Night");
    let trident = attach_new(&mut t, P0, "Surestrike Trident", child);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, child, "Unattach").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    assert_eq!(attached_to(&t, trident), None);
}

/// The index among `id`'s activated abilities of the one whose text contains `needle`.
fn granted(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    use mtg_engine::ability::AbilityKind;
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(needle))
        .expect("granted ability")
}

#[test]
fn only_the_creatures_controller_activates_and_only_by_paying_what_they_control() {
    cr!("602.2", "701.21a", "301.5d");
    ruling!(
        "Wolfhunter's Quiver",
        "Only the equipped creature’s controller can activate the abilities it gains from Wolfhunter’s Quiver."
    );
    ruling!(
        "Blazing Torch",
        "If a Blazing Torch controlled by one player somehow winds up equipping a creature a different player controls, the damage ability can’t be activated by either player. Only the creature’s controller may activate the ability — but since that player can’t sacrifice Blazing Torch (a permanent they don’t control), the ability’s cost can’t be paid."
    );
    ruling!(
        "Citizen's Crowbar",
        "If another player gains control of the equipped creature but doesn't gain control of the Equipment, they will not be able to activate the ability the Equipment grants because they can't pay a cost that includes sacrificing a permanent they don't control."
    );
    supported("Wolfhunter's Quiver");
    supported("Citizen's Crowbar");
    // Wolfhunter's Quiver (P0's) on P1's creature: P1 activates; P0 can't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Wolfhunter's Quiver", bears);
    t.g.recompute();
    let i = granted(&t, bears, "1 damage");
    assert!(t.activate(P0, bears, i, &[Entity::Player(P1)]).is_err());
    t.activate(P1, bears, i, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // Blazing Torch and Citizen's Crowbar (P0's) on a creature P1 controls: neither
    // player can activate the granted ability.
    for (eq, target_name) in [("Blazing Torch", None), ("Citizen's Crowbar", Some("Bonesplitter"))] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let e = attach_new(&mut t, P0, eq, bears);
        give_control(&mut t, bears, P1);
        assert_eq!(t.obj_now(e).controller, P0);
        t.lands(P0, "Plains", 1);
        t.lands(P1, "Plains", 1);
        let target = match target_name {
            Some(n) => Entity::Object(t.battlefield(P1, n)),
            None => Entity::Player(P0),
        };
        t.g.recompute();
        let i = granted(&t, bears, "Sacrifice");
        for p in [P0, P1] {
            assert!(t.activate(p, bears, i, &[target]).is_err(), "{eq} by {p:?}");
        }
        assert!(t.on_battlefield(e), "{eq}");
    }
}

#[test]
fn assault_suit_you_must_sacrifice_another_creature() {
    cr!("701.21a", "101.3");
    ruling!(
        "Assault Suit",
        "If an effect instructs you to sacrifice a creature, and you control any creatures other than the creature equipped with Assault Suit, you must sacrifice one of them. You can't try to sacrifice the equipped creature, fail, and therefore ignore the effect."
    );
    supported("Assault Suit");
    supported("Diabolic Edict");
    for other in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Assault Suit", bears);
        if other {
            t.battlefield(P0, "Silvercoat Lion");
        }
        // P0 tries to pick the Bears.
        t.answer_choose(P0, &[Entity::Object(bears)]);
        crate::r_p209_common::cast_spell(&mut t, P1, "Diabolic Edict", &[Entity::Player(P0)]);
        t.resolve_all();
        assert!(t.on_battlefield(bears));
        assert_eq!(t.in_graveyard(P0, "Silvercoat Lion"), other);
    }
}

// ---------------------------------------------------------------------------------------
// Protection, hexproof and ward
// ---------------------------------------------------------------------------------------

#[test]
fn champions_helm_on_an_opponents_creature_stops_you_targeting_it() {
    cr!("702.11b", "301.5d");
    ruling!(
        "Champion's Helm",
        "It's possible for a Champion's Helm you control to be attached to a creature an opponent controls (if an opponent gains control of your equipped creature, for example). In this case, the creature couldn't be the target of spells or abilities you control, as you are an opponent of the controller of the creature with hexproof."
    );
    supported("Champion's Helm");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, "Champion's Helm", isamaru);
    assert!(spell_targets(&mut t, P1, "Lightning Bolt").iter().all(|e| *e != Entity::Object(isamaru)));
    give_control(&mut t, isamaru, P1);
    assert!(!spell_targets(&mut t, P0, "Lightning Bolt").contains(&Entity::Object(isamaru)));
    assert!(spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Object(isamaru)));
}

#[test]
fn armor_ward_and_the_creatures_own_ward_both_trigger() {
    cr!("702.21a", "702.21b");
    ruling!(
        "Leather Armor",
        "If a creature with ward is equipped with Leather Armor, becoming the target of a spell or ability an opponent controls will cause both ward abilities to trigger."
    );
    ruling!(
        "Plate Armor",
        "If a creature with ward is equipped with Plate Armor, becoming the target of a spell or ability an opponent controls will cause both ward abilities to trigger."
    );
    for eq in ["Leather Armor", "Plate Armor"] {
        supported(eq);
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let c = custom(&mut t, P0, "Creature — Human", (1, 1), "Ward {2}");
        attach_new(&mut t, P0, eq, c);
        t.lands(P1, "Mountain", 4);
        crate::r_p209_common::cast_spell(&mut t, P1, "Lightning Bolt", &[Entity::Object(c)]);
        t.settle();
        assert_eq!(t.stack_len(), 3, "{eq}");
        // P1 pays {2} and {1}: Bolt resolves.
        t.answer_yes(P1, true);
        t.answer_yes(P1, true);
        t.resolve_all();
        // The Bolt resolved.
        assert!(!t.on_battlefield(c) || t.obj_now(c).damage == 3, "{eq}");
    }
}

#[test]
fn sword_of_wealth_and_power_protection_from_instants_and_sorceries() {
    cr!("702.16b", "702.16e", "702.16a");
    ruling!(
        "Sword of Wealth and Power",
        "Protection from instants and from sorceries means that the equipped creature can’t be the target of instant spells, sorcery spells, or abilities from instant or sorcery cards (such as an ability of an instant card that triggers when that card is cycled), and all damage that an instant or sorcery spell or card would deal to it is prevented. Nothing other than these events is prevented or illegal."
    );
    supported("Sword of Wealth and Power");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Sword of Wealth and Power", wurm);
    // Not a legal target for an instant or sorcery spell.
    assert!(!spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Object(wurm)));
    assert!(!spell_targets(&mut t, P1, "Searing Spear").contains(&Entity::Object(wurm)));
    // A creature's ability can target it and deal damage.
    let shocker = t.battlefield(P1, "Prodigal Pyromancer");
    let i = granted(&t, shocker, "damage");
    assert!(ability_targets(&mut t, shocker, i).contains(&Entity::Object(wurm)));
    deal(&mut t, shocker, 1, wurm);
    assert_eq!(t.obj_now(wurm).damage, 1);
    // A sorcery's damage that doesn't target it is prevented.
    t.set_step(P1, Step::PrecombatMain);
    cast_x(&mut t, P1, "Earthquake", 3);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 1);
    assert_eq!(t.life(P0), 17);
}

// ---------------------------------------------------------------------------------------
// Eater of Virtue
// ---------------------------------------------------------------------------------------

#[test]
fn eater_of_virtue_grants_hexproof_and_protection_from_exiled_cards() {
    cr!("702.11a", "702.16a", "607.2a");
    ruling!(
        "Eater of Virtue",
        "The Equipped creature gains any hexproof and protection abilities of cards exiled with Eater of Virtue."
    );
    ruling!(
        "Eater of Virtue",
        "If Eater of Virtue leaves the battlefield and returns to the battlefield, it will be a new object with no association to any cards it exiled from any previous times it was on the battlefield. It will have to exile new cards to grant abilities to creatures it equips from that point forward."
    );
    supported("Eater of Virtue");
    let mut t = TestGame::new(2);
    let eater = t.battlefield(P0, "Eater of Virtue");
    for name in ["Gladecover Scout", "Paladin en-Vec"] {
        let c = t.battlefield(P0, name);
        move_eq(&mut t, eater, c);
        destroy(&mut t, c);
        t.resolve_all();
        assert!(t.in_exile(name));
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    move_eq(&mut t, eater, bears);
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    assert!(has_kw(&t, bears, KeywordKind::Protection));
    // Not targetable by P1; black spells can't target it (protection from black).
    assert!(!spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Object(bears)));
    // The Eater leaves and returns: a new object with nothing exiled.
    crate::r_s05_common::move_to(&mut t, eater, Zone::Hand(P0));
    let eater = t.battlefield(P0, "Eater of Virtue");
    move_eq(&mut t, eater, bears);
    assert!(on(&t, eater, bears));
    assert!(!has_kw(&t, bears, KeywordKind::Hexproof));
    assert!(!has_kw(&t, bears, KeywordKind::Protection));
    assert_eq!(t.pt(bears), (4, 2));
}
