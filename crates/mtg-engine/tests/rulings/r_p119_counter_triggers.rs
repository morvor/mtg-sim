//! Rulings batch P119 — triggered abilities of "+1/+1 counters matter" cards: abilities
//! that trigger on counters being put (including entering with counters, CR 122.6),
//! once per life-gaining event (CR 119.9), on casting (resolving before the spell, CR
//! 603.3), on dying with a counter (leaves-the-battlefield abilities look back, CR
//! 603.10a), and abilities that use last known information (CR 608.2h).

use crate::r_p119_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- counters being put ------------------------------------------------------------------

#[test]
fn enduring_scalelord_triggers_for_a_creature_entering_with_counters() {
    cr!("122.6", "603.2");
    ruling!(
        "Enduring Scalelord",
        "A creature entering the battlefield with one or more +1/+1 counters on it will cause Enduring Scalelord's ability to trigger."
    );
    supported("Enduring Scalelord");
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Enduring Scalelord");
    t.answer_yes(P0, true);
    enter_with(&mut t, P0, "Grizzly Bears", 1, false);
    t.resolve_all();
    assert_eq!(plus1(&t, lord), 1);
}

#[test]
fn wildwood_scourge_triggers_on_entering_with_counters_and_on_counters_put() {
    cr!("122.6", "603.2");
    ruling!(
        "Wildwood Scourge",
        "Abilities that trigger when counters are put on a creature trigger when a creature enters with counters and when a player puts counters on a creature."
    );
    supported("Wildwood Scourge");
    let mut t = TestGame::new(2);
    let scourge = with_counters(&mut t, P0, "Wildwood Scourge", 1);
    let bears = enter_with(&mut t, P0, "Grizzly Bears", 2, false);
    t.resolve_all();
    assert_eq!(plus1(&t, scourge), 2);
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, scourge), 3);
}

/// P0 casts Travel Preparations on two of its creatures (counters put on both at once);
/// `answers` yes/no answers are queued first.
fn travel_preparations(t: &mut TestGame, answers: usize) {
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Elite Vanguard");
    for _ in 0..answers {
        t.answer_yes(P0, true);
    }
    cast_slots(t, P0, "Travel Preparations", &[&[obj(a), obj(b)]]);
    t.resolve_all();
    assert_eq!((plus1(t, a), plus1(t, b)), (1, 1));
}

#[test]
fn counters_put_on_several_permanents_at_once_trigger_once_for_each() {
    cr!("603.2c", "122.6");
    ruling!(
        "Animation Module",
        "If +1/+1 counters are put on more than one permanent you control at the same time, Animation Module's first ability triggers once for each of those permanents."
    );
    ruling!(
        "Enduring Scalelord",
        "If +1/+1 counters are put on multiple creatures you control (other than Enduring Scalelord) at the same time, Enduring Scalelord's ability will trigger once for each of those creatures."
    );
    supported("Animation Module");
    supported("Travel Preparations");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Animation Module");
    t.lands(P0, "Wastes", 2);
    travel_preparations(&mut t, 2);
    assert_eq!(tokens_named(&t, P0, "Servo"), 2);

    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Enduring Scalelord");
    travel_preparations(&mut t, 2);
    assert_eq!(plus1(&t, lord), 2);
}

