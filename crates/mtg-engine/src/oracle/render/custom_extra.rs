//! More nodes implemented in code put into words from what they do: choices made as an
//! effect resolves (CR 608.2d: a player chosen at random, "an opponent chooses"),
//! repeatable additional costs and the numbers counting their payments (CR 601.2b, 601.2f),
//! "the player or planeswalker it's attacking" (CR 506.2), "pay any amount of life",
//! abilities lost by text ("loses this ability", CR 613.1f), and the special action
//! ending a Licid's effect (CR 116.2c). Also the state the renderer keeps across the
//! abilities of a card for these.

use super::costs::OwnerYou;
use super::players::Case;
use super::*;

/// What the renderer remembers for these nodes.
#[derive(Clone, Default)]
pub(crate) struct ExtraState {
    /// Repeatable additional costs of the card ("you may sacrifice any number of
    /// creatures"), by the name of the number counting their payments, with what one
    /// payment is ("creature", "sacrificed").
    pub paid_costs: Vec<(String, String, &'static str)>,
    /// X in the ability being rendered is the amount paid by "pay any amount of ...".
    pub x_paid: bool,
    /// The card pays any amount of life as it enters: its chosen number is that life.
    pub life_paid: bool,
}

/// The past tense of a past participle ("drawn" -> "drew").
fn past_tense(participle: &str) -> &str {
    match participle {
        "drawn" => "drew",
        "put" => "put",
        other => other,
    }
}

/// A mana-letter kind ("{r}", "mana", "life") as paid: "{R}", "mana", "life".
fn paid_kind(kind: &str) -> String {
    if kind.starts_with('{') {
        kind.to_uppercase()
    } else {
        kind.to_string()
    }
}

impl Renderer<'_> {
    /// `Effect::Custom` names put into words here (see the module docs).
    pub(crate) fn custom_effect_extra(&mut self, name: &str) -> Option<String> {
        use crate::kw::choice_grammar as cg;
        if name == crate::kw::end_this_effect::END_THIS_EFFECT {
            return Some("end this effect".into());
        }
        // The player, planeswalker or battle an attacking creature is attacking (CR 506.2,
        // 508.1b), remembered: later mentions of it are that phrase.
        if let Some(j) = name.strip_prefix(crate::kw::attack_recipient::ATTACK_RECIPIENT) {
            let (var, who): (Var, Sel) = serde_json::from_str(j).ok()?;
            let attacker = match who {
                Sel::This => "{alt:it's|it is|~ is}",
                Sel::TriggerObject => "{alt:that creature is|it's|it is}",
                _ => return None,
            };
            self.target_vars.push((
                var,
                format!("the player or planeswalker {attacker} attacking"),
                false,
            ));
            return Some(String::new());
        }
        if let Some(j) = name.strip_prefix(cg::RANDOM_PLAYER) {
            let (_, from): (Var, PlayerRef) = serde_json::from_str(j).ok()?;
            return Some(match from {
                PlayerRef::EachOpponent => "choose an opponent at random".into(),
                PlayerRef::EachPlayer => "choose a player at random".into(),
                _ => return None,
            });
        }
        if let Some(j) = name.strip_prefix(cg::RANDOM_NUMBER) {
            let (_, nums): (Var, Vec<i32>) = serde_json::from_str(j).ok()?;
            let w: Vec<String> = nums.iter().map(|n| n.to_string()).collect();
            return Some(format!("choose {} at random", join_list(&w, "or")));
        }
        // "An opponent chooses one of them": the opponent who makes the choice (CR
        // 801.5a), first mentioned as "an opponent".
        if let Some(j) = name.strip_prefix(cg::DECIDING_OPPONENT) {
            let var: Var = serde_json::from_str(j).ok()?;
            self.target_vars.push((var, "an opponent".into(), false));
            return Some(String::new());
        }
        if let Some(j) = name.strip_prefix(cg::CHOOSE_PLAYER) {
            let (_, pf): (Var, PlayerFilter) = serde_json::from_str(j).ok()?;
            let n = match pf {
                PlayerFilter::NotYou => "another player".to_string(),
                PlayerFilter::Life(Cmp::Ge, v)
                    if matches!(v.as_ref(), Value::OverPlayers(AggOp::Max, PlayerFilter::Any, l)
                        if matches!(l.as_ref(), Value::LifeTotal(PlayerRef::Iterated))) =>
                {
                    "a player with the most life or tied for most life".to_string()
                }
                other => with_article(&self.player_filter_noun(&other, Num::One)),
            };
            return Some(format!("choose {n}"));
        }
        if name.starts_with(cg::EACH_CHOOSES_CREATURE_TYPE) {
            return Some("each player chooses a creature type".into());
        }
        // The controller may look at the face-down objects remembered as "it" (CR 406.3,
        // 708.5).
        if name == crate::zones::MAY_LOOK_AT_EXILED {
            let it = self.sel(&Sel::Var(vars::IT), Case::Obj);
            return Some(format!("{{alt:look at|you may look at}} {it}"));
        }
        if name == crate::kw::pay_any_amount_of_life::PAY_ANY_LIFE {
            self.extra.life_paid = true;
            return Some("pay any amount of life".into());
        }
        if crate::kw::spell_cost_grammar::is_pay_any_amount(name) {
            let (head, may) = name.rsplit_once(':')?;
            let (_, kind) = head.rsplit_once(':')?;
            self.extra.x_paid = true;
            let may = if may == "may" { "you may " } else { "" };
            return Some(format!("{may}pay any amount of {}", paid_kind(kind)));
        }
        if name == crate::search_rules::FOUND_ON_TOP_ANY_ORDER {
            return Some(
                "put {alt:the chosen cards|them|those cards} on top of your library in any order"
                    .into(),
            );
        }
        if name == crate::kw::convoke::CONVOKERS_CONNIVE {
            let m = self.me();
            return Some(format!("each creature that convoked {m} connives"));
        }
        if name == crate::kw::space_sculptor::CHOOSE_SECTOR {
            // Named where the objects in it are ("in the sector of your choice").
            return Some(String::new());
        }
        if name == crate::kw::space_sculptor::SAME_SECTOR_BLOCKING {
            return Some(
                "creatures in each sector can be blocked this turn only by creatures in the same sector"
                    .into(),
            );
        }
        None
    }

