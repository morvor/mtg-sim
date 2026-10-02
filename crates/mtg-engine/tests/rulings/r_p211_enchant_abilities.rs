//! Rulings batch P211 — enchant (CR 303.4, 702.5): what an Aura may enchant, control of
//! an Aura vs. the enchanted permanent, abilities granted to the enchanted creature, and
//! triggered abilities of Auras.

use crate::r_p208_common::put_in_graveyard;
use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts the real Aura `name` (with lands for its cost) targeting `target`, and it
/// resolves. The targets `p` was offered.
fn aura_targets(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) -> Vec<Entity> {
    let aura = in_hand_with_mana(t, p, name);
    let from = t.asked().len();
    t.cast(p, aura).target(target).go();
    t.resolve_all();
    target_candidates(t, p, from).concat()
}

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

// ---------------------------------------------------------------------------------------
// What the Aura may enchant
// ---------------------------------------------------------------------------------------

#[test]
fn threads_of_disloyalty_cant_enchant_a_creature_with_mana_value_3_or_greater() {
    cr!("303.4a", "303.4c", "704.5m", "115.4");
    ruling!(
        "Threads of Disloyalty",
        "Threads of Disloyalty can't target a creature with mana value 3 or greater. If it somehow enchants a creature with mana value 3 or greater, Threads of Disloyalty is put into the graveyard the next time state-based actions are checked."
    );
    supported("Threads of Disloyalty");
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let targets = aura_targets(&mut t, P0, "Threads of Disloyalty", bears);
    assert!(targets.contains(&Entity::Object(bears)));
    assert!(!targets.contains(&Entity::Object(giant)));
    // It enchants the Bears; Cytoshape makes the Bears a copy of Hill Giant (mana value
    // 3): Threads of Disloyalty is put into the graveyard and P1 gets the Bears back.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Threads of Disloyalty", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn."
    let shape = in_hand_with_mana(&mut t, P0, "Cytoshape");
    t.cast_with(P0, shape, &[Entity::Object(bears)]).unwrap();
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert!(t.in_graveyard(P0, "Threads of Disloyalty"));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn wurmweaver_coil_can_enchant_only_a_green_creature() {
    cr!("303.4a", "303.4c", "303.4d", "704.5m", "701.3b");
    ruling!(
        "Wurmweaver Coil",
        "Wurmweaver Coil can enchant only a green creature. It can't enter attached to a nongreen creature, it can't be moved onto a nongreen creature, and if the creature it's attached to stops being green, the Aura is put into its owner's graveyard."
    );
    supported("Wurmweaver Coil");
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let targets = aura_targets(&mut t, P0, "Wurmweaver Coil", bears);
    assert!(targets.contains(&Entity::Object(bears)));
    assert!(!targets.contains(&Entity::Object(giant)));
    // Can't be moved onto the (red) Hill Giant.
    let coil = t.g.current(t.named_on_battlefield("Wurmweaver Coil")[0]);
    assert!(!t.g.attach(coil, Entity::Object(giant)));
    assert_eq!(attached_to(&t, coil), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (8, 8));
    // Turn to Frog makes the Bears a blue Frog: the Coil is put into the graveyard.
    let frog = in_hand_with_mana(&mut t, P1, "Turn to Frog");
    t.cast_with(P1, frog, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wurmweaver Coil"));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn volition_reins_may_enchant_an_untapped_permanent() {
    cr!("303.4a", "603.4", "702.5a");
    ruling!(
        "Volition Reins",
        "Volition Reins may target and may enchant an untapped permanent."
    );
    supported("Volition Reins");
    // "When this Aura enters, if enchanted permanent is tapped, untap it. You control
    // enchanted permanent."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = in_hand_with_mana(&mut t, P0, "Volition Reins");
    t.cast(P0, aura).target(giant).go();
    t.resolve();
    // The untap ability didn't trigger (its condition is false).
    assert_eq!(t.stack_len(), 0);
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(giant)));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn hold_for_questioning_on_a_tapped_permanent_still_investigates() {
    cr!("603.2", "701.26a", "701.16a");
    ruling!(
        "Hold for Questioning",
        "You can enchant a permanent that is already tapped with Hold for Questioning. If you do, it doesn't become tapped again, but you will still investigate."
    );
    supported("Hold for Questioning");
    // "When this Aura enters, tap enchanted permanent and investigate." Grizzly Bears is
    // already tapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let aura = in_hand_with_mana(&mut t, P0, "Hold for Questioning");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
}

// ---------------------------------------------------------------------------------------
// Control
// ---------------------------------------------------------------------------------------

#[test]
fn steal_enchantment_changes_only_the_auras_controller() {
    cr!("303.4e", "108.4", "613.1b");
    ruling!(
        "Steal Enchantment",
        "When a player takes control of an enchantment, they do not get to change anything about the enchantment (such as what creature it is on, what choices it has or anything) at that time. They just become its controller."
    );
    supported("Steal Enchantment");
    supported("Holy Strength");
    // P1's Holy Strength enchants P1's Grizzly Bears. P0 steals the Aura: it stays on the
    // Bears, which still gets +1/+2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = attach_new(&mut t, P1, "Holy Strength", bears);
    let steal = in_hand_with_mana(&mut t, P0, "Steal Enchantment");
    t.cast(P0, steal).target(strength).go();
    t.resolve_all();
    assert_eq!(t.obj_now(strength).controller, P0);
    assert_eq!(attached_to(&t, strength), Some(Entity::Object(bears)));
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn only_the_auras_controller_can_activate_its_ability() {
    cr!("602.2", "303.4e", "108.4");
    ruling!(
        "Ghostly Wings",
        "You control Ghostly Wings even while it enchants an opponent’s creature. Only you can activate the last ability of Ghostly Wings."
    );
    supported("Ghostly Wings");
    // "Discard a card: Return enchanted creature to its owner's hand."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wings = attach_new(&mut t, P0, "Ghostly Wings", bears);
    t.hand(P0, "Hill Giant");
    t.hand(P1, "Hill Giant");
    assert_eq!(t.obj_now(wings).controller, P0);
    assert!(activatable(&mut t, P1, wings).is_empty());
    assert_eq!(activatable(&mut t, P0, wings).len(), 1);
    assert!(t.activate(P1, wings, 0, &[]).is_err());
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn frenzied_fugue_steals_after_the_upkeep_began() {
    cr!("503.1a", "603.2", "603.3b");
    ruling!(
        "Frenzied Fugue",
        "You won't control the enchanted permanent as your upkeep begins. Any \"at the beginning of your upkeep\" abilities it has won't trigger during your upkeep (unless you already controlled the permanent)."
    );
    supported("Frenzied Fugue");
    supported("Phyrexian Arena");
    // P0's Frenzied Fugue enchants P1's Phyrexian Arena ("At the beginning of your upkeep,
    // you draw a card and you lose 1 life."). At P0's upkeep P0 gains control of it, too
    // late for its ability to trigger.
    let mut t = TestGame::new(2);
    let arena = t.battlefield(P1, "Phyrexian Arena");
    attach_new(&mut t, P0, "Frenzied Fugue", arena);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(on_stack(&t, "gain control"), 1);
    assert_eq!(on_stack(&t, "draw a card"), 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.obj_now(arena).controller, P0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------------
// Abilities granted to the enchanted creature
// ---------------------------------------------------------------------------------------

#[test]
fn trollhides_regeneration_shield_must_exist_before_the_destruction() {
    cr!("701.19a", "701.19b", "614.8");
    ruling!(
        "Trollhide",
        "To work, the regeneration shield must be created before the enchanted creature is destroyed. This usually means activating its ability during the declare blockers step, or in response to a spell or ability that would destroy it."
    );
    supported("Trollhide");
    // Activated in response to Murder: it survives, tapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Trollhide", bears);
    let murder = in_hand_with_mana(&mut t, P1, "Murder");
    t.cast(P1, murder).target(bears).go();
    t.lands(P0, "Forest", 2);
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    // Without a shield, it's destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Trollhide", bears);
    let murder = in_hand_with_mana(&mut t, P1, "Murder");
    t.cast(P1, murder).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // In combat: activated during the declare blockers step, before damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Trollhide", bears);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(wurm, Entity::Player(P0))], &[(bears, wurm)]);
    t.lands(P0, "Forest", 2);
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.on_battlefield(bears));
}

#[test]
fn treefolk_umbra_changes_only_combat_damage_assignment() {
    cr!("510.1a", "510.1", "701.14a");
    ruling!(
        "Treefolk Umbra",
        "Treefolk Umbra's effect doesn't actually change the enchanted creature's power. It changes only the amount of combat damage the creature assigns. All other rules and effects that check power or toughness use the real values. For example, Savage Swipe won't cause the enchanted creature to fight with its toughness."
    );
    supported("Treefolk Umbra");
    supported("Savage Swipe");
    // Grizzly Bears with Treefolk Umbra: 2/4, assigns 4 combat damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Treefolk Umbra", bears);
    assert_eq!(t.pt(bears), (2, 4));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    // Savage Swipe ("Target creature you control gets +2/+2 until end of turn if its power
    // is 2. Then it fights up to one target creature you don't control."): its power is 2,
    // so it gets +2/+2 and fights Imperial Ceratops (3/5) with its power (4), not its
    // toughness (6).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Treefolk Umbra", bears);
    let ceratops = t.battlefield(P1, "Imperial Ceratops");
    let swipe = in_hand_with_mana(&mut t, P0, "Savage Swipe");
    t.cast_with(
        P0,
        swipe,
        &[Entity::Object(bears), Entity::Object(ceratops)],
    )
    .unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 6));
    assert_eq!(t.obj_now(bears).damage, 3);
    assert!(t.on_battlefield(ceratops));
    assert_eq!(t.obj_now(ceratops).damage, 4);
}

#[test]
fn spirit_links_life_gain_is_a_trigger_that_comes_too_late() {
    cr!("603.2", "704.5a", "702.15a", "800.4a");
    ruling!(
        "Spirit Link",
        "Unlike the lifelink ability, the ability of Spirit Link is a triggered ability that goes on the stack and may be responded to. Notably, if the enchanted creature deals damage at the same time you’re dealt enough damage to reduce your life total to 0 or less, you’ll lose the game before you can gain any life."
    );
    supported("Spirit Link");
    // The trigger uses the stack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Spirit Link", bears);
    let target = t.battlefield(P1, "Hill Giant");
    damage(&mut t, bears, 2, target);
    assert_eq!(t.life(P0), 20);
    assert_eq!(on_stack(&t, "gain that much life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // P0 at 2 life blocks one of two attacking Bears with the Spirit Linked creature: P0
    // loses before gaining life.
    let mut t = TestGame::new(3);
    t.g.players[0].life = 2;
    let mine = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Spirit Link", mine);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[(mine, a)],
    );
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
}

#[test]
fn lunarch_mantles_ability_can_sacrifice_the_aura_itself() {
    cr!("602.2", "118.3", "611.2c");
    ruling!(
        "Lunarch Mantle",
        "You can activate the ability Lunarch Mantle grants to your enchanted creature by sacrificing any permanent you control, including Lunarch Mantle or the enchanted creature itself. If you sacrifice Lunarch Mantle this way, the creature it previously enchanted still gains flying."
    );
    supported("Lunarch Mantle");
    // "{1}, Sacrifice a permanent: This creature gains flying until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mantle = attach_new(&mut t, P0, "Lunarch Mantle", bears);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(mantle)]);
    t.activate(P0, bears, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Lunarch Mantle"));
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    // Sacrificing the creature itself is allowed too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Lunarch Mantle", bears);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, bears, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
}

#[test]
fn bear_umbra_untaps_the_lands_of_the_creatures_controller() {
    cr!("109.5", "113.10", "508.1m");
    ruling!(
        "Bear Umbra",
        "When the enchanted creature attacks, all lands controlled by that creature's controller (who is not necessarily the Aura's controller) untap."
    );
    supported("Bear Umbra");
    // P0's Bear Umbra on P1's Grizzly Bears. P1 attacks: P1's lands untap, P0's don't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Bear Umbra", bears);
    let mine = t.lands(P0, "Forest", 2);
    let theirs = t.lands(P1, "Forest", 2);
    for l in mine.iter().chain(theirs.iter()) {
        t.g.tap(*l);
    }
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert!(theirs.iter().all(|l| !t.obj_now(*l).tapped));
    assert!(mine.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn predatory_urges_creature_can_fight_itself() {
    cr!("120.3", "608.2c", "115.3");
    ruling!(
        "Predatory Urge",
        "You may have the enchanted creature target itself with its own ability. If you do, it will deal damage to itself equal to its power, then immediately do it again."
    );
    supported("Predatory Urge");
    // Enchanted creature has "{T}: This creature deals damage equal to its power to target
    // creature. That creature deals damage equal to its power to this creature." Horned
    // Turtle (1/4) targets itself: 1 damage, then 1 more.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Horned Turtle");
    attach_new(&mut t, P0, "Predatory Urge", turtle);
    t.activate(P0, turtle, 0, &[Entity::Object(turtle)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(turtle).damage, 2);
    // Grizzly Bears (2/2) targeting itself dies (2 + 2 damage).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Predatory Urge", bears);
    t.activate(P0, bears, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn helm_of_the_ghastlord_checks_the_color_only_as_damage_is_dealt() {
    cr!("603.2", "603.4", "611.2");
    ruling!(
        "Helm of the Ghastlord",
        "With regards to the two triggered abilities that Helm of the Ghastlord grants to the enchanted creature, the point when damage is actually dealt is the only time it matters what color the creature is. If the creature is blue or black at that time, the appropriate ability or abilities will trigger. They’ll resolve even if the creature changes color or loses its Aura."
    );
    supported("Helm of the Ghastlord");
    supported("Coral Merfolk");
    // Blue Coral Merfolk deals combat damage to P1: "draw a card" triggers; the Helm is
    // destroyed before it resolves; P0 still draws.
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let helm = attach_new(&mut t, P0, "Helm of the Ghastlord", merfolk);
    attack_with(&mut t, &[(merfolk, Entity::Player(P1))]);
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    assert_eq!(on_stack(&t, "draw a card"), 1);
    destroy(&mut t, helm);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A red creature: no trigger.
    let mut t = TestGame::new(2);
    let piker = t.battlefield(P0, "Goblin Piker");
    attach_new(&mut t, P0, "Helm of the Ghastlord", piker);
    let hand = t.hand_size(P0);
    t.attack(&[(piker, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn withercrowns_base_power_overwrites_earlier_setting_effects_only() {
    cr!("613.4b", "613.7a", "613.7e");
    ruling!(
        "Withercrown",
        "Withercrown overwrites all previous effects that set the enchanted creature’s power to specific a value. Other effects that set its base power to specific values that start to apply after Withercrown becomes attached to the creature will overwrite this effect."
    );
    supported("Withercrown");
    supported("Turn to Frog");
    // Turn to Frog (base 1/1) first, then Withercrown: 0/1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let frog = in_hand_with_mana(&mut t, P0, "Turn to Frog");
    t.cast_with(P0, frog, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    attach_new(&mut t, P0, "Withercrown", giant);
    assert_eq!(t.pt(giant), (0, 1));
    // Withercrown first, then Turn to Frog: 1/1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Withercrown", giant);
    assert_eq!(t.pt(giant), (0, 3));
    let frog = in_hand_with_mana(&mut t, P0, "Turn to Frog");
    t.cast_with(P0, frog, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn jeskai_runemark_losing_flying_doesnt_undo_a_block() {
    cr!("509.1b", "509.1h", "506.4");
    ruling!(
        "Jeskai Runemark",
        "Whether a creature has flying is checked as blocking creatures are declared. Once the enchanted creature blocks or is blocked, losing flying won’t change or undo that block."
    );
    supported("Jeskai Runemark");
    // "Enchanted creature gets +2/+2. Enchanted creature has flying as long as you control
    // a red or white permanent." P1's Bears (4/4 flying with Goblin Piker) blocks Wind
    // Drake; then the Goblin leaves: the block stands.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Jeskai Runemark", bears);
    let piker = t.battlefield(P1, "Goblin Piker");
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    let drake = t.battlefield(P0, "Wind Drake");
    to_blockers(&mut t, &[(drake, Entity::Player(P1))], &[(bears, drake)]);
    destroy(&mut t, piker);
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Wind Drake"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn briar_shield_sacrificed_gives_no_bonus_until_its_ability_resolves() {
    cr!("602.2", "118.3", "704.5g", "611.2c");
    ruling!(
        "Briar Shield",
        "When you cast Briar Shield's activated ability, Briar Shield is put into the graveyard as a cost. The +1/+1 effect immediately ends, but the +3/+3 effect won't begin until the ability resolves. The enchanted creature gets no bonus while Briar Shield's ability is on the stack. If its toughness is 0 during this time, it'll be put into the graveyard."
    );
    supported("Briar Shield");
    // "Sacrifice this Aura: Enchanted creature gets +3/+3 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shield = attach_new(&mut t, P0, "Briar Shield", bears);
    assert_eq!(t.pt(bears), (3, 3));
    t.activate(P0, shield, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Briar Shield"));
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // A 1/1 with a -1/-1 counter survives only thanks to the Shield: it dies while the
    // ability is on the stack.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.add_counters(Entity::Object(elves), counters::MINUS1, 1, None);
    let shield = attach_new(&mut t, P0, "Briar Shield", elves);
    t.settle();
    assert_eq!(t.pt(elves), (1, 1));
    t.activate(P0, shield, 0, &[]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

// ---------------------------------------------------------------------------------------
// Triggered abilities
// ---------------------------------------------------------------------------------------

#[test]
fn demonic_appetite_may_sacrifice_the_enchanted_creature() {
    cr!("603.2", "701.21a");
    ruling!(
        "Demonic Appetite",
        "When Demonic Appetite’s triggered ability resolves, you may sacrifice the creature enchanted by Demonic Appetite. If you control no other creatures, you’ll have to sacrifice that one."
    );
    supported("Demonic Appetite");
    // Only the enchanted creature: it's sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Demonic Appetite", bears);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // With another creature, P0 may choose the enchanted one.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Demonic Appetite", bears);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Demonic Appetite"));
}

#[test]
fn luminous_wake_triggers_once_for_blocking_two_attackers() {
    cr!("509.1a", "509.3c", "603.2c");
    ruling!(
        "Luminous Wake",
        "When the enchanted creature blocks, Luminous Wake’s ability triggers just once, even if that creature somehow blocked multiple attacking creatures."
    );
    supported("Luminous Wake");
    supported("Echo Circlet");
    // P1's Hill Giant, equipped with Echo Circlet, blocks two attacking Bears.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P1, "Luminous Wake", giant);
    attach_new(&mut t, P1, "Echo Circlet", giant);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(giant, a), (giant, b)],
    );
    assert_eq!(triggers_on_stack(&t, "gain 4 life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 24);
}

#[test]
fn mortal_obstinacy_must_be_on_the_battlefield_to_be_sacrificed() {
    cr!("603.2", "608.2c", "701.21a");
    ruling!(
        "Mortal Obstinacy",
        "You decide whether to sacrifice Mortal Obstinacy as its triggered ability resolves. If it's not on the battlefield at that time, you can't sacrifice it and destroy the target enchantment."
    );
    supported("Mortal Obstinacy");
    // "Whenever enchanted creature deals combat damage to a player, you may sacrifice this
    // Aura. If you do, destroy target enchantment."
    let setup = || {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ob = attach_new(&mut t, P0, "Mortal Obstinacy", bears);
        let anthem = t.battlefield(P1, "Glorious Anthem");
        t.answer_targets(P0, &[Entity::Object(anthem)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.advance_to_step(Step::CombatDamage);
        t.settle();
        assert_eq!(on_stack(&t, "you may sacrifice"), 1);
        (t, ob, anthem)
    };
    // It leaves before resolution: nothing is destroyed even if P0 says yes.
    let (mut t, ob, anthem) = setup();
    put_in_graveyard(&mut t, ob);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(anthem));
    // Still there: P0 decides on resolution and sacrifices it.
    let (mut t, ob, anthem) = setup();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(ob));
    assert!(!t.on_battlefield(anthem));
}

#[test]
fn keen_sense_and_curiosity_draw_one_card_per_damage_event() {
    cr!("603.2", "120.3");
    ruling!(
        "Keen Sense",
        "You draw one card each time the enchanted creature deals damage to an opponent, no matter how much damage it deals or whether it was dealt in combat."
    );
    ruling!(
        "Curiosity",
        "You draw one card each time the enchanted creature deals damage to an opponent, no matter how much damage it deals."
    );
    supported("Keen Sense");
    supported("Curiosity");
    for aura in ["Keen Sense", "Curiosity"] {
        // Combat damage: 3 damage, one card.
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        attach_new(&mut t, P0, aura, giant);
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.attack(&[(giant, Entity::Player(P1))], &[]);
        assert_eq!(t.life(P1), 17, "{aura}");
        assert_eq!(t.hand_size(P0), hand + 1, "{aura}");
        // Noncombat damage: one card.
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        attach_new(&mut t, P0, aura, giant);
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        let mut ctx = mtg_engine::eval::Ctx::new(Some(giant), P0);
        ctx.targets = vec![vec![Entity::Player(P1)]];
        t.g.exec(
            &mtg_engine::ability::Effect::DealDamage {
                source: mtg_engine::ability::Sel::This,
                amount: mtg_engine::ability::Value::Const(5),
                to: mtg_engine::ability::Sel::Target(0),
            },
            &mut ctx,
        );
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.life(P1), 15, "{aura}");
        assert_eq!(t.hand_size(P0), hand + 1, "{aura}");
    }
}

#[test]
fn the_attacking_token_may_attack_a_different_player() {
    cr!("508.4", "506.2", "802.2");
    ruling!(
        "Invocation of Saint Traft",
        "You declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn’t have to be the same player or planeswalker the enchanted creature is attacking."
    );
    ruling!(
        "Dorothea, Vengeful Victim // Dorothea's Retribution",
        "You declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker the enchanted creature is attacking."
    );
    supported("Invocation of Saint Traft");
    supported("Dorothea, Vengeful Victim // Dorothea's Retribution");
    // Enchanted creature has "Whenever this creature attacks, create a 4/4 white Angel
    // (Spirit) creature token with flying that's tapped and attacking." A three-player
    // game: the Bears attacks P1, the token attacks P2.
    for (name, sub) in [
        ("Invocation of Saint Traft", "Angel"),
        ("Dorothea, Vengeful Victim // Dorothea's Retribution", "Spirit"),
    ] {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if sub == "Angel" {
            attach_new(&mut t, P0, name, bears);
        } else {
            // Dorothea's Retribution: cast from the graveyard with disturb {1}{W}{U}.
            let card = t.graveyard(P0, name);
            t.lands(P0, "Plains", 1);
            t.lands(P0, "Island", 1);
            t.lands(P0, "Wastes", 1);
            t.cast(P0, card)
                .method(CastMethod::Keyword(KeywordKind::Disturb))
                .target(bears)
                .go();
            t.resolve_all();
            let aura = t.g.current(card);
            assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
        }
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.resolve_all();
        let tok = with_subtype(&t, P0, sub);
        assert_eq!(tok.len(), 1, "{name}");
        let combat = t.g.combat.as_ref().unwrap();
        assert_eq!(combat.attack_target(tok[0]), Some(Entity::Player(P2)), "{name}");
        assert_eq!(combat.attack_target(bears), Some(Entity::Player(P1)), "{name}");
    }
}
