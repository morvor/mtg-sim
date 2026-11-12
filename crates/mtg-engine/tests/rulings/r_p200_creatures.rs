//! Rulings batch P200 — creatures: Death's Shadow's life total (CR 119, 810.9a), Shanna
//! and Timberpack Wolf's counts (604.3, 704.5g), Faeburrow Elder, Bloom Tender and Glister
//! Bairn's colors (105.1), Mightstone, "enters with" counters that ignore creatures
//! entering at the same time (614.12), Gladewalker Ritualist and Chrome Replicator's
//! triggers (603.6a, 603.4), Idolized's "attacks alone" (506.5), South Wind Avatar's life
//! gain events (119.10, 603.10a, 810.9), mandatory enters-the-battlefield targets (603.3d),
//! The Foretold Soldier, Aladdin and Ali Baba, and the werewolves' transform triggers.

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, target_candidates};
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s20_common::tap_for_mana;
use crate::r_s31_common::wave;
use crate::r_s33_common::{life_gains_since, set_life, two_headed_giant};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Death's Shadow
// ---------------------------------------------------------------------------------------

#[test]
fn deaths_shadow_x_follows_the_life_total_and_is_zero_below_zero() {
    cr!("107.1b", "611.3a", "613.4c");
    ruling!(
        "Death's Shadow",
        "The value of X changes as you gain and lose life. It's not locked in as Death's Shadow enters the battlefield."
    );
    ruling!(
        "Death's Shadow",
        "If your life total is negative, X is considered to be 0."
    );
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    let shadow = t.battlefield(P0, "Death's Shadow");
    assert_eq!(t.pt(shadow), (3, 3));
    crate::r_p194_common::gain(&mut t, P0, 2);
    assert_eq!(t.pt(shadow), (1, 1));
    crate::r_p194_common::lose(&mut t, P0, 7);
    assert_eq!(t.pt(shadow), (8, 8));
    // Below zero (Platinum Angel keeps P0 in the game): 13/13, not bigger.
    t.battlefield(P0, "Platinum Angel");
    crate::r_p194_common::lose(&mut t, P0, 9);
    assert_eq!(t.life(P0), -4);
    assert_eq!(t.pt(shadow), (13, 13));
    assert!(!t.has_lost(P0));
}

#[test]
fn deaths_shadow_uses_the_team_life_total() {
    cr!("810.9a");
    ruling!(
        "Death's Shadow",
        "In a Two-Headed Giant game, your life total is your team's life total."
    );
    let mut t = two_headed_giant();
    assert_eq!(t.life(P0), 30);
    // Team life 30: a 13/13 that's -17/-17 dies.
    let shadow = t.battlefield(P0, "Death's Shadow");
    t.settle();
    assert!(!t.on_battlefield(shadow));
    // The teammate loses life: the shared total goes down for both.
    crate::r_p194_common::lose(&mut t, P1, 21);
    assert_eq!(t.life(P0), 9);
    let shadow = t.battlefield(P0, "Death's Shadow");
    assert_eq!(t.pt(shadow), (4, 4));
}

// ---------------------------------------------------------------------------------------
// Shanna and Timberpack Wolf
// ---------------------------------------------------------------------------------------

#[test]
fn shanna_counts_itself_and_is_zero_zero_elsewhere() {
    cr!("604.3", "611.3a");
    ruling!(
        "Shanna, Sisay's Legacy",
        "As long as it's on the battlefield, Shanna's last ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Shanna, Sisay's Legacy",
        "Shanna's ability that modifies its power and toughness applies only while it's on the battlefield. In all other zones, it's a 0/0 creature card."
    );
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Shanna, Sisay's Legacy");
    let gy = t.graveyard(P0, "Shanna, Sisay's Legacy");
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(hand), (0, 0));
    assert_eq!(t.pt(gy), (0, 0));
    let shanna = t.battlefield(P1, "Shanna, Sisay's Legacy");
    assert_eq!(t.pt(shanna), (1, 1));
}

