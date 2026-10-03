//! Rulings batch P210 — enchant (CR 303.4, 702.5): what Auras' triggered and activated
//! abilities see as they trigger and resolve (CR 603, 608.2): the creature enchanted at
//! resolution, last known information, intervening "if" clauses, delayed exile, damage
//! triggers, and control effects tied to an Aura.

use crate::r_p209_common::*;
use crate::r_p210_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s09_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves the Aura (or Equipment) to `to`, as an effect that attaches it would.
fn move_aura(t: &mut TestGame, aura: ObjectId, to: ObjectId) {
    assert!(t.g.attach(aura, to.into()));
    t.g.recompute();
}

/// `p` gains control of `id` until end of turn (as a resolving "Threaten" would).
fn control_until_eot(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// `p` casts the Aura `name` on `on`; as it resolves, its enters trigger targets
/// `targets`; everything resolves. The Aura.
fn cast_aura_then(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    on: ObjectId,
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    let aura = crate::r_s03_common::in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast(p, aura).target(on).go();
    t.g.resolve_top();
    t.answer_targets(p, targets);
    t.resolve_all();
    t.g.current(aura)
}

// ---------------------------------------------------------------------------------------
// The creature enchanted as the ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn auras_abilities_affect_the_creature_enchanted_as_they_resolve() {
    cr!("608.2h", "611.2c", "603.3");
    ruling!(
        "Firebreathing",
        "The ability affects whichever creature is enchanted by Firebreathing at the time the ability resolves. The bonus remains even if Firebreathing stops enchanting that creature."
    );
    ruling!(
        "Forced Adaptation",
        "The creature that gets the +1/+1 counter is the creature enchanted by Forced Adaptation when the ability resolves."
    );
    ruling!(
        "Dreadful Apathy",
        "The creature that's exiled is the creature Dreadful Apathy enchants as its last ability resolves."
    );
    // Firebreathing.
    supported("Firebreathing");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Raging Goblin");
    let fb = attach_new(&mut t, P0, "Firebreathing", a);
    lots_of_mana(&mut t, P0);
    activate_containing(&mut t, P0, fb, "+1/+0").unwrap();
    move_aura(&mut t, fb, b);
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (4, 3));
    move_aura(&mut t, fb, c);
    assert_eq!(t.pt(b), (4, 3));
    assert_eq!(t.pt(c), (1, 1));

    // Forced Adaptation.
    supported("Forced Adaptation");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let fa = attach_new(&mut t, P0, "Forced Adaptation", a);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    move_aura(&mut t, fa, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 0);
    assert_eq!(t.counters(b, counters::PLUS1), 1);

    // Dreadful Apathy.
    supported("Dreadful Apathy");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let da = attach_new(&mut t, P0, "Dreadful Apathy", a);
    lots_of_mana(&mut t, P0);
    activate_containing(&mut t, P0, da, "Exile").unwrap();
    move_aura(&mut t, da, b);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn elemental_mastery_uses_last_known_power_and_its_tokens_are_exiled_anyway() {
    cr!("608.2h", "603.7", "113.7a");
    ruling!(
        "Elemental Mastery",
        "The enchanted creature's power is checked at the time the ability resolves. If the enchanted creature has left the battlefield by then, its last known information is used."
    );
    ruling!(
        "Elemental Mastery",
        "The tokens will be exiled in the End step even if Elemental Mastery or the enchanted creature has left the battlefield by then."
    );
    supported("Elemental Mastery");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Elemental Mastery", giant);
    activate_containing(&mut t, P0, giant, "Create X").unwrap();
    // In response, the Giant gets +3/+0 ... then is destroyed (and the Aura with it).
    t.g.recompute();
    cast_spell(&mut t, P0, "Brute Force", &[giant.into()]);
    t.g.resolve_top();
    t.settle();
    destroy(&mut t, giant);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Elemental Mastery"));
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Elemental").len(), 6);
    t.advance_to(P1, Step::Upkeep);
    assert!(tokens_named(&t, "Elemental").is_empty());
}

