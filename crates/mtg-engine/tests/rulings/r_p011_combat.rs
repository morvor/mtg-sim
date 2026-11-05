//! Rulings batch P011 — combat: creatures that assign combat damage equal to their
//! toughness (CR 510.1a), keywords and conditions that matter only as attackers or blockers
//! are declared (CR 508.1, 509.1, 702.111b, 702.20b), double strike changing mid-combat
//! (CR 702.4c, 510.4), simultaneous combat damage and state-based actions (CR 510.2, 704.3),
//! "attacks and isn't blocked" and creatures put onto the battlefield attacking (CR 508.3,
//! 508.4, 509.3g).

use crate::r_p009_common::{obj, pl};
use crate::r_p011_common::*;
use crate::r_p012_common::fill_graveyard;
use crate::r_p127_common::exile_now;
use crate::r_s01_common::{attack_with, block_and_finish, supported, tokens};
use crate::r_s06_common::attach_new;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Assigning combat damage equal to toughness -----------------------------------------------

/// Sets up the toughness-assigner `card` for P0 and returns P0's creature that is affected
/// by it (a 1/4 Ancient Lumberknot style creature with toughness greater than power).
fn toughness_setup(t: &mut TestGame, card: &str) -> ObjectId {
    supported(card);
    match card {
        "Bark of Doran" => {
            // Equipped Wall of Wood-like creature: Grizzly Bears 2/2 becomes 2/3.
            let bears = t.battlefield(P0, "Grizzly Bears");
            attach_new(t, P0, card, bears);
            bears
        }
        _ => {
            t.battlefield(P0, card);
            t.battlefield(P0, "Wall of Wood")
        }
    }
}

#[test]
fn toughness_assigners_dont_change_power() {
    cr!("510.1a", "701.14a");
    ruling!(
        "Ancient Lumberknot",
        "Ancient Lumberknot's ability doesn't actually change any creature's power. It changes only the amount of combat damage the creature assigns. All other rules and effects that check power or toughness use the real values, even if they cause damage “equal to a creature's power” to be dealt."
    );
    ruling!(
        "Bark of Doran",
        "Bark of Doran's second ability doesn't actually change any creature's power. It changes only the amount of combat damage the creature assigns. All other rules and effects that check power or toughness use the real values, even if they cause damage \"equal to a creature's power\" to be dealt."
    );
    ruling!(
        "Belligerent Brontodon",
        "Belligerent Brontodon’s ability doesn’t actually change any creature’s power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, Hunt the Weak won’t cause a creature to fight with its toughness."
    );
    ruling!(
        "Doran, the Siege Tower",
        "Doran's ability doesn't actually change creatures' power; it changes only the value of the combat damage they assign. All other rules and effects that check power or toughness use the real values."
    );
    supported("Hunt the Weak");
    supported("Wall of Wood");
    for card in [
        "Ancient Lumberknot",
        "Bark of Doran",
        "Belligerent Brontodon",
        "Doran, the Siege Tower",
    ] {
        // Combat: the creature (power p, toughness q > p) assigns q.
        let mut t = TestGame::new(2);
        let c = toughness_setup(&mut t, card);
        let (p, q) = t.pt(c);
        assert!(q > p, "{card}");
        let can_attack = card == "Bark of Doran";
        if can_attack {
            attack_with(&mut t, &[(c, pl(P1))]);
            block_and_finish(&mut t, P1, &[]);
            assert_eq!(t.life(P1), 20 - q, "{card}: combat damage is toughness");
        } else {
            // The Wall blocks P1's Craw Wurm and assigns its toughness.
            let wurm = t.battlefield(P1, "Craw Wurm");
            t.set_step(P1, Step::BeginningOfCombat);
            attack_with(&mut t, &[(wurm, pl(P0))]);
            block_and_finish(&mut t, P0, &[(c, wurm)]);
            assert_eq!(t.obj_now(wurm).damage, q as u32, "{card}: blocker assigns toughness");
        }
        // ... but its power is unchanged.
        assert_eq!(t.pt(c).0, p, "{card}");

        // Hunt the Weak: the creature fights with its power (+1 from the counter), not its
        // toughness: a 6/6 survives.
        let mut t = TestGame::new(2);
        let c = toughness_setup(&mut t, card);
        let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
        let (p, _) = t.pt(c);
        cast_new(&mut t, P0, "Hunt the Weak", &[obj(c), obj(dreadmaw)]);
        t.resolve_all();
        assert_eq!(t.obj_now(dreadmaw).damage, (p + 1) as u32, "{card}: fight uses power");
    }
}

