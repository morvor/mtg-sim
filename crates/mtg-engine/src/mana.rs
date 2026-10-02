//! Mana, mana symbols, mana costs, mana pools, and payment (CR 106, 107.4, 118, 202).

use crate::types::{CardType, CardTypeSet, Color, ColorSet, ObjectId, Subtype};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt;

/// One of the six types of mana (CR 106.1b).
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum ManaType {
    W,
    U,
    B,
    R,
    G,
    C,
}

impl ManaType {
    pub const ALL: [ManaType; 6] = [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ];

    pub fn color(self) -> Option<Color> {
        Some(match self {
            ManaType::W => Color::White,
            ManaType::U => Color::Blue,
            ManaType::B => Color::Black,
            ManaType::R => Color::Red,
            ManaType::G => Color::Green,
            ManaType::C => return None,
        })
    }

    pub fn from_color(c: Color) -> ManaType {
        match c {
            Color::White => ManaType::W,
            Color::Blue => ManaType::U,
            Color::Black => ManaType::B,
            Color::Red => ManaType::R,
            Color::Green => ManaType::G,
        }
    }

    pub fn from_letter(c: char) -> Option<ManaType> {
        match c.to_ascii_uppercase() {
            'C' => Some(ManaType::C),
            c => Color::from_letter(c).map(ManaType::from_color),
        }
    }
}

/// A mana symbol as it appears in a cost (CR 107.4).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ManaSymbol {
    /// {0}, {1}, {2}, ...
    Generic(u32),
    /// {W} {U} {B} {R} {G}
    Colored(Color),
    /// {C}
    Colorless,
    /// {S}
    Snow,
    /// {X}
    X,
    /// {Y}
    Y,
    /// {Z}
    Z,
    /// {W/U} etc.
    Hybrid(Color, Color),
    /// {2/W} etc. — one colored mana or two generic.
    TwoHybrid(Color),
    /// {C/W} etc. — one colorless mana or one colored mana.
    ColorlessHybrid(Color),
    /// {W/P} etc. — one colored mana or 2 life.
    Phyrexian(Color),
    /// {W/U/P} etc.
    PhyrexianHybrid(Color, Color),
    /// Un-set half mana ({HW}, {½}). Treated as contributing 0.5 to mana value.
    Half(Option<Color>),
    /// {∞}
    Infinity,
}

impl ManaSymbol {
    /// Contribution to mana value (CR 202.3), with X counted as `x`.
    pub fn mana_value(self, x: u32) -> u32 {
        match self {
            ManaSymbol::Generic(n) => n,
            ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z => x,
            ManaSymbol::TwoHybrid(_) => 2,
            ManaSymbol::Half(_) => 0,
            ManaSymbol::Infinity => 1_000_000,
            _ => 1,
        }
    }

    pub fn colors(self) -> ColorSet {
        match self {
            ManaSymbol::Colored(c)
            | ManaSymbol::TwoHybrid(c)
            | ManaSymbol::ColorlessHybrid(c)
            | ManaSymbol::Phyrexian(c)
            | ManaSymbol::Half(Some(c)) => ColorSet::single(c),
            ManaSymbol::Hybrid(a, b) | ManaSymbol::PhyrexianHybrid(a, b) => {
                let mut s = ColorSet::single(a);
                s.insert(b);
                s
            }
            _ => ColorSet::NONE,
        }
    }

    /// One of the fifteen Phyrexian mana symbols (CR 107.4f, 107.4g).
    pub fn is_phyrexian(self) -> bool {
        matches!(
            self,
            ManaSymbol::Phyrexian(_) | ManaSymbol::PhyrexianHybrid(..)
        )
    }

    pub fn is_variable(self) -> bool {
        matches!(self, ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z)
    }

    pub fn parse(inner: &str) -> Option<ManaSymbol> {
        let s = inner
            .trim_matches(|c| c == '{' || c == '}')
            .to_ascii_uppercase();
        if let Ok(n) = s.parse::<u32>() {
            return Some(ManaSymbol::Generic(n));
        }
        let parts: Vec<&str> = s.split('/').collect();
        Some(match parts.as_slice() {
            ["C"] => ManaSymbol::Colorless,
            ["S"] => ManaSymbol::Snow,
            ["X"] => ManaSymbol::X,
            ["Y"] => ManaSymbol::Y,
            ["Z"] => ManaSymbol::Z,
            ["∞"] => ManaSymbol::Infinity,
            ["½"] => ManaSymbol::Half(None),
            [c] if c.len() == 1 => ManaSymbol::Colored(Color::from_letter(c.chars().next()?)?),
            [c] if c.len() == 2 && c.starts_with('H') => {
                ManaSymbol::Half(Some(Color::from_letter(c.chars().nth(1)?)?))
            }
            ["2", c] => ManaSymbol::TwoHybrid(Color::from_letter(c.chars().next()?)?),
            ["C", c] => ManaSymbol::ColorlessHybrid(Color::from_letter(c.chars().next()?)?),
            [c, "P"] => ManaSymbol::Phyrexian(Color::from_letter(c.chars().next()?)?),
            [a, b, "P"] => ManaSymbol::PhyrexianHybrid(
                Color::from_letter(a.chars().next()?)?,
                Color::from_letter(b.chars().next()?)?,
            ),
            [a, b] => ManaSymbol::Hybrid(
                Color::from_letter(a.chars().next()?)?,
                Color::from_letter(b.chars().next()?)?,
            ),
            _ => return None,
        })
    }
}

