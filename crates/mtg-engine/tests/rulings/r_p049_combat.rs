//! Rulings batch P049 — combat: attack triggers that don't care what happens to the
//! attackers afterwards (Aurelia, Oath of Kaya once per planeswalker, Campaign of
//! Vengeance's defending player), Kaito's non-additive extra activation, Obelisk Spider's
//! combat damage trigger and its own death, Souls of the Faultless attacking and in
//! Two-Headed Giant (CR 805.10c), Axelrod Gunnarson's look-back, and Bontu's attack
//! restriction.

use crate::r_p049_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s06_common::damage as damage_from;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The life `p` has gained this turn, as the game's history records it.
fn life_gained(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.life_gained.get(&p).copied().unwrap_or(0)
}

#[test]
fn aurelias_triggers_dont_care_what_happens_to_the_attackers() {
    cr!("603.2", "608.2", "508.1");
    ruling!(
        "Aurelia, the Law Above",
        "For both triggered abilities, it doesn't matter what happens to the attacking creatures in response. As long as a player attacked with at least the appropriate number of creatures, the effects of Aurelia's triggered abilities will still occur."
    );
    supported("Aurelia, the Law Above");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aurelia, the Law Above");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    // Two of the three attackers leave in response.
    exile_now(&mut t, a);
    exile_now(&mut t, b);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn oath_of_kaya_triggers_once_for_each_planeswalker_attacked() {
    cr!("603.2c", "508.1b"); // CR 603.2c: once per event; each planeswalker attacked.
    ruling!(
        "Oath of Kaya",
        "If an opponent attacks more than one planeswalker you control, the last ability of Oath of Kaya triggers once for each of those attacked planeswalkers."
    );
    supported("Oath of Kaya");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Kaya");
    let jace = t.battlefield(P0, "Jace Beleren");
    let liliana = t.battlefield(P0, "Liliana Vess");
    t.set_step(P1, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(a, obj(jace)), (b, obj(liliana)), (c, obj(jace))]);
    assert_eq!(triggers_on_stack(&t, "2 damage"), 2);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn campaign_of_vengeance_drains_the_controller_of_an_attacked_planeswalker() {
    cr!("506.2", "508.1b");
    ruling!(
        "Campaign of Vengeance",
        "If a creature attacks a planeswalker, the defending player is the player who controls that planeswalker."
    );
    supported("Campaign of Vengeance");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Campaign of Vengeance");
    let jace = t.battlefield(P2, "Jace Beleren");
    let bear = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bear, obj(jace))]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 20, 19));
}