#[test]
fn entering_with_counters_triggers_once_per_counter() {
    cr!("122.6", "603.2");
    ruling!(
        "Bloodcrazed Hoplite",
        "If Bloodcrazed Hoplite enters the battlefield with +1/+1 counters on it, its last ability will trigger once for each of those counters."
    );
    ruling!(
        "Fathom Mage",
        "If Fathom Mage enters with a +1/+1 counter on it (perhaps due to Master Biomancer), its last ability will trigger once for each +1/+1 counter it entered with."
    );
    supported("Bloodcrazed Hoplite");
    supported("Fathom Mage");
    let mut t = TestGame::new(2);
    let theirs = with_counters(&mut t, P1, "Grizzly Bears", 3);
    enter_with(&mut t, P0, "Bloodcrazed Hoplite", 2, false);
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, theirs), 1);

    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    enter_with(&mut t, P0, "Fathom Mage", 2, false);
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn generous_pup_and_grakmaw_put_just_one_counter() {
    cr!("603.2c");
    ruling!(
        "Generous Pup",
        "Generous Pup's last ability puts only one +1/+1 counter on each other creature you control, no matter how many +1/+1 counters were put on Generous Pup."
    );
    ruling!(
        "Grakmaw, Skyclave Ravager",
        "Grakmaw's second ability puts just one +1/+1 counter on it, no matter how many counters the dying creature had."
    );
    supported("Generous Pup");
    supported("Grakmaw, Skyclave Ravager");
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Generous Pup");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, pup, 3);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);

    let mut t = TestGame::new(2);
    let grakmaw = with_counters(&mut t, P0, "Grakmaw, Skyclave Ravager", 3);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 3);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(plus1(&t, grakmaw), 4);
}

// --- Cursed Wombat ---------------------------------------------------------------------

#[test]
fn cursed_wombat_grants_its_ability_to_noncreature_permanents() {
    cr!("613.1f", "122.6");
    ruling!(
        "Cursed Wombat",
        "Cursed Wombat's last ability grants a triggered ability to itself as well as other permanents you control, even if those permanents don't have power and toughness."
    );
    supported("Cursed Wombat");
    let mut t = TestGame::new(2);
    let wombat = t.battlefield(P0, "Cursed Wombat");
    let forest = t.battlefield(P0, "Forest");
    give_plus1(&mut t, forest, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, forest), 2);
    give_plus1(&mut t, wombat, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, wombat), 2);
}

#[test]
fn a_new_cursed_wombat_grants_a_new_ability() {
    cr!("400.7", "603.2h");
    ruling!(
        "Cursed Wombat",
        "If Cursed Wombat leaves the battlefield and returns to the battlefield in the same turn, or if one Cursed Wombat leaves and another Cursed Wombat appears, the ability granted by the old one is different than the ability granted by the new one."
    );
    let mut t = TestGame::new(2);
    let wombat = t.battlefield(P0, "Cursed Wombat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    // Only once each turn.
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3);
    // Another Cursed Wombat: a new ability.
    destroy(&mut t, wombat);
    t.battlefield(P0, "Cursed Wombat");
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 5);
}

