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
