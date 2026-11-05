//! Rulings batch P034 — statics that look at counters, counters on permanents that die
//! or leave (CR 122.2, 122.8), phasing (CR 702.26) and the remaining "counters matter"
//! rulings.

use crate::r_p034_common::*;
use crate::r_p038_common::{add, kill, phased_out};
use crate::r_s01_common::{supported, tokens, watch};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

fn minus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::MINUS1)
}

#[test]
fn hancock_gives_a_zombie_mutant_its_bonus_once() {
    cr!("613.4c");
    ruling!(
        "Hancock, Ghoulish Mayor",
        "A creature that's both a Zombie and a Mutant will get +X/+X only once"
    );
    supported("Hancock, Ghoulish Mayor");
    supported("Twisted Abomination");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hancock, Ghoulish Mayor");
    put(&mut t, h, counters::PLUS1, 1);
    put(&mut t, h, "stun", 1);
    let ta = t.battlefield(P0, "Twisted Abomination");
    assert_eq!(t.pt(ta), (7, 5), "5/3 + 2/2");
    let theirs = t.battlefield(P1, "Twisted Abomination");
    assert_eq!(t.pt(theirs), (5, 3));
}

#[test]
fn spark_rupture_planeswalkers_lose_all_abilities_and_can_gain_new_ones() {
    cr!("613.1f", "613.4b", "613.7");
    ruling!(
        "Spark Rupture",
        "loses all abilities it has, not only loyalty abilities. The resulting creature may gain abilities after Spark Rupture"
    );
    supported("Spark Rupture");
    supported("Jace Beleren");
    supported("Jump");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.battlefield(P0, "Spark Rupture");
    assert!(t.obj_now(jace).is(CardType::Creature));
    assert_eq!(t.pt(jace), (3, 3));
    assert!(t.obj_now(jace).chars.abilities.is_empty());
    cast_card(&mut t, P0, "Jump", &[obj(jace)]);
    t.resolve_all();
    assert!(t.obj_now(jace).has_keyword(KeywordKind::Flying));
}

#[test]
fn spark_rupture_stops_applying_once_the_loyalty_counters_are_gone() {
    cr!("704.5i", "613.1d");
    ruling!(
        "Spark Rupture",
        "If a planeswalker-turned-creature has all loyalty counters removed from it, Spark Rupture's effect will stop applying to it."
    );
    supported("Vampire Hexmage");
    supported("Glorious Anthem");
    // With Glorious Anthem, a 0/0 creature would survive as a 1/1: it's a planeswalker
    // with no loyalty instead.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.battlefield(P0, "Spark Rupture");
    t.battlefield(P1, "Glorious Anthem");
    assert_eq!(t.pt(jace), (4, 4));
    let hm = t.battlefield(P0, "Vampire Hexmage");
    t.activate(P0, hm, 0, &[obj(jace)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
}

#[test]
fn backstreet_bruiser_counts_individual_counters_among_your_creatures() {
    cr!("702.3b", "122.1");
    ruling!(
        "Backstreet Bruiser",
        "Backstreet Bruiser's ability looks for two or more individual counters."
    );
    supported("Backstreet Bruiser");
    // (counters on P0's creatures, can it attack)
    let cases: &[(&[(usize, &str)], bool)] = &[
        (&[], false),
        (&[(1, counters::PLUS1)], false),
        (&[(1, counters::PLUS1), (1, counters::PLUS1)], true),
        (&[(1, counters::PLUS1), (2, "stun")], true),
        (&[(0, counters::PLUS1), (1, counters::MINUS1)], true),
    ];
    for (counters_on, attacks) in cases {
        let mut t = TestGame::new(2);
        let bb = t.battlefield(P0, "Backstreet Bruiser");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        put(&mut t, theirs, counters::PLUS1, 2);
        for (i, k) in counters_on.iter() {
            put(&mut t, [bb, a, b][*i], k, 1);
        }
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(t.g.can_attack(bb), *attacks, "{counters_on:?}");
    }
}

#[test]
fn hazardous_conditions_ignores_creatures_with_any_counter() {
    cr!("122.1");
    ruling!(
        "Hazardous Conditions",
        "any kind of counter causes a creature to be unaffected"
    );
    supported("Hazardous Conditions");
    let mut t = TestGame::new(2);
    let plain = t.battlefield(P1, "Hill Giant");
    let fate = t.battlefield(P1, "Hill Giant");
    put(&mut t, fate, "fate", 1);
    let charged = t.battlefield(P0, "Hill Giant");
    put(&mut t, charged, counters::CHARGE, 1);
    let m1 = t.battlefield(P1, "Craw Wurm");
    put(&mut t, m1, counters::MINUS1, 1);
    cast_card(&mut t, P0, "Hazardous Conditions", &[]);
    t.resolve_all();
    assert_eq!(t.pt(plain), (1, 1));
    assert_eq!(t.pt(fate), (3, 3));
    assert_eq!(t.pt(charged), (3, 3));
    assert_eq!(t.pt(m1), (5, 3));
}

#[test]
fn chimeric_mass_reactivated_overrides_an_earlier_set_effect() {
    cr!("613.4b", "613.7", "613.4c");
    ruling!(
        "Chimeric Mass",
        "Activating Chimeric Mass’s last ability a second time would make it 6/6 again until end of turn."
    );
    supported("Chimeric Mass");
    supported("Diminish");
    let mut t = TestGame::new(2);
    let cm = t.battlefield(P0, "Chimeric Mass");
    put(&mut t, cm, counters::CHARGE, 3);
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, cm, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(cm), (3, 3));
    cast_card(&mut t, P0, "Giant Growth", &[obj(cm)]);
    t.resolve_all();
    assert_eq!(t.pt(cm), (6, 6));
    cast_card(&mut t, P1, "Diminish", &[obj(cm)]);
    t.resolve_all();
    assert_eq!(t.pt(cm), (4, 4));
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, cm, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(cm), (6, 6));
}

