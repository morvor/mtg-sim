//! Rulings batch P163 — spell-cast triggers and mana (Urabrask // The Great Work, Ashling,
//! Flame Dancer): cast triggers aren't mana abilities (CR 605.1a, 605.5a), spells cast
//! vs copies (CR 707.10, 707.12, 702.40c), permission to cast from a graveyard follows
//! the normal timing (CR 601.3, 307.1, 117.3b), emptying mana pools (CR 500.5, 106.4);
//! votes (CR 701.38); intervening "if" clauses and enters triggers (CR 603.4, 603.6d,
//! 611.2c, 613.4); cost reductions (CR 601.2f, 602.2b, 118.7); activation triggers
//! (CR 602.2).

use crate::r_p163_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn red(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.count(ManaType::R)
}

/// P0 casts Shock at P1 (with a Mountain for it) and the stack is settled.
fn shock(t: &mut TestGame) -> ObjectId {
    let s = cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    s
}

#[test]
fn urabrask_trigger_uses_the_stack_and_adds_mana_on_resolution() {
    cr!("605.1a", "605.5a", "603.3");
    ruling!(
        "Urabrask // The Great Work",
        "Urabrask’s second ability isn’t a mana ability, even though it causes you to add mana. It uses the stack and can be responded to."
    );
    supported("Urabrask // The Great Work");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urabrask // The Great Work");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    shock(&mut t);
    assert_eq!(stacked_triggers(&t), 1);
    assert_eq!(red(&t, P0), 0);
    t.resolve();
    assert_eq!(red(&t, P0), 1);
    assert_eq!(t.life(P1), 19);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

/// Urabrask on the battlefield for P0 with P1 as the target of its triggers.
fn urabrask(t: &mut TestGame) -> ObjectId {
    let u = t.battlefield(P0, "Urabrask // The Great Work");
    for _ in 0..6 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    u
}

#[test]
fn urabrask_counts_countered_spells_but_not_copies() {
    cr!("707.10", "707.12");
    ruling!(
        "Urabrask // The Great Work",
        "Urabrask will count any instant or sorcery spells you’ve cast during the turn, whether those spells resolved, didn’t resolve, were countered, or left the stack some other way. Urabrask won’t count copies of instant and sorcery spells that were created on the stack and not cast."
    );
    supported("Dualcaster Mage");
    // Three Shocks, one countered: Urabrask can transform.
    let mut t = TestGame::new(2);
    let u = urabrask(&mut t);
    for i in 0..3 {
        let s = shock(&mut t);
        if i == 1 {
            counter(&mut t, s);
        }
        t.resolve_all();
    }
    t.lands(P0, "Mountain", 1);
    assert!(can_activate(&mut t, P0, u));
    // Two Shocks and a copy made by Dualcaster Mage: it can't.
    let mut t = TestGame::new(2);
    let u = urabrask(&mut t);
    shock(&mut t);
    t.resolve_all();
    let s = shock(&mut t);
    t.resolve();
    // Dualcaster Mage (flash) copies the second Shock.
    t.answer_targets(P0, &[obj(s)]);
    cast_new(&mut t, P0, "Dualcaster Mage", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 2 - 2 - 2);
    t.lands(P0, "Mountain", 1);
    assert!(!can_activate(&mut t, P0, u));
}

/// Transforms Urabrask into The Great Work (three Shocks and its {R} ability) and adds
/// lore counters until chapter III has resolved. Returns the Saga.
fn great_work_chapter_three(t: &mut TestGame) -> ObjectId {
    let u = urabrask(t);
    for _ in 0..3 {
        shock(t);
        t.resolve_all();
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_resolve(t, P0, u, 0, &[]);
    let saga = t.g.current(u);
    assert!(t.obj(saga).chars.subtypes.iter().any(|s| s == "Saga"));
    put_counters(t, saga, counters::LORE, 1);
    t.resolve_all();
    put_counters(t, saga, counters::LORE, 1);
    assert_eq!(stacked_triggers(t), 1);
    saga
}

#[test]
fn the_great_work_graveyard_casting_follows_normal_timing() {
    cr!("601.3", "307.1", "117.3b", "117.1a");
    ruling!(
        "Urabrask // The Great Work",
        "After chapter III of The Great Work resolves, casting spells from graveyards follows the normal rules for casting those cards. You must pay their costs, and you must follow all applicable timing rules. For example, to cast a sorcery spell from a graveyard, you can do so by paying its mana cost only during your main phase while the stack is empty."
    );
    ruling!(
        "Urabrask // The Great Work",
        "If multiple players are allowed to cast the same card, the player with priority at any given time determines who can cast it. The player whose turn it is has priority as each step and phase begins, and they receive priority after each spell and ability resolves. This means immediately after chapter III resolves you’ll have priority to cast spells before any other player does."
    );
    let mut t = TestGame::new(2);
    let sorcery = t.graveyard(P1, "Divination");
    let instant = t.graveyard(P1, "Lightning Bolt");
    great_work_chapter_three(&mut t);
    // Both players pass: chapter III resolves and P0 gets priority first.
    let ok = t.g.run_until(1000, |g| g.stack.is_empty());
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    // No mana (chapter II's Treasures and the lands gone): neither can be cast.
    t.g.players[0].mana_pool = Default::default();
    for c in t.g.battlefield.clone() {
        if t.g.obj(c).controller == P0 && !t.g.obj(c).chars.subtypes.iter().any(|s| s == "Saga") {
            move_to(&mut t, c, mtg_engine::object::Zone::Exile);
        }
    }
    assert!(!can_cast(&mut t, P0, instant, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, sorcery, CastMethod::Normal));
    // With mana, both in the main phase with an empty stack.
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, sorcery, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, instant, CastMethod::Normal));
    // With a spell on the stack, only the instant.
    t.lands(P0, "Mountain", 1);
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(t.g.stack.contains(&shock));
    assert!(!can_cast(&mut t, P0, sorcery, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, instant, CastMethod::Normal));
    t.resolve_all();
    // In combat, only the instant.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, sorcery, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, instant, CastMethod::Normal));
}

