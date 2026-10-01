//! Structural fingerprints of abilities, and the structure log.
//!
//! Cards whose abilities compile to the same structure — differing only in numbers, names,
//! mana amounts, creature types and the like — run the same engine code, so a test that
//! exercises one of them exercises all of them. [`fingerprint`] computes that structure:
//! the ability's AST ([`crate::ability`]) with literal parameters abstracted and every
//! semantic choice kept.
//!
//! The fingerprint is derived from the AST's `Serialize` implementations through a
//! dedicated serializer, so every variant and field of every AST type is part of it
//! automatically (a new variant can't be forgotten); the abstractions below are keyed by
//! type, variant and field names, and `tests` check that each of them still applies.
//!
//! Abstracted (replaced by a placeholder):
//! * numbers, keeping only their sign: `#` (positive), `0`, `-#` — so "up to one target"
//!   (minimum 0) differs from "target" (minimum 1), and +N/+N from -N/-N;
//! * mana costs: only the kinds of special symbols ({X}, hybrid, Phyrexian, snow, ...) are
//!   kept, not the amounts or colors;
//! * colors (and colored mana types; colorless is kept);
//! * the card's own name, names in "named ..." filters, token names, chosen words and
//!   display text;
//! * creature types (other subtypes, like Aura, Equipment or basic land types, are kept);
//! * counter kinds with no rules of their own ("charge", "quest", ...).
//!
//! Kept: every enum variant (effects, filters and their qualifiers, zones, players,
//! targets, durations, conditions, card types, keywords, triggers, replacement shape,
//! modes) and every boolean flag.
//!
//! # Structure log
//!
//! When the environment variable `MTG_STRUCTURE_LOG` names a directory, the engine records
//! the fingerprint of every ability that is actually exercised in a game — a spell or
//! ability that resolves, a mana ability that's activated, a static ability whose
//! continuous effect applies to something or whose other effect is in force, a replacement
//! effect that's applied, and a keyword the engine consults on an object that has it — to
//! a file of its own per process in that directory. `mtg-tools structure-coverage` reads
//! them. With the variable unset, each hook costs one relaxed atomic load.

