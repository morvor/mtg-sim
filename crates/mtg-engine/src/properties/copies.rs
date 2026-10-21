//! A copy has the original's copiable values (CR 707.2–707.4, 707.8–707.10).
//!
//! The static check ([`check_card`]) runs over every fully supported card ([`pool`]): for
//! a permanent card, it puts the card onto the battlefield (each face that can be up: the
//! back face of a double-faced card, a flip card flipped) and copies it in each way the
//! engine knows — a token that's a copy of it (CR 707.2, 111.10, 707.8a), an object that
//! becomes a copy of it (CR 707.4), a permanent that enters as a copy of it (CR 707.5) and,
//! for permanents whose face can be cast, a copy of the spell that resolves as a token
//! (CR 707.10f) — and compares each copy's copiable values with the card's: name, mana
//! cost, color indicator, colors, card types, subtypes and supertypes, power, toughness,
//! loyalty and defense, and the abilities by their structural fingerprints
//! ([`crate::structure`]; which abilities share a link, CR 607.5, 707.7, is compared as a
//! partition, since a copy's links are its own). For an instant or sorcery it casts the card
//! (without paying its mana cost, on a board with something to target) and compares a copy
//! of the spell with the spell: the same characteristics and the same choices — modes,
//! targets, X, additional and alternative costs, the objects that paid its costs (CR
//! 707.10).
//!
//! Allowed differences are only those the rules call for: a token copy is a token, and a
//! copy effect's own exceptions (CR 707.9) — the "enters as a copy" cards used here have
//! none.
//!
//! The dynamic check ([`check_dynamic`]) plays the same script with the card and with a
//! token copy of it (it enters, each activated ability is activated, it attacks, a turn
//! passes) and compares the outcomes.

use crate::ability::*;
use crate::card::{CardDb, CardDef, Layout};
use crate::object::*;
use crate::testing::{TestGame, P0, P1};
use crate::types::*;
use std::sync::{Arc, OnceLock};

/// A card name with what the copies of it got wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub card: String,
    /// How it was copied: "token copy", "becomes a copy", "enters as a copy", "spell copy",
    /// "permanent spell copy", "original" (the uncopied card's own copiable values).
    pub path: &'static str,
    /// Which face was up ("front", "back", "flipped").
    pub face: &'static str,
    pub field: String,
    pub expected: String,
    pub got: String,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}, {} face up): {}: expected {}, got {}",
            self.card, self.path, self.face, self.field, self.expected, self.got
        )
    }
}

/// What checking one card did.
#[derive(Clone, Debug, Default)]
pub struct CardResult {
    /// Copies compared.
    pub copies: usize,
    pub mismatches: Vec<Mismatch>,
    /// Copies that couldn't be made in the fixture (a spell that can't be cast there),
    /// with the reason.
    pub skipped: Vec<String>,
    /// Fields not compared for the copies of this card (see [`check_card`]).
    exempt: Vec<&'static str>,
}

/// The names of every real (paper) playable card, one per name, sorted.
pub fn pool_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for c in mtg_data::cards().iter() {
            if !c.is_playable_card() {
                continue;
            }
            if !c.games.is_empty() && c.games.iter().all(|g| g != "paper") {
                continue;
            }
            if seen.insert(c.name.to_lowercase()) {
                names.push(c.name.clone());
            }
        }
        names.sort();
        names
    })
}

/// Every fully supported card: real (paper) playable cards, one per name, sorted by name
/// (the pool `mtg-sim --every-card` uses). Compiling them all takes a while (about half a
/// minute in a release build, several minutes in a debug build).
pub fn pool() -> &'static [Arc<CardDef>] {
    static POOL: OnceLock<Vec<Arc<CardDef>>> = OnceLock::new();
    POOL.get_or_init(|| supported(pool_names()))
}

/// The fully supported cards among `names`, compiled on a few threads, in order.
pub fn supported(names: &[String]) -> Vec<Arc<CardDef>> {
    compile_all(names)
        .into_iter()
        .filter(|d| d.is_fully_supported())
        .collect()
}

/// Compiles the named cards on a few threads (in their order).
fn compile_all(names: &[String]) -> Vec<Arc<CardDef>> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        .clamp(1, 3);
    let chunk = names.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        let handles: Vec<_> = names
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    part.iter()
                        .filter_map(|n| CardDb::global().get(n))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap_or_default())
            .collect()
    })
}

/// The copiable values compared, as text.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Values(Vec<(&'static str, String)>);