/// Ashling, Flame Dancer on the battlefield for P0, with a card in hand for its rummage.
fn ashling(t: &mut TestGame) -> ObjectId {
    supported("Ashling, Flame Dancer");
    t.battlefield(P0, "Ashling, Flame Dancer")
}

#[test]
fn ashling_keeps_red_mana_across_steps_and_turns() {
    cr!("500.5", "106.4", "611.3a");
    ruling!(
        "Ashling, Flame Dancer",
        "You can keep unspent red mana indefinitely while Ashling is on the battlefield. That means if you add a red mana during one step or phase, you can spend it during a later step or phase, or even a later turn. You will still lose other types of unspent mana as each step and phase ends."
    );
    ruling!(
        "Ashling, Flame Dancer",
        "Once Ashling leaves the battlefield, you have until the end of the current step or phase to spend whatever red mana you have before you lose that mana as normal. There is no penalty associated with this other than the loss of the mana."
    );
    let mut t = TestGame::new(2);
    let a = ashling(&mut t);
    crate::r_p160_common::mana(&mut t, P0, ManaType::R, 2);
    crate::r_p160_common::mana(&mut t, P0, ManaType::G, 1);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(red(&t, P0), 2);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(red(&t, P0), 2);
    // Ashling leaves: the mana stays until the step ends, then it's lost.
    move_to(&mut t, a, mtg_engine::object::Zone::Hand(P0));
    assert_eq!(red(&t, P0), 2);
    t.advance_to(P1, Step::Draw);
    assert_eq!(red(&t, P0), 0);
}

#[test]
fn ashling_triggers_for_each_copy_an_effect_creates() {
    cr!("702.40a", "707.10", "707.10c");
    ruling!(
        "Ashling, Flame Dancer",
        "If an effect creates multiple copies of an instant or sorcery spell, Ashling's last ability triggers once for each copy created by the effect."
    );
    supported("Grapeshot");
    // Two spells, then Grapeshot: its storm trigger makes two copies at once.
    let mut t = TestGame::new(2);
    ashling(&mut t);
    crate::r_p146_common::fill_hand(&mut t, P0, 6);
    for _ in 0..2 {
        cast_new(&mut t, P0, "Opt", &[]);
        t.resolve_all();
    }
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    cast_new(&mut t, P0, "Grapeshot", &[]);
    t.settle();
    // The cast trigger and the storm trigger.
    assert_eq!(stacked_triggers(&t), 2);
    // Resolve until the storm trigger has resolved.
    let before = triggers_named(&t, "discard a card");
    while triggers_named(&t, "discard a card") == before {
        t.g.resolve_top();
        t.g.flush_events();
        t.settle();
        assert!(!t.g.stack.is_empty());
    }
    assert_eq!(triggers_named(&t, "discard a card"), before + 2);
}

