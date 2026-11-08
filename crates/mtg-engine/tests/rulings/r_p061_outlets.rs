//! Rulings batch P061 — free sacrifice and discard outlets: regeneration shields (CR
//! 701.19, 614.8, 704.5f–h), sacrificing a permanent to its own ability's cost (CR
//! 602.2b, 118.3), "can't be countered" (CR 101.2), activation restrictions on a
//! permanent's activated abilities (CR 602.1), training (CR 702.149a), scavenge's cost
//! (CR 702.97a), damage that stays marked (CR 120.6), "assigns combat damage equal to its
//! toughness" (CR 510.1a), and a formidable-style intervening "if" clause (CR 603.4).

use crate::r_p061_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::damage;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::*;

/// A token `p` creates, not summoning sick.
fn ready_token(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = create_token(t, p, name);
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

/// Gives `id` a regeneration shield with its own "Sacrifice ...: Regenerate" ability,
/// sacrificing `fodder`.
fn shield(t: &mut TestGame, id: ObjectId, fodder: ObjectId) {
    let i = ability(t, id, "Sacrifice");
    t.answer_choose(P0, &[o(fodder)]);
    t.activate(P0, id, i, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(fodder), "sacrificed");
}

#[test]
fn a_regeneration_shield_saves_from_destruction_only() {
    cr!("701.19a", "614.8", "704.5f", "704.5g", "704.5h", "702.2b");
    ruling!(
        "Vampire Warlord",
        "Activating Vampire Warlord’s ability causes a “regeneration shield” to be created for it. The next time that creature would be destroyed that turn, the regeneration shield is used up instead. This works only if the creature is dealt lethal damage, dealt damage from a source with deathtouch, or affected by a spell or ability that says to “destroy” it. Other effects that cause the creature to be put into the graveyard (such as reducing its toughness to 0 or sacrificing it) don’t destroy it, so regeneration won’t save it. If the regeneration shield isn’t used, it goes away as the turn ends."
    );
    // Vampire Warlord (4/2): "Sacrifice another creature: Regenerate this creature."
    #[derive(Debug, Clone, Copy)]
    enum Kill {
        Lethal,
        Deathtouch,
        Destroy,
        Toughness0,
        Sacrifice,
        NextTurn,
    }
    for (kill, survives) in [
        (Kill::Lethal, true),
        (Kill::Deathtouch, true),
        (Kill::Destroy, true),
        (Kill::Toughness0, false),
        (Kill::Sacrifice, false),
        (Kill::NextTurn, false),
    ] {
        let mut t = TestGame::new(2);
        let warlord = t.battlefield(P0, "Vampire Warlord");
        let bears = t.battlefield(P0, "Grizzly Bears");
        shield(&mut t, warlord, bears);
        match kill {
            Kill::Lethal => {
                let giant = t.battlefield(P1, "Hill Giant");
                damage(&mut t, giant, 2, warlord);
            }
            Kill::Deathtouch => {
                let rats = t.battlefield(P1, "Typhoid Rats");
                damage(&mut t, rats, 1, warlord);
            }
            Kill::Destroy => {
                cast_resolve(&mut t, P1, "Murder", &[o(warlord)]);
            }
            Kill::Toughness0 => {
                cast_resolve(&mut t, P1, "Last Gasp", &[o(warlord)]);
            }
            Kill::Sacrifice => {
                // P0's Fallen Angel: "Sacrifice a creature: ..."
                let angel = t.battlefield(P0, "Fallen Angel");
                let i = ability(&mut t, angel, "Sacrifice");
                t.answer_choose(P0, &[o(warlord)]);
                t.activate(P0, angel, i, &[]).unwrap();
                t.resolve_all();
            }
            Kill::NextTurn => {
                t.advance_to(P1, Step::PrecombatMain);
                destroy(&mut t, warlord);
            }
        }
        t.settle();
        assert_eq!(t.on_battlefield(warlord), survives, "{kill:?}");
        if survives {
            assert!(t.obj_now(warlord).tapped, "{kill:?}: regenerated");
            assert_eq!(t.obj_now(warlord).damage, 0);
        }
    }
}

#[test]
fn one_shield_covers_lethal_and_deathtouch_damage_at_once() {
    cr!("701.19a", "704.5g", "704.5h", "704.3");
    ruling!(
        "Vampire Warlord",
        "If Vampire Warlord is dealt lethal damage and is dealt damage by a source with deathtouch during the same combat damage step, a single regeneration shield will save it."
    );
    let mut t = TestGame::new(2);
    let warlord = t.battlefield(P0, "Vampire Warlord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rats = t.battlefield(P1, "Typhoid Rats");
    let giant = t.battlefield(P1, "Hill Giant");
    shield(&mut t, warlord, bears);
    declare_from(&mut t, P0, &[(warlord, pl(P1))]);
    block(&mut t, P1, &[(rats, warlord), (giant, warlord)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(warlord), "{}", t.dump_log());
    assert!(t.obj_now(warlord).tapped);
    assert_eq!(t.obj_now(warlord).damage, 0);
}

#[test]
fn goblin_turncoats_shield_works_against_destroy_and_lethal_damage() {
    cr!("701.19a", "614.8", "704.5g");
    ruling!(
        "Goblin Turncoat",
        "Activating the ability creates a replacement effect that acts like a shield, replacing the next time Goblin Turncoat would be destroyed that turn. This shield works against effects that try to destroy Goblin Turncoat or lethal damage that would be dealt to Goblin Turncoat."
    );
    for by_damage in [false, true] {
        let mut t = TestGame::new(2);
        let turncoat = t.battlefield(P0, "Goblin Turncoat");
        let piker = t.battlefield(P0, "Goblin Piker");
        let piker2 = t.battlefield(P0, "Goblin Piker");
        shield(&mut t, turncoat, piker);
        if by_damage {
            cast_resolve(&mut t, P1, "Lightning Bolt", &[o(turncoat)]);
        } else {
            cast_resolve(&mut t, P1, "Murder", &[o(turncoat)]);
        }
        assert!(t.on_battlefield(turncoat), "{by_damage}");
        assert!(t.obj_now(turncoat).tapped);
        // The shield was used up: the next destruction this turn kills it.
        destroy(&mut t, turncoat);
        assert!(!t.on_battlefield(turncoat), "{by_damage}: used up");
        let _ = piker2;
    }
}

#[test]
fn outlets_can_sacrifice_their_own_source() {
    cr!("602.2b", "118.3", "608.2b");
    ruling!("Fallen Angel", "Can sacrifice itself.");
    ruling!("Goblin Chirurgeon", "Can sacrifice itself.");
    // Fallen Angel: "Sacrifice a creature: This creature gets +2/+1 until end of turn."
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Fallen Angel");
    let i = ability(&mut t, angel, "Sacrifice");
    t.answer_choose(P0, &[o(angel)]);
    assert!(t.activate(P0, angel, i, &[]).is_ok());
    assert!(t.in_graveyard(P0, "Fallen Angel"));
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    // Goblin Chirurgeon: "Sacrifice a Goblin: Regenerate target creature." — sacrificing
    // itself to regenerate the Bears, which then survive Murder.
    let mut t = TestGame::new(2);
    let chirurgeon = t.battlefield(P0, "Goblin Chirurgeon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let i = ability(&mut t, chirurgeon, "Sacrifice");
    t.answer_choose(P0, &[o(chirurgeon)]);
    t.activate(P0, chirurgeon, i, &[o(bears)]).unwrap();
    assert!(t.in_graveyard(P0, "Goblin Chirurgeon"));
    t.resolve_all();
    cast_resolve(&mut t, P1, "Murder", &[o(bears)]);
    assert!(t.on_battlefield(bears));

    ruling!(
        "Extruder",
        "Extruder can be the target of its own activated ability, and you can sacrifice Extruder to pay for that activation. This won't accomplish much, but it does allow you to sacrifice Extruder if you want to."
    );
    // Extruder: "Sacrifice an artifact: Put a +1/+1 counter on target creature."
    let mut t = TestGame::new(2);
    let extruder = t.battlefield(P0, "Extruder");
    let i = ability(&mut t, extruder, "Sacrifice");
    t.answer_choose(P0, &[o(extruder)]);
    assert!(t.activate(P0, extruder, i, &[o(extruder)]).is_ok());
    assert!(t.in_graveyard(P0, "Extruder"));
    t.resolve_all();
    assert!(t.g.stack.is_empty(), "the ability did nothing");
}

#[test]
fn goblin_trashmaster_leaving_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.1g");
    ruling!(
        "Goblin Trashmaster",
        "Because damage remains marked on a creature until it’s removed as the turn ends, nonlethal damage dealt to a Goblin you control may become lethal if Goblin Trashmaster leaves the battlefield during that turn."
    );
    let mut t = TestGame::new(2);
    let trashmaster = t.battlefield(P0, "Goblin Trashmaster");
    let piker = t.battlefield(P0, "Goblin Piker");
    let relic = t.battlefield(P1, "Ornithopter");
    assert_eq!(t.pt(piker), (3, 2));
    let rats = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, rats, 1, piker);
    assert!(t.on_battlefield(piker));
    // Trashmaster sacrifices itself to its own ability ("Sacrifice a Goblin: Destroy
    // target artifact.").
    let i = ability(&mut t, trashmaster, "Sacrifice");
    t.answer_choose(P0, &[o(trashmaster)]);
    t.activate(P0, trashmaster, i, &[o(relic)]).unwrap();
    t.settle();
    assert!(!t.on_battlefield(piker), "1 damage on a 2/1");
}

#[test]
fn bill_the_ponys_effect_is_only_about_combat_damage() {
    cr!("510.1a", "701.14a");
    ruling!(
        "Bill the Pony",
        "Bill the Pony's last ability doesn't actually change any creature's power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, since having two creatures fight doesn't result in combat damage, an effect that causes the affected creature to fight another creature will still use its power to determine how much damage is dealt."
    );
    let mut t = TestGame::new(2);
    let bill = t.enter(P0, "Bill the Pony");
    t.resolve_all();
    t.g.objects[bill.0 as usize].summoning_sick = false;
    let i = ability(&mut t, bill, "Sacrifice");
    t.activate(P0, bill, i, &[o(bill)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bill), (1, 4), "its power isn't changed");
    // Bill (1/4) fights Grizzly Bears: it deals 1 damage.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Prey Upon", &[o(bill), o(bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 1);
    // In combat it assigns 4.
    let ogre = t.battlefield(P1, "Hill Giant");
    t.g.untap(bill);
    declare_from(&mut t, P0, &[(bill, pl(P1))]);
    block(&mut t, P1, &[(ogre, bill)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(ogre), "4 combat damage to a 3/3");
}

#[test]
fn grimgrin_gets_its_counter_even_if_the_creature_isnt_destroyed() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Grimgrin, Corpse-Born",
        "If Grimgrin's last ability resolves, but the targeted creature isn't destroyed (perhaps because it regenerated or has indestructible), you'll still put a +1/+1 on Grimgrin."
    );
    let mut t = TestGame::new(2);
    let grimgrin = t.battlefield(P0, "Grimgrin, Corpse-Born");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[o(myr)]);
    declare_from(&mut t, P0, &[(grimgrin, pl(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.counters(grimgrin, PLUS1), 1);
}

#[test]
fn yahenni_dying_with_the_creature_isnt_saved() {
    cr!("603.10a", "704.3");
    ruling!(
        "Yahenni, Undying Partisan",
        "If Yahenni and a creature an opponent controls die simultaneously (perhaps because they fought or were in combat together), Yahenni won't be on the battlefield as its triggered ability resolves. It can't be saved by the +1/+1 counter that would have been put on it."
    );
    let mut t = TestGame::new(2);
    let yahenni = t.battlefield(P0, "Yahenni, Undying Partisan");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Prey Upon", &[o(yahenni), o(bears)]);
    assert!(t.in_graveyard(P0, "Yahenni, Undying Partisan"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = yahenni;
}

#[test]
fn training_checks_power_as_attackers_are_declared() {
    cr!("702.149a", "508.1");
    ruling!(
        "Elder Arthur Maxson",
        "A creature’s training ability triggers only when both that creature and a creature with greater power are declared as attackers. Increasing a creature’s power after attackers are declared won’t cause a training ability to trigger."
    );
    // Elder Arthur Maxson: "Creature tokens you control have training."
    // A 1/1 token attacks with Grizzly Bears: it trains.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elder Arthur Maxson");
    let token = ready_token(&mut t, P0, "Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare_from(&mut t, P0, &[(token, pl(P1)), (bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(token, PLUS1), 1);
    // Two 1/1 tokens attack together; one gets +3/+3 afterward: neither trains.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elder Arthur Maxson");
    let a = ready_token(&mut t, P0, "Soldier");
    let b = ready_token(&mut t, P0, "Soldier");
    declare_from(&mut t, P0, &[(a, pl(P1)), (b, pl(P1))]);
    t.settle();
    assert!(t.g.stack.is_empty());
    cast_resolve(&mut t, P0, "Giant Growth", &[o(b)]);
    t.resolve_all();
    assert_eq!((t.counters(a, PLUS1), t.counters(b, PLUS1)), (0, 0));
}

#[test]
fn koma_isnt_countered_but_the_rest_of_the_counterspell_happens() {
    cr!("101.2", "608.2c");
    ruling!(
        "Koma, Cosmos Serpent",
        "A spell or ability that counters spells can still target Koma, Cosmos Serpent. When that spell or ability resolves, Koma won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    let mut t = TestGame::new(2);
    let koma = cast(&mut t, P0, "Koma, Cosmos Serpent", &[]);
    let from = t.asked().len();
    // Dissolve: "Counter target spell. Scry 1."
    cast(&mut t, P1, "Dissolve", &[o(koma)]);
    t.resolve();
    let scried = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Scry { .. }));
    assert!(scried, "P1 scries");
    assert!(t.g.stack.contains(&koma), "Koma is still on the stack");
    t.resolve_all();
    assert!(!t.named_on_battlefield("Koma, Cosmos Serpent").is_empty());
}

#[test]
fn komas_tap_mode_stops_every_activated_ability() {
    cr!("602.1", "606.2", "702.6a");
    ruling!(
        "Koma, Cosmos Serpent",
        "Activated abilities contain a colon and appear in the form \"[Cost]: [Effect].\" Some keywords such as equip are activated abilities and will have colons in their reminder texts. Loyalty abilities of planeswalkers are also activated abilities."
    );
    // Koma's "Sacrifice another Serpent: ... Tap target permanent. Its activated abilities
    // can't be activated this turn." on P1's Bonesplitter (equip) and Garruk Wildspeaker
    // (loyalty abilities), on P1's turn.
    for (walker, koma_first) in [(false, true), (true, true), (false, false), (true, false)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let koma = t.battlefield(P0, "Koma, Cosmos Serpent");
        let serpent = t.battlefield(P0, "Serpent of Yawning Depths");
        let target = if walker {
            t.battlefield(P1, "Garruk Wildspeaker")
        } else {
            t.battlefield(P1, "Bonesplitter")
        };
        t.battlefield(P1, "Grizzly Bears");
        t.lands(P1, "Plains", 2);
        if !koma_first {
            // Without Koma's ability, P1 can activate it.
            assert!(t.activate(P1, target, 0, &[]).is_ok(), "walker {walker}");
            continue;
        }
        let i = ability(&mut t, koma, "Sacrifice");
        t.answer_choose(P0, &[o(serpent)]);
        t.answer(
            P0,
            DecisionKind::Modes,
            mtg_engine::decision::Answer::Indices(vec![0]),
        );
        t.activate(P0, koma, i, &[o(target)]).unwrap();
        t.resolve_all();
        assert!(t.obj_now(target).tapped);
        let r = t.activate(P1, target, 0, &[]);
        assert!(r.is_err(), "walker {walker}: {r:?}");
    }
}

#[test]
fn scavenge_costs_arent_reduced_by_spell_cost_effects() {
    cr!("702.97a", "601.2f", "118.9");
    ruling!(
        "Varolz, the Scar-Striped",
        "Abilities that reduce the cost to cast a creature spell or alternative costs you can pay rather than a card's mana cost don't apply to activating a card's scavenge ability."
    );
    // Varolz gives Grizzly Bears in P0's graveyard scavenge {1}{G}. Heartless Summoning
    // ("Creature spells you cast cost {2} less to cast.") or Omniscience ("You may cast
    // spells from your hand without paying their mana costs.") doesn't help: one Forest
    // isn't enough.
    for helper in ["Heartless Summoning", "Omniscience"] {
        for lands in [1, 2] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, "Varolz, the Scar-Striped");
            t.battlefield(P0, helper);
            let bears = t.graveyard(P0, "Grizzly Bears");
            let target = t.battlefield(P0, "Hill Giant");
            t.lands(P0, "Forest", lands);
            let r = t.activate(P0, bears, 0, &[o(target)]);
            assert_eq!(r.is_ok(), lands == 2, "{helper}, {lands} lands: {r:?}");
        }
    }
}

#[test]
fn draw_then_discard_leaves_no_window_to_act() {
    cr!("608.2", "117.1");
    ruling!(
        "Seismic Assault",
        "If an effect instructs you to draw cards and then discard cards, you can't take actions in between. For example, if you have no cards in hand and cast Faithless Looting, you can't activate Seismic Assault's ability between the time you draw two cards and discard those two cards."
    );
    ruling!(
        "Ayula's Influence",
        "If an effect instructs you to draw cards and then discard cards, you can’t take actions in between. For example, if you have no cards in hand and cast Bazaar Trademage, you can’t activate Ayula’s Influence’s ability between the time you draw two cards and discard those cards."
    );
    for (outlet, spell) in [
        ("Seismic Assault", "Faithless Looting"),
        ("Ayula's Influence", "Bazaar Trademage"),
    ] {
        supported(outlet);
        let mut t = TestGame::new(2);
        t.battlefield(P0, outlet);
        stack_library(&mut t, P0, &["Mountain", "Forest"]);
        assert_eq!(t.hand_size(P0), 0);
        // P0's hand each time P0 gets priority, everything passing through the priority
        // loop (`resolve_all` gives no one priority): with a drawn land in hand, P0 could
        // activate the outlet.
        let hands = crate::r_s01_common::watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::Priority { .. }),
            |g| g.player(PlayerId(0)).hand.len(),
        );
        cast(&mut t, P0, spell, &[]);
        t.advance_to(P0, Step::BeginningOfCombat);
        let hands = hands.lock().unwrap().clone();
        assert!(hands.len() >= 2, "{spell}: priority before and after: {hands:?}");
        assert!(hands.iter().all(|n| *n == 0), "{spell}: {hands:?}");
        assert_eq!(t.hand_size(P0), 0, "{spell}");
        assert!(t.in_graveyard(P0, "Mountain") && t.in_graveyard(P0, "Forest"));
        // The outlet's ability was never activated.
        assert!(t.named_on_battlefield("Bear").is_empty());
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn owlbear_shepherd_checks_total_power_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Owlbear Shepherd",
        "Owlbear Shepherd's ability has an intervening if clause, which means that it checks to see if creatures you control have total power 8 or greater both as it would trigger and as it resolves. If enough creatures have been removed or their total power is reduced to below 8 by the time the ability tries to resolve, you will not draw a card."
    );
    // Owlbear Shepherd (1) + Craw Wurm (6) + Grizzly Bears (2) = 9; without the Bears, 7.
    for (bears, remove, draws) in [(true, false, 1), (true, true, 0), (false, false, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Owlbear Shepherd");
        t.battlefield(P0, "Craw Wurm");
        let b = bears.then(|| t.battlefield(P0, "Grizzly Bears"));
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.g.stack.len(), usize::from(bears), "triggered");
        if remove {
            destroy(&mut t, b.unwrap());
        }
        let before = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0) - before, draws, "{bears} {remove}");
    }
}