    /// `Filter::Custom` qualities put into words here: (adjective?, words).
    pub(crate) fn custom_filter_extra(&mut self, name: &str) -> Option<(bool, String)> {
        if name.starts_with(crate::kw::choice_grammar::OF_A_TYPE_CHOSEN) {
            return Some((false, "of a type chosen this way".into()));
        }
        if name == crate::kw::space_sculptor::IN_CHOSEN_SECTOR {
            return Some((false, "in the sector of your choice".into()));
        }
        None
    }

    /// `Value::Custom` names put into words here.
    pub(crate) fn custom_value_extra(&mut self, name: &str) -> Option<String> {
        if let Some((noun, done)) = self.paid_cost(name) {
            return Some(format!("the number of {} {done} this way", plural(&noun)));
        }
        if let Some(d) = name.strip_prefix("dungeon completed:") {
            return Some(format!(
                "the number of times you've completed {}",
                dungeon_name(d)
            ));
        }
        Some(match name {
            "noncombat damage dealt this turn to:opponents" => {
                "the total amount of noncombat damage dealt to your opponents this turn".into()
            }
            "types: creature types it has" => "the number of creature types it has".into(),
            "types: supertypes, card types and subtypes it has" => {
                "the number of supertypes, card types, and subtypes it has".into()
            }
            "trigger spell:colors of mana spent" => {
                "the number of colors of mana spent to cast {alt:that spell|it}".into()
            }
            "trigger spell:times kicked" => {
                "the number of times {alt:that spell|it} was kicked".into()
            }
            "parley:land cards revealed this way" => {
                "the number of land cards revealed this way".into()
            }
            "players who have lost the game" => {
                "the number of players who have lost the game".into()
            }
            "vote: votes for it" => "the number of votes for it".into(),
            "heal: damage marked on it" => "the damage marked on it".into(),
            n if n == crate::kw::spell_cost_grammar::OWN_TARGETS => {
                "the number of targets it has".into()
            }
            n if n == crate::kw::spell_cost_grammar::CHOSEN_COLORS_IT_IS => {
                "the number of the chosen colors it is".into()
            }
            n if n.starts_with("spell cost:targets of type:") => {
                let t = &n["spell cost:targets of type:".len()..];
                format!("the number of {} it targets", plural(t))
            }
            _ => return None,
        })
    }

