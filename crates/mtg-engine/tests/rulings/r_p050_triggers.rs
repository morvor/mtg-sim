//! Rulings batch P050 — how drain-life triggered abilities trigger: intervening "if"
//! clauses (CR 603.4), look-back for sacrifices and leaving the battlefield (CR 603.10a),
//! one trigger per event or per object (CR 603.2c), when they're put on the stack
//! relative to the spell that caused them (CR 603.3), and how players who lose the game
//! at the same time never see them (CR 104.3b, 800.4a).

use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::{attack_with, give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, tokens_with_subtype};
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

#[test]
fn gloom_sower_uses_the_blockers_last_controller() {
    cr!("608.2h");
    ruling!(
        "Gloom Sower",
        "If the creature that blocked Gloom Sower leaves the battlefield before Gloom Sower's triggered ability resolves, the player who controlled it before it left is the player who loses 2 life."
    );
    supported("Gloom Sower");
    let mut t = TestGame::new(2);
    let sower = t.battlefield(P0, "Gloom Sower");
    let bears = t.battlefield(P1, BEARS);
    attack_with(&mut t, &[(sower, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, sower)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn trickster_gods_heist_same_controller_does_nothing() {
    cr!("701.12a");
    ruling!(
        "The Trickster-God's Heist",
        "If the same player controls both target permanents as the chapter I or chapter II ability resolves, nothing happens."
    );
    supported("The Trickster-God's Heist");
    // Both targets controlled by P1 by the time chapter I resolves: control doesn't change.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, BEARS);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(theirs)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "The Trickster-God's Heist");
    assert_eq!(t.stack_len(), 1);
    // In response, P1 gains control of P0's Bears.
    gain_control(&mut t, mine, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.obj_now(theirs).controller, P1);
    // Different controllers: they're exchanged.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, BEARS);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(theirs)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "The Trickster-God's Heist");
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.obj_now(theirs).controller, P0);
}

/// `who` gains control of `id` indefinitely (as a resolving effect would).
fn gain_control(t: &mut TestGame, id: ObjectId, who: PlayerId) {
    let id = t.g.current(id);
    crate::r_s05_common::run_from(
        t,
        who,
        None,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![id])),
            who: PlayerRef::Player(who),
            duration: Duration::Permanent,
        },
        &[],
    );
}

#[test]
fn parasitic_impetus_on_your_own_creature() {
    cr!("104.3b", "608.2c");
    ruling!(
        "Parasitic Impetus",
        "If you are the attacking creature's controller, you lose 2 life and gain 2 life. You won't lose the game if your life total is 0 in between these two events."
    );
    supported("Parasitic Impetus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, BEARS);
    give_mana_for(&mut t, P0, "Parasitic Impetus");
    let aura = t.hand(P0, "Parasitic Impetus");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    t.g.players[0].life = 2;
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 1);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 2);
}

