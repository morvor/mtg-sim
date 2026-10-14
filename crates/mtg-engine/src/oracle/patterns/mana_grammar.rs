//! Mana grammar (CR 106): spell phrases in spending restrictions, mana production variants,
//! and rules about which mana may pay a spell's cost.
//!
//! - Spell phrases after "Spend this mana only to cast" (see `mana_restrictions.rs` for the
//!   purposes): "kicked spells" (CR 702.33d), "Dwarf, Equipment, and Saga spells" (a list
//!   joined by "and": any of them), "a Chandra planeswalker spell" (a subtype before a card
//!   type), "spells that have foretell", "a spell that's one or more colors without {X} in
//!   its mana cost".
//! - Production: "Add two mana of different colors", "Add two mana of any one color and two
//!   mana of any other color" ([`ManaProduction::DistinctColors`]); "Add one mana of any
//!   color among legendary permanents you control"; "Add one mana of any of the exiled
//!   card's colors" (CR 106.5: none if it has no color).
//! - "Spend only mana produced by basic lands to cast ~." ([`CostRule::OnlyManaFrom`]).

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

use super::mana_restrictions::spell_alternatives;

/// Spell phrases the core object-phrase parser doesn't read (see the module docs).
/// `s` has no leading article.
pub(crate) fn spell_phrase(s: &str) -> Option<Filter> {
    let s = s.trim();
    // "kicked spells", "a kicked spell" (CR 702.33d: the spell's controller declared the
    // intention to pay a kicker cost).
    if let Some(r) = s.strip_prefix("kicked ") {
        let f = if matches!(r, "spell" | "spells") {
            Filter::Spell
        } else {
            spell_alternatives(r)?
        };
        return Some(Filter::and(vec![
            f,
            Filter::CastWithCost(crate::kw::kicker::KICKED.into()),
        ]));
    }
    // "spells that have foretell": a keyword the spell has.
    for p in [" that have ", " that has "] {
        if let Some(i) = s.find(p) {
            let k = crate::keywords::KeywordKind::from_name(&s[i + p.len()..])?;
            let f = spell_alternatives(&s[..i])?;
            return Some(Filter::and(vec![f, Filter::HasKeyword(k)]));
        }
    }
    // "... without {X} in its mana cost" (Titans' Nest's ruling: an {X} in an additional
    // or alternative cost doesn't matter).
    for p in [
        " without {x} in its mana cost",
        " without {x} in their mana costs",
    ] {
        if let Some(head) = s.strip_suffix(p) {
            let f = spell_alternatives(head)?;
            return Some(Filter::and(vec![f, Filter::not(Filter::HasX)]));
        }
    }
    // "Dwarf, Equipment, and Saga spells", "instant, sorcery, Demon, and Spirit spells":
    // a spell that's any of them.
    if let Some(head) = s.strip_suffix(" spells") {
        if head.contains(", ") {
            let items: Vec<&str> = head.split(", ").collect();
            let (last, rest) = items.split_last()?;
            let last = last
                .strip_prefix("and ")
                .or_else(|| last.strip_prefix("or "))?;
            let mut out = Vec::new();
            for w in rest.iter().copied().chain(std::iter::once(last)) {
                if w.contains(' ') {
                    return None;
                }
                out.push(spell_alternatives(&format!("{w} spells"))?);
            }
            return Some(Filter::Or(out));
        }
    }
    None
}

/// "Chandra planeswalker spell": a subtype before the card type it belongs to.
pub(crate) fn subtype_spell(s: &str) -> Option<Filter> {
    let (w, rest) = s.split_once(' ')?;
    if CardType::from_word(w).is_some() || adjective(w).is_some() {
        return None;
    }
    let sub = subtype_word(w)?;
    let (t, _) = rest.split_once(' ')?;
    CardType::from_word(t)?;
    let f = spell_alternatives(rest)?;
    Some(Filter::and(vec![Filter::Subtype(sub), f]))
}

/// A number of mana: "one", "two", ...
fn count_word(s: &str) -> Option<(u32, &str)> {
    let (v, rest) = parse_number(s)?;
    let n = v.as_const()?;
    (n > 0).then_some((n as u32, rest))
}