/// The Magecraft triggers on the stack.
fn triggers_named(t: &TestGame, text: &str) -> usize {
    crate::r_s01_common::triggers_on_stack(t, text)
}

#[test]
fn ashling_copying_a_card_outside_the_stack_doesnt_trigger_but_casting_it_does() {
    cr!("707.12", "702.40a", "601.2i");
    ruling!(
        "Ashling, Flame Dancer",
        "Some effects instruct you to copy an instant or sorcery card in a zone other than the stack. These copies will not cause Ashling's last ability to trigger. However, most effects that do this also allow you to cast the copy, and casting the copy will cause Ashling's last ability to trigger."
    );
    supported("Isochron Scepter");
    for cast_it in [true, false] {
        let mut t = TestGame::new(2);
        ashling(&mut t);
        crate::r_p146_common::fill_hand(&mut t, P0, 2);
        let bolt = t.hand(P0, "Lightning Bolt");
        yes(&mut t, P0);
        t.answer_choose(P0, &[obj(bolt)]);
        let sc = enter(&mut t, P0, "Isochron Scepter");
        t.resolve_all();
        assert!(t.in_exile("Lightning Bolt"));
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, cast_it);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.activate(P0, sc, 0, &[]).unwrap();
        t.resolve();
        t.settle();
        assert_eq!(triggers_named(&t, "discard a card"), usize::from(cast_it));
        t.resolve_all();
        assert_eq!(t.life(P1), if cast_it { 17 } else { 20 });
    }
}

#[test]
fn mob_verdict_votes_on_resolution_and_vote_triggers_wait() {
    cr!("701.38a", "603.3");
    ruling!(
        "Mob Verdict",
        "Votes are cast during the resolution of Mob Verdict, so any responses to Mob Verdict must be made without knowing the outcome of the vote."
    );
    ruling!(
        "Mob Verdict",
        "Abilities that trigger “whenever players finish voting” trigger once all players have voted or once all secret votes are revealed, but they won’t go on the stack until the current spell or ability finishes resolving."
    );
    supported("Mob Verdict");
    supported("Grudge Keeper");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grudge Keeper");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseOption { .. } | Decision::ChooseEntities { .. }),
        |g| g.stack.len(),
    );
    let spell = cast_new(&mut t, P0, "Mob Verdict", &[]);
    // Nothing is voted while casting.
    assert!(seen.lock().unwrap().is_empty());
    t.g.resolve_top();
    t.g.flush_events();
    // P1 voted while Mob Verdict was on the stack, resolving.
    assert!(!seen.lock().unwrap().is_empty());
    assert!(seen.lock().unwrap().iter().all(|n| *n == 1));
    assert!(resolved(&t, spell));
    // Grudge Keeper's trigger isn't on the stack yet: Mob Verdict just finished.
    assert!(t.g.stack.is_empty());
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
}

#[test]
fn bite_of_the_black_rose_affects_all_opponents_the_same() {
    cr!("701.38a");
    ruling!(
        "Bite of the Black Rose",
        "The effect will be the same for all your opponents, no matter how they voted."
    );
    supported("Bite of the Black Rose");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Hill Giant");
    pick_options(&mut t, P0, &["sickness"]);
    pick_options(&mut t, P1, &["sickness"]);
    pick_options(&mut t, P2, &["psychosis"]);
    let hands = (t.hand_size(P1), t.hand_size(P2));
    cast_resolve(&mut t, P0, "Bite of the Black Rose", &[]);
    assert_eq!(t.pt(a), (1, 1));
    assert_eq!(t.pt(b), (1, 1));
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), hands);
}

/// Sigil Captain on the battlefield for P0.
fn sigil_captain(t: &mut TestGame) -> ObjectId {
    supported("Sigil Captain");
    t.battlefield(P0, "Sigil Captain")
}