fn values(c: &Characteristics, names: &[&str]) -> Values {
    let mut abilities = Vec::new();
    let mut links: Vec<u16> = Vec::new();
    for a in &c.abilities {
        // A copy's linked abilities get links of their own (CR 607.5, 707.7): the
        // fingerprint is taken without the link, and the abilities sharing links are
        // compared as a partition.
        let unlinked = AbilityDef {
            uid: 0,
            kind: a.kind.clone(),
            text: String::new(),
            link: 0,
        };
        let class = links
            .iter()
            .position(|l| *l == a.link)
            .unwrap_or(links.len());
        links.push(a.link);
        abilities.push(format!(
            "[link group {class}] {}",
            crate::structure::fingerprint(&unlinked, names)
        ));
    }
    Values(vec![
        ("name", c.name.to_string()),
        (
            "mana cost",
            c.mana_cost
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "none".into()),
        ),
        ("color indicator", format!("{:?}", c.color_indicator)),
        ("colors", format!("{:?}", c.colors)),
        ("supertypes", format!("{:?}", c.supertypes)),
        ("card types", format!("{:?}", c.card_types)),
        ("subtypes", format!("{:?}", c.subtypes)),
        ("every creature type", c.all_creature_types.to_string()),
        ("power", format!("{:?}", c.power)),
        ("toughness", format!("{:?}", c.toughness)),
        ("loyalty", format!("{:?}", c.loyalty)),
        ("defense", format!("{:?}", c.defense)),
        ("abilities", abilities.join("\n")),
        (
            "printed card",
            c.printed
                .as_ref()
                .map(|p| p.0.name.to_string())
                .unwrap_or_default(),
        ),
    ])
}

/// The card's printed copiable values with `face` up (the card they come from is part of
/// them, CR 709.5b, 715.2b).
fn printed_values(def: &Arc<CardDef>, face: FaceState, names: &[&str]) -> Values {
    let mut c = def.characteristics(face);
    c.printed = Some(crate::card::PrintedCard(def.clone()));
    values(&c, names)
}

fn compare(
    out: &mut CardResult,
    card: &str,
    path: &'static str,
    face: &'static str,
    expected: &Values,
    got: &Values,
) {
    out.copies += 1;
    for ((field, e), (_, g)) in expected.0.iter().zip(&got.0) {
        if e != g && !out.exempt.contains(field) {
            out.mismatches.push(Mismatch {
                card: card.to_string(),
                path,
                face,
                field: field.to_string(),
                expected: e.clone(),
                got: g.clone(),
            });
        }
    }
}

fn names_of(def: &CardDef) -> Vec<&str> {
    let mut v = vec![def.name.as_str()];
    for f in &def.faces {
        if !v.contains(&f.chars.name.as_str()) {
            v.push(f.chars.name.as_str());
        }
    }
    v
}

/// Whether the card is one the static check copies as a permanent.
fn is_permanent_card(def: &CardDef) -> bool {
    def.faces.iter().any(|f| f.chars.is_permanent_type())
        && !crate::variants::is_nontraditional(def)
}

fn is_instant_or_sorcery(def: &CardDef) -> bool {
    let f = &def.front().chars;
    (f.is(CardType::Instant) || f.is(CardType::Sorcery)) && !crate::variants::is_nontraditional(def)
}

/// The faces of a permanent card that can be up on the battlefield, with the face state.
fn permanent_faces(def: &CardDef) -> Vec<(FaceState, &'static str)> {
    let mut v = Vec::new();
    if def.characteristics(FaceState::Front).is_permanent_type() {
        v.push((FaceState::Front, "front"));
    }
    match def.layout {
        Layout::Transform | Layout::ModalDfc | Layout::Battle | Layout::Meld
            if def.faces.len() > 1 && def.faces[1].chars.is_permanent_type() =>
        {
            v.push((FaceState::Back, "back"));
        }
        Layout::Flip if def.faces.len() > 1 => v.push((FaceState::Flipped, "flipped")),
        _ => {}
    }
    v
}

/// A permanent of `def` with `face` up, put onto the battlefield under `p`'s control
/// directly (no replacement effects, so its copiable values are the printed ones).
fn put_onto_battlefield(
    t: &mut TestGame,
    p: PlayerId,
    def: &Arc<CardDef>,
    face: FaceState,
) -> ObjectId {
    let id = t.g.create_card_object(def.clone(), p, Zone::Battlefield);
    {
        let o = &mut t.g.objects[id.0 as usize];
        o.face = face;
        o.flipped = face == FaceState::Flipped;
        o.base = def.characteristics(face);
        o.controller = p;
        o.base_controller = p;
        o.summoning_sick = false;
    }
    t.g.battlefield.push(id);
    t.g.dirty = true;
    t.g.recompute();
    id
}