/// Production phrases after "add " (see the module docs).
fn production(r: &str) -> Option<ManaProduction> {
    let r = end(r).trim();
    // "two mana of different colors".
    if let Some((n, rest)) = count_word(r) {
        if rest.trim() == "mana of different colors" && (2..=5).contains(&n) {
            return Some(ManaProduction::DistinctColors(n, 1));
        }
        // "two mana of any one color and two mana of any other color".
        if let Some(x) = rest.trim().strip_prefix("mana of any one color and ") {
            let (m, rest) = count_word(x)?;
            if m == n && rest.trim() == "mana of any other color" {
                return Some(ManaProduction::DistinctColors(2, n));
            }
        }
    }
    // "one mana of any color among legendary permanents you control".
    if let Some(x) = r.strip_prefix("one mana of any color among ") {
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).trim().is_empty() {
            return None;
        }
        return Some(ManaProduction::AnyColorAmong(f));
    }
    // "one mana of any of this creature's colors" (granted by Katilda, Dawnhart Prime):
    // the object with the ability.
    if r == "one mana of any of ~'s colors" {
        return Some(ManaProduction::AnyColorAmong(Filter::Source));
    }
    // "one mana of any of the exiled card's colors": the cards the source's linked ability
    // exiled (CR 607.2a; CR 106.5: no mana if they're colorless).
    if matches!(
        r,
        "one mana of any of the exiled card's colors" | "one mana of any of the exiled cards' colors"
    ) {
        return Some(ManaProduction::AnyColorAmong(Filter::and(vec![
            Filter::In(Box::new(Sel::Linked)),
            Filter::InZone(ZoneKind::Exile),
        ])));
    }
    None
}

/// "add [production]" for the productions above.
fn add_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("add ")?;
    let mana = production(r)?;
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana,
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "mana grammar: add two mana of different colors / any color among / the exiled card's colors", priority: 60, parse: add_mana } }

/// "Each player adds {B}{R}{G}." (Yurlok of Scorch Thrash), "the active player adds
/// {C}{C} for each ..." (Belbe, Corrupted Observer): mana of a fixed type added to other
/// players' pools (no choice for anyone to make).
fn players_add(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("each player adds ") {
        (None, r)
    } else if let Some(r) = l.strip_prefix("the active player adds ") {
        (Some(PlayerRef::ActivePlayer), r)
    } else {
        return None;
    };
    // "for each of your opponents who lost life this turn".
    let r = r.replace("for each of your opponents who ", "for each opponent who ");
    // "{C}{C} for each ...": twice "{C} for each ...".
    let (r, times) = match r.split_once(" for each ") {
        Some((syms, each)) if syms.len() > 3 && syms.len() % 3 == 0 => {
            let first = &syms[..3];
            let n = syms.len() / 3;
            if syms != first.repeat(n) {
                return None;
            }
            (format!("{first} for each {each}"), n as i64)
        }
        _ => (r, 1),
    };
    let saved = b.targets.len();
    let add = crate::oracle::effects::parse_sentence(&format!("add {r}"), b);
    let add = match add {
        Some(Effect::AddMana {
            who,
            mana: ManaProduction::Amount(t, v),
            restriction,
        }) if times > 1 => Some(Effect::AddMana {
            who,
            mana: ManaProduction::Amount(t, Value::Mul(Box::new(Value::c(times as i32)), Box::new(v))),
            restriction,
        }),
        Some(_) if times > 1 => None,
        other => other,
    };
    let (mana, restriction) = match add {
        Some(Effect::AddMana {
            who: PlayerRef::You,
            mana: mana @ (ManaProduction::Fixed(_) | ManaProduction::Amount(..)),
            restriction,
        }) if b.targets.len() == saved => (mana, restriction),
        _ => {
            b.targets.truncate(saved);
            return None;
        }
    };
    Some(match who {
        Some(who) => Effect::AddMana {
            who,
            mana,
            restriction,
        },
        None => Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::AddMana {
                who: PlayerRef::Iterated,
                mana,
                restriction,
            }),
        },
    })
}

inventory::submit! { EffectPattern { name: "mana grammar: each player adds / the active player adds [mana]", priority: 60, parse: players_add } }