#[test]
fn two_sigil_captains_dont_stack() {
    cr!("603.4", "603.2");
    ruling!(
        "Sigil Captain",
        "For that reason, two Sigil Captains don't work well together. When a 1/1 creature enters under your control, the abilities of both Sigil Captains will trigger. When the first resolves, two +1/+1 counters are put on the 1/1 creature. When the second resolves, it will do nothing because the creature that entered is now 3/3."
    );
    let mut t = TestGame::new(2);
    sigil_captain(&mut t);
    sigil_captain(&mut t);
    let elves = enter(&mut t, P0, "Llanowar Elves");
    assert_eq!(stacked_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(t.counters(elves, counters::PLUS1), 2);
    assert_eq!(t.pt(elves), (3, 3));
}

#[test]
fn sigil_captain_entering_as_a_1_1_triggers_itself() {
    cr!("603.6a", "603.4");
    ruling!(
        "Sigil Captain",
        "If, for some reason, Sigil Captain enters as a 1/1 creature (due to a pair of Engineered Plagues, for example), it will cause its own ability to trigger."
    );
    supported("Engineered Plague");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        pick_options(&mut t, P1, &["Rhino"]);
        enter(&mut t, P1, "Engineered Plague");
    }
    let c = enter(&mut t, P0, "Sigil Captain");
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn sigil_captain_checks_the_power_and_toughness_it_enters_with() {
    cr!("603.4", "603.6d", "613.4c");
    ruling!(
        "Sigil Captain",
        "Sigil Captain's ability checks a creature's initial power and toughness upon being put on the battlefield, so it will take into account counters that it enters with and static abilities that may give it a continuous power boost once it's on the battlefield (such as the one on Glorious Anthem). After the creature is already on the battlefield, changing its power and toughness to 1/1 with a spell, activated ability, or triggered ability won't allow this ability to trigger; it's too late by then."
    );
    ruling!(
        "Sigil Captain",
        "Sigil Captain's ability has an \"intervening 'if' clause.\" That means (1) the ability won't trigger at all unless the creature that enters under your control is 1/1, and (2) the ability will do nothing unless that creature is still 1/1 at the time the ability resolves."
    );
    // Walking Ballista for X = 1 enters with a counter as a 1/1: it triggers.
    let mut t = TestGame::new(2);
    sigil_captain(&mut t);
    t.lands(P0, "Wastes", 2);
    let wb = t.hand(P0, "Walking Ballista");
    t.cast(P0, wb).x(1).go();
    t.resolve();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(wb, counters::PLUS1), 3);
    // A 1/1 with Glorious Anthem out enters as a 2/2: no trigger.
    let mut t = TestGame::new(2);
    sigil_captain(&mut t);
    t.battlefield(P0, "Glorious Anthem");
    enter(&mut t, P0, "Llanowar Elves");
    assert_eq!(stacked_triggers(&t), 0);
    // A 2/2 shrunk to 1/1 after it entered: no trigger.
    let mut t = TestGame::new(2);
    sigil_captain(&mut t);
    let bears = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(stacked_triggers(&t), 0);
    crate::r_p057_common::pump(&mut t, bears, -1, -1);
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(stacked_triggers(&t), 0);
    // A 1/1 pumped in response: the trigger does nothing.
    let mut t = TestGame::new(2);
    sigil_captain(&mut t);
    let elves = enter(&mut t, P0, "Llanowar Elves");
    assert_eq!(stacked_triggers(&t), 1);
    crate::r_p057_common::pump(&mut t, elves, 1, 0);
    t.resolve_all();
    assert_eq!(t.counters(elves, counters::PLUS1), 0);
}