#[test]
fn parasitic_impetus_you_gain_the_life() {
    cr!("603.3a");
    ruling!(
        "Parasitic Impetus",
        "Parasitic Impetus causes you to gain life, not the attacking creature's controller."
    );
    supported("Parasitic Impetus");
    // P0's Impetus on P1's creature; P1 attacks P0 on P1's turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, BEARS);
    give_mana_for(&mut t, P0, "Parasitic Impetus");
    let aura = t.hand(P0, "Parasitic Impetus");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn parasitic_strix_intervening_if() {
    cr!("603.4");
    ruling!(
        "Parasitic Strix",
        "If you don't control a black permanent immediately after Parasitic Strix enters the battlefield, its ability doesn't trigger."
    );
    supported("Parasitic Strix");
    // No black permanent: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    enter(&mut t, P0, "Parasitic Strix");
    assert_eq!(t.stack_len(), 0);
    // A black permanent that's gone as it resolves: no life changes.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Neonate");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    enter(&mut t, P0, "Parasitic Strix");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, v);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // A different black permanent as it resolves is fine.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vampire Neonate");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    enter(&mut t, P0, "Parasitic Strix");
    t.battlefield(P0, "Walking Corpse");
    destroy(&mut t, v);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn parasitic_strix_triggers_once() {
    cr!("603.4");
    ruling!(
        "Parasitic Strix",
        "Parasitic Strix's ability triggers only once, no matter how many black permanents you control beyond the first."
    );
    supported("Parasitic Strix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vampire Neonate");
    t.battlefield(P0, "Walking Corpse");
    t.battlefield(P0, "Night Market Lookout");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    enter(&mut t, P0, "Parasitic Strix");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

/// Advances from P0's postcombat main phase into P0's end step (its "at the beginning of
/// the end step" triggers on the stack).
fn into_end_step(t: &mut TestGame) {
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
}

#[test]
fn syndicate_heavy_checks_life_gained_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Syndicate Heavy",
        "If you haven't gained 4 or more life by the time an end step begins, Syndicate Heavy's last ability won't trigger at all."
    );
    supported("Syndicate Heavy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndicate Heavy");
    t.g.gain_life(P0, 3);
    into_end_step(&mut t);
    assert_eq!(t.stack_len(), 0, "3 life isn't enough");
    // Gaining more during the end step is too late.
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.advance_to(P1, Step::Upkeep);
    assert!(tokens_with_subtype(&t, P0, "Clue").is_empty());
    // 4 life: a Clue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndicate Heavy");
    t.g.gain_life(P0, 4);
    into_end_step(&mut t);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn restless_bloodseeker_checks_life_gained_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Restless Bloodseeker // Bloodsoaked Reveler",
        "If you haven’t gained life during the turn when your end step begins, Restless Bloodseeker’s ability (and similarly, Bloodsoaked Reveler’s ability) won’t trigger at all. Gaining life during your end step won’t cause the ability to trigger."
    );
    supported("Restless Bloodseeker // Bloodsoaked Reveler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Restless Bloodseeker // Bloodsoaked Reveler");
    into_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.advance_to(P1, Step::Upkeep);
    assert!(tokens_with_subtype(&t, P0, "Blood").is_empty());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Restless Bloodseeker // Bloodsoaked Reveler");
    t.g.gain_life(P0, 1);
    into_end_step(&mut t);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Blood").len(), 1);
}

#[test]
fn twilight_prophet_needs_the_citys_blessing_already() {
    cr!("603.4", "702.131c");
    ruling!(
        "Twilight Prophet",
        "You must already have the city's blessing in order for these abilities to trigger; otherwise they do nothing."
    );
    supported("Twilight Prophet");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twilight Prophet");
    t.library_top(P0, BEARS);
    into_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0, "no blessing: no trigger");
    // Getting the blessing now doesn't help.
    t.g.players[0].has_citys_blessing = true;
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.in_hand(P0, BEARS));
}