/// "{T}: Choose a color. Add one mana of that color for each different power among
/// creatures you control." (Selvala, Eager Trailblazer; Astral Cornucopia: "... for each
/// charge counter on ~"): that many mana of the one color chosen.
fn choose_color_add_for_each(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost_s, eff) = crate::oracle::split_cost(block.trim())?;
    let eff = eff.to_lowercase();
    let r = end(eff.trim()).strip_prefix("choose a color. add one mana of that color for each ")?;
    let n = super::statics::parse_for_each(r, None)?;
    let (cost, _) = crate::oracle::costs::parse_cost(cost_s)?;
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::AnyOneColor(n),
            restriction: None,
        }),
    );
    act.is_mana_ability = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "mana grammar: choose a color. add one mana of that color for each ...", priority: 60, parse: choose_color_add_for_each } }

/// "Mana of any type can be spent to activate ~'s abilities." (Sharkey, Tyrant of the
/// Shire; CR 118.14, 609.4b): each mana symbol of their costs can be paid with mana of any
/// type.
fn any_type_for_own_abilities(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "mana of any type can be spent to activate ~'s abilities" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Abilities(Filter::Source),
                who: PlayerRel::Any,
                change: CostChange::SpendAnyType,
            },
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "mana grammar: mana of any type can be spent to activate ~'s abilities", priority: 60, parse: any_type_for_own_abilities } }

/// "Spend only mana produced by basic lands to cast ~." / "... by creatures ...": only
/// mana from such permanents may pay the spell's total cost (Myr Superion's ruling: mana
/// from abilities of creature cards not on the battlefield can't).
fn only_mana_from(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.trim().to_lowercase();
    let r = end(&lower).strip_prefix("spend only mana produced by ")?;
    let r = r
        .strip_suffix(" to cast ~")
        .or_else(|| r.strip_suffix(" to cast this spell"))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if !plural || !end(tail).trim().is_empty() {
        return None;
    }
    let f = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::Rule(CostRule::OnlyManaFrom(crate::mana::SpendFilter::new(f))),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "mana grammar: spend only mana produced by [permanents] to cast ~", priority: 80, parse: only_mana_from } }

/// "A player losing unspent mana causes that player to lose that much life." (Yurlok of
/// Scorch Thrash; see `kw/mana_grammar.rs`).
fn mana_burn(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "a player losing unspent mana causes that player to lose that much life" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected: PlayerFilter::Any,
            effect: PlayerModification::Custom(crate::kw::mana_grammar::MANA_BURN.into()),
        })),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "mana grammar: a player losing unspent mana loses that much life", priority: 60, parse: mana_burn } }

/// "All lands lose all abilities except mana abilities." (Blood Sun; layer 6).
fn lose_all_but_mana_abilities(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let subject = l
        .strip_suffix(" lose all abilities except mana abilities")
        .or_else(|| l.strip_suffix(" loses all abilities except mana abilities"))?;
    let subject = subject.strip_prefix("all ").unwrap_or(subject);
    let (f, _, tail) = parse_object_phrase(subject)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    let affected = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: vec![Modification::Custom {
                name: crate::kw::mana_grammar::EXCEPT_MANA_ABILITIES.into(),
                layer: Layer::L6Ability,
            }],
        })),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "mana grammar: [permanents] lose all abilities except mana abilities", priority: 60, parse: lose_all_but_mana_abilities } }

/// "that is snow or could produce {C}", "that could produce {R}" after a noun (CR 106.7).
fn could_produce_suffix<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that ")?;
    let (head, rest) = match r.find("could produce {") {
        Some(i) => (&r[..i], &r[i + "could produce ".len()..]),
        None => return None,
    };
    let sym = rest.strip_prefix('{')?;
    let (inner, rest) = sym.split_once('}')?;
    let mut cs = inner.chars();
    let c = cs.next()?.to_ascii_uppercase();
    if cs.next().is_some() {
        return None;
    }
    crate::mana::ManaType::from_letter(c)?;
    let could = Filter::Custom(format!("{}{c}", crate::kw::mana_grammar::COULD_PRODUCE).into());
    let f = match head.trim() {
        "" => could,
        h => {
            let adj = h.strip_prefix("is ")?.strip_suffix(" or")?;
            Filter::Or(vec![adjective(adj)?, could])
        }
    };
    Some((f, rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "mana grammar: that could produce {C}", priority: 60, parse: could_produce_suffix } }

