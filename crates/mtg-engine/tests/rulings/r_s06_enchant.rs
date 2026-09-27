//! Rulings batch S06 — enchant (CR 702.5, 303): Auras with umbra armor, color-dependent
//! bonuses, Runes, Cartouches, "draw a card" Auras, activated-ability locks, type-changing
//! Auras, "attacks each combat if able", infect, and control-changing Auras.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::ability::{AbilityKind, Effect, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Umbra armor
// ---------------------------------------------------------------------------------------

#[test]
fn a_destroy_spell_destroys_the_umbra_itself_but_lethal_damage_leaves_it_to_the_sbas() {
    cr!("702.89a", "614.6", "704.5g");
    ruling!(
        "Hyena Umbra",
        "If a spell or ability says that it would \"destroy\" a creature enchanted with an Aura that has umbra armor, that spell or ability is what causes the Aura to be destroyed instead. Umbra armor doesn't destroy the Aura; rather, it changes the effects of the spell or ability. On the other hand, if a spell or ability deals lethal damage to a creature enchanted with an Aura that has umbra armor, the game rules regarding lethal damage cause the Aura to be destroyed, not that spell or ability."
    );
    supported("Hyena Umbra");
    // Murder: the Aura is destroyed as part of Murder's resolution, before any
    // state-based action is checked.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = attach_new(&mut t, P0, "Hyena Umbra", bears);
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(bears).go();
    t.g.resolve_top();
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    assert!(!t.on_battlefield(umbra));
    assert!(t.on_battlefield(bears));
    // Lightning Bolt: the Bolt only deals damage; the Aura is still on the battlefield
    // after it resolves, and is destroyed when state-based actions are checked.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = attach_new(&mut t, P0, "Hyena Umbra", bears);
    assert_eq!(t.pt(bears), (3, 3));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.g.resolve_top();
    assert!(t.on_battlefield(umbra));
    assert_eq!(t.obj_now(bears).damage, 3);
    t.settle();
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

// ---------------------------------------------------------------------------------------
// Color-dependent bonuses (Shadowmoor hybrid Auras)
// ---------------------------------------------------------------------------------------

#[test]
fn a_creature_of_both_colors_gets_both_bonuses() {
    cr!("303.4", "611.3a", "613.4c");
    ruling!(
        "Steel of the Godhead",
        "If the enchanted creature is both of the listed colors, it will get both bonuses."
    );
    supported("Steel of the Godhead");
    // Steel of the Godhead: white → +1/+1 and lifelink; blue → +1/+1 and can't be blocked.
    // Azorius Guildmage is white and blue.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Azorius Guildmage");
    attach_new(&mut t, P0, "Steel of the Godhead", mage);
    assert_eq!(t.pt(mage), (4, 4));
    assert!(has_kw(&t, mage, KeywordKind::Lifelink));
    let wall = t.battlefield(P1, "Hill Giant");
    t.attack(&[(mage, Entity::Player(P1))], &[(wall, mage)]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert!(t.on_battlefield(wall));
    // A white-only creature gets only the white bonus.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Savannah Lions");
    attach_new(&mut t, P0, "Steel of the Godhead", knight);
    assert_eq!(t.pt(knight), (3, 2));
    assert!(has_kw(&t, knight, KeywordKind::Lifelink));
    let giant = t.battlefield(P1, "Hill Giant");
    t.attack(&[(knight, Entity::Player(P1))], &[(giant, knight)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Savannah Lions"));
}

// ---------------------------------------------------------------------------------------
// Aura spells whose target is illegal as they resolve
// ---------------------------------------------------------------------------------------

/// Casts the Aura `name` (with the mana for it) targeting `target`, removes the target
/// with the spell on the stack, and resolves the spell. The Aura must not resolve, enter,
/// or trigger.
fn aura_fizzles(name: &str) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, name);
    t.cast(P0, aura).target(bears).go();
    let hand = t.hand_size(P0);
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve();
    t.resolve_all();
    assert!(t.in_graveyard(P0, name), "{name} should be in the graveyard");
    assert!(t.named_on_battlefield(name).is_empty());
    // The bounced Grizzly Bears is the only new card in hand: no card was drawn.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_rune_whose_target_is_illegal_doesnt_resolve_or_draw() {
    cr!("608.2b", "608.3b");
    ruling!(
        "Rune of Might",
        "If the target of an Aura spell is an illegal target as that spell tries to resolve, it won't resolve, it won't enter the battlefield, and none of its enters-the-battlefield triggered abilities will trigger."
    );
    supported("Rune of Might");
    aura_fizzles("Rune of Might");
}

#[test]
fn a_cartouche_whose_target_is_illegal_doesnt_resolve() {
    cr!("608.2b", "608.3b");
    ruling!(
        "Cartouche of Knowledge",
        "If the target creature becomes an illegal target before a Cartouche spell resolves, the spell doesn't resolve. It doesn't enter the battlefield."
    );
    supported("Cartouche of Knowledge");
    aura_fizzles("Cartouche of Knowledge");
}

#[test]
fn an_aura_whose_target_is_illegal_doesnt_let_you_draw() {
    cr!("608.2b", "608.3b");
    ruling!(
        "Chosen by Heliod",
        "If the target of an Aura is illegal when it tries to resolve, the Aura won't resolve. The Aura doesn't enter the battlefield, so you won't get to draw a card."
    );
    supported("Chosen by Heliod");
    aura_fizzles("Chosen by Heliod");
}

#[test]
fn a_rune_can_enchant_a_vehicle_and_applies_once_it_is_a_creature() {
    cr!("303.4a", "301.7", "702.122a");
    ruling!(
        "Rune of Might",
        "Runes can target and be attached to any permanent, even one that isn't currently a creature or an Equipment. You can cast a Rune choosing a Vehicle as the target, for example. The Rune's enters-the-battlefield ability will trigger, and you'll draw a card. The Rune won't do anything while the Vehicle isn't a creature, but if it becomes one later, the appropriate ability will start applying."
    );
    // Rune of Might: "As long as enchanted permanent is a creature, it gets +1/+1 and has
    // trample."
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rune = in_hand_with_mana(&mut t, P0, "Rune of Might");
    t.cast(P0, rune).target(copter).go();
    let hand = t.hand_size(P0);
    t.resolve();
    t.resolve_all();
    let rune = t.named_on_battlefield("Rune of Might")[0];
    assert_eq!(t.g.obj(rune).attached_to, Some(Entity::Object(copter)));
    assert_eq!(t.hand_size(P0), hand + 1);
    // A noncreature permanent has no power or toughness (CR 208.3), and no trample.
    assert!(!is_creature(&t, copter));
    assert_eq!(t.obj_now(copter).chars.power, None);
    assert!(!has_kw(&t, copter, KeywordKind::Trample));
    // Crewed, the Vehicle is a creature: the bonus starts applying.
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (4, 4));
    assert!(has_kw(&t, copter, KeywordKind::Trample));
}

// ---------------------------------------------------------------------------------------
// Cartouches
// ---------------------------------------------------------------------------------------

#[test]
fn cartouche_is_an_enchantment_subtype_other_cards_care_about() {
    cr!("205.3h", "205.3a");
    ruling!(
        "Cartouche of Strength",
        "Cartouche is an enchantment subtype with no special meaning. Other cards may care about which enchantments are Cartouches."
    );
    supported("Trial of Strength");
    assert_eq!(subtype_kinds("Cartouche"), vec![SubtypeKind::Enchantment]);
    // Trial of Strength: "When a Cartouche you control enters, return this enchantment to
    // its owner's hand."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trial of Strength");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A non-Cartouche Aura entering doesn't return it.
    let rancor = in_hand_with_mana(&mut t, P0, "Rancor");
    t.cast(P0, rancor).target(bears).go();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Trial of Strength").is_empty());
    let cartouche = in_hand_with_mana(&mut t, P0, "Cartouche of Strength");
    t.cast(P0, cartouche).target(bears).go();
    t.resolve_all();
    let c = t.named_on_battlefield("Cartouche of Strength")[0];
    assert!(t.g.obj(c).chars.has_subtype("Cartouche"));
    assert!(t.named_on_battlefield("Trial of Strength").is_empty());
    assert!(t.in_hand(P0, "Trial of Strength"));
}

#[test]
fn a_cartouche_falls_off_if_only_one_of_it_and_the_creature_changes_control() {
    cr!("303.4c", "704.5m");
    ruling!(
        "Cartouche of Knowledge",
        "Each Cartouche is an Aura with \"Enchant creature you control.\" If another player gains control of either the Cartouche or the enchanted creature (but not both), then the Cartouche will be enchanting an illegal permanent and be put into its owner's graveyard as a state-based action."
    );
    // The opponent gains control of the creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Cartouche of Knowledge", bears);
    t.settle();
    assert!(!t.named_on_battlefield("Cartouche of Knowledge").is_empty());
    give_control(&mut t, bears, P1);
    assert!(t.in_graveyard(P0, "Cartouche of Knowledge"));
    // The opponent gains control of the Cartouche.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = attach_new(&mut t, P0, "Cartouche of Knowledge", bears);
    give_control(&mut t, c, P1);
    assert!(t.in_graveyard(P0, "Cartouche of Knowledge"));
    // Both change control: it stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = attach_new(&mut t, P0, "Cartouche of Knowledge", bears);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(bears)], vec![Entity::Object(c)]];
    for slot in 0..2u8 {
        t.g.exec(
            &Effect::GainControl {
                what: Sel::Target(slot),
                who: mtg_engine::ability::PlayerRef::You,
                duration: mtg_engine::ability::Duration::Permanent,
            },
            &mut ctx,
        );
    }
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).controller, P1);
    assert_eq!(t.pt(bears), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Activated-ability locks
// ---------------------------------------------------------------------------------------

/// The texts of the activated abilities `p` could activate of `source` now.
fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId) -> Vec<String> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let s = t.g.current(source);
    t.g.activatable_abilities(p)
        .into_iter()
        .filter(|(id, _)| *id == s)
        .map(|(_, a)| a.text.to_string())
        .collect()
}

