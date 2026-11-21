//! Rulings batch S08 — Food (CR 111.10b, 205.3g): a predefined artifact token with "{2},
//! {T}, Sacrifice this token: You gain 3 life." Food is an artifact type; "a Food" is any
//! Food artifact; one Food pays one cost. Forage (CR 701.61): exile three cards from your
//! graveyard or sacrifice a Food.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn food_is_an_artifact_type_and_never_a_creature_type() {
    cr!("205.3g", "205.3m", "702.73a");
    ruling!(
        "Gingerbrute",
        "Food is an artifact type. Even though it appears on some creatures, it's never a creature type."
    );
    supported("Gingerbrute");
    supported("Bosco, Just a Bear");
    supported("Changeling Outcast");
    assert!(!mtg_engine::types::is_creature_type("Food"));
    // Bosco: "{2}{G}, Sacrifice a Food: Put two +1/+1 counters on Bosco." Changeling
    // Outcast is every creature type, but not a Food.
    let mut t = TestGame::new(2);
    let bosco = t.battlefield(P0, "Bosco, Just a Bear");
    let outcast = t.battlefield(P0, "Changeling Outcast");
    t.lands(P0, "Forest", 3);
    assert!(t.obj(outcast).chars.has_subtype("Golem"));
    assert!(!t.obj(outcast).chars.has_subtype("Food"));
    assert!(!can_activate(&mut t, P0, bosco));
    // Gingerbrute (Artifact Creature — Food Golem) is a Food.
    let brute = t.battlefield(P0, "Gingerbrute");
    assert!(t.obj(brute).chars.has_subtype("Food"));
    assert!(can_activate(&mut t, P0, bosco));
}

#[test]
fn a_food_pays_only_one_cost() {
    cr!("602.2b", "601.2g", "601.2h");
    ruling!(
        "Gilded Goose",
        "You can't sacrifice a Food to pay multiple costs. For example, you can't sacrifice a Food token to activate its own ability and also to activate Maraleaf Rider's ability."
    );
    supported("Gilded Goose");
    // Gilded Goose: "{T}, Sacrifice a Food: Add one mana of any color." A Food token's own
    // ability costs {2}, {T} and sacrificing it: the Goose's mana can't come from
    // sacrificing that same Food.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gilded Goose");
    t.lands(P0, "Forest", 1);
    let food = create_token(&mut t, P0, "Food");
    assert!(t.activate(P0, food, 0, &[]).is_err());
    assert!(t.on_battlefield(food));
    assert_eq!(t.life(P0), 20);
    // With a second Food, the Goose sacrifices that one.
    let other = create_token(&mut t, P0, "Food");
    t.activate(P0, food, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(!t.g.is_live(food) && !t.g.is_live(other));
}

#[test]
fn a_food_making_spell_whose_target_is_illegal_makes_no_food() {
    cr!("608.2b", "111.10b");
    ruling!(
        "Bake into a Pie",
        "Some spells and abilities that create Food tokens may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't create any Food tokens."
    );
    supported("Bake into a Pie");
    // Bake into a Pie: "Destroy target creature. Create a Food token."
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let pie = t.hand(P0, "Bake into a Pie");
        t.cast(P0, pie).target(bears).go();
        if respond {
            // Hexproof: an illegal target.
            t.lands(P1, "Forest", 1);
            let defense = t.hand(P1, "Blossoming Defense");
            t.cast(P1, defense).target(bears).go();
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), respond);
        assert_eq!(with_subtype(&t, P0, "Food").len(), usize::from(!respond));
    }
}

#[test]
fn any_food_artifact_pays_a_sacrifice_a_food_cost_once() {
    cr!("205.3g", "602.2b");
    ruling!(
        "Bosco, Just a Bear",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, you can sacrifice Tough Cookie (an Artifact Creature — Food Golem) to activate Bosco, Just a Bear's ability (an ability with \"Sacrifice a Food\" in its cost). You can't sacrifice a Food to pay multiple costs."
    );
    supported("Tough Cookie");
    // Bosco: "{2}{G}, Sacrifice a Food: Put two +1/+1 counters on Bosco. He gains trample
    // until end of turn."
    let mut t = TestGame::new(2);
    let bosco = t.battlefield(P0, "Bosco, Just a Bear");
    let cookie = t.battlefield(P0, "Tough Cookie");
    t.lands(P0, "Forest", 6);
    t.answer_choose(P0, &[Entity::Object(cookie)]);
    t.activate(P0, bosco, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tough Cookie"));
    assert_eq!(t.counters(bosco, "+1/+1"), 2);
    // The Cookie is gone: nothing left to sacrifice for a second activation.
    assert!(!can_activate(&mut t, P0, bosco));
}

#[test]
fn opponents_cant_respond_while_you_forage_to_cast_a_spell() {
    cr!("601.2", "601.2h", "701.61a");
    ruling!(
        "Feed the Cycle",
        "Once you announce that you're casting a spell or activating an ability, players can't take actions until you've finished doing so. Notably, opponents can't try to remove cards from your graveyard or Foods you control to stop you from foraging."
    );
    supported("Feed the Cycle");
    // Feed the Cycle ({1}{B}): "As an additional cost to cast this spell, forage or pay
    // {B}. Destroy target creature or planeswalker."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    for _ in 0..3 {
        t.graveyard(P0, "Hill Giant");
    }
    let feed = t.hand(P0, "Feed the Cycle");
    let from = t.asked().len();
    t.cast(P0, feed).target(bears).go();
    // Only P0 made decisions while casting it; the graveyard was exiled as a cost.
    let asked = t.asked()[from..].to_vec();
    assert!(asked.iter().all(|(p, _)| *p == P0), "{asked:?}");
    assert!(!asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
