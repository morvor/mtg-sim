//! Costs (CR 118): unpayable costs (CR 118.6), cost reductions by mana symbols
//! (CR 118.7a–g), choosing how to pay symbols that can be paid in more than one way
//! (CR 118.13), and casting "if able" (CR 118.8c).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::{ManaCost, ManaSymbol};
use crate::types::*;
use smallvec::smallvec;

/// Keys in a spell's `chosen_values` recording which half of each hybrid mana symbol of a
/// cost reduction the player chose (CR 118.7e): `REDUCTION_HALF_KEY + i` for the i-th
/// hybrid symbol among the reductions that apply.
pub const REDUCTION_HALF_KEY: u16 = 0x7e00;

/// The mana cost of an object with no mana cost: an unpayable cost (CR 118.6). No
/// payment can satisfy it, and increasing it or adding to it leaves it unpayable
/// (CR 118.6a).
pub fn unpayable() -> ManaCost {
    ManaCost {
        symbols: smallvec![ManaSymbol::Infinity],
    }
}

/// A half of a hybrid mana symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Color(Color),
    Colorless,
    /// The generic half of a {2/W}-style symbol.
    Generic(u32),
}

/// The halves of a hybrid (or Phyrexian hybrid) mana symbol, if it is one.
pub fn halves(s: ManaSymbol) -> Option<[Half; 2]> {
    Some(match s {
        ManaSymbol::Hybrid(a, b) | ManaSymbol::PhyrexianHybrid(a, b) => {
            [Half::Color(a), Half::Color(b)]
        }
        ManaSymbol::TwoHybrid(c) => [Half::Color(c), Half::Generic(2)],
        ManaSymbol::ColorlessHybrid(c) => [Half::Colorless, Half::Color(c)],
        _ => return None,
    })
}

fn reduce_one_colored(cost: &mut ManaCost, c: Color, colored_only: bool) {
    if let Some(i) = cost
        .symbols
        .iter()
        .position(|s| *s == ManaSymbol::Colored(c))
    {
        cost.symbols.remove(i);
    } else if !colored_only {
        // CR 118.7b, 118.7c: the generic component is reduced instead.
        cost.reduce_generic(1);
    }
}

fn reduce_one_colorless(cost: &mut ManaCost, colored_only: bool) {
    if let Some(i) = cost
        .symbols
        .iter()
        .position(|s| *s == ManaSymbol::Colorless)
    {
        cost.symbols.remove(i);
    } else if !colored_only {
        // CR 118.7b, 118.7d.
        cost.reduce_generic(1);
    }
}

/// The half of a hybrid reduction symbol that reduces `cost` the most, used when the
/// player hasn't chosen.
pub fn default_half(cost: &ManaCost, s: ManaSymbol) -> Half {
    let [a, b] = halves(s).expect("hybrid symbol");
    let useful = |h: Half| match h {
        Half::Color(c) => cost.symbols.contains(&ManaSymbol::Colored(c)),
        Half::Colorless => cost.symbols.contains(&ManaSymbol::Colorless),
        Half::Generic(n) => cost.generic_amount() >= n,
    };
    if useful(a) || !useful(b) {
        a
    } else {
        b
    }
}

/// Reduces `cost` by the mana symbols of `by` (CR 118.7a–g). `half` gives the half the
/// player chose for the i-th hybrid symbol of `by` (CR 118.7e). With `colored_only`
/// ("This effect reduces only the amount of colored mana you pay"), colored reductions
/// don't reduce the generic component.
pub fn reduce_by(
    cost: &mut ManaCost,
    by: &ManaCost,
    colored_only: bool,
    mut half: impl FnMut(usize, &ManaCost, ManaSymbol) -> Half,
) {
    let mut hybrid_index = 0;
    for s in by.symbols.iter().copied() {
        match s {
            // CR 118.7a: generic reductions affect only the generic component.
            ManaSymbol::Generic(n) => cost.reduce_generic(n),
            // CR 118.7b, 118.7c; CR 118.7f: a Phyrexian symbol reduces the cost by one
            // mana of its color.
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => {
                reduce_one_colored(cost, c, colored_only)
            }
            // CR 118.7b, 118.7d.
            ManaSymbol::Colorless => reduce_one_colorless(cost, colored_only),
            // CR 118.7g: snow symbols reduce the generic component.
            ManaSymbol::Snow => cost.reduce_generic(1),
            // CR 118.7e: the player chooses one half of a hybrid symbol.
            ManaSymbol::Hybrid(..)
            | ManaSymbol::TwoHybrid(_)
            | ManaSymbol::ColorlessHybrid(_)
            | ManaSymbol::PhyrexianHybrid(..) => {
                let h = half(hybrid_index, cost, s);
                hybrid_index += 1;
                match h {
                    Half::Color(c) => reduce_one_colored(cost, c, colored_only),
                    Half::Colorless => reduce_one_colorless(cost, colored_only),
                    Half::Generic(n) => cost.reduce_generic(n),
                }
            }
            ManaSymbol::X | ManaSymbol::Y | ManaSymbol::Z | ManaSymbol::Half(_) => {}
            ManaSymbol::Infinity => {}
        }
    }
}

