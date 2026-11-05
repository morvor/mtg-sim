//! Rulings batch P110 — destroy-an-artifact spells with a rider (CR 608.2b, 608.2c), "this
//! creature enters with a -1/-1 counter" and the counter annihilation SBA (CR 704.5q),
//! "can't be blocked by creatures with power N or greater / or less" (CR 509.1b, 506.4),
//! and power-5 ("ferocious") abilities: targets checked twice (CR 115.1, 608.2b),
//! untargeted sets fixed on resolution (CR 611.2c), last known information for dies
//! triggers (CR 603.10a), and Sacellum Godspeaker's mana ability (CR 605).

use crate::r_p110_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s02_common::target_candidates;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Containment Breach and Decommission -------------------------------------------------

#[test]
fn containment_breach_makes_a_pest_even_if_the_artifact_survives() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Containment Breach",
        "If Containment Breach resolves, and the target artifact has a mana value of 2 or less, but it wasn't destroyed (perhaps because it has indestructible), you will still create a Pest."
    );
    supported("Containment Breach");
    // Darksteel Relic: an indestructible artifact with mana value 0.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    cast_new(&mut t, P0, "Containment Breach", &[obj(relic)]);
    t.resolve_all();
    assert!(t.on_battlefield(relic));
    assert_eq!(tokens(&t, P0).len(), 1);
    // Mana value 3 (Darksteel Myr): no Pest.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_new(&mut t, P0, "Containment Breach", &[obj(myr)]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn a_destroy_spell_with_an_illegal_target_does_nothing_else() {
    cr!("608.2b");
    ruling!(
        "Containment Breach",
        "If the artifact or enchantment is an illegal target as Containment Breach tries to resolve, it won't resolve and none of its effects will happen. You won't create a Pest."
    );
    ruling!(
        "Decommission",
        "If the target artifact or enchantment becomes an illegal target, Decommission doesn't resolve and none of its effects happen. You won't gain life."
    );
    supported("Decommission");
    // Containment Breach on a Memnite (mana value 0) that's bounced in response.
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P1, "Memnite");
    let spell = cast_new(&mut t, P0, "Containment Breach", &[obj(memnite)]);
    t.lands(P1, "Island", 1);
    cast_new(&mut t, P1, "Unsummon", &[obj(memnite)]);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(tokens(&t, P0).is_empty());
    // Decommission with revolt (a permanent of P0's left the battlefield this turn).
    let decommission = |bounce: bool| {
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, b);
        let relic = t.battlefield(P1, "Ornithopter");
        let spell = cast_new(&mut t, P0, "Decommission", &[obj(relic)]);
        if bounce {
            t.lands(P1, "Island", 1);
            cast_new(&mut t, P1, "Unsummon", &[obj(relic)]);
        }
        t.resolve_all();
        (resolved(&t, spell), t.life(P0))
    };
    assert_eq!(decommission(true), (false, 20));
    assert_eq!(decommission(false), (true, 23));
}

// --- Wickerbough Elder -------------------------------------------------------------------

#[test]
fn wickerbough_elder_with_a_plus_one_counter_loses_both_counters() {
    cr!("704.5q", "614.1c", "118.3");
    ruling!(
        "Wickerbough Elder",
        "If an effect puts Wickerbough Elder onto the battlefield with one or more +1/+1 counters, its -1/-1 counter and one of those +1/+1 counters are removed as a state-based action before you can activate its ability."
    );
    supported("Wickerbough Elder");
    supported("Grumgully, the Generous");
    // Grumgully: "Each other non-Human creature you control enters with an additional
    // +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    t.battlefield(P1, "Ornithopter");
    let elder = t.enter(P0, "Wickerbough Elder");
    assert_eq!(t.counters(elder, counters::MINUS1), 1);
    assert_eq!(t.counters(elder, counters::PLUS1), 1);
    t.settle();
    assert_eq!(t.counters(elder, counters::MINUS1), 0);
    assert_eq!(t.counters(elder, counters::PLUS1), 0);
    assert_eq!(t.pt(elder), (4, 4));
    t.lands(P0, "Forest", 1);
    assert!(!can_activate(&mut t, P0, elder));
}