impl fmt::Display for ManaSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManaSymbol::Generic(n) => write!(f, "{{{n}}}"),
            ManaSymbol::Colored(c) => write!(f, "{{{}}}", c.letter()),
            ManaSymbol::Colorless => write!(f, "{{C}}"),
            ManaSymbol::Snow => write!(f, "{{S}}"),
            ManaSymbol::X => write!(f, "{{X}}"),
            ManaSymbol::Y => write!(f, "{{Y}}"),
            ManaSymbol::Z => write!(f, "{{Z}}"),
            ManaSymbol::Hybrid(a, b) => write!(f, "{{{}/{}}}", a.letter(), b.letter()),
            ManaSymbol::TwoHybrid(c) => write!(f, "{{2/{}}}", c.letter()),
            ManaSymbol::ColorlessHybrid(c) => write!(f, "{{C/{}}}", c.letter()),
            ManaSymbol::Phyrexian(c) => write!(f, "{{{}/P}}", c.letter()),
            ManaSymbol::PhyrexianHybrid(a, b) => write!(f, "{{{}/{}/P}}", a.letter(), b.letter()),
            ManaSymbol::Half(Some(c)) => write!(f, "{{H{}}}", c.letter()),
            ManaSymbol::Half(None) => write!(f, "{{½}}"),
            ManaSymbol::Infinity => write!(f, "{{∞}}"),
        }
    }
}

/// A mana cost: an ordered list of mana symbols (CR 202.1). An empty cost with
/// `is_none == false` is `{0}`-like "no mana"; cards with *no* mana cost (e.g. lands)
/// are represented with `Option<ManaCost> = None` at the characteristic level.
#[derive(Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ManaCost {
    pub symbols: SmallVec<[ManaSymbol; 6]>,
}