    /// A custom number compared with a number, as cards word it.
    pub(crate) fn custom_compare_extra(&mut self, name: &str, cmp: Cmp, n: i32) -> Option<String> {
        let d = name.strip_prefix("dungeon completed:")?;
        let d = dungeon_name(d);
        match (cmp, n) {
            (Cmp::Eq, 0) | (Cmp::Le, 0) | (Cmp::Lt, 1) => {
                Some(format!("you haven't completed {d}"))
            }
            (Cmp::Ge, 1) | (Cmp::Gt, 0) => Some(format!("you've completed {d}")),
            _ => None,
        }
    }

    /// A cost change's amount counted per payment of a repeatable additional cost or per
    /// something the spell does ("{2} less to cast for each card exiled this way", "{1}
    /// less to cast for each creature it targets").
    pub(crate) fn cost_amount_extra(&mut self, v: &Value) -> Option<(String, String)> {
        let (k, inner) = match v {
            Value::Mul(a, b) => match (a.as_ref(), b.as_ref()) {
                (Value::Const(k), Value::Custom(n)) | (Value::Custom(n), Value::Const(k)) => {
                    (*k, n.as_str())
                }
                _ => return None,
            },
            Value::Custom(n) => (1, n.as_str()),
            _ => return None,
        };
        let each = if let Some((noun, done)) = self.paid_cost(inner) {
            format!("{noun} {done} this way")
        } else if let Some(t) = inner.strip_prefix("spell cost:targets of type:") {
            format!("{t} it targets")
        } else if inner == crate::kw::spell_cost_grammar::CHOSEN_COLORS_IT_IS {
            "of the chosen colors it is".into()
        } else {
            return None;
        };
        Some((format!("{{{k}}}"), format!(" for each {each}")))
    }

