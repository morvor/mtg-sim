//! Shared helpers for the tests of rulings batch S22 (`r_s22_*.rs`): casting spells —
//! costs and alternative costs, casting without paying the mana cost, cost reductions,
//! modes, timing, and what happens while a spell is being cast. (The helpers of batches
//! S01–S20 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// A way an effect lets P0 cast a card without paying its mana cost (CR 118.9).
pub struct FreeCaster {
    /// The effect can cast only instant cards (the mandatory additional cost is then
    /// checked with Village Rites rather than Tormenting Voice).
    pub instants_only: bool,
    /// Puts the real card `name` where the effect casts it from, setting up whatever the
    /// effect needs, and returns the card.
    pub place: fn(&mut TestGame, &str) -> ObjectId,
    /// Makes the effect happen, P0 choosing to cast `card`: `answers` queues P0's answers
    /// for casting the spell (called once the effect's own choices are queued). Runs
    /// until the stack is empty. Pays for the effect with mana added to P0's pool, never
    /// with lands.
    pub run: fn(&mut TestGame, ObjectId, &dyn Fn(&mut TestGame)),
}

/// The number of lands named `name` that `p` controls and that are tapped.
pub fn tapped_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name && o.tapped)
        .count()
}

/// CR 118.9a–b: a spell cast without paying its mana cost can't be cast for an
/// alternative cost, but its optional additional costs (kicker) may be paid and its
/// mandatory additional costs must be.
pub fn check_free_cast_costs(fc: &FreeCaster) {
    // Kicker {4} may be paid: kicked, Burst Lightning deals 4 damage to Hill Giant rather
    // than 2.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = (fc.place)(&mut t, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    (fc.run)(&mut t, bolt, &|t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert_eq!(tapped_named(&t, P0, "Wastes"), 4, "the kicker cost was paid");
    assert!(t.in_graveyard(P1, "Hill Giant"), "the spell was kicked");
    // Not kicked: 2 damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = (fc.place)(&mut t, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    (fc.run)(&mut t, bolt, &|t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert_eq!(tapped_named(&t, P0, "Wastes"), 0);
    assert!(t.on_battlefield(giant));
    // It resolved (the card may be P1's, cast from P1's library or graveyard).
    assert!(
        t.in_graveyard(P0, "Burst Lightning")
            || t.in_graveyard(P1, "Burst Lightning")
            || t.in_exile("Burst Lightning")
    );
    assert_eq!(t.stack_len(), 0);

    // A mandatory additional cost must be paid.
    let (name, fodder) = if fc.instants_only {
        // "As an additional cost to cast this spell, sacrifice a creature. Draw two cards."
        ("Village Rites", "Grizzly Bears")
    } else {
        // "As an additional cost to cast this spell, discard a card. Draw two cards."
        ("Tormenting Voice", "Forest")
    };
    let mut t = TestGame::new(2);
    let spell = (fc.place)(&mut t, name);
    if fc.instants_only {
        t.battlefield(P0, fodder);
    } else {
        t.hand(P0, fodder);
    }
    let lib = t.library_size(P0);
    (fc.run)(&mut t, spell, &|_| {});
    assert!(t.in_graveyard(P0, fodder), "the additional cost was paid");
    assert_eq!(t.library_size(P0), lib - 2, "the spell resolved");
    // Without a creature to sacrifice (a card to discard), it can't be cast.
    let mut t = TestGame::new(2);
    let spell = (fc.place)(&mut t, name);
    let zone = t.zone(spell);
    let lib = t.library_size(P0);
    (fc.run)(&mut t, spell, &|_| {});
    assert_eq!(t.library_size(P0), lib, "the spell wasn't cast");
    // It stays where it was (or where the effect exiled it before offering to cast it).
    assert!(t.zone(spell) == zone || t.zone(spell) == Zone::Exile);
    assert_eq!(t.stack_len(), 0);

    // No alternative cost: Cyclonic Rift can't be cast for its overload cost {6}{U}.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rift = (fc.place)(&mut t, "Cyclonic Rift");
    t.lands(P0, "Island", 7);
    let from = t.asked().len();
    (fc.run)(&mut t, rift, &|t| {
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    assert_eq!(tapped_named(&t, P0, "Island"), 0);
    let offered_overload = t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseOption { options, .. } => {
            options.iter().any(|o| o.to_lowercase().contains("overload"))
        }
        Decision::ChooseCastingMethod { options, .. } => {
            options.iter().any(|o| o.to_lowercase().contains("overload"))
        }
        _ => false,
    });
    assert!(!offered_overload, "overload was offered");
}

/// Moves P0 to combat and has `attacker` attack P1 unblocked; the rest of combat is
/// played out (both players passing), resolving the triggered abilities.
pub fn attack_p1_unblocked(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

/// Adds `n` colorless mana to P0's pool (to pay for an effect without tapping lands).
pub fn pool(t: &mut TestGame, n: u32) {
    add_mana(t, P0, ManaType::C, n);
}

/// Where the object (followed across zone changes) is now.
pub fn zone_now(t: &TestGame, id: ObjectId) -> Zone {
    t.zone(id)
}

/// Wraps `p`'s agent: the first "choose" decision offering an object named `name` is
/// answered with that object (a card that has changed zones is a new object by then,
/// CR 400.7, so its id can't be scripted in advance); every other decision gets the
/// scripted answer.
pub fn choose_named_when_offered(t: &mut TestGame, p: PlayerId, name: &str) {
    struct PickNamed {
        inner: Box<dyn mtg_engine::decision::Agent>,
        name: String,
        done: bool,
    }
    impl mtg_engine::decision::Agent for PickNamed {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if !self.done {
                if let Decision::ChooseEntities { candidates, .. } = d {
                    let hit = candidates.iter().find(|e| match e {
                        Entity::Object(o) => g.obj(*o).chars.name == self.name.as_str(),
                        _ => false,
                    });
                    if let Some(e) = hit {
                        self.done = true;
                        return Answer::Entities(vec![*e]);
                    }
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(PickNamed {
        inner,
        name: name.to_string(),
        done: false,
    });
}