#[test]
fn cursed_wombat_leaving_before_the_counters_are_put_grants_nothing() {
    cr!("611.3a", "603.2");
    ruling!(
        "Cursed Wombat",
        "If Cursed Wombat leaves the battlefield while a spell or ability that would put one or more +1/+1 counters on one or more permanents you control is still on the stack, those permanents won't have the additional ability granted by Cursed Wombat when those counters would be put on those permanents, so that additional ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let wombat = t.battlefield(P0, "Cursed Wombat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_slots(&mut t, P0, "Travel Preparations", &[&[obj(bears)]]);
    destroy(&mut t, wombat);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn cursed_wombat_leaving_after_the_granted_ability_triggered() {
    cr!("113.7a");
    ruling!(
        "Cursed Wombat",
        "If Cursed Wombat leaves the battlefield while one or more triggered abilities generated by the ability it grants are still on the stack, those abilities will still resolve as normal and put additional +1/+1 counters on the appropriate permanents."
    );
    let mut t = TestGame::new(2);
    let wombat = t.battlefield(P0, "Cursed Wombat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(stack(&t), 1);
    destroy(&mut t, wombat);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
}

// --- life gain ---------------------------------------------------------------------------

#[test]
fn life_gain_triggers_once_per_life_gaining_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Aerith Gainsborough",
        "Aerith Gainsborough's second ability triggers just once for each life-gaining event, whether it's 1 life from Al Bhed Salvagers or 3 life from Balamb T-Rexaur."
    );
    ruling!(
        "Exemplar of Light",
        "Exemplar of Light's second ability triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    for (name, life) in [("Aerith Gainsborough", 3), ("Exemplar of Light", 4)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        gain_life(&mut t, P0, life);
        t.resolve_all();
        assert_eq!(plus1(&t, c), 1, "{name}");
    }
}

/// `name` is on P0's battlefield; two Vampire Nighthawks attack unblocked, then one
/// Nighthawk attacks and is blocked by two Bears. Returns the counters put each time.
fn lifelink_events(name: &str) -> (u32, u32) {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    let two = plus1(&t, c);

    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 1]));
    block_and_finish(&mut t, P1, &[(x, a), (y, a)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    (two, plus1(&t, c))
}

#[test]
fn each_lifelink_creature_dealing_combat_damage_is_a_separate_life_gain() {
    cr!("119.9", "702.15b", "510.2");
    ruling!(
        "Exemplar of Light",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Exemplar of Light's second ability will trigger twice."
    );
    ruling!(
        "Voice of the Blessed",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Voice of the Blessed's first ability will trigger twice."
    );
    supported("Voice of the Blessed");
    assert_eq!(lifelink_events("Exemplar of Light"), (2, 1));
    assert_eq!(lifelink_events("Voice of the Blessed"), (2, 1));
}

#[test]
fn a_counter_from_life_gained_with_lethal_damage_comes_too_late() {
    cr!("704.3", "603.3", "510.2");
    ruling!(
        "Exemplar of Light",
        "If Exemplar of Light is dealt lethal damage at the same time that you gain life, it won't receive a counter from its second ability in time to save it."
    );
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Exemplar of Light");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    // 3 damage: lethal to the 3/3 Exemplar, but not to the 4/4 it would become.
    let monster = t.battlefield(P1, "Phantom Monster");
    attack_with(
        &mut t,
        &[(ex, Entity::Player(P1)), (hawk, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(monster, ex)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Exemplar of Light"));
}

#[test]
fn aerith_dealt_lethal_damage_while_gaining_life_uses_its_last_counters() {
    cr!("608.2h", "603.10a", "510.2");
    ruling!(
        "Aerith Gainsborough",
        "If Aerith Gainsborough is dealt lethal damage at the same time that you gain life, it will die before its second ability would resolve. Its last ability will use the number of counters that were on it when it was last on the battlefield."
    );
    let mut t = TestGame::new(2);
    let aerith = with_counters(&mut t, P0, "Aerith Gainsborough", 1);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(aerith, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, aerith)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P0, "Aerith Gainsborough"));
    assert_eq!(plus1(&t, isamaru), 1);
}

#[test]
fn voice_of_the_blessed_gaining_flying_after_being_blocked_stays_blocked() {
    cr!("509.1h", "702.9b");
    ruling!(
        "Voice of the Blessed",
        "Gaining flying after it has become blocked won't cause Voice of the Blessed to become unblocked."
    );
    let mut t = TestGame::new(2);
    let voice = with_counters(&mut t, P0, "Voice of the Blessed", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(voice, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, voice)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    gain_life(&mut t, P0, 1);
    t.resolve_all();
    assert!(has_kw(
        &mut t,
        voice,
        mtg_engine::keywords::KeywordKind::Flying
    ));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// --- casting triggers resolve first ---------------------------------------------------------

#[test]
fn angelic_cub_trigger_resolves_before_the_spell_targeting_it() {
    cr!("603.3", "405.5");
    ruling!(
        "Angelic Cub",
        "Angelic Cub's first ability will resolve before the spell or ability that caused it to trigger."
    );
    supported("Angelic Cub");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Angelic Cub");
    cast_new(&mut t, P0, "Giant Growth", &[obj(cub)]);
    t.settle();
    assert_eq!(stack(&t), 2);
    t.resolve();
    assert_eq!(plus1(&t, cub), 1);
    assert_eq!(stack(&t), 1, "Giant Growth is still on the stack");
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Animar, Soul of Elements",
        "Animar's triggered ability resolves before the creature spell that causes it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Gleeful Arsonist",
        "Gleeful Arsonist's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Hallar, the Firefletcher",
        "Hallar's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Animar, Soul of Elements");
    supported("Gleeful Arsonist");
    supported("Hallar, the Firefletcher");
    // Animar.
    let mut t = TestGame::new(2);
    let animar = t.battlefield(P0, "Animar, Soul of Elements");
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    counter_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(plus1(&t, animar), 1);
    // Gleeful Arsonist: an opponent's noncreature spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gleeful Arsonist");
    let spell = cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    counter_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Hallar: a kicked spell.
    let mut t = TestGame::new(2);
    let hallar = t.battlefield(P0, "Hallar, the Firefletcher");
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_new(&mut t, P0, "Burst Lightning", &[obj(bears)]);
    t.settle();
    counter_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(plus1(&t, hallar), 1);
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(bears));
}

#[test]
fn animars_counter_for_a_spell_doesnt_reduce_that_spells_cost() {
    cr!("601.2f", "603.3");
    ruling!(
        "Animar, Soul of Elements",
        "Animar's triggered ability triggers only when a creature spell is cast, after costs are paid. The counter put on Animar for a creature spell won't affect the cost of that creature spell, only future ones."
    );
    let mut t = TestGame::new(2);
    let animar = t.battlefield(P0, "Animar, Soul of Elements");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(
        t.cast(P0, bears).try_go().is_err(),
        "{{1}}{{G}} isn't reduced"
    );
    give_plus1(&mut t, animar, 1);
    assert!(t.cast(P0, bears).try_go().is_ok());
}

#[test]
fn forgotten_ancient_counter_is_optional_and_comes_first() {
    cr!("603.3", "603.5");
    ruling!(
        "Forgotten Ancient",
        "Forgotten Ancient's first ability will resolve before the spell that caused it to trigger. Putting a +1/+1 counter on Forgotten Ancient is optional."
    );
    supported("Forgotten Ancient");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let ancient = t.battlefield(P0, "Forgotten Ancient");
        cast_new(&mut t, P1, "Opt", &[]);
        t.answer_yes(P0, yes);
        t.resolve();
        assert_eq!(stack(&t), 1, "Divination hasn't resolved");
        assert_eq!(plus1(&t, ancient), u32::from(yes));
    }
}

#[test]
fn gleeful_arsonist_that_left_uses_its_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Gleeful Arsonist",
        "If Gleeful Arsonist is no longer on the battlefield when its first ability resolves, use its power as it last existed on the battlefield to determine how much damage it deals."
    );
    let mut t = TestGame::new(2);
    let arsonist = with_counters(&mut t, P0, "Gleeful Arsonist", 1);
    cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    destroy(&mut t, arsonist);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn hallar_that_left_deals_damage_for_its_last_counters() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Hallar, the Firefletcher",
        "If Hallar leaves the battlefield after its last ability has triggered but before it resolves, you don't put a +1/+1 counter on anything as the ability resolves, but you do use the number of +1/+1 counters that were on Hallar before it left the battlefield to determine how much damage it deals to each opponent."
    );
    let mut t = TestGame::new(2);
    let hallar = with_counters(&mut t, P0, "Hallar, the Firefletcher", 2);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Burst Lightning", &[obj(bears)]);
    t.settle();
    destroy(&mut t, hallar);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- dying with counters -----------------------------------------------------------------