fn vanilla(name: &str) -> CardDef {
    CardDef::custom(Characteristics {
        name: name.into(),
        card_types: CardTypeSet::single(CardType::Creature),
        power: Some(1),
        toughness: Some(1),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// Two players; each has a creature and a land (for an Aura or Equipment copy to be
/// attached to).
fn fixture() -> TestGame {
    let mut t = TestGame::new(2);
    t.g.logging = false;
    for p in [P0, P1] {
        t.custom(p, vanilla("Anchor"), Zone::Battlefield);
        t.battlefield(p, "Wastes");
    }
    t
}

const ORIG: Var = vars::USER + 9601;
const TARGET: Var = vars::USER + 9602;

/// The new objects on the battlefield since `before` objects existed.
fn new_permanents(t: &TestGame, before: usize) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| id.0 as usize >= before)
        .collect()
}

/// The "enters as a copy" card for a permanent with these types: Clone (a creature),
/// Sculpting Steel (an artifact), Copy Enchantment (an enchantment), Vesuva (a land).
fn clone_card(c: &Characteristics) -> Option<&'static str> {
    if c.is(CardType::Creature) {
        Some("Clone")
    } else if c.is(CardType::Artifact) {
        Some("Sculpting Steel")
    } else if c.is(CardType::Enchantment) {
        Some("Copy Enchantment")
    } else if c.is(CardType::Land) {
        Some("Vesuva")
    } else {
        None
    }
}

/// Whether one of the card's abilities sets its copiable values as it enters ("as this
/// enters, it becomes your choice of ...", "you may have it enter as a copy of ...", CR
/// 707.2, 707.5): its copiable values on the battlefield may then differ from the printed
/// ones.
fn sets_copiable_values_as_it_enters(def: &CardDef, names: &[&str]) -> bool {
    def.faces.iter().any(|f| {
        f.chars.abilities.iter().any(|a| {
            let fp = crate::structure::fingerprint(a, names);
            fp.contains("EnterAs(") || fp.contains("EnterAsCopy") || fp.contains("EnterCopy")
        })
    })
}

/// Whether the card has an "as this enters" ability that sets characteristics ("it becomes
/// your choice of a 3/3 creature with flying, ..."). A copy of it makes its own choice as
/// it enters, which becomes part of its copiable values on top of the copied ones (CR
/// 707.2; "there will be two copy effects", Corrupted Shapeshifter's rulings): the
/// characteristics such a choice sets ([`OWN_CHOICE_FIELDS`]) aren't compared.
fn makes_own_choice_as_it_enters(def: &CardDef, names: &[&str]) -> bool {
    def.faces.iter().any(|f| {
        f.chars
            .abilities
            .iter()
            .any(|a| crate::structure::fingerprint(a, names).contains("EnterAs("))
    })
}

const OWN_CHOICE_FIELDS: [&str; 6] = [
    "power",
    "toughness",
    "abilities",
    "card types",
    "subtypes",
    "colors",
];

/// Moves a new object of `def` onto the battlefield under `p`'s control through a real
/// zone change (replacement effects apply, CR 614.12; "may" choices are declined). Falls
/// back to putting it there directly if it can't enter that way (an Aura with nothing to
/// enchant).
fn enter(t: &mut TestGame, p: PlayerId, def: &Arc<CardDef>, face: FaceState) -> ObjectId {
    if face == FaceState::Front {
        t.answer_choose(p, &[]);
        t.answer_yes(p, false);
        let id = t.g.create_card_object(def.clone(), p, Zone::Nowhere);
        let r = t.g.move_object_ev(crate::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: crate::events::MoveCause::Effect,
            by: Some(p),
            etb: crate::replacement::EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        });
        t.clear_answers();
        if let Some(n) = r.filter(|n| t.g.obj(*n).zone == Zone::Battlefield) {
            t.g.recompute();
            return n;
        }
    }
    put_onto_battlefield(t, p, def, face)
}

fn push_mismatch(
    out: &mut CardResult,
    card: &str,
    path: &'static str,
    face: &'static str,
    field: &str,
    expected: &str,
    got: String,
) {
    out.mismatches.push(Mismatch {
        card: card.to_string(),
        path,
        face,
        field: field.into(),
        expected: expected.into(),
        got,
    });
}

