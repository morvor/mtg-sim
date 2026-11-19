//! Rulings batch P084 — graveyard hate that casts the exiled card or a copy of it
//! "without paying its mana cost" (CR 118.9, 608.2g, 707.12): no alternative cost
//! (CR 118.9a), optional additional costs may be paid and mandatory ones must be
//! (CR 601.2b, 601.2f), X is 0 (CR 107.3b), and the cast happens during the resolution.
//! One scenario per card, run with a kicker spell, a spell with a mandatory additional
//! cost and a spell with an alternative cost.

use crate::r_p084_common::*;
use crate::r_s01_common::tapped_lands;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::draft::{Draft, DraftStyle};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The cards that cast a card or a copy of it without paying its mana cost.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Via {
    MemoryPlunder,
    Wrexial,
    Diluvian,
    WildfireDevils,
    Reenact,
    MnemonicDeluge,
    AetherSearcher,
}

const ALL: [Via; 7] = [
    Via::MemoryPlunder,
    Via::Wrexial,
    Via::Diluvian,
    Via::WildfireDevils,
    Via::Reenact,
    Via::MnemonicDeluge,
    Via::AetherSearcher,
];

impl Via {
    fn name(self) -> &'static str {
        match self {
            Via::MemoryPlunder => "Memory Plunder",
            Via::Wrexial => "Wrexial, the Risen Deep",
            Via::Diluvian => "Diluvian Primordial",
            Via::WildfireDevils => "Wildfire Devils",
            Via::Reenact => "Reenact the Crime",
            Via::MnemonicDeluge => "Mnemonic Deluge",
            Via::AetherSearcher => "Aether Searcher",
        }
    }

    /// A game for the card (Aether Searcher noted `spell`'s name as it was drafted).
    fn game(self, spell: &str) -> TestGame {
        if self != Via::AetherSearcher {
            return TestGame::new(2);
        }
        let mut d = Draft::new(
            vec![vec![vec![card("Aether Searcher"), card(spell)]]],
            DraftStyle::Conspiracy,
            1,
        );
        d.pick(P0, &[0]).unwrap();
        d.pick(P0, &[0]).unwrap();
        let (_, info) = d.finish();
        let mut t = TestGame::new(3);
        t.g.start.draft = info;
        t.set_step(P0, Step::PrecombatMain);
        t
    }

    /// Puts `spell` where the card finds it and queues the answers asked before the
    /// spell (or its copy) is cast. Returns the spell card.
    fn setup(self, t: &mut TestGame, spell: &str) -> ObjectId {
        supported(self.name());
        match self {
            Via::MemoryPlunder | Via::Wrexial | Via::Diluvian => {
                let c = t.graveyard(P1, spell);
                t.answer_targets(P0, &[obj(c)]);
                t.answer_yes(P0, true);
                c
            }
            Via::WildfireDevils => {
                // Whichever player is chosen at random exiles their copy of the card.
                t.graveyard(P0, spell);
                let c = t.graveyard(P1, spell);
                t.answer_yes(P0, true);
                c
            }
            Via::Reenact => {
                // Put into the graveyard this turn.
                let c = t.hand(P1, spell);
                let c = move_to(t, c, Zone::Graveyard(P1)).unwrap();
                t.answer_targets(P0, &[obj(c)]);
                t.answer_yes(P0, true);
                c
            }
            Via::MnemonicDeluge => {
                let c = t.graveyard(P1, spell);
                t.answer_targets(P0, &[obj(c)]);
                // Cast the first of the three copies only.
                t.answer_yes(P0, true);
                t.answer_yes(P0, false);
                t.answer_yes(P0, false);
                c
            }
            Via::AetherSearcher => {
                let c = t.hand(P0, spell);
                // Search the hand, not the library; the noted card; cast it.
                t.answer_yes(P0, true);
                t.answer_yes(P0, false);
                t.answer_choose(P0, &[obj(c)]);
                t.answer_yes(P0, true);
                c
            }
        }
    }

    /// The combat damage the card itself deals to P1 in the scenario.
    fn combat_damage(self) -> i32 {
        if self == Via::Wrexial {
            5
        } else {
            0
        }
    }

    /// Runs the card so that `spell` (or a copy) is cast, and resolves everything.
    fn go(self, t: &mut TestGame) {
        match self {
            Via::MemoryPlunder => {
                cast_modes(t, P0, "Memory Plunder", "UUUU", &[], &[]);
            }
            Via::Wrexial => {
                let w = t.battlefield(P0, "Wrexial, the Risen Deep");
                t.attack(&[(w, pl(P1))], &[]);
            }
            Via::Diluvian => {
                t.enter(P0, "Diluvian Primordial");
            }
            Via::WildfireDevils => {
                t.enter(P0, "Wildfire Devils");
            }
            Via::Reenact => {
                cast_modes(t, P0, "Reenact the Crime", "1UUU", &[], &[]);
            }
            Via::MnemonicDeluge => {
                cast_modes(t, P0, "Mnemonic Deluge", "6UUU", &[], &[]);
            }
            Via::AetherSearcher => {
                t.enter(P0, "Aether Searcher");
            }
        }
        t.resolve_all();
    }
}

fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> Option<ObjectId> {
    crate::r_s05_common::move_to(t, id, zone)
}

/// Whether anyone was asked to choose a casting method (an alternative cost) since `from`.
fn asked_method(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseCastingMethod { .. } => true,
        Decision::ChooseOption { prompt, .. } => prompt.contains("how to cast"),
        _ => false,
    })
}

/// The number of cards P0 drew (library fillers in hand).
fn drawn(t: &TestGame) -> usize {
    t.g.player(P0)
        .hand
        .iter()
        .filter(|x| t.g.obj(**x).chars.name == "Filler")
        .count()
}

/// Burst Lightning (kicker {4}) is cast kicked: the kicker is paid with four lands.
fn kicker(via: Via) {
    let name = via.name();
    let mut t = via.game("Burst Lightning");
    via.setup(&mut t, "Burst Lightning");
    t.lands(P0, "Mountain", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    via.go(&mut t);
    let log = t.dump_log();
    assert!(!asked_method(&t, from), "{name}");
    assert_eq!(
        tapped_lands(&t, P0),
        4,
        "{name}: the kicker was paid: {log}"
    );
    assert_eq!(t.life(P1), 16 - via.combat_damage(), "{name}: {log}");
}

/// Village Rites ("As an additional cost to cast this spell, sacrifice a creature.") is
/// cast: the creature is sacrificed.
fn mandatory(via: Via) {
    let name = via.name();
    let mut t = via.game("Village Rites");
    via.setup(&mut t, "Village Rites");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(elves)]);
    via.go(&mut t);
    let log = t.dump_log();
    assert!(t.in_graveyard(P0, "Llanowar Elves"), "{name}: {log}");
    assert_eq!(drawn(&t), 2, "{name}: {log}");
}

/// Invigorate ("If you control a Forest, rather than pay this spell's mana cost, you may
/// have an opponent gain 3 life.") is cast, but not for its alternative cost.
fn alternative(via: Via) {
    let name = via.name();
    let mut t = via.game("Invigorate");
    via.setup(&mut t, "Invigorate");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[obj(elves)]);
    let from = t.asked().len();
    via.go(&mut t);
    let log = t.dump_log();
    assert!(!asked_method(&t, from), "{name}: {log}");
    assert_eq!(t.life(P1), 20 - via.combat_damage(), "{name}: {log}");
    assert_eq!(t.pt(elves), (5, 5), "{name}: {log}");
}

#[test]
fn free_casts_pay_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "601.2b", "601.2f", "707.12");
    ruling!(
        "Aether Searcher",
        "If you cast a card \"without paying its mana cost,\" you can't pay alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Diluvian Primordial",
        "If you cast a card \"without paying its mana cost,\" you can't pay alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Wrexial, the Risen Deep",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. On the other hand, if the card has additional costs (such as kicker or multikicker), you may pay those."
    );
    ruling!(
        "Memory Plunder",
        "If you cast a card “without paying its mana cost,” you can't pay any alternative costs. On the other hand, if the card has additional costs (such as conspire), you may pay those."
    );
    ruling!(
        "Wildfire Devils",
        "If you cast a card “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Momentous Fall, you must pay those to cast the card."
    );
    ruling!(
        "Reenact the Crime",
        "If you cast a spell \"without paying its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, such as that of Demand Answers, those must be paid to cast the card."
    );
    ruling!(
        "Mnemonic Deluge",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast it."
    );
    for via in ALL {
        kicker(via);
        mandatory(via);
        alternative(via);
    }
}