#[test]
fn dies_with_a_counter_triggers_for_each_creature_dying_at_once_including_itself() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Basri's Lieutenant",
        "If Basri's Lieutenant dies at the same time as other creatures, its last ability will trigger for any of the creatures that had a +1/+1 counter on it, including itself."
    );
    ruling!(
        "Meltstrider Eulogist",
        "If Meltstrider Eulogist dies at the same time as one or more creatures you control with +1/+1 counters on them, its ability will trigger for each of those creatures. This includes Meltstrider Eulogist itself if it had a +1/+1 counter on it when it died."
    );
    ruling!(
        "Rayblade Trooper",
        "If Rayblade Trooper dies at the same time as one or more creatures you control with +1/+1 counters on them, its second ability will trigger for each of those creatures. This includes Rayblade Trooper itself if it had a +1/+1 counter on it when it died."
    );
    ruling!(
        "Gladehart Cavalry",
        "If Gladehart Cavalry dies at the same time as a creature you control with a +1/+1 counter on it, its ability will trigger."
    );
    // (card, triggers expected: dying with a counter itself and one Bears with a counter)
    let cases = [
        "Basri's Lieutenant",
        "Meltstrider Eulogist",
        "Rayblade Trooper",
        "Gladehart Cavalry",
    ];
    for name in cases {
        supported(name);
        let mut t = TestGame::new(2);
        with_counters(&mut t, P0, name, 1);
        with_counters(&mut t, P0, "Grizzly Bears", 1);
        t.battlefield(P0, "Elite Vanguard");
        wrath(&mut t);
        assert_eq!(stack(&t), 2, "{name}");
    }
    // The outcome for each.
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Basri's Lieutenant", 1);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    wrath(&mut t);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Knight"), 2);
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Gladehart Cavalry", 0);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    wrath(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn self_dying_with_a_counter_triggers_its_own_ability() {
    cr!("603.10a");
    ruling!(
        "Alharu, Solemn Ritualist",
        "If Alharu has a +1/+1 counter on it and dies, its ability triggers for itself."
    );
    ruling!(
        "Gladehart Cavalry",
        "If Gladehart Cavalry somehow gets a +1/+1 counter, it dying will cause its own ability to trigger."
    );
    ruling!(
        "Marchesa, the Black Rose",
        "If Marchesa has a +1/+1 counter on it when it dies, it will return to the battlefield under your control because of its own ability."
    );
    supported("Alharu, Solemn Ritualist");
    supported("Marchesa, the Black Rose");
    let t = {
        let mut t = TestGame::new(2);
        let a = with_counters(&mut t, P0, "Alharu, Solemn Ritualist", 1);
        destroy(&mut t, a);
        t.resolve_all();
        t
    };
    assert_eq!(tokens_named(&t, P0, "Spirit"), 1);
    let mut t = TestGame::new(2);
    let c = with_counters(&mut t, P0, "Gladehart Cavalry", 1);
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    let mut t = TestGame::new(2);
    let m = with_counters(&mut t, P0, "Marchesa, the Black Rose", 1);
    destroy(&mut t, m);
    t.resolve_all();
    end_step(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Marchesa, the Black Rose").len(), 1);
}