fn tokens_named(t: &TestGame, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

#[test]
fn splinter_twin_token_is_exiled_whoever_controls_it_and_whatever_left() {
    cr!("603.7", "603.7b", "111.1");
    ruling!(
        "Splinter Twin",
        "The token is exiled at the beginning of the next end step regardless of who controls it at that time, or whether Splinter Twin or the enchanted creature is still on the battlefield at that time."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Splinter Twin", bears);
    activate_containing(&mut t, P0, bears, "Create a token").unwrap();
    t.resolve_all();
    let tok = tokens(&t, P0)[0];
    give_control(&mut t, tok, P1);
    assert_eq!(t.obj_now(tok).controller, P1);
    destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Splinter Twin"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
}

#[test]
fn splinter_twin_token_copies_only_the_printed_creature_plus_haste() {
    cr!("707.2", "111.1", "113.6");
    ruling!(
        "Splinter Twin",
        "The token that's put onto the battlefield copies exactly what's printed on the enchanted creature"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(bears.into(), counters::PLUS1, 1, None);
    attach_new(&mut t, P0, "Splinter Twin", bears);
    attach_new(&mut t, P0, "Lifelink", bears);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    activate_containing(&mut t, P0, bears, "Create a token").unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    let tok = tokens(&t, P0)[0];
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Grizzly Bears");
    assert!(!o.tapped);
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert!(t.g.attachments_of(tok.into()).is_empty());
    assert!(has_kw(&t, tok, mtg_engine::keywords::KeywordKind::Haste));
    assert!(!has_kw(&t, tok, mtg_engine::keywords::KeywordKind::Lifelink));
    assert!(!can_activate(&mut t, P0, tok));
}

// ---------------------------------------------------------------------------------------
// Intervening "if" and resolution checks
// ---------------------------------------------------------------------------------------

#[test]
fn artificers_hex_checks_the_equipment_as_it_triggers_and_resolves() {
    cr!("603.4");
    ruling!(
        "Artificer's Hex",
        "The ability checks to see if the enchanted Equipment is attached to a creature as your upkeep begins. If it’s not, the ability won’t trigger at all. If it is, the ability will check again as it tries to resolve. If the enchanted Equipment isn’t attached to a creature at that time, the ability won’t do anything."
    );
    // Unattached Equipment: no trigger.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P1, "Bonesplitter");
    attach_new(&mut t, P0, "Artificer's Hex", sword);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "destroy"), 0);
    // Unattached in response: the ability does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = attach_new(&mut t, P1, "Bonesplitter", bears);
    attach_new(&mut t, P0, "Artificer's Hex", sword);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "destroy"), 1);
    t.g.unattach(sword);
    t.g.recompute();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(sword));
}

#[test]
fn ordeal_of_nylea_checks_counters_only_as_the_attack_trigger_resolves() {
    cr!("603.2", "608.2c");
    ruling!(
        "Ordeal of Nylea",
        "The check of whether the enchanted creature has three or more +1/+1 counters on it happens as part of the resolution of the attack triggered ability. If the third +1/+1 counter is put on the enchanted creature any other way, you won't sacrifice Ordeal of Nylea until the next time the creature attacks."
    );
    supported("Ordeal of Nylea");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ordeal = attach_new(&mut t, P0, "Ordeal of Nylea", bears);
    t.g.add_counters(bears.into(), counters::PLUS1, 3, None);
    t.settle();
    assert!(t.on_battlefield(ordeal));
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 4);
    assert!(t.in_graveyard(P0, "Ordeal of Nylea"));
}

#[test]
fn see_red_is_kept_if_any_of_your_creatures_attacked() {
    cr!("603.4", "508.1");
    ruling!(
        "See Red",
        "See Red's last ability is satisfied if any creature has attacked, similar to raid abilities. The creature it enchants doesn't have to have attacked."
    );
    supported("See Red");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let sr = attach_new(&mut t, P0, "See Red", bears);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, P1.into())]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(sr));
    // Next turn, nothing attacks: it's sacrificed.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "See Red"));
}

// ---------------------------------------------------------------------------------------
// Damage triggers: any damage
// ---------------------------------------------------------------------------------------