/// P0's permanents `names`, the first a 3/3 because of the others (Shanna and two
/// creatures, or two Wolves and a Bears): it's dealt 2 damage and survives, then the
/// second one leaves and it dies.
fn damage_becomes_lethal(names: &[&str]) {
    let mut t = TestGame::new(2);
    let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
    let victim = ids[0];
    assert_eq!(t.pt(victim), (3, 3));
    crate::r_p194_common::shock(&mut t, P1, obj(victim));
    assert!(t.on_battlefield(victim));
    move_to(&mut t, ids[1], Zone::Hand(P0));
    assert!(!t.on_battlefield(victim), "{names:?}");
    assert_eq!(t.zone(victim), Zone::Graveyard(P0));
}

#[test]
fn damage_on_shanna_or_a_wolf_becomes_lethal_when_creatures_leave() {
    cr!("704.5g", "514.2");
    ruling!(
        "Shanna, Sisay's Legacy",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Shanna may become lethal if other creatures you control leave the battlefield during that turn."
    );
    ruling!(
        "Timberpack Wolf",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Timberpack Wolf may become lethal if other Timberpack Wolves leave the battlefield during that turn."
    );
    damage_becomes_lethal(&["Shanna, Sisay's Legacy", "Grizzly Bears", "Hill Giant"]);
    damage_becomes_lethal(&["Timberpack Wolf", "Timberpack Wolf", "Grizzly Bears"]);
}

#[test]
fn shanna_can_be_targeted_by_opponents_spells_but_not_abilities() {
    cr!("115.1", "113.1");
    ruling!(
        "Shanna, Sisay's Legacy",
        "Spells your opponents control can target Shanna."
    );
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let shanna = t.battlefield(P0, "Shanna, Sisay's Legacy");
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, shock).target(shanna).go();
    t.resolve_all();
    assert!(!t.on_battlefield(shanna));
    // P1's Prodigal Pyromancer can't target it.
    let mut t = TestGame::new(2);
    let shanna = t.battlefield(P0, "Shanna, Sisay's Legacy");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let cands = ability_targets(&mut t, pyro, 0);
    assert!(!cands.contains(&obj(shanna)));
    assert!(cands.contains(&Entity::Player(P0)));
}

// ---------------------------------------------------------------------------------------
// Colors among permanents
// ---------------------------------------------------------------------------------------

fn pool_of(t: &TestGame, p: PlayerId) -> Vec<(ManaType, u32)> {
    let pool = &t.g.player(p).mana_pool;
    [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G, ManaType::C]
        .into_iter()
        .map(|ty| (ty, pool.count(ty) as u32))
        .filter(|(_, n)| *n > 0)
        .collect()
}

#[test]
fn faeburrow_elder_counts_at_most_five_colors_and_at_least_its_own() {
    cr!("105.1", "105.2c", "604.3");
    ruling!(
        "Faeburrow Elder",
        "Faeburrow Elder's middle ability can give it at most +5/+5, that is, +1/+1 each for white, blue, black, red, and green. \"Gold,\" \"multicolor,\" and \"colorless\" aren't colors. Similarly, Faeburrow Elder's last ability can produce at most five mana."
    );
    ruling!(
        "Faeburrow Elder",
        "Since Faeburrow Elder is a green and white permanent, its middle ability usually gives it at least +2/+2 and its last ability usually produces at least {G}{W}."
    );
    supported("Child of Alara");
    // Alone: 2/2, {G}{W}.
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Faeburrow Elder");
    t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(elder), (2, 2));
    assert!(tap_for_mana(&mut t, P0, elder, "add"));
    assert_eq!(pool_of(&t, P0), vec![(ManaType::W, 1), (ManaType::G, 1)]);
    // With every color twice over, a gold and a colorless permanent: 5/5, five mana.
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Faeburrow Elder");
    t.battlefield(P0, "Child of Alara");
    t.battlefield(P0, "Tithe Drinker");
    t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(elder), (5, 5));
    assert!(tap_for_mana(&mut t, P0, elder, "add"));
    assert_eq!(
        pool_of(&t, P0),
        vec![
            (ManaType::W, 1),
            (ManaType::U, 1),
            (ManaType::B, 1),
            (ManaType::R, 1),
            (ManaType::G, 1)
        ]
    );
}

