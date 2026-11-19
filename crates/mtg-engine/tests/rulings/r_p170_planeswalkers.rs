//! Rulings batch P170 — planeswalker synergies: loyalty ability limits (CR 606.3), The
//! Chain Veil, loyalty counters from spells (CR 608.2b), "dies" triggers of creatures and
//! planeswalkers (CR 603.6c, 603.10a), damage redirection (Gideon's Sacrifice, CR 614.9,
//! 616.1), costs paid before anyone can respond (CR 601.2h), and enters-the-battlefield
//! sacrifices (CR 614.12).

use crate::r_p170_common::*;
use crate::r_p189_common::triggers_of;
use crate::r_s01_common::attack_with;
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn loyalty(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "loyalty")
}

/// P0 sacrifices the permanent and settles.
fn sacrifice(t: &mut TestGame, id: ObjectId) {
    crate::r_p127_common::sacrifice(t, id);
}

#[test]
fn ichormoon_gauntlet_grants_abilities_but_not_more_activations() {
    cr!("606.3");
    ruling!(
        "Ichormoon Gauntlet",
        "Only one loyalty ability of each planeswalker may be activated per turn, no matter how many it has gained."
    );
    supported("Ichormoon Gauntlet");
    // The granted [0]: Proliferate, then no other loyalty ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichormoon Gauntlet");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.answer_choose(P0, &[obj(jace)]);
    activate_containing(&mut t, P0, jace, "Proliferate").expect("the granted ability");
    t.resolve_all();
    assert_eq!(loyalty(&t, jace), 4, "proliferated");
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    assert!(activate_containing(&mut t, P0, jace, "Proliferate").is_err());
    // A printed one first, then not the granted one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichormoon Gauntlet");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    assert!(activate_containing(&mut t, P0, jace, "Proliferate").is_err());
}

