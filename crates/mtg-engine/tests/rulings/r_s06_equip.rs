//! Rulings batch S06 — equip (CR 702.6) and Equipment: "When this Equipment enters,
//! attach it to target creature you control", living weapon Germs (CR 702.92), kindred
//! Equipment that attach themselves to creatures of a type, manifest dread Equipment, and
//! protection from colors.

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Germ tokens `p` controls.
fn germs(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.controller == p && o.chars.has_subtype("Germ"))
        .map(|o| o.id)
        .collect()
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
// "When this Equipment enters, attach it to target creature you control."
// ---------------------------------------------------------------------------------------

#[test]
fn an_equipment_whose_attach_target_is_illegal_stays_unattached() {
    cr!("608.2b", "701.3a");
    ruling!(
        "Utility Knife",
        "If the target creature becomes an illegal target, the Equipment remains on the battlefield unattached."
    );
    supported("Utility Knife");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let knife = t.enter(P0, "Utility Knife");
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "attach it"), 1);
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(knife));
    assert_eq!(attached_to(&t, knife), None);
}

#[test]
fn the_enters_trigger_attaches_for_free_at_any_time() {
    cr!("603.2", "701.3a", "702.6a");
    ruling!(
        "Utility Knife",
        "Attaching an Equipment with its enters-the-battlefield triggered ability isn't the same as using its equip ability. You don't pay mana for the attachment, and if the Equipment enters at a time you couldn't cast a sorcery, you can still attach it to a creature you control."
    );
    // The Equipment enters during the opponent's combat; its controller has no mana.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let knife = t.enter(P0, "Utility Knife");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(attached_to(&t, knife), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    // Its equip ability couldn't have done that now (it's an opponent's turn), even with
    // the mana and another creature to move it to.
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 3);
    assert!(activatable(&mut t, P0, knife).is_empty());
}

#[test]
fn a_flash_equipment_attaches_without_equip_timing_or_cost() {
    cr!("603.2", "701.3a", "702.6a", "702.8a");
    ruling!(
        "Galadhrim Bow",
        "Attaching an Equipment with its enters-the-battlefield triggered ability isn't the same as using its equip ability. You don't pay mana for the attachment, and the timing restrictions for equip abilities don't apply."
    );
    supported("Galadhrim Bow");
    // Galadhrim Bow ({2}{G}, flash): "When this Equipment enters, attach it to target
    // creature you control. Untap that creature." Cast during the opponent's turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let bow = in_hand_with_mana(&mut t, P0, "Galadhrim Bow");
    t.set_step(P1, Step::BeginningOfCombat);
    t.cast(P0, bow).go();
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    t.resolve_all();
    let bow = t.named_on_battlefield("Galadhrim Bow")[0];
    assert_eq!(attached_to(&t, bow), Some(Entity::Object(bears)));
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.pt(bears), (3, 4));
    // Only the spell's mana was spent: three lands.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(tapped_lands(&t, P0), 3);
}

// ---------------------------------------------------------------------------------------
// Living weapon (Phyrexian Germs)
// ---------------------------------------------------------------------------------------

