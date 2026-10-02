//! Rulings batch P120 — abilities that look back at a permanent's counters as it left the
//! battlefield (CR 603.10a, 608.2h): a creature with +1/+1 counters that gets enough
//! -1/-1 counters to die still had all its +1/+1 counters as it last existed, because the
//! counters annihilate (CR 704.5q) in the same state-based action check that puts it into
//! the graveyard (CR 704.5f, 704.3). Leaves-the-battlefield abilities of permanents that
//! leave at the same time also trigger (CR 603.10a), and a permanent dealt lethal damage
//! dies before a trigger can save it (CR 704.5g, 603.3).

use crate::r_p120_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The permanents named `name` controlled by `p`.
fn named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .count()
}

/// Destroys every creature on the battlefield at once (P0 casts Wrath of God).
fn wrath(t: &mut TestGame) {
    cast_new(t, P0, "Wrath of God", &[]);
    t.resolve();
}

// --- "with a +1/+1 counter on it dies" -----------------------------------------------------

#[test]
fn skyclave_shadowcat_triggers_for_creatures_dying_with_it() {
    cr!("603.10a");
    ruling!(
        "Skyclave Shadowcat",
        "If Skyclave Shadowcat dies at the same time as another creature you control with a +1/+1 counter on it, its last ability triggers for that creature."
    );
    supported("Skyclave Shadowcat");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyclave Shadowcat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    wrath(&mut t);
    // One trigger: the Bears had a counter; the Shadowcat and the Giant didn't.
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn skyclave_shadowcat_triggers_for_itself() {
    cr!("603.10a");
    ruling!(
        "Skyclave Shadowcat",
        "If Skyclave Shadowcat has a +1/+1 counter on it and it dies, its last ability triggers."
    );
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Skyclave Shadowcat");
    give_plus1(&mut t, cat, 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, cat);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn skyclave_shadowcat_sees_counters_annihilated_as_it_died() {
    cr!("603.10a", "704.5q", "704.5f", "704.3");
    ruling!(
        "Skyclave Shadowcat",
        "If a creature with a +1/+1 counter on it died due to receiving too many -1/-1 counters, it dies at the same time that the counters are removed, and Skyclave Shadowcat’s last ability triggers."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyclave Shadowcat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    let hand = t.hand_size(P0);
    give_minus1(&mut t, bears, 4);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn basris_lieutenant_sees_counters_annihilated_as_it_died() {
    cr!("603.10a", "704.5q", "704.5f", "603.4");
    ruling!(
        "Basri's Lieutenant",
        "If a creature with +1/+1 counters on it receives a greater or equal number of -1/-1 counters and this causes it to be destroyed by lethal damage or put into its owner's graveyard for having 0 or less toughness, the last ability triggers."
    );
    supported("Basri's Lieutenant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Basri's Lieutenant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    give_minus1(&mut t, bears, 3);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Knight"), 1);
    // Lethal damage: an equal number of -1/-1 counters, with damage already marked.
    let elves = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, elves, 2);
    let src = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, src, 3, elves);
    assert!(t.on_battlefield(elves));
    give_minus1(&mut t, elves, 2);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Knight"), 2);
}

#[test]
fn grakmaw_uses_counters_from_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Grakmaw, Skyclave Ravager",
        "If a creature with a +1/+1 counter on it dies after receiving too many -1/-1 counters, it dies at the same time that the counters are removed, and Grakmaw's second ability triggers."
    );
    supported("Grakmaw, Skyclave Ravager");
    let mut t = TestGame::new(2);
    let grakmaw = t.enter(P0, "Grakmaw, Skyclave Ravager");
    assert_eq!(plus1(&t, grakmaw), 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    give_minus1(&mut t, bears, 4);
    t.resolve_all();
    assert_eq!(plus1(&t, grakmaw), 4);
    // Grakmaw itself: four +1/+1 counters, five -1/-1 counters: a 4/4 Hydra.
    give_minus1(&mut t, grakmaw, 5);
    assert!(t.in_graveyard(P0, "Grakmaw, Skyclave Ravager"));
    t.resolve_all();
    let hydra = t
        .g
        .permanents()
        .find(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Hydra"))
        .map(|o| o.id)
        .expect("a Hydra token");
    assert_eq!(t.pt(hydra), (4, 4));
}

/// The +1/+1 counters it had before getting `minus` -1/-1 counters (with `plus` +1/+1
/// counters on it) are what a "for each +1/+1 counter on it" dies ability counts.
fn dies_with_minus_counters(t: &mut TestGame, id: ObjectId, plus: u32, minus: u32) {
    let have = plus1(t, id);
    if plus > have {
        give_plus1(t, id, plus - have);
    }
    assert_eq!(plus1(t, id), plus);
    give_minus1(t, id, minus);
    assert!(!t.on_battlefield(id), "it should have died");
    t.resolve_all();
}

#[test]
fn big_mother_mouser_counts_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Big Mother Mouser",
        "For example, if there are three +1/+1 counters on Big Mother Mouser and it gets four -1/-1 counters, you'll get three Robot tokens."
    );
    supported("Big Mother Mouser");
    let mut t = TestGame::new(2);
    let mouser = t.enter(P0, "Big Mother Mouser");
    assert_eq!(plus1(&t, mouser), 2);
    dies_with_minus_counters(&mut t, mouser, 3, 4);
    assert_eq!(tokens_named(&t, P0, "Robot"), 3);
}