impl ManaCost {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses "{2}{W}{U}" style costs. Returns `None` for an empty string.
    pub fn parse(s: &str) -> Option<ManaCost> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        let mut symbols = SmallVec::new();
        let mut rest = s;
        while let Some(start) = rest.find('{') {
            let end = rest[start..].find('}')? + start;
            symbols.push(ManaSymbol::parse(&rest[start + 1..end])?);
            rest = &rest[end + 1..];
        }
        Some(ManaCost { symbols })
    }

    pub fn generic(n: u32) -> ManaCost {
        let mut symbols = SmallVec::new();
        if n > 0 {
            symbols.push(ManaSymbol::Generic(n));
        }
        ManaCost { symbols }
    }

    /// Mana value with the given value for X (CR 202.3). X is 0 everywhere except on
    /// the stack (CR 107.3f).
    pub fn mana_value_with_x(&self, x: u32) -> u32 {
        self.symbols.iter().map(|s| s.mana_value(x)).sum()
    }

    pub fn mana_value(&self) -> u32 {
        self.mana_value_with_x(0)
    }

    /// Colors of the cost (CR 202.2).
    pub fn colors(&self) -> ColorSet {
        self.symbols
            .iter()
            .fold(ColorSet::NONE, |acc, s| acc.union(s.colors()))
    }

    pub fn has_x(&self) -> bool {
        self.symbols.iter().any(|s| matches!(s, ManaSymbol::X))
    }

    pub fn x_count(&self) -> u32 {
        self.symbols
            .iter()
            .filter(|s| matches!(s, ManaSymbol::X))
            .count() as u32
    }

    pub fn is_zero(&self) -> bool {
        self.symbols
            .iter()
            .all(|s| matches!(s, ManaSymbol::Generic(0)))
    }

    /// Count of a specific colored symbol (for devotion, CR 700.5).
    pub fn devotion_to(&self, c: Color) -> u32 {
        self.symbols
            .iter()
            .filter(|s| s.colors().contains(c))
            .count() as u32
    }

    /// Replaces X/Y/Z with generic amounts.
    pub fn with_x(&self, x: u32) -> ManaCost {
        let mut out = ManaCost::default();
        let mut generic = 0;
        for s in &self.symbols {
            match s {
                ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z => generic += x,
                ManaSymbol::Generic(n) => generic += n,
                other => out.symbols.push(*other),
            }
        }
        if generic > 0 {
            out.symbols.insert(0, ManaSymbol::Generic(generic));
        }
        out
    }

    pub fn generic_amount(&self) -> u32 {
        self.symbols
            .iter()
            .map(|s| {
                if let ManaSymbol::Generic(n) = s {
                    *n
                } else {
                    0
                }
            })
            .sum()
    }

    /// Adds another cost's symbols (for additional costs / cost increases).
    pub fn add(&mut self, other: &ManaCost) {
        let mut generic = self.generic_amount() + other.generic_amount();
        let mut syms: SmallVec<[ManaSymbol; 6]> = SmallVec::new();
        for s in self.symbols.iter().chain(other.symbols.iter()) {
            if !matches!(s, ManaSymbol::Generic(_)) {
                syms.push(*s);
            }
        }
        if generic > 0 {
            syms.insert(0, ManaSymbol::Generic(generic));
            generic = 0;
        }
        let _ = generic;
        self.symbols = syms;
    }

    /// Reduces the generic component by up to `n` (CR 601.2f; can't go below {0}).
    pub fn reduce_generic(&mut self, n: u32) {
        let g = self.generic_amount().saturating_sub(n);
        self.symbols
            .retain(|s| !matches!(s, ManaSymbol::Generic(_)));
        if g > 0 {
            self.symbols.insert(0, ManaSymbol::Generic(g));
        }
    }

    /// Removes one colored symbol of the given color if present (for effects like
    /// "costs {W} less"). Returns true if one was removed.
    pub fn reduce_colored(&mut self, c: Color) -> bool {
        if let Some(i) = self
            .symbols
            .iter()
            .position(|s| *s == ManaSymbol::Colored(c))
        {
            self.symbols.remove(i);
            true
        } else {
            false
        }
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.symbols.is_empty() {
            return write!(f, "{{0}}");
        }
        for s in &self.symbols {
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Restrictions on how a unit of mana may be spent (CR 106.6).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ManaRestriction {
    /// "Spend this mana only to cast creature spells."
    SpellOfType(CardType),
    /// "Spend this mana only to cast a creature spell of the chosen type."
    SpellWithSubtype(Subtype),
    /// [`ManaRestriction::SpellWithSubtype`] of the type chosen for the mana's source
    /// (CR 607.2d); bound when the mana is produced. Unbound (no choice was made), it
    /// permits nothing (CR 607.5a).
    SpellOfChosenType,
    /// "Spend this mana only to cast spells" (not activate abilities).
    SpellsOnly,
    /// "Spend this mana only to activate abilities."
    AbilitiesOnly,
    /// "Spend this mana only on costs that include {X}" etc.
    XCostsOnly,
    /// "Spend this mana only to cast artifact spells or activate abilities of artifacts."
    ArtifactSpellOrAbility,
    /// "Spend this mana only to cast instant or sorcery spells."
    InstantOrSorcery,
    /// "Spend this mana only to cast noncreature spells."
    NoncreatureSpell,
    /// "This mana can't be spent to cast a nonartifact spell." (Powerstone, CR 111.10h):
    /// it can pay for anything except casting a nonartifact spell.
    NotNonartifactSpell,
    /// Several restrictions, any of which permits the spend.
    AnyOf(Vec<ManaRestriction>),
    /// "Spend this mana only to cast Dragon spells": the spell being cast matches the
    /// filter (checked with the game, see [`ManaRestriction::allows_in`]).
    CastSpell(SpendFilter),
    /// "... or activate abilities of Dragons": the source of the ability being activated
    /// matches the filter.
    ActivateAbilityOf(SpendFilter),
    /// "... or to gain a Class level": to activate a class level bar's ability
    /// (CR 716.2c).
    ClassLevel,
    /// "This mana can't be spent to pay generic mana costs." (Jegantha, the Wellspring):
    /// it can be spent on anything, but only to pay a colored or colorless mana symbol (or
    /// the colored half of a hybrid one), never generic mana: numbers, {X}, {S} or the
    /// generic half of a {2/W}-style symbol. Checked symbol by symbol as the cost is paid
    /// (see [`Mana::pays_generic`]).
    NotGeneric,
}

/// A filter inside a [`ManaRestriction`], compared structurally.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendFilter(pub Box<crate::ability::Filter>);

impl SpendFilter {
    pub fn new(f: crate::ability::Filter) -> Self {
        SpendFilter(Box::new(f))
    }
}

impl PartialEq for SpendFilter {
    fn eq(&self, o: &Self) -> bool {
        format!("{:?}", self.0) == format!("{:?}", o.0)
    }
}
impl Eq for SpendFilter {}
impl std::hash::Hash for SpendFilter {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        format!("{:?}", self.0).hash(h)
    }
}

/// What the mana is being spent on, for checking restrictions.
#[derive(Clone, Debug, Default)]
pub struct SpendContext {
    pub is_spell: bool,
    pub is_ability: bool,
    pub card_types: CardTypeSet,
    pub subtypes: Vec<Subtype>,
    /// The spell is every creature type (changeling, CR 702.73a) besides `subtypes`.
    pub all_creature_types: bool,
    pub has_x: bool,
    pub source: Option<ObjectId>,
    /// Mana types that may be spent as though they were mana of any color for this
    /// payment (e.g. "You may spend blue mana as though it were mana of any color to pay
    /// the activation costs of this creature's abilities", CR 602.1e).
    pub any_color: Vec<ManaType>,
    /// A rough "could this be paid" check made without knowing what the mana would be
    /// spent on: spending restrictions (CR 106.6) are ignored. An actual payment never
    /// sets it, so restricted mana isn't planned for a purpose it can't pay.
    pub check_only: bool,
    /// The ability being activated is a class level bar's: activating it is gaining a
    /// Class level (CR 716.2c).
    pub class_level: bool,
    /// What the costs being paid are for: casting a spell or activating an ability, or
    /// `None` for a cost a resolving spell or ability asks for (CR 118.3; see
    /// `rule_statics::payment`).
    pub cost_of: Option<crate::rule_statics::payment::CostOf>,
    /// "Spend only black mana on X": how much of the generic mana of the cost X still
    /// represents, and the mana that may pay it (see `payment_rules.rs`).
    pub x_spend: Option<XSpend>,
    /// Colors whose mana symbols the payer may pay with life instead ("For each {B} in a
    /// cost, you may pay 2 life rather than pay that mana"), with the life each costs. The
    /// payment fills it in from the payer's modifications (`payment_rules::life_for_mana`).
    pub pay_life_for: Vec<(Color, u32)>,
    /// The ability on the stack whose activation cost is being paid ("When you spend this
    /// mana to cast a spell or activate an ability", CR 106.6).
    pub ability_on_stack: Option<ObjectId>,
}

/// The part of a cost's generic mana that X represents, which only mana of `colors` may pay
/// ("Spend only black mana on X", "Spend only colored mana on X"; with `distinct`, "No more
/// than one mana of each color may be spent this way"). Mana that may be spent as though it
/// were mana of any color counts as any of them (CR 609.4b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XSpend {
    pub amount: u32,
    pub colors: ColorSet,
    pub distinct: bool,
}

