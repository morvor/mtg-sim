//! Rulings batch P185 — Sliver typal cards: granted "When this creature enters" abilities
//! (CR 603.6a, 113.6), "exile ... until this leaves the battlefield" (CR 610.3), granted
//! "dies" triggers and look-back (CR 603.10a), granted replacement effects for Slivers
//! dying together (CR 614, 616.1), triggers controlled by the entering Sliver's controller
//! (CR 603.3a), Slivers granting haste (CR 506.4), and damage from several sources at once
//! (CR 510.2, 120.3).

use crate::r_p185_common::*;
use crate::r_p079_common::concede;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, attached_to, damage, give_control};
use crate::r_s08_common::actions_of;
use crate::r_s25_common::cast_new;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, Color, ColorSet};
use mtg_engine::*;

/// P0's `name` enters; its "you may exile target creature an opponent controls until
/// this creature leaves the battlefield" trigger targets `target` and resolves.
fn constrict(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    let s = t.enter(P0, name);
    t.answer_targets(P0, &[Entity::Object(t.g.current(target))]);
    t.answer_yes(P0, true);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    s
}

/// A vanilla 2/2 blue Sliver card.
fn blue_sliver() -> mtg_engine::card::CardDef {
    let mut def = custom_card("Blue Sliver", "Creature — Sliver", "{1}{U}", Some((2, 2)), "");
    def.faces[0].chars.colors = ColorSet::single(Color::Blue);
    def
}

#[test]
fn constricting_sliver_grants_itself_the_exile_ability_and_it_lasts_until_that_sliver_leaves() {
    cr!("610.3", "603.6a", "400.7");
    ruling!(
        "Constricting Sliver",
        "When Constricting Sliver enters the battlefield, the ability it gives itself will trigger."
    );
    ruling!(
        "Constricting Sliver",
        "Constricting Sliver gives Slivers you control an ability that causes a zone change with a duration, a newer style of ability that’s somewhat reminiscent of older cards like Oblivion Ring. However, unlike Oblivion Ring’s abilities, this ability creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after the Sliver leaves the battlefield."
    );
    ruling!(
        "Constricting Sliver",
        "If a Sliver you control leaves the battlefield, only the card that particular Sliver exiled will return to the battlefield. Each card exiled by another Sliver will remain exiled until that Sliver leaves the battlefield."
    );
    ruling!(
        "Constricting Sliver",
        "The exiled card will return even if the Sliver whose ability exiled it doesn’t have that ability at the time (perhaps because Constricting Sliver is no longer on the battlefield)."
    );
    supported("Constricting Sliver");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let cs = constrict(&mut t, "Constricting Sliver", bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    let vs = constrict(&mut t, "Venser's Sliver", giant);
    assert_eq!(t.zone(giant), Zone::Exile);
    // Constricting Sliver leaves: only the Bears returns, as a new object.
    destroy(&mut t, cs);
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(giant), Zone::Exile);
    // Venser's Sliver no longer has the ability; when it leaves, the Giant still returns.
    destroy(&mut t, vs);
    t.settle();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn constricting_sliver_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b");
    ruling!(
        "Constricting Sliver",
        "If a Sliver leaves the battlefield before its enters-the-battlefield ability resolves, the target creature won’t be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Constricting Sliver");
    let s = t.enter(P0, "Venser's Sliver");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, s);
    t.resolve_all();
    // Never exiled (not exiled and returned): still the same object.
    assert!(t.g.is_live(bears) && t.g.obj(bears).zone == Zone::Battlefield);
    assert!(t.g.untils.is_empty());
}

#[test]
fn constricting_sliver_exile_drops_auras_unattaches_equipment_and_loses_counters() {
    cr!("400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Constricting Sliver",
        "Auras attached to the exiled creature will be put into their owners’ graveyards (unless they’re Theros block cards with the bestow ability). Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Holy Strength", bears);
    let sword = attach_new(&mut t, P1, "Bonesplitter", bears);
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, None);
    let cs = constrict(&mut t, "Constricting Sliver", bears);
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.on_battlefield(sword));
    assert_eq!(attached_to(&t, sword), None);
    destroy(&mut t, cs);
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(back, counters::PLUS1), 0);
    assert_eq!(t.pt(back), (2, 2));
}