#[test]
fn the_phyrexian_germ_enters_as_a_0_0_creature() {
    cr!("702.92a", "603.6a", "603.10");
    ruling!(
        "Nettlecyst",
        "The Phyrexian Germ token enters the battlefield as a 0/0 creature and the Equipment becomes attached to it before state-based actions would cause the token to die. Abilities that trigger as the token enters the battlefield see that a 0/0 creature entered the battlefield."
    );
    supported("Nettlecyst");
    supported("Garruk's Packleader");
    // Nettlecyst: "Equipped creature gets +1/+1 for each artifact and/or enchantment you
    // control." With two more artifacts, the equipped Germ is 3/3; but it entered as a 0/0,
    // so Garruk's Packleader ("Whenever another creature you control with power 3 or
    // greater enters, you may draw a card") doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Packleader");
    t.battlefield(P0, "Millstone");
    t.battlefield(P0, "Sol Ring");
    let cyst = t.enter(P0, "Nettlecyst");
    t.g.flush_events();
    t.settle();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    let germ = germs(&t, P0);
    assert_eq!(germ.len(), 1);
    assert_eq!(attached_to(&t, cyst), Some(Entity::Object(germ[0])));
    assert_eq!(t.pt(germ[0]), (3, 3));
    assert_eq!(t.hand_size(P0), hand);
    // A 3/3 creature entering does trigger it.
    t.answer_yes(P0, true);
    t.enter(P0, "Hill Giant");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn with_two_phyrexian_germs_the_equipment_is_attached_to_one() {
    cr!("702.92a", "614.1a", "704.5f");
    ruling!(
        "Nettlecyst",
        "If the living weapon trigger causes two Phyrexian Germs to be created (due to an effect such as that of Doubling Season), the Equipment becomes attached to one of them. The other will be put into your graveyard and subsequently cease to exist, unless another effect raises its toughness above 0."
    );
    // Parallel Lives: "If an effect would create one or more tokens under your control, it
    // creates twice that many of those tokens instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    let cyst = t.enter(P0, "Nettlecyst");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let created = t
        .g
        .objects
        .iter()
        .filter(|o| o.is_token() && o.prev.is_none() && o.chars.has_subtype("Germ"))
        .count();
    assert_eq!(created, 2);
    let germ = germs(&t, P0);
    assert_eq!(germ.len(), 1);
    assert_eq!(attached_to(&t, cyst), Some(Entity::Object(germ[0])));
    // The other one died: no token card remains in the graveyard.
    assert!(!t.g.player(P0).graveyard.iter().any(|c| t.g.obj(*c).is_token()));
    // With something raising the toughness of every creature above 0, both would live:
    // Glorious Anthem.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Nettlecyst");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(germs(&t, P0).len(), 2);
}

#[test]
fn moving_the_equipment_off_the_phyrexian_germ_kills_it() {
    cr!("702.92a", "702.6a", "704.5f");
    ruling!(
        "Nettlecyst",
        "Like other Equipment, each Equipment with living weapon has an equip cost. You can pay this cost to attach an Equipment to another creature you control. Once the Phyrexian Germ token is no longer equipped, it will be put into your graveyard and subsequently cease to exist, unless another effect raises its toughness above 0."
    );
    let mut t = TestGame::new(2);
    let cyst = t.enter(P0, "Nettlecyst");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let germ = germs(&t, P0)[0];
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, cyst, "Equip", 0).expect("equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, cyst), Some(Entity::Object(bears)));
    assert!(!t.g.is_live(germ));
    assert!(germs(&t, P0).is_empty());
    // Nettlecyst is the only artifact: +1/+1.
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn the_equipment_stays_when_the_phyrexian_germ_is_destroyed() {
    cr!("702.92a", "704.5n");
    ruling!(
        "Nettlecyst",
        "If the Phyrexian Germ token is destroyed, the Equipment remains on the battlefield as with any other Equipment."
    );
    let mut t = TestGame::new(2);
    let cyst = t.enter(P0, "Nettlecyst");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let germ = germs(&t, P0)[0];
    destroy(&mut t, germ);
    assert!(germs(&t, P0).is_empty());
    assert!(t.on_battlefield(cyst));
    assert_eq!(attached_to(&t, cyst), None);
}

// ---------------------------------------------------------------------------------------
// Kindred Equipment: "Whenever a [type] creature enters, you may attach this to it."
// ---------------------------------------------------------------------------------------

#[test]
fn kindred_equipment_has_a_creature_type_and_the_equipment_type() {
    cr!("205.3d", "205.3g", "308.3", "301.5");
    ruling!(
        "Thornbite Staff",
        "Each of these Equipment has two subtypes listed on its type line. The first one is a creature type, which in this case is also a subtype of kindred. The second one is Equipment, which is a subtype of artifact."
    );
    supported("Thornbite Staff");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Thornbite Staff");
    let o = t.g.obj(staff);
    assert!(o.is(CardType::Kindred) && o.is(CardType::Artifact));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Shaman") && o.chars.has_subtype("Equipment"));
    assert_eq!(subtype_kinds("Shaman"), vec![SubtypeKind::Creature]);
    assert_eq!(subtype_kinds("Equipment"), vec![SubtypeKind::Artifact]);
    // It's an Equipment: it can be attached with its equip ability.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, staff, "Equip", 0).expect("equip");
    t.resolve();
    assert_eq!(attached_to(&t, staff), Some(Entity::Object(bears)));
    // Being a Shaman doesn't make it a creature: it doesn't trigger its own ability, and
    // it doesn't die with a creature's toughness of 0.
    assert!(t.on_battlefield(staff));
}

#[test]
fn a_kindred_equipment_can_attach_itself_to_an_opponents_creature() {
    cr!("301.5c", "301.5d", "702.6a", "113.8", "603.2");
    ruling!(
        "Thornbite Staff",
        "This triggers whenever any creature of the specified creature type enters, no matter who controls it. You may attach your Equipment to another player's creature this way, even though you can't do so with the equip ability."
    );
    ruling!(
        "Thornbite Staff",
        "If you attach an Equipment you control to another player's creature, you retain control of the Equipment, but you don't control the creature. Only you can activate the Equipment's equip ability, and if the Equipment's ability triggers again, you choose whether to move the Equipment. Only the creature's controller can activate any activated abilities the Equipment grants to the creature, and \"you\" in any abilities granted to the creature refers to that player."
    );
    // Thornbite Staff: "Equipped creature has "{2}, {T}: This creature deals 1 damage to
    // any target" and "Whenever a creature dies, untap this creature." Whenever a Shaman
    // creature enters, you may attach this Equipment to it. Equip {4}"
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Thornbite Staff");
    let mine = t.battlefield(P0, "Grizzly Bears");
    // The equip ability can target only P0's creatures.
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(their_bears)]);
    let _ = activate_named(&mut t, P0, staff, "Equip", 0);
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(their_bears))));
    assert!(cands.iter().any(|c| c.contains(&Entity::Object(mine))));
    t.clear_answers();
    t.resolve_all();
    let staff_now = t.g.current(staff);
    // An opponent's Shaman enters: P0 may attach the Staff to it.
    let from = t.asked().len();
    t.answer_yes(P0, true);
    let shaman = t.enter(P1, "Burning-Tree Emissary");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(attached_to(&t, staff_now), Some(Entity::Object(shaman)));
    assert_eq!(t.obj_now(staff_now).controller, P0);
    assert_eq!(t.obj_now(shaman).controller, P1);
    // Only the creature's controller can activate the granted ability (on their next
    // turn, once the Shaman isn't summoning sick).
    t.set_step(P1, Step::PrecombatMain);
    t.g.objects[shaman.0 as usize].summoning_sick = false;
    assert!(activatable(&mut t, P0, shaman).is_empty());
    t.lands(P1, "Wastes", 2);
    assert!(activatable(&mut t, P1, shaman)
        .iter()
        .any(|a| a.contains("damage")));
    // Only P0 can move the Equipment (P1 can't activate its equip ability).
    assert!(activatable(&mut t, P1, staff_now).is_empty());
    // Another Shaman entering (under P0's control): P0 chooses whether to move it.
    t.set_step(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    let second = t.enter(P0, "Burning-Tree Emissary");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(attached_to(&t, staff_now), Some(Entity::Object(second)));
}