impl ManaRestriction {
    pub fn allows(&self, ctx: &SpendContext) -> bool {
        match self {
            ManaRestriction::SpellOfType(t) => ctx.is_spell && ctx.card_types.contains(*t),
            ManaRestriction::SpellWithSubtype(s) => {
                ctx.is_spell
                    && ctx.card_types.contains(CardType::Creature)
                    && (ctx.subtypes.iter().any(|x| x == s)
                        || (ctx.all_creature_types && crate::types::is_creature_type(s)))
            }
            ManaRestriction::SpellOfChosenType => false,
            ManaRestriction::SpellsOnly => ctx.is_spell,
            ManaRestriction::AbilitiesOnly => ctx.is_ability,
            ManaRestriction::XCostsOnly => ctx.has_x,
            ManaRestriction::ArtifactSpellOrAbility => {
                ctx.card_types.contains(CardType::Artifact) && (ctx.is_spell || ctx.is_ability)
            }
            ManaRestriction::InstantOrSorcery => {
                ctx.is_spell
                    && (ctx.card_types.contains(CardType::Instant)
                        || ctx.card_types.contains(CardType::Sorcery))
            }
            ManaRestriction::NoncreatureSpell => {
                ctx.is_spell && !ctx.card_types.contains(CardType::Creature)
            }
            ManaRestriction::NotNonartifactSpell => {
                !ctx.is_spell || ctx.card_types.contains(CardType::Artifact)
            }
            ManaRestriction::ClassLevel => ctx.is_ability && ctx.class_level,
            // What it may pay is decided symbol by symbol.
            ManaRestriction::NotGeneric => true,
            ManaRestriction::AnyOf(v) => v.iter().any(|r| r.allows(ctx)),
            // Filters need the game: see `allows_in`.
            ManaRestriction::CastSpell(_) | ManaRestriction::ActivateAbilityOf(_) => false,
        }
    }

    /// [`ManaRestriction::allows`], also checking filter restrictions against the spell
    /// or the ability's source being paid for (`ctx.source`). "You" in a filter is the
    /// player spending the mana.
    pub fn allows_in(
        &self,
        g: &crate::game::Game,
        payer: crate::types::PlayerId,
        mana_source: Option<ObjectId>,
        ctx: &SpendContext,
    ) -> bool {
        let matches = |f: &SpendFilter| {
            ctx.source.is_some_and(|s| {
                g.try_obj(s).is_some()
                    && g.matches(s, &f.0, &crate::eval::Ctx::new(mana_source, payer))
            })
        };
        match self {
            ManaRestriction::CastSpell(f) => ctx.is_spell && matches(f),
            ManaRestriction::ActivateAbilityOf(f) => ctx.is_ability && matches(f),
            ManaRestriction::AnyOf(v) => v.iter().any(|r| r.allows_in(g, payer, mana_source, ctx)),
            other => other.allows(ctx),
        }
    }
}

