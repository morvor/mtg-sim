//! Rulings batch P034 — counters a permanent enters with, and the replacement effects
//! that modify them (CR 614.1c, 614.12, 122.6): X costs, "enters with an additional
//! counter", "that many plus one".

use crate::r_p034_common::*;
use crate::r_p038_common::cast_only;
use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// `p` casts the creature `name` with X = `x` (paying from a pool of `mana` generic plus
/// the colored symbols) and it resolves. Returns the permanent.
fn cast_x(t: &mut TestGame, p: PlayerId, name: &str, x: i64, colored: &[ManaType]) -> ObjectId {
    supported(name);
    for ty in colored {
        pool(t, p, *ty, 1);
    }
    let c = t.hand(p, name);
    t.cast(p, c).x(x).go();
    t.resolve_all();
    t.g.current(c)
}

#[test]
fn walking_ballista_x_x_costs_twice_x() {
    cr!("107.3a", "601.2f");
    ruling!(
        "Walking Ballista",
        "A casting cost of {X}{X} means that you pay twice X."
    );
    supported("Walking Ballista");
    let mut t = TestGame::new(2);
    pool(&mut t, P0, ManaType::C, 5);
    let wb = t.hand(P0, "Walking Ballista");
    assert!(t.cast(P0, wb).x(3).try_go().is_err(), "X = 3 costs {{6}}");
    pool(&mut t, P0, ManaType::C, 1);
    t.cast(P0, wb).x(3).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, wb), 3);
}

#[test]
fn apocalypse_hydra_doubles_its_counters_from_x_5_using_the_chosen_x() {
    cr!("107.3a", "614.1c");
    ruling!(
        "Apocalypse Hydra",
        "if X is 4, Apocalypse Hydra enters with four +1/+1 counters on it, but if X is 5"
    );
    ruling!(
        "Apocalypse Hydra",
        "The first ability checks the number you chose for X, not the amount of mana you actually spent."
    );
    let mut t = TestGame::new(2);
    pool(&mut t, P0, ManaType::C, 4);
    let h = cast_x(&mut t, P0, "Apocalypse Hydra", 4, &[ManaType::R, ManaType::G]);
    assert_eq!(plus1(&t, h), 4);
    let mut t = TestGame::new(2);
    pool(&mut t, P0, ManaType::C, 5);
    let h = cast_x(&mut t, P0, "Apocalypse Hydra", 5, &[ManaType::R, ManaType::G]);
    assert_eq!(plus1(&t, h), 10);
    // With Krosan Drover (mana value 6 or greater costs {2} less), X = 5 costs only {3}
    // for X: the Hydra still enters with ten.
    supported("Krosan Drover");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krosan Drover");
    pool(&mut t, P0, ManaType::C, 3);
    let h = cast_x(&mut t, P0, "Apocalypse Hydra", 5, &[ManaType::R, ManaType::G]);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "only five mana was spent");
    assert_eq!(plus1(&t, h), 10);
}