#[test]
fn bound_in_gold_stops_equip_and_loyalty_abilities_but_not_mana_abilities() {
    cr!("602.1", "602.5", "606.2", "702.6a", "605.1a");
    ruling!(
        "Bound in Gold",
        "Activated abilities contain a colon and appear in the form “[Cost]: [Effect].” Some keywords (such as equip) are activated abilities and will have colons in their reminder texts. Loyalty abilities of planeswalkers are also activated abilities."
    );
    supported("Bound in Gold");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    // A creature for Bonesplitter's equip ability to target.
    t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let lili = t.battlefield(P0, "Liliana of the Veil");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(!activatable(&mut t, P0, blade).is_empty());
    assert!(!activatable(&mut t, P0, lili).is_empty());
    assert!(!activatable(&mut t, P0, elves).is_empty());
    for p in [blade, lili, elves] {
        attach_new(&mut t, P1, "Bound in Gold", p);
    }
    // Equip (a keyword standing for an activated ability) and loyalty abilities can't be
    // activated; the Elves' mana ability still can.
    assert!(activatable(&mut t, P0, blade).is_empty());
    assert!(activatable(&mut t, P0, lili).is_empty());
    assert!(!activatable(&mut t, P0, elves).is_empty());
}

// ---------------------------------------------------------------------------------------
// Type-changing Auras
// ---------------------------------------------------------------------------------------

