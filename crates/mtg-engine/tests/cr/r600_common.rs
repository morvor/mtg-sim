//! Shared helpers for the CR 600–608 tests: small builders for custom card definitions
//! (rules-level objects) and for compiling oracle text snippets.

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::Layout;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaCost;
use mtg_engine::object::Characteristics;
use mtg_engine::oracle;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Builder for a custom card definition.
pub struct CB(pub Characteristics);

impl CB {
    pub fn new(name: &str) -> CB {
        CB(Characteristics {
            name: SmolStr::new(name),
            rules_text: Arc::from(""),
            ..Default::default()
        })
    }
    pub fn types(mut self, ts: &[CardType]) -> CB {
        for t in ts {
            self.0.card_types.insert(*t);
        }
        self
    }
    pub fn subtypes(mut self, ss: &[&str]) -> CB {
        for s in ss {
            self.0.subtypes.push(SmolStr::new(s));
        }
        self
    }
    pub fn creature(self, p: i32, t: i32) -> CB {
        let mut s = self.types(&[CardType::Creature]);
        s.0.power = Some(p);
        s.0.toughness = Some(t);
        s
    }
    pub fn artifact(self) -> CB {
        self.types(&[CardType::Artifact])
    }
    pub fn enchantment(self) -> CB {
        self.types(&[CardType::Enchantment])
    }
    pub fn instant(self) -> CB {
        self.types(&[CardType::Instant])
    }
    pub fn sorcery(self) -> CB {
        self.types(&[CardType::Sorcery])
    }
    pub fn land(self) -> CB {
        self.types(&[CardType::Land])
    }
    pub fn planeswalker(mut self, loyalty: i32) -> CB {
        self.0.card_types.insert(CardType::Planeswalker);
        self.0.loyalty = Some(loyalty);
        self
    }
    /// Sets the mana cost and derives the colors from it.
    pub fn cost(mut self, c: &str) -> CB {
        let m = ManaCost::parse(c).expect("bad mana cost");
        self.0.colors = m.colors();
        self.0.mana_cost = Some(m);
        self
    }
    pub fn colors(mut self, cs: &[Color]) -> CB {
        self.0.colors = ColorSet::NONE;
        for c in cs {
            self.0.colors.insert(*c);
        }
        self
    }
    pub fn ability(mut self, a: Ability) -> CB {
        self.0.abilities.push(a);
        self
    }
    pub fn keyword(self, k: KeywordKind) -> CB {
        self.ability(kw(k))
    }
    pub fn spell(self, body: Body) -> CB {
        self.ability(spell(body))
    }
    pub fn build(self) -> CardDef {
        CardDef::custom(self.0)
    }
}

pub fn kw(k: KeywordKind) -> Ability {
    AbilityDef::new(
        AbilityKind::Keyword(keywords::Keyword::new(k)),
        k.name(),
    )
}

pub fn spell(body: Body) -> Ability {
    AbilityDef::new(AbilityKind::Spell(SpellAbility { body }), "spell")
}

pub fn trig(cond: TriggerCond, body: Body) -> Ability {
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(cond, body)),
        "triggered",
    )
}

pub fn trig_from(t: TriggeredAbility) -> Ability {
    AbilityDef::new(AbilityKind::Triggered(t), "triggered")
}

pub fn act(cost: Cost, body: Body) -> Ability {
    AbilityDef::new(
        AbilityKind::Activated(ActivatedAbility::new(cost, body)),
        "activated",
    )
}

pub fn act_from(a: ActivatedAbility) -> Ability {
    AbilityDef::new(AbilityKind::Activated(a), "activated")
}

pub fn stat(effect: StaticEffect) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        "static",
    )
}

pub fn stat_from(s: StaticAbility) -> Ability {
    AbilityDef::new(AbilityKind::Static(s), "static")
}

pub fn mana(s: &str) -> ManaCost {
    ManaCost::parse(s).expect("bad mana cost")
}

pub fn mana_cost(s: &str) -> Cost {
    Cost::mana(mana(s))
}

pub fn gain(n: i32) -> Effect {
    Effect::GainLife {
        who: PlayerRef::You,
        n: Value::c(n),
    }
}

pub fn draw(n: i32) -> Effect {
    Effect::Draw {
        who: PlayerRef::You,
        n: Value::c(n),
    }
}

pub fn damage_target(n: i32) -> Body {
    Body::simple(
        vec![TargetSpec::any_target()],
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
    )
}

pub fn target_creature() -> TargetSpec {
    TargetSpec::object(Filter::creature(), "target creature")
}

/// Compiles oracle text for a card with the given type line into a custom definition,
/// exactly as the card database would (CR 113 ability kinds are decided by the compiler).
pub fn compile_def(name: &str, type_line: &str, cost: &str, text: &str) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let ctx = oracle::CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    let m = ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// The abilities of a definition (front face).
pub fn abilities(def: &CardDef) -> Vec<Ability> {
    def.faces[0].chars.abilities.clone()
}

/// The uid of the `i`th activated ability of an object.
pub fn activated_uid(t: &testing::TestGame, obj: ObjectId, i: usize) -> u64 {
    t.obj(obj)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(i)
        .map(|a| a.uid)
        .expect("no such activated ability")
}