use crate::ability::{AbilityDef, AbilityKind};
use serde::ser::{self, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

/// The environment variable naming the structure log directory.
pub const LOG_ENV: &str = "MTG_STRUCTURE_LOG";

/// The structural fingerprint of an ability (see the module docs). `self_names` are the
/// names of the card the ability is on (its full name and face names), which are
/// abstracted wherever they appear.
pub fn fingerprint(a: &AbilityDef, self_names: &[&str]) -> String {
    let mut fp = Fp {
        out: String::with_capacity(256),
        names: self_names,
        ctx: Vec::new(),
        structs: Vec::new(),
        mana: Vec::new(),
    };
    // The serializer never fails.
    let _ = a.serialize(&mut fp);
    fp.out
}

/// A stable 64-bit hash of a fingerprint (FNV-1a), as written to the structure log.
pub fn hash(fingerprint: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in fingerprint.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The hash of an ability's fingerprint.
pub fn ability_hash(a: &AbilityDef, self_names: &[&str]) -> u64 {
    hash(&fingerprint(a, self_names))
}

/// A short label for the kind of an ability ("spell", "activated", ...).
pub fn kind_label(a: &AbilityDef) -> &'static str {
    match &a.kind {
        AbilityKind::Spell(_) => "spell",
        AbilityKind::Activated(x) if x.is_mana_ability => "mana",
        AbilityKind::Activated(_) => "activated",
        AbilityKind::Triggered(_) => "triggered",
        AbilityKind::Static(_) => "static",
        AbilityKind::Keyword(_) => "keyword",
        AbilityKind::Unsupported(_) => "unsupported",
    }
}

// ---------------------------------------------------------------------------
// Abstraction rules
// ---------------------------------------------------------------------------

/// Fields that aren't part of the structure: identity and display text.
fn skipped_field(container: &str, field: &str) -> bool {
    matches!(
        (container, field),
        ("AbilityDef", "uid") | ("AbilityDef", "text") | ("Mode", "text") | ("TargetSpec", "text")
    )
}

/// Counter kinds that have rules of their own (CR 122) and so stay distinct.
fn rules_counter(s: &str) -> bool {
    use crate::types::counters::*;
    [
        PLUS1, MINUS1, LOYALTY, DEFENSE, POISON, ENERGY, EXPERIENCE, RAD, TICKET, LORE, TIME, FADE,
        AGE, SHIELD, STUN, FINALITY, OIL, LEVEL,
    ]
    .contains(&s)
        || crate::layers::KEYWORD_COUNTERS.contains(&s)
}

/// Whether strings in this context are counter kinds.
fn counter_context(container: &str, field: &str) -> bool {
    matches!(field, "kind" | "counter" | "counters" | "with_counters")
        || matches!(
            (container, field),
            ("Value", "CountersOn")
                | ("Value", "PlayerCounters")
                | ("Filter", "HasCounter")
                | ("ReplacementAction", "EnterWithCounters")
                | ("PlayerFilter", "Counters")
        )
}

/// The placeholder for a string in this context, or `None` to keep it.
fn abstract_str(names: &[&str], ctx: Option<(&str, &str)>, s: &str) -> Option<&'static str> {
    if names
        .iter()
        .any(|n| !n.is_empty() && n.eq_ignore_ascii_case(s))
    {
        return Some("~");
    }
    let (container, field) = ctx.unwrap_or(("", ""));
    match (container, field) {
        ("Filter", "Named")
        | ("Sel", "ExiledWithCardsNamed")
        | ("TokenSpec", "name")
        | ("TokenSpec", "scryfall_name")
        | ("Modification", "SetName")
        | ("ChoiceKind", "CardNameFiltered") => return Some("<name>"),
        ("Filter", "NameOriginallyPrintedIn") => return Some("<set>"),
        ("ChooseOne", "options")
        | ("ChoiceKind", "OneOf")
        | ("ChoiceKind", "Word")
        | ("Condition", "ChosenWord")
        | ("NameSticker", "word")
        | ("ChangeText", "from")
        | ("ChangeText", "to") => return Some("<word>"),
        (_, "text") => return Some("<text>"),
        _ => {}
    }
    if counter_context(container, field) {
        if rules_counter(s) {
            return None;
        }
        if crate::layers::parse_pt_counter(s).is_some() {
            return Some("<pt-counter>");
        }
        return Some("<counter>");
    }
    if crate::types::is_creature_type(s) {
        return Some("<creature-type>");
    }
    None
}

/// Enum types whose values are colors.
fn color_variant(name: &str, variant: &str) -> bool {
    name == "Color" || (name == "ManaType" && variant != "C")
}

/// Mana symbols that only say how much mana of which color: abstracted away in costs.
fn plain_mana_symbol(variant: &str) -> bool {
    matches!(variant, "Generic" | "Colored" | "Colorless")
}

// ---------------------------------------------------------------------------
// The fingerprint serializer
// ---------------------------------------------------------------------------

struct Fp<'a> {
    out: String,
    names: &'a [&'a str],
    /// (container, field or variant) of the value being serialized.
    ctx: Vec<(&'static str, &'static str)>,
    /// Open structs: (name, start of a mana cost's output).
    structs: Vec<(&'static str, Option<usize>)>,
    /// Special symbols of the mana costs being serialized.
    mana: Vec<BTreeSet<&'static str>>,
}

#[derive(Debug)]
struct FpError;

impl std::fmt::Display for FpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fingerprint error")
    }
}

impl std::error::Error for FpError {}

impl ser::Error for FpError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        FpError
    }
}

type R = Result<(), FpError>;

impl Fp<'_> {
    fn num(&mut self, sign: i8) -> R {
        self.out.push_str(match sign {
            0 => "0",
            s if s < 0 => "-#",
            _ => "#",
        });
        Ok(())
    }
    fn variant(&mut self, name: &'static str, variant: &'static str) {
        if name == "ManaSymbol" && !plain_mana_symbol(variant) {
            if let Some(set) = self.mana.last_mut() {
                set.insert(variant);
            }
        }
        if color_variant(name, variant) {
            self.out.push_str("color");
        } else {
            self.out.push_str(variant);
        }
    }
}

