//! Counter grammar II (`oracle/patterns/counters_grammar_2.rs`, `kw/counter_groups.rs`,
//! CR 122): a counter of a kind chosen from a list or at random, "all but one", counters
//! removed or moved from among a group, "a counter of each kind not on ~", counters copied
//! from what a triggering event put, and doubling a player's counters.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn elspeth_resplendent_puts_a_p1p1_counter_and_the_chosen_keyword_counter() {
    cr!("122.1b", "606.3");
    supported("Elspeth Resplendent");
    let mut t = TestGame::new(2);
    let elspeth = t.battlefield(P0, "Elspeth Resplendent");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Options: flying, first strike, lifelink, vigilance; choose lifelink.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, elspeth, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "lifelink"), 1);
    assert_eq!(t.counters(bears, "flying"), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn aragorn_copies_one_of_each_kind_put_on_him_onto_another_creature() {
    cr!("122.6", "603.2");
    supported("Aragorn, Company Leader");
    let mut t = TestGame::new(2);
    let aragorn = t.battlefield(P0, "Aragorn, Company Leader");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    // Three +1/+1 counters put on Aragorn by P0: one +1/+1 counter on the Bears.
    add(&mut t, aragorn, "+1/+1", 3);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // A flying counter: a flying counter.
    t.answer_targets(P0, &[obj(bears)]);
    add(&mut t, aragorn, "flying", 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, "flying"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn crystalline_giant_gets_a_kind_it_does_not_have() {
    cr!("122.1b", "706.1");
    ruling!(
        "Crystalline Giant",
        "As long as Crystalline Giant doesn't have all ten of the listed counters on it, you'll put one on it"
    );
    supported("Crystalline Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Crystalline Giant");
    // Nine of the ten kinds: it can only get the tenth (reach).
    for k in [
        "flying",
        "first strike",
        "deathtouch",
        "hexproof",
        "lifelink",
        "menace",
        "trample",
        "vigilance",
        "+1/+1",
    ] {
        add(&mut t, giant, k, 1);
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(giant, "reach"), 1);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    // With none, exactly one counter of one of the kinds.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Crystalline Giant");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let total: u32 = t.obj_now(giant).counters.values().sum();
    assert_eq!(total, 1);
    // With all ten, nothing more is put on it.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Crystalline Giant");
    for k in [
        "flying",
        "first strike",
        "deathtouch",
        "hexproof",
        "lifelink",
        "menace",
        "reach",
        "trample",
        "vigilance",
        "+1/+1",
    ] {
        add(&mut t, giant, k, 1);
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let total: u32 = t.obj_now(giant).counters.values().sum();
    assert_eq!(total, 10);
}

#[test]
fn lily_bowen_removes_all_but_one_counter_and_gains_life_for_each() {
    cr!("122.1", "608.2c");
    supported("Lily Bowen, Raging Grandma");
    let mut t = TestGame::new(2);
    let lily = t.battlefield(P0, "Lily Bowen, Raging Grandma");
    add(&mut t, lily, "+1/+1", 20);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(lily, "+1/+1"), 1);
    assert_eq!(t.life(P0), 20 + 19);
    // Power 16 or less: doubled instead.
    let mut t = TestGame::new(2);
    let lily = t.battlefield(P0, "Lily Bowen, Raging Grandma");
    add(&mut t, lily, "+1/+1", 5);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(lily, "+1/+1"), 10);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn eventides_shadow_removes_counters_from_any_permanents_and_draws_that_many() {
    cr!("122.1", "608.2c");
    ruling!(
        "Eventide's Shadow",
        "allows you to remove counters from permanents controlled by any player"
    );
    supported("Eventide's Shadow");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    add(&mut t, mine, "+1/+1", 2);
    add(&mut t, theirs, "+1/+1", 3);
    t.lands(P0, "Swamp", 5);
    // One from my Bears, all three from their Giant.
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    let hand = t.hand_size(P0);
    let s = t.hand(P0, "Eventide's Shadow");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn overseer_of_vault_76_removes_exactly_three_quest_counters_from_among_permanents() {
    cr!("122.1", "603.12");
    ruling!(
        "Overseer of Vault 76",
        "lets you choose to remove exactly three quest counters"
    );
    supported("Overseer of Vault 76");
    let mut t = TestGame::new(2);
    let overseer = t.battlefield(P0, "Overseer of Vault 76");
    let other = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, overseer, "quest", 2);
    add(&mut t, other, "quest", 4);
    t.answer_yes(P0, true);
    // Two from the Overseer: the third must come from the Bears.
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(overseer, "quest"), 0);
    assert_eq!(t.counters(other, "quest"), 3);
    assert_eq!(t.counters(overseer, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 1);
    // Fewer than three: nothing is removed and the reflexive ability doesn't trigger.
    let mut t = TestGame::new(2);
    let overseer = t.battlefield(P0, "Overseer of Vault 76");
    add(&mut t, overseer, "quest", 2);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(overseer, "quest"), 2);
    assert_eq!(t.counters(overseer, "+1/+1"), 0);
}

#[test]
fn slippery_bogbonder_moves_counters_from_among_creatures_onto_the_target() {
    cr!("122.5");
    ruling!(
        "Slippery Bogbonder",
        "You may move counters from any number of other creatures you control"
    );
    supported("Slippery Bogbonder");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let target = t.battlefield(P0, "Craw Wurm");
    add(&mut t, a, "+1/+1", 2);
    add(&mut t, b, "flying", 1);
    t.answer_targets(P0, &[obj(target)]);
    // Both +1/+1 counters from the Bears, the flying counter from the Giant.
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.enter(P0, "Slippery Bogbonder");
    t.resolve_all();
    assert_eq!(t.counters(target, "hexproof"), 1);
    assert_eq!(t.counters(target, "+1/+1"), 2);
    assert_eq!(t.counters(target, "flying"), 1);
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(b, "flying"), 0);
}

#[test]
fn galloping_lizrog_puts_twice_the_removed_counters_on_itself() {
    cr!("122.1", "608.2c");
    ruling!(
        "Galloping Lizrog",
        "You remove counters and put new ones on Galloping Lizrog all while its ability is resolving."
    );
    supported("Galloping Lizrog");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add(&mut t, a, "+1/+1", 2);
    add(&mut t, b, "+1/+1", 1);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let lizrog = t.enter(P0, "Galloping Lizrog");
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(b, "+1/+1"), 0);
    assert_eq!(t.counters(lizrog, "+1/+1"), 6);
}

