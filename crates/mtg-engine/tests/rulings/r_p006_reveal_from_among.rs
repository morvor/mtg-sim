//! Rulings batch P006 — "You may reveal a [card] from among them and put it into your
//! hand": the card chosen is revealed (CR 701.20a), and only that card. Augur of Bolas'
//! ruling ("You don't reveal the other two cards") exposed that these compiled to an
//! `Effect::Dig` that revealed nothing; they now compile to the dig's steps
//! (`DigStep::Take` with `reveal`). This file checks every card whose compiled abilities
//! that change touched, in a game: its revealing dig, run with a library of cards it can
//! take, reveals the card put into the hand and nothing else.

use crate::r_p006_common::*;
use crate::r_s01_common::{stack_library, supported};
use mtg_engine::ability::{AbilityKind, Effect, Filter};
use mtg_engine::card::card;
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use serde_json::Value as Json;

/// The cards whose compiled abilities changed when "you may reveal ... from among them"
/// moved to the dig's steps (every one of them is fully supported).
const CHANGED: &[&str] = &[
    "Adaptive Omnitool",
    "Adventure Awaits",
    "Adventurous Impulse",
    "Ajani's Influence",
    "Ajani, Mentor of Heroes",
    "Ancient Stirrings",
    "Arcane Infusion",
    "Arcanist's Owl",
    "Ashe, Princess of Dalmasca",
    "Astor, Bearer of Blades",
    "Augur of Bolas",
    "Avengers Tower",
    "Basic Conjuration",
    "Beastrider Vanguard",
    "Belisarius Cawl",
    "Blossom Prancer",
    "Board the Weatherlight",
    "Bond of Flourishing",
    "Boromir, Gondor's Hope",
    "Boughside Wanderers",
    "Brazen Upstart",
    "Brightwood Tracker",
    "Bucky Barnes, Eager Ally",
    "Calix, Destiny's Hand",
    "Carth the Lion",
    "Case File Auditor",
    "Casey Jones, Jury-Rig Justiciar",
    "Commune with Beavers",
    "Commune with Dinosaurs",
    "Commune with Nature",
    "Commune with Spirits",
    "Conjurer's Mantle",
    "Contagious Vorrac",
    "Coralhelm Chronicler",
    "Courageous Outrider",
    "Cowabunga!",
    "Definitely Not a Turtle",
    "Director Nick Fury",
    "Domri, Chaos Bringer",
    "Doomskar Warrior",
    "Dragonologist",
    "Duskwatch Recruiter // Krallenhorde Howler",
    "Dáin's Company",
    "Eclipsed Boggart",
    "Eclipsed Elf",
    "Eclipsed Flamekin",
    "Eclipsed Kithkin",
    "Eclipsed Merrow",
    "Ellywick Tumblestrum",
    "Faerie Mechanist",
    "Floriferous Vinewall",
    "Follow the Lumarets",
    "Foul Emissary",
    "Frontier Seeker",
    "Garruk's Harbinger",
    "Gavony Dawnguard",
    "Glint-Nest Crane",
    "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun",
    "Guru Pathik",
    "Harald, King of Skemfar",
    "Heroes' Podium",
    "Horn of the Mark",
    "Howlpack Piper // Wildsong Howler",
    "Icon of Ancestry",
    "Incubation // Incongruity",
    "Ingenious Smith",
    "Invasion of Ixalan // Belligerent Regisaur",
    "Invasion of Ravnica // Guildpact Paragon",
    "Kolaghan Warmonger",
    "Kolvori, God of Kinship // The Ringhart Crest",
    "Liliana's Spoils",
    "Living Phone",
    "Lupinflower Village",
    "Master Piandao",
    "Meletis Astronomer",
    "Memorial to Unity",
    "Militia Bugler",
    "Monumental Henge",
    "Nahiri, Heir of the Ancients",
    "Narset, Parter of Veils",
    "Nessian Game Warden",
    "Nessian Wanderer",
    "Oath of Nissa",
    "Once Upon a Time",
    "Orb of Dragonkind",
    "Overgrown Pest",
    "Paradox Surveyor",
    "Peer Through Depths",
    "Pictures of Spider-Man",
    "Pulsar Squadron Ace",
    "Quandrix Apprentice",
    "Reality Heist",
    "Recruitment Officer",
    "Rosecot Knight",
    "Sandstalker Moloch",
    "Sarkhan's Dragonfire",
    "Search for Azcanta // Azcanta, the Sunken Ruin",
    "Search for Dagger",
    "Seek the Wilds",
    "Seismic Sense",
    "Sigarda, Champion of Light",
    "Silundi Vision // Silundi Isle",
    "Siona, Captain of the Pyleas",
    "Skalla Wolf",
    "Squad Rallier",
    "Star Charter",
    "Staunch Crewmate",
    "Stoneforge Acolyte",
    "Sumala Woodshaper",
    "Tapping at the Window",
    "Tazri, Beacon of Unity",
    "Tezzeret's Gatebreaker",
    "Tezzeret, Agent of Bolas",
    "The Antiquities War",
    "The World Spell",
    "Thran Spider",
    "Thundertrap Trainer",
    "Tony Stark // The Invincible Iron Man",
    "Trail of Crumbs",
    "Twists and Turns // Mycoid Maze",
    "Vivien Reid",
    "Vivien's Talent",
    "Wandering Mind",
    "Warm Welcome",
    "Water Tribe Rallier",
    "Weatherlight",
    "You Meet in a Tavern"
];

/// Takes the first candidates offered (as many as allowed, at least one), says yes.
struct TakeFirst;