#[test]
fn kaitos_extra_activation_isnt_additive() {
    cr!("606.3", "603.2", "510.4");
    ruling!(
        "Kaito, Dancing Shadow",
        "you won't be able to activate Kaito's loyalty abilities more than twice that turn. That is, Kaito's triggered ability is not additive."
    );
    supported("Kaito, Dancing Shadow");
    supported("Youthful Knight");
    let mut t = TestGame::new(2);
    let kaito = t.battlefield(P0, "Kaito, Dancing Shadow");
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Both attack unblocked: combat damage in the first-strike step (the Knight) and in
    // the regular step (the Bears), a trigger each; each returns the creature that dealt
    // the damage.
    for c in [knight, bears] {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
    }
    t.attack(&[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_hand(P0, "Youthful Knight"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Two activations this turn, not three.
    t.advance_to(P0, Step::PostcombatMain);
    for _ in 0..2 {
        assert!(can_activate(&mut t, P0, kaito));
        t.activate(P0, kaito, 1, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.hand_size(P0), 2 + 2);
    assert!(!can_activate(&mut t, P0, kaito));
    assert!(t.activate(P0, kaito, 1, &[]).is_err());
}

/// P0's Obelisk Spider blocks P1's attacking `attacker`; combat damage is dealt and
/// settled (state-based actions and triggers on the stack), nothing resolved.
fn spider_blocks(attacker_name: &str) -> (TestGame, ObjectId, ObjectId) {
    supported("Obelisk Spider");
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Obelisk Spider");
    t.set_step(P1, Step::PrecombatMain);
    let attacker = t.battlefield(P1, attacker_name);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(spider, attacker)]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    (t, spider, attacker)
}

#[test]
fn obelisk_spiders_counter_can_make_marked_damage_lethal() {
    cr!("704.5g", "122.1a", "510.2");
    ruling!(
        "Obelisk Spider",
        "the counter Obelisk Spider’s first triggered ability puts on a creature may cause previously marked damage to become lethal."
    );
    // A 2/2 is dealt 1 by the Spider, then gets a -1/-1 counter: 1 damage is lethal.
    let (mut t, spider, bears) = spider_blocks("Grizzly Bears");
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(triggers_on_stack(&t, "-1/-1 counter on that creature"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(spider));
    // The second ability triggered for the counter: a drain of 1.
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn obelisk_spider_dying_as_it_deals_damage_still_puts_the_counter_but_doesnt_drain() {
    cr!("603.10a", "704.5g", "603.2f");
    ruling!(
        "Obelisk Spider",
        "If Obelisk Spider deals combat damage to a creature at the same time Obelisk Spider is dealt lethal damage, its first triggered ability will put a -1/-1 counter on the other creature. However, because Obelisk Spider has left the battlefield before that ability resolves, its last ability won’t trigger."
    );
    // Serra Angel (4/4) deals lethal damage to the 1/4 Spider, which deals it 1.
    let (mut t, spider, angel) = spider_blocks("Serra Angel");
    assert!(t.in_graveyard(P0, "Obelisk Spider"));
    assert!(!t.on_battlefield(spider));
    assert_eq!(triggers_on_stack(&t, "-1/-1 counter on that creature"), 1);
    t.resolve_all();
    assert_eq!(t.counters(angel, counters::MINUS1), 1);
    assert_eq!(t.pt(angel), (3, 3));
    assert!(t.on_battlefield(angel));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn obelisk_spider_puts_no_counter_on_a_creature_that_has_left() {
    cr!("608.2b", "400.7"); // No target: the creature is just gone.
    ruling!(
        "Obelisk Spider",
        "If Obelisk Spider deals combat damage to a creature but that creature isn’t on the battlefield as Obelisk Spider’s first triggered ability resolves, most likely because Obelisk Spider killed that creature already, that ability won’t put a -1/-1 counter on it."
    );
    // The Spider's 1 damage kills a 1/1 Llanowar Elves: no counter, no drain.
    let (mut t, spider, elves) = spider_blocks("Llanowar Elves");
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(triggers_on_stack(&t, "-1/-1 counter on that creature"), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(elves).counter(counters::MINUS1), 0);
    assert!(t.on_battlefield(spider));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn souls_of_the_faultless_attacking_drains_its_own_controller() {
    cr!("506.2", "119.3");
    ruling!(
        "Souls of the Faultless",
        "If Souls of the Faultless somehow attacks and is dealt combat damage, you will both gain and lose life when its ability resolves."
    );
    supported("Souls of the Faultless");
    let mut t = TestGame::new(2);
    let souls = t.battlefield(P0, "Souls of the Faultless");
    // It loses defender for the turn and attacks; P1 blocks with a 2/2.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        souls,
        vec![Modification::RemoveKeyword(KeywordKind::Defender)],
    );
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, souls));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(souls, Entity::Player(P1))], &[(bears, souls)]);
    t.resolve_all();
    // P0 is the attacking player: P0 gained 2 and lost 2.
    assert_eq!(life_gained(&t, P0), 2);
    assert_eq!(life_lost(&t, P0), 2);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn souls_of_the_faultless_in_two_headed_giant_drains_the_controller_of_the_creature_it_blocks() {
    cr!("805.10c", "810.7", "810.9");
    // The 2006 ruling ("you may choose either member of the attacking team") predates
    // CR 805.10c; it's outdated (docs/rulings-exemptions/rulings-P049.tsv).
    supported("Souls of the Faultless");
    let mut t = two_headed_giant();
    let souls = t.battlefield(P0, "Souls of the Faultless");
    // P2's team's turn: P2's Grizzly Bears attacks; P3's Hill Giant attacks too.
    t.set_step(P2, Step::PrecombatMain);
    let bears = t.battlefield(P2, "Grizzly Bears");
    let giant = t.battlefield(P3, "Hill Giant");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P0, &[(souls, bears)]);
    t.resolve_all();
    // The attacking player for the blocking Souls is the Bears' controller, P2.
    assert_eq!(life_lost(&t, P2), 2);
    assert_eq!(life_lost(&t, P3), 0);
    assert_eq!(life_gained(&t, P0), 2);
    // Team life totals: P0's team took 3 from the Giant and gained 2; P2's team lost 2.
    assert_eq!(t.life(P0), 29);
    assert_eq!(t.life(P2), 28);
}

#[test]
fn axelrod_gunnarson_triggers_for_any_creature_it_damaged_this_turn_that_dies() {
    cr!("603.2", "603.10a", "700.4");
    ruling!(
        "Axelrod Gunnarson",
        "Each time a creature is put into a graveyard from play, check whether Axelrod Gunnarson had dealt any damage to it at any time during that turn. (This includes combat damage.) If so, Axelrod Gunnarson's second ability will trigger. It doesn't matter who controlled the creature or whose graveyard it was put into."
    );
    supported("Axelrod Gunnarson");
    let mut t = TestGame::new(2);
    let axelrod = t.battlefield(P0, "Axelrod Gunnarson");
    // Noncombat damage to P0's own creature, which then dies: a trigger.
    let own = t.battlefield(P0, "Hill Giant");
    damage_from(&mut t, axelrod, 1, own);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, own);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Combat damage to a blocker that dies of it.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(axelrod, Entity::Player(P1))], &[(giant, axelrod)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 22);
    // A creature it didn't damage dying: no trigger.
    let other = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, other);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 0);
}