#[test]
fn gatewatch_beacon_moves_a_loyalty_counter_onto_the_entering_planeswalker() {
    cr!("122.5");
    supported("Gatewatch Beacon");
    let mut t = TestGame::new(2);
    let beacon = t.enter(P0, "Gatewatch Beacon");
    assert_eq!(t.counters(beacon, "loyalty"), 3);
    t.answer_yes(P0, true);
    let jace = t.enter(P0, "Jace, Mirror Mage");
    t.resolve_all();
    assert_eq!(t.counters(beacon, "loyalty"), 2);
    assert_eq!(t.counters(jace, "loyalty"), 5);
}

#[test]
fn bold_plagiarist_gets_the_same_counters_put_by_the_opponent() {
    cr!("122.6", "603.2");
    ruling!(
        "Bold Plagiarist",
        "Your opponent places the counters on both permanents."
    );
    supported("Bold Plagiarist");
    let mut t = TestGame::new(2);
    let plag = t.battlefield(P0, "Bold Plagiarist");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    // The opponent puts two +1/+1 counters on their creature.
    add(&mut t, theirs, "+1/+1", 2);
    t.resolve_all();
    assert_eq!(t.counters(plag, "+1/+1"), 2);
    // Counters the opponent puts on my creature don't count, nor do mine on mine.
    t.g.add_counters(obj(mine), "flying", 1, Some(theirs));
    add(&mut t, mine, "+1/+1", 1);
    t.resolve_all();
    assert_eq!(t.counters(plag, "flying"), 0);
    assert_eq!(t.counters(plag, "+1/+1"), 2);
    // A second Plagiarist the opponent controls doesn't trigger on the copied counters.
    let mut t = TestGame::new(2);
    let mine_p = t.battlefield(P0, "Bold Plagiarist");
    let theirs_p = t.battlefield(P1, "Bold Plagiarist");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, theirs, "+1/+1", 1);
    t.resolve_all();
    assert_eq!(t.counters(mine_p, "+1/+1"), 1);
    assert_eq!(t.counters(theirs_p, "+1/+1"), 0);
}