#[test]
fn bloom_tender_counts_a_permanent_for_each_of_its_colors() {
    cr!("105.2c", "106.1a");
    ruling!(
        "Bloom Tender",
        "For each color (white, blue, black, red, and green), check to see if you control a permanent of that color. You can count the same permanent for multiple colors."
    );
    supported("Tithe Drinker");
    let mut t = TestGame::new(2);
    let tender = t.battlefield(P0, "Bloom Tender");
    t.battlefield(P0, "Tithe Drinker");
    // An opponent's red permanent doesn't count.
    t.battlefield(P1, "Shivan Dragon");
    assert!(tap_for_mana(&mut t, P0, tender, "add"));
    assert_eq!(
        pool_of(&t, P0),
        vec![(ManaType::W, 1), (ManaType::B, 1), (ManaType::G, 1)]
    );
}

#[test]
fn glister_bairn_x_is_locked_in_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Glister Bairn",
        "The value of X is calculated only once, as Glister Bairn's ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glister Bairn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // Green and blue: +2/+2.
    assert_eq!(t.pt(bears), (4, 4));
    t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Tithe Drinker");
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn mightstone_pumps_every_players_attackers() {
    cr!("611.3a", "506.4");
    ruling!(
        "Mightstone",
        "The +1/+0 applies to attacking creatures from all players."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mightstone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(other), (2, 2));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------------
// Entering at the same time
// ---------------------------------------------------------------------------------------

#[test]
fn enters_with_counters_ignores_creatures_entering_with_it() {
    cr!("614.12", "614.1c", "603.6a");
    ruling!(
        "Crowd-Control Warden",
        "If Crowd-Control Warden enters the battlefield at the same time as one or more creatures, those creatures won't count for the purposes of Crowd-Control Warden's first ability."
    );
    ruling!(
        "Sheriff of Safe Passage",
        "If Sheriff of Safe Passage enters the battlefield under your control at the same time as one or more other creatures, it doesn’t take those creatures into consideration when determining how many +1/+1 counters it enters the battlefield with."
    );
    ruling!(
        "Squad Captain",
        "If Squad Captain somehow enters the battlefield at the same time as another creature you control, it doesn't count that other creature. It counts only creatures that are already on the battlefield."
    );
    ruling!(
        "Luxknight Breacher",
        "In the rare case where Luxknight Breacher enters at the same time as one or more other creatures and/or artifacts you control, it doesn’t count those other creatures and/or artifacts. It counts only creatures and/or artifacts that are already on the battlefield under your control."
    );
    supported("Genesis Wave");
    // (card, counters with one Grizzly Bears already there; Breacher also counts the
    // Mind Stone already there)
    for (name, expected) in [
        ("Crowd-Control Warden", 1),
        ("Sheriff of Safe Passage", 2),
        ("Squad Captain", 1),
        ("Luxknight Breacher", 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Mind Stone");
        // Entering with two creatures (one an artifact) and an artifact; Islands pad X up
        // to the card's mana value.
        let cards = wave(
            &mut t,
            &[name, "Grizzly Bears", "Memnite", "Mind Stone", "Island"],
        );
        assert!(t.on_battlefield(cards[0]), "{name}");
        assert_eq!(t.counters(cards[0], counters::PLUS1), expected, "{name}");
    }
}

#[test]
fn gladewalker_ritualists_entering_together_see_each_other() {
    cr!("603.6a", "603.2");
    ruling!(
        "Gladewalker Ritualist",
        "If multiple creatures named Gladewalker Ritualist all enter the battlefield under your control at the same time, each of their abilities will trigger once for each of the others."
    );
    supported("Genesis Wave");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    wave(
        &mut t,
        &[
            "Gladewalker Ritualist",
            "Gladewalker Ritualist",
            "Gladewalker Ritualist",
        ],
    );
    // Three Ritualists: each sees the two others, six draws.
    assert_eq!(t.hand_size(P0), hand + 6);
}

#[test]
fn chrome_replicator_one_token_however_many_pairs() {
    cr!("603.4", "608.2c");
    ruling!(
        "Chrome Replicator",
        "Chrome Replicator's ability creates just one Construct, no matter how many namesake pairs of permanents you control."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Chrome Replicator");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn chrome_replicator_checks_for_a_pair_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Chrome Replicator",
        "If you don't control two nonland, nontoken permanents with the same name immediately after Chrome Replicator enters the battlefield, its ability doesn't trigger. If you don't control two as the ability resolves, you don't create a token. They don't have to be the same pair, however."
    );
    // Two lands and two tokens with the same names don't count: no trigger.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    create_token(&mut t, P0, "Soldier");
    create_token(&mut t, P0, "Soldier");
    t.enter(P0, "Chrome Replicator");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A pair, broken up before it resolves: no token.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Mind Stone");
    t.enter(P0, "Chrome Replicator");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, stone, Zone::Hand(P0));
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Another pair by then: a token.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Mind Stone");
    t.enter(P0, "Chrome Replicator");
    t.settle();
    move_to(&mut t, stone, Zone::Hand(P0));
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Chrome Replicator itself can be one of the pair.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chrome Replicator");
    t.enter(P0, "Chrome Replicator");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------------------------------
// Idolized
// ---------------------------------------------------------------------------------------

#[test]
fn idolized_attacks_alone_counts_every_declared_attacker() {
    cr!("506.5", "805.10a");
    ruling!(
        "Idolized",
        "A creature attacks alone if it’s the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable)."
    );
    supported("Idolized");
    // Alone: +X/+X, X = nonland permanents P0 controls (Bears, Idolized, Mind Stone).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Idolized", bears);
    t.battlefield(P0, "Mind Stone");
    t.lands(P0, "Forest", 2);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // With another attacker removed from combat before blockers: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Idolized", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 0);
    move_to(&mut t, giant, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // Two-Headed Giant: the teammate's attacker counts.
    let mut t = two_headed_giant();
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Idolized", bears);
    let mate = t.battlefield(P1, "Hill Giant");
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(mate, Entity::Player(P2))]),
    );
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P2)), (mate, Entity::Player(P2))],
    );
    assert!(t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attackers.len() == 2));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn idolized_ignores_creatures_put_onto_the_battlefield_attacking() {
    cr!("506.5", "508.4");
    ruling!(
        "Idolized",
        "Similarly, creatures that enter the battlefield attacking later in combat won’t be considered when determining whether or not a creature attacked alone."
    );
    supported("Geist of Saint Traft");
    // Geist of Saint Traft attacks alone; its Angel token enters attacking: still alone.
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    crate::r_s06_common::attach_new(&mut t, P0, "Idolized", geist);
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let angel = tokens(&t, P0)[0];
    assert!(t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == angel)));
    // Idolized's ability triggered and resolved: +2/+2 (Geist, Idolized) or +3/+3 (and
    // the Angel token, if Geist's trigger resolved first).
    assert!(t.pt(geist).0 >= 4, "{:?}", t.pt(geist));
}

