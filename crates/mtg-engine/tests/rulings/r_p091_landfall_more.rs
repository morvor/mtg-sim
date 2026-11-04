//! Rulings batch P091 — more landfall cards: Cosima's voyage, Turntimber Basilisk and
//! Trench Behemoth's combat requirements, Nissa, Zimone, Null Profusion, Spitfire Lagac in
//! Two-Headed Giant, Surrakar Marauder, Pia Nalaar, Hormagaunt Horde, Seer's Sundial and
//! The Ozolith.

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s09_common::legal_attack;
use crate::r_s21_common::legal_blocks;
use mtg_engine::ability::Modification;
use mtg_engine::events::MoveCause;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let new = t.g.move_object(id, zone, MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
    new.unwrap_or(id)
}

const COSIMA: &str = "Cosima, God of the Voyage // The Omenkeel";

/// Cosima on the battlefield; P0's next upkeep trigger is on the stack.
fn cosima_upkeep() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, COSIMA);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, c), 1);
    (t, c)
}

/// Cosima exiled by its upkeep ability (with the gained ability).
fn cosima_exiled() -> (TestGame, ObjectId) {
    let (mut t, c) = cosima_upkeep();
    t.answer_yes(P0, true);
    t.resolve_all();
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    (t, c)
}

#[test]
fn cosima_upkeep_and_exile_rulings() {
    cr!("603.4", "608.2b", "113.6", "400.7");
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "Cosima's ability triggers only if Cosima is on the battlefield at the beginning of your upkeep, and you may exile Cosima only if it is still on the battlefield as the ability resolves."
    );
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "If a land enters the battlefield under your control while Cosima is exiled and has the added triggered ability, that ability will trigger. As the ability tries to resolve, it will check to see if Cosima is still in exile. If Cosima has left exile before the ability tries to resolve, it won't resolve and none of its effects will happen. This is true even if Cosima returns to exile before that ability tries to resolve."
    );
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "While the triggered ability Cosima gains is resolving, if you don't put a voyage counter on Cosima, you'll immediately return it to the battlefield and draw cards equal to the number of voyage counters Cosima had on it in exile."
    );
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "You may choose to return Cosima to the battlefield even if it doesn't have any voyage counters on it. If you do, it won't enter with any +1/+1 counters on it and you won't draw any cards."
    );
    // Not on the battlefield at the beginning of the upkeep: no trigger.
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, COSIMA);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    let _ = c;
    // Bounced in response: it stays in the hand.
    let (mut t, c) = cosima_upkeep();
    let c = move_to(&mut t, c, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Hand(P0));
    // Exiled; a land enters; it leaves exile and comes back before the ability resolves:
    // nothing happens.
    let (mut t, c) = cosima_exiled();
    put_onto_battlefield(&mut t, P0, "Forest");
    assert_eq!(t.stack_len(), 1);
    let gy = move_to(&mut t, c, Zone::Graveyard(P0));
    let back = move_to(&mut t, gy, Zone::Exile);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(back), Zone::Exile);
    assert_eq!(t.counters(back, "voyage"), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.named_on_battlefield("Cosima, God of the Voyage").is_empty());
    // Zero voyage counters: declining returns it with no counters and draws nothing.
    let (mut t, _c) = cosima_exiled();
    put_onto_battlefield(&mut t, P0, "Forest");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.resolve();
    let back = t.named_on_battlefield("Cosima, God of the Voyage");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "+1/+1"), 0);
    assert_eq!(t.hand_size(P0), hand);
    // One voyage counter, then decline: it returns at once with one counter and P0 draws
    // one, as part of the same resolution.
    let (mut t, c) = cosima_exiled();
    t.answer_yes(P0, true);
    put_onto_battlefield(&mut t, P0, "Forest");
    t.resolve_all();
    assert_eq!(t.counters(c, "voyage"), 1);
    put_onto_battlefield(&mut t, P0, "Forest");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    let back = t.named_on_battlefield("Cosima, God of the Voyage");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn turntimber_basilisk_block_requirement() {
    cr!("509.1c", "509.1b");
    ruling!(
        "Turntimber Basilisk",
        "Deciding to have the targeted creature block doesn't force you to attack with Turntimber Basilisk that turn. If Turntimber Basilisk doesn't attack, the targeted creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    ruling!(
        "Turntimber Basilisk",
        "If you choose to have the targeted creature block but that creature isn't able to block Turntimber Basilisk (for example, because the targeted creature can block only creatures with flying), the requirement to block does nothing."
    );
    supported("Turntimber Basilisk");
    supported("Cloud Sprite");
    let setup = |blocker: &str, attack_basilisk: bool| {
        let mut t = TestGame::new(2);
        let basilisk = t.battlefield(P0, "Turntimber Basilisk");
        let giant = t.battlefield(P0, "Hill Giant");
        let b = t.battlefield(P1, blocker);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(b)]);
        play_from_hand(&mut t, P0, "Forest");
        t.resolve_all();
        let attacker = if attack_basilisk { basilisk } else { giant };
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        (t, attacker, b)
    };
    // Positive control: the Bears must block an attacking Basilisk.
    let (mut t, basilisk, bears) = setup("Grizzly Bears", true);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, basilisk)]));
    // The Basilisk doesn't attack: free to block the Giant or not.
    let (mut t, giant, bears) = setup("Grizzly Bears", false);
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, giant)]));
    // Cloud Sprite can't block the Basilisk: no requirement.
    let (mut t, _basilisk, _sprite) = setup("Cloud Sprite", true);
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn trench_behemoth_attack_requirement_and_untap_ability() {
    cr!("508.1d", "506.4", "602.2");
    ruling!(
        "Trench Behemoth",
        "If the target creature can't attack for any reason (such as being tapped), then it doesn't attack."
    );
    ruling!(
        "Trench Behemoth",
        "If the target creature does attack, its controller still chooses which player or planeswalker it attacks, as normal. It can attack opponents who don't control untapped Trench Behemoths waiting to crush it, if any are available."
    );
    ruling!(
        "Trench Behemoth",
        "You can activate Trench Behemoth's first ability even if it's already untapped and/or already has hexproof."
    );
    supported("Trench Behemoth");
    // Three players: P1's Bears must attack in P1's next combat, P0 or P2.
    let mut t = TestGame::new(3);
    let behemoth = t.battlefield(P0, "Trench Behemoth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    play_from_hand(&mut t, P0, "Island");
    t.resolve_all();
    // The untap ability works on an untapped Behemoth, twice (hexproof already).
    for _ in 0..2 {
        let land = t.named_on_battlefield("Island")[0];
        crate::r_s06_common::activate_containing(&mut t, P0, behemoth, "Return")
            .unwrap_or_else(|e| panic!("{e:?}"));
        t.resolve_all();
        assert!(t.in_hand(P0, "Island") || t.zone(land) == Zone::Hand(P0));
        assert!(crate::r_s06_common::has_kw(
            &t,
            behemoth,
            mtg_engine::keywords::KeywordKind::Hexproof
        ));
        t.battlefield(P0, "Island");
    }
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trench Behemoth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    play_from_hand(&mut t, P0, "Island");
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.g.tap(bears);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn nissa_sage_animist_rulings() {
    cr!("302.6", "704.5j", "113.6");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "If a land becomes a creature but hasn't continuously been under its controller's control since that player's most recent turn began, it won't be able to attack and its abilities with {T} in the cost (including mana abilities) won't be able to be activated."
    );
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "You can activate Nissa, Sage Animist's second ability even if you already control an Ashaya, the Awoken World. Just after the second token is created, you'll choose one to remain on the battlefield."
    );
    supported("Nissa, Vastwood Seer // Nissa, Sage Animist");
    const NISSA: &str = "Nissa, Vastwood Seer // Nissa, Sage Animist";
    let mut t = TestGame::new(2);
    let nissa = crate::r_s17_common::enter_transformed(&mut t, P0, NISSA);
    t.resolve_all();
    t.g.objects[nissa.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    let old = t.battlefield(P0, "Forest");
    let new = t.battlefield_sick(P0, "Forest");
    t.g.tap(old);
    t.g.tap(new);
    t.answer_targets(P0, &[obj(old), obj(new)]);
    t.activate(P0, nissa, 2, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(old).tapped && !t.obj_now(new).tapped);
    assert_eq!(t.pt(new), (6, 6));
    assert!(!crate::r_s20_common::tap_for_mana(&mut t, P0, new, "{G}"));
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, old, "{G}"));
    t.g.untap(old);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, old));
    assert!(!crate::r_s02_common::can_attack(&mut t, new));
    // −2 with an Ashaya already there: one remains.
    let mut t = TestGame::new(2);
    let nissa = crate::r_s17_common::enter_transformed(&mut t, P0, NISSA);
    t.resolve_all();
    t.g.objects[nissa.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 5);
    t.activate(P0, nissa, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ashaya, the Awoken World").len(), 1);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let nissa = t.g.current(nissa);
    assert!(t.activate(P0, nissa, 1, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ashaya, the Awoken World").len(), 1);
    assert_eq!(t.counters(nissa, counters::LOYALTY), 1);
}

#[test]
fn zimone_needs_a_land_this_turn_and_any_prime() {
    cr!("603.4", "513.1a");
    ruling!(
        "Zimone, All-Questioning",
        "If a land hasn't entered the battlefield under your control during your turn by the time your end step begins, Zimone's ability won't trigger at all."
    );
    ruling!(
        "Zimone, All-Questioning",
        "In case you do end up with more than 31 lands, the remaining prime numbers below 100 are 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, and 97."
    );
    ruling!(
        "Zimone, All-Questioning",
        "There are a few more prime numbers, but we couldn't fit them all on the card."
    );
    supported("Zimone, All-Questioning");
    // Three lands, none entered this turn: no trigger.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zimone, All-Questioning");
    t.lands(P0, "Forest", 3);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, z), 0);
    // (lands with play, total, primo)
    for (total, primo) in [(3usize, true), (37, true), (101, true), (35, false), (39, false)] {
        let mut t = TestGame::new(2);
        let z = t.battlefield(P0, "Zimone, All-Questioning");
        t.lands(P0, "Forest", total - 1);
        play_from_hand(&mut t, P0, "Island");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_of(&t, z), usize::from(primo), "{total}");
        t.resolve_all();
        let p = t.named_on_battlefield("Primo, the Indivisible");
        if primo {
            assert_eq!(t.counters(p[0], "+1/+1"), total as u32, "{total}");
        } else {
            assert!(p.is_empty());
        }
    }
}