#[test]
fn benevolent_hydra_adds_one_to_counters_put_on_or_entering_with_another_creature() {
    cr!("614.1a", "614.1c", "616.1");
    ruling!(
        "Benevolent Hydra",
        "If another creature would enter the battlefield with a number of +1/+1 counters on it while you control Benevolent Hydra, it enters with that many counters plus one."
    );
    ruling!(
        "Benevolent Hydra",
        "Each additional Benevolent Hydra you control will increase the number of +1/+1 counters placed on anther creature by one."
    );
    supported("Benevolent Hydra");
    supported("Give // Take");
    // One Hydra: Walking Ballista with X = 1 enters with two; Give puts four on Bears.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Benevolent Hydra");
    pool(&mut t, P0, ManaType::C, 2);
    let wb = cast_x(&mut t, P0, "Walking Ballista", 1, &[]);
    assert_eq!(plus1(&t, wb), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give(&mut t, P0, bears);
    assert_eq!(plus1(&t, bears), 4);
    // Not on the Hydra itself.
    give(&mut t, P0, hydra);
    assert_eq!(plus1(&t, hydra), 3);
    // An opponent's creature isn't "another creature you control".
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give(&mut t, P0, theirs);
    assert_eq!(plus1(&t, theirs), 3);

    // Two Hydras: five on Bears; each Hydra gets one more from the other.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Benevolent Hydra");
    t.battlefield(P0, "Benevolent Hydra");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give(&mut t, P0, bears);
    assert_eq!(plus1(&t, bears), 5);
    give(&mut t, P0, a);
    assert_eq!(plus1(&t, a), 4);
}

#[test]
fn sage_of_fables_adds_a_counter_to_each_other_wizard_entering() {
    cr!("614.1c", "614.12", "616.1");
    ruling!(
        "Sage of Fables",
        "If a Wizard would normally enter with no +1/+1 counters on it, it enters with one +1/+1 counter on it instead."
    );
    ruling!(
        "Sage of Fables",
        "if you have two Sage of Fables on the battlefield, Wizard creatures you control enter with two additional"
    );
    supported("Sage of Fables");
    supported("Cytoplast Manipulator");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sage of Fables");
    let cm = t.enter(P0, "Cytoplast Manipulator");
    assert_eq!(plus1(&t, cm), 3, "graft 2 plus one");
    let ps = t.enter(P0, "Prodigal Sorcerer");
    assert_eq!(plus1(&t, ps), 1);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(plus1(&t, bears), 0, "not a Wizard");
    let theirs = t.enter(P1, "Prodigal Sorcerer");
    assert_eq!(plus1(&t, theirs), 0, "not one you control");
    // Two Sages: two additional counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sage of Fables");
    t.battlefield(P0, "Sage of Fables");
    let ps = t.enter(P0, "Prodigal Sorcerer");
    assert_eq!(plus1(&t, ps), 2);
}

#[test]
fn sage_of_fables_doesnt_affect_a_wizard_entering_at_the_same_time() {
    cr!("614.12");
    ruling!(
        "Sage of Fables",
        "If Sage of Fables enters at the same time as another Wizard"
    );
    supported("Ghostly Flicker");
    let mut t = TestGame::new(2);
    let sage = t.battlefield(P0, "Sage of Fables");
    let ps = t.battlefield(P0, "Prodigal Sorcerer");
    cast_card(&mut t, P0, "Ghostly Flicker", &[obj(sage), obj(ps)]);
    t.resolve_all();
    let ps = t.named_on_battlefield("Prodigal Sorcerer")[0];
    let sage = t.named_on_battlefield("Sage of Fables")[0];
    assert_eq!(plus1(&t, ps), 0);
    assert_eq!(plus1(&t, sage), 0);
    // A Wizard entering afterwards gets the counter.
    let later = t.enter(P0, "Prodigal Sorcerer");
    assert_eq!(plus1(&t, later), 1);
}

#[test]
fn zameck_guildmage_affects_only_creatures_entering_later_and_is_cumulative() {
    cr!("614.1c");
    ruling!(
        "Zameck Guildmage",
        "Zameck Guildmage's first ability has no effect on creatures already on the battlefield"
    );
    ruling!(
        "Zameck Guildmage",
        "if it resolves twice in a turn, each creature that enters under your control that turn will do so with two"
    );
    supported("Zameck Guildmage");
    let mut t = TestGame::new(2);
    let zg = t.battlefield(P0, "Zameck Guildmage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    t.activate(P0, zg, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
    assert_eq!(plus1(&t, zg), 0);
    let one = t.enter(P0, "Grizzly Bears");
    assert_eq!(plus1(&t, one), 1);
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(plus1(&t, theirs), 0, "only creatures you control");
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    t.activate(P0, zg, 0, &[]).unwrap();
    t.resolve_all();
    let two = t.enter(P0, "Grizzly Bears");
    assert_eq!(plus1(&t, two), 2);
}

#[test]
fn galloping_lizrog_can_double_the_counters_it_entered_with() {
    cr!("614.1c");
    ruling!(
        "Galloping Lizrog",
        "you may remove those counters with its triggered ability and put twice as many back on it"
    );
    supported("Galloping Lizrog");
    let mut t = TestGame::new(2);
    let zg = t.battlefield(P0, "Zameck Guildmage");
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    t.activate(P0, zg, 0, &[]).unwrap();
    t.resolve_all();
    let lizrog = crate::r_p038_common::enter(&mut t, P0, "Galloping Lizrog");
    assert_eq!(plus1(&t, lizrog), 1, "entered with Zameck's counter");
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Number,
        mtg_engine::decision::Answer::Number(1),
    );
    t.resolve_all();
    assert_eq!(plus1(&t, lizrog), 2);
}

#[test]
fn realm_seekers_counts_the_cards_in_hand_as_it_enters() {
    cr!("614.1c", "614.12");
    ruling!(
        "Realm Seekers",
        "If Realm Seekers enters the battlefield directly from a player's hand without being cast, it counts itself"
    );
    ruling!(
        "Realm Seekers",
        "The value of X is determined as Realm Seekers enters the battlefield."
    );
    supported("Realm Seekers");
    supported("Elvish Piper");
    // Put onto the battlefield from the hand by Elvish Piper: it counts itself (four
    // cards: itself, one more of P0's, two of P1's).
    let mut t = TestGame::new(2);
    let piper = t.battlefield(P0, "Elvish Piper");
    let rs = t.hand(P0, "Realm Seekers");
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    pool(&mut t, P0, ManaType::G, 1);
    t.activate(P0, piper, 0, &[]).unwrap();
    t.answer_choose(P0, &[obj(rs)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(rs));
    assert_eq!(plus1(&t, rs), 4);

    // Cast: P1 casts a card from hand in response, so it enters with one fewer counter.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let rs = cast_only(&mut t, P0, "Realm Seekers", &[]);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve();
    // Hands now: P0's Grizzly Bears and P1's Grizzly Bears.
    t.resolve_all();
    assert_eq!(plus1(&t, rs), 2);
}

#[test]
fn give_take_take_can_target_a_creature_without_counters() {
    cr!("115.1", "709.3");
    ruling!(
        "Give // Take",
        "You can target any creature you control with Take; the target does not need to have any +1/+1 counters"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let gt = t.hand(P0, "Give // Take");
    let hand = t.hand_size(P0);
    t.cast(P0, gt)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1, "Take resolved and drew no cards");
    assert!(t.in_graveyard(P0, "Give // Take"));
    // With two counters it draws two.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    t.lands(P0, "Island", 3);
    let gt = t.hand(P0, "Give // Take");
    let hand = t.hand_size(P0);
    t.cast(P0, gt)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn winding_constrictor_sees_a_permanent_entering_as_an_artifact() {
    cr!("614.12", "306.5b", "122.6");
    ruling!(
        "Winding Constrictor",
        "Winding Constrictor's effect will give that permanent another of those counters."
    );
    supported("Winding Constrictor");
    supported("Mycosynth Lattice");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Winding Constrictor");
    let jace = t.enter(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3, "not an artifact or creature");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Winding Constrictor");
    t.battlefield(P0, "Mycosynth Lattice");
    let jace = t.enter(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 4);
}