/// P0 activates The Chain Veil's last ability (lands added) and it resolves.
fn veil(t: &mut TestGame, veil: ObjectId) {
    t.lands(P0, "Wastes", 4);
    t.activate(P0, veil, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn the_chain_veil_covers_later_planeswalkers_and_stacks() {
    cr!("606.3", "613.11");
    ruling!(
        "The Chain Veil",
        "Because the last ability modifies the rules of the game, it affects not only planeswalkers you control when it resolves, but also planeswalkers that come under your control later in the turn."
    );
    ruling!(
        "The Chain Veil",
        "Each additional time The Chain Veil’s last ability resolves will allow you to activate a loyalty ability of each planeswalker you control an additional time. For example, if you activate The Chain Veil’s last ability, untap it, then activate it again, you can activate a loyalty ability of a planeswalker you control three times that turn."
    );
    ruling!(
        "The Chain Veil",
        "The second loyalty ability you activate doesn’t have to be the same as the first ability. For example, you could activate a planeswalker’s first ability twice, or you could activate a planeswalker’s first ability, then activate its second ability."
    );
    // Activated with no planeswalker; Jace enters later: two activations, the second a
    // different ability (+2, then −1).
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "The Chain Veil");
    veil(&mut t, v);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, jace, 1, &[pl(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(loyalty(&t, jace), 4);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    // Twice (untapped in between): three activations of the same ability.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "The Chain Veil");
    let jace = t.battlefield(P0, "Jace Beleren");
    veil(&mut t, v);
    crate::r_p127_common::untap(&mut t, v);
    veil(&mut t, v);
    for _ in 0..3 {
        t.activate(P0, jace, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(loyalty(&t, jace), 9);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
}

#[test]
fn the_chain_veil_counts_an_activation_of_a_planeswalker_that_left() {
    cr!("603.4", "606.3");
    ruling!(
        "The Chain Veil",
        "For the first ability, it doesn’t matter whether the planeswalker is still on the battlefield as your end step begins. If you activated one of its loyalty abilities that turn, The Chain Veil’s triggered ability won’t trigger."
    );
    for activated in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Chain Veil");
        let jace = t.battlefield(P0, "Jace Beleren");
        if activated {
            t.activate(P0, jace, 0, &[]).unwrap();
            t.resolve_all();
        }
        destroy(&mut t, jace);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.life(P0), if activated { 20 } else { 18 });
    }
}

#[test]
fn settle_the_score_puts_both_counters_on_one_planeswalker_if_it_resolves() {
    cr!("608.2b", "122.1");
    ruling!(
        "Settle the Score",
        "Both loyalty counters must be put onto the same planeswalker."
    );
    ruling!(
        "Settle the Score",
        "If the target creature is an illegal target by the time Settle the Score tries to resolve, the spell doesn’t resolve. You won’t put loyalty counters on a planeswalker."
    );
    supported("Settle the Score");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(ajani)]);
    cast_card(&mut t, P0, "Settle the Score", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!((loyalty(&t, jace), loyalty(&t, ajani)), (3, 6));
    // The creature leaves in response: no counters.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = cast_card(&mut t, P0, "Settle the Score", &[obj(giant)]);
    crate::r_p127_common::bounce(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Settle the Score"));
    assert!(!crate::r_p108_common::resolved(&t, spell));
    assert_eq!(loyalty(&t, jace), 3);
}

#[test]
fn the_elderspell_works_without_your_own_planeswalker_or_targets() {
    cr!("115.1", "601.2c", "608.2b");
    ruling!(
        "The Elderspell",
        "You can cast The Elderspell without controlling any planeswalkers. The target planeswalkers will still be destroyed. You can also cast it without targeting any planeswalkers if you really want to do so."
    );
    supported("The Elderspell");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Jace Beleren");
    let b = t.battlefield(P1, "Ajani Goldmane");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    cast_card(&mut t, P0, "The Elderspell", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert!(t.in_graveyard(P1, "Ajani Goldmane"));
    // No targets at all: it resolves (it isn't removed for having no legal target).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Jace Beleren");
    t.answer_targets(P0, &[]);
    let spell = cast_card(&mut t, P0, "The Elderspell", &[]);
    t.resolve_all();
    assert!(crate::r_p108_common::resolved(&t, spell));
    assert!(t.in_graveyard(P0, "The Elderspell"));
    assert!(t.on_battlefield(a));
}

#[test]
fn planebound_accomplices_planeswalker_is_sacrificed_even_without_the_accomplice() {
    cr!("603.7", "603.7a");
    ruling!(
        "Planebound Accomplice",
        "At the beginning of the next end step, you’ll sacrifice the planeswalker if it’s still on the battlefield, even if Planebound Accomplice has left the battlefield."
    );
    supported("Planebound Accomplice");
    let mut t = TestGame::new(2);
    let acc = t.battlefield(P0, "Planebound Accomplice");
    let jace = t.hand(P0, "Jace Beleren");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(jace)]);
    t.activate(P0, acc, 0, &[]).unwrap();
    t.resolve_all();
    let jace = t.g.current(jace);
    assert!(t.on_battlefield(jace));
    destroy(&mut t, acc);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Jace Beleren"));
}

#[test]
fn stenn_exiled_during_an_end_step_returns_at_the_next_one() {
    // 603.7a: a delayed trigger created after its event (this end step began) waits for
    // the next one.
    cr!("603.7a", "513.1");
    ruling!(
        "Stenn, Paranoid Partisan",
        "If Stenn's controller activates its last ability during a player's end step, the delayed triggered ability that returns it to the battlefield won't trigger until the next time an end step begins, usually during the next player's turn."
    );
    supported("Stenn, Paranoid Partisan");
    let mut t = TestGame::new(2);
    let stenn = t.enter(P0, "Stenn, Paranoid Partisan");
    t.advance_to(P0, Step::End);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, stenn, 0, &[]).expect("Stenn's ability");
    t.resolve_all();
    assert!(t.in_exile("Stenn, Paranoid Partisan"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Stenn, Paranoid Partisan"), "not back this turn");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Stenn, Paranoid Partisan").len(), 1);
}

#[test]
fn heartfires_sacrifice_is_a_cost_paid_before_anyone_can_respond() {
    cr!("601.2h", "601.2i", "117.3c");
    ruling!(
        "Heartfire",
        "Players can respond to Heartfire only after it’s been cast and all its costs have been paid. No one can try to destroy the permanent you sacrificed to stop you from casting this spell."
    );
    supported("Heartfire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let spell = cast_card(&mut t, P0, "Heartfire", &[pl(P1)]);
    // Once it's cast (the first moment anyone could respond), the Bears are already in
    // the graveyard: sacrificed while it was being cast.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn devouring_hellion_can_sacrifice_only_permanents_already_there() {
    cr!("614.12", "614.1c", "603.6a");
    ruling!(
        "Devouring Hellion",
        "If Devouring Hellion somehow enters the battlefield at the same time as another creature or planeswalker, you can’t sacrifice that creature or planeswalker. You may choose only permanents that are already on the battlefield."
    );
    ruling!(
        "Devouring Hellion",
        "If any of the sacrificed permanents have abilities that trigger when Devouring Hellion enters the battlefield (such as that of Kronch Wrangler), those abilities won’t trigger."
    );
    supported("Devouring Hellion");
    supported("Kronch Wrangler");
    // Entering with a Grizzly Bears: only the Hill Giant already there can be sacrificed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![obj(giant)]),
    );
    let from = t.asked().len();
    let objs =
        crate::r_p116_common::enter_together(&mut t, P0, &["Devouring Hellion", "Grizzly Bears"]);
    t.settle();
    let candidates: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(candidates.contains(&obj(giant)));
    assert!(!candidates
        .iter()
        .any(|e| objs.iter().any(|o| obj(*o) == *e)));
    let hellion = objs
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).chars.name == "Devouring Hellion")
        .unwrap();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.counters(hellion, "+1/+1"), 2);
    // Kronch Wrangler sacrificed: it doesn't see the 4/4 Hellion enter.
    for sac_wrangler in [true, false] {
        let mut t = TestGame::new(2);
        let wrangler = t.battlefield(P0, "Kronch Wrangler");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let victim = if sac_wrangler { wrangler } else { bears };
        t.answer_yes(P0, true);
        t.answer(
            P0,
            DecisionKind::Entities,
            Answer::Entities(vec![obj(victim)]),
        );
        let hellion = t.enter(P0, "Devouring Hellion");
        t.resolve_all();
        assert_eq!(t.pt(hellion), (4, 4));
        if sac_wrangler {
            assert!(t.in_graveyard(P0, "Kronch Wrangler"));
            assert_eq!(triggers_of(&t, wrangler), 0);
        } else {
            assert_eq!(t.counters(wrangler, "+1/+1"), 1, "the control: it triggers");
        }
    }
}

