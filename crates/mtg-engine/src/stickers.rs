//! Stickers (CR 123): markers on objects that modify their characteristics. Each sticker
//! is represented by a continuous effect on the object it's on; the effect follows the
//! object to public zones (CR 123.5) and has its own timestamp (CR 613.7k).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::*;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// The name of the event emitted when a player puts a sticker on an object.
pub const PLACED_EVENT: &str = "sticker placed";

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
    let record = kind.clone();
    let kind_for_event = kind.clone();
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
    g.special.stickers.placed.push((id, None, record));
    g.dirty = true;
    // "Whenever you place a sticker", "whenever you put a sticker on ~".
    g.emit(crate::events::Event::Custom {
        name: PLACED_EVENT.into(),
        player: Some(p),
        obj: Some(obj),
        amount: sticker_type(&kind_for_event) as i32,
    });
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

/// The stickers on any of `from` are also on `to` (CR 123.5a: cards melding into one
/// permanent). Their timestamps keep their relative order (CR 613.7k).
pub fn redirect(g: &mut Game, from: &[ObjectId], to: ObjectId) {
    let ids = g.stickers.clone();
    for e in g.effects.iter_mut().filter(|e| ids.contains(&e.id)) {
        if let Affected::Objects(v) = &mut e.affected {
            if v.iter().any(|o| from.contains(o)) && !v.contains(&to) {
                v.push(to);
            }
        }
    }
    g.dirty = true;
}

/// Moves the stickers on `from` to `to` (CR 123.5c: the object a melded or merged
/// permanent became that keeps its stickers).
pub fn transfer(g: &mut Game, from: ObjectId, to: ObjectId) {
    let ids = g.stickers.clone();
    for e in g.effects.iter_mut().filter(|e| ids.contains(&e.id)) {
        if let Affected::Objects(v) = &mut e.affected {
            if v.contains(&from) {
                v.retain(|o| *o != from);
                v.push(to);
            }
        }
    }
    g.dirty = true;
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

// ---------------------------------------------------------------------------
// Sticker sheets, access and ticket costs (CR 123.2, 123.3)
// ---------------------------------------------------------------------------

/// A sticker printed on a sticker sheet: its kind and ticket cost (CR 123.2, 123.3c).
#[derive(Clone, Debug)]
pub struct StickerDef {
    pub kind: StickerKind,
    pub ticket_cost: u32,
}

/// A sticker sheet: a predetermined combination of stickers (CR 123.2). Not a card; no
/// characteristics.
#[derive(Clone, Debug)]
pub struct StickerSheet {
    pub name: SmolStr,
    pub stickers: Vec<StickerDef>,
}

/// One particular sticker a player has access to: each is distinct from every other,
/// even with the same text (CR 123.3a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StickerId {
    pub player: PlayerId,
    pub sheet: usize,
    pub index: usize,
}

/// How a player's sticker sheets are chosen (CR 123.2a, 123.2b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetFormat {
    Constructed,
    Limited,
}

/// Sticker state kept by the game.
#[derive(Clone, Debug, Default)]
pub struct StickerState {
    /// Each player's sticker sheets for this game, which remain revealed (CR 123.2c).
    pub sheets: Vec<(PlayerId, Vec<StickerSheet>)>,
    /// The stickers put on objects: (continuous effect, sticker from a sheet, kind).
    pub placed: Vec<(u32, Option<StickerId>, StickerKind)>,
}

/// The type of a sticker (CR 123.1).
pub fn sticker_type(k: &StickerKind) -> StickerType {
    match k {
        StickerKind::Name(_) => StickerType::Name,
        StickerKind::Ability(_) => StickerType::Ability,
        StickerKind::PowerToughness(..) => StickerType::PowerToughness,
        StickerKind::Art => StickerType::Art,
    }
}

/// Sets up the sticker sheets `p` plays with. In constructed play they bring at least ten
/// different sheets and three are chosen at random (CR 123.2a); in limited play they
/// choose up to three (CR 123.2b). Returns the chosen sheets' names.
pub fn choose_sheets(
    g: &mut Game,
    p: PlayerId,
    sheets: Vec<StickerSheet>,
    format: SheetFormat,
) -> Result<Vec<SmolStr>, &'static str> {
    let chosen = match format {
        SheetFormat::Constructed => {
            if sheets.len() < 10 {
                return Err("constructed play requires at least ten sticker sheets");
            }
            let mut names: Vec<&SmolStr> = sheets.iter().map(|s| &s.name).collect();
            names.sort();
            if names.windows(2).any(|w| w[0] == w[1]) {
                return Err("each sticker sheet must be unique");
            }
            let mut pool = sheets;
            let mut chosen = Vec::new();
            for _ in 0..3 {
                let i = g.random_range(0, pool.len() as u32 - 1) as usize;
                chosen.push(pool.remove(i));
            }
            chosen
        }
        SheetFormat::Limited => {
            if sheets.len() > 3 {
                return Err("limited play allows up to three sticker sheets");
            }
            sheets
        }
    };
    let names = chosen.iter().map(|s| s.name.clone()).collect();
    g.special.stickers.sheets.retain(|(q, _)| *q != p);
    g.special.stickers.sheets.push((p, chosen));
    Ok(names)
}