macro_rules! signed {
    ($($f:ident: $t:ty),*) => {$(
        fn $f(self, v: $t) -> R {
            self.num(if v > 0 as $t { 1 } else if v < 0 as $t { -1 } else { 0 })
        }
    )*};
}

macro_rules! unsigned {
    ($($f:ident: $t:ty),*) => {$(
        fn $f(self, v: $t) -> R {
            self.num(if v > 0 { 1 } else { 0 })
        }
    )*};
}

impl<'a, 'b> ser::Serializer for &'a mut Fp<'b> {
    type Ok = ();
    type Error = FpError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    signed!(serialize_i8: i8, serialize_i16: i16, serialize_i32: i32, serialize_i64: i64,
        serialize_f32: f32, serialize_f64: f64);
    unsigned!(serialize_u8: u8, serialize_u16: u16, serialize_u32: u32, serialize_u64: u64);

    fn serialize_bool(self, v: bool) -> R {
        self.out.push(if v { 'T' } else { 'F' });
        Ok(())
    }
    fn serialize_char(self, v: char) -> R {
        self.serialize_str(v.encode_utf8(&mut [0u8; 4]))
    }
    fn serialize_str(self, v: &str) -> R {
        match abstract_str(self.names, self.ctx.last().copied(), v) {
            Some(p) => self.out.push_str(p),
            None => {
                let _ = write!(self.out, "{v:?}");
            }
        }
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> R {
        self.out.push_str("<bytes>");
        Ok(())
    }
    fn serialize_none(self) -> R {
        self.out.push('_');
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> R {
        value.serialize(self)
    }
    fn serialize_unit(self) -> R {
        self.out.push_str("()");
        Ok(())
    }
    fn serialize_unit_struct(self, name: &'static str) -> R {
        self.out.push_str(name);
        Ok(())
    }
    fn serialize_unit_variant(self, name: &'static str, _i: u32, variant: &'static str) -> R {
        self.variant(name, variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, name: &'static str, value: &T) -> R {
        if name == "ColorSet" {
            self.out.push_str("colors");
            return Ok(());
        }
        self.out.push_str(name);
        self.out.push('(');
        self.ctx.push((name, "0"));
        value.serialize(&mut *self)?;
        self.ctx.pop();
        self.out.push(')');
        Ok(())
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        value: &T,
    ) -> R {
        self.variant(name, variant);
        if color_variant(name, variant) {
            return Ok(());
        }
        self.out.push('(');
        self.ctx.push((name, variant));
        value.serialize(&mut *self)?;
        self.ctx.pop();
        self.out.push(')');
        Ok(())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self, FpError> {
        self.out.push('[');
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self, FpError> {
        self.out.push('(');
        Ok(self)
    }
    fn serialize_tuple_struct(self, name: &'static str, _len: usize) -> Result<Self, FpError> {
        self.out.push_str(name);
        self.out.push('(');
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self, FpError> {
        self.variant(name, variant);
        self.out.push('(');
        self.ctx.push((name, variant));
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self, FpError> {
        self.out.push('{');
        Ok(self)
    }
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self, FpError> {
        if name == "ManaCost" {
            self.structs.push((name, Some(self.out.len())));
            self.mana.push(BTreeSet::new());
        } else {
            self.structs.push((name, None));
        }
        self.out.push('{');
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self, FpError> {
        self.variant(name, variant);
        self.out.push('{');
        self.ctx.push((name, variant));
        self.structs.push((variant, None));
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> R {
        value.serialize(&mut **self)?;
        self.out.push(',');
        Ok(())
    }
    fn end(self) -> R {
        self.out.push(']');
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> R {
        value.serialize(&mut **self)?;
        self.out.push(',');
        Ok(())
    }
    fn end(self) -> R {
        self.out.push(')');
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> R {
        value.serialize(&mut **self)?;
        self.out.push(',');
        Ok(())
    }
    fn end(self) -> R {
        self.out.push(')');
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> R {
        value.serialize(&mut **self)?;
        self.out.push(',');
        Ok(())
    }
    fn end(self) -> R {
        self.ctx.pop();
        self.out.push(')');
        Ok(())
    }
}

impl ser::SerializeMap for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> R {
        key.serialize(&mut **self)?;
        self.out.push(':');
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> R {
        value.serialize(&mut **self)?;
        self.out.push(',');
        Ok(())
    }
    fn end(self) -> R {
        self.out.push('}');
        Ok(())
    }
}

impl Fp<'_> {
    fn field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> R {
        let container = self.structs.last().map_or("", |s| s.0);
        if skipped_field(container, key) {
            return Ok(());
        }
        self.out.push_str(key);
        self.out.push(':');
        self.ctx.push((container, key));
        value.serialize(&mut *self)?;
        self.ctx.pop();
        self.out.push(',');
        Ok(())
    }
    fn end_struct(&mut self) {
        let Some((_, mana_start)) = self.structs.pop() else {
            self.out.push('}');
            return;
        };
        match mana_start {
            Some(start) => {
                // A mana cost: only its special symbols.
                let set = self.mana.pop().unwrap_or_default();
                self.out.truncate(start);
                self.out.push_str("Mana");
                if !set.is_empty() {
                    let v: Vec<&str> = set.into_iter().collect();
                    let _ = write!(self.out, "<{}>", v.join(","));
                }
            }
            None => self.out.push('}'),
        }
    }
}

impl ser::SerializeStruct for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> R {
        self.field(key, value)
    }
    fn end(self) -> R {
        self.end_struct();
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Fp<'_> {
    type Ok = ();
    type Error = FpError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> R {
        self.field(key, value)
    }
    fn end(self) -> R {
        self.end_struct();
        self.ctx.pop();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The structure log
// ---------------------------------------------------------------------------

/// 0: not yet checked; 1: off; 2: on.
static STATE: AtomicU8 = AtomicU8::new(0);

struct Log {
    file: std::fs::File,
    /// (ability uid, how it was exercised) pairs already written.
    seen: HashSet<(u64, &'static str)>,
    /// The keyword ability each ability a keyword stands for comes from (by uid).
    derived_from: HashMap<u64, (std::sync::Arc<AbilityDef>, String)>,
}

fn log() -> &'static Option<Mutex<Log>> {
    static LOG: OnceLock<Option<Mutex<Log>>> = OnceLock::new();
    LOG.get_or_init(|| {
        let dir = std::env::var_os(LOG_ENV)?;
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).ok()?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = dir.join(format!("{}-{nanos}.log", std::process::id()));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()?;
        Some(Mutex::new(Log {
            file,
            seen: HashSet::new(),
            derived_from: HashMap::new(),
        }))
    })
}

/// Whether the structure log is on (`MTG_STRUCTURE_LOG` is set).
#[inline]
pub fn enabled() -> bool {
    match STATE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = log().is_some();
            STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            on
        }
    }
}

/// Records that `a`, an ability of an object named `name`, was exercised `how`
/// ("resolved", "static", "replacement", "keyword", ...). No-op unless the log is on.
#[inline]
pub fn record(a: &AbilityDef, name: &str, how: &'static str) {
    if enabled() {
        record_slow(a, name, how);
    }
}

thread_local! {
    static SEEN: std::cell::RefCell<HashSet<(u64, &'static str)>> =
        std::cell::RefCell::new(HashSet::new());
}

fn record_slow(a: &AbilityDef, name: &str, how: &'static str) {
    if matches!(a.kind, AbilityKind::Unsupported(_)) {
        return;
    }
    let key = (a.uid, how);
    if SEEN.with(|s| !s.borrow_mut().insert(key)) {
        return;
    }
    let Some(log) = log() else {
        return;
    };
    let Ok(mut log) = log.lock() else {
        return;
    };
    if !log.seen.insert(key) {
        return;
    }
    write_line(&mut log, a, name, how);
    // An ability a keyword stands for exercises the keyword too.
    if let Some((kw, kw_name)) = log.derived_from.get(&a.uid).cloned() {
        if log.seen.insert((kw.uid, "keyword")) {
            write_line(&mut log, &kw, &kw_name, "keyword");
        }
    }
}

fn write_line(log: &mut Log, a: &AbilityDef, name: &str, how: &str) {
    use std::io::Write;
    let h = ability_hash(a, &[name]);
    let name = name.replace(['\t', '\n'], " ");
    let _ = writeln!(log.file, "{h:016x}\t{how}\t{}\t{name}", kind_label(a));
}

/// Notes that `derived` is one of the abilities the keyword ability `keyword` (of an object
/// named `name`) stands for, so that exercising it exercises the keyword. No-op unless the
/// log is on.
#[inline]
pub fn note_derived(derived: &AbilityDef, keyword: &std::sync::Arc<AbilityDef>, name: &str) {
    if !enabled() {
        return;
    }
    if let Some(Ok(mut log)) = log().as_ref().map(|l| l.lock()) {
        log.derived_from
            .entry(derived.uid)
            .or_insert_with(|| (keyword.clone(), name.to_string()));
    }
}

/// Reads every structure log in `dir`: fingerprint hash → (how, card name) examples.
pub fn read_logs(dir: &std::path::Path) -> HashMap<u64, Vec<(String, String)>> {
    let mut out: HashMap<u64, Vec<(String, String)>> = HashMap::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in rd.flatten() {
        let Ok(text) = std::fs::read_to_string(e.path()) else {
            continue;
        };
        for line in text.lines() {
            let mut parts = line.split('\t');
            let (Some(h), Some(how), Some(_kind), Some(name)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Ok(h) = u64::from_str_radix(h, 16) else {
                continue;
            };
            let v = out.entry(h).or_default();
            if v.len() < 8 && !v.iter().any(|(a, b)| a == how && b == name) {
                v.push((how.to_string(), name.to_string()));
            }
        }
    }
    out
}

/// Records the keyword abilities of a spell that were used to cast it: a keyword's
/// alternative cost or permission (flashback, morph, ...) or an optional additional cost
/// that was paid (kicker, ...), convoke and delve. No-op unless the log is on.
pub fn record_cast(g: &crate::game::Game, spell: crate::types::ObjectId) {
    if !enabled() {
        return;
    }
    use crate::keywords::KeywordKind;
    use crate::object::CastMethod;
    let o = g.obj(spell);
    let Some(si) = o.stack.as_deref() else {
        return;
    };
    let cast = &si.cast;
    let norm = |s: &str| s.to_lowercase().replace([' ', '-', '_'], "");
    for a in &o.chars.abilities {
        let AbilityKind::Keyword(kw) = &a.kind else {
            continue;
        };
        let name = norm(&format!("{:?}", kw.kind));
        let used = matches!(cast.method,
            CastMethod::Keyword(k) | CastMethod::FaceDown(k) if k == kw.kind)
            || cast.paid.iter().any(|p| norm(p).starts_with(&name))
            || (kw.kind == KeywordKind::Convoke && !cast.convoked.is_empty())
            || (kw.kind == KeywordKind::Delve && !cast.delved.is_empty());
        if used {
            record(a, &o.chars.name, "keyword");
        }
    }
}

/// Records that the replacement effect of static ability `uid` of `src` was applied.
/// No-op unless the log is on.
pub fn record_replacement(g: &crate::game::Game, src: crate::types::ObjectId, uid: u64) {
    if !enabled() {
        return;
    }
    let o = g.obj(src);
    let found = g
        .statics
        .replacements
        .iter()
        .find(|(s, _, _, a, _)| *s == src && a.uid == uid)
        .map(|(_, _, _, a, _)| a)
        .or_else(|| o.chars.abilities.iter().find(|a| a.uid == uid))
        .or_else(|| {
            // An entering object's other face (CR 614.12).
            o.card
                .iter()
                .flat_map(|c| c.faces.iter())
                .flat_map(|f| f.chars.abilities.iter())
                .find(|a| a.uid == uid)
        });
    if let Some(a) = found {
        record(a, &o.chars.name, "replacement");
    }
}