/// A delayed triggered ability created by the spell or ability that produced a unit of
/// mana, which triggers when that mana is spent to cast a matching spell (CR 106.6,
/// 603.7a). Each unit of mana carries its own, so an effect that increases the amount of
/// mana produced creates one per mana (CR 106.6a).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManaRider {
    pub id: u64,
    /// The spell the mana must be spent to cast.
    pub spell_filter: crate::ability::Filter,
    /// It also triggers when the mana is spent to activate an ability ("When you spend
    /// this mana to cast a spell or activate an ability", CR 106.6).
    #[serde(default)]
    pub abilities: bool,
    /// The triggered ability's effect ("that spell" is the spell the mana was spent on).
    pub body: crate::ability::Body,
    pub controller: crate::types::PlayerId,
    pub source: Option<ObjectId>,
}

impl PartialEq for ManaRider {
    fn eq(&self, o: &Self) -> bool {
        self.id == o.id
    }
}
impl Eq for ManaRider {}

/// A single unit of mana in a pool.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Mana {
    pub ty: ManaType,
    /// Produced by a snow source (CR 106.3, 107.4h).
    pub snow: bool,
    /// The object that produced it (for "mana from a Treasure", etc.).
    pub source: Option<ObjectId>,
    pub restriction: Option<ManaRestriction>,
    /// Doesn't empty from the pool at end of steps/phases (e.g. Upwelling-style effects
    /// or "until end of turn" mana).
    pub persistent: bool,
    /// "Until end of combat, you don't lose this mana as steps and phases end"
    /// (firebending, CR 702.189a): it doesn't empty from the pool as combat steps end, and
    /// is lost as the combat phase ends.
    #[serde(default)]
    pub until_end_of_combat: bool,
    /// "When that mana is spent to cast ..." (CR 106.6).
    #[serde(default)]
    pub rider: Option<Box<ManaRider>>,
    /// In Grand Melee, with several turns at once: the number of the turn it was added
    /// during (0 until known), whose steps empty it (see `multiplayer::grand_melee`).
    #[serde(default)]
    pub turn: u32,
}

impl Mana {
    pub fn new(ty: ManaType) -> Self {
        Mana {
            ty,
            snow: false,
            source: None,
            restriction: None,
            persistent: false,
            until_end_of_combat: false,
            rider: None,
            turn: 0,
        }
    }
    pub fn can_spend(&self, ctx: &SpendContext) -> bool {
        self.restriction.as_ref().is_none_or(|r| r.allows(ctx))
    }
    /// [`Mana::can_spend`], checking filter restrictions with the game (CR 106.6).
    pub fn can_spend_in(
        &self,
        g: &crate::game::Game,
        payer: crate::types::PlayerId,
        ctx: &SpendContext,
    ) -> bool {
        self.restriction
            .as_ref()
            .is_none_or(|r| r.allows_in(g, payer, self.source, ctx))
    }
    /// Whether it may pay generic mana ("This mana can't be spent to pay generic mana
    /// costs", [`ManaRestriction::NotGeneric`]).
    pub fn pays_generic(&self) -> bool {
        !matches!(self.restriction, Some(ManaRestriction::NotGeneric))
    }
}

/// A player's mana pool (CR 106.4).
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ManaPool {
    pub mana: Vec<Mana>,
}

impl ManaPool {
    pub fn add(&mut self, m: Mana) {
        self.mana.push(m);
    }
    pub fn add_type(&mut self, ty: ManaType, n: u32) {
        for _ in 0..n {
            self.mana.push(Mana::new(ty));
        }
    }
    pub fn total(&self) -> usize {
        self.mana.len()
    }
    pub fn count(&self, ty: ManaType) -> usize {
        self.mana.iter().filter(|m| m.ty == ty).count()
    }
    pub fn is_empty(&self) -> bool {
        self.mana.is_empty()
    }
    /// Empties the pool except persistent mana; returns the mana lost.
    pub fn empty(&mut self) -> Vec<Mana> {
        let (keep, lose): (Vec<_>, Vec<_>) = self.mana.drain(..).partition(|m| m.persistent);
        self.mana = keep;
        lose
    }
}

/// One way to pay each symbol, produced by [`find_payment`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaymentPlan {
    /// Indices into the pool that will be spent.
    pub pool_indices: Vec<usize>,
    /// Life to pay instead of mana: for Phyrexian symbols, and for symbols an effect lets
    /// the payer pay with life ([`SpendContext::pay_life_for`]).
    pub life: u32,
    /// How many Phyrexian symbols are paid with life (CR 107.4f, 702.150a).
    pub phyrexian: u32,
    /// The indices into the pool (among `pool_indices`) of the mana that pays the X a
    /// spell or ability limits to mana of particular colors ([`XSpend`]): "the amount of
    /// {B} spent on X".
    pub x_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