/// The sticker sheets `p` has access to (CR 123.2c).
pub fn sheets_of(g: &Game, p: PlayerId) -> &[StickerSheet] {
    g.special
        .stickers
        .sheets
        .iter()
        .find(|(q, _)| *q == p)
        .map(|(_, v)| &v[..])
        .unwrap_or(&[])
}

pub fn sticker_def(g: &Game, id: StickerId) -> Option<&StickerDef> {
    sheets_of(g, id.player)
        .get(id.sheet)?
        .stickers
        .get(id.index)
}

/// The objects a placed sticker (by its effect) is on right now.
fn sticker_objects(g: &Game, effect: u32) -> Vec<ObjectId> {
    g.effects
        .iter()
        .filter(|e| e.id == effect)
        .flat_map(|e| match &e.affected {
            Affected::Objects(v) => v.clone(),
            _ => vec![],
        })
        .filter(|o| g.is_live(*o))
        .collect()
}

/// Whether the sticker is on an object `p` owns (CR 123.3).
fn on_object_owned_by(g: &Game, id: StickerId, p: PlayerId) -> bool {
    g.special
        .stickers
        .placed
        .iter()
        .filter(|(_, s, _)| *s == Some(id))
        .any(|(e, _, _)| sticker_objects(g, *e).iter().any(|o| g.obj(*o).owner == p))
}

/// The stickers `p` may put on an object `owner` owns: stickers on their sheets that
/// aren't on an object they own (CR 123.3), of the right kind, and whose ticket cost the
/// owner can pay unless it's `free` (CR 123.3c).
pub fn available(
    g: &Game,
    p: PlayerId,
    owner: PlayerId,
    kind: Option<StickerType>,
    max_ticket: Option<u32>,
    free: bool,
) -> Vec<StickerId> {
    let tickets = g.player(owner).counter(counters::TICKET);
    let mut out = Vec::new();
    for (si, sheet) in sheets_of(g, p).iter().enumerate() {
        for (i, st) in sheet.stickers.iter().enumerate() {
            let id = StickerId {
                player: p,
                sheet: si,
                index: i,
            };
            if kind.is_some_and(|k| k != sticker_type(&st.kind))
                || max_ticket.is_some_and(|m| st.ticket_cost > m)
                || (!free && st.ticket_cost > tickets)
                || on_object_owned_by(g, id, p)
            {
                continue;
            }
            out.push(id);
        }
    }
    out
}