// ---------------------------------------------------------------------------------------
// South Wind Avatar
// ---------------------------------------------------------------------------------------

#[test]
fn south_wind_avatar_one_trigger_per_life_gain_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "South Wind Avatar",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and South Wind Avatar's last ability will trigger only once."
    );
    ruling!(
        "South Wind Avatar",
        "South Wind Avatar's last ability triggers just once for each life-gaining event, whether it's 1 life from Illegitimate Business or 3 life from Guac & Marshmallow Pizza."
    );
    supported("Congregate");
    // Congregate: "Target player gains 2 life for each creature on the battlefield."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "South Wind Avatar");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let from = t.g.turn_events.len();
    let c = t.hand(P0, "Congregate");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, c).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(life_gains_since(&t, from, P0), vec![6]);
    assert_eq!((t.life(P0), t.life(P1)), (26, 19));
    // A Food token's 3 life and a single point of life: one trigger each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "South Wind Avatar");
    let food = create_token(&mut t, P0, "Food");
    mana(&mut t, P0, ManaType::C, 2);
    activate_containing(&mut t, P0, food, "gain 3 life").unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (23, 19));
    crate::r_p194_common::gain(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 18));
}

#[test]
fn south_wind_avatar_ignores_the_teammates_life_gain() {
    cr!("810.9", "603.2");
    ruling!(
        "South Wind Avatar",
        "In a Two-Headed Giant game, life gained by your teammate won't cause South Wind Avatar's last ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "South Wind Avatar");
    crate::r_p194_common::gain(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 32);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P2), 30);
    // P0's own gain: each opponent loses 1.
    crate::r_p194_common::gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 34);
    assert_eq!(t.life(P2), 28);
}