/// The static check of one card (see the module docs).
pub fn check_card(def: &Arc<CardDef>) -> CardResult {
    let mut out = CardResult::default();
    let names = names_of(def);
    let card = def.name.to_string();
    if is_permanent_card(def) {
        let as_enters = sets_copiable_values_as_it_enters(def, &names);
        let own_choice = makes_own_choice_as_it_enters(def, &names);
        for (face, face_name) in permanent_faces(def) {
            let printed = printed_values(def, face, &names);
            // The original's own copiable values are the printed ones: put onto the
            // battlefield directly, and (unless an ability of its sets them as it enters)
            // entering it.
            {
                let mut t = fixture();
                let orig = put_onto_battlefield(&mut t, P0, def, face);
                let got = values(&t.g.obj(orig).copiable, &names);
                compare(&mut out, &card, "original", face_name, &printed, &got);
            }
            if !as_enters && face == FaceState::Front {
                let mut t = fixture();
                let orig = enter(&mut t, P0, def, face);
                let got = values(&t.g.obj(orig).copiable, &names);
                compare(
                    &mut out,
                    &card,
                    "original, entered",
                    face_name,
                    &printed,
                    &got,
                );
            }
            if own_choice {
                out.exempt = OWN_CHOICE_FIELDS.to_vec();
            }
            // The copies copy the original as it entered (with its own "as this enters"
            // choices, CR 707.2); they're the opponent's, so the original's abilities about
            // its controller's tokens or permanents don't apply to them. A copy that may
            // itself enter as a copy of something doesn't (CR 707.9: so none of its own
            // exceptions apply either).
            //
            // A token that's a copy of it (CR 707.2, 111.10; a double-faced permanent's
            // token copy is double-faced with the same face up, CR 707.8a).
            {
                let mut t = fixture();
                let orig = enter(&mut t, P0, def, face);
                let expected = values(&t.g.obj(orig).copiable, &names);
                t.answer_choose(P1, &[]);
                t.answer_yes(P1, false);
                let before = t.g.objects.len();
                let mut ctx = crate::eval::Ctx::new(None, P1);
                ctx.set_var(ORIG, vec![Entity::Object(orig)]);
                t.g.exec(
                    &Effect::CreateTokenCopy {
                        of: Sel::Var(ORIG),
                        count: Value::Const(1),
                        controller: PlayerRef::You,
                        tapped: false,
                        attacking: false,
                        mods: vec![],
                    },
                    &mut ctx,
                );
                t.g.recompute();
                match new_permanents(&t, before).first() {
                    Some(tok) => {
                        let o = t.g.obj(*tok);
                        compare(
                            &mut out,
                            &card,
                            "token copy",
                            face_name,
                            &expected,
                            &values(&o.copiable, &names),
                        );
                        if !o.is_token() {
                            push_mismatch(
                                &mut out,
                                &card,
                                "token copy",
                                face_name,
                                "token",
                                "a token",
                                "not a token".into(),
                            );
                        }
                        if face == FaceState::Back && o.face != FaceState::Back {
                            push_mismatch(
                                &mut out,
                                &card,
                                "token copy",
                                face_name,
                                "face up",
                                "back face up (CR 707.8a)",
                                format!("{:?}", o.face),
                            );
                        }
                    }
                    None => out
                        .skipped
                        .push(format!("{face_name}: no token copy was created")),
                }
            }
            // An object that becomes a copy of it (CR 707.4): a vanilla creature.
            {
                let mut t = fixture();
                let orig = enter(&mut t, P0, def, face);
                let expected = values(&t.g.obj(orig).copiable, &names);
                let target = t.custom(P1, vanilla("Copy Target"), Zone::Battlefield);
                let mut ctx = crate::eval::Ctx::new(None, P1);
                ctx.set_var(ORIG, vec![Entity::Object(orig)]);
                ctx.set_var(TARGET, vec![Entity::Object(target)]);
                t.g.exec(
                    &Effect::BecomeCopy {
                        what: Sel::Var(TARGET),
                        of: Sel::Var(ORIG),
                        duration: Duration::Permanent,
                    },
                    &mut ctx,
                );
                t.g.dirty = true;
                t.g.recompute();
                compare(
                    &mut out,
                    &card,
                    "becomes a copy",
                    face_name,
                    &expected,
                    &values(&t.g.obj(target).copiable, &names),
                );
            }
            // A permanent that enters as a copy of it (CR 707.5), chosen by a card that
            // can copy what it is now.
            {
                let mut t = fixture();
                let orig = enter(&mut t, P0, def, face);
                let expected = values(&t.g.obj(orig).copiable, &names);
                if let Some(cloner) = clone_card(&t.g.obj(orig).chars.clone()) {
                    t.answer_choose(P1, &[Entity::Object(orig)]);
                    t.answer_choose(P1, &[]);
                    t.answer_yes(P1, false);
                    let before = t.g.objects.len();
                    let id =
                        t.g.create_card_object(crate::card::card(cloner), P1, Zone::Nowhere);
                    let r = t.g.move_object_ev(crate::replacement::MoveEv {
                        obj: id,
                        to: Zone::Battlefield,
                        pos: LibraryPosition::Top,
                        cause: crate::events::MoveCause::Effect,
                        by: Some(P1),
                        etb: crate::replacement::EtbInfo {
                            controller: Some(P1),
                            ..Default::default()
                        },
                        source: None,
                    });
                    t.g.recompute();
                    match r.filter(|_| !new_permanents(&t, before).is_empty()) {
                        Some(c) => compare(
                            &mut out,
                            &card,
                            "enters as a copy",
                            face_name,
                            &expected,
                            &values(&t.g.obj(t.g.current(c)).copiable, &names),
                        ),
                        None => out
                            .skipped
                            .push(format!("{face_name}: {cloner} didn't enter")),
                    }
                }
            }
            out.exempt.clear();
        }
    }
    if is_instant_or_sorcery(def) || (is_permanent_card(def) && !def.front().chars.is_land()) {
        check_spell_copy(def, &names, &mut out);
    }
    out
}