#[test]
fn doran_affects_every_creature() {
    cr!("510.1a");
    ruling!(
        "Doran, the Siege Tower",
        "This effect is mandatory and affects all creatures."
    );
    // P0's Doran; P1's Wall of Wood (0/3) blocks P0's Grizzly Bears (2/2) and assigns 3;
    // P0's Bears assign 2 regardless (toughness 2).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doran, the Siege Tower");
    let wall = t.battlefield(P1, "Wall of Wood");
    let lions = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(lions, pl(P1))]);
    block_and_finish(&mut t, P1, &[(wall, lions)]);
    // The Lions (2/1) assign 1 (their toughness): the Wall survives with 1 damage; the
    // opponent's Wall assigns 3, its toughness.
    assert_eq!(t.obj_now(wall).damage, 1);
    assert!(!t.on_battlefield(lions));
    // An unblocked opponent's creature on its own turn also assigns toughness.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doran, the Siege Tower");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 17);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doran, the Siege Tower");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(lions, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 19, "a 2/1 assigns 1");
}

// --- Triggers on blocks --------------------------------------------------------------------

#[test]
fn escaped_null_triggers_once_for_several_blockers() {
    cr!("509.3c");
    ruling!(
        "Escaped Null",
        "When Escaped Null blocks or becomes blocked, its last ability triggers just once, no matter how many creatures Escaped Null blocked or became blocked by."
    );
    supported("Escaped Null");
    let mut t = TestGame::new(2);
    let en = t.battlefield(P0, "Escaped Null");
    let a = t.battlefield(P1, "Wall of Wood");
    let b = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(en, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, en), (b, en)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.pt(en), (6, 2));
}

// --- Keywords that matter only as attackers / blockers are declared -------------------------

#[test]
fn menace_gained_after_blocks_doesnt_unblock() {
    cr!("702.111b", "509.1h");
    ruling!(
        "Hound of the Farbogs",
        "Menace only matters as blockers are chosen. Causing Hound of the Farbogs to gain menace after blockers are chosen won't cause it to become unblocked."
    );
    ruling!(
        "Thraben Foulbloods",
        "Menace only matters as blockers are chosen. Causing Thraben Foulbloods to gain menace after blockers are chosen won't cause it to become unblocked."
    );
    for (card, power) in [("Hound of the Farbogs", 5), ("Thraben Foulbloods", 4)] {
        supported(card);
        // Positive control: with delirium already, a single creature can't block it.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, card);
        delirium(&mut t, P0);
        attack_with(&mut t, &[(c, pl(P1))]);
        assert_eq!(t.g.min_blockers(c), 2, "{card}");

        // Blocked by one creature, then it gains menace: still blocked.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, card);
        let wall = t.battlefield(P1, "Colossal Dreadmaw");
        attack_with(&mut t, &[(c, pl(P1))]);
        assert_eq!(t.g.min_blockers(c), 1, "{card}");
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wall, c)]));
        t.advance_to(P0, Step::DeclareBlockers);
        delirium(&mut t, P0);
        t.g.recompute();
        assert!(t.obj_now(c).has_keyword(KeywordKind::Menace), "{card}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{card}: still blocked");
        assert_eq!(t.obj_now(wall).damage, power, "{card}");
    }
}