#[test]
fn enchantmentize_removes_the_lost_types_subtypes_but_keeps_supertypes() {
    cr!("205.1a", "613.1d");
    ruling!(
        "Enchantmentize",
        "The enchanted permanent loses any subtypes associated with the card types it lost. It keeps any supertypes, such as legendary."
    );
    ruling!(
        "One with the Stars",
        "The enchanted permanent loses any subtypes associated with the card types it lost. It keeps any supertypes, such as legendary."
    );
    // "Enchanted permanent is an enchantment and loses all other card types."
    supported("Enchantmentize");
    supported("One with the Stars");
    supported("Go-Shintai of Life's Origin");
    for aura in ["Enchantmentize", "One with the Stars"] {
        // Isamaru, Hound of Konda: a legendary Dog creature.
        let mut t = TestGame::new(2);
        let dog = t.battlefield(P0, "Isamaru, Hound of Konda");
        assert!(t.g.obj(dog).chars.has_subtype("Dog"));
        attach_new(&mut t, P1, aura, dog);
        t.settle();
        let o = t.obj_now(dog);
        assert!(o.is(CardType::Enchantment));
        assert!(!o.is(CardType::Creature));
        assert!(!o.chars.has_subtype("Dog"));
        assert!(o.chars.supertypes.contains(Supertype::Legendary));
        assert_eq!(o.chars.power, None);
        assert!(t.on_battlefield(dog));
        // Go-Shintai of Life's Origin: a legendary enchantment creature — Shrine. It keeps
        // its enchantment type, so it keeps the Shrine subtype.
        let shrine = t.battlefield(P0, "Go-Shintai of Life's Origin");
        attach_new(&mut t, P1, aura, shrine);
        let o = t.obj_now(shrine);
        assert!(o.is(CardType::Enchantment) && !o.is(CardType::Creature));
        assert!(o.chars.has_subtype("Shrine"));
        assert!(o.chars.supertypes.contains(Supertype::Legendary));
    }
}