/// "Whenever you cast a spell, if at least seven mana was spent to cast it, ..."
/// (Raggadragga, Goreguts Boss): the spell that caused the ability to trigger, as in "if
/// seven or more mana was spent to cast that spell" (see `opus.rs`).
fn cast_trigger_mana_spent(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') || !block.starts_with("Whenever ") {
        return None;
    }
    let i = block.find(", if at least ")?;
    let head = &block[..i];
    if !head.contains(" cast") {
        return None;
    }
    let r = &block[i + ", if at least ".len()..];
    let (n, rest) = r.split_once(' ')?;
    let rest = rest.strip_prefix("mana was spent to cast it, ")?;
    let rewritten = format!("{head}, if {n} or more mana was spent to cast that spell, {rest}");
    let out = crate::oracle::parse_ability(&rewritten, ctx)?;
    let ok = out.iter().all(|a| {
        matches!(&a.kind, AbilityKind::Triggered(t) if t.intervening_if.is_some())
    });
    ok.then(|| {
        out.into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), block))
            .collect()
    })
}

inventory::submit! { AbilityPattern { name: "mana grammar: whenever you cast ..., if at least N mana was spent to cast it", priority: 60, parse: cast_trigger_mana_spent } }

/// "[cost]: [effect]. Spend only mana of the chosen color to activate this ability."
/// (Throne of Eldraine): only mana of the color chosen for the source may pay the cost's
/// generic mana ([`CostRule::OnlyChosenColor`]). Only costs whose mana is all generic.
fn only_chosen_color_to_activate(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let head = block
        .trim_end()
        .strip_suffix(" Spend only mana of the chosen color to activate this ability.")?;
    if block.contains('\n') || !ctx.is_permanent() {
        return None;
    }
    let out = crate::oracle::parse_ability(head, ctx)?;
    let [a] = out.as_slice() else {
        return None;
    };
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let mut act = act.clone();
    let m = act.cost.mana.as_ref()?;
    if m.symbols.is_empty()
        || !m
            .symbols
            .iter()
            .all(|s| matches!(s, crate::mana::ManaSymbol::Generic(_)))
    {
        return None;
    }
    act.own_cost_changes.push(OwnCostChange {
        change: CostChange::Rule(CostRule::OnlyChosenColor),
        condition: None,
    });
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "mana grammar: spend only mana of the chosen color to activate this ability", priority: 60, parse: only_chosen_color_to_activate } }

#[cfg(test)]
mod probe {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    fn compiled(ty: &str, text: &str) -> Option<String> {
        let tl = TypeLine::parse(ty);
        let ctx = CompileContext {
            card_name: "Test Card",
            full_name: "Test Card",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let r = compile(text, &ctx);
        r.unsupported.is_empty().then(|| format!("{:?}", r.abilities))
    }

