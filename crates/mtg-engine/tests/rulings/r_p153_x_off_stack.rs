//! Rulings batch P153 — "scales with mana value": the mana value of a card or permanent
//! with {X} in its mana cost anywhere but the stack counts X as 0 (CR 202.3e), and a card
//! with no mana cost has mana value 0 (CR 202.3a). Each card's ruling restates this for
//! the zone its ability looks at: a compact case per card.

use crate::r_p153_common::*;
use crate::r_s01_common::{stack_library, supported, tokens};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::activate_containing;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A permanent with `n` +1/+1 counters (its mana value doesn't change).
fn with_counters(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize]
        .counters
        .insert("+1/+1".into(), n);
    t.g.recompute();
    id
}

/// Runs `p`'s end step from their precombat main phase: the end step triggers are on the
/// stack when this returns.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

#[test]
fn x_is_0_for_a_card_with_x_in_its_cost_in_a_hand_library_or_graveyard() {
    cr!("202.3e", "202.3a");
    ruling!(
        "Friendly Fire",
        "Cards that don't have mana costs, such as land cards, have a mana value of 0."
    );
    ruling!(
        "Ill-Timed Explosion",
        "If a card discarded this way has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Monstrous War-Leech",
        "If a card in a graveyard has an {X} in its mana cost, X is 0."
    );
    ruling!(
        "Darkstar Augur",
        "If a card in a player’s library has {X} in its cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Blast of Genius",
        "If a discarded card has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Mercurial Chemister",
        "If the discarded card has no mana symbols in its upper right corner (because it's a land card, for example), its mana value is 0."
    );
    ruling!(
        "Infernal Genesis",
        "If the card has no mana cost (for instance, if it's a land), then its mana value is 0. This means no tokens will be created."
    );
    ruling!(
        "Heretic's Punishment",
        "If all three cards have a mana value of 0, no damage will be dealt."
    );
    for card in [
        "Friendly Fire",
        "Ill-Timed Explosion",
        "Monstrous War-Leech",
        "Darkstar Augur",
        "Blast of Genius",
        "Mercurial Chemister",
        "Infernal Genesis",
        "Heretic's Punishment",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        match card {
            "Friendly Fire" => {
                // P1's only card in hand is a Forest: no damage to the creature or P1.
                let bears = t.battlefield(P1, "Grizzly Bears");
                t.hand(P1, "Forest");
                let ff = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, ff).target(bears).go();
                t.resolve();
                assert_eq!(t.life(P1), 20, "{card}");
                assert!(t.on_battlefield(bears) && t.obj_now(bears).damage == 0, "{card}");
            }
            "Ill-Timed Explosion" => {
                // Ingenious Prodigy ({X}{U}) and a Forest are discarded: X is 1.
                let prodigy = t.hand(P0, "Ingenious Prodigy");
                let forest = t.hand(P0, "Forest");
                let memnite = t.battlefield(P1, "Memnite");
                let bears = t.battlefield(P1, "Grizzly Bears");
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).go();
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(prodigy), obj(forest)]);
                t.resolve_all();
                assert_eq!(t.zone(memnite), Zone::Graveyard(P1), "{card}");
                assert!(t.on_battlefield(bears), "{card}");
                assert_eq!(t.obj_now(bears).damage, 1, "{card}");
            }
            "Monstrous War-Leech" => {
                t.graveyard(P0, "Ingenious Prodigy");
                t.graveyard(P0, "Forest");
                let leech = t.battlefield(P0, card);
                assert_eq!(t.pt(leech), (1, 1), "{card}");
            }
            "Darkstar Augur" => {
                t.battlefield(P0, card);
                let prodigy = t.library_top(P0, "Ingenious Prodigy");
                next_upkeep(&mut t, P0);
                t.resolve_all();
                assert_eq!(t.zone(prodigy), Zone::Hand(P0), "{card}");
                assert_eq!(t.life(P0), 19, "{card}");
            }
            "Blast of Genius" => {
                let prodigy = t.hand(P0, "Ingenious Prodigy");
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).target(Entity::Player(P1)).go();
                t.answer_choose(P0, &[obj(prodigy)]);
                t.resolve();
                assert_eq!(t.zone(prodigy), Zone::Graveyard(P0), "{card}");
                assert_eq!(t.life(P1), 19, "{card}");
            }
            "Mercurial Chemister" => {
                let chem = t.battlefield(P0, card);
                let forest = t.hand(P0, "Forest");
                let bears = t.battlefield(P1, "Grizzly Bears");
                mana(&mut t, P0, R, 1);
                t.answer_choose(P0, &[obj(forest)]);
                activate_containing(&mut t, P0, chem, "Discard a card").unwrap();
                t.answer_targets(P0, &[obj(bears)]);
                t.resolve_all();
                assert_eq!(t.zone(forest), Zone::Graveyard(P0), "{card}");
                assert_eq!(t.obj_now(bears).damage, 0, "{card}");
            }
            "Infernal Genesis" => {
                t.battlefield(P0, card);
                t.library_top(P0, "Forest");
                next_upkeep(&mut t, P0);
                t.resolve_all();
                assert!(t.in_graveyard(P0, "Forest"), "{card}");
                assert!(tokens(&t, P0).is_empty(), "{card}");
            }
            "Heretic's Punishment" => {
                let hp = t.battlefield(P0, card);
                stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
                mana(&mut t, P0, R, 1);
                mana(&mut t, P0, C, 3);
                activate_containing(&mut t, P0, hp, "mill three").unwrap();
                t.answer_targets(P0, &[Entity::Player(P1)]);
                t.resolve_all();
                assert_eq!(t.graveyard_size(P0), 3, "{card}");
                assert_eq!(t.life(P1), 20, "{card}");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn x_is_0_for_a_permanent_with_x_in_its_cost() {
    cr!("202.3e");
    ruling!(
        "Kethek, Crucible Goliath",
        "If a creature on the battlefield has an {X} in its mana cost, X is 0."
    );
    ruling!(
        "Zur, Eternal Schemer",
        "If a permanent on the battlefield has an {X} in its mana cost, X is zero when determining its mana value."
    );
    ruling!(
        "Feed the Swarm",
        "If a permanent on the battlefield has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Boon of Boseiju",
        "If a permanent on the battlefield has {X} in its mana cost, X is 0 when determining that permanent's mana value."
    );
    ruling!(
        "Traverse Eternity",
        "If a historic permanent has an {X} in its mana cost, the value of {X} is 0 when determining the mana value of that permanent."
    );
    ruling!(
        "Glint Raker",
        "If an artifact on the battlefield has an {X} in its mana cost, that X is 0."
    );
    ruling!(
        "Bosh, Iron Golem",
        "If an artifact on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Fateful Handoff",
        "If an artifact or creature on the battlefield has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Edgar, Master Machinist",
        "If an artifact you control has {X} in its mana cost, X is 0 for the purpose of calculating its mana value."
    );
    ruling!(
        "Sunderflock",
        "If an Elemental on the battlefield has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Earthquake Dragon",
        "If a permanent has {X} in its mana cost, X is 0 when calculating its mana value."
    );
    for card in [
        "Kethek, Crucible Goliath",
        "Zur, Eternal Schemer",
        "Feed the Swarm",
        "Boon of Boseiju",
        "Traverse Eternity",
        "Glint Raker",
        "Bosh, Iron Golem",
        "Fateful Handoff",
        "Edgar, Master Machinist",
        "Sunderflock",
        "Earthquake Dragon",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        match card {
            "Kethek, Crucible Goliath" => {
                // Grizzly Bears (2) is sacrificed; the top card Ingenious Prodigy ({X}{U})
                // is a nonlegendary creature card with lesser mana value (1): it enters
                // (with no counters, as a 0/1).
                t.battlefield(P0, card);
                let bears = t.battlefield(P0, "Grizzly Bears");
                let prodigy = t.library_top(P0, "Ingenious Prodigy");
                to_end_step(&mut t, P0);
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(bears)]);
                t.resolve_all();
                assert!(t.on_battlefield(prodigy), "{card}: {}", t.dump_log());
                assert_eq!(t.pt(prodigy), (0, 1), "{card}");
            }
            "Zur, Eternal Schemer" => {
                // Lattice Library ({X}{G}{G}) becomes a 2/2.
                let zur = t.battlefield(P0, card);
                let lib = t.battlefield(P0, "Lattice Library");
                mana(&mut t, P0, W, 1);
                mana(&mut t, P0, C, 1);
                activate_containing(&mut t, P0, zur, "becomes a creature").unwrap();
                t.answer_targets(P0, &[obj(lib)]);
                t.resolve_all();
                assert!(t.obj_now(lib).is(CardType::Creature), "{card}");
                assert_eq!(t.pt(lib), (2, 2), "{card}");
            }
            "Feed the Swarm" => {
                let ballista = with_counters(&mut t, P1, "Walking Ballista", 2);
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).target(ballista).go();
                t.resolve();
                assert_eq!(t.zone(ballista), Zone::Graveyard(P1), "{card}");
                assert_eq!(t.life(P0), 20, "{card}");
            }
            "Boon of Boseiju" => {
                // Ingenious Prodigy ({X}{U}, 1) and Walking Ballista ({X}{X}, 0): +1/+1.
                let prodigy = with_counters(&mut t, P0, "Ingenious Prodigy", 2);
                with_counters(&mut t, P0, "Walking Ballista", 3);
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).target(prodigy).go();
                t.resolve();
                assert_eq!(t.pt(prodigy), (3, 4), "{card}");
            }
            "Traverse Eternity" => {
                with_counters(&mut t, P0, "Walking Ballista", 3);
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).go();
                t.resolve();
                assert_eq!(t.hand_size(P0), 0, "{card}");
            }
            "Glint Raker" => {
                let raker = t.battlefield(P0, card);
                let chalice = t.battlefield(P0, "Chalice of the Void");
                t.g.objects[chalice.0 as usize]
                    .counters
                    .insert("charge".into(), 2);
                t.g.recompute();
                assert_eq!(t.pt(raker), (1, 3), "{card}");
            }
            "Bosh, Iron Golem" => {
                let bosh = t.battlefield(P0, card);
                let ballista = with_counters(&mut t, P0, "Walking Ballista", 2);
                mana(&mut t, P0, R, 1);
                mana(&mut t, P0, C, 3);
                t.answer_choose(P0, &[obj(ballista)]);
                t.answer_targets(P0, &[Entity::Player(P1)]);
                activate_containing(&mut t, P0, bosh, "Sacrifice an artifact").unwrap();
                t.resolve_all();
                assert_eq!(t.zone(ballista), Zone::Graveyard(P0), "{card}");
                assert_eq!(t.life(P1), 20, "{card}");
            }
            "Fateful Handoff" => {
                let ballista = with_counters(&mut t, P0, "Walking Ballista", 2);
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).target(ballista).go();
                t.resolve();
                assert_eq!(t.hand_size(P0), 0, "{card}");
                assert_eq!(t.obj_now(ballista).controller, P1, "{card}");
            }
            "Edgar, Master Machinist" => {
                let edgar = t.battlefield(P0, card);
                with_counters(&mut t, P0, "Walking Ballista", 3);
                crate::r_s01_common::attack_with(&mut t, &[(edgar, Entity::Player(P1))]);
                t.resolve_all();
                assert_eq!(t.pt(edgar), (2, 4), "{card}");
            }
            "Sunderflock" => {
                // Ivy Elemental ({X}{G}) with three counters reduces the cost by 1 only:
                // {6}{U}{U} is exactly enough.
                with_counters(&mut t, P0, "Ivy Elemental", 3);
                let s = t.hand(P0, card);
                mana(&mut t, P0, U, 2);
                mana(&mut t, P0, C, 5);
                assert!(t.cast(P0, s).try_go().is_err(), "{card}");
                mana(&mut t, P0, C, 1);
                t.cast(P0, s).go();
                assert_eq!(t.g.players[0].mana_pool.total(), 0, "{card}");
            }
            "Earthquake Dragon" => {
                // Shivan Devastator ({X}{R}) with three counters is a Dragon with mana
                // value 1: {13}{G} is exactly enough.
                with_counters(&mut t, P0, "Shivan Devastator", 3);
                let s = t.hand(P0, card);
                mana(&mut t, P0, G, 1);
                mana(&mut t, P0, C, 12);
                assert!(t.cast(P0, s).try_go().is_err(), "{card}");
                mana(&mut t, P0, C, 1);
                t.cast(P0, s).go();
                assert_eq!(t.g.players[0].mana_pool.total(), 0, "{card}");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn x_is_0_for_cards_an_ability_looks_at_in_several_zones() {
    cr!("202.3e");
    ruling!(
        "Eldritch Evolution",
        "If a card in a library has {X} in its mana cost, X is considered to be 0. If you find such a card this way, you won't have an opportunity to spend mana on {X}."
    );
    ruling!(
        "The Aesir Escape Valhalla",
        "If a card in exile has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Death's Oasis",
        "If a creature on the battlefield or a card in your graveyard has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Eddie Brock // Venom, Lethal Protector",
        "If a creature or a card in a player's hand or graveyard has {X} in its mana cost, X is 0 when determining its mana value."
    );
    ruling!(
        "Kodama of the East Tree",
        "If a permanent or a permanent card in your hand has {X} in its mana cost, {X} is considered to be 0."
    );
    ruling!(
        "Smelting Vat",
        "If an artifact revealed this way has an {X} is its mana cost, X is 0."
    );
    ruling!(
        "Birthing Ritual",
        "If one of the revealed creature cards has {X} in its mana cost, X is 0 when determining that card's mana value."
    );
    ruling!(
        "Havi, the All-Father",
        "If a permanent or a card in a graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Orah, Skyclave Hierophant",
        "If a creature on the battlefield or a creature card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    for card in [
        "Eldritch Evolution",
        "The Aesir Escape Valhalla",
        "Death's Oasis",
        "Eddie Brock // Venom, Lethal Protector",
        "Kodama of the East Tree",
        "Smelting Vat",
        "Birthing Ritual",
        "Havi, the All-Father",
        "Orah, Skyclave Hierophant",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        match card {
            "Eldritch Evolution" => {
                // Grizzly Bears is sacrificed (X = 4); Walking Ballista ({X}{X}, 0) is
                // found and enters with no X chosen: no counters, and it dies.
                let bears = t.battlefield(P0, "Grizzly Bears");
                let ballista = t.library_top(P0, "Walking Ballista");
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                let from = t.asked().len();
                t.answer_choose(P0, &[obj(bears)]);
                t.answer_choose(P0, &[obj(ballista)]);
                t.cast(P0, s).go();
                t.resolve();
                assert_eq!(x_asked(&t, P0, from), 0, "{card}");
                assert_eq!(t.zone(ballista), Zone::Graveyard(P0), "{card}");
                assert!(t.in_exile(card), "{card}");
            }
            "The Aesir Escape Valhalla" => {
                let prodigy = t.graveyard(P0, "Ingenious Prodigy");
                t.answer_choose(P0, &[obj(prodigy)]);
                crate::r_s05_common::enter(&mut t, P0, card);
                t.resolve_all();
                assert_eq!(t.zone(prodigy), Zone::Exile, "{card}");
                assert_eq!(t.life(P0), 21, "{card}");
            }
            "Death's Oasis" => {
                // Greatest mana value among Ingenious Prodigy (1) and Walking Ballista
                // (0), both with counters: 1 life.
                let oasis = t.battlefield(P0, card);
                with_counters(&mut t, P0, "Ingenious Prodigy", 2);
                with_counters(&mut t, P0, "Walking Ballista", 3);
                mana(&mut t, P0, C, 1);
                activate_containing(&mut t, P0, oasis, "gain life").unwrap();
                t.resolve_all();
                assert_eq!(t.life(P0), 21, "{card}");
            }
            "Eddie Brock // Venom, Lethal Protector" => {
                // Mana value 1 or less: Ingenious Prodigy (1) and Walking Ballista (0),
                // not Hydroid Krasis ({X}{G}{U}, 2).
                let prodigy = t.graveyard(P0, "Ingenious Prodigy");
                let ballista = t.graveyard(P0, "Walking Ballista");
                let krasis = t.graveyard(P0, "Hydroid Krasis");
                let from = t.asked().len();
                t.answer_targets(P0, &[obj(prodigy)]);
                crate::r_s05_common::enter(&mut t, P0, card);
                t.resolve_all();
                let cands = crate::r_s02_common::target_candidates(&t, P0, from);
                assert_eq!(
                    cands,
                    vec![vec![obj(prodigy), obj(ballista)]],
                    "{card}: {cands:?}"
                );
                assert!(t.on_battlefield(prodigy), "{card}");
                assert_eq!(t.zone(krasis), Zone::Graveyard(P0), "{card}");
            }
            "Kodama of the East Tree" => {
                // Shivan Devastator ({X}{R}, three counters) enters: mana value 1, so
                // Memnite (0) may be put onto the battlefield, Grizzly Bears (2) may not.
                t.battlefield(P0, card);
                let memnite = t.hand(P0, "Memnite");
                let bears = t.hand(P0, "Grizzly Bears");
                let from = t.asked().len();
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(memnite)]);
                // (With no counters, the Devastator then dies as a 0/0.)
                crate::r_s05_common::enter(&mut t, P0, "Shivan Devastator");
                t.resolve_all();
                assert!(t.on_battlefield(memnite), "{card}: {}", t.dump_log());
                let cands = entity_candidates(&t, P0, from);
                assert!(
                    cands.iter().all(|c| !c.contains(&obj(bears))),
                    "{card}: {cands:?}"
                );
                // Grizzly Bears enters: Hydroid Krasis ({X}{G}{U}) in the hand has mana
                // value 2, equal, and may be put onto the battlefield.
                let krasis = t.hand(P0, "Hydroid Krasis");
                let from = t.asked().len();
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(krasis)]);
                crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
                t.resolve_all();
                let cands = entity_candidates(&t, P0, from);
                assert!(
                    cands.iter().any(|c| c.contains(&obj(krasis)) && c.contains(&obj(bears))),
                    "{card}: {cands:?}"
                );
                assert_eq!(x_asked(&t, P0, from), 0, "{card}");
                assert_eq!(t.zone(krasis), Zone::Graveyard(P0), "{card}: a 0/0");
            }
            "Smelting Vat" => {
                // Ornithopter (0) is sacrificed; Chalice of the Void ({X}{X}, 0) may be
                // put onto the battlefield, Sol Ring (1) may not.
                let vat = t.battlefield(P0, card);
                let thopter = t.battlefield(P0, "Ornithopter");
                let lib = stack_library(&mut t, P0, &["Chalice of the Void", "Sol Ring"]);
                mana(&mut t, P0, C, 1);
                // (The total is a requirement on the cards chosen together, not on each
                // candidate: choosing Sol Ring too is an illegal answer.)
                t.answer_choose(P0, &[obj(thopter)]);
                t.answer_choose_moved(P0, &[obj(lib[0]), obj(lib[1])]);
                activate_containing(&mut t, P0, vat, "Reveal the top eight").unwrap();
                t.resolve_all();
                assert_eq!(t.zone(lib[1]), Zone::Library(P0), "{card}: {}", t.dump_log());
                let mut t = TestGame::new(2);
                let vat = t.battlefield(P0, card);
                let thopter = t.battlefield(P0, "Ornithopter");
                let lib = stack_library(&mut t, P0, &["Chalice of the Void", "Sol Ring"]);
                mana(&mut t, P0, C, 1);
                t.answer_choose(P0, &[obj(thopter)]);
                t.answer_choose_moved(P0, &[obj(lib[0])]);
                activate_containing(&mut t, P0, vat, "Reveal the top eight").unwrap();
                t.resolve_all();
                assert!(t.on_battlefield(lib[0]), "{card}: {}", t.dump_log());
                assert_eq!(t.zone(lib[1]), Zone::Library(P0), "{card}");
            }
            "Birthing Ritual" => {
                // Memnite (0) is sacrificed, X = 1: Walking Ballista (0) and Ingenious
                // Prodigy (1) may be put onto the battlefield, Hydroid Krasis (2) may not.
                t.battlefield(P0, card);
                let memnite = t.battlefield(P0, "Memnite");
                let lib = stack_library(
                    &mut t,
                    P0,
                    &["Walking Ballista", "Hydroid Krasis", "Ingenious Prodigy"],
                );
                to_end_step(&mut t, P0);
                let from = t.asked().len();
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(memnite)]);
                t.answer_yes(P0, true);
                t.answer_choose_moved(P0, &[obj(lib[2])]);
                t.resolve_all();
                assert!(t.on_battlefield(lib[2]), "{card}: {}", t.dump_log());
                let cands = entity_candidates(&t, P0, from);
                assert!(
                    cands.iter().any(|c| c.contains(&obj(lib[0]))
                        && c.contains(&obj(lib[2]))
                        && !c.contains(&obj(lib[1]))),
                    "{card}: {cands:?}"
                );
            }
            "Havi, the All-Father" => {
                // Big Bertha ({X}{G}, three counters) dies with mana value 1: Farmer
                // Cotton ({X}{G}{W}, 2) in the graveyard isn't a lesser mana value, so
                // the trigger has no legal target and is removed.
                let havi = t.battlefield(P0, card);
                let bertha = with_counters(&mut t, P0, "Big Bertha", 3);
                let cotton = t.graveyard(P0, "Farmer Cotton");
                crate::r_s02_common::destroy(&mut t, bertha);
                assert_eq!(t.stack_len(), 0, "{card}");
                assert_eq!(t.zone(cotton), Zone::Graveyard(P0), "{card}");
                // Havi (6) dies: Farmer Cotton (2) is returned tapped.
                t.answer_targets(P0, &[obj(cotton)]);
                crate::r_s02_common::destroy(&mut t, havi);
                t.resolve_all();
                assert!(t.on_battlefield(cotton), "{card}");
                assert!(t.obj_now(cotton).tapped, "{card}");
            }
            "Orah, Skyclave Hierophant" => {
                // Mikaeus, the Lunarch ({X}{W}, three counters) dies with mana value 1:
                // Progenitor Exarch ({X}{X}{W}, 1) in the graveyard isn't lesser, so the
                // trigger has no legal target and is removed.
                let orah = t.battlefield(P0, card);
                let mik = with_counters(&mut t, P0, "Mikaeus, the Lunarch", 3);
                let exarch = t.graveyard(P0, "Progenitor Exarch");
                crate::r_s02_common::destroy(&mut t, mik);
                assert_eq!(t.stack_len(), 0, "{card}");
                // Orah (4) dies: Mikaeus (1) in the graveyard is a legal target, Exarch
                // (1) too.
                let mik = t.g.current(mik);
                let from = t.asked().len();
                t.answer_targets(P0, &[obj(mik)]);
                crate::r_s02_common::destroy(&mut t, orah);
                assert_eq!(t.stack_len(), 1, "{card}");
                let cands = crate::r_s02_common::target_candidates(&t, P0, from);
                assert_eq!(cands, vec![vec![obj(exarch), obj(mik)]], "{card}");
                t.resolve_all();
                assert_eq!(t.zone(exarch), Zone::Graveyard(P0), "{card}");
                assert!(t.g.current(mik) != mik, "{card}: Mikaeus left the graveyard");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn a_library_of_lands_only_gives_x_0_and_no_damage() {
    cr!("202.3a", "107.2");
    ruling!(
        "Goblin Machinist",
        "If all the cards in your library are lands, X is zero."
    );
    ruling!(
        "Erratic Explosion",
        "If all your cards are lands, no damage is dealt."
    );
    ruling!(
        "Explosive Revelation",
        "If all the cards in your library are land cards, you’ll reveal all of them, then arrange them in your library in whatever order you like. No damage is dealt."
    );
    for card in ["Goblin Machinist", "Erratic Explosion", "Explosive Revelation"] {
        supported(card);
        let mut t = TestGame::new(2);
        set_library(&mut t, P0, &["Forest", "Mountain", "Plains"]);
        match card {
            "Goblin Machinist" => {
                let gm = t.battlefield(P0, card);
                mana(&mut t, P0, R, 1);
                mana(&mut t, P0, C, 2);
                activate_containing(&mut t, P0, gm, "Reveal cards").unwrap();
                t.resolve_all();
                assert_eq!(t.pt(gm), (0, 5), "{card}");
            }
            _ => {
                let s = t.hand(P0, card);
                mana_for(&mut t, P0, card);
                t.cast(P0, s).target(Entity::Player(P1)).go();
                t.resolve();
                assert_eq!(t.life(P1), 20, "{card}");
            }
        }
        assert_eq!(t.library_size(P0), 3, "{card}");
        assert_eq!(t.hand_size(P0), 0, "{card}");
    }
}

#[test]
fn emissary_escort_has_power_0_off_the_battlefield() {
    cr!("604.2", "113.6");
    ruling!(
        "Emissary Escort",
        "Emissary Escort’s ability applies only while it is on the battlefield. In all other zones, its power is 0."
    );
    supported("Emissary Escort");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bosh, Iron Golem");
    let on = t.battlefield(P0, "Emissary Escort");
    assert_eq!(t.pt(on), (8, 4));
    let in_hand = t.hand(P0, "Emissary Escort");
    assert_eq!(t.g.obj(in_hand).power(), 0);
    let in_gy = t.graveyard(P0, "Emissary Escort");
    assert_eq!(t.g.obj(in_gy).power(), 0);
}

#[test]
fn bosh_can_sacrifice_itself_for_8_damage() {
    cr!("602.2", "113.7a");
    ruling!(
        "Bosh, Iron Golem",
        "Bosh can be sacrificed to pay the cost of its last ability."
    );
    let mut t = TestGame::new(2);
    let bosh = t.battlefield(P0, "Bosh, Iron Golem");
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, C, 3);
    t.answer_choose(P0, &[obj(bosh)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, bosh, "Sacrifice an artifact").unwrap();
    assert_eq!(t.zone(bosh), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "{}", t.dump_log());
}
