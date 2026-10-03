//! CR 702.157 Squad.

use crate::common_k702_038_051::with_cost;
use crate::common_k702_153_167::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn squad_creates_a_token_copy_for_each_time_its_cost_was_paid() {
    cr!("702.157", "702.157a");
    ruling!(
        "Galadhrim Brigade",
        "If you paid the squad cost multiple times, the tokens will all enter the battlefield simultaneously."
    );
    ruling!(
        "Wasteland Raider",
        "The tokens created by the squad ability aren’t “cast,” so any abilities that trigger when a spell is cast won’t trigger for the copies."
    );
    ruling!(
        "Galadhrim Brigade",
        "The tokens created by the squad ability aren't \"cast,\" so any abilities that trigger when a spell is cast won't trigger for the copies."
    );
    assert_supported("Galadhrim Brigade");
    // Galadhrim Brigade {2}{G}: squad {1}{G}; other Elves you control get +1/+1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let brigade = t.hand(P0, "Galadhrim Brigade");
    pay_times(&mut t, P0, 2);
    t.cast(P0, brigade).go();
    assert_eq!(optional_costs_offered(&t, P0), vec!["squad#1".to_string()]);
    t.resolve();
    // The permanent's enters ability.
    assert_eq!(triggers_named(&t, "Squad").len(), 1);
    t.resolve_all();
    let all = named(&t, P0, "Galadhrim Brigade");
    assert_eq!(all.len(), 3);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    // They're copies (each gets +1/+1 from the other two Brigades' static abilities).
    for id in &all {
        assert_eq!(t.pt(*id), (4, 4));
    }
    // The copies weren't cast: their squad abilities didn't trigger, and only one spell
    // was cast.
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn without_paying_the_squad_cost_nothing_triggers() {
    cr!("702.157a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let brigade = t.hand(P0, "Galadhrim Brigade");
    pay_times(&mut t, P0, 0);
    t.cast(P0, brigade).go();
    t.resolve();
    assert!(triggers_named(&t, "Squad").is_empty());
    assert!(tokens(&t, P0).is_empty());
    // A Brigade put onto the battlefield without being cast has no payments either.
    let mut t = TestGame::new(2);
    t.enter(P0, "Galadhrim Brigade");
    t.settle();
    assert!(triggers_named(&t, "Squad").is_empty());
}

#[test]
fn the_copies_are_created_even_if_the_creature_left_the_battlefield() {
    cr!("702.157a");
    ruling!(
        "Wasteland Raider",
        "If the spell resolves but the creature with squad leaves the battlefield before its squad ability resolves, you’ll still create the token copies."
    );
    ruling!(
        "Ruthless Radrat",
        "You will create a token that is a copy of that permanent for each time you paid the squad cost."
    );
    assert_supported("Ruthless Radrat");
    // Ruthless Radrat: "Squad—Exile four cards from your graveyard."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    for _ in 0..8 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let rat = t.hand(P0, "Ruthless Radrat");
    pay_times(&mut t, P0, 2);
    t.cast(P0, rat).go();
    assert_eq!(t.graveyard_size(P0), 0, "eight cards were exiled");
    t.resolve();
    t.settle();
    assert_eq!(triggers_named(&t, "Squad").len(), 1);
    let rat_now = named(&t, P0, "Ruthless Radrat")[0];
    let owner = t.obj(rat_now).owner;
    t.g.move_object(
        rat_now,
        Zone::Hand(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.resolve_all();
    assert_eq!(named(&t, P0, "Ruthless Radrat").len(), 2);
    assert!(named(&t, P0, "Ruthless Radrat")
        .iter()
        .all(|id| t.obj(*id).is_token()));
}

#[test]
fn a_countered_squad_spell_or_a_permanent_without_squad_makes_no_tokens() {
    cr!("702.157a");
    ruling!(
        "Galadhrim Brigade",
        "If the spell is countered, the squad ability will not trigger, and no tokens will be created."
    );
    ruling!(
        "Galadhrim Brigade",
        "If, for some reason, the creature doesn't have the squad ability when it's on the battlefield, the ability won't trigger, even if you've paid the squad cost one or more times."
    );
    // Countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let brigade = t.hand(P0, "Galadhrim Brigade");
    pay_times(&mut t, P0, 1);
    let spell = t.cast(P0, brigade).go();
    run_effect(
        &mut t,
        None,
        P1,
        mtg_engine::ability::Effect::CounterSpell {
            what: mtg_engine::ability::Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Galadhrim Brigade"));
    assert!(tokens(&t, P0).is_empty());
    // Dress Down: "Creatures lose all abilities." The Brigade has no squad ability on the
    // battlefield: nothing triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dress Down");
    t.lands(P0, "Forest", 5);
    let brigade = t.hand(P0, "Galadhrim Brigade");
    pay_times(&mut t, P0, 1);
    t.cast(P0, brigade).go();
    t.resolve();
    t.settle();
    assert!(triggers_named(&t, "Squad").is_empty());
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

/// A {W} 1/1 creature with two squad abilities.
fn twin_squadron() -> mtg_engine::card::CardDef {
    with_cost(
        custom_card(
            "Twin Squadron",
            "Creature — Human Soldier",
            Some((1, 1)),
            "Squad {1}, squad {2}",
        ),
        "{W}",
    )
}

#[test]
fn each_instance_of_squad_is_paid_separately_and_counts_its_own_payments() {
    cr!("702.157b");
    // A creature with two squad abilities.
    let def = twin_squadron();
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let c = t.custom(P0, def, Zone::Hand(P0));
    // Squad {1} twice, squad {2} once: {W} + 2 + 2 + ... = 6 mana with the spell.
    pay_times(&mut t, P0, 2);
    pay_times(&mut t, P0, 1);
    t.cast(P0, c).go();
    assert_eq!(
        optional_costs_offered(&t, P0),
        vec!["squad#1".to_string(), "squad#2".to_string()]
    );
    t.resolve();
    t.settle();
    // Each instance triggers for its own payments.
    let trig = triggers_named(&t, "Squad");
    assert_eq!(trig.len(), 2);
    t.resolve();
    let after_first = tokens(&t, P0).len();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    assert!(after_first == 1 || after_first == 2);
    // Paying only the second instance: only its ability triggers.
    let def = twin_squadron();
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let c = t.custom(P0, def, Zone::Hand(P0));
    pay_times(&mut t, P0, 0);
    pay_times(&mut t, P0, 1);
    t.cast(P0, c).go();
    t.resolve();
    t.settle();
    assert_eq!(triggers_named(&t, "Squad").len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn the_squad_cost_may_be_paid_any_number_of_times() {
    cr!("702.157a");
    ruling!(
        "Sicarian Infiltrator",
        "You may pay the squad cost any number of times. You will get a token that is a copy of that permanent for each time you paid the squad cost."
    );
    assert_supported("Sicarian Infiltrator");
    // Sicarian Infiltrator {2}{U}: flash, squad {2}, "When this creature enters, draw a
    // card." Paid three times: three tokens, and each of the four draws a card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    let card = t.hand(P0, "Sicarian Infiltrator");
    let hand = t.hand_size(P0) - 1;
    pay_times(&mut t, P0, 3);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(named(&t, P0, "Sicarian Infiltrator").len(), 4);
    assert_eq!(tokens(&t, P0).len(), 3);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn a_permanent_without_its_squad_ability_doesnt_trigger() {
    cr!("702.157a");
    ruling!(
        "Wasteland Raider",
        "If, for some reason, the creature doesn’t have the squad ability when it’s on the battlefield, the ability won’t trigger, even if you’ve paid the squad cost one or more times."
    );
    ruling!(
        "Roadkill Rodney",
        "If, for some reason, the permanent doesn't have the squad ability when it's on the battlefield, the ability won't trigger, even if you've paid the squad cost one or more times."
    );
    for name in ["Wasteland Raider", "Roadkill Rodney"] {
        assert_supported(name);
        // Dress Down: "Creatures lose all abilities."
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Dress Down");
        t.lands(P0, "Swamp", 8);
        let card = t.hand(P0, name);
        pay_times(&mut t, P0, 1);
        t.cast(P0, card).go();
        t.resolve();
        t.settle();
        assert!(triggers_named(&t, "Squad").is_empty(), "{name}");
        t.resolve_all();
        assert!(tokens(&t, P0).is_empty(), "{name}");
    }
}

#[test]
fn roadkill_rodneys_copies_are_created_after_it_left_the_battlefield() {
    cr!("702.157a");
    ruling!(
        "Roadkill Rodney",
        "If the spell resolves but the permanent with squad leaves the battlefield before its squad ability resolves, you'll still create the token copies."
    );
    // Roadkill Rodney {2}: squad {3}, deathtouch.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 8);
    let card = t.hand(P0, "Roadkill Rodney");
    pay_times(&mut t, P0, 2);
    t.cast(P0, card).go();
    t.resolve();
    t.settle();
    assert_eq!(triggers_named(&t, "Squad").len(), 1);
    let rodney = named(&t, P0, "Roadkill Rodney")[0];
    t.g.move_object(
        rodney,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.resolve_all();
    let copies = named(&t, P0, "Roadkill Rodney");
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert!(t.obj(c).is_token());
        assert!(has_kw(&t, c, mtg_engine::keywords::KeywordKind::Deathtouch));
    }
}

#[test]
fn a_miracle_spell_may_also_pay_its_squad_cost() {
    cr!("702.157a", "702.94a", "118.8");
    ruling!(
        "Zephyrim",
        "If you cast Zephyrim for its miracle cost, you may also choose to pay its squad cost one or more times."
    );
    ruling!(
        "Zephyrim",
        "You may pay the squad cost any number of times. You will get a token that is a copy of that permanent for each time you paid the squad cost."
    );
    assert_supported("Zephyrim");
    // Zephyrim {3}{W} 3/3: squad {2}, flying, vigilance, miracle {1}{W}. Drawn first this
    // turn and cast for {1}{W} plus squad {2} twice.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Zephyrim");
    t.lands(P0, "Plains", 6);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast
    pay_times(&mut t, P0, 2);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    let all = named(&t, P0, "Zephyrim");
    assert_eq!(all.len(), 3);
    assert_eq!(tokens(&t, P0).len(), 2);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Plains")
        .all(|o| o.tapped));
}

#[test]
fn securitron_squadrons_copies_see_each_other_enter() {
    cr!("702.157a", "603.6a");
    ruling!(
        "Securitron Squadron",
        "If Securitron Squadron enters the battlefield at the same time as one or more creature tokens, its last ability will trigger for each of those creature tokens. If that Securitron Squadron is itself a creature token, its last ability will trigger when it enters the battlefield as well."
    );
    ruling!(
        "Securitron Squadron",
        "After all of the triggered abilities resolve, each of the three token copies of Securitron Squadron will have four +1/+1 counters on it."
    );
    assert_supported("Securitron Squadron");
    // Securitron Squadron {1}{W}: squad {3}, vigilance, "Whenever a creature token you
    // control enters, put a +1/+1 counter on it." Squad paid three times: each token gets
    // a counter from each of the four Squadrons.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 11);
    let card = t.hand(P0, "Securitron Squadron");
    pay_times(&mut t, P0, 3);
    t.cast(P0, card).go();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 3);
    for tok in &toks {
        assert_eq!(plus1(&t, *tok), 4);
    }
    let original = named(&t, P0, "Securitron Squadron")
        .into_iter()
        .find(|id| !t.obj(*id).is_token())
        .unwrap();
    assert_eq!(plus1(&t, original), 0);
}

#[test]
fn no_player_acts_while_the_squad_spell_is_being_cast() {
    cr!("702.157a", "601.2");
    ruling!(
        "Ruthless Radrat",
        "Once you announce that you’re casting Ruthless Radrat, no player may take actions until you’re done casting it."
    );
    ruling!(
        "Ruthless Radrat",
        "If, for some reason, the creature doesn’t have the squad ability when it’s on the battlefield, the ability won’t trigger, even if you’ve paid the squad cost one or more times."
    );
    // Ruthless Radrat: "Squad—Exile four cards from your graveyard." The opponent isn't
    // asked anything while it's cast, so the cards can't be removed from the graveyard
    // before the cost is paid.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    for _ in 0..4 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let rat = t.hand(P0, "Ruthless Radrat");
    let asked_before = t.asked().len();
    pay_times(&mut t, P0, 1);
    t.cast(P0, rat).go();
    assert!(t.asked()[asked_before..].iter().all(|(p, _)| *p == P0));
    assert_eq!(t.graveyard_size(P0), 0);
    // With its abilities lost (Dress Down), its squad ability doesn't trigger.
    t.battlefield(P1, "Dress Down");
    t.resolve();
    t.settle();
    assert!(triggers_named(&t, "Squad").is_empty());
}