#[test]
fn south_wind_avatar_uses_the_dead_creatures_last_toughness() {
    cr!("603.10a", "608.2h");
    ruling!(
        "South Wind Avatar",
        "Use the creature's toughness as it last existed on the battlefield to determine how much life you gain with South Wind Avatar's second ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "South Wind Avatar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 2, None);
    let growth = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------------------
// Mandatory targets
// ---------------------------------------------------------------------------------------

#[test]
fn mandatory_enters_triggers_target_your_own_permanents() {
    cr!("603.3d", "115.1");
    ruling!(
        "Nekrataal",
        "Nothing happens if there are no nonartifact, nonblack creatures on the battlefield when Nekrataal enters the battlefield, but if there is one, you must target it, even if it’s a creature you control."
    );
    ruling!(
        "Manic Vandal",
        "Manic Vandal’s ability is mandatory. If you’re the only player who controls an artifact, you must target one of them."
    );
    ruling!(
        "Vithian Renegades",
        "The \"enters\" ability is mandatory. If you're the only player who controls any artifacts, you must target one of them."
    );
    ruling!(
        "Oxidda Scrapmelter",
        "This ability is mandatory. If you’re the only player who controls any artifacts, you must target one of them."
    );
    ruling!(
        "Yeva's Forcemage",
        "Yeva’s Forcemage’s ability is mandatory, although you can choose Yeva’s Forcemage as the target."
    );
    // Nekrataal with no legal target: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Drudge Skeletons");
    t.enter(P0, "Nekrataal");
    t.resolve_all();
    assert_eq!(t.g.player(P1).graveyard.len(), 0);
    // Only P0's own creature: it's the target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Ornithopter");
    t.enter(P0, "Nekrataal");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // The artifact destroyers with only P0's artifact.
    for name in ["Manic Vandal", "Vithian Renegades", "Oxidda Scrapmelter"] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P0, "Mind Stone");
        t.battlefield(P1, "Grizzly Bears");
        t.enter(P0, name);
        t.resolve_all();
        assert_eq!(t.zone(stone), Zone::Graveyard(P0), "{name}");
    }
    // Yeva's Forcemage alone: it targets itself.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let yeva = t.enter(P0, "Yeva's Forcemage");
    t.settle();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(yeva))));
    t.resolve_all();
    assert_eq!(t.pt(yeva), (4, 4));
}

// ---------------------------------------------------------------------------------------
// The Foretold Soldier, Aladdin, Ali Baba
// ---------------------------------------------------------------------------------------

/// Whether `p` could cast the card (followed across zone changes) now.
fn castable(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == c))
}

#[test]
fn foretold_soldier_only_its_owner_casts_it() {
    cr!("702.143a", "108.3");
    ruling!(
        "The Foretold Soldier",
        "Regardless of who controlled The Foretold Soldier and exiled it with its triggered ability, only its owner may cast it from exile this way."
    );
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "The Foretold Soldier");
    give_control(&mut t, soldier, P1);
    t.g.objects[soldier.0 as usize].summoning_sick = false;
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(soldier, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.zone(soldier), Zone::Exile);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 2);
    // P1 can't cast it, on their own turn or later.
    assert!(!castable(&mut t, P1, soldier));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, soldier));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, soldier));
}