fn half_label(h: Half) -> String {
    match h {
        Half::Color(c) => format!("{{{}}}", c.letter()),
        Half::Colorless => "{C}".into(),
        Half::Generic(n) => format!("{{{n}}}"),
    }
}

fn half_symbol(h: Half) -> ManaSymbol {
    match h {
        Half::Color(c) => ManaSymbol::Colored(c),
        Half::Colorless => ManaSymbol::Colorless,
        Half::Generic(n) => ManaSymbol::Generic(n),
    }
}

/// The hybrid reduction symbols that apply to casting `spell`, in the order
/// [`Game::base_total_cost`] applies them.
fn hybrid_reduction_symbols(g: &Game, p: PlayerId, spell: ObjectId) -> Vec<ManaSymbol> {
    let mut out = Vec::new();
    let o = g.obj(spell);
    let mut push = |m: &ManaCost| {
        out.extend(m.symbols.iter().copied().filter(|s| halves(*s).is_some()));
    };
    for a in &o.chars.abilities {
        if let AbilityKind::Static(s) = &a.kind {
            if let StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                change: CostChange::ReduceMana { mana, .. },
                ..
            }) = &s.effect
            {
                push(mana);
            }
        }
    }
    for (src, ctl, cm) in &g.statics.cost_modifiers {
        let ctx = Ctx::new(Some(*src), *ctl);
        if let (CostTarget::Spells(f), CostChange::ReduceMana { mana, .. }) =
            (&cm.applies_to, &cm.change)
        {
            if g.player_rel_matches(cm.who, p, &ctx)
                && g.matches(spell, &crate::casting::as_spell_filter(f), &ctx)
            {
                push(mana);
            }
        }
    }
    out
}

/// CR 118.7e, 601.2f: as the total cost of `spell` is determined, its controller chooses
/// a half of each hybrid mana symbol by which the cost is reduced. The choices are
/// recorded on the spell for [`Game::base_total_cost`].
pub fn choose_reduction_halves(g: &mut Game, p: PlayerId, spell: ObjectId) {
    for (i, s) in hybrid_reduction_symbols(g, p, spell)
        .into_iter()
        .enumerate()
    {
        let hs = halves(s).expect("hybrid symbol");
        let labels = hs.iter().map(|h| half_label(*h)).collect();
        let pick = g.ask_option(
            p,
            Some(spell),
            &format!("Cost reduction {s}: choose a half"),
            labels,
        );
        if let Some(si) = g.objects[spell.0 as usize].stack.as_mut() {
            si.chosen_values
                .insert(REDUCTION_HALF_KEY + i as u16, pick as i64);
        }
    }
}

/// The half of the i-th hybrid reduction symbol chosen for `spell` (CR 118.7e), or the
/// default choice.
pub fn chosen_half(g: &Game, spell: ObjectId, i: usize, cost: &ManaCost, s: ManaSymbol) -> Half {
    let chosen = g
        .obj(spell)
        .stack
        .as_deref()
        .and_then(|si| si.chosen_values.get(&(REDUCTION_HALF_KEY + i as u16)))
        .copied();
    match (chosen, halves(s)) {
        (Some(k), Some(hs)) if (k as usize) < 2 => hs[k as usize],
        _ => default_half(cost, s),
    }
}