#[test]
fn pendelhaven_elder_affects_actual_1_1s_at_resolution() {
    cr!("611.2c", "613.4c", "208.3");
    ruling!(
        "Pendelhaven Elder",
        "Pendelhaven Elder's ability affects only 1/1 creatures you control at the time it resolves. Creatures you begin to control later in the turn or that become 1/1 later in the turn won't get +1/+2."
    );
    ruling!(
        "Pendelhaven Elder",
        "Pendelhaven Elder's ability looks at each creature's actual power and toughness taking into account counters and effects that raise or lower its power and/or toughness, not just its base power and toughness."
    );
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Pendelhaven Elder");
    // A 0/0 Walking Ballista with a counter is a 1/1; a pumped Lions is a 2/1.
    let wb = t.battlefield(P0, "Walking Ballista");
    put_counters(&mut t, wb, counters::PLUS1, 1);
    let pumped = t.battlefield(P0, "Llanowar Elves");
    crate::r_p057_common::pump(&mut t, pumped, 1, 0);
    let theirs = t.battlefield(P1, "Llanowar Elves");
    activate_resolve(&mut t, P0, elder, 0, &[]);
    assert_eq!(t.pt(elder), (2, 3));
    assert_eq!(t.pt(wb), (2, 3));
    assert_eq!(t.pt(pumped), (2, 1));
    assert_eq!(t.pt(theirs), (1, 1));
    // Later: a 1/1 entering and one taken from P1 aren't affected; a Bears shrunk to 1/1
    // isn't either.
    let later = t.enter(P0, "Llanowar Elves");
    give_control(&mut t, theirs, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p057_common::pump(&mut t, bears, -1, -1);
    assert_eq!(t.pt(later), (1, 1));
    assert_eq!(t.pt(theirs), (1, 1));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn biomancers_familiar_reduces_activated_abilities_to_at_least_one_mana() {
    cr!("601.2f", "602.2b", "118.7a", "702.87a");
    ruling!(
        "Biomancer's Familiar",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities are activated abilities and will have colons in their reminder text. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by the cost reduction ability of Biomancer's Familiar."
    );
    ruling!(
        "Biomancer's Familiar",
        "An ability that has only one generic mana and some amount of other mana symbols in its activation cost will have its cost reduced by {1}."
    );
    supported("Biomancer's Familiar");
    // Level up {1}{R} (a keyword activated ability) costs {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biomancer's Familiar");
    let lord = t.battlefield(P0, "Lord of Shatterskull Pass");
    t.lands(P0, "Mountain", 1);
    assert!(can_activate(&mut t, P0, lord));
    activate_resolve(&mut t, P0, lord, 0, &[]);
    assert_eq!(t.counters(lord, counters::LEVEL), 1);
    // Dragon Engine's {2} is reduced to {1}, not below one mana.
    supported("Dragon Engine");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biomancer's Familiar");
    let d = t.battlefield(P0, "Dragon Engine");
    assert!(!can_activate(&mut t, P0, d));
    t.lands(P0, "Wastes", 1);
    assert!(can_activate(&mut t, P0, d));
    activate_resolve(&mut t, P0, d, 0, &[]);
    assert_eq!(t.pt(d), (2, 3));
    // Kavaron Harrier's attack trigger ("you may pay {2}") isn't reduced: with one land
    // it can't be paid; with two it can.
    supported("Kavaron Harrier");
    for lands in [1, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Biomancer's Familiar");
        let h = t.battlefield(P0, "Kavaron Harrier");
        t.lands(P0, "Wastes", lands);
        attack_with(&mut t, &[(h, Entity::Player(P1))]);
        yes(&mut t, P0);
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), lands - 1, "{lands}");
    }
}

#[test]
fn crackdown_construct_triggers_for_each_zero_cost_activation() {
    cr!("602.2", "603.2", "605.1a");
    ruling!(
        "Crackdown Construct",
        "An ability whose cost is simply {0} can be activated as many times as you'd like. Each activation causes Crackdown Construct's ability to trigger."
    );
    supported("Crackdown Construct");
    supported("Chimeric Idol");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crackdown Construct");
    let idol = t.battlefield(P0, "Chimeric Idol");
    for _ in 0..3 {
        t.activate(P0, idol, 0, &[]).unwrap();
    }
    t.settle();
    assert_eq!(stacked_triggers(&t), 3);
    t.resolve_all();
    assert_eq!(t.pt(c), (5, 5));
}

/// The votes revealed (reported) so far this turn, including the resolving spell's.
fn votes_revealed(g: &mtg_engine::game::Game) -> usize {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter(|e| {
            matches!(e, mtg_engine::events::Event::Custom { name, .. }
                if name == mtg_engine::kwa::vote::VOTED)
        })
        .count()
}

#[test]
fn mob_verdict_votes_stay_secret_until_revealed_together() {
    cr!("701.38a");
    ruling!(
        "Mob Verdict",
        "Before secret votes are revealed, players may announce how they intend to vote, but they can’t reveal what they actually wrote down until all votes are simultaneously revealed."
    );
    fn is_vote(d: &Decision) -> bool {
        matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Vote")
    }
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| crate::r_s01_common::watch(&mut t, p, is_vote, votes_revealed))
        .collect();
    // P0 and P2 vote for P1; P1 votes for P0.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    t.answer_choose(P2, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Mob Verdict", &[]);
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![0]);
    }
    assert_eq!(votes_revealed(&t.g), 3);
    // Two votes for P1: 4 damage to P1 and the Bears; one vote for P0: a card.
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(bears));
    assert_eq!(marked(&t, giant), 0);
    assert_eq!(t.life(P2), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
}