#[test]
fn constricting_sliver_returns_the_card_under_its_owners_control() {
    cr!("610.3c", "110.2");
    ruling!(
        "Constricting Sliver",
        "The exiled card will return to the battlefield under its owner’s control."
    );
    let mut t = TestGame::new(3);
    // P2's Bears, controlled by P1, exiled by P0's Sliver: it returns under P2's control
    // (not its last controller's, nor the Sliver controller's).
    let bears = t.battlefield(P2, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    let cs = constrict(&mut t, "Constricting Sliver", bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    destroy(&mut t, cs);
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj_now(back).controller, P2);
}

#[test]
fn constricting_sliver_returned_card_cant_copy_the_sliver_that_left() {
    cr!("610.3", "610.3c", "707.5", "614.1c");
    ruling!(
        "Constricting Sliver",
        "The exiled card returns to the battlefield immediately after the Sliver leaves the battlefield. Nothing happens between the two events, including state-based actions. The two creatures aren’t on the battlefield at the same time. For example, if the returning creature is a Mercurial Pretender, it can’t enter the battlefield as a copy of the Sliver."
    );
    supported("Mercurial Pretender");
    let mut t = TestGame::new(2);
    // P0's Mercurial Pretender (copying a Bears), controlled by P1: it returns under
    // P0's control, where P0 controls only another Bears.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    let mp = crate::r_p023_common::enter_copying(&mut t, P0, "Mercurial Pretender", bears);
    give_control(&mut t, mp, P1);
    let cs = constrict(&mut t, "Constricting Sliver", mp);
    assert_eq!(t.zone(mp), Zone::Exile);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    destroy(&mut t, cs);
    t.settle();
    // The Sliver wasn't among the creatures it could copy.
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.is_empty());
    assert!(!offered.contains(&Entity::Object(cs)));
    assert!(offered.contains(&Entity::Object(bears)));
    let copies = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(copies.len(), 2, "it returned as a copy of the Bears");
    assert!(copies.iter().all(|c| t.obj_now(*c).controller == P0));
}