#[test]
fn aladdin_keeps_control_when_the_permanent_stops_being_an_artifact() {
    cr!("608.2b", "611.2b");
    ruling!(
        "Aladdin",
        "You do not lose control of the permanent if it stops being an artifact. The validity of the target is checked only on announcement and resolution of the ability."
    );
    supported("Liquimetal Coating");
    let mut t = TestGame::new(2);
    let aladdin = t.battlefield(P0, "Aladdin");
    let coating = t.battlefield(P0, "Liquimetal Coating");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, coating, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).is(CardType::Artifact));
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, aladdin, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).is(CardType::Artifact));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn ali_baba_taps_walls_while_tapped_or_sick_and_more_than_once() {
    cr!("602.5a", "302.6");
    ruling!(
        "Ali Baba",
        "The ability may tap walls even when Ali Baba is tapped or entered that turn, since the activation cost doesn't include {T}."
    );
    ruling!(
        "Ali Baba",
        "This ability may be used to tap more than one Wall per turn if you have enough mana, since tapping isn't part of the activation cost."
    );
    let mut t = TestGame::new(2);
    let ali = t.battlefield_sick(P0, "Ali Baba");
    t.g.objects[ali.0 as usize].tapped = true;
    let a = t.battlefield(P1, "Wall of Wood");
    let b = t.battlefield(P1, "Wall of Stone");
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, ali, 0, &[obj(a)]).unwrap();
    t.activate(P0, ali, 0, &[obj(b)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
}

// ---------------------------------------------------------------------------------------
// Werewolves
// ---------------------------------------------------------------------------------------

/// P0's Afflicted Deserter transforms at P1's upkeep (no spells were cast in P0's turn),
/// with the trigger targeting `target` and destroying it if `destroy` (asked as it
/// resolves). The decisions asked while it was on the stack are recorded.
fn ransack(t: &mut TestGame, target: ObjectId, yes: bool) -> ObjectId {
    let d = t.battlefield(P0, "Afflicted Deserter // Werewolf Ransacker");
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P0, yes);
    next_upkeep(t, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(d).chars.name, "Werewolf Ransacker");
    d
}