#[test]
fn vigilance_gained_or_lost_after_attacking_doesnt_untap_or_tap() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Inquisitor's Ox",
        "Gaining vigilance any time after the moment you choose to attack with Inquisitor's Ox won't cause it to become untapped."
    );
    ruling!(
        "Aang, A Lot to Learn",
        "Gaining vigilance any time after the moment you choose to attack with Aang, A Lot to Learn won't cause him to become untapped, and losing vigilance after that time won't cause it to become tapped."
    );
    supported("Inquisitor's Ox");
    supported("Aang, A Lot to Learn");
    // Inquisitor's Ox attacks without delirium (taps), then gets delirium: stays tapped.
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Inquisitor's Ox");
    attack_with(&mut t, &[(ox, pl(P1))]);
    assert!(t.obj_now(ox).tapped);
    delirium(&mut t, P0);
    t.g.recompute();
    assert!(t.obj_now(ox).has_keyword(KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(t.obj_now(ox).tapped);
    assert_eq!(t.life(P1), 17, "3/5 with delirium");
    // Positive control: with delirium beforehand it attacks without tapping.
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Inquisitor's Ox");
    delirium(&mut t, P0);
    attack_with(&mut t, &[(ox, pl(P1))]);
    assert!(!t.obj_now(ox).tapped);

    // Aang: attacks without a Lesson (taps), then gains vigilance: stays tapped.
    let mut t = TestGame::new(2);
    let aang = t.battlefield(P0, "Aang, A Lot to Learn");
    attack_with(&mut t, &[(aang, pl(P1))]);
    assert!(t.obj_now(aang).tapped);
    t.graveyard(P0, "Environmental Sciences");
    t.g.recompute();
    assert!(t.obj_now(aang).has_keyword(KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(t.obj_now(aang).tapped);
    // Attacks with a Lesson (doesn't tap), then loses vigilance: stays untapped.
    let mut t = TestGame::new(2);
    let aang = t.battlefield(P0, "Aang, A Lot to Learn");
    let lesson = t.graveyard(P0, "Environmental Sciences");
    attack_with(&mut t, &[(aang, pl(P1))]);
    assert!(!t.obj_now(aang).tapped);
    exile_now(&mut t, lesson);
    t.g.recompute();
    assert!(!t.obj_now(aang).has_keyword(KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.obj_now(aang).tapped);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn geist_stays_in_combat_after_losing_delirium() {
    cr!("508.1c", "506.4");
    ruling!(
        "Geist of the Lonely Vigil",
        "Once you've attacked with Geist of the Lonely Vigil, lowering the number of card types in your graveyard won't remove it from combat."
    );
    supported("Geist of the Lonely Vigil");
    // Without delirium it can't attack (defender).
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Geist of the Lonely Vigil");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.g.can_attack(geist));
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Geist of the Lonely Vigil");
    let cards = delirium(&mut t, P0);
    attack_with(&mut t, &[(geist, pl(P1))]);
    assert!(t.g.is_attacking(geist));
    for c in cards {
        exile_now(&mut t, c);
    }
    assert!(t.g.is_attacking(geist));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mournwillow_checks_power_as_blockers_are_chosen() {
    cr!("611.2c", "509.1a");
    ruling!(
        "Mournwillow",
        "Because the last ability’s effect doesn’t change the characteristics of any permanents, the set of creatures affected is constantly updated. Any creature with power 2 or less at the moment blockers are chosen will be unable to block."
    );
    supported("Mournwillow");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    delirium(&mut t, P0);
    crate::r_s05_common::enter(&mut t, P0, "Mournwillow");
    t.resolve_all();
    let willow = t.named_on_battlefield("Mournwillow")[0];
    // After the effect began: the Bears grow to 3 power, the Giant shrinks to 2.
    crate::r_p057_common::pump(&mut t, bears, 1, 0);
    crate::r_p057_common::pump(&mut t, giant, -1, 0);
    attack_with(&mut t, &[(willow, pl(P1))]);
    assert!(t.g.can_block(bears, willow), "the Bears now have power 3");
    assert!(!t.g.can_block(giant, willow), "the Giant now has power 2");
    let offered: Vec<ObjectId> = mtg_engine::combat::block_options(&t.g, &[P1])
        .into_iter()
        .map(|(b, _)| b)
        .collect();
    assert_eq!(offered, vec![bears]);
}

// --- Double strike gained or lost mid-combat ------------------------------------------------

#[test]
fn spelleater_wolverine_double_strike_changes_mid_combat() {
    cr!("702.4c", "510.4");
    ruling!(
        "Spelleater Wolverine",
        "If Spelleater Wolverine loses double strike after it deals first-strike damage, it won't deal regular combat damage. If it gains double strike after other creatures have dealt first-strike damage, it won't deal first-strike damage."
    );
    supported("Spelleater Wolverine");
    // Has double strike, deals first-strike damage, then loses it: 3 damage in all.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Spelleater Wolverine");
    let opts = fill_graveyard(&mut t, P0, "Opt", 3);
    attack_with(&mut t, &[(w, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 17);
    exile_now(&mut t, opts[0]);
    t.g.recompute();
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 17, "no regular combat damage");
    // Positive control: keeps double strike: 6.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Spelleater Wolverine");
    fill_graveyard(&mut t, P0, "Opt", 3);
    attack_with(&mut t, &[(w, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14);
    // Gains double strike after a first striker dealt first-strike damage: it deals only
    // regular combat damage.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Spelleater Wolverine");
    let knight = t.battlefield(P0, "White Knight");
    fill_graveyard(&mut t, P0, "Opt", 2);
    attack_with(&mut t, &[(w, pl(P1)), (knight, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18, "only the Knight");
    t.graveyard(P0, "Opt");
    t.g.recompute();
    assert!(t.obj_now(w).has_keyword(KeywordKind::DoubleStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 15, "the Wolverine deals 3 once");
}

// --- Simultaneous combat damage -------------------------------------------------------------

#[test]
fn deathcap_cultivator_deathtouch_is_checked_as_damage_is_dealt() {
    cr!("510.2", "702.2b");
    ruling!(
        "Deathcap Cultivator",
        "If you have three noncreature card types among cards in your graveyard at the time damage is dealt by Deathcap Cultivator, and lethal damage is dealt to another creature you control at the same time, the damage from Deathcap Cultivator won't be from a source with deathtouch."
    );
    supported("Deathcap Cultivator");
    for with_creature in [false, true] {
        let mut t = TestGame::new(2);
        let dc = t.battlefield(P0, "Deathcap Cultivator");
        let bears = t.battlefield(P0, "Grizzly Bears");
        for n in ["Forest", "Opt", "Divination"] {
            t.graveyard(P0, n);
        }
        if with_creature {
            t.graveyard(P0, "Hill Giant");
        }
        let wurm = t.battlefield(P1, "Craw Wurm");
        let giant = t.battlefield(P1, "Hill Giant");
        attack_with(&mut t, &[(dc, pl(P1)), (bears, pl(P1))]);
        block_and_finish(&mut t, P1, &[(wurm, dc), (giant, bears)]);
        assert!(!t.on_battlefield(bears));
        // The Bears' death made four types, but too late: the Wurm survives the 2 damage.
        assert_eq!(t.on_battlefield(wurm), !with_creature, "creature card before: {with_creature}");
    }
}

#[test]
fn jaces_phantasm_dies_before_the_tenth_card_arrives() {
    cr!("510.2", "704.3", "704.5g");
    ruling!(
        "Jace's Phantasm",
        "If Jace's Phantasm is dealt lethal damage at the same time as an opponent's creature, and that creature will become that opponent's tenth card in their graveyard, Jace's Phantasm will be destroyed before it gets +4/+4. This is because all creatures that have been dealt lethal damage are destroyed at the same time."
    );
    let mut t = TestGame::new(2);
    let jp = t.battlefield(P0, "Jace's Phantasm");
    fill_graveyard(&mut t, P1, "Opt", 9);
    let lions = t.battlefield(P1, "Savannah Lions");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(lions, pl(P0))]);
    block_and_finish(&mut t, P0, &[(jp, lions)]);
    assert_eq!(t.graveyard_size(P1), 10);
    assert!(!t.on_battlefield(jp));
    assert!(t.in_graveyard(P0, "Jace's Phantasm"));
}

// --- Attacking: defending player, "isn't blocked", creatures put onto the battlefield ------

#[test]
fn vantress_gargoyle_attacking_a_planeswalker_checks_its_controller() {
    cr!("802.2a", "508.1c");
    ruling!(
        "Vantress Gargoyle",
        "If Vantress Gargoyle attacks a planeswalker, the controller of that planeswalker is the defending player."
    );
    supported("Vantress Gargoyle");
    let mut t = TestGame::new(3);
    let vg = t.battlefield(P0, "Vantress Gargoyle");
    let jace1 = t.battlefield(P1, "Jace Beleren");
    let jace2 = t.battlefield(P2, "Jace Beleren");
    fill_graveyard(&mut t, P1, "Opt", 7);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.g.can_attack_target(vg, obj(jace1)));
    assert!(!t.g.can_attack_target(vg, obj(jace2)));
    assert!(t.g.can_attack_target(vg, pl(P1)));
    assert!(!t.g.can_attack_target(vg, pl(P2)));
    attack_with(&mut t, &[(vg, obj(jace1))]);
    assert!(t.g.is_attacking(vg));
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.on_battlefield(jace1), "5 damage to a 3-loyalty Jace");
}

#[test]
fn guiltfeeder_triggers_when_put_onto_the_battlefield_attacking() {
    cr!("509.3g", "508.4");
    ruling!(
        "Guiltfeeder",
        "An ability that triggers when something “attacks and isn't blocked” triggers in the declare blockers step after blockers are declared if (1) that creature is attacking and (2) no creatures are declared to block it. It will trigger even if that creature was put onto the battlefield attacking rather than having been declared as an attacker in the declare attackers step."
    );
    supported("Guiltfeeder");
    for blocked in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Wood");
        fill_graveyard(&mut t, P1, "Opt", 3);
        attack_with(&mut t, &[(bears, pl(P1))]);
        let gf = t.battlefield(P0, "Guiltfeeder");
        mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, gf, pl(P1));
        t.g.recompute();
        assert!(t.g.is_attacking(gf));
        // Wall of Wood (green) can't block Guiltfeeder (fear); "blocked" means the
        // Bears are blocked, Guiltfeeder isn't either way.
        let blocks = if blocked { vec![(wall, bears)] } else { vec![] };
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks));
        t.advance_to(P0, Step::DeclareBlockers);
        t.resolve_all();
        assert_eq!(t.life(P1), 17, "blocked bears: {blocked}");
    }
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_werent_declared() {
    cr!("508.3a", "508.4");
    // Misattached: this ruling describes a card whose triggered ability looks for creatures
    // declared as attackers attacking you (Scuttletide has no triggered ability); it's
    // tested with Search the Premises ("Whenever a creature attacks you or a planeswalker
    // you control, investigate.").
    ruling!(
        "Scuttletide",
        "The triggered ability triggers when creatures are declared as attackers attacking you or a planeswalker you control. It will not trigger if an effect puts a creature onto the battlefield attacking you or a planeswalker you control."
    );
    supported("Scuttletide");
    supported("Search the Premises");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scuttletide");
    t.battlefield(P0, "Search the Premises");
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, obj(jace))]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "declared attacking a planeswalker P0 controls");
    let giant = t.battlefield(P1, "Hill Giant");
    mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, giant, pl(P0));
    t.g.recompute();
    assert!(t.g.is_attacking(giant));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "put onto the battlefield attacking: no trigger");
}