#[test]
fn death_triggers_for_creatures_and_planeswalkers() {
    cr!("603.6c", "603.10a", "700.4");
    ruling!(
        "Cruel Celebrant",
        "If Cruel Celebrant dies at the same time as one or more other creatures and/or planeswalkers you control, its ability triggers for each of them."
    );
    ruling!(
        "Cruel Celebrant",
        "If a planeswalker that's also a creature dies, Cruel Celebrant's ability triggers only once."
    );
    ruling!(
        "Rising Populace",
        "If a planeswalker that’s also a creature dies, Rising Populace’s ability triggers only once."
    );
    ruling!(
        "Rising Populace",
        "If Rising Populace and another creature you control die simultaneously (perhaps because they were both dealt lethal damage in combat), Rising Populace won’t be on the battlefield as its triggered ability resolves. It can’t be saved by the +1/+1 counter that would have been put on it."
    );
    supported("Cruel Celebrant");
    supported("Rising Populace");
    // Pyroclasm kills Cruel Celebrant and two Grizzly Bears: three triggers.
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Cruel Celebrant");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, cc), 3);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    // Gideon Blackblade is a creature during P0's turn: one trigger each.
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Cruel Celebrant");
    let rp = t.battlefield(P0, "Rising Populace");
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    assert!(t.obj_now(gideon).is_creature());
    sacrifice(&mut t, gideon);
    t.resolve_all();
    assert_eq!(triggers_of(&t, cc), 1);
    assert_eq!(triggers_of(&t, rp), 1);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    assert_eq!(t.counters(rp, "+1/+1"), 1);
    // Rising Populace and a Bears die together: the trigger resolves, Populace is gone.
    let mut t = TestGame::new(2);
    let rp = t.battlefield(P0, "Rising Populace");
    t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, rp), 1);
    assert!(t.in_graveyard(P0, "Rising Populace"));
}

#[test]
fn cruel_celebrant_in_two_headed_giant() {
    cr!("810.9", "603.6c");
    ruling!(
        "Cruel Celebrant",
        "In a Two-Headed Giant game, Cruel Celebrant's ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    let cc = t.battlefield(P0, "Cruel Celebrant");
    // P0 and P1 are a team; P2 and P3 the opposing team.
    let before = (t.life(P0), t.life(P2));
    sacrifice(&mut t, cc);
    t.resolve_all();
    assert_eq!(t.life(P0), before.0 + 1);
    assert_eq!(t.life(P1), t.life(P0));
    assert_eq!(t.life(P2), before.1 - 2);
    assert_eq!(t.life(P3), t.life(P2));
}

/// P0 casts Gideon's Sacrifice choosing `chosen` (when given).
fn gideons_sacrifice(t: &mut TestGame, chosen: Option<ObjectId>) {
    supported("Gideon's Sacrifice");
    if let Some(c) = chosen {
        t.answer_choose(P0, &[obj(c)]);
    }
    cast_card(t, P0, "Gideon's Sacrifice", &[]);
    t.resolve_all();
}

/// The damage events since `from`: (source, target, amount, combat).
fn damage_since(t: &TestGame, from: usize) -> Vec<(ObjectId, Entity, u32, bool)> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                combat,
            } => Some((*source, *target, *amount, *combat)),
            _ => None,
        })
        .collect()
}