// ---------------------------------------------------------------------------------------
// Manifest dread Equipment
// ---------------------------------------------------------------------------------------

#[test]
fn the_equipment_manifests_dread_even_if_it_left_the_battlefield() {
    cr!("701.62a", "608.2b", "603.3");
    ruling!(
        "Cursed Windbreaker",
        "You'll still manifest dread even if this Equipment isn't on the battlefield when its first ability resolves."
    );
    supported("Cursed Windbreaker");
    let mut t = TestGame::new(2);
    let wb = t.enter(P0, "Cursed Windbreaker");
    t.g.flush_events();
    t.settle();
    assert_eq!(on_stack(&t, "manifest dread"), 1);
    destroy(&mut t, wb);
    assert!(t.in_graveyard(P0, "Cursed Windbreaker"));
    let grave = t.graveyard_size(P0);
    t.resolve_all();
    let face_down: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.face_down)
        .map(|o| o.id)
        .collect();
    assert_eq!(face_down.len(), 1);
    assert_eq!(t.pt(face_down[0]), (2, 2));
    // The other card went to the graveyard; nothing is attached to the manifested creature.
    assert_eq!(t.graveyard_size(P0), grave + 1);
    assert!(t.in_graveyard(P0, "Cursed Windbreaker"));
}

// ---------------------------------------------------------------------------------------
// Protection from colors
// ---------------------------------------------------------------------------------------

#[test]
fn protection_from_a_color_prevents_only_what_it_says() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f");
    ruling!(
        "Sword of Sinew and Steel",
        "Protection from a color means that the equipped creature can't be blocked by creatures of that color, can't be the target of spells of that color or abilities from sources of that color, can't be enchanted or equipped by Auras or Equipment of that color, and all damage that sources of that color would deal to it is prevented. Nothing other than these events is prevented or illegal."
    );
    supported("Sword of Sinew and Steel");
    // Sword of Sinew and Steel: +2/+2, protection from black and from red.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Sinew and Steel", bears);
    assert_eq!(t.pt(bears), (4, 4));
    // Can't be targeted by a red spell.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    let _ = t.cast(P1, bolt).target(bears).try_go();
    let cands = target_candidates(&t, P1, from);
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(bears))));
    t.clear_answers();
    // Damage from a red source is prevented (Pyroclasm: 2 damage to each creature).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Sinew and Steel", bears);
    let other = t.battlefield(P0, "Hill Giant");
    let pyro = in_hand_with_mana(&mut t, P0, "Pyroclasm");
    t.cast(P0, pyro).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(other).damage, 2);
    // A black Aura can't enchant it: it's put into its owner's graveyard.
    let phyresis = t.battlefield(P1, "Phyresis");
    t.g.attach(phyresis, Entity::Object(bears));
    t.settle();
    assert!(t.in_graveyard(P1, "Phyresis"));
    // Can't be blocked by a red creature, but can be by a white one.
    let piker = t.battlefield(P1, "Goblin Piker");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(piker, bears)]);
    assert!(!t.g.is_blocking(piker));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Sinew and Steel", bears);
    let lions = t.battlefield(P1, "Savannah Lions");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(lions, bears)]);
    assert!(t.g.is_blocking(lions));
    // Nothing else is prevented: a black sweeper that doesn't target still destroys it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Sinew and Steel", bears);
    let damnation = in_hand_with_mana(&mut t, P0, "Damnation");
    t.cast(P0, damnation).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