#[test]
fn attacking_tokens_enter_tapped_and_may_attack_something_else() {
    cr!("508.4", "702.20b");
    ruling!(
        "Inexorable Blob",
        "If an effect gives the token vigilance, the token will still enter the battlefield tapped."
    );
    ruling!(
        "Invocation of Saint Traft",
        "If an effect gives the token vigilance, the token will still enter the battlefield tapped."
    );
    ruling!(
        "Inexorable Blob",
        "You declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Inexorable Blob is attacking."
    );
    supported("Inexorable Blob");
    supported("Invocation of Saint Traft");
    supported("Brave the Sands");
    // Inexorable Blob attacks P1; its token attacks P1's planeswalker.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brave the Sands");
    let blob = t.battlefield(P0, "Inexorable Blob");
    let jace = t.battlefield(P1, "Jace Beleren");
    delirium(&mut t, P0);
    t.answer_choose(P0, &[obj(jace)]);
    attack_with(&mut t, &[(blob, pl(P1))]);
    t.resolve_all();
    let ooze = tokens(&t, P0)[0];
    assert!(t.obj_now(ooze).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(ooze).tapped);
    assert_eq!(t.g.combat.as_ref().unwrap().attack_target(ooze), Some(obj(jace)));
    assert_eq!(t.g.combat.as_ref().unwrap().attack_target(blob), Some(pl(P1)));
    // P0 was asked to choose what it attacks, with both options offered.
    let offered = t.asked().iter().any(|(p, d)| {
        *p == P0
            && matches!(d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&obj(jace)) && candidates.contains(&pl(P1)))
    });
    assert!(offered);

    // Invocation of Saint Traft's Angel with vigilance also enters tapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brave the Sands");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Invocation of Saint Traft", bears);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let angel = tokens(&t, P0)[0];
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(angel).tapped);
    assert!(t.g.is_attacking(angel));
}

#[test]
fn hasran_ogress_pays_or_takes_damage_on_resolution() {
    cr!("118.12a", "118.12");
    ruling!(
        "Hasran Ogress",
        "You either take damage or pay the 2 mana during resolution."
    );
    supported("Hasran Ogress");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let ogress = t.battlefield(P0, "Hasran Ogress");
        t.lands(P0, "Swamp", 2);
        attack_with(&mut t, &[(ogress, pl(P1))]);
        // The trigger is on the stack; nothing has been paid or asked yet.
        assert_eq!(t.stack_len(), 1);
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
        let before = t.asked().len();
        t.answer_yes(P0, pay);
        t.resolve_all();
        let asked = t.asked()[before..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }));
        assert!(asked, "asked as it resolves");
        assert_eq!(t.life(P0), if pay { 20 } else { 17 });
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), if pay { 2 } else { 0 });
    }
}