enum Req {
    Colored(Color),
    Colorless,
    Snow,
    Generic,
    /// One generic mana that X represents, which only mana of these colors may pay
    /// ([`XSpend`]).
    XOf(ColorSet),
    Hybrid(Color, Color),
    /// one colored or two generic
    TwoHybrid(Color),
    ColorlessHybrid(Color),
    Phyrexian(Color),
    PhyrexianHybrid(Color, Color),
}

impl Req {
    fn rank(self) -> u8 {
        match self {
            Req::Colored(_) | Req::Colorless => 0,
            Req::ColorlessHybrid(_) | Req::Hybrid(..) => 1,
            Req::Snow | Req::XOf(_) => 2,
            Req::TwoHybrid(_) | Req::Phyrexian(_) | Req::PhyrexianHybrid(..) => 3,
            Req::Generic => 4,
        }
    }

    /// The life that may pay the symbol instead of mana, and whether that's paying for a
    /// Phyrexian symbol (CR 107.4f): 2 life for a Phyrexian symbol, or what an effect lets
    /// the payer pay for a symbol of (or with a half of) a color ("For each {B} in a cost,
    /// you may pay 2 life rather than pay that mana" — never for generic mana).
    fn life(self, life_for: &[(Color, u32)]) -> Option<(u32, bool)> {
        let of = |c: Color| life_for.iter().find(|(x, _)| *x == c).map(|(_, n)| *n);
        match self {
            Req::Phyrexian(_) | Req::PhyrexianHybrid(..) => Some((2, true)),
            Req::Colored(c) | Req::TwoHybrid(c) | Req::ColorlessHybrid(c) => {
                of(c).map(|n| (n, false))
            }
            Req::Hybrid(a, b) => match (of(a), of(b)) {
                (Some(x), Some(y)) => Some((x.min(y), false)),
                (Some(x), None) | (None, Some(x)) => Some((x, false)),
                (None, None) => None,
            },
            Req::Colorless | Req::Snow | Req::Generic | Req::XOf(_) => None,
        }
    }
}

/// The requirements of a mana cost (X already substituted), with `x` of its generic mana
/// being X that only mana of particular colors may pay. `None` for an unpayable cost.
fn requirements(cost: &ManaCost, x: Option<XSpend>) -> Option<Vec<Req>> {
    let mut reqs: Vec<Req> = Vec::new();
    for s in &cost.symbols {
        match *s {
            ManaSymbol::Generic(n) => reqs.extend(std::iter::repeat_n(Req::Generic, n as usize)),
            ManaSymbol::Colored(c) => reqs.push(Req::Colored(c)),
            ManaSymbol::Colorless => reqs.push(Req::Colorless),
            ManaSymbol::Snow => reqs.push(Req::Snow),
            ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z => {}
            ManaSymbol::Hybrid(a, b) => reqs.push(Req::Hybrid(a, b)),
            ManaSymbol::TwoHybrid(c) => reqs.push(Req::TwoHybrid(c)),
            ManaSymbol::ColorlessHybrid(c) => reqs.push(Req::ColorlessHybrid(c)),
            ManaSymbol::Phyrexian(c) => reqs.push(Req::Phyrexian(c)),
            ManaSymbol::PhyrexianHybrid(a, b) => reqs.push(Req::PhyrexianHybrid(a, b)),
            ManaSymbol::Half(_) => {}
            ManaSymbol::Infinity => return None,
        }
    }
    if let Some(x) = x {
        // The generic mana X represents (as much of it as is left to pay).
        let n = x.amount.min(cost.generic_amount()) as usize;
        let mut left = n;
        reqs.retain(|r| {
            if left > 0 && matches!(r, Req::Generic) {
                left -= 1;
                false
            } else {
                true
            }
        });
        reqs.extend(std::iter::repeat_n(Req::XOf(x.colors), n));
    }
    reqs.sort_by_key(|r| r.rank());
    Some(reqs)
}

/// Finds a way to pay `cost` (X already substituted) from `pool`, spending only mana
/// allowed by `ctx`. `max_life` bounds the life that may be paid for Phyrexian
/// symbols (use 0 to require mana). Prefers mana payment over life.
pub fn find_payment(
    pool: &[Mana],
    cost: &ManaCost,
    ctx: &SpendContext,
    max_life: u32,
) -> Option<PaymentPlan> {
    let usable: Vec<bool> = pool.iter().map(|m| m.can_spend(ctx)).collect();
    find_payment_with(pool, cost, ctx, max_life, &usable)
}