#[test]
fn bone_devourer_counts_the_counters_it_had_as_it_died() {
    cr!("704.5f", "704.5q", "603.10a");
    ruling!(
        "Bone Devourer",
        "If Bone Devourer has +1/+1 counters put on it, and then enough -1/-1 counters are put on it to cause its toughness to be 0, X is equal to the number of +1/+1 counters it had at the time those -1/-1 counters were put on it."
    );
    supported("Bone Devourer");
    let mut t = TestGame::new(2);
    let bd = with_counters(&mut t, P0, "Bone Devourer", 2);
    let hand = t.hand_size(P0);
    give_minus1(&mut t, bd, 4);
    assert!(t.in_graveyard(P0, "Bone Devourer"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn iron_apprentice_puts_its_minus_counters_too() {
    cr!("603.10a", "704.5q");
    ruling!(
        "Iron Apprentice",
        "If Iron Apprentice has -1/-1 counters on it when it dies, the last ability will include those as well. This may result in the recipient also dying."
    );
    supported("Iron Apprentice");
    let mut t = TestGame::new(2);
    let ia = with_counters(&mut t, P0, "Iron Apprentice", 1);
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    t.answer_targets(P0, &[obj(vanguard)]);
    give_minus1(&mut t, ia, 2);
    assert!(t.in_graveyard(P0, "Iron Apprentice"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Elite Vanguard"), "net -1/-1 on a 2/1");
}

#[test]
fn enduring_bondwarden_puts_each_kind_of_counter_it_had() {
    cr!("603.10a", "122.1");
    ruling!(
        "Enduring Bondwarden",
        "Enduring Bondwarden’s last ability doesn’t cause you to move counters from the creature with the ability onto the target creature. Rather, you put the same number of each kind of counter the first creature had when it died onto the target creature."
    );
    ruling!(
        "Enduring Bondwarden",
        "Enduring Bondwarden’s last ability puts counters that were on the creature with the ability onto the target creature, not just its +1/+1 counters."
    );
    supported("Enduring Bondwarden");
    let mut t = TestGame::new(2);
    let bw = with_counters(&mut t, P0, "Enduring Bondwarden", 2);
    put_counters(&mut t, bw, "flying", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, bw);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.counters(bears, "flying"), 1);
    assert!(has_kw(
        &mut t,
        bears,
        mtg_engine::keywords::KeywordKind::Flying
    ));
}

#[test]
fn ribtruss_roaster_that_left_uses_its_last_counters() {
    cr!("608.2h");
    ruling!(
        "Ribtruss Roaster",
        "If Ribtruss Roaster is no longer on the battlefield when its last ability resolves, use the number of +1/+1 counters that were on it as it last existed on the battlefield to determine how many tokens to create."
    );
    supported("Ribtruss Roaster");
    let mut t = TestGame::new(2);
    let r = with_counters(&mut t, P0, "Ribtruss Roaster", 2);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stack(&t), 1);
    destroy(&mut t, r);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Pest"), 2);
}

#[test]
fn anim_pakal_that_left_still_creates_gnomes_for_its_last_counters() {
    cr!("608.2h");
    ruling!(
        "Anim Pakal, Thousandth Moon",
        "If Anim Pakal is no longer on the battlefield when the triggered ability resolves, you'll still create Gnomes. Use the number of +1/+1 counters that were on it when it was last on the battlefield."
    );
    supported("Anim Pakal, Thousandth Moon");
    let mut t = TestGame::new(2);
    let pakal = with_counters(&mut t, P0, "Anim Pakal, Thousandth Moon", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    destroy(&mut t, pakal);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Gnome"), 2);
}

#[test]
fn anim_pakal_gnomes_entering_attacking_never_attacked() {
    cr!("508.4", "508.3a");
    ruling!(
        "Anim Pakal, Thousandth Moon",
        "Although the Gnome tokens enter the battlefield as attacking creatures, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when they enter the battlefield attacking."
    );
    ruling!(
        "Anim Pakal, Thousandth Moon",
        "Anim Pakal, Thousandth Moon doesn't have to be one of the non-Gnome creatures you attack with in order for its ability to trigger, but it can be."
    );
    supported("Campaign of Vengeance");
    let mut t = TestGame::new(2);
    let pakal = t.battlefield(P0, "Anim Pakal, Thousandth Moon");
    t.battlefield(P0, "Campaign of Vengeance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(plus1(&t, pakal), 1);
    assert_eq!(tokens_named(&t, P0, "Gnome"), 1);
    // Only the Bears' attack triggered Campaign of Vengeance.
    assert_eq!(t.life(P1), 19);
}

#[test]
fn primal_vigor_extra_tokens_are_also_tapped_and_attacking() {
    cr!("614.1a", "111.1");
    ruling!(
        "Primal Vigor",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens created by Primal Vigor's replacement effect."
    );
    supported("Primal Vigor");
    let mut t = TestGame::new(2);
    let pakal = t.battlefield(P0, "Anim Pakal, Thousandth Moon");
    t.battlefield(P0, "Primal Vigor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(plus1(&t, pakal), 2, "the counter is doubled too");
    let gnomes: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Gnome"))
            .map(|o| o.id)
            .collect();
    assert_eq!(gnomes.len(), 4);
    for g in gnomes {
        assert!(t.obj_now(g).tapped);
        assert!(t.g.combat.as_ref().unwrap().attacker(g).is_some());
    }
}

// --- leaving before resolution -------------------------------------------------------------

#[test]
fn thought_gorger_that_left_doesnt_discard() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Thought Gorger",
        "As Thought Gorger's first ability resolves, putting a +1/+1 counter on it for each card in your hand is mandatory. The effect says \"if you do\" because performing that action might be impossible. If Thought Gorger has left the battlefield by the time the ability resolves, you can't put any +1/+1 counters on it, so you won't discard your hand."
    );
    supported("Thought Gorger");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    let g = t.enter(P0, "Thought Gorger");
    t.settle();
    assert_eq!(stack(&t), 1);
    destroy(&mut t, g);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // On the battlefield: it gets 3 counters and you discard your hand.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    let g = t.enter(P0, "Thought Gorger");
    t.resolve_all();
    assert_eq!(plus1(&t, g), 3);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn clockwork_hydra_that_left_deals_no_damage() {
    cr!("608.2c");
    ruling!(
        "Clockwork Hydra",
        "If Clockwork Hydra leaves the battlefield while its trigger is on the stack, you can't remove a +1/+1 counter from it, so it won't deal damage."
    );
    supported("Clockwork Hydra");
    let mut t = TestGame::new(2);
    let h = with_counters(&mut t, P0, "Clockwork Hydra", 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn noosegraf_mob_without_a_counter_to_remove_makes_no_zombie() {
    cr!("608.2c");
    ruling!(
        "Noosegraf Mob",
        "If Noosegraf Mob is no longer on the battlefield or no longer has +1/+1 counters on it as its triggered ability resolves, you can’t remove a counter from it and won’t get a Zombie token."
    );
    supported("Noosegraf Mob");
    // Gone.
    let mut t = TestGame::new(2);
    let mob = with_counters(&mut t, P0, "Noosegraf Mob", 5);
    cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    destroy(&mut t, mob);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Zombie"), 0);
    // No counters (kept alive by Glorious Anthem).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let mob = t.battlefield(P0, "Noosegraf Mob");
    assert!(t.on_battlefield(mob));
    cast_new(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Zombie"), 0);
}

#[test]
fn ayaras_oathsworn_with_four_counters_or_gone() {
    cr!("603.4", "608.2c");
    ruling!(
        "Ayara's Oathsworn",
        "If Ayara's Oathsworn has four or more +1/+1 counters on it as it deals combat damage to a player, its ability won't trigger at all."
    );
    ruling!(
        "Ayara's Oathsworn",
        "If Ayara's Oathsworn isn't on the battlefield as its ability tries to resolve, you won't search for a card, even if Ayara's Oathsworn had three +1/+1 counters on it before if left the battlefield"
    );
    supported("Ayara's Oathsworn");
    let mut t = TestGame::new(2);
    let a = with_counters(&mut t, P0, "Ayara's Oathsworn", 4);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 14);
    assert_eq!(stack(&t), 0);
    assert_eq!(plus1(&t, a), 4);
    // Three counters, gone before it resolves.
    let mut t = TestGame::new(2);
    let a = with_counters(&mut t, P0, "Ayara's Oathsworn", 3);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(stack(&t), 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn constable_that_left_exiles_nothing() {
    cr!("610.3c", "608.2b");
    ruling!(
        "Constable of the Realm",
        "If Constable of the Realm leaves the battlefield before its last triggered ability resolves, the target permanent won't be exiled."
    );
    supported("Constable of the Realm");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Constable of the Realm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    give_plus1(&mut t, c, 1);
    assert_eq!(stack(&t), 1);
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn constable_exiled_permanents_return_together_under_their_owners() {
    cr!("610.3a", "610.3c", "303.4c", "301.5c", "122.2");
    ruling!(
        "Constable of the Realm",
        "All permanents exiled by Constable of the Realm will return to the battlefield at the same time, and each one will enter under its owner's control."
    );
    ruling!(
        "Constable of the Realm",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to it will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Constable of the Realm");
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 1);
    let mine = t.battlefield(P0, "Hill Giant");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.attach(pacifism, obj(bears));
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    t.answer_targets(P0, &[obj(bears)]);
    give_plus1(&mut t, c, 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert!(t.on_battlefield(sword));
    assert!(t.obj_now(sword).attached_to.is_none());
    t.answer_targets(P0, &[obj(mine)]);
    give_plus1(&mut t, c, 1);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    destroy(&mut t, c);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!((b.len(), g.len()), (1, 1));
    assert_eq!(t.obj_now(b[0]).controller, P1);
    assert_eq!(t.obj_now(g[0]).controller, P0);
    assert_eq!(plus1(&t, b[0]), 0);
}