#[test]
fn axelrod_gunnarson_dying_with_a_creature_it_damaged_still_triggers() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Axelrod Gunnarson",
        "If Axelrod Gunnarson and a creature it dealt damage to are both put into a graveyard at the same time, Axelrod Gunnarson's second ability will trigger."
    );
    supported("Axelrod Gunnarson");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    let axelrod = t.battlefield(P0, "Axelrod Gunnarson");
    let angel = t.battlefield(P1, "Serra Angel");
    damage_from(&mut t, axelrod, 1, angel);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Axelrod Gunnarson"));
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn bontu_can_attack_only_if_a_creature_died_under_your_control_before_attackers() {
    cr!("508.1", "508.1a", "700.4");
    ruling!(
        "Bontu the Glorified",
        "All attackers are chosen at once. You can't attack with one, sacrifice it to satisfy Bontu's combat restriction, and then attack with Bontu."
    );
    supported("Bontu the Glorified");
    let mut t = TestGame::new(2);
    let bontu = t.battlefield(P0, "Bontu the Glorified");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    // Nothing has died: Bontu can't be declared as an attacker, though a creature it
    // could sacrifice is there.
    assert!(!can_attack(&mut t, bontu));
    assert!(can_attack(&mut t, bears));
    // Sacrificing the Bears to Bontu's ability before attackers are declared lets Bontu
    // attack.
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, bontu, "Sacrifice another creature").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(can_attack(&mut t, bontu));
    ruling!(
        "Bontu the Glorified",
        "Creature tokens that die are put into your graveyard as normal (and cease to exist soon after). If one died this turn, it satisfies Bontu's combat restriction."
    );
    let mut t = TestGame::new(2);
    let bontu = t.battlefield(P0, "Bontu the Glorified");
    let token = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    assert!(!t.on_battlefield(token));
    assert_eq!(t.graveyard_size(P0), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bontu));
}