#[test]
fn wickerbough_elder_needs_a_target() {
    cr!("602.2b", "601.2c", "115.1");
    ruling!(
        "Wickerbough Elder",
        "You must choose a target artifact or enchantment to activate Wickerbough Elder's last ability. You can't pay {G} just to remove a -1/-1 counter from it without a legal target."
    );
    let mut t = TestGame::new(2);
    let elder = t.enter(P0, "Wickerbough Elder");
    t.settle();
    assert_eq!(t.pt(elder), (3, 3));
    t.lands(P0, "Forest", 1);
    assert!(
        !can_activate(&mut t, P0, elder),
        "no artifact or enchantment"
    );
    assert!(act(&mut t, P0, elder, "Destroy", &[]).is_err());
    assert_eq!(t.counters(elder, counters::MINUS1), 1);
    // With an Ornithopter around, it can.
    let thopter = t.battlefield(P1, "Ornithopter");
    assert!(can_activate(&mut t, P0, elder));
    act(&mut t, P0, elder, "Destroy", &[obj(thopter)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.pt(elder), (4, 4));
}

// --- Blocking restrictions by power ------------------------------------------------------

/// P0 attacks P1 with `attacker`; P1 blocks with `blocker`; in the declare blockers step
/// `during` acts; combat finishes. Returns whether `attacker` was still blocked after
/// `during`, and P1's life after combat.
fn block_then(
    attacker: &str,
    blocker: &str,
    during: impl FnOnce(&mut TestGame, ObjectId),
) -> (bool, i32) {
    supported(attacker);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, attacker);
    let b = t.battlefield(P1, blocker);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(b, a)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(
        t.g.combat.as_ref().unwrap().is_blocked(a),
        "{blocker} blocked"
    );
    during(&mut t, b);
    let blocked = t.g.combat.as_ref().unwrap().is_blocked(a);
    t.advance_to(P0, Step::EndOfCombat);
    (blocked, t.life(P1))
}

#[test]
fn changing_a_blockers_power_doesnt_unblock() {
    cr!("509.1h", "509.1b");
    ruling!(
        "Stormkeld Vanguard // Bear Down",
        "Once Stormkeld Vanguard has become blocked, reducing the power of a creature blocking it to 2 or less won't cause it to become unblocked."
    );
    ruling!(
        "Amrou Kithkin",
        "If the power of a creature that’s already blocking Amrou Kithkin is increased to 3 or greater, it will continue to block Amrou Kithkin."
    );
    ruling!(
        "Beloved Princess",
        "Once a creature with power 2 or less has blocked this creature, changing the power of the blocking creature won't cause this creature to become unblocked."
    );
    // Stormkeld Vanguard (6/7) blocked by a Hill Giant (3/3) that gets -2/-2.
    let shrink = |t: &mut TestGame, b: ObjectId| {
        t.lands(P1, "Swamp", 1);
        cast_new(t, P1, "Disfigure", &[obj(b)]);
        t.resolve_all();
        assert_eq!(t.pt(b).0, 1);
    };
    assert_eq!(
        block_then("Stormkeld Vanguard // Bear Down", "Hill Giant", shrink),
        (true, 20)
    );
    // Amrou Kithkin and Beloved Princess blocked by a Grizzly Bears that gets +3/+3.
    let grow = |t: &mut TestGame, b: ObjectId| {
        t.lands(P1, "Forest", 1);
        cast_new(t, P1, "Giant Growth", &[obj(b)]);
        t.resolve_all();
        assert_eq!(t.pt(b).0, 5);
    };
    assert_eq!(
        block_then("Amrou Kithkin", "Grizzly Bears", grow),
        (true, 20)
    );
    assert_eq!(
        block_then("Beloved Princess", "Grizzly Bears", grow),
        (true, 20)
    );
}