#[test]
fn bloodtracker_draws_for_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Bloodtracker",
        "For example, if there are three +1/+1 counters on Bloodtracker and it gets six -1/-1 counters, you'll draw three cards."
    );
    supported("Bloodtracker");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bloodtracker");
    let hand = t.hand_size(P0);
    dies_with_minus_counters(&mut t, b, 3, 6);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn chasm_skulker_makes_squids_for_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Chasm Skulker",
        "For example, if there are two +1/+1 counters on Chasm Skulker and it gets three -1/-1 counters, you'll get two Squid tokens."
    );
    supported("Chasm Skulker");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Chasm Skulker");
    dies_with_minus_counters(&mut t, s, 2, 3);
    assert_eq!(tokens_named(&t, P0, "Squid"), 2);
}

#[test]
fn hangarback_walker_makes_thopters_for_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Hangarback Walker",
        "For example, if there are three +1/+1 counters on Hangarback Walker and it gets four -1/-1 counters, you'll get three Thopter tokens."
    );
    supported("Hangarback Walker");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Hangarback Walker");
    dies_with_minus_counters(&mut t, w, 3, 4);
    assert_eq!(tokens_named(&t, P0, "Thopter"), 3);
}

#[test]
fn toothy_draws_for_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Toothy, Imaginary Friend",
        "For example, if there are two +1/+1 counters on Toothy and it gets three -1/-1 counters, you'll draw two cards."
    );
    supported("Toothy, Imaginary Friend");
    let mut t = TestGame::new(2);
    let toothy = t.battlefield(P0, "Toothy, Imaginary Friend");
    let hand = t.hand_size(P0);
    dies_with_minus_counters(&mut t, toothy, 2, 3);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn the_ooze_counts_counters_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "The Ooze",
        "For example, if there are three +1/+1 counters on a creature and it gets four -1/-1 counters, you'll get three Mutagen tokens."
    );
    supported("The Ooze");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Ooze");
    // A 0/0 (Hangarback Walker) with three +1/+1 counters gets four -1/-1 counters.
    let w = t.battlefield(P0, "Hangarback Walker");
    dies_with_minus_counters(&mut t, w, 3, 4);
    assert_eq!(named(&t, P0, "Mutagen Token"), 3);
}