#[test]
fn damage_triggers_of_auras_see_noncombat_damage_too() {
    cr!("120.3", "603.2");
    ruling!(
        "Snake Umbra",
        "The ability triggers when the enchanted creature deals any damage, not just combat damage."
    );
    ruling!(
        "Helm of the Ghastlord",
        "They trigger from any damage, not just combat damage."
    );
    ruling!(
        "Elder Mastery",
        "The last ability triggers from any damage dealt by the enchanted creature, not just combat damage."
    );
    ruling!(
        "Spirit Link",
        "The triggered ability triggers when the enchanted creature deals any damage, not only combat damage."
    );
    ruling!(
        "Liliana's Talent",
        "The last ability of Liliana's Talent will trigger whenever a creature deals any damage to the enchanted planeswalker, not just combat damage."
    );
    // Snake Umbra: P0 draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Snake Umbra", bears);
    let h = t.hand_size(P0);
    t.answer_yes(P0, true);
    deal(&mut t, bears, 1, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1);
    // Helm of the Ghastlord on a black creature: P1 discards.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Black Knight");
    attach_new(&mut t, P0, "Helm of the Ghastlord", knight);
    t.hand(P1, "Forest");
    deal(&mut t, knight, 1, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // Elder Mastery: P1 discards two.
    supported("Elder Mastery");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Elder Mastery", bears);
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    deal(&mut t, bears, 1, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    // Spirit Link: damage to a creature.
    supported("Spirit Link");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Spirit Link", giant);
    let bears = t.battlefield(P1, "Grizzly Bears");
    deal(&mut t, giant, 3, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Liliana's Talent: a creature deals noncombat damage to the planeswalker.
    supported("Liliana's Talent");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    attach_new(&mut t, P0, "Liliana's Talent", jace);
    let bears = t.battlefield(P1, "Grizzly Bears");
    deal(&mut t, bears, 1, jace);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn pooling_venom_triggers_on_any_tapping() {
    cr!("603.2", "701.26a");
    ruling!(
        "Pooling Venom",
        "The triggered ability triggers whenever the enchanted land becomes tapped, not just when it’s tapped for mana."
    );
    supported("Pooling Venom");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    attach_new(&mut t, P0, "Pooling Venom", land);
    tap(&mut t, land);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn curse_of_stalked_prey_counts_any_creatures_combat_damage() {
    cr!("510.2", "603.2");
    ruling!(
        "Curse of Stalked Prey",
        "The ability will trigger when any creature deals combat damage to the enchanted player, including one controlled by another opponent"
    );
    supported("Curse of Stalked Prey");
    let mut t = TestGame::new(3);
    attach_new(&mut t, P0, "Curse of Stalked Prey", P1);
    let bears = t.battlefield(P2, "Grizzly Bears");
    to_combat(&mut t, P2);
    attack_with(&mut t, &[(bears, P1.into())]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn favor_of_the_woods_triggers_once_however_many_creatures_it_blocks() {
    cr!("509.1i", "509.1a", "603.2");
    ruling!(
        "Favor of the Woods",
        "The ability will trigger only once per combat, even if the enchanted creature somehow blocks multiple attacking creatures."
    );
    ruling!(
        "Iona's Blessing",
        "The ability to block an additional creature is cumulative. If a creature is enchanted with two Iona’s Blessings, it can block three creatures each combat."
    );
    supported("Favor of the Woods");
    supported("Iona's Blessing");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Favor of the Woods", wall);
    attach_new(&mut t, P1, "Iona's Blessing", wall);
    attach_new(&mut t, P1, "Iona's Blessing", wall);
    assert_eq!(t.pt(wall), (6, 6));
    let atk: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Raging Goblin")).collect();
    to_combat(&mut t, P0);
    attack_with(
        &mut t,
        &atk.iter().map(|a| (*a, P1.into())).collect::<Vec<_>>(),
    );
    // Four blocks are too many; three are fine.
    let four: Vec<_> = atk.iter().map(|a| (wall, *a)).collect();
    assert_eq!(t.g.max_blocks(wall), Some(3));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(four[..3].to_vec()),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gain 3 life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
}

#[test]
fn necromantic_thirst_may_return_the_enchanted_creature_itself() {
    cr!("603.3", "603.3d", "702.19c", "702.2c");
    ruling!(
        "Necromantic Thirst",
        "The target is chosen just after any creatures dealt lethal damage at the same time that the enchanted creature dealt damage have been put into the graveyard. That might include the enchanted creature itself, if it had trample and was blocked, for example."
    );
    supported("Necromantic Thirst");
    let mut t = TestGame::new(2);
    let baloths = t.battlefield(P0, "Rampaging Baloths");
    attach_new(&mut t, P0, "Necromantic Thirst", baloths);
    let rats = t.battlefield(P1, "Typhoid Rats");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(baloths, P1.into())]);
    t.answer_yes(P0, true);
    block_and_finish(&mut t, P1, &[(rats, baloths)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_hand(P0, "Rampaging Baloths"));
}

#[test]
fn grasp_of_the_hieromancer_taps_the_target_before_blockers() {
    cr!("508.1m", "509.1a");
    ruling!(
        "Grasp of the Hieromancer",
        "The triggered ability granted to the enchanted creature will tap the creature before blockers are declared."
    );
    supported("Grasp of the Hieromancer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Grasp of the Hieromancer", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, P1.into())]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.obj_now(giant).tapped);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn glistening_oil_returns_however_it_goes_to_the_graveyard() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Glistening Oil",
        "The last ability will trigger no matter how Glistening Oil is put into the graveyard from the battlefield, not just when the enchanted creature is put into the graveyard."
    );
    supported("Glistening Oil");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let oil = attach_new(&mut t, P0, "Glistening Oil", giant);
    destroy(&mut t, oil);
    t.resolve_all();
    assert!(t.in_hand(P0, "Glistening Oil"));
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------------------
// Targets of an Aura's enters ability
// ---------------------------------------------------------------------------------------

#[test]
fn aura_enters_abilities_can_target_beyond_the_enchanted_creature() {
    cr!("115.1", "603.3d");
    ruling!(
        "Cartouche of Ambition",
        "The target creature for the triggered ability of Cartouche of Ambition doesn't have to be the creature it enchants."
    );
    ruling!(
        "Sporogenic Infection",
        "Sporogenic Infection's second ability can target any player, not just the controller of the enchanted creature."
    );
    // Cartouche on P0's Bears, the -1/-1 counter on P1's Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    cast_aura_then(&mut t, P0, "Cartouche of Ambition", bears, &[giant.into()]);
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    // Sporogenic Infection on P1's Bears, targeting P2, who sacrifices a creature.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    cast_aura_then(&mut t, P0, "Sporogenic Infection", bears, &[P2.into()]);
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Durations tied to the Aura
// ---------------------------------------------------------------------------------------

#[test]
fn faith_unbroken_returns_the_card_when_it_leaves_and_waits_for_sbas_otherwise() {
    cr!("610.3", "610.3c", "704.5m", "603.6c");
    ruling!(
        "Faith Unbroken",
        "The exiled card returns to the battlefield immediately after Faith Unbroken leaves the battlefield. Nothing happens between the two events, including state-based actions. However, if the creature Faith Unbroken enchants leaves the battlefield, Faith Unbroken won't leave itself until state-based actions are checked."
    );
    supported("Faith Unbroken");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let faith = cast_aura(&mut t, P0, "Faith Unbroken", bears).unwrap();
    assert!(t.in_exile("Hill Giant"));
    // The Bears leaves: until SBAs are checked, Faith stays and the Giant stays exiled.
    let b = t.g.current(bears);
    t.g.destroy(b, None);
    t.g.flush_events();
    assert!(t.on_battlefield(faith));
    assert!(t.in_exile("Hill Giant"));
    t.settle();
    assert!(t.in_graveyard(P0, "Faith Unbroken"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(!t.g.is_live(giant));
    // Faith itself leaves: the card is back right away.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let faith = cast_aura(&mut t, P0, "Faith Unbroken", bears).unwrap();
    let f = t.g.current(faith);
    t.g.destroy(f, None);
    t.g.flush_events();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn giants_grasp_control_lasts_until_the_aura_leaves() {
    cr!("611.2b", "303.4d", "704.5m");
    ruling!(
        "Giant's Grasp",
        "The control-changing effect doesn't expire if another player gains control of Giant's Grasp (although this is likely to cause it to leave the battlefield because it can enchant only a Giant controlled by the controller of Giant's Grasp). The effect expires only once Giant's Grasp is no longer on the battlefield."
    );
    supported("Giant's Grasp");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let grasp = cast_aura_then(&mut t, P0, "Giant's Grasp", giant, &[bears.into()]);
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1 gains control of the Aura: until SBAs, the Bears stays with P0.
    {
        use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
        let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
        ctx.targets = vec![vec![Entity::Object(t.g.current(grasp))]];
        t.g.exec(
            &Effect::GainControl {
                what: Sel::Target(0),
                who: PlayerRef::You,
                duration: Duration::Permanent,
            },
            &mut ctx,
        );
        t.g.recompute();
    }
    assert_eq!(t.obj_now(grasp).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.settle();
    assert!(t.in_graveyard(P0, "Giant's Grasp"));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn fealty_to_the_realm_control_effect_keeps_its_timestamp() {
    cr!("613.7a", "613.2", "725.1");
    ruling!(
        "Fealty to the Realm",
        "The timestamp of Fealty to the Realm's control-changing effect is the time at which it entered the battlefield and won't change when another player becomes the monarch."
    );
    use mtg_engine::designations::become_monarch;
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Fealty to the Realm", bears);
    become_monarch(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P0);
    // P2 gains control until end of turn; P1 then becomes the monarch.
    control_until_eot(&mut t, bears, P2);
    assert_eq!(t.obj_now(bears).controller, P2);
    become_monarch(&mut t.g, P1);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P2);
    // The P2 effect ends: the current monarch controls it.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P1);
}