#[test]
fn amrou_kithkin_cant_be_blocked_by_power_three_or_greater() {
    cr!("509.1b");
    ruling!(
        "Amrou Kithkin",
        "A creature with power 3 or greater can’t be declared as a blocker for Amrou Kithkin."
    );
    let mut t = TestGame::new(2);
    let kithkin = t.battlefield(P0, "Amrou Kithkin");
    let giant = t.battlefield(P1, "Hill Giant");
    let bear = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(kithkin, Entity::Player(P1))]);
    t.g.recompute();
    assert!(!t.g.can_block(giant, kithkin));
    let opts = mtg_engine::combat::block_options(&t.g, &[P1]);
    let can = |id: ObjectId| opts.iter().any(|(b, a)| *b == id && a.contains(&kithkin));
    assert!(!can(giant));
    assert!(can(bear));
}

// --- Mycoid Shepherd ---------------------------------------------------------------------

/// The number of Mycoid Shepherd triggers waiting or on the stack.
fn shepherd_triggers(t: &TestGame) -> usize {
    t.g.stack.len() + t.g.pending_triggers.len()
}

#[test]
fn mycoid_shepherd_triggers_for_itself_and_a_big_creature_dying_together() {
    cr!("603.10a", "603.2");
    ruling!(
        "Mycoid Shepherd",
        "If Mycoid Shepherd and another creature you control with power 5 or greater are put into a graveyard from the battlefield at the same time, Mycoid Shepherd’s ability will trigger twice."
    );
    supported("Mycoid Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycoid Shepherd");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(shepherd_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 30);
}

#[test]
fn mycoid_shepherd_uses_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Mycoid Shepherd",
        "If another creature you control is put into a graveyard from the battlefield, Mycoid Shepherd’s ability checks that creature’s power as it last existed on the battlefield. If it was 5 or greater, the ability will trigger."
    );
    // A Grizzly Bears with +3/+3 from Giant Growth dies as a 5/5: it triggers. An
    // unpumped one doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycoid Shepherd");
    let pumped = t.battlefield(P0, "Grizzly Bears");
    let plain = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(pumped)]);
    t.resolve_all();
    assert_eq!(t.pt(pumped), (5, 5));
    destroy(&mut t, plain);
    assert_eq!(shepherd_triggers(&t), 0);
    t.answer_yes(P0, true);
    destroy(&mut t, pumped);
    assert_eq!(shepherd_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn mycoid_shepherd_triggers_on_its_own_death_whatever_its_power() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Mycoid Shepherd",
        "Mycoid Shepherd’s ability will trigger when it’s put into a graveyard from the battlefield no matter what its power is at that time."
    );
    // Disfigured to 3/2, then destroyed.
    let mut t = TestGame::new(2);
    let shepherd = t.battlefield(P0, "Mycoid Shepherd");
    t.lands(P1, "Swamp", 1);
    cast_new(&mut t, P1, "Disfigure", &[obj(shepherd)]);
    t.resolve_all();
    assert_eq!(t.pt(shepherd), (3, 2));
    t.answer_yes(P0, true);
    destroy(&mut t, shepherd);
    assert_eq!(shepherd_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

// --- Abilities targeting a creature with power 5 or greater ------------------------------

/// The ferocious targeted abilities: card, the ability's text, its mana, and whether the
/// creature has its effect.
type Granted = fn(&mut TestGame, ObjectId) -> bool;

fn ferocious_cases() -> Vec<(
    &'static str,
    &'static str,
    &'static [(ManaType, u32)],
    Granted,
)> {
    fn kw(k: KeywordKind) -> impl Fn(&mut TestGame, ObjectId) -> bool {
        move |t, id| {
            t.g.recompute();
            has_kw(t, id, k)
        }
    }
    vec![
        ("Bloodthorn Taunter", "haste", &[], |t, id| {
            kw(KeywordKind::Haste)(t, id)
        }),
        ("Mosstodon", "trample", &[(ManaType::C, 1)], |t, id| {
            kw(KeywordKind::Trample)(t, id)
        }),
        (
            "Rakeclaw Gargantuan",
            "first strike",
            &[(ManaType::C, 1)],
            |t, id| kw(KeywordKind::FirstStrike)(t, id),
        ),
        (
            "Godtoucher",
            "Prevent",
            &[(ManaType::C, 1), (ManaType::W, 1)],
            |t, id| {
                // 1 damage to the creature is prevented.
                let src = t.battlefield(P1, "Grizzly Bears");
                let before = t.obj_now(id).damage;
                damage(t, src, 1, id);
                t.on_battlefield(id) && t.obj_now(id).damage == before
            },
        ),
    ]
}

/// P0's `card` (untapped, with `costs` in the pool) and P1's Craw Wurm (6/4). P0 activates
/// the ability targeting the Wurm; `respond` acts before it resolves.
fn ferocious(
    card: &str,
    needle: &str,
    costs: &[(ManaType, u32)],
    respond: impl FnOnce(&mut TestGame, ObjectId),
) -> (TestGame, ObjectId, ObjectId) {
    supported(card);
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, card);
    let wurm = t.battlefield(P1, "Craw Wurm");
    for (ty, n) in costs {
        mana(&mut t, P0, *ty, *n);
    }
    let ab = act(&mut t, P0, src, needle, &[obj(wurm)])
        .unwrap()
        .expect("on the stack");
    respond(&mut t, wurm);
    t.resolve_all();
    (t, wurm, ab)
}