    /// The payment counted by a "times paid" number, if the card's repeatable cost was
    /// rendered: (noun, participle).
    fn paid_cost(&self, name: &str) -> Option<(String, &'static str)> {
        self.extra
            .paid_costs
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, noun, done)| (noun.clone(), *done))
    }

    /// A repeatable additional cost paid any number of times (CR 601.2b): "you may
    /// sacrifice any number of creatures". Remembers what one payment is, for "for each
    /// creature sacrificed this way".
    pub(crate) fn repeated_any_number(&mut self, cost: &Cost, times: &Value) -> Option<String> {
        let Value::Custom(name) = times else {
            return None;
        };
        if !name.starts_with("spell cost:times paid:") || cost.mana.is_some() {
            return None;
        }
        let [part] = cost.parts.as_slice() else {
            return None;
        };
        let (phrase, noun, done) = match part {
            CostPart::Exile {
                filter,
                zone,
                count: Value::Const(1),
            } if !matches!(zone, ZoneKind::Battlefield | ZoneKind::Stack) => {
                let f = filter.clone().in_zone(*zone).and_owner_you();
                let n = self.noun(&f, Num::Many);
                let one = self.noun(filter, Num::One);
                let noun = if one == "card" {
                    one
                } else {
                    format!("{{alt:card|{one}}}")
                };
                (format!("exile any number of {n}"), noun, "exiled")
            }
            CostPart::Sacrifice {
                filter,
                count: Value::Const(1),
            } => {
                let f = effects::strip_controller(filter);
                let n = self.noun(&f, Num::Many);
                let one = self.noun(&f, Num::One);
                (
                    format!("sacrifice any number of {n}"),
                    format!("{{alt:{one}|permanent}}"),
                    "sacrificed",
                )
            }
            CostPart::TapUntapped {
                filter,
                count: Value::Const(1),
            } => {
                let f = Filter::and(vec![Filter::Untapped, filter.clone()]);
                let n = self.noun(&f, Num::Many);
                let one = self.noun(&effects::strip_controller(filter), Num::One);
                (format!("tap any number of {n}"), one, "tapped")
            }
            CostPart::RemoveCountersFromAmong {
                kind,
                filter,
                count: Value::Const(1),
            } => {
                let c = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let n = self.noun(filter, Num::Many);
                (
                    format!("remove any number of {} from among {n}", plural(&c)),
                    "counter".into(),
                    "removed",
                )
            }
            _ => return None,
        };
        self.extra
            .paid_costs
            .retain(|(n, _, _)| n.as_str() != name.as_str());
        self.extra.paid_costs.push((name.to_string(), noun, done));
        Some(format!("{{opt:you may}} {phrase}"))
    }

    /// "~ costs 3 life more to cast for each target" (an additional cost paid once per
    /// target the spell has).
    pub(crate) fn additional_cost_extra(&mut self, c: &Cost) -> Option<String> {
        let [CostPart::Repeated { cost, times }] = c.parts.as_slice() else {
            return None;
        };
        if c.mana.is_some()
            || !matches!(times, Value::Custom(n) if n == crate::kw::spell_cost_grammar::OWN_TARGETS)
        {
            return None;
        }
        let [CostPart::PayLife(Value::Const(n))] = cost.parts.as_slice() else {
            return None;
        };
        if cost.mana.is_some() {
            return None;
        }
        Some(format!("~ costs {n} life more to cast for each target"))
    }

    /// `Modification::Custom` names put into words here, as verb phrases.
    pub(crate) fn custom_modification_extra(&mut self, name: &str) -> Option<String> {
        if name == crate::kw::end_this_effect::LOSE_THIS_ABILITY {
            return Some(LOSES_THIS_ABILITY.into());
        }
        if let Some(text) = name.strip_prefix(crate::kw::grant_filters::LOSE_ABILITY_PREFIX) {
            return Some(format!("loses \"{}\"", capitalize(text.trim())));
        }
        if name == crate::kw::grant_filters::LOSE_ALL_LAND_TYPES {
            return Some("loses all land types".into());
        }
        None
    }

    /// `StaticEffect::Custom` names put into words here.
    pub(crate) fn custom_static_extra(&mut self, name: &str) -> Option<String> {
        if let Some(n) = name.strip_prefix("plot:plotting cards from your hand costs less:") {
            return Some(format!("plotting cards from your hand costs {{{n}}} less"));
        }
        if let Some(n) = name.strip_prefix("spell cost:unlock costs less:") {
            return Some(format!("unlock costs you pay cost {{{n}}} less"));
        }
        Some(match name {
            "facedown: may look at opponents' face-down creatures" => {
                "you may look at face-down creatures your opponents control {opt:any time}".into()
            }
            "plot:exile this spell instead of putting it into your graveyard as it resolves; it becomes plotted" => {
                "exile {alt:that spell|it} instead of putting it into your graveyard as it resolves. If you do, it becomes plotted".into()
            }
            _ => return None,
        })
    }

    /// The special action a Licid's effect offers (CR 116.2c: it can be taken any time
    /// its player has priority, for as long as the effect allows it): "You may pay {W} to
    /// end this effect."
    pub(crate) fn special_action_extra(&mut self, def: &SpecialActionDef) -> Option<String> {
        let SpecialActionEffect::Effect(Effect::Custom(n)) = &def.action else {
            return None;
        };
        if n.as_str() != crate::kw::end_this_effect::END_THIS_EFFECT || def.mana_timing {
            return None;
        }
        let c = self.cost(&def.cost);
        Some(format!("you may pay {c} to end this effect"))
    }
}

/// What [`Renderer::custom_modification_extra`] says for "loses this ability".
pub(crate) const LOSES_THIS_ABILITY: &str = "loses this ability";