/// A board with something for most spells to target: creatures, an artifact, an
/// enchantment and a planeswalker on each side, cards in graveyards and hands, and a spell
/// of the opponent's on the stack.
fn spell_fixture() -> TestGame {
    let mut t = fixture();
    for p in [P0, P1] {
        t.battlefield(p, "Grizzly Bears");
        t.battlefield(p, "Serra Angel");
        t.battlefield(p, "Sol Ring");
        t.battlefield(p, "Pacifism");
        t.battlefield(p, "Forest");
        t.battlefield(p, "Island");
        t.battlefield(p, "Jace Beleren");
        t.graveyard(p, "Grizzly Bears");
        t.graveyard(p, "Lightning Bolt");
        t.hand(p, "Shock");
    }
    let bolt = t.hand(P1, "Lightning Bolt");
    fill_pool(&mut t, P1);
    let _ = t.cast(P1, bolt).target(Entity::Player(P0)).try_go();
    t.g.turn.priority = Some(P0);
    t
}

/// Plenty of mana of each type in `p`'s mana pool.
fn fill_pool(t: &mut TestGame, p: PlayerId) {
    for ty in crate::mana::ManaType::ALL {
        t.g.players[p.idx()].mana_pool.add_type(ty, 8);
    }
}

/// The choices made for a spell that its copy has too (CR 707.10), as text.
fn spell_choices(o: &GameObject) -> Vec<(&'static str, String)> {
    let Some(si) = o.stack.as_deref() else {
        return vec![("stack", "none".into())];
    };
    vec![
        ("modes and targets", format!("{:?}", si.chosen)),
        ("X", format!("{:?} / {:?}", si.x, si.cast.x)),
        ("times kicked", si.cast.times_kicked.to_string()),
        ("costs paid", format!("{:?}", si.cast.paid)),
        ("cast method", format!("{:?}", si.cast.method)),
        (
            "objects that paid costs",
            format!("{:?}", si.cast.cost_objects),
        ),
        ("chosen values", format!("{:?}", si.chosen_values)),
        ("choices", format!("{:?}", o.choices)),
        ("face", format!("{:?}", o.face)),
    ]
}