#[test]
fn ferocious_targets_are_checked_on_targeting_and_resolution() {
    cr!("115.1", "608.2b", "611.2c");
    ruling!(
        "Bloodthorn Taunter",
        "The ability checks the targeted creature’s power twice: when the creature becomes the target, and when the ability resolves. Once the ability resolves, it will continue to apply to the affected creature no matter what its power may become later in the turn."
    );
    ruling!(
        "Godtoucher",
        "The ability checks the targeted creature’s power twice: when the creature becomes the target, and when the ability resolves. Once the ability resolves, it will continue to apply to the affected creature no matter what its power may become later in the turn."
    );
    ruling!(
        "Mosstodon",
        "The ability checks the targeted creature’s power twice: when the creature becomes the target, and when the ability resolves. Once the ability resolves, it will continue to apply to the affected creature no matter what its power may become later in the turn."
    );
    ruling!(
        "Rakeclaw Gargantuan",
        "The ability checks the targeted creature’s power twice: when the creature becomes the target, and when the ability resolves. Once the ability resolves, it will continue to apply to the affected creature no matter what its power may become later in the turn."
    );
    let shrink = |t: &mut TestGame, wurm: ObjectId| {
        t.lands(P1, "Swamp", 1);
        cast_new(t, P1, "Disfigure", &[obj(wurm)]);
        t.resolve();
    };
    for (card, needle, costs, has) in ferocious_cases() {
        // Only creatures with power 5 or greater are offered as targets.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let wurm = t.battlefield(P0, "Craw Wurm");
        t.battlefield(P0, "Grizzly Bears");
        for (ty, n) in costs {
            mana(&mut t, P0, *ty, *n);
        }
        let bear = t.named_on_battlefield("Grizzly Bears")[0];
        let from = t.asked().len();
        act(&mut t, P0, src, needle, &[obj(wurm)]).unwrap();
        let offered = target_candidates(&t, P0, from);
        assert!(offered[0].contains(&obj(wurm)), "{card}");
        assert!(!offered[0].contains(&obj(bear)), "{card}");
        // Shrunk below 5 power in response: it doesn't resolve.
        let (mut t, wurm, _) = ferocious(card, needle, costs, |t, w| shrink(t, w));
        assert_eq!(t.pt(wurm).0, 4, "{card}");
        assert!(!has(&mut t, wurm), "{card}: didn't resolve");
        // Resolved, then shrunk: the effect still applies.
        let (mut t, wurm, _) = ferocious(card, needle, costs, |_, _| {});
        assert!(has(&mut t, wurm), "{card}");
        shrink(&mut t, wurm);
        assert_eq!(t.pt(wurm).0, 4, "{card}");
        assert!(has(&mut t, wurm), "{card}: still applies");
    }
}