/// [`find_payment`] where `usable[i]` says whether `pool[i]` may be spent on this payment
/// (its restrictions, CR 106.6, already checked).
pub fn find_payment_with(
    pool: &[Mana],
    cost: &ManaCost,
    ctx: &SpendContext,
    max_life: u32,
    usable: &[bool],
) -> Option<PaymentPlan> {
    let reqs = requirements(cost, ctx.x_spend)?;
    let solver = PoolSolver {
        pool,
        usable,
        any: &ctx.any_color,
        life_for: &ctx.pay_life_for,
        max_life,
        x_distinct: ctx.x_spend.is_some_and(|x| x.distinct),
    };
    let mut used = vec![false; pool.len()];
    let mut plan = PaymentPlan::default();
    if solver.solve(&reqs, 0, &mut used, &mut plan, ColorSet::NONE) {
        plan.pool_indices.sort_unstable();
        Some(plan)
    } else {
        None
    }
}

fn matches_color(m: &Mana, c: Color, any: &[ManaType]) -> bool {
    m.ty.color() == Some(c) || any.contains(&m.ty)
}

/// The backtracking search behind [`find_payment_with`].
struct PoolSolver<'a> {
    pool: &'a [Mana],
    usable: &'a [bool],
    /// Mana types that may be spent as though they were mana of any color.
    any: &'a [ManaType],
    life_for: &'a [(Color, u32)],
    max_life: u32,
    /// Each mana paying X is of a different color.
    x_distinct: bool,
}