    #[test]
    fn probe() {
        for r in [
            "spend this mana only to cast kicked spells.",
            "spend this mana only to cast dwarf, equipment, and saga spells.",
            "spend this mana only to cast instant, sorcery, demon, and spirit spells.",
            "spend this mana only to cast an elemental spell or a chandra planeswalker spell.",
            "spend this mana only to cast a dragon spell or an omen spell.",
            "spend this mana only to cast an instant or sorcery spell or a kicked spell.",
            "spend this mana only to cast equipment spells or activate equip abilities.",
            "spend this mana only to cast an equipment spell or activate an equip ability.",
            "spend this mana only to activate power-up abilities.",
            "spend this mana only to cast room spells and unlock doors.",
            "spend this mana only to pay cumulative upkeep costs.",
            "spend this mana only to foretell cards or cast spells that have foretell.",
            "spend this mana only to foretell a card from your hand or cast an instant or sorcery spell.",
            "spend this mana only to pay a disturb cost or cast an instant or sorcery spell.",
            "spend this mana only to cast a colorless spell, activate an ability of a colorless permanent, or pay a cost that contains {c}.",
            "spend this mana only to cast a spell that's one or more colors without {x} in its mana cost.",
            "spend this mana only to cast a creature spell of the chosen type or a legendary creature spell.",
            "spend this mana only to cast a spell of the chosen type or activate an ability of a source of the chosen type.",
            "spend this mana only to cast a creature spell of the chosen type or activate an ability of a creature source of the chosen type.",
            "spend this mana only to cast monocolored spells of that color.",
        ] {
            println!("{r} => {:?}", crate::oracle::patterns::r106_mana::parse_restriction(r));
        }
        for (ty, t) in [
            ("Artifact", "{T}: Add {C}{C} for each opponent who lost life this turn."),
            ("Artifact", "{T}: Add {C} for each opponent who lost life this turn."),
            ("Artifact", "Whenever a player casts a spell, any other player may pay that spell's mana cost."),
            ("Artifact", "Whenever a player casts a spell, any player may pay {1}. If a player does, counter the spell."),
            ("Artifact", "Whenever a player casts a spell, any player may pay {1}. If a player does, counter that spell."),
            ("Artifact", "Whenever a player casts a spell, counter the spell."),
            ("Artifact", "{T}: Add two mana of different colors."),
            ("Artifact", "{T}: Add one mana of any color among legendary permanents you control."),
            ("Artifact", "{T}: Add one mana of any color among legendary creatures and planeswalkers you control."),
            ("Land", "{T}: Add one mana of any color among legendary creature cards in your graveyard."),
            ("Creature", "When ~ enters, choose a player. That player adds two mana of any one color they choose."),
            ("Artifact", "{1}, {T}: Each player adds {B}{R}{G}."),
            ("Artifact", "{T}: Add one mana of any of the exiled card's colors."),
            ("Artifact", "Imprint — When ~ enters, you may exile a nonartifact, nonland card from your hand.\n{T}: Add one mana of any of the exiled card's colors."),
            ("Artifact", "{T}: Choose a color. Add one mana of that color for each charge counter on ~."),
            ("Creature", "{T}: Choose a color. Add one mana of that color for each different power among creatures you control."),
            ("Creature", "At the beginning of each postcombat main phase, the active player adds {C}{C} for each of your opponents who lost life this turn."),
            ("Creature", "Spend only mana produced by basic lands to cast ~."),
            ("Sorcery", "Choose one —\n• Add two mana of any one color and two mana of any other color. Spend this mana only to cast creature or enchantment spells.\n• Creatures you control get +1/+1 until end of turn."),
            ("Artifact", "{T}: Add four mana of the chosen color. Spend this mana only to cast monocolored spells of that color."),
            ("Artifact", "{3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability."),
            ("Creature", "Mana of any type can be spent to activate ~'s abilities."),
            ("Creature", "~ has flying and indestructible as long as you have six or more unspent mana."),
            ("Enchantment", "All lands lose all abilities except mana abilities."),
            ("Sorcery", "Destroy target land that is snow or could produce {C}."),
            ("Creature", "~'s power and toughness are each equal to the number of black mana symbols in the mana costs of cards in your graveyard."),
            ("Sorcery", "Target creature you control fights target creature an opponent controls. If the creature the opponent controls is dealt excess damage this way, add that much {R}."),
            ("Enchantment", "Sacrifice a land: Add one mana of any type the sacrificed land could produce."),
            ("Creature", "Whenever you cast a spell, if at least seven mana was spent to cast it, untap target creature. It gets +7/+7 and gains trample until end of turn."),
            ("Artifact", "A player losing unspent mana causes that player to lose that much life."),
            ("Enchantment", "Cumulative upkeep {1}\nWhenever a player casts a spell, any other player may pay that spell's mana cost. If a player does, counter the spell."),
            ("Creature", "Whenever ~ deals combat damage to a player, look at the top card of that player's library, then exile it face down. You may play it for as long as it remains exiled. Mana of any type can be spent to cast a spell this way."),
            ("Instant", "Put a +1/+1 counter on target creature you control, then it fights target creature you don't control.\nAdamant — If at least three green mana was spent to cast ~, the creature you control gains indestructible until end of turn."),
        ] {
            println!("{t} => {:?}", compiled(ty, t));
        }
    }
}