#[test]
fn aerith_uses_counters_from_before_the_minus_counters() {
    cr!("603.10a", "704.5q", "704.5f");
    ruling!(
        "Aerith Gainsborough",
        "In the rare case where enough -1/-1 counters are put on Aerith Gainsborough at the same time to make its toughness 0 or less, the number of +1/+1 counters on it before it got those -1/-1 counters will be used to determine the value of X"
    );
    supported("Aerith Gainsborough");
    let mut t = TestGame::new(2);
    let aerith = t.battlefield(P0, "Aerith Gainsborough");
    let legend = t.battlefield(P0, "Toothy, Imaginary Friend");
    let other = t.battlefield(P0, "Grizzly Bears");
    dies_with_minus_counters(&mut t, aerith, 2, 4);
    assert_eq!(plus1(&t, legend), 2);
    assert_eq!(plus1(&t, other), 0);
}

// --- several abilities, several creatures ---------------------------------------------------

#[test]
fn laid_to_rest_both_abilities_trigger_for_a_human_with_a_counter() {
    cr!("603.2", "603.3b");
    ruling!(
        "Laid to Rest",
        "If a Human you control with a +1/+1 counter on it dies, both abilities will trigger."
    );
    supported("Laid to Rest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laid to Rest");
    let human = t.battlefield(P0, "Elite Vanguard");
    give_plus1(&mut t, human, 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, human);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn alharu_triggers_for_creatures_dying_with_it() {
    cr!("603.10a");
    ruling!(
        "Alharu, Solemn Ritualist",
        "If a creature you control with a +1/+1 counter on it dies at the same time as Alharu, Alharu's ability triggers for that creature."
    );
    supported("Alharu, Solemn Ritualist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alharu, Solemn Ritualist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    t.battlefield(P0, "Hill Giant");
    wrath(&mut t);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Spirit"), 1);
}

#[test]
fn taborax_dealt_lethal_damage_with_other_creatures_still_dies() {
    cr!("704.5g", "603.10a", "603.3");
    ruling!(
        "Taborax, Hope's Demise",
        "If Taborax is dealt lethal damage at the same time as one or more nontoken creatures you control die or are dealt lethal damage, it won't receive a counter from its ability in time to save it."
    );
    supported("Taborax, Hope's Demise");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    let taborax = t.battlefield(P0, "Taborax, Hope's Demise");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve();
    // Both are destroyed by the same state-based action check.
    assert!(!t.on_battlefield(taborax));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Taborax, Hope's Demise"));
}

#[test]
fn taborax_dying_with_a_cleric_may_still_draw() {
    cr!("603.10a");
    ruling!(
        "Taborax, Hope's Demise",
        "If another nontoken Cleric dies at the same time as Taborax, you won't put a +1/+1 counter on anything, but you may draw a card as its last ability resolves. You still lose 1 life if you do."
    );
    let mut t = TestGame::new(2);
    let taborax = t.battlefield(P0, "Taborax, Hope's Demise");
    t.battlefield(P0, "Voice of the Blessed");
    let other = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve();
    assert!(!t.on_battlefield(taborax));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
    assert_eq!(plus1(&t, other), 0);
}

#[test]
fn voice_of_the_blessed_dealt_lethal_damage_as_you_gain_life_dies() {
    cr!("704.5g", "603.3", "510.2");
    ruling!(
        "Voice of the Blessed",
        "If Voice of the Blessed is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    supported("Voice of the Blessed");
    let mut t = TestGame::new(2);
    let voice = t.battlefield(P0, "Voice of the Blessed");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    // Exactly lethal for a 2/2; a 3/3 (had the counter come first) would survive.
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(voice, Entity::Player(P1)), (hawk, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(bears, voice)]);
    // The Nighthawk's lifelink damage and the Bears' damage are dealt simultaneously.
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Voice of the Blessed"));
}

// --- putting a dead creature's counters on another creature --------------------------------

#[test]
fn enduring_bondwarden_ability_includes_minus_counters() {
    cr!("603.10a", "702.165a");
    ruling!(
        "Enduring Bondwarden",
        "If the creature with Enduring Bondwarden’s last ability has -1/-1 counters on it when it dies, that ability will include those as well. This may result in the recipient also dying."
    );
    supported("Enduring Bondwarden");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let recipient = t.battlefield(P0, "Grizzly Bears");
    // Backup 1 on the Giant: a +1/+1 counter and the dies ability until end of turn.
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Enduring Bondwarden");
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 1);
    // Four -1/-1 counters: it dies with one +1/+1 and four -1/-1 counters.
    t.answer_targets(P0, &[obj(recipient)]);
    give_minus1(&mut t, giant, 4);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    // The Bears gets one +1/+1 and four -1/-1 counters, and dies too.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn enduring_bondwarden_with_the_ozolith_puts_counters_on_both() {
    cr!("603.10a", "603.3b");
    ruling!(
        "Enduring Bondwarden",
        "For example, if you control The Ozolith when Enduring Bondwarden dies, you’ll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let ozolith = t.battlefield(P0, "The Ozolith");
    let bond = t.battlefield(P0, "Enduring Bondwarden");
    give_plus1(&mut t, bond, 2);
    let target = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(target)]);
    destroy(&mut t, bond);
    t.resolve_all();
    assert_eq!(plus1(&t, ozolith), 2);
    assert_eq!(plus1(&t, target), 2);
}

