//! Item `rulings-residue-counters` — rulings that restate what the card's own text says
//! (no interaction with other rules or cards beyond the minimal setup): one table, each
//! case with its card, a minimal setup and the asserted outcome, citing its rule and its
//! ruling.

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// A restatement ruling: its card and the check (setup, outcome, citations).
type Case = (&'static str, fn());

#[test]
fn restatement_rulings() {
    let cases: &[Case] = &[
        // tracker targeting itself deals damage to itself twice
        ("Tracker", || {
            cr!("120.3e", "608.2c");
            ruling!(
                "Tracker",
                "Tracker will deal damage to itself equal to its power, then immediately do it again"
            );
            supported("Tracker");
            supported("Giant Growth");
            let mut t = TestGame::new(2);
            let tracker = t.battlefield(P0, "Tracker");
            // Giant Growth in response makes it 5/5: it deals 5 damage to itself, then 5 again.
            t.lands(P0, "Forest", 2);
            t.activate(P0, tracker, 0, &[obj(tracker)]).unwrap();
            giant_growth(&mut t, tracker);
            t.resolve_all();
            assert_eq!(dealt_to(&t, obj(tracker)), vec![5, 5]);
            assert!(t.in_graveyard(P0, "Tracker"));
        }),
        // dismantle counts counters of every kind
        ("Dismantle", || {
            cr!("122.1", "608.2h");
            ruling!(
                "Dismantle",
                "It doesn't matter what kind of counters the destroyed artifact had on it, only how many."
            );
            supported("Dismantle");
            let mut t = TestGame::new(2);
            let target = t.battlefield(P1, "Ornithopter");
            put(&mut t, target, "fuse", 5);
            put(&mut t, target, "trap", 2);
            let stone = dismantle(&mut t, target, |_| {});
            assert!(t.in_graveyard(P1, "Ornithopter"));
            assert_eq!(dismantled(&t, stone), 7);
        }),
        // descent into madness with too many counters exiles everything
        ("Descent into Madness", || {
            cr!("609.3", "101.4");
            ruling!(
                "Descent into Madness",
                "you’ll exile all permanents you control (including Descent into Madness) and all cards from your hand"
            );
            supported("Descent into Madness");
            let mut t = TestGame::new(2);
            let d = t.battlefield(P0, "Descent into Madness");
            put(&mut t, d, "despair", 3);
            t.battlefield(P0, "Forest");
            t.hand(P0, "Grizzly Bears");
            t.advance_to(P0, Step::Upkeep);
            t.settle();
            // P0 draws nothing yet (upkeep); four counters against two permanents and one card.
            let hand_before = t.hand_size(P0);
            t.resolve_all();
            assert!(t.g.permanents().all(|o| o.controller != P0));
            assert_eq!(t.hand_size(P0), 0, "had {hand_before}");
            assert!(t.in_exile("Descent into Madness"));
        }),
        // nissa card she doesnt put onto the battlefield stays on top
        ("Nissa, Steward of Elements", || {
            cr!("608.2c", "609.3");
            ruling!(
                "Nissa, Steward of Elements",
                "While resolving Nissa's second ability, if you can't or don't put the card onto the battlefield, it remains on top of your library."
            );
            supported("Nissa, Steward of Elements");
            // Can't: Hill Giant's mana value (4) is above her loyalty (3).
            let mut t = TestGame::new(2);
            let (_, giant, _) = nissa_zero(&mut t, 3, "Hill Giant");
            t.resolve_all();
            assert_eq!(t.zone(giant), Zone::Library(P0));
            assert_eq!(*t.g.player(P0).library.last().unwrap(), giant);
            // Doesn't: a Forest she may put onto the battlefield, declined.
            let mut t = TestGame::new(2);
            let (_, forest, _) = nissa_zero(&mut t, 3, "Forest");
            t.answer_yes(P0, false);
            t.resolve_all();
            assert_eq!(*t.g.player(P0).library.last().unwrap(), forest);
        }),
        // nissa zero ability gives no one priority while it resolves
        ("Nissa, Steward of Elements", || {
            cr!("608.2", "117.3");
            ruling!(
                "Nissa, Steward of Elements",
                "Once Nissa's second ability begins to resolve, no player may take actions until it's done."
            );
            let mut t = TestGame::new(2);
            let (n, bears, _) = nissa_zero(&mut t, 3, "Grizzly Bears");
            let from = t.asked().len();
            t.answer_yes(P0, true);
            t.resolve();
            // While it resolves only P0's choice is asked: no one gets priority between the
            // look and the card entering.
            let asked = asked_since(&t, from);
            assert_eq!(asked.len(), 1, "{asked:?}");
            assert!(matches!(asked[0], (P0, Decision::YesNo { .. })));
            assert!(t.on_battlefield(t.g.current(bears)));
            assert_eq!(t.counters(n, "loyalty"), 3);
        }),
        // afiya grove must target even an opponents creature
        ("Afiya Grove", || {
            cr!("603.3d", "115.1");
            ruling!(
                "Afiya Grove",
                "If there are any creatures on the battlefield, even if they are just your opponent’s, you must put a +1/+1 on one of them."
            );
            supported("Afiya Grove");
            // Only P1's Bears: the counter moves onto them.
            let mut t = TestGame::new(2);
            let grove = with_counters(&mut t, P0, "Afiya Grove", 3);
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.advance_to(P0, Step::Upkeep);
            t.resolve_all();
            assert_eq!((plus1(&t, grove), plus1(&t, bears)), (2, 1));
            // No creatures: no target, no counter removed.
            let mut t = TestGame::new(2);
            let grove = with_counters(&mut t, P0, "Afiya Grove", 3);
            t.advance_to(P0, Step::Upkeep);
            t.resolve_all();
            assert_eq!(plus1(&t, grove), 3);
        }),
        // xolatoyac flooded land keeps its types and abilities
        ("Xolatoyac, the Smiling Flood", || {
            cr!("205.1b", "305.6");
            ruling!(
                "Xolatoyac, the Smiling Flood",
                "The land retains any land types and abilities it already had. An Island has the ability \"{T}: Add {U}.\""
            );
            supported("Xolatoyac, the Smiling Flood");
            let mut t = TestGame::new(2);
            let (_, forest) = flood(&mut t);
            let o = t.obj_now(forest);
            assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Island"));
            let texts = ability_texts(&mut t, forest);
            assert!(texts.iter().any(|s| s.contains("Add {G}")), "{texts:?}");
            assert!(texts.iter().any(|s| s.contains("Add {U}")), "{texts:?}");
        }),
        // daxos spirit tracks your experience counters
        ("Daxos the Returned", || {
            cr!("604.3", "122.1");
            ruling!(
                "Daxos the Returned",
                "The power and toughness of the tokens will change as the number of experience counters you have does."
            );
            supported("Daxos the Returned");
            let mut t = TestGame::new(2);
            put(&mut t, Entity::Player(P0), "experience", 2);
            let spirit = daxos_spirit(&mut t);
            assert_eq!(t.pt(spirit), (2, 2));
            put(&mut t, Entity::Player(P0), "experience", 1);
            assert_eq!(t.pt(spirit), (3, 3));
        }),
        // rumbling ruin with no counters stops creatures with no power
        ("Rumbling Ruin", || {
            cr!("509.1b", "608.2h");
            ruling!(
                "Rumbling Ruin",
                "If there are no +1/+1 counters on creatures you control as Rumbling Ruin’s ability resolves, creatures with power 0 or less can’t block this turn."
            );
            supported("Rumbling Ruin");
            // The 0/8 wall's block is refused; a 2/2's block is allowed.
            for (name, life) in [("Wall of Stone", 18), ("Grizzly Bears", 20)] {
                let mut t = TestGame::new(2);
                t.set_step(P0, Step::PrecombatMain);
                t.enter(P0, "Rumbling Ruin");
                t.resolve_all();
                let attacker = t.battlefield(P0, "Grizzly Bears");
                let blocker = t.battlefield(P1, name);
                attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
                t.answer(
                    P1,
                    DecisionKind::Blockers,
                    Answer::Blockers(vec![(blocker, attacker)]),
                );
                t.advance_to(P0, Step::EndOfCombat);
                assert_eq!(t.life(P1), life, "{name}");
            }
        }),
        // kate stewart counts time counters on permanents only
        ("Kate Stewart", || {
            cr!("122.1", "608.2h");
            ruling!(
                "Kate Stewart",
                "Similarly, its last ability counts only time counters on permanents, not on cards you own in exile."
            );
            supported("Kate Stewart");
            let mut t = TestGame::new(2);
            let kate = t.battlefield(P0, "Kate Stewart");
            let rock = t.battlefield(P0, "Mind Stone");
            put(&mut t, rock, "time", 1);
            t.resolve_all();
            let exiled = t.exile(P0, "Rift Bolt");
            put(&mut t, exiled, "time", 3);
            t.resolve_all();
            t.lands(P0, "Wastes", 8);
            t.answer_yes(P0, true);
            attack_with(&mut t, &[(kate, Entity::Player(P1))]);
            t.resolve_all();
            assert_eq!(t.pt(kate), (4, 4));
        }),
        // armageddon clock counters can be removed repeatedly by any player
        ("Armageddon Clock", || {
            // Its activation instructions let any player activate it during an upkeep, with
            // no limit on how many times (CR 602.1b, 602.2, 117.1b).
            cr!("602.1b", "602.2", "117.1b");
            ruling!(
                "Armageddon Clock",
                "Players can remove as many counters as they can pay for each turn."
            );
            supported("Armageddon Clock");
            let mut t = TestGame::new(2);
            let clock = t.battlefield(P0, "Armageddon Clock");
            put(&mut t, clock, "doom", 3);
            t.set_step(P0, Step::Upkeep);
            t.lands(P1, "Wastes", 8);
            t.activate(P1, clock, 0, &[]).unwrap();
            t.resolve();
            t.activate(P1, clock, 0, &[]).unwrap();
            t.resolve();
            assert_eq!(t.counters(clock, "doom"), 1);
        }),
        // combine guildmage gives a counter to a creature entering without any
        ("Combine Guildmage", || {
            cr!("614.1c", "122.6");
            ruling!(
                "Combine Guildmage",
                "If a creature would normally enter the battlefield with no +1/+1 counters on it, Combine Guildmage’s first ability causes it to enter the battlefield with a +1/+1 counter on it."
            );
            supported("Combine Guildmage");
            let mut t = TestGame::new(2);
            let gm = t.battlefield(P0, "Combine Guildmage");
            t.lands(P0, "Forest", 2);
            t.activate(P0, gm, 0, &[]).unwrap();
            t.resolve_all();
            let bears = t.enter(P0, "Grizzly Bears");
            t.settle();
            assert_eq!(plus1(&t, bears), 1);
            assert_eq!(t.pt(bears), (3, 3));
        }),
        // combine guildmage moves nothing from a creature without counters
        ("Combine Guildmage", || {
            cr!("122.5", "609.3");
            ruling!(
                "Combine Guildmage",
                "If the first target creature for Combine Guildmage’s second ability has no +1/+1 counters on it as the ability resolves, nothing happens."
            );
            let mut t = TestGame::new(2);
            let a = with_counters(&mut t, P0, "Grizzly Bears", 1);
            let b = t.battlefield(P0, "Hill Giant");
            guildmage_move(&mut t, a, b, |t| remove_plus1(t, a, 1));
            assert_eq!((plus1(&t, a), plus1(&t, b)), (0, 0));
        }),
        // emperor of bones finality counter exiles instead of the graveyard
        ("Emperor of Bones", || {
            cr!("122.1h", "614.6");
            ruling!(
                "Emperor of Bones",
                "If any permanent with a finality counter on it would go to a graveyard from the battlefield, exile it instead."
            );
            supported("Emperor of Bones");
            let mut t = TestGame::new(2);
            let emperor = t.battlefield(P0, "Emperor of Bones");
            let bears = t.graveyard(P1, "Grizzly Bears");
            // The Emperor exiles P1's Bears at the beginning of combat, then reanimates it with a
            // finality counter when it adapts.
            t.answer_targets(P0, &[obj(bears)]);
            t.advance_to(P0, Step::BeginningOfCombat);
            t.resolve_all();
            assert!(t.in_exile("Grizzly Bears"));
            t.lands(P0, "Swamp", 2);
            t.answer_choose(P0, &[obj(t.g.current(bears))]);
            t.activate(P0, emperor, 0, &[]).unwrap();
            t.resolve_all();
            let back = t.named_on_battlefield("Grizzly Bears");
            assert_eq!(back.len(), 1);
            assert_eq!(t.counters(back[0], "finality"), 1);
            destroy(&mut t, back[0]);
            assert!(t.in_exile("Grizzly Bears"));
            assert!(!t.in_graveyard(P1, "Grizzly Bears"));
        }),
        // sarulf choice is made as the ability resolves
        ("Sarulf, Realm Eater", || {
            cr!("608.2c", "117.1");
            ruling!(
                "Sarulf, Realm Eater",
                "You choose whether to remove +1/+1 counters as the last ability is resolving."
            );
            let mut t = TestGame::new(2);
            let sarulf = with_counters(&mut t, P0, "Sarulf, Realm Eater", 2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let from = t.asked().len();
            to_upkeep(&mut t);
            assert_eq!(triggers_on_stack(&t, "remove all"), 1);
            // Nothing is asked about the counters as the ability triggers and waits on the
            // stack.
            assert!(!was_asked(&t, P0, from, |d| matches!(
                d,
                Decision::YesNo { .. }
            )));
            // As it resolves, P0's choice is the only decision: no one gets priority between
            // the choice and the exile.
            t.answer_yes(P0, true);
            let resolving = t.asked().len();
            t.resolve_all();
            let asked = asked_since(&t, resolving);
            assert_eq!(asked.len(), 1, "{asked:?}");
            assert!(matches!(asked[0], (P0, Decision::YesNo { .. })));
            assert_eq!(plus1(&t, sarulf), 0);
            assert!(!t.on_battlefield(bears));
        }),
        // sarulf removes only its plus1 counters
        ("Sarulf, Realm Eater", || {
            cr!("122.1");
            ruling!(
                "Sarulf, Realm Eater",
                "Choosing to remove all the +1/+1 counters on Sarulf won't affect any other counters it may have."
            );
            let mut t = TestGame::new(2);
            let sarulf = with_counters(&mut t, P0, "Sarulf, Realm Eater", 1);
            put(&mut t, sarulf, "flying", 1);
            to_upkeep(&mut t);
            t.answer_yes(P0, true);
            t.resolve_all();
            assert_eq!(plus1(&t, sarulf), 0);
            assert_eq!(t.counters(sarulf, "flying"), 1);
        }),
        // iridescent hornbeetle that left doesnt trigger
        ("Iridescent Hornbeetle", || {
            cr!("113.6");
            ruling!(
                "Iridescent Hornbeetle",
                "If Iridescent Hornbeetle leaves the battlefield before your end step, its ability won’t trigger at all."
            );
            let mut t = TestGame::new(2);
            let beetle = t.battlefield(P0, "Iridescent Hornbeetle");
            let bears = t.battlefield(P0, "Grizzly Bears");
            give_plus1(&mut t, bears, 2);
            destroy(&mut t, beetle);
            to_end_step(&mut t, P0);
            assert_eq!(t.stack_len(), 0);
            t.resolve_all();
            assert_eq!(tokens_named(&t, P0, "Insect"), 0);
        }),
        // berta mana trigger uses the stack
        ("Berta, Wise Extrapolator", || {
            cr!("605.1b", "603.3");
            ruling!(
                "Berta, Wise Extrapolator",
                "Berta's second ability isn't a mana ability. It uses the stack and can be responded to."
            );
            supported("Berta, Wise Extrapolator");
            let mut t = TestGame::new(2);
            let berta = t.battlefield(P0, "Berta, Wise Extrapolator");
            give_plus1(&mut t, berta, 1);
            assert_eq!(triggers_on_stack(&t, "add one mana"), 1);
            assert_eq!(t.g.player(P0).mana_pool.total(), 0);
            t.resolve();
            assert_eq!(t.g.player(P0).mana_pool.total(), 1);
        }),
    ];
    for (card, check) in cases {
        eprintln!("case: {card}");
        check();
    }
}