#[test]
fn constricting_sliver_card_returns_when_the_slivers_owner_leaves_the_game() {
    cr!("800.4a", "610.3c");
    ruling!(
        "Constricting Sliver",
        "In a multiplayer game, if the Sliver’s owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    constrict(&mut t, "Constricting Sliver", bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    concede(&mut t, P0);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn constricting_sliver_ability_source_is_the_entering_sliver() {
    cr!("113.7", "702.16b");
    ruling!(
        "Constricting Sliver",
        "The Sliver that enters the battlefield is the source of the ability that exiles the creature. For example, if a blue Sliver enters the battlefield under your control, the ability can target and exile a creature with protection from white."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Goblin Outlander");
    t.battlefield(P0, "Constricting Sliver");
    let blue = t.custom(P0, blue_sliver(), Zone::Hand(P0));
    t.lands(P0, "Island", 2);
    t.cast(P0, blue).go();
    t.answer_targets(P0, &[Entity::Object(goblin)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(goblin), Zone::Exile);
    // White Constricting Sliver's own trigger can't target it.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Goblin Outlander");
    constrict(&mut t, "Constricting Sliver", goblin);
    assert!(t.on_battlefield(goblin));
}

#[test]
fn haste_slivers_leaving_dont_remove_attacking_slivers() {
    cr!("506.4", "508.1");
    ruling!(
        "Blur Sliver",
        "If Blur Sliver leaves the battlefield during combat, any attacking Sliver creatures that came under your control this turn will continue to attack, even though they will no longer have haste."
    );
    ruling!(
        "Cloudshredder Sliver",
        "If Cloudshredder Sliver leaves the battlefield during combat, any attacking Slivers that came under your control this turn will continue to attack, even though they will no longer have haste."
    );
    for lord in ["Blur Sliver", "Cloudshredder Sliver"] {
        supported(lord);
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let s = t.battlefield_sick(P0, "Venser's Sliver");
        attack_with(&mut t, &[(s, Entity::Player(P1))]);
        destroy(&mut t, l);
        assert!(t.g.is_attacking(s), "{lord}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 17, "{lord}");
    }
}

#[test]
fn vampiric_sliver_checks_the_ability_as_the_damaged_creature_dies() {
    cr!("603.10a", "603.2", "613.1f");
    ruling!(
        "Vampiric Sliver",
        "If Vampiric Sliver is put into a graveyard from the battlefield at the same time as a creature dealt damage by another Sliver, the Vampiric Sliver’s ability will trigger for the other Sliver. That Sliver will get a +1/+1 counter."
    );
    ruling!(
        "Vampiric Sliver",
        "If a Sliver doesn’t have this ability when it deals damage, but does have it when the damaged creature is put into a graveyard, the ability will trigger."
    );
    ruling!(
        "Vampiric Sliver",
        "If a Sliver has this ability when it deals damage, but not when the damaged creature is put into a graveyard, nothing happens."
    );
    supported("Vampiric Sliver");
    // Gains the ability after dealing the damage: triggers.
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Venser's Sliver");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, vs, 1, giant);
    t.battlefield(P0, "Vampiric Sliver");
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(vs, counters::PLUS1), 1);
    // Loses it before the creature dies: nothing.
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Venser's Sliver");
    let vamp = t.battlefield(P0, "Vampiric Sliver");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, vs, 1, giant);
    destroy(&mut t, vamp);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(vs, counters::PLUS1), 0);
    // Vampiric Sliver dies at the same time as the damaged creature: it triggers.
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Venser's Sliver");
    let vamp = t.battlefield(P0, "Vampiric Sliver");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, vs, 1, giant);
    crate::r_p044_common::damage_at_once(
        &mut t,
        &[(Entity::Object(vamp), 3), (Entity::Object(giant), 2)],
    );
    assert!(!t.on_battlefield(vamp));
    assert!(!t.on_battlefield(giant));
    t.resolve_all();
    assert_eq!(t.counters(vs, counters::PLUS1), 1);
}

#[test]
fn pulmonic_sliver_replacement_applies_to_slivers_dying_with_it() {
    cr!("614.1a", "616.1", "614.6");
    ruling!(
        "Pulmonic Sliver",
        "The replacement effect that Pulmonic Sliver grants to all Slivers applies to Pulmonic Sliver and any Slivers that would die at the same time as it."
    );
    ruling!(
        "Pulmonic Sliver",
        "If multiple Slivers owned by one player would die at the same time, that player chooses any of those Slivers and puts them on top of their library in any order. They don't reveal this order."
    );
    supported("Pulmonic Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pulmonic Sliver");
    t.battlefield(P0, "Venser's Sliver");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    let lib = &t.g.players[0].library;
    let top: Vec<String> = lib
        .iter()
        .rev()
        .take(2)
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    let mut sorted = top.clone();
    sorted.sort();
    assert_eq!(sorted, vec!["Pulmonic Sliver", "Venser's Sliver"], "{top:?}");
    assert!(!t.in_graveyard(P0, "Pulmonic Sliver"));
}