#[test]
fn null_profusion_and_spellbook_apply_in_timestamp_order() {
    cr!("402.2", "613.7", "514.1");
    ruling!(
        "Null Profusion",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Spellbook (an artifact that says you have no maximum hand size) onto the battlefield and then put Null Profusion onto the battlefield, your maximum hand size will be two. However, if those permanents entered in the opposite order, you would have no maximum hand size."
    );
    supported("Spellbook");
    for (first, second, max) in [
        ("Spellbook", "Null Profusion", Some(2)),
        ("Null Profusion", "Spellbook", None),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, first);
        t.battlefield(P0, second);
        t.g.recompute();
        assert_eq!(t.g.player(P0).max_hand_size, max, "{first} then {second}");
    }
}

#[test]
fn spitfire_lagac_in_two_headed_giant() {
    cr!("810.9", "810.9a");
    ruling!(
        "Spitfire Lagac",
        "In a Two-Headed Giant game, Spitfire Lagac's ability causes the opposing team to lose 2 life."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Spitfire Lagac");
    let start = t.life(P0);
    play_from_hand(&mut t, P0, "Mountain");
    t.resolve_all();
    assert_eq!(t.life(P0), start);
    assert_eq!(t.life(P2), start);
    assert_eq!(t.life(P1), start - 2);
    assert_eq!(t.life(P3), start - 2);
}

#[test]
fn surrakar_marauders_intimidate_uses_its_current_colors() {
    cr!("702.13b", "105.2");
    ruling!(
        "Surrakar Marauder",
        "Intimidate looks at the current colors of a creature that has it. Normally, if Surrakar Marauder has intimidate, it can't be blocked except by artifact creatures and/or black creatures. If it's turned white, then it can't be blocked except by artifact creatures and/or white creatures."
    );
    supported("Surrakar Marauder");
    for white in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Surrakar Marauder");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let lion = t.battlefield(P1, "Silvercoat Lion");
        let knight = t.battlefield(P1, "Vampire Nighthawk");
        play_from_hand(&mut t, P0, "Swamp");
        t.resolve_all();
        if white {
            crate::r_s26_common::modify_until_eot(
                &mut t,
                m,
                vec![Modification::SetColors(ColorSet::single(Color::White))],
            );
        }
        attack_with(&mut t, &[(m, Entity::Player(P1))]);
        assert!(!legal_blocks(&mut t, P1, &[(bears, m)]), "white {white}");
        assert_eq!(legal_blocks(&mut t, P1, &[(lion, m)]), white);
        assert_eq!(legal_blocks(&mut t, P1, &[(knight, m)]), !white);
    }
}