#[test]
fn ezio_losing_first_strike_after_first_strike_damage_deals_no_regular_damage() {
    cr!("510.4");
    ruling!(
        "Ezio, Brash Novice",
        "it won't also deal regular combat damage (unless it somehow has double strike)"
    );
    supported("Ezio, Brash Novice");
    let mut t = TestGame::new(2);
    let ezio = t.battlefield(P0, "Ezio, Brash Novice");
    put(&mut t, ezio, counters::PLUS1, 1);
    crate::r_s01_common::attack_with(&mut t, &[(ezio, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.obj_now(ezio).has_keyword(KeywordKind::FirstStrike));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 17);
    // A -1/-1 counter: one +1/+1 counter left, no first strike.
    put(&mut t, ezio, counters::MINUS1, 1);
    assert!(!t.obj_now(ezio).has_keyword(KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn quarantine_field_exile_and_return() {
    cr!("610.3d", "122.2", "704.5m", "704.5n");
    ruling!(
        "Quarantine Field",
        "Auras attached to the exiled permanents will be put into their owners’ graveyards."
    );
    ruling!(
        "Quarantine Field",
        "If Quarantine Field exiles multiple permanents, those cards all return to the battlefield at the same time."
    );
    supported("Soul Warden");
    supported("Holy Strength");
    supported("Bonesplitter");
    // Two Soul Wardens exiled and returned together: each sees the other enter.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Soul Warden");
    let b = t.battlefield(P1, "Soul Warden");
    put(&mut t, a, counters::PLUS1, 1);
    let hs = t.battlefield(P1, "Holy Strength");
    t.g.attach(hs, obj(a));
    let bs = t.battlefield(P1, "Bonesplitter");
    t.g.attach(bs, obj(b));
    t.g.recompute();
    pool(&mut t, P0, ManaType::W, 2);
    pool(&mut t, P0, ManaType::C, 4);
    let qf = t.hand(P0, "Quarantine Field");
    t.cast(P0, qf).x(2).go();
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.on_battlefield(bs));
    assert_eq!(t.obj_now(bs).attached_to, None);
    let field = t.named_on_battlefield("Quarantine Field")[0];
    let life = t.life(P1);
    kill(&mut t, field);
    t.resolve_all();
    let wardens = t.named_on_battlefield("Soul Warden");
    assert_eq!(wardens.len(), 2);
    assert!(wardens.iter().all(|w| total_counters(&t, *w) == 0));
    assert_eq!(t.life(P1), life + 2, "each Soul Warden saw the other");
}

#[test]
fn counters_of_a_dying_permanent_are_counted_by_kind() {
    cr!("122.8", "603.10a");
    ruling!(
        "Yuna, Grand Summoner",
        "Yuna's last ability counts all counters that were on the permanent, not just +1/+1 counters."
    );
    ruling!(
        "Dockworker Drone",
        "Dockworker Drone’s second ability puts all counters that were on Dockworker Drone onto the target creature"
    );
    ruling!(
        "Dockworker Drone",
        "Rather, you put the same number of each kind of counter Dockworker Drone had when it died onto the target creature."
    );
    supported("Yuna, Grand Summoner");
    supported("Dockworker Drone");
    // Yuna: Bears with a +1/+1 counter and two stun counters dies: three +1/+1 counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yuna, Grand Summoner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    put(&mut t, bears, "stun", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 3);
    // Dockworker Drone with two +1/+1 counters and a stun counter.
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dockworker Drone");
    put(&mut t, dd, counters::PLUS1, 2);
    put(&mut t, dd, "stun", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, dd);
    t.resolve_all();
    assert_eq!((plus1(&t, giant), t.counters(giant, "stun")), (2, 1));
}

#[test]
fn dockworker_drone_passes_on_its_minus1_counters_too() {
    cr!("122.8", "704.5f", "704.5q", "704.3");
    ruling!(
        "Dockworker Drone",
        "If Dockworker Drone has -1/-1 counters on it when it dies, that ability will include those as well."
    );
    ruling!(
        "Dockworker Drone",
        "its second ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters"
    );
    supported("Simic Ascendancy");
    // Only a -1/-1 counter: the Llanowar Elves it goes to dies.
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dockworker Drone");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[obj(elves)]);
    put(&mut t, dd, counters::MINUS1, 1);
    assert!(!t.on_battlefield(dd));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // One +1/+1 counter, then two -1/-1 counters at once: it dies with all three (the
    // annihilation of 704.5q happens at the same time), so the Hill Giant gets a +1/+1
    // counter (Simic Ascendancy sees it) and two -1/-1 counters.
    let mut t = TestGame::new(2);
    let sa = t.battlefield(P0, "Simic Ascendancy");
    let dd = t.battlefield(P0, "Dockworker Drone");
    put(&mut t, dd, counters::PLUS1, 1);
    t.resolve_all();
    assert_eq!(t.counters(sa, "growth"), 1);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    put(&mut t, dd, counters::MINUS1, 2);
    assert!(!t.on_battlefield(dd));
    t.resolve_all();
    assert_eq!(t.counters(sa, "growth"), 2, "a +1/+1 counter was put on the Giant");
    assert_eq!((plus1(&t, giant), minus1(&t, giant)), (0, 1));
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn dockworker_drone_and_the_ozolith_both_get_the_counters() {
    cr!("122.8", "603.10a");
    ruling!(
        "Dockworker Drone",
        "you’ll put the appropriate number of each kind of counter onto both The Ozolith and the target creature"
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let oz = t.battlefield(P0, "The Ozolith");
    let dd = t.battlefield(P0, "Dockworker Drone");
    put(&mut t, dd, counters::PLUS1, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, dd);
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 2);
    assert_eq!(plus1(&t, oz), 2);
}

#[test]
fn dismantle_chooses_the_kind_and_artifact_on_resolution() {
    cr!("608.2d");
    ruling!(
        "Dismantle",
        "When Dismantle resolves, you choose which type of counters you want and choose an artifact you control"
    );
    supported("Dismantle");
    for (choice, kind) in [(0usize, counters::PLUS1), (1, counters::CHARGE)] {
        let mut t = TestGame::new(2);
        let ms = t.battlefield(P1, "Mind Stone");
        put(&mut t, ms, counters::CHARGE, 3);
        let orni = t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Mind Stone");
        let from = t.asked().len();
        cast_card(&mut t, P0, "Dismantle", &[obj(ms)]);
        // Nothing is chosen while casting beyond the target.
        assert!(!crate::r_s01_common::asked_since(&t, from)
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
        let pick = t.asked().len();
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.answer_choose(P0, &[obj(orni)]);
        t.resolve_all();
        assert!(!t.on_battlefield(ms));
        assert_eq!(t.counters(orni, kind), 3, "{kind}");
        assert!(crate::r_s01_common::asked_since(&t, pick).len() >= 2);
    }
}

#[test]
fn descent_into_madness_players_choose_in_turn_order_then_exile_together() {
    cr!("101.4", "608.2h");
    ruling!(
        "Descent into Madness",
        "Then all the chosen permanents and cards are exiled simultaneously."
    );
    ruling!(
        "Descent into Madness",
        "use the number of counters on it when it left the battlefield to determine how many permanents"
    );
    supported("Descent into Madness");
    // One despair counter, destroyed in response: each player exiles one.
    let mut t = TestGame::new(2);
    let dim = add(&mut t, P0, "Descent into Madness");
    put(&mut t, dim, "despair", 1);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let mine2 = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let theirs2 = t.battlefield(P1, "Hill Giant");
    crate::r_p038_common::next_p0_upkeep(&mut t);
    t.settle();
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    kill(&mut t, dim);
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(theirs)]);
    let before = t.g.battlefield.len();
    t.resolve_all();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert!(t.on_battlefield(mine2) && t.on_battlefield(theirs2));
    // P1 chose after P0 did, with P0's choice still on the battlefield.
    assert_eq!(seen.lock().unwrap().clone(), vec![before]);
}

#[test]
fn rottenmouth_viper_opponent_may_always_lose_4_life() {
    cr!("118.12a");
    ruling!(
        "Rottenmouth Viper",
        "An opponent can always choose not to sacrifice a nonland permanent or discard a card"
    );
    supported("Rottenmouth Viper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    // P1 chooses nothing to sacrifice or discard, though it could.
    t.answer_choose(P1, &[]);
    crate::r_p038_common::enter(&mut t, P0, "Rottenmouth Viper");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Hand(P1));
}

#[test]
fn thief_of_blood_removes_loyalty_but_not_player_counters() {
    cr!("122.1", "704.5i", "614.1c");
    ruling!(
        "Thief of Blood",
        "Experience counters aren't removed from players this way, but loyalty counters are removed from planeswalkers."
    );
    supported("Thief of Blood");
    let mut t = TestGame::new(2);
    put(&mut t, Entity::Player(P0), "experience", 2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    let thief = t.enter(P0, "Thief of Blood");
    t.g.flush_events();
    t.settle();
    assert_eq!(plus1(&t, thief), 4, "three loyalty and one +1/+1");
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    let _ = jace;
    assert_eq!(t.g.player(P0).counter("experience"), 2);
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn nissa_with_x_0_dies_before_her_abilities_can_be_activated() {
    cr!("306.5b", "107.3m", "704.5i");
    ruling!(
        "Nissa, Steward of Elements",
        "If X is 0, Nissa enters the battlefield with no loyalty"
    );
    let mut t = TestGame::new(2);
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    let nissa = t.hand(P0, "Nissa, Steward of Elements");
    t.cast(P0, nissa).x(0).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Nissa, Steward of Elements"));
    // With X = 2, she enters with two loyalty counters (her printed loyalty is X, CR
    // 107.3m; the engine used to give her none).
    let mut t = TestGame::new(2);
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 2);
    let nissa = t.hand(P0, "Nissa, Steward of Elements");
    t.cast(P0, nissa).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(nissa, counters::LOYALTY), 2);
}

#[test]
fn a_phased_out_token_phases_in_and_triggers_king_of_the_oathbreakers() {
    cr!("702.26a", "702.26b");
    ruling!(
        "King of the Oathbreakers",
        "A phased-out token will phase in at the beginning of its controller's untap step"
    );
    supported("King of the Oathbreakers");
    supported("Lingering Souls");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "King of the Oathbreakers");
    cast_card(&mut t, P0, "Lingering Souls", &[]);
    t.resolve_all();
    let spirits = tokens(&t, P0);
    assert_eq!(spirits.len(), 2);
    let target = spirits[0];
    cast_card(&mut t, P1, "Giant Growth", &[obj(target)]);
    t.resolve_all();
    assert!(phased_out(&t, target));
    t.advance_to(P1, Step::Upkeep);
    assert!(phased_out(&t, target));
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, target));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3, "a new tapped Spirit");
}

