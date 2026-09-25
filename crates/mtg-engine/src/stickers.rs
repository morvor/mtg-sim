//! Stickers (CR 123): markers on objects that modify their characteristics. Each sticker
//! is represented by a continuous effect on the object it's on; the effect follows the
//! object to public zones (CR 123.5) and has its own timestamp (CR 613.7k).

use crate::ability::*;
use crate::game::*;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// The kinds of stickers (CR 123.1).
#[derive(Clone, Debug)]
pub enum StickerKind {
    /// A name sticker: one or more words (CR 123.6).
    Name(SmolStr),
    /// An ability sticker (CR 123.7).
    Ability(Vec<Ability>),
    /// A power and toughness sticker (CR 123.8).
    PowerToughness(i32, i32),
    /// An art sticker: only a marker (CR 123.9).
    Art,
}

/// A blank line in a name ("Wolf in ________ Clothing") isn't a word (CR 123.6a).
fn is_blank(w: &str) -> bool {
    w.chars().all(|c| c == '_')
}

/// The number of words in a name (CR 123.6a).
pub fn word_count(name: &str) -> usize {
    name.split_whitespace().filter(|w| !is_blank(w)).count()
}

/// Adds `word` to `name` after `position` of its words, or at the end if it has fewer
/// words; a nameless object's name becomes the word (CR 123.6b, 123.6c).
pub fn add_name_word(name: &str, word: &str, position: usize) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut count = 0;
    let mut inserted = false;
    if position == 0 {
        out.push(word);
        inserted = true;
    }
    for t in name.split_whitespace() {
        out.push(t);
        if !is_blank(t) {
            count += 1;
            if !inserted && count == position {
                out.push(word);
                inserted = true;
            }
        }
    }
    if !inserted {
        out.push(word);
    }
    out.join(" ")
}

/// CR 123.3: `p` puts a sticker on `obj`. A player can't put a sticker on an object they
/// don't own (CR 123.3b). The sticker receives a timestamp as it's put on the object
/// (CR 613.7k). Returns true if it was put on.
pub fn put_sticker(g: &mut Game, p: PlayerId, obj: ObjectId, kind: StickerKind) -> bool {
    if !g.is_live(obj) || g.obj(obj).owner != p {
        return false;
    }
    let mods = match kind {
        StickerKind::Name(word) => {
            // CR 123.6b: the object's controller chooses where the word goes.
            let name = g.obj(obj).chars.name.to_string();
            let n = word_count(&name);
            let position = if n == 0 {
                0
            } else {
                let options: Vec<String> =
                    (0..=n).map(|i| add_name_word(&name, &word, i)).collect();
                let ctl = g.obj(obj).controller;
                g.ask_option(
                    ctl,
                    Some(obj),
                    "Where does the name sticker's word go?",
                    options,
                )
            };
            let new_name = add_name_word(&name, &word, position);
            g.log(|_| format!("{obj} is now named {new_name}"));
            vec![Modification::NameSticker {
                word,
                position: position as u32,
            }]
        }
        StickerKind::Ability(abilities) => abilities
            .into_iter()
            .map(Modification::AddAbility)
            .collect(),
        StickerKind::PowerToughness(pw, t) => {
            vec![Modification::SetPT(Some(Value::c(pw)), Some(Value::c(t)))]
        }
        StickerKind::Art => vec![],
    };
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: None,
        controller: p,
        timestamp: ts,
        duration: Duration::Permanent,
        affected: Affected::Objects(vec![obj]),
        mods,
        layer1: None,
        created_turn: g.turn.number,
    });
    g.stickers.push(id);
    g.dirty = true;
    true
}

/// The stickers on `obj`, in timestamp order.
pub fn stickers_on(g: &Game, obj: ObjectId) -> Vec<&ContinuousEffect> {
    let mut v: Vec<&ContinuousEffect> = g
        .effects
        .iter()
        .filter(|e| g.stickers.contains(&e.id))
        .filter(|e| matches!(&e.affected, Affected::Objects(v) if v.contains(&obj)))
        .collect();
    v.sort_by_key(|e| effect_timestamp(g, e));
    v
}

/// CR 123.4: an object is "stickered" if it currently has any sticker on it.
pub fn is_stickered(g: &Game, obj: ObjectId) -> bool {
    !stickers_on(g, obj).is_empty()
}

/// The timestamp a continuous effect applies with. A sticker receives a new timestamp
/// immediately after the object it's on whenever that object receives a new timestamp,
/// keeping the relative order of the stickers on it (CR 613.7k).
pub fn effect_timestamp(g: &Game, e: &ContinuousEffect) -> (Timestamp, Timestamp) {
    if !g.stickers.contains(&e.id) {
        return (e.timestamp, 0);
    }
    let obj_ts = match &e.affected {
        Affected::Objects(v) => v
            .iter()
            .rev()
            .find(|o| g.is_live(**o))
            .map(|o| g.obj(*o).timestamp)
            .unwrap_or(0),
        _ => 0,
    };
    if e.timestamp > obj_ts {
        (e.timestamp, 0)
    } else {
        (obj_ts, e.timestamp)
    }
}

/// CR 123.5: stickers stay on an object that moves to a public zone and apply to the new
/// object it becomes; they aren't retained as it moves to a hidden zone.
pub fn follow(g: &mut Game, old: ObjectId, new: ObjectId, to: Zone) {
    if g.stickers.is_empty() || matches!(to, Zone::Library(_) | Zone::Hand(_)) {
        return;
    }
    let ids = g.stickers.clone();
    for e in g.effects.iter_mut().filter(|e| ids.contains(&e.id)) {
        if let Affected::Objects(v) = &mut e.affected {
            if v.contains(&old) {
                v.push(new);
            }
        }
    }
}

/// CR 123.5b, 613.7k: `obj`, with stickers on it, becomes part of the merged permanent
/// `merged`: its stickers are on the merged permanent, and each receives a new timestamp
/// at that time, keeping their relative order.
pub fn merge_into(g: &mut Game, obj: ObjectId, merged: ObjectId) {
    let ids: Vec<u32> = stickers_on(g, obj).iter().map(|e| e.id).collect();
    for id in ids {
        let ts = g.new_timestamp();
        if let Some(e) = g.effects.iter_mut().find(|e| e.id == id) {
            e.timestamp = ts;
            if let Affected::Objects(v) = &mut e.affected {
                v.retain(|o| *o != obj);
                v.push(merged);
            }
        }
    }
    g.dirty = true;
}