/// A copy of the spell has the spell's characteristics and choices (CR 707.10); a copy of
/// a permanent spell resolves as a token with the card's copiable values (CR 707.10f).
fn check_spell_copy(def: &Arc<CardDef>, names: &[&str], out: &mut CardResult) {
    let card = def.name.to_string();
    let mut t = spell_fixture();
    let id = t.custom(P0, (**def).clone(), Zone::Hand(P0));
    // Respect the card's own timing: it's P0's main phase with an empty stack below
    // the opponent's spell; sorcery-speed cards are cast with the stack empty.
    let instant_speed = def.front().chars.is(CardType::Instant)
        || def
            .front()
            .chars
            .abilities
            .iter()
            .any(|a| matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == crate::keywords::KeywordKind::Flash));
    if !instant_speed {
        t.resolve_all();
        t.g.turn.priority = Some(P0);
    }
    fill_pool(&mut t, P0);
    let spell = match t.cast(P0, id).try_go() {
        Ok(s) => s,
        Err(e) => {
            out.skipped
                .push(format!("spell: can't be cast in the fixture ({e:?})"));
            return;
        }
    };
    if t.g.obj(spell).zone != Zone::Stack {
        out.skipped
            .push("spell: not on the stack after casting".into());
        return;
    }
    let Some(copy) = crate::copy::copy_spell(&mut t.g, spell, P0, false) else {
        out.skipped.push("spell: can't be copied".into());
        return;
    };
    t.g.recompute();
    let face_name = "front";
    let orig = t.g.obj(spell);
    let cp = t.g.obj(copy);
    compare(
        out,
        &card,
        "spell copy",
        face_name,
        &values(&orig.copiable, names),
        &values(&cp.copiable, names),
    );
    out.copies -= 1;
    compare(
        out,
        &card,
        "spell copy",
        face_name,
        &Values(spell_choices(orig)),
        &Values(spell_choices(cp)),
    );
    if cp.kind != ObjKind::SpellCopy || cp.controller != P0 || cp.owner != P0 {
        out.mismatches.push(Mismatch {
            card: card.clone(),
            path: "spell copy",
            face: face_name,
            field: "kind, controller and owner".into(),
            expected: "a spell copy controlled and owned by its controller (CR 707.10)".into(),
            got: format!(
                "{:?} controlled by {} owned by {}",
                cp.kind, cp.controller, cp.owner
            ),
        });
    }
    // A copy of a permanent spell becomes a token as it resolves (CR 707.10f, 608.3f).
    if def.characteristics(orig.face).is_permanent_type() && orig.face == FaceState::Front {
        let expected = printed_values(def, FaceState::Front, names);
        let before = t.g.objects.len();
        t.g.settle();
        // The token doesn't enter as a copy of something else (see `check_card`).
        t.answer_choose(P0, &[]);
        t.answer_yes(P0, false);
        if makes_own_choice_as_it_enters(def, names) {
            out.exempt = OWN_CHOICE_FIELDS.to_vec();
        }
        if t.g.stack.last() == Some(&copy) {
            t.g.resolve_top();
            t.g.recompute();
            match new_permanents(&t, before).first() {
                Some(tok) => {
                    let o = t.g.obj(*tok);
                    compare(
                        out,
                        &card,
                        "permanent spell copy",
                        face_name,
                        &expected,
                        &values(&o.copiable, names),
                    );
                    if !o.is_token() {
                        out.mismatches.push(Mismatch {
                            card,
                            path: "permanent spell copy",
                            face: face_name,
                            field: "token".into(),
                            expected: "a token (CR 707.10f)".into(),
                            got: "not a token".into(),
                        });
                    }
                }
                None => out
                    .skipped
                    .push("permanent spell copy: didn't resolve to a permanent".into()),
            }
        }
    }
}

/// Runs the static check over `cards`, catching panics (each reported as a mismatch).
pub fn check_cards(
    cards: &[Arc<CardDef>],
) -> (
    usize,
    Vec<Mismatch>,
    std::collections::BTreeMap<String, usize>,
) {
    let mut copies = 0;
    let mut mismatches = Vec::new();
    let mut skipped = std::collections::BTreeMap::new();
    for def in cards {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check_card(def)));
        match r {
            Ok(r) => {
                copies += r.copies;
                for k in r.skipped {
                    *skipped.entry(k).or_insert(0) += 1;
                }
                mismatches.extend(r.mismatches);
            }
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                mismatches.push(Mismatch {
                    card: def.name.to_string(),
                    path: "panic",
                    face: "front",
                    field: "panic".into(),
                    expected: "no panic".into(),
                    got: msg,
                });
            }
        }
    }
    (copies, mismatches, skipped)
}

/// The fully supported cards among every `stride`th name of the pool starting at
/// `offset` (only those are compiled).
pub fn sample(offset: usize, stride: usize) -> Vec<Arc<CardDef>> {
    let names: Vec<String> = pool_names()
        .iter()
        .skip(offset)
        .step_by(stride.max(1))
        .cloned()
        .collect();
    supported(&names)
}