/// CR 118.13a–c: for each mana symbol of a cost that can be paid in more than one way
/// (hybrid and Phyrexian symbols), the player chooses how they'll pay for it before
/// paying. The cost is rewritten accordingly; paying 2 life for a Phyrexian symbol
/// becomes a life payment. The first option leaves the choice to the automatic payment.
pub fn choose_payment_ways(g: &mut Game, p: PlayerId, source: Option<ObjectId>, cost: &mut Cost) {
    let Some(mana) = cost.mana.as_mut() else {
        return;
    };
    let mut life = 0;
    let mut out: Vec<ManaSymbol> = Vec::new();
    for s in mana.symbols.clone() {
        let (mut options, phyrexian): (Vec<Half>, bool) = match s {
            ManaSymbol::Phyrexian(c) => (vec![Half::Color(c)], true),
            ManaSymbol::PhyrexianHybrid(..) => (halves(s).unwrap().to_vec(), true),
            _ => match halves(s) {
                Some(hs) => (hs.to_vec(), false),
                None => {
                    out.push(s);
                    continue;
                }
            },
        };
        let mut labels = vec![format!("{s}: either way")];
        labels.extend(options.iter().map(|h| half_label(*h)));
        if phyrexian {
            labels.push("2 life".into());
        }
        let pick = g.ask_option(p, source, &format!("How will you pay {s}?"), labels);
        if pick == 0 {
            out.push(s);
        } else if pick <= options.len() {
            out.push(half_symbol(options.swap_remove(pick - 1)));
        } else {
            life += 2;
        }
    }
    let mut m = ManaCost::default();
    m.symbols = out.into_iter().collect();
    // Keep generic mana as a single symbol.
    let generic = m.generic_amount();
    m.symbols.retain(|s| !matches!(s, ManaSymbol::Generic(_)));
    if generic > 0 {
        m.symbols.insert(0, ManaSymbol::Generic(generic));
    }
    *mana = m;
    if life > 0 {
        cost.parts.push(CostPart::PayLife(Value::c(life)));
    }
}

/// Whether a mana cost contains symbols that can be paid in more than one way
/// (CR 118.13).
pub fn has_payment_choices(cost: &Cost) -> bool {
    cost.mana.as_ref().is_some_and(|m| {
        m.symbols
            .iter()
            .any(|s| halves(*s).is_some() || matches!(s, ManaSymbol::Phyrexian(_)))
    })
}

/// CR 118.8c: casting a spell "if able" doesn't require casting it if it has a mandatory
/// additional cost that involves cards with a stated quality in a hidden zone.
pub fn additional_cost_involves_hidden_quality(g: &Game, card: ObjectId) -> bool {
    let hidden_with_quality = |part: &CostPart| match part {
        CostPart::Discard { filter, .. }
        | CostPart::RevealFromHand { filter, .. }
        | CostPart::PutFromHandOnLibrary { filter, .. } => !matches!(filter, Filter::Any),
        CostPart::Exile {
            filter,
            zone: ZoneKind::Hand | ZoneKind::Library,
            ..
        } => !matches!(filter, Filter::Any),
        _ => false,
    };
    g.obj(card).chars.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Static(s) => matches!(
            &s.effect,
            StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                change: CostChange::AdditionalCost(c),
                ..
            }) if c.parts.iter().any(hidden_with_quality)
        ),
        _ => false,
    })
}

/// Asks whether to cast a card "if able" when it isn't required (CR 118.8c).
pub fn may_decline_cast_if_able(g: &mut Game, p: PlayerId, card: ObjectId) -> bool {
    additional_cost_involves_hidden_quality(g, card)
        && !matches!(
            g.ask(
                p,
                Decision::YesNo {
                    source: Some(card),
                    prompt: "Cast this spell?".into(),
                },
            ),
            Answer::Bool(true) | Answer::Default
        )
}

/// Whether `p` may spend mana of any type to cast `spell` (CR 118.14): an effect allowed
/// it for the card the spell was cast from.
pub fn may_spend_any_type(g: &Game, p: PlayerId, spell: ObjectId) -> bool {
    let card = g.obj(spell).prev;
    g.special.any_type_mana.iter().any(|(q, o, d, src, turn)| {
        *q == p
            && (*o == spell || Some(*o) == card)
            && match d {
                Duration::EndOfTurn | Duration::ThisTurn => *turn == g.turn.number,
                other => !g.effect_expired(other, *src, p),
            }
    })
}

/// CR 118.14: when mana of any type can be spent to cast a spell, mana may be spent as
/// though it were colorless mana or mana of any color: each colored, colorless or hybrid
/// symbol of its cost can be paid with one mana of any type.
pub fn spend_any_type(g: &Game, p: PlayerId, spell: ObjectId, cost: &mut Cost) {
    if !may_spend_any_type(g, p, spell) {
        return;
    }
    let Some(m) = cost.mana.as_mut() else {
        return;
    };
    for s in m.symbols.iter_mut() {
        if matches!(
            s,
            ManaSymbol::Colored(_)
                | ManaSymbol::Colorless
                | ManaSymbol::Hybrid(..)
                | ManaSymbol::TwoHybrid(_)
                | ManaSymbol::ColorlessHybrid(_)
        ) {
            *s = ManaSymbol::Generic(1);
        }
    }
    let generic = m.generic_amount();
    m.symbols.retain(|s| !matches!(s, ManaSymbol::Generic(_)));
    if generic > 0 {
        m.symbols.insert(0, ManaSymbol::Generic(generic));
    }
}