impl Agent for TakeFirst {
    fn decide(&mut self, _g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::ChooseEntities {
                candidates, min, max, ..
            } => {
                let n = (*max).max(*min).max(1) as usize;
                Answer::Entities(candidates.iter().take(n.min(candidates.len())).copied().collect())
            }
            Decision::YesNo { .. } => Answer::Bool(true),
            Decision::Priority { .. } => Answer::Action(mtg_engine::decision::Action::Pass),
            _ => Answer::Default,
        }
    }
}

/// The first revealing `DigStep::Take` in `v`.
fn revealing_take(v: &Json) -> Option<&Json> {
    match v {
        Json::Object(m) => {
            if let Some(t) = m.get("DigStep").and_then(|d| d.get("Take")) {
                if t.get("reveal") == Some(&Json::Bool(true)) {
                    return Some(t);
                }
            }
            m.values().find_map(revealing_take)
        }
        Json::Array(a) => a.iter().find_map(revealing_take),
        _ => None,
    }
}

/// The innermost `Seq` in `v` that starts a dig (has an `Effect::Dig` among its items) and
/// holds a revealing `DigStep::Take`, and that step.
fn revealing_seq(v: &Json) -> Option<(Json, Json)> {
    match v {
        Json::Object(m) => {
            if let Some(Json::Array(items)) = m.get("Seq") {
                if let Some(found) = items.iter().find_map(revealing_seq) {
                    return Some(found);
                }
                if items.iter().any(|e| e.get("Dig").is_some()) {
                    if let Some(t) = revealing_take(v) {
                        return Some((v.clone(), t.clone()));
                    }
                }
                return None;
            }
            m.values().find_map(revealing_seq)
        }
        Json::Array(a) => a.iter().find_map(revealing_seq),
        _ => None,
    }
}

/// Cards to fill the library with: the first one the dig can take is used.
const CANDIDATES: &[&str] = &[
    "Grizzly Bears", "Forest", "Lightning Bolt", "Divination", "Ornithopter", "Pacifism",
    "Llanowar Elves", "Goblin Raider", "Serra Angel", "Shivan Dragon", "Chandra Nalaar",
    "Mogg Fanatic", "Memnite", "Plains", "Island", "Swamp", "Mountain", "Elvish Visionary",
    "Youthful Knight", "Spider-Ham, Peter Porker", "Bonesplitter", "Isamaru, Hound of Konda",
    "Urborg Skeleton", "Doom Blade", "Boros Swiftblade", "Wedgelight Rammer",
    "Summon: Primal Odin", "Airdrop Aeronauts", "Birds of Paradise",
    "Pipsqueak, Rebel Strongarm",
];

#[test]
fn revealing_digs_reveal_the_card_taken() {
    cr!("701.20a", "608.2c");
    let mut untested = vec![];
    let mut failed: Vec<String> = vec![];
    for name in CHANGED {
        supported(name);
        let def = card(name);
        let mut found = None;
        for f in [Some(def.front()), def.back()].into_iter().flatten() {
            for a in &f.chars.abilities {
                let j = serde_json::to_value(&a.kind).unwrap();
                if let Some(x) = revealing_seq(&j) {
                    found = Some(x);
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        let (seq, take) = found.unwrap_or_else(|| panic!("{name}: no revealing dig"));
        let effect: Effect = serde_json::from_value(seq).unwrap();
        let filter: Filter = serde_json::from_value(take["filter"].clone()).unwrap();
        // A card the dig can take.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        // "Of the chosen type" (Icon of Ancestry): Bear. "That creature" (the equipped
        // creature that attacked, Conjurer's Mantle): a Bear.
        t.g.objects[src.0 as usize].choices.creature_type = Some("Bear".into());
        let bear = t.battlefield(P0, "Grizzly Bears");
        let event = mtg_engine::object::EventInfo {
            amount: 3,
            object: Some(bear),
            player: Some(P1),
            ..Default::default()
        };
        let mut ctx = mtg_engine::eval::Ctx::for_object(&t.g, src);
        ctx.event = Some(event.clone());
        let pick = CANDIDATES.iter().find(|c| {
            let id = t.library_top(P0, c);
            t.g.matches(id, &filter, &ctx)
        });
        let Some(pick) = pick else {
            untested.push(*name);
            continue;
        };
        t.g.player_mut(P0).library.clear();
        stack_library(&mut t, P0, &[*pick; 12]);
        t.lands(P0, "Forest", 3);
        t.g.agents.0.lock().unwrap()[0] = Box::new(TakeFirst);
        let mut ctx = mtg_engine::eval::Ctx::for_object(&t.g, src);
        // "Look at the top X cards": X from a cost is five; "that many" (combat damage) is
        // three (the event); "the number of lands (Forests) you control" is three.
        ctx.x = 5;
        ctx.x_defined = true;
        ctx.event = Some(event);
        let hand_before = t.hand_size(P0);
        t.g.exec(&effect, &mut ctx);
        t.g.flush_events();
        let revealed: Vec<ObjectId> = t
            .g
            .turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
                _ => None,
            })
            .collect();
        let in_hand: Vec<ObjectId> = t.g.player(P0).hand[hand_before..].to_vec();
        let only_taken = revealed
            .iter()
            .all(|r| t.zone(t.g.current(*r)) == Zone::Hand(P0));
        if in_hand.is_empty() || revealed.is_empty() || !only_taken {
            failed.push(format!(
                "{name}: took {} revealed {} only-taken {only_taken}",
                in_hand.len(),
                revealed.len()
            ));
        }
    }
    assert!(failed.is_empty(), "{failed:#?}");
    assert!(untested.is_empty(), "no candidate card for {untested:?}");
}