#[test]
fn captain_marvel_copies_counters_put_on_another_non_kree_creature() {
    cr!("122.6", "603.4");
    supported("Captain Marvel, Apex Avenger");
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Captain Marvel, Apex Avenger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    add(&mut t, bears, "+1/+1", 3);
    t.resolve_all();
    assert_eq!(t.counters(cap, "+1/+1"), 3);
}

#[test]
fn sensational_spider_man_removes_up_to_three_stun_counters_and_draws() {
    cr!("122.1", "608.2c");
    ruling!(
        "Sensational Spider-Man",
        "The stun counters don't need to be removed from the same permanent."
    );
    supported("Sensational Spider-Man");
    let mut t = TestGame::new(2);
    let spidey = t.battlefield(P0, "Sensational Spider-Man");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    add(&mut t, b, "stun", 2);
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    // One from the Bears (just put there), two from the Giant.
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let hand = t.hand_size(P0);
    t.attack(&[(spidey, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.obj_now(a).tapped);
    assert_eq!(t.counters(a, "stun"), 0);
    assert_eq!(t.counters(b, "stun"), 0);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn etched_slith_removes_a_counter_from_another_target_permanent() {
    cr!("115.1", "603.12");
    supported("Etched Slith");
    let mut t = TestGame::new(2);
    let slith = t.battlefield(P0, "Etched Slith");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, theirs, "+1/+1", 2);
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_yes(P0, true);
    t.attack(&[(slith, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.counters(slith, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
}

#[test]
fn aetheric_amplifier_doubles_each_kind_of_counter_its_controller_has() {
    cr!("122.1", "701.10e");
    ruling!(
        "Aetheric Amplifier",
        "To double the number of a kind of counters on a permanent"
    );
    supported("Aetheric Amplifier");
    let mut t = TestGame::new(2);
    let amp = t.battlefield(P0, "Aetheric Amplifier");
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    t.g.add_counters(Entity::Player(P0), "poison", 1, None);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, amp, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 6);
    assert_eq!(t.g.player(P0).counter("poison"), 2);
}

#[test]
fn goldberry_moves_a_counter_of_each_missing_kind_and_moves_some_away() {
    cr!("122.5");
    ruling!(
        "Goldberry, River-Daughter",
        "To move a counter from one permanent to another"
    );
    supported("Goldberry, River-Daughter");
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P0, "Goldberry, River-Daughter");
    let other = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, gold, "+1/+1", 1);
    add(&mut t, other, "+1/+1", 2);
    add(&mut t, other, "flying", 2);
    add(&mut t, other, "stun", 1);
    t.activate(P0, gold, 0, &[obj(other)]).unwrap();
    t.resolve_all();
    // One flying and one stun counter (Goldberry already has a +1/+1 counter).
    assert_eq!(t.counters(gold, "flying"), 1);
    assert_eq!(t.counters(gold, "stun"), 1);
    assert_eq!(t.counters(gold, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 2);
    assert_eq!(t.counters(other, "flying"), 1);
    assert_eq!(t.counters(other, "stun"), 0);
    // Move one or more counters away and draw.
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P0, "Goldberry, River-Daughter");
    let other = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, gold, "+1/+1", 2);
    t.lands(P0, "Island", 1);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let hand = t.hand_size(P0);
    t.activate(P0, gold, 1, &[obj(other)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(other, "+1/+1"), 2);
    assert_eq!(t.counters(gold, "+1/+1"), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    // With no counters to move, no card is drawn.
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P0, "Goldberry, River-Daughter");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, gold, 1, &[obj(other)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn exotic_pets_puts_a_counter_of_each_kind_among_creatures_on_the_fish() {
    cr!("122.1", "111.1");
    supported("Exotic Pets");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add(&mut t, a, "+1/+1", 3);
    add(&mut t, b, "flying", 1);
    add(&mut t, b, "+1/+1", 1);
    t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let pets = t.hand(P0, "Exotic Pets");
    t.cast(P0, pets).go();
    t.resolve_all();
    let fish = t.named_on_battlefield("Fish Token");
    assert_eq!(fish.len(), 2);
    let plus: u32 = fish.iter().map(|f| t.counters(*f, "+1/+1")).sum();
    let flying: u32 = fish.iter().map(|f| t.counters(*f, "flying")).sum();
    assert_eq!((plus, flying), (1, 1));
}

#[test]
fn the_caves_of_androzani_may_add_a_counter_of_a_chosen_kind_to_each_permanent() {
    cr!("122.1", "714.2b");
    supported("The Caves of Androzani");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    add(&mut t, a, "+1/+1", 1);
    add(&mut t, b, "stun", 1);
    add(&mut t, b, "+1/+1", 1);
    let caves = t.battlefield(P0, "The Caves of Androzani");
    add(&mut t, caves, "lore", 1);
    // The Bears: yes. The Giant: choose its stun counter (second option), then yes.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(caves, "lore"), 2);
    assert_eq!(t.counters(a, "+1/+1"), 2);
    let stun = t.counters(b, "stun");
    let plus = t.counters(b, "+1/+1");
    assert_eq!(stun + plus, 3);
}

#[test]
fn jace_mirror_mage_removes_loyalty_equal_to_the_drawn_cards_mana_value() {
    cr!("122.1", "606.4");
    ruling!(
        "Jace, Mirror Mage",
        "If the mana value of the drawn card is greater than Jace's loyalty, you remove all loyalty counters from him."
    );
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Mirror Mage");
    assert_eq!(t.counters(jace, "loyalty"), 4);
    t.library_top(P0, "Craw Wurm");
    t.activate(P0, jace, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jace, "loyalty"), 0);
    assert!(!t.on_battlefield(jace));
}

#[test]
fn kathril_puts_one_keyword_counter_per_keyword_in_the_graveyard() {
    cr!("122.1b", "608.2c");
    ruling!(
        "Kathril, Aspect Warper",
        "You check once for each keyword. If you have two creature cards with flying in your graveyard, you put just one flying counter on a creature you control, not two."
    );
    ruling!(
        "Kathril, Aspect Warper",
        "The counters don't need to all be put on the same creature."
    );
    supported("Kathril, Aspect Warper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Serra Angel");
    t.graveyard(P0, "Wind Drake");
    t.graveyard(P0, "Typhoid Rats");
    // Flying on the Bears, then deathtouch and vigilance on Kathril (by default).
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let k = t.enter(P0, "Kathril, Aspect Warper");
    t.resolve_all();
    use mtg_engine::keywords::KeywordKind;
    assert_eq!(t.counters(bears, "flying"), 1);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    let on_k = t.counters(k, "deathtouch") + t.counters(k, "vigilance");
    let on_b = t.counters(bears, "deathtouch") + t.counters(bears, "vigilance");
    assert_eq!(on_k + on_b, 2);
    assert_eq!(t.counters(k, "lifelink") + t.counters(bears, "lifelink"), 0);
    // Three counters were put on creatures this way.
    assert_eq!(t.counters(k, "+1/+1"), 3);
}