#[test]
fn for_as_long_as_effects_end_when_their_object_phases_out() {
    cr!("702.26f", "611.2b");
    ruling!(
        "Galadriel's Dismissal",
        "Any continuous effects with a \"for as long as\" duration ignore phased-out objects."
    );
    ruling!(
        "Talon Gates of Madara",
        "Any continuous effects with a \"for as long as\" duration ignore phased-out objects."
    );
    ruling!(
        "Ripples of Potential",
        "Any continuous effects with a \"for as long as\" duration ignore phased-out objects."
    );
    supported("Galadriel's Dismissal");
    supported("Talon Gates of Madara");
    supported("Ripples of Potential");
    supported("Sower of Temptation");
    for how in ["Galadriel's Dismissal", "Talon Gates of Madara", "Ripples of Potential"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let sower = crate::r_p038_common::enter(&mut t, P0, "Sower of Temptation");
        t.resolve_all();
        assert_eq!(t.obj_now(bears).controller, P0, "{how}");
        match how {
            "Talon Gates of Madara" => {
                t.answer_targets(P0, &[obj(sower)]);
                crate::r_p038_common::enter(&mut t, P0, how);
            }
            "Ripples of Potential" => {
                put(&mut t, sower, counters::PLUS1, 1);
                t.answer_choose(P0, &[obj(sower)]);
                t.answer_choose(P0, &[obj(sower)]);
                cast_card(&mut t, P0, how, &[]);
            }
            _ => {
                cast_card(&mut t, P0, how, &[obj(sower)]);
            }
        }
        t.resolve_all();
        assert!(phased_out(&t, sower), "{how}");
        assert_eq!(t.obj_now(bears).controller, P1, "{how}");
        // Phasing back in doesn't restart the effect.
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        assert!(!phased_out(&t, sower), "{how}");
        assert_eq!(t.obj_now(bears).controller, P1, "{how}");
    }
}