impl PoolSolver<'_> {
    /// Pays `reqs[i..]`. `x_used`: the colors of the mana paying X so far.
    fn solve(
        &self,
        reqs: &[Req],
        i: usize,
        used: &mut Vec<bool>,
        plan: &mut PaymentPlan,
        x_used: ColorSet,
    ) -> bool {
        if i == reqs.len() {
            return true;
        }
        let (pool, any) = (self.pool, self.any);
        let req = reqs[i];
        // Generic requirements are all at the end: just count free mana that may pay
        // generic mana.
        if let Req::Generic = req {
            let need = reqs.len() - i;
            let mut free: Vec<usize> = (0..pool.len())
                .filter(|&j| self.usable[j] && !used[j] && pool[j].pays_generic())
                .collect();
            if free.len() < need {
                return false;
            }
            // Prefer spending colorless, then the most plentiful colors first to keep
            // options (mana that triggers when it's spent before other mana of its type).
            free.sort_by_key(|&j| {
                (
                    pool[j].ty != ManaType::C,
                    pool[j].restriction.is_none(),
                    pool[j].rider.is_none(),
                )
            });
            for &j in free.iter().take(need) {
                used[j] = true;
                plan.pool_indices.push(j);
            }
            return true;
        }
        let rest = |used: &mut Vec<bool>, plan: &mut PaymentPlan| {
            self.solve(reqs, i + 1, used, plan, x_used)
        };
        let paid = match req {
            Req::Colored(c) | Req::Phyrexian(c) => {
                self.try_unit(&|m| matches_color(m, c, any), used, plan, &rest)
            }
            Req::Colorless => self.try_unit(&|m| m.ty == ManaType::C, used, plan, &rest),
            // {S} is generic mana that mana from a snow source pays (CR 107.4h).
            Req::Snow => self.try_unit(&|m| m.snow && m.pays_generic(), used, plan, &rest),
            Req::XOf(colors) => self.pay_x(reqs, i, colors, used, plan, x_used),
            Req::Hybrid(a, b) | Req::PhyrexianHybrid(a, b) => self.try_unit(
                &|m| matches_color(m, a, any) || matches_color(m, b, any),
                used,
                plan,
                &rest,
            ),
            Req::ColorlessHybrid(c) => self.try_unit(
                &|m| matches_color(m, c, any) || m.ty == ManaType::C,
                used,
                plan,
                &rest,
            ),
            Req::TwoHybrid(c) => {
                self.try_unit(&|m| matches_color(m, c, any), used, plan, &rest) || {
                    // Pay two generic instead: splice two Generic reqs at the end.
                    let mut more: Vec<Req> = reqs[i + 1..].to_vec();
                    more.push(Req::Generic);
                    more.push(Req::Generic);
                    more.sort_by_key(|r| r.rank());
                    self.solve(&more, 0, used, plan, x_used)
                }
            }
            Req::Generic => unreachable!(),
        };
        if paid {
            return true;
        }
        // Life instead of mana.
        let Some((life, phyrexian)) = req.life(self.life_for) else {
            return false;
        };
        if self.max_life < plan.life + life {
            return false;
        }
        plan.life += life;
        plan.phyrexian += phyrexian as u32;
        if rest(used, plan) {
            return true;
        }
        plan.life -= life;
        plan.phyrexian -= phyrexian as u32;
        false
    }

    /// Pays the requirement with one unit of mana `pred` accepts, then the rest (`next`).
    fn try_unit(
        &self,
        pred: &dyn Fn(&Mana) -> bool,
        used: &mut Vec<bool>,
        plan: &mut PaymentPlan,
        next: &dyn Fn(&mut Vec<bool>, &mut PaymentPlan) -> bool,
    ) -> bool {
        let pool = self.pool;
        // Try restricted mana first (it's less flexible elsewhere), then mana that
        // triggers when it's spent (CR 106.6: its player made it to spend).
        let mut cands: Vec<usize> = (0..pool.len())
            .filter(|&j| self.usable[j] && !used[j] && pred(&pool[j]))
            .collect();
        cands.sort_by_key(|&j| (pool[j].restriction.is_none(), pool[j].rider.is_none()));
        let mut seen_plain: Vec<ManaType> = Vec::new();
        for j in cands {
            // Symmetry breaking: unrestricted, non-snow mana of the same type with no
            // trigger is interchangeable.
            if pool[j].restriction.is_none() && !pool[j].snow && pool[j].rider.is_none() {
                if seen_plain.contains(&pool[j].ty) {
                    continue;
                }
                seen_plain.push(pool[j].ty);
            }
            used[j] = true;
            plan.pool_indices.push(j);
            if next(used, plan) {
                return true;
            }
            plan.pool_indices.pop();
            used[j] = false;
        }
        false
    }

    /// Pays the generic mana `reqs[i]` of X with mana of one of `colors` (one not used for
    /// X yet, if each must differ), then the rest.
    fn pay_x(
        &self,
        reqs: &[Req],
        i: usize,
        colors: ColorSet,
        used: &mut Vec<bool>,
        plan: &mut PaymentPlan,
        x_used: ColorSet,
    ) -> bool {
        for c in Color::ALL {
            if !colors.contains(c) || (self.x_distinct && x_used.contains(c)) {
                continue;
            }
            let mut now = x_used;
            now.insert(c);
            if self.try_unit(
                &|m| m.pays_generic() && matches_color(m, c, self.any),
                used,
                plan,
                &|u, p| {
                    let j = p.pool_indices.last().copied();
                    p.x_indices.extend(j);
                    if self.solve(reqs, i + 1, u, p, now) {
                        return true;
                    }
                    if j.is_some() {
                        p.x_indices.pop();
                    }
                    false
                },
            ) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(types: &[ManaType]) -> Vec<Mana> {
        types.iter().map(|t| Mana::new(*t)).collect()
    }

    #[test]
    fn parse_and_value() {
        let c = ManaCost::parse("{2}{W}{U}").unwrap();
        assert_eq!(c.mana_value(), 4);
        assert_eq!(c.colors().count(), 2);
        let c = ManaCost::parse("{X}{R}{R}").unwrap();
        assert_eq!(c.mana_value(), 2);
        assert_eq!(c.mana_value_with_x(3), 5);
        let c = ManaCost::parse("{2/W}{W/U/P}{G/P}{C/R}").unwrap();
        assert_eq!(c.mana_value(), 5);
        assert_eq!(c.to_string(), "{2/W}{W/U/P}{G/P}{C/R}");
        assert!(ManaCost::parse("").is_none());
    }

    #[test]
    fn pay_basic() {
        use ManaType::*;
        let c = ManaCost::parse("{1}{W}{W}").unwrap();
        let ctx = SpendContext::default();
        assert!(find_payment(&pool(&[W, W, G]), &c, &ctx, 0).is_some());
        assert!(find_payment(&pool(&[W, G, G]), &c, &ctx, 0).is_none());
        let h = ManaCost::parse("{W/U}{W/U}").unwrap();
        assert!(find_payment(&pool(&[W, U]), &h, &ctx, 0).is_some());
        let two = ManaCost::parse("{2/R}").unwrap();
        assert!(find_payment(&pool(&[G, G]), &two, &ctx, 0).is_some());
        assert!(find_payment(&pool(&[G]), &two, &ctx, 0).is_none());
        let phy = ManaCost::parse("{1}{B/P}").unwrap();
        let p = find_payment(&pool(&[G]), &phy, &ctx, 20).unwrap();
        assert_eq!(p.life, 2);
        let colorless = ManaCost::parse("{C}").unwrap();
        assert!(find_payment(&pool(&[G]), &colorless, &ctx, 0).is_none());
        assert!(find_payment(&pool(&[C]), &colorless, &ctx, 0).is_some());
    }

    #[test]
    fn restricted_mana() {
        let mut m = Mana::new(ManaType::G);
        m.restriction = Some(ManaRestriction::SpellOfType(CardType::Creature));
        let c = ManaCost::parse("{1}").unwrap();
        let creature = SpendContext {
            is_spell: true,
            card_types: CardTypeSet::single(CardType::Creature),
            ..Default::default()
        };
        let sorcery = SpendContext {
            is_spell: true,
            card_types: CardTypeSet::single(CardType::Sorcery),
            ..Default::default()
        };
        assert!(find_payment(&[m.clone()], &c, &creature, 0).is_some());
        assert!(find_payment(&[m], &c, &sorcery, 0).is_none());
    }
}