#[test]
fn fumulus_triggers_when_sacrificed_itself() {
    cr!("603.10a");
    ruling!(
        "Fumulus, the Infestation",
        "If you sacrifice Fumulus, its thirdsecond ability will still trigger."
    );
    supported("Fumulus, the Infestation");
    supported("Lampad of Death's Vigil");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let lampad = t.battlefield(P0, "Lampad of Death's Vigil");
    let f = t.battlefield(P0, "Fumulus, the Infestation");
    t.answer_choose(P0, &[Entity::Object(f)]);
    t.activate(P0, lampad, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fumulus, the Infestation"));
    assert_eq!(tokens_with_subtype(&t, P0, "Insect").len(), 1);
}

#[test]
fn popular_egotist_triggers_for_itself_and_each_permanent() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Popular Egotist",
        "If you sacrifice Popular Egotist, its last ability will trigger. If you sacrifice other permanents at the same time, its last ability will trigger for each of those permanents as well."
    );
    supported("Popular Egotist");
    // Sacrificed itself (to Lampad of Death's Vigil): it triggers.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let lampad = t.battlefield(P0, "Lampad of Death's Vigil");
    let e = t.battlefield(P0, "Popular Egotist");
    t.answer_choose(P0, &[Entity::Object(e)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, lampad, 0, &[]).unwrap();
    t.resolve_all();
    // Lampad: 1; Egotist: 1.
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    // Sacrificed with two other permanents at once: three triggers.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Popular Egotist");
    let a = t.battlefield(P0, BEARS);
    let b = t.battlefield(P0, "Swamp");
    t.g.sacrifice_simultaneously(&[(e, P0), (a, P0), (b, P0)]);
    t.g.flush_events();
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn popular_egotist_resolves_before_the_spell_it_was_sacrificed_for() {
    cr!("603.3");
    ruling!(
        "Popular Egotist",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Popular Egotist's last ability will resolve before that spell or ability. It will resolve even if that spell or ability is countered."
    );
    supported("Popular Egotist");
    supported("Village Rites");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Popular Egotist");
    let bears = t.battlefield(P0, BEARS);
    t.lands(P0, "Swamp", 1);
    let rites = t.hand(P0, "Village Rites");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, rites).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    assert_ne!(*t.g.stack.last().unwrap(), spell, "the trigger is on top");
    // P1 counters Village Rites; the trigger still resolves.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Village Rites"));
    assert_eq!(t.hand_size(P0), 0, "Village Rites was countered");
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn whispering_snitch_counts_surveils_before_it_entered() {
    cr!("603.2");
    ruling!(
        "Whispering Snitch",
        "If you surveil before Whispering Snitch enters the battlefield, surveilling again during that turn won't cause its ability to trigger."
    );
    supported("Whispering Snitch");
    supported("Consider");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Consider");
    let c = t.hand(P0, "Consider");
    t.cast(P0, c).go();
    t.resolve_all();
    t.battlefield(P0, "Whispering Snitch");
    give_mana_for(&mut t, P0, "Consider");
    let c = t.hand(P0, "Consider");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no trigger");
    // Its first surveil of a turn triggers it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Whispering Snitch");
    give_mana_for(&mut t, P0, "Consider");
    let c = t.hand(P0, "Consider");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// P1 casts a spell that deals `n` damage to each creature (with or without flying) and
/// each player while P0 is at `n` life and controls `name`: P0 loses the game before the
/// creature's dies/leaves trigger goes on the stack.
fn dies_with_its_controller(name: &str, spell: &str, n: i64) {
    supported(name);
    supported(spell);
    let mut t = TestGame::new(3);
    t.battlefield(P0, name);
    t.g.players[0].life = n as i32;
    t.set_step(P1, Step::PrecombatMain);
    // {X}{G} or {X}{R}: one land for the color, one Wastes per X.
    give_mana_for(&mut t, P1, spell);
    t.lands(P1, "Wastes", n as usize - 1);
    let s = t.hand(P1, spell);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P1, s).x(n).go();
    t.resolve_all();
    assert!(t.has_lost(P0), "{name}");
    assert!(t.named_on_battlefield(name).is_empty(), "{name}: gone");
    let expect = 20 - n as i32;
    assert_eq!(t.life(P1), expect, "{name}: P1 lost no life to the trigger");
    assert_eq!(t.life(P2), expect, "{name}: P2 lost no life to the trigger");
}

#[test]
fn losing_the_game_at_the_same_time_as_the_creature_dies() {
    cr!("104.3b", "704.3", "800.4a");
    ruling!(
        "Nocturnal Feeder",
        "If your life total is brought to 0 or less at the same time that Nocturnal Feeder is dealt lethal damage, you lose the game before its ability goes on the stack."
    );
    ruling!(
        "Spirit of Malevolence",
        "If your life total is brought to 0 or less at the same time that Spirit of Malevolence is dealt lethal damage, you lose the game before its ability goes on the stack."
    );
    ruling!(
        "Ukkima, Stalking Shadow",
        "If your life total is brought to 0 or less at the same time that Ukkima is dealt lethal damage, you lose the game before the last ability goes on the stack."
    );
    // Nocturnal Feeder flies: Hurricane. The others don't: Earthquake.
    dies_with_its_controller("Nocturnal Feeder", "Hurricane", 1);
    dies_with_its_controller("Spirit of Malevolence", "Earthquake", 1);
    dies_with_its_controller("Ukkima, Stalking Shadow", "Earthquake", 2);
}

#[test]
fn kambal_resolves_before_the_spell() {
    cr!("603.3");
    ruling!(
        "Kambal, Consul of Allocation",
        "Kambal's ability resolves before the spell that caused it to trigger."
    );
    supported("Kambal, Consul of Allocation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kambal, Consul of Allocation");
    give_mana_for(&mut t, P1, "Opt");
    let d = t.hand(P1, "Opt");
    t.cast(P1, d).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    assert_eq!(t.stack_len(), 1, "Opt is still on the stack");
    assert_eq!(t.hand_size(P1), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn faerie_bladecrafter_triggers_once_per_player_per_damage_step() {
    cr!("603.2c", "510.2");
    ruling!(
        "Faerie Bladecrafter",
        "Faerie Bladecrafter's first triggered ability triggers once for each player that Faeries you control dealt combat damage to, regardless of how many creatures were dealing that damage."
    );
    supported("Faerie Bladecrafter");
    let mut t = TestGame::new(3);
    let b = t.battlefield(P0, "Faerie Bladecrafter");
    let f1 = t.battlefield(P0, "Faerie Miscreant");
    let f2 = t.battlefield(P0, "Faerie Miscreant");
    // Two Faeries hit P1 and one hits P2: two triggers.
    t.attack(
        &[
            (b, Entity::Player(P1)),
            (f1, Entity::Player(P1)),
            (f2, Entity::Player(P2)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 2);
    // With double strike, each combat damage step triggers it.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Faerie Bladecrafter");
    let f1 = t.battlefield(P0, "Faerie Miscreant");
    crate::r_s05_common::run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![f1])),
            mods: vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
                mtg_engine::keywords::KeywordKind::DoubleStrike,
            ))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    t.attack(&[(b, Entity::Player(P1)), (f1, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 2);
}

#[test]
fn rin_and_seri_dont_see_themselves_being_cast() {
    cr!("113.6", "603.2");
    ruling!(
        "Rin and Seri, Inseparable",
        "Rin and Seri's first two abilities don't trigger when you cast Rin and Seri."
    );
    supported("Rin and Seri, Inseparable");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Rin and Seri, Inseparable");
    let rs = t.hand(P0, "Rin and Seri, Inseparable");
    t.cast(P0, rs).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no triggers");
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Cat").is_empty());
    assert!(tokens_with_subtype(&t, P0, "Dog").is_empty());
    // With Rin and Seri on the battlefield, casting a Cat spell creates a Dog.
    give_mana_for(&mut t, P0, "Cauldron Familiar");
    let cat = t.hand(P0, "Cauldron Familiar");
    t.cast(P0, cat).go();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Dog").len(), 1);
}

#[test]
fn maccready_checks_power_only_as_it_triggers() {
    cr!("603.2", "603.4");
    ruling!(
        "MacCready, Lamplight Mayor",
        "Similarly, the power of the creature attacking you is checked only when MacCready’s second ability triggers."
    );
    supported("MacCready, Lamplight Mayor");
    // A 4/4 attacks P0; it's destroyed before the ability resolves: still 2 life each way.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "MacCready, Lamplight Mayor");
    let big = t.battlefield(P1, "Rumbling Baloth");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(big, Entity::Player(P0))]);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 1);
    destroy(&mut t, big);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // A 2/2 that becomes 4/4 after attacking: it didn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "MacCready, Lamplight Mayor");
    let bears = t.battlefield(P1, BEARS);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    crate::r_p057_common::pump(&mut t, bears, 2, 2);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 0);
}
