//! Helpers shared by the mtg-api tests.
#![allow(dead_code)]

use mtg_engine::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::{Arc, OnceLock};

/// Nonland cards for random decks: fully supported, mana value at most 6, with their
/// colour identity as a WUBRG mask.
fn pool() -> &'static [(String, u8)] {
    static POOL: OnceLock<Vec<(String, u8)>> = OnceLock::new();
    POOL.get_or_init(|| {
        let mut v = Vec::new();
        for c in mtg_data::cards().iter() {
            if !c.is_playable_card() || !c.is_legal_somewhere() {
                continue;
            }
            if !c.games.is_empty() && c.games.iter().all(|g| g != "paper") {
                continue;
            }
            let front = c.type_line.as_deref().unwrap_or("");
            if front.is_empty() || front.contains("Land") || c.cmc.unwrap_or(0.0) > 6.0 {
                continue;
            }
            let mask = c.color_identity.iter().fold(0u8, |m, s| {
                m | match s.as_str() {
                    "W" => 1,
                    "U" => 2,
                    "B" => 4,
                    "R" => 8,
                    "G" => 16,
                    _ => 0,
                }
            });
            v.push((c.name.clone(), mask));
        }
        v
    })
}

/// A random 40-card deck: one or two colours, 17 basic lands, 23 spells.
pub fn random_deck(seed: u64) -> Vec<Arc<CardDef>> {
    const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut colors: Vec<usize> = (0..5).collect();
    colors.shuffle(&mut rng);
    colors.truncate(rng.gen_range(1..=2));
    let mask = colors.iter().fold(0u8, |m, &c| m | 1 << c);
    let candidates: Vec<&str> = pool()
        .iter()
        .filter(|(_, ci)| ci & !mask == 0)
        .map(|(n, _)| n.as_str())
        .collect();
    let db = CardDb::global();
    let mut deck: Vec<Arc<CardDef>> = (0..17)
        .map(|i| db.get(BASICS[colors[i % colors.len()]]).expect("basic"))
        .collect();
    let mut spells: Vec<Arc<CardDef>> = Vec::new();
    for _ in 0..2000 {
        if spells.len() == 12 {
            break;
        }
        let name = *candidates.choose(&mut rng).expect("cards");
        if spells.iter().any(|d| d.name.eq_ignore_ascii_case(name)) {
            continue;
        }
        if let Some(d) = db.get(name).filter(|d| d.is_fully_supported()) {
            spells.push(d);
        }
    }
    for i in 0..23 {
        if let Some(s) = spells.get(i % spells.len().max(1)) {
            deck.push(s.clone());
        }
    }
    deck
}

/// A simple two-colour deck of well-known cards.
pub fn simple_deck() -> Vec<Arc<CardDef>> {
    let db = CardDb::global();
    let mut v = Vec::new();
    for (n, name) in [
        (9, "Mountain"),
        (8, "Forest"),
        (3, "Grizzly Bears"),
        (3, "Lightning Bolt"),
        (2, "Hill Giant"),
        (2, "Shock"),
        (3, "Llanowar Elves"),
        (2, "Giant Growth"),
        (2, "Craw Wurm"),
        (2, "Raging Goblin"),
        (2, "Prodigal Pyromancer"),
        (2, "Rampant Growth"),
    ] {
        for _ in 0..n {
            v.push(db.get(name).expect(name));
        }
    }
    v
}

/// A random valid-looking answer to a request (like `examples/random_client.py`): random
/// legal options, preferring actions over passing at priority. Answers that break a
/// rule only the engine checks (attack requirements, for example) are possible; the
/// agent is then asked again.
pub fn random_answer(req: &mtg_api::Request, rng: &mut ChaCha8Rng) -> mtg_api::JsonAnswer {
    use mtg_api::{AnswerSpec, JsonAnswer};
    let n = req.options.len();
    match &req.answer {
        AnswerSpec::ChooseOne => {
            if req.kind == "priority" {
                let acts: Vec<usize> = req
                    .options
                    .iter()
                    .filter(|o| {
                        o.action.as_ref().is_some_and(|a| {
                            !matches!(a.kind.as_str(), "pass" | "concede" | "mana_ability")
                        })
                    })
                    .map(|o| o.index)
                    .collect();
                if !acts.is_empty() && rng.gen_bool(0.6) {
                    return JsonAnswer::index(*acts.choose(rng).unwrap());
                }
                return JsonAnswer::index(0);
            }
            JsonAnswer::index(rng.gen_range(0..n.max(1)))
        }
        AnswerSpec::YesNo => JsonAnswer::yes_no(req.kind != "mulligan" && rng.gen_bool(0.5)),
        AnswerSpec::ChooseMany {
            min,
            max,
            distinct,
            budget,
        } => {
            let hi = if *distinct {
                (*max as usize).min(n)
            } else {
                *max as usize
            };
            let want = rng.gen_range(*min as usize..=hi.max(*min as usize));
            let mut order: Vec<usize> = (0..n).collect();
            order.shuffle(rng);
            if !distinct && n > 0 {
                order = (0..want).map(|_| rng.gen_range(0..n)).collect();
            }
            let mut chosen = Vec::new();
            let mut groups: std::collections::BTreeMap<u32, u32> = Default::default();
            let mut spent = 0;
            for i in order {
                if chosen.len() >= want {
                    break;
                }
                let o = &req.options[i];
                if let (Some(g), Some(m)) = (o.group, o.group_max) {
                    if groups.get(&g).copied().unwrap_or(0) >= m {
                        continue;
                    }
                }
                let c = o.cost.unwrap_or(0);
                if budget.is_some_and(|b| spent + c > b) {
                    continue;
                }
                spent += c;
                if let Some(g) = o.group {
                    *groups.entry(g).or_default() += 1;
                }
                chosen.push(i);
            }
            JsonAnswer::indices(chosen)
        }
        AnswerSpec::Order => {
            let mut v: Vec<usize> = (0..n).collect();
            v.shuffle(rng);
            JsonAnswer::indices(v)
        }
        AnswerSpec::Number { min, max } => {
            let hi = max.unwrap_or(min + 2).min(min + 10);
            JsonAnswer::number(rng.gen_range(*min..=hi.max(*min)))
        }
        AnswerSpec::Divide { total, min_each } => {
            let mut v = vec![*min_each as i64; n];
            for _ in 0..(*total as i64 - *min_each as i64 * n as i64).max(0) {
                v[rng.gen_range(0..n)] += 1;
            }
            JsonAnswer::numbers(v)
        }
        AnswerSpec::AssignDamage { total, trample } => {
            let mut v = vec![0i64; n];
            if !trample {
                for _ in 0..*total {
                    v[rng.gen_range(0..n)] += 1;
                }
                return JsonAnswer::numbers(v);
            }
            let mut left = *total as i64;
            for (i, o) in req.options.iter().enumerate() {
                let Some(l) = o.lethal else { break };
                let give = (l as i64).min(left);
                v[i] += give;
                left -= give;
            }
            if left > 0 {
                v[n - 1] += left;
            }
            JsonAnswer::numbers(v)
        }
        AnswerSpec::Split { .. } => {
            let mut v: Vec<usize> = (0..n).collect();
            v.shuffle(rng);
            let k = rng.gen_range(0..=n);
            let b = v.split_off(k);
            JsonAnswer::split(v, b)
        }
        AnswerSpec::Text { .. } => JsonAnswer::text("Lightning Bolt"),
    }
}