/// "tomb of annihilation" -> "Tomb of Annihilation".
fn dungeon_name(d: &str) -> String {
    d.split(' ')
        .map(|w| {
            if matches!(w, "of" | "the") {
                w.to_string()
            } else {
                capitalize(w)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether a modification list makes its objects lose all land types and all abilities
/// ("loses all land types and abilities", CR 305.7).
pub(crate) fn loses_land_types_and_abilities(mods: &[Modification]) -> bool {
    mods.iter().any(|m| matches!(m, Modification::RemoveAllAbilities))
        && mods.iter().any(|m| {
            matches!(m, Modification::Custom { name, .. }
                if name.as_str() == crate::kw::grant_filters::LOSE_ALL_LAND_TYPES)
        })
}

impl Renderer<'_> {
    /// [`Value::OverPlayers`] forms put into words here: "the greatest number of cards a
    /// player discarded this way", "the number of opponents who drew two or more cards
    /// this turn".
    pub(crate) fn over_players_extra(
        &mut self,
        op: AggOp,
        pf: &PlayerFilter,
        v: &Value,
    ) -> Option<String> {
        let (one, many) = match pf {
            PlayerFilter::Opponent => ("an opponent", "{alt:opponents|your opponents}"),
            PlayerFilter::Any => ("a player", "players"),
            _ => return None,
        };
        match (op, v) {
            (AggOp::Max, Value::CountSel(s)) => {
                let (verb, card) = self.done_to_own(s)?;
                let noun = if card { "cards" } else { "permanents" };
                Some(format!(
                    "the greatest number of {noun} {one} {} this way",
                    past_tense(verb)
                ))
            }
            (AggOp::Max, Value::EventsThisTurn(c, Tally::Events))
                if matches!(c.as_ref(), TriggerCond::Draws { who: PlayerRel::Iterated }) =>
            {
                Some(format!(
                    "the greatest number of cards {one} has drawn this turn"
                ))
            }
            (AggOp::Sum, Value::If(c, a, b))
                if matches!((a.as_ref(), b.as_ref()), (Value::Const(1), Value::Const(0))) =>
            {
                let Condition::Compare(x, Cmp::Ge, Value::Const(k)) = c.as_ref() else {
                    return None;
                };
                let did = self.player_did(x, *k)?;
                Some(format!("the number of {many} who {did}"))
            }
            _ => None,
        }
    }

    /// "the cards each player discarded this way": the objects an earlier instruction
    /// acted on that the iterated player owns. (What was done, whether they're cards.)
    fn done_to_own(&self, s: &Sel) -> Option<(&'static str, bool)> {
        let Sel::Matching(inner, Filter::OwnedBy(PlayerRel::Iterated)) = s else {
            return None;
        };
        let (verb, card, _) = self.this_way_of(inner)?;
        Some((verb, card))
    }

    /// What the iterated player did at least `k` times: "drew a card this way", "had two
    /// or more lands enter the battlefield under their control this turn".
    fn player_did(&mut self, x: &Value, k: i32) -> Option<String> {
        let amount = |noun: &str| {
            if k <= 1 {
                with_article(noun)
            } else {
                format!("{} or more {}", number_word(k), plural(noun))
            }
        };
        match x {
            Value::CountSel(s) => {
                let (verb, card) = self.done_to_own(s)?;
                let noun = if card { "card" } else { "permanent" };
                Some(format!("{} {} this way", past_tense(verb), amount(noun)))
            }
            Value::EventsThisTurn(c, Tally::Events)
                if matches!(c.as_ref(), TriggerCond::Draws { who: PlayerRel::Iterated }) =>
            {
                Some(format!("drew {} this turn", amount("card")))
            }
            Value::PermanentsEnteredThisTurn(PlayerRef::Iterated, f) => {
                let n = self.noun(f, Num::One);
                Some(format!(
                    "had {} enter the battlefield under their control this turn",
                    amount(&n)
                ))
            }
            _ => None,
        }
    }

    /// [`Sel::ThisTurn`]: the objects of this turn's events ("spells you've cast this
    /// turn", "permanents you've sacrificed this turn").
    pub(crate) fn this_turn_objects(&mut self, c: &TriggerCond) -> Option<String> {
        if let TriggerCond::DealsDamage {
            source,
            to: DamageRecipient::Player(PlayerRel::Any),
            combat_only,
        } = c
        {
            let mine = |x: &Filter| matches!(x, Filter::ControlledBy(PlayerRel::You));
            let (f, yours) = match source {
                Filter::And(v) if v.iter().any(mine) => (
                    Filter::and(v.iter().filter(|x| !mine(x)).cloned().collect()),
                    true,
                ),
                other => (other.clone(), false),
            };
            let n = self.noun(&f, Num::Many);
            let ctl = if yours { " you controlled" } else { "" };
            let combat = if *combat_only { "combat " } else { "" };
            return Some(format!(
                "{n}{ctl} that dealt {combat}damage to a player this turn"
            ));
        }
        if !matches!(
            c,
            TriggerCond::CastSpell { .. } | TriggerCond::YouSacrifice(_)
        ) {
            return None;
        }
        let s = self.events_this_turn(c, Tally::Events);
        s.strip_prefix("the number of ").map(str::to_string)
    }
}