#[test]
fn fungus_sliver_gets_one_counter_for_simultaneous_damage() {
    cr!("510.2", "120.3", "603.2c");
    ruling!(
        "Fungus Sliver",
        "If multiple sources deal damage to a Sliver at the same time and that Sliver survives the damage, it gets only one +1/+1 counter despite its impressive efforts."
    );
    supported("Fungus Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fungus Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(vs, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(a, vs), (b, vs)]);
    t.resolve_all();
    assert!(t.on_battlefield(vs));
    assert_eq!(t.counters(vs, counters::PLUS1), 1);
}

#[test]
fn mindwhip_sliver_ability_can_be_activated_as_soon_as_the_sliver_resolves() {
    cr!("117.3b", "602.5d");
    ruling!(
        "Mindwhip Sliver",
        "If you cast a Sliver as normal during your main phase, it will enter the battlefield and you’ll receive priority. If no abilities trigger because of this, you can activate its ability immediately, before any other player has a chance to remove it from the battlefield."
    );
    supported("Mindwhip Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindwhip Sliver");
    t.lands(P0, "Wastes", 5);
    let vs = cast_new(&mut t, P0, "Venser's Sliver", &[]);
    t.resolve_all();
    let vs = t.g.current(vs);
    assert!(t.on_battlefield(vs));
    assert_eq!(t.g.turn.priority, Some(P0));
    let acts = actions_of(&mut t, P0);
    assert!(
        acts.iter()
            .any(|a| matches!(a, Action::Activate { source, .. } if *source == vs)),
        "{acts:?}"
    );
}

#[test]
fn two_thorncaster_slivers_trigger_separately() {
    cr!("113.2c", "508.1m");
    ruling!(
        "Thorncaster Sliver",
        "If you control multiple Thorncaster Slivers and a Sliver you control attacks, each ability will trigger separately."
    );
    supported("Thorncaster Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thorncaster Sliver");
    t.battlefield(P0, "Thorncaster Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    attack_with(&mut t, &[(vs, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn harmonic_and_dormant_slivers_grant_triggers_controlled_by_the_entering_slivers_controller() {
    cr!("603.3a", "603.6a");
    ruling!(
        "Harmonic Sliver",
        "The ability Harmonic Sliver grants itself triggers when it enters the battlefield."
    );
    ruling!(
        "Harmonic Sliver",
        "The controller of the Sliver that enters the battlefield controls the triggered ability for that Sliver and chooses what to destroy, not necessarily the controller of Harmonic Sliver."
    );
    ruling!(
        "Dormant Sliver",
        "The last ability Dormant Sliver grants itself triggers when it enters the battlefield."
    );
    ruling!(
        "Dormant Sliver",
        "The controller of the Sliver that enters the battlefield draws a card, not necessarily the controller of Dormant Sliver."
    );
    ruling!(
        "Mesmeric Sliver",
        "This ability triggers when Mesmeric Sliver itself enters."
    );
    supported("Harmonic Sliver");
    supported("Dormant Sliver");
    supported("Mesmeric Sliver");
    // Harmonic Sliver entering: its own trigger.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    enter_with(&mut t, P0, "Harmonic Sliver", &[Entity::Object(anthem)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // P1's Sliver entering with P0's Harmonic Sliver out: P1 controls the trigger and
    // chooses the target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Harmonic Sliver");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    enter_with(&mut t, P1, "Venser's Sliver", &[Entity::Object(anthem)]);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).controller, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    // Dormant Sliver entering: P0 draws. P1's Sliver entering: P1 draws.
    let mut t = TestGame::new(2);
    let h0 = t.hand_size(P0);
    enter_with(&mut t, P0, "Dormant Sliver", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    enter_with(&mut t, P1, "Venser's Sliver", &[]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1 + 1));
    // Mesmeric Sliver entering: its own fateseal trigger.
    let mut t = TestGame::new(2);
    enter_with(&mut t, P0, "Mesmeric Sliver", &[]);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn diffusion_sliver_taxes_opponents_spells_activated_and_triggered_abilities() {
    cr!("603.2", "115.1", "701.6a");
    ruling!(
        "Diffusion Sliver",
        "The granted ability applies to any spell (including Aura spells), activated ability, or triggered ability that’s controlled by an opponent that targets a Sliver you control."
    );
    supported("Diffusion Sliver");
    // An Aura spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diffusion Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 2);
    let pac = t.hand(P1, "Pacifism");
    t.cast(P1, pac).target(vs).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Pacifism"));
    // An activated ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diffusion Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P1, pyro, 0, &[Entity::Object(vs)]).unwrap();
    t.resolve_all();
    assert_eq!(marked(&t, vs), 0);
    // A triggered ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diffusion Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    enter_with(&mut t, P1, "Flametongue Kavu", &[Entity::Object(vs)]);
    t.resolve_all();
    assert!(t.on_battlefield(vs));
    // P0's own spell isn't taxed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Diffusion Sliver");
    let vs = t.battlefield(P0, "Venser's Sliver");
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(vs)]);
    t.resolve_all();
    assert_eq!(t.pt(vs), (6, 6));
}