#[test]
fn gideons_sacrifice_redirects_without_changing_source_or_combat() {
    cr!("614.9", "614.1a", "510.2");
    ruling!(
        "Gideon's Sacrifice",
        "Gideon's Sacrifice doesn't change the source of the damage or whether the damage is combat damage."
    );
    ruling!(
        "Gideon's Sacrifice",
        "Gideon's Sacrifice has no effect on damage already dealt earlier in the turn."
    );
    ruling!(
        "Gideon's Sacrifice",
        "More damage can be redirected to the chosen creature than it has toughness or to the chosen planeswalker than it has loyalty, as long as that damage is all dealt at once (like combat damage is) or is all dealt while a single spell or ability is resolving."
    );
    // Everything happens during P0's turn ("this turn"); P1 casts instants.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Earlier damage stays where it was dealt.
    cast_card(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    gideons_sacrifice(&mut t, Some(bears));
    assert_eq!(t.life(P0), 18);
    // P1's Lightning Bolt: 3 damage to the 2/2 Bears, by the Bolt, not combat damage.
    let from = t.g.turn_events.len();
    let bolt = cast_card(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(damage_since(&t, from), vec![(bolt, obj(bears), 3, false)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Combat damage stays combat damage from its source: P1's lifelinking Vampire
    // Nighthawk blocks P0's Grizzly Bears; its damage goes to the chosen Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    gideons_sacrifice(&mut t, Some(giant));
    attack_with(&mut t, &[(bears, pl(P1))]);
    let from = t.g.turn_events.len();
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(hawk, bears)]);
    let d = damage_since(&t, from);
    assert!(d.contains(&(hawk, obj(giant), 2, true)), "{d:?}");
    assert!(!d.iter().any(|x| x.1 == obj(bears)));
    assert_eq!(t.life(P1), 22, "lifelink: the Nighthawk dealt it");
    // Deathtouch: it's still the Nighthawk's damage.
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn gideons_sacrifice_needs_its_permanent_and_one_to_choose() {
    cr!("614.9", "609.3");
    ruling!(
        "Gideon's Sacrifice",
        "If the chosen creature or planeswalker isn't on the battlefield or isn't a creature or planeswalker at the time damage would be dealt, the damage won't be redirected."
    );
    ruling!(
        "Gideon's Sacrifice",
        "If you control no creatures or planeswalkers as Gideon's Sacrifice resolves, nothing happens. The damage redirection effect won't be able to be applied to any damage events."
    );
    // The chosen Bears died: P0 takes the damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    gideons_sacrifice(&mut t, Some(bears));
    destroy(&mut t, bears);
    cast_card(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // The chosen Bears is still there but no longer a creature (Imprisoned in the Moon
    // makes it a land): P0 takes the damage, the land none.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    gideons_sacrifice(&mut t, Some(bears));
    cast_card(&mut t, P0, "Imprisoned in the Moon", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    let land = mtg_engine::types::CardType::Land;
    assert!(!t.obj_now(bears).is_creature() && t.obj_now(bears).is(land));
    cast_card(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.obj_now(bears).damage, 0);
    // No creature or planeswalker: nothing happens; a Bears arriving later doesn't help.
    let mut t = TestGame::new(2);
    gideons_sacrifice(&mut t, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn two_gideons_sacrifices_send_each_damage_event_to_one_chosen_permanent() {
    cr!("616.1", "616.1e", "614.9");
    ruling!(
        "Gideon's Sacrifice",
        "If you cast more than one Gideon's Sacrifice in one turn, all damage that would be dealt at once to you and/or permanents you control is dealt to one of the chosen permanents of your choice. It's not dealt to all of them, and you can't split the damage between them. The next time damage would be dealt to you, you may choose a different one of those permanents to be dealt that damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Craw Wurm");
    gideons_sacrifice(&mut t, Some(a));
    gideons_sacrifice(&mut t, Some(b));
    let mut hits = vec![];
    for choice in [0usize, 1] {
        let from = t.g.turn_events.len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(choice));
        cast_card(&mut t, P1, "Shock", &[pl(P0)]);
        t.resolve_all();
        let d = damage_since(&t, from);
        assert_eq!(d.len(), 1, "one damage event, not split: {d:?}");
        assert_eq!(d[0].2, 2);
        assert!(d[0].1 == obj(a) || d[0].1 == obj(b));
        hits.push(d[0].1);
    }
    assert_eq!(t.life(P0), 20);
    assert_ne!(hits[0], hits[1], "a different one chosen the next time");
}

#[test]
fn doubling_season_copies_tapped_and_attacking() {
    cr!("614.1a", "111.1", "508.4");
    ruling!(
        "Doubling Season",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens created by Doubling Season's replacement effect. For example, if an effect tells you to create a token \"tapped and attacking,\" the additional tokens will also be tapped and attacking."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    attack_with(&mut t, &[(hero, pl(P1))]);
    t.resolve_all();
    let soldiers = tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 4);
    for s in soldiers {
        assert!(t.obj_now(s).tapped);
        assert_eq!(crate::r_p015_common::attacking(&t, s), Some(pl(P1)));
    }
}