#[test]
fn werewolf_ransacker_damages_the_controller_of_a_destroyed_artifact_token() {
    cr!("111.7", "704.5d", "701.8a");
    ruling!(
        "Afflicted Deserter // Werewolf Ransacker",
        "An artifact token that's destroyed is put into its owner's graveyard before it ceases to exist. If a token is destroyed by Werewolf Ransacker ability, Werewolf Ransacker deals damage to that token's controller."
    );
    let mut t = TestGame::new(2);
    let treasure = create_token(&mut t, P1, "Treasure");
    ransack(&mut t, treasure, true);
    assert!(!t.g.is_live(treasure));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn werewolf_ransacker_no_damage_unless_the_artifact_reaches_a_graveyard() {
    cr!("701.8a", "614.6", "608.2c");
    ruling!(
        "Afflicted Deserter // Werewolf Ransacker",
        "If the targeted artifact has indestructible or regenerates (or you choose not to destroy it), Werewolf Ransacker doesn't deal damage to that artifact's controller. Similarly, if the targeted artifact is destroyed but a replacement effect moves it to a different zone instead of its owner's graveyard, Werewolf Ransacker doesn't deal damage to that artifact's controller."
    );
    ruling!(
        "Afflicted Deserter // Werewolf Ransacker",
        "You choose the target artifact when Werewolf Ransacker's first triggered ability goes on the stack. You choose whether or not to destroy the artifact when that ability resolves."
    );
    supported("Darksteel Relic");
    supported("Rest in Peace");
    // Indestructible.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    ransack(&mut t, relic, true);
    assert!(t.on_battlefield(relic));
    assert_eq!(t.life(P1), 20);
    // Declined as it resolves.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    ransack(&mut t, stone, false);
    assert!(t.on_battlefield(stone));
    assert_eq!(t.life(P1), 20);
    // Exiled instead (Rest in Peace).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let stone = t.battlefield(P1, "Mind Stone");
    ransack(&mut t, stone, true);
    assert_eq!(t.zone(stone), Zone::Exile);
    assert_eq!(t.life(P1), 20);
    // Destroyed: 3 damage.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    ransack(&mut t, stone, true);
    assert_eq!(t.zone(stone), Zone::Graveyard(P1));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn hinterland_scourge_must_be_blocked_if_able() {
    cr!("509.1c");
    ruling!(
        "Hinterland Hermit // Hinterland Scourge",
        "If Hinterland Scourge is attacking, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block it."
    );
    let mut t = TestGame::new(2);
    let scourge = crate::r_p079_common::back_face(&mut t, P0, "Hinterland Hermit // Hinterland Scourge");
    let scourge = t.g.current(scourge);
    t.g.objects[scourge.0 as usize].summoning_sick = false;
    assert_eq!(t.obj_now(scourge).chars.name, "Hinterland Scourge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(scourge, Entity::Player(P1))]);
    // P1 declares no blocks: not allowed; the Bears block the Scourge.
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn oakshade_stalker_flash_for_two_more_even_at_night() {
    cr!("702.8a", "601.2f", "731.1");
    ruling!(
        "Oakshade Stalker // Moonlit Ambusher",
        "You may cast Oakshade Stalker as though it had flash by paying {2} more even if it's night."
    );
    for (generic, ok) in [(4, true), (3, false)] {
        let mut t = TestGame::new(2);
        t.g.day = Some(false);
        t.set_step(P1, Step::PrecombatMain);
        let c = t.hand(P0, "Oakshade Stalker // Moonlit Ambusher");
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, generic);
        t.g.turn.priority = Some(P0);
        let uid = t
            .obj(c)
            .chars
            .abilities
            .iter()
            .find(|a| a.text.contains("as though it had flash"))
            .expect("flash ability")
            .uid;
        // Not without paying {2} more.
        assert!(t.cast(P0, c).try_go().is_err());
        t.g.turn.priority = Some(P0);
        let r = t
            .cast(P0, c)
            .method(mtg_engine::object::CastMethod::Alternative(uid))
            .try_go();
        assert_eq!(r.is_ok(), ok, "{generic}");
        if ok {
            t.resolve_all();
            assert!(t.on_battlefield(c));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Brutal Cathar
// ---------------------------------------------------------------------------------------

#[test]
fn brutal_cathar_exile_lasts_through_transforming_until_it_leaves() {
    cr!("610.3", "610.3a", "712.18");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "Brutal Cathar's ability creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after Brutal Cathar (or Moonrage Brute) leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let cathar = t.enter(P0, "Brutal Cathar // Moonrage Brute");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // It becomes night: Moonrage Brute; the Bears stay exiled.
    t.g.day = Some(true);
    t.advance_to(P1, Step::Upkeep);
    t.g.day = Some(false);
    t.g.recompute();
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(cathar).chars.name, "Moonrage Brute");
    assert_eq!(t.zone(bears), Zone::Exile);
    move_to(&mut t, cathar, Zone::Graveyard(P0));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn brutal_cathar_returns_the_card_with_nothing_in_between() {
    cr!("610.3", "610.3c", "707.5");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "The exiled card returns to the battlefield immediately after this creature leaves the battlefield. Nothing happens between the two events, including state-based actions. The two creatures aren't on the battlefield at the same time. For example, if the returning creature is a Clone, it can't enter the battlefield as a copy of Brutal Cathar or Moonrage Brute."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    t.answer_targets(P0, &[obj(clone)]);
    let cathar = t.enter(P0, "Brutal Cathar // Moonrage Brute");
    t.resolve_all();
    assert_eq!(t.zone(clone), Zone::Exile);
    let from = t.asked().len();
    let cathar_now = t.g.current(cathar);
    // As the Clone returns, P1 has it copy the Bears (Cathar isn't among the choices).
    t.answer_choose(P1, &[obj(bears)]);
    move_to(&mut t, cathar, Zone::Graveyard(P0));
    let cands = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(cands.iter().all(|c| !c.contains(&obj(cathar_now))), "{cands:?}");
    assert!(t.on_battlefield(clone));
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
}

#[test]
fn brutal_cathar_card_returns_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "In a multiplayer game, if Brutal Cathar's owner leaves the game, the exiled card will return to the battlefield."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Brutal Cathar // Moonrage Brute");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.perform_action(P0, Action::Concede).unwrap();
    t.settle();
    assert!(t.has_lost(P0));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}