#[test]
fn a_permanent_that_doesnt_untap_normally_can_be_untapped_by_effects() {
    cr!("502.3", "701.26b");
    ruling!(
        "Encrust",
        "The permanent can be untapped by other spells and abilities."
    );
    supported("Encrust");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    attach_new(&mut t, P0, "Encrust", bears);
    // It doesn't untap during its controller's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // Vitalize: "Untap all creatures you control."
    t.lands(P1, "Forest", 1);
    let vitalize = t.hand(P1, "Vitalize");
    t.cast(P1, vitalize).go();
    t.resolve();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn hold_for_questioning_doesnt_stop_untap_effects() {
    cr!("502.3", "701.26b");
    ruling!(
        "Hold for Questioning",
        "The permanent can be untapped by other spells and abilities."
    );
    supported("Hold for Questioning");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, "Hold for Questioning");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.lands(P1, "Forest", 1);
    let vitalize = t.hand(P1, "Vitalize");
    t.cast(P1, vitalize).go();
    t.resolve();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn pendrell_flux_the_payment_is_chosen_on_resolution() {
    cr!("603.2", "608.2", "118.12a", "701.21a");
    ruling!(
        "Pendrell Flux",
        "You choose whether to pay or not on resolution. If not, then you sacrifice the creature. You can choose to not pay if you no longer control the creature on resolution."
    );
    supported("Pendrell Flux");
    // The trigger is put on the stack at the beginning of the upkeep, when no mana is
    // available; the mana is only needed as it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Pendrell Flux", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(on_stack(&t, "sacrifice"), 1);
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(untapped_lands(&t, P0), 0);
    // Not paying: the creature is sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Pendrell Flux", bears);
    next_upkeep(&mut t, P0);
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(untapped_lands(&t, P0), 2);
    // No longer controlling the creature: its former controller doesn't pay, and can't
    // sacrifice a creature they don't control.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Pendrell Flux", bears);
    next_upkeep(&mut t, P0);
    assert_eq!(on_stack(&t, "sacrifice"), 1);
    give_control(&mut t, bears, P1);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn animated_artifact_keeps_pt_modifications_counters_and_switches() {
    cr!("613.4b", "613.4c", "613.4d", "613.1d");
    ruling!(
        "Mightstone's Animation",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Giant Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch power and toughness."
    );
    supported("Mightstone's Animation");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Millstone");
    // A +1/+1 counter put on it while it isn't a creature.
    run_with(
        &mut t,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: mtg_engine::ability::Value::Const(1),
        },
        &[Entity::Object(stone)],
    );
    t.battlefield(P0, "Glorious Anthem");
    attach_new(&mut t, P0, "Mightstone's Animation", stone);
    // Base 4/4, +1/+1 counter, +1/+1 from the Anthem.
    assert_eq!(t.pt(stone), (6, 6));
    // Giant Growth, cast once it's a creature.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(stone).go();
    t.resolve();
    assert_eq!(t.pt(stone), (9, 9));
    assert!(t.on_battlefield(stone));
    // Bonesplitter (+2/+0), then an effect switching its power and toughness.
    attach_new(&mut t, P0, "Bonesplitter", stone);
    assert_eq!(t.pt(stone), (11, 9));
    t.lands(P0, "Island", 2);
    let inside = t.hand(P0, "Inside Out");
    t.cast(P0, inside).target(stone).go();
    t.resolve();
    assert_eq!(t.pt(stone), (9, 11));
}

#[test]
fn zoetic_glyph_golem_keeps_pt_modifications() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Zoetic Glyph",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Giant Growth, will apply to the creature no matter when they started to take effect."
    );
    supported("Zoetic Glyph");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Millstone");
    run_with(
        &mut t,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: mtg_engine::ability::Value::Const(2),
        },
        &[Entity::Object(stone)],
    );
    attach_new(&mut t, P0, "Zoetic Glyph", stone);
    // Base 5/4 with two +1/+1 counters.
    assert_eq!(t.pt(stone), (7, 6));
    assert!(t.obj_now(stone).chars.has_subtype("Golem"));
    t.lands(P0, "Island", 2);
    let inside = t.hand(P0, "Inside Out");
    t.cast(P0, inside).target(stone).go();
    t.resolve();
    assert_eq!(t.pt(stone), (6, 7));
}

