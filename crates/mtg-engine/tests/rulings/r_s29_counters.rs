//! Rulings batch S29 — counters (CR 122): who puts the counters a permanent enters with
//! (CR 122.6a), poison counters (CR 704.5c), the order of effects that modify how many
//! counters are put on a permanent (CR 616.1), replacement effects that apply to entering
//! creatures of a type (CR 614.12), abilities that trigger on life gain and on entering
//! with a power (CR 603.2, 603.6a, 704.3).

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s03_common::respond;
use crate::r_s10_common::poison;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_controller_of_an_entering_permanent_puts_its_counters_on_it() {
    cr!("122.6a", "614.12");
    ruling!(
        "Halving Season",
        "If a permanent enters the battlefield with counters on it, the effect causing the permanent to be given counters may specify which player puts those counters on it. If the effect doesn’t specify a player, the object’s controller puts those counters on it."
    );
    supported("Halving Season");
    supported("Star Pupil");
    // Halving Season (P0): "If an opponent would put one or more counters on a permanent
    // or player, they put half that many of each of those kinds of counters on that
    // permanent or player instead, rounded down." Star Pupil (0/0): "This creature enters
    // with a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halving Season");
    // P0's Star Pupil: P0 puts the counter on it; not halved.
    let mine = cast_new(&mut t, P0, "Star Pupil", &[]);
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert_eq!(t.pt(mine), (1, 1));
    // P1's Star Pupil: P1 (its controller) puts the counter on it: half of one is none,
    // and the 0/0 dies.
    t.set_step(P1, Step::PrecombatMain);
    let theirs = cast_new(&mut t, P1, "Star Pupil", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Star Pupil"));
    assert_eq!(t.zone(theirs), mtg_engine::object::Zone::Graveyard(P1));
    assert!(t.on_battlefield(mine));
}

#[test]
fn ten_poison_counters_lose_the_game_as_a_state_based_action() {
    cr!("704.5c", "122.1f", "704.3");
    ruling!(
        "Noxious Bayou",
        "A player with ten or more poison counters loses the game as a state-based action."
    );
    supported("Noxious Bayou");
    // Noxious Bayou: "{T}: Add {B} or {G}. You get a poison counter." P0 has nine poison
    // counters and taps it for mana.
    let mut t = TestGame::new(2);
    let bayou = t.battlefield(P0, "Noxious Bayou");
    t.g.add_counters(Entity::Player(P0), counters::POISON, 9, None);
    assert_eq!(poison(&t, P0), 9);
    t.activate(P0, bayou, 0, &[]).unwrap();
    assert_eq!(poison(&t, P0), 10);
    assert!(!t.has_lost(P0), "not until state-based actions are checked");
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn the_permanents_controller_orders_counter_modifying_effects() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Caradora, Heart of Alacria",
        "If two or more effects attempt to modify how many counters would be put on a permanent you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Caradora, Heart of Alacria");
    supported("Primal Vigor");
    // Caradora (P0): "If one or more +1/+1 counters would be put on a creature or Vehicle
    // you control, that many plus one +1/+1 counters are put on it instead." Primal Vigor
    // (P1): "... If one or more +1/+1 counters would be put on a creature, twice that many
    // +1/+1 counters are put on that creature instead."
    for (order, expected) in [(plus_one_first as Responder, 4), (twice_first, 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Caradora, Heart of Alacria");
        t.battlefield(P1, "Primal Vigor");
        let bears = t.battlefield(P0, "Grizzly Bears");
        respond(&mut t, P0, order);
        let from = t.asked().len();
        put_counters(&mut t, bears, counters::PLUS1, 1);
        assert_eq!(t.counters(bears, counters::PLUS1), expected);
        // P0 (the controller of the Bears) chose, not P1.
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

#[test]
fn a_creature_that_becomes_the_chosen_type_gets_the_types_entering_bonus() {
    cr!("614.12", "613.1d");
    ruling!(
        "Xenograft",
        "Replacement effects that modify creatures of a certain type as they enter the battlefield will apply (or not apply) after you apply this effect. For example, if Warrior is the chosen creature type and you control Bramblewood Paragon, a Runeclaw Bear would enter the battlefield with an additional +1/+1 counter."
    );
    supported("Xenograft");
    supported("Bramblewood Paragon");
    supported("Runeclaw Bear");
    // Xenograft: "Each creature you control is the chosen type in addition to its other
    // types." Bramblewood Paragon: "Each other Warrior creature you control enters with
    // an additional +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramblewood Paragon");
    choose_creature_type(&mut t, P0, "Warrior");
    t.enter(P0, "Xenograft");
    let bear = cast_new(&mut t, P0, "Runeclaw Bear", &[]);
    t.resolve_all();
    assert!(t.obj_now(bear).chars.has_subtype("Warrior"));
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
    assert_eq!(t.pt(bear), (3, 3));
    // Without Xenograft, a Runeclaw Bear isn't a Warrior and gets no counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramblewood Paragon");
    let bear = cast_new(&mut t, P0, "Runeclaw Bear", &[]);
    t.resolve_all();
    assert_eq!(t.counters(bear, counters::PLUS1), 0);
}

#[test]
fn a_pridemate_dealt_lethal_damage_as_you_gain_life_dies_before_its_counter() {
    cr!("510.2", "704.3", "704.5g", "603.3");
    ruling!(
        "Ajani's Pridemate",
        "If Ajani's Pridemate is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    supported("Ajani's Pridemate");
    supported("Child of Night");
    // P1 attacks with a Hill Giant and a Grizzly Bears; P0's Ajani's Pridemate (2/2,
    // "Whenever you gain life, put a +1/+1 counter on this creature.") blocks the Giant
    // and Child of Night (2/1 lifelink) blocks the Bears. Combat damage is dealt at once.
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let child = t.battlefield(P0, "Child of Night");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    block_and_finish(&mut t, P0, &[(pridemate, giant), (child, bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "P0 gained 2 life");
    assert!(t.in_graveyard(P0, "Ajani's Pridemate"));
    assert!(!t.on_battlefield(pridemate));
}

#[test]
fn an_entering_creatures_power_is_checked_only_as_it_enters() {
    cr!("603.2", "603.6a", "613.4c");
    ruling!(
        "Where Ancients Tread",
        "The ability checks that creature's power only once: when that creature enters. The trigger checks a creature's initial power upon being put on the battlefield, so it will take into account counters that it enters with and static abilities that may give it a continuous power boost once it's on the battlefield (such as the one on Glorious Anthem). After the creature is already on the battlefield, boosting its power with a spell (such as Giant Growth), activated ability, or triggered ability won't allow this ability to trigger; it's too late by then. Once the ability triggers, it will resolve no matter what the creature's power may become while the ability is on the stack."
    );
    supported("Where Ancients Tread");
    supported("Mighty Emergence");
    supported("Rumbling Baloth");
    // Where Ancients Tread: "Whenever a creature you control with power 5 or greater
    // enters, you may have this enchantment deal 5 damage to any target." Rumbling
    // Baloth is a 4/4.
    let tread = |t: &mut TestGame| {
        t.battlefield(P0, "Where Ancients Tread");
    };
    // A 4/4 alone: no trigger; Giant Growth afterwards is too late.
    let mut t = TestGame::new(2);
    tread(&mut t);
    let baloth = cast_new(&mut t, P0, "Rumbling Baloth", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(baloth)]);
    t.resolve_all();
    assert_eq!(t.pt(baloth), (7, 7));
    assert_eq!(t.life(P1), 20);
    // With Glorious Anthem, it's a 5/5 as it enters: the ability triggers.
    let mut t = TestGame::new(2);
    tread(&mut t);
    t.battlefield(P0, "Glorious Anthem");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Rumbling Baloth", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Entering with a +1/+1 counter (riot from Rhythm of the Wild): it triggers; then it
    // shrinks while the ability is on the stack (Disfigure, -2/-2): it still resolves.
    let mut t = TestGame::new(2);
    tread(&mut t);
    t.battlefield(P0, "Rhythm of the Wild");
    t.answer_yes(P0, true); // riot: the counter
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let baloth = cast_new(&mut t, P0, "Rumbling Baloth", &[]);
    t.resolve();
    assert_eq!(t.pt(baloth), (5, 5));
    assert_eq!(t.stack_len(), 1, "Where Ancients Tread triggered");
    cast_new(&mut t, P1, "Disfigure", &[Entity::Object(baloth)]);
    t.resolve();
    assert_eq!(t.pt(baloth), (3, 3));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Mighty Emergence: "Whenever a creature you control with power 5 or greater enters,
    // you may put two +1/+1 counters on it." A 4/4 pumped after it entered: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mighty Emergence");
    let baloth = cast_new(&mut t, P0, "Rumbling Baloth", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(baloth)]);
    t.resolve_all();
    assert_eq!(t.counters(baloth, counters::PLUS1), 0);
}