fn describe(k: &StickerKind) -> String {
    match k {
        StickerKind::Name(w) => format!("name sticker \"{w}\""),
        StickerKind::Ability(v) => format!(
            "ability sticker ({})",
            v.iter()
                .map(|a| a.text.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ),
        StickerKind::PowerToughness(p, t) => format!("power and toughness sticker {p}/{t}"),
        StickerKind::Art => "art sticker".into(),
    }
}

/// CR 123.3: `p` puts one of the stickers they have access to on `obj`, choosing among
/// those not currently on objects they own; the object's owner pays its ticket cost
/// unless it's `free` (CR 123.3c). Returns true if a sticker was put on.
pub fn put_from_sheets(
    g: &mut Game,
    p: PlayerId,
    obj: ObjectId,
    kind: Option<StickerType>,
    max_ticket: Option<u32>,
    free: bool,
) -> bool {
    // CR 123.3b: only on an object the player owns.
    if !g.is_live(obj) || g.obj(obj).owner != p {
        return false;
    }
    let owner = g.obj(obj).owner;
    let options: Vec<(StickerId, StickerDef)> = available(g, p, owner, kind, max_ticket, free)
        .into_iter()
        .filter_map(|id| sticker_def(g, id).map(|d| (id, d.clone())))
        .collect();
    if options.is_empty() {
        return false;
    }
    let labels = options
        .iter()
        .map(|(_, d)| format!("{} (ticket cost {})", describe(&d.kind), d.ticket_cost))
        .collect();
    let pick = g.ask_option(p, Some(obj), "Choose a sticker", labels);
    let (id, def) = options[pick.min(options.len() - 1)].clone();
    if !free && def.ticket_cost > 0 {
        let paid = g.remove_counters_by(
            Entity::Player(owner),
            counters::TICKET,
            def.ticket_cost,
            Some(owner),
        );
        if paid < def.ticket_cost {
            return false;
        }
    }
    if !put_sticker(g, p, obj, def.kind) {
        return false;
    }
    if let Some(last) = g.special.stickers.placed.last_mut() {
        last.1 = Some(id);
    }
    true
}

/// CR 123.3d: moves a sticker (by its effect) that's on an object to another object; its
/// ticket cost isn't paid again. Returns true if it moved.
pub fn move_sticker(g: &mut Game, effect: u32, to: ObjectId) -> bool {
    let Some(i) = g
        .special
        .stickers
        .placed
        .iter()
        .position(|(e, _, _)| *e == effect)
    else {
        return false;
    };
    let (_, id, kind) = g.special.stickers.placed[i].clone();
    let p = g
        .effects
        .iter()
        .find(|e| e.id == effect)
        .map(|e| e.controller)
        .unwrap_or(g.obj(to).owner);
    if !g.is_live(to) || g.obj(to).owner != p {
        return false;
    }
    g.effects.retain(|e| e.id != effect);
    g.stickers.retain(|e| *e != effect);
    g.special.stickers.placed.remove(i);
    if !put_sticker(g, p, to, kind) {
        return false;
    }
    if let Some(last) = g.special.stickers.placed.last_mut() {
        last.1 = id;
    }
    true
}

/// The stickers (kinds) on `obj`, in timestamp order.
pub fn kinds_on(g: &Game, obj: ObjectId) -> Vec<StickerKind> {
    stickers_on(g, obj)
        .iter()
        .filter_map(|e| {
            g.special
                .stickers
                .placed
                .iter()
                .find(|(id, _, _)| *id == e.id)
                .map(|(_, _, k)| k.clone())
        })
        .collect()
}

/// Whether `obj` has a sticker (of the given type) on it (CR 123.4).
pub fn has_sticker(g: &Game, obj: ObjectId, kind: Option<StickerType>) -> bool {
    match kind {
        None => is_stickered(g, obj),
        Some(k) => kinds_on(g, obj).iter().any(|s| sticker_type(s) == k),
    }
}

/// The words of the name stickers on `obj`, in timestamp order.
pub fn name_sticker_words(g: &Game, obj: ObjectId) -> Vec<SmolStr> {
    kinds_on(g, obj)
        .into_iter()
        .filter_map(|k| match k {
            StickerKind::Name(w) => Some(w),
            _ => None,
        })
        .collect()
}

/// The number of times a letter appears on a name sticker; a lowercase letter and its
/// uppercase equivalent are the same letter (CR 123.6d).
pub fn letter_count(word: &str, letter: char) -> u32 {
    let l = letter.to_ascii_lowercase();
    word.chars().filter(|c| c.to_ascii_lowercase() == l).count() as u32
}

/// The number of different vowels (A, E, I, O, U and Y) on a name sticker, in either case
/// (CR 123.6e).
pub fn unique_vowels(word: &str) -> u32 {
    "aeiouy"
        .chars()
        .filter(|v| letter_count(word, *v) > 0)
        .count() as u32
}

/// The abilities of the ability stickers on `obj` (CR 123.7a): the abilities those
/// stickers grant, even if the object doesn't currently have them because of another
/// effect.
pub fn ability_sticker_abilities(g: &Game, obj: ObjectId) -> Vec<Ability> {
    kinds_on(g, obj)
        .into_iter()
        .flat_map(|k| match k {
            StickerKind::Ability(v) => v,
            _ => vec![],
        })
        .collect()
}

/// The power and toughness printed on the power and toughness stickers on `obj`
/// (CR 123.8a): other stickers have none.
pub fn sticker_power_toughness(g: &Game, obj: ObjectId) -> Vec<(i32, i32)> {
    kinds_on(g, obj)
        .into_iter()
        .filter_map(|k| match k {
            StickerKind::PowerToughness(p, t) => Some((p, t)),
            _ => None,
        })
        .collect()
}

/// Values referring to stickers, for custom values: "sticker unique vowels" (of the
/// latest name sticker on the source, CR 123.6e), "sticker letter:o" (o's in name
/// stickers on the source, CR 123.6d), "sticker power"/"sticker toughness" (total printed
/// on stickers on permanents you control, CR 123.8a).
pub fn sticker_value(g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
    let src = ctx.source;
    if name == "sticker unique vowels" {
        let words = src.map(|s| name_sticker_words(g, s)).unwrap_or_default();
        return Some(words.last().map_or(0, |w| unique_vowels(w)) as i64);
    }
    if let Some(l) = name.strip_prefix("sticker letter:") {
        let c = l.chars().next()?;
        let words = src.map(|s| name_sticker_words(g, s)).unwrap_or_default();
        return Some(words.iter().map(|w| letter_count(w, c)).sum::<u32>() as i64);
    }
    let pt = match name {
        "sticker power" => 0,
        "sticker toughness" => 1,
        _ => return None,
    };
    let total: i32 = g
        .battlefield
        .iter()
        .filter(|o| g.obj(**o).controller == ctx.controller)
        .flat_map(|o| sticker_power_toughness(g, *o))
        .map(|(p, t)| if pt == 0 { p } else { t })
        .sum();
    Some(total as i64)
}