#[test]
fn an_animated_equipment_keeps_its_types_and_becomes_unattached() {
    cr!("301.5c", "704.5n", "205.1b");
    ruling!(
        "Mightstone's Animation",
        "The artifact retains any types, subtypes, or supertypes it has. Notably, an Equipment that becomes an artifact creature usually can't be attached to another creature. If it was attached to a creature, it becomes unattached."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    attach_new(&mut t, P0, "Mightstone's Animation", blade);
    t.settle();
    let o = t.obj_now(blade);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Equipment"));
    assert_eq!(o.attached_to, None);
    assert_eq!(t.pt(blade), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
    // Its equip ability can't attach it to a creature any more.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let _ = activate_named(&mut t, P0, blade, "Equip", 0);
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn eaten_by_piranhas_keeps_pt_modifications() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Eaten by Piranhas",
        "Effects that modify the creature's power and/or toughness, such as the effect of Giant Growth, will apply to the creature no matter when they started to take effect. The same is true for counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Eaten by Piranhas");
    // Hill Giant (3/3) equipped with Bonesplitter (+2/+0), with a +1/+1 counter, and
    // Giant Growth'd: 9/7.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P1, "Bonesplitter", giant);
    run_with(
        &mut t,
        P1,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: mtg_engine::ability::Value::Const(1),
        },
        &[Entity::Object(giant)],
    );
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (9, 7));
    attach_new(&mut t, P0, "Eaten by Piranhas", giant);
    // Base 1/1, +2/+0, +1/+1 counter, Giant Growth's +3/+3 (all started earlier).
    assert_eq!(t.pt(giant), (7, 5));
    assert!(t.obj_now(giant).chars.has_subtype("Skeleton"));
    assert!(!t.obj_now(giant).chars.has_subtype("Giant"));
    // An effect switching its power and toughness applies too.
    t.lands(P1, "Island", 2);
    let inside = t.hand(P1, "Inside Out");
    t.cast(P1, inside).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (5, 7));
}

// ---------------------------------------------------------------------------------------
// Auras on lands
// ---------------------------------------------------------------------------------------

#[test]
fn goblin_caves_works_on_an_opponents_mountain() {
    cr!("303.4e", "613.4c");
    ruling!(
        "Goblin Caves",
        "Works even if placed on one of your opponent's Mountains."
    );
    supported("Goblin Caves");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Raging Goblin");
    let theirs = t.battlefield(P1, "Goblin Piker");
    let mountain = t.battlefield(P1, "Mountain");
    attach_new(&mut t, P0, "Goblin Caves", mountain);
    assert_eq!(t.pt(mine), (1, 3));
    assert_eq!(t.pt(theirs), (2, 3));
}

#[test]
fn goblin_shrine_works_on_an_opponents_mountain() {
    cr!("303.4e", "613.4c");
    ruling!(
        "Goblin Shrine",
        "Works even if placed on one of your opponent's Mountains."
    );
    supported("Goblin Shrine");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Raging Goblin");
    let mountain = t.battlefield(P1, "Mountain");
    attach_new(&mut t, P0, "Goblin Shrine", mountain);
    assert_eq!(t.pt(mine), (2, 1));
}

#[test]
fn a_land_granted_a_damage_ability_deals_colorless_damage() {
    cr!("105.2c", "702.16e", "113.3b");
    ruling!(
        "Barbed Field",
        "Land are normally colorless, so effects that care about the color of the damage’s source will not find a color."
    );
    supported("Barbed Field");
    // Kor Firewalker has protection from red; Barbed Field is red, but the damage source
    // is the colorless land.
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Kor Firewalker");
    let land = t.battlefield(P0, "Wastes");
    attach_new(&mut t, P0, "Barbed Field", land);
    t.answer_targets(P0, &[Entity::Object(walker)]);
    activate_containing(&mut t, P0, land, "damage")
        .expect("the land's granted ability can target the pro-red creature");
    t.resolve();
    assert_eq!(t.obj_now(walker).damage, 1);
}