#[test]
fn pia_nalaars_thopter_keeps_attacking_after_losing_haste() {
    cr!("506.4", "302.6");
    ruling!(
        "Pia Nalaar, Consul of Revival",
        "Once a Thopter that came under your control this turn has legally attacked, causing it to lose haste by removing Pia Nalaar won't remove that Thopter from combat."
    );
    supported("Pia Nalaar, Consul of Revival");
    let mut t = TestGame::new(2);
    let pia = t.battlefield(P0, "Pia Nalaar, Consul of Revival");
    // A Thopter token created this turn (the helper clears summoning sickness).
    let thopter = crate::r_s02_common::create_token(&mut t, P0, "Thopter");
    t.g.objects[thopter.0 as usize].summoning_sick = true;
    // Without Pia, a Thopter that came under P0's control this turn can't attack.
    let mut t2 = TestGame::new(2);
    let other = crate::r_s02_common::create_token(&mut t2, P0, "Thopter");
    t2.g.objects[other.0 as usize].summoning_sick = true;
    t2.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t2, other));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, thopter));
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(thopter, Entity::Player(P1))]);
    destroy(&mut t, pia);
    assert!(crate::r_s10_common::attacking(&t, thopter));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn hormagaunt_horde_payment_is_decided_on_resolution() {
    cr!("603.3", "608.2c", "117.3");
    ruling!(
        "Hormagaunt Horde",
        "Players can respond to the triggered ability, but once it starts resolving and you decide whether to pay {2}{G}, it's too late for anyone to respond."
    );
    ruling!(
        "Seer's Sundial",
        "You choose whether to pay {2} as the ability resolves. You may pay {2} only once per resolution."
    );
    supported("Hormagaunt Horde");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let horde = t.graveyard(P0, "Hormagaunt Horde");
        t.lands(P0, "Forest", 3);
        let from = t.asked().len();
        play_from_hand(&mut t, P0, "Forest");
        assert_eq!(triggers_of(&t, horde), 1);
        assert!(asked_since(&t, from)
            .iter()
            .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
        if respond {
            move_to(&mut t, horde, Zone::Exile);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Hormagaunt Horde"), !respond);
    }
    // Seer's Sundial: plenty of mana, one payment and one card.
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Seer's Sundial");
    t.lands(P0, "Wastes", 6);
    play_from_hand(&mut t, P0, "Forest");
    assert_eq!(triggers_of(&t, dial), 1);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn the_ozoliths_counters_on_a_noncreature_artifact() {
    cr!("122.1b", "702.11b");
    ruling!(
        "The Ozolith",
        "+1/+1 counters on The Ozolith have no effect unless it becomes a creature. Keyword counters on The Ozolith will grant it keywords that may have no practical effect."
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let oz = t.battlefield(P0, "The Ozolith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 2);
    crate::r_s29_common::put_counters(&mut t, bears, "hexproof", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(oz, "+1/+1"), 2);
    assert_eq!(t.counters(oz, "hexproof"), 1);
    assert!(!t.obj_now(oz).is(CardType::Creature));
    assert!(crate::r_s06_common::has_kw(
        &t,
        oz,
        mtg_engine::keywords::KeywordKind::Hexproof
    ));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let targets = crate::r_s04_common::spell_targets(&mut t, P1, "Shatter");
    assert!(!targets.contains(&obj(oz)));
    let other = t.battlefield(P0, "Sol Ring");
    let targets = crate::r_s04_common::spell_targets(&mut t, P1, "Shatter");
    assert!(targets.contains(&obj(other)));
}