#[test]
fn the_magic_mirror_doesnt_count_adventurer_cards() {
    cr!("715.4", "601.2f");
    ruling!(
        "The Magic Mirror",
        "Adventurer cards aren't instant or sorcery cards while they're in your graveyard."
    );
    supported("The Magic Mirror");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Bonecrusher Giant // Stomp");
    pool(&mut t, P0, ManaType::U, 3);
    pool(&mut t, P0, ManaType::C, 5);
    let mm = t.hand(P0, "The Magic Mirror");
    assert!(t.cast(P0, mm).try_go().is_err(), "costs {{6}}{{U}}{{U}}{{U}}");
    t.graveyard(P0, "Divination");
    t.cast(P0, mm).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn diary_of_dreams_is_a_book_artifact() {
    cr!("205.3g");
    ruling!(
        "Diary of Dreams",
        "Book is a new artifact type with no inherent rules meaning."
    );
    supported("Diary of Dreams");
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Diary of Dreams");
    assert!(t.obj_now(dd).chars.has_subtype("Book"));
    assert!(t.obj_now(dd).is(CardType::Artifact));
    put(&mut t, dd, "page", 2);
    pool(&mut t, P0, ManaType::C, 3);
    let hand = t.hand_size(P0);
    t.activate(P0, dd, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