#[test]
fn noxious_field_damage_comes_from_the_colorless_land() {
    cr!("105.2c", "702.16e");
    ruling!(
        "Noxious Field",
        "Land are normally colorless, so effects that care about the color of the damage’s source will not find a color."
    );
    supported("Noxious Field");
    // Noxious Field is black; a creature with protection from black is still dealt damage
    // by the land.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let land = t.battlefield(P0, "Wastes");
    attach_new(&mut t, P0, "Noxious Field", land);
    activate_containing(&mut t, P0, land, "damage").expect("activate");
    t.resolve();
    assert_eq!(t.obj_now(knight).damage, 1);
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------------------
// Attacks each combat if able
// ---------------------------------------------------------------------------------------

#[test]
fn a_creature_that_must_attack_doesnt_if_it_cant_or_if_attacking_costs_something() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Furor of the Bitten",
        "If the enchanted creature can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Furor of the Bitten");
    // Able to attack: declaring no attackers isn't legal; it attacks.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Furor of the Bitten", bears);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 16);
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Furor of the Bitten", bears);
    t.g.tap(bears);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 20);
    // Came under its controller's control this turn: it doesn't attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Furor of the Bitten", bears);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 20);
    // Attacking costs {2} (Ghostly Prison): it isn't forced to attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Furor of the Bitten", bears);
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn impending_doom_doesnt_force_a_tapped_creature_to_attack() {
    cr!("508.1d");
    ruling!(
        "Impending Doom",
        "If the enchanted creature can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack."
    );
    supported("Impending Doom");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Impending Doom", bears);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 15);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Impending Doom", bears);
    t.g.tap(bears);
    t.attack(&[], &[]);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Infect
// ---------------------------------------------------------------------------------------

#[test]
fn multiple_instances_of_infect_are_redundant() {
    cr!("702.90a", "702.90c", "702.90f");
    ruling!(
        "Phyresis",
        "Multiple instances of infect on the same creature are redundant."
    );
    supported("Phyresis");
    supported("Corrupted Conscience");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Phyresis", giant);
    attach_new(&mut t, P0, "Phyresis", giant);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.g.player(P1).poison(), 3);
    assert_eq!(t.life(P1), 20);
    // Blocked by a 4/4: three -1/-1 counters, not six.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Phyresis", giant);
    attach_new(&mut t, P0, "Corrupted Conscience", giant);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    run_with(
        &mut t,
        P1,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: mtg_engine::ability::Value::Const(2),
        },
        &[Entity::Object(blocker)],
    );
    t.attack(&[(giant, Entity::Player(P1))], &[(blocker, giant)]);
    // Three -1/-1 counters, two of which annihilate with its +1/+1 counters.
    assert_eq!(t.counters(blocker, "-1/-1"), 1);
    assert_eq!(t.counters(blocker, "+1/+1"), 0);
    assert!(t.on_battlefield(blocker));
    assert_eq!(t.pt(blocker), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Control-changing Auras
// ---------------------------------------------------------------------------------------

#[test]
fn gaining_control_of_a_permanent_doesnt_give_control_of_what_is_attached_to_it() {
    cr!("303.4e", "301.5d", "613.1b");
    ruling!(
        "Confiscate",
        "Gaining control of a permanent doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    supported("Confiscate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = attach_new(&mut t, P1, "Bonesplitter", bears);
    let rancor = attach_new(&mut t, P1, "Rancor", bears);
    let conf = in_hand_with_mana(&mut t, P0, "Confiscate");
    t.cast(P0, conf).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(blade).controller, P1);
    assert_eq!(t.obj_now(rancor).controller, P1);
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(bears)));
    // P0 can't move the Equipment; P1 still can.
    assert!(activatable(&mut t, P0, blade).is_empty());
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    activate_named(&mut t, P1, blade, "Equip", 0).expect("the Equipment's controller equips");
    t.resolve();
    assert_eq!(attached_to(&t, blade), Some(Entity::Object(giant)));
}

// ---------------------------------------------------------------------------------------
// Optional counters on Auras
// ---------------------------------------------------------------------------------------

#[test]
fn putting_the_counter_on_momentum_is_optional() {
    cr!("603.5", "122.1");
    ruling!("Momentum", "Putting on a counter is optional.");
    supported("Momentum");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = attach_new(&mut t, P0, "Momentum", bears);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.counters(m, "growth"), 0);
    assert_eq!(t.pt(bears), (2, 2));
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(m, "growth"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}