#[test]
fn gustrider_exuberant_checks_power_only_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Gustrider Exuberant",
        "The ability doesn’t target, so it checks the creatures’ powers only once: when the ability resolves. Once the ability resolves, it will continue to apply to the affected creatures no matter what their powers may become later in the turn."
    );
    supported("Gustrider Exuberant");
    let mut t = TestGame::new(2);
    let gust = t.battlefield(P0, "Gustrider Exuberant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, gust, "Sacrifice", &[]).unwrap();
    // In response, a Bears gets +3/+3: it's a 5/5 as the ability resolves.
    t.lands(P0, "Forest", 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(bear)]);
    t.resolve_all();
    t.g.recompute();
    assert!(has_kw(&t, wurm, KeywordKind::Flying));
    assert!(has_kw(&t, bear, KeywordKind::Flying));
    assert!(!has_kw(&t, other, KeywordKind::Flying));
    // The Wurm shrinks afterwards: it keeps flying; the other Bears grows: it doesn't
    // gain flying.
    t.lands(P1, "Swamp", 1);
    cast_new(&mut t, P1, "Disfigure", &[obj(wurm)]);
    t.resolve_all();
    t.lands(P0, "Forest", 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(other)]);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(wurm).0, 4);
    assert!(has_kw(&t, wurm, KeywordKind::Flying));
    assert_eq!(t.pt(other).0, 5);
    assert!(!has_kw(&t, other, KeywordKind::Flying));
}

// --- Sacellum Godspeaker -----------------------------------------------------------------

#[test]
fn sacellum_godspeaker_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Sacellum Godspeaker",
        "This is a mana ability. It doesn’t use the stack, and it can’t be responded to."
    );
    ruling!(
        "Sacellum Godspeaker",
        "You can activate this ability even if you have no creature cards with power 5 or greater in your hand, or you have some but choose not to reveal any of them. In those cases, the ability won’t produce any mana."
    );
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Sacellum Godspeaker");
    let wurm = t.hand(P0, "Craw Wurm");
    let maw = t.hand(P0, "Colossal Dreadmaw");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(wurm), obj(maw)]);
    assert!(tap_for_mana(&mut t, P0, gs, "Reveal"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_of(&t, P0, ManaType::G), 2);
    // With no big creature card in hand: it can be activated, for no mana.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Sacellum Godspeaker");
    t.hand(P0, "Grizzly Bears");
    assert!(!tap_for_mana(&mut t, P0, gs, "Reveal"));
    assert!(t.obj_now(gs).tapped, "activated");
    assert_eq!(pool_total(&t, P0), 0);
    // Choosing to reveal none of them.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Sacellum Godspeaker");
    t.hand(P0, "Craw Wurm");
    t.answer_choose(P0, &[]);
    assert!(!tap_for_mana(&mut t, P0, gs, "Reveal"));
    assert!(t.obj_now(gs).tapped, "activated");
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn sacellum_godspeaker_cant_reveal_the_spell_being_cast() {
    cr!("601.2a", "605.3a");
    ruling!(
        "Sacellum Godspeaker",
        "While you’re casting a creature spell, that card is on the stack, so you can’t reveal it with Sacellum Godspeaker’s ability to produce mana. You can, however, activate this mana ability and reveal that creature card from your hand before you begin to cast the card as a spell."
    );
    // Craw Wurm ({4}{G}{G}) with five Forests and the Godspeaker: revealed before casting,
    // the Wurm's {G} pays for it.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Sacellum Godspeaker");
    t.lands(P0, "Forest", 5);
    let wurm = t.hand(P0, "Craw Wurm");
    t.answer_choose(P0, &[obj(wurm)]);
    assert!(tap_for_mana(&mut t, P0, gs, "Reveal"));
    assert_eq!(pool_of(&t, P0, ManaType::G), 1);
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    // Once the Wurm is a spell on the stack, it isn't in hand to be revealed: the
    // Godspeaker offers nothing and adds nothing.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Sacellum Godspeaker");
    t.lands(P0, "Forest", 6);
    let wurm = t.hand(P0, "Craw Wurm");
    let spell = t.cast(P0, wurm).go();
    assert!(t.g.stack.contains(&spell));
    let from = t.asked().len();
    assert!(!tap_for_mana(&mut t, P0, gs, "Reveal"));
    assert!(t.obj_now(gs).tapped);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })));
    assert_eq!(pool_total(&t, P0), 0);
}