#[test]
fn iron_apprentice_puts_copies_of_its_counters_with_the_ozolith() {
    cr!("603.10a", "603.4");
    ruling!(
        "Iron Apprentice",
        "For example, if you control The Ozolith when Iron Apprentice dies, you will put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    ruling!(
        "Iron Apprentice",
        "Rather, you put the same number of each kind of counter Iron Apprentice had when it died onto that creature."
    );
    supported("Iron Apprentice");
    let mut t = TestGame::new(2);
    let ozolith = t.battlefield(P0, "The Ozolith");
    let app = t.enter(P0, "Iron Apprentice");
    assert_eq!(plus1(&t, app), 1);
    give_plus1(&mut t, app, 1);
    let target = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(target)]);
    destroy(&mut t, app);
    t.resolve_all();
    assert_eq!(plus1(&t, ozolith), 2);
    assert_eq!(plus1(&t, target), 2);
}

#[test]
fn iron_apprentice_puts_every_kind_of_counter() {
    cr!("603.10a");
    ruling!(
        "Iron Apprentice",
        "Iron Apprentice puts all of its counters onto the target creature, not just its +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let app = t.enter(P0, "Iron Apprentice");
    put_counters(&mut t, app, "flying", 1);
    put_counters(&mut t, app, "oil", 2);
    let target = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(target)]);
    destroy(&mut t, app);
    t.resolve_all();
    assert_eq!(plus1(&t, target), 1);
    assert_eq!(t.counters(target, "flying"), 1);
    assert_eq!(t.counters(target, "oil"), 2);
}

#[test]
fn marchesa_doesnt_return_a_card_that_left_the_graveyard() {
    cr!("400.7", "603.7c");
    ruling!(
        "Marchesa, the Black Rose",
        "If the creature card leaves the graveyard before the delayed triggered ability resolves, that card won't return to the battlefield, even if it's back in the graveyard when the delayed triggered ability resolves."
    );
    supported("Marchesa, the Black Rose");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Marchesa, the Black Rose");
        let bears = t.battlefield(P0, "Grizzly Bears");
        give_plus1(&mut t, bears, 1);
        destroy(&mut t, bears);
        t.resolve_all();
        if leave {
            let card = t.g.current(bears);
            let exiled = crate::r_s05_common::move_to(&mut t, card, Zone::Exile).unwrap();
            crate::r_s05_common::move_to(&mut t, exiled, Zone::Graveyard(P0));
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
        }
        end_step(&mut t, P0);
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            usize::from(!leave),
            "leave = {leave}"
        );
    }
}

