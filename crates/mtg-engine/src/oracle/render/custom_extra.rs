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
    /// That amount is of {E} (an X in a cost reads "{1} for each {E} paid this way").
    pub x_energy: bool,
    /// The card pays any amount of life as it enters: its chosen number is that life.
    pub life_paid: bool,
    /// How X reads after "pay any amount" when it was redefined ("twice that many").
    pub x_words: Option<&'static str>,
    /// X in the activated ability being rendered is the number of counters removed to pay
    /// its cost ("Remove any number of charge counters from ~: It deals that much
    /// damage").
    pub x_removed: bool,
    /// What the triggering event's amount is in words, in a trigger whose event has one
    /// ("the amount of damage it dealt to that player", "the number of cards looked at
    /// while scrying this way").
    pub event_amount: Option<&'static str>,
    /// In a "whenever the Ring tempts you" trigger, whose event object is the creature
    /// chosen as your Ring-bearer (CR 701.54a).
    pub ring_tempts: bool,
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
            self.extra.x_energy = kind == "{e}";
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
        // CR 106.4: the mana of one type in your mana pool.
        if let Some(t) = name.strip_prefix(crate::kw::unspent_mana_value::UNSPENT_MANA_OF_TYPE) {
            let t = crate::mana::ManaType::from_letter_str(t)?;
            let c = t.color().map_or("colorless", |c| c.word());
            return Some(format!(
                "the {{alt:number|amount}} of unspent {c} mana you have"
            ));
        }
        if let Some(d) = name.strip_prefix("dungeon completed:") {
            return Some(format!(
                "the number of times you've completed {}",
                dungeon_name(d)
            ));
        }
        Some(match name {
            // CR 701.52a: the Attractions you visited after rolling to visit them.
            crate::kw::visit::VISITED_THIS_TURN => {
                "the number of Attractions you've visited this turn".into()
            }
            // CR 903.8: how many times the commander was cast from the command zone.
            crate::kw::game_rules::TIMES_IT_WAS_CAST => {
                "the number of times it's been cast from the command zone this game".into()
            }
            // Pack tactics: the attackers' power as they were declared.
            crate::kw::pack_tactics::ATTACKED_WITH_TOTAL_POWER => {
                "the total power of the creatures you attacked with this combat".into()
            }
            crate::kwa::ring::TIMES_TEMPTED => {
                "the number of times the Ring has tempted you this game".into()
            }
            // CR 107.14: the energy counters removed from you this turn.
            crate::kw::energy_paid::ENERGY_PAID_OR_LOST_THIS_TURN => {
                "the number of {E} you've paid or lost this turn".into()
            }
            // CR 106.4: the mana in your mana pool.
            crate::kw::unspent_mana_value::UNSPENT_MANA_YOU_HAVE => {
                "the {alt:amount|number} of unspent mana you have".into()
            }
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
        // "you have six or more unspent mana", "you've paid or lost four or more {E} this
        // turn".
        let min = match cmp {
            Cmp::Ge => Some(n),
            Cmp::Gt => Some(n + 1),
            _ => None,
        };
        if let Some(min) = min {
            let w = number_word(min);
            if name == crate::kw::unspent_mana_value::UNSPENT_MANA_YOU_HAVE {
                return Some(format!("you have {w} or more unspent mana"));
            }
            if let Some(t) = name.strip_prefix(crate::kw::unspent_mana_value::UNSPENT_MANA_OF_TYPE)
            {
                let t = crate::mana::ManaType::from_letter_str(t)?;
                let c = t.color().map_or("colorless", |c| c.word());
                return Some(format!("you have {w} or more unspent {c} mana"));
            }
            if name == crate::kw::energy_paid::ENERGY_PAID_OR_LOST_THIS_TURN {
                return Some(format!("you've paid or lost {w} or more {{E}} this turn"));
            }
        }
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
                (format!("exile any number of {n}"), "card".into(), "exiled")
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
        Some(phrase)
    }

    /// "~ costs 3 life more to cast for each target" (an additional cost paid once per
    /// target the spell has).
    pub(crate) fn additional_cost_extra(&mut self, c: &Cost) -> Option<String> {
        let [CostPart::Repeated { cost, times }] = c.parts.as_slice() else {
            return None;
        };
        // "As an additional cost to cast this spell, you may sacrifice any number of
        // creatures": paying it zero times is choosing not to pay it.
        if c.mana.is_none() {
            if let Some(s) = self.repeated_any_number(cost, times) {
                return Some(format!(
                    "as an additional cost to cast ~, {{opt:you may}} {s}"
                ));
            }
        }
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
    mods.iter()
        .any(|m| matches!(m, Modification::RemoveAllAbilities))
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
                if matches!(
                    c.as_ref(),
                    TriggerCond::Draws {
                        who: PlayerRel::Iterated
                    }
                ) =>
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
        if let Some((verb, card, _)) = self.this_way_of(inner) {
            return Some((verb, card));
        }
        // The cards the discard instructions of the effect discarded.
        match inner.as_ref() {
            Sel::Var(v) if *v == crate::discard_rules::DISCARDED => Some(("discarded", true)),
            _ => None,
        }
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
                if matches!(
                    c.as_ref(),
                    TriggerCond::Draws {
                        who: PlayerRel::Iterated
                    }
                ) =>
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

impl Renderer<'_> {
    /// Instructions of a sequence put into words together here, and how many: "you may
    /// pay {1}{G} any number of times. When you pay this cost one or more times, ..."
    /// (the payments counted in a number, then a reflexive trigger, CR 603.12).
    pub(crate) fn seq_extra(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        if let Some(r) = self.seq_more(v) {
            return Some(r);
        }
        if let Some(s) = self.revealed_this_way(v) {
            return Some((s, 4));
        }
        if let Some(r) = self.countered_source(v) {
            return Some(r);
        }
        // "You may look at and play those cards for as long as they remain exiled."
        if let [Effect::Custom(n), next, ..] = v {
            if n.as_str() == crate::zones::MAY_LOOK_AT_EXILED {
                // (The cards looked at and played were exiled.)
                let saved = std::mem::replace(&mut self.after_exile, true);
                let s = self.effect(next);
                self.after_exile = saved;
                if let Some(rest) = s.strip_prefix("you may play ") {
                    return Some((format!("you may look at and play {rest}"), 2));
                }
            }
        }
        // "An opponent chooses one of them. Put that card into your hand.": the chosen
        // card, first mentioned as "that card".
        if let [st @ Effect::Store {
            var,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::Var(_),
                    filter: Filter::In(_),
                    count: Value::Const(1),
                    up_to: false,
                    ..
                },
        }, ..] = v
        {
            let s = self.effect(st);
            self.target_vars
                .push((*var, "{alt:that card|it}".into(), false));
            return Some((s, 1));
        }
        // "Choose up to two target creature cards in your graveyard. An opponent chooses
        // one of them.": the targets, then the opponent's choice among them.
        if let [d @ Effect::Custom(n), Effect::Store {
            var,
            sel: Sel::Target(i),
        }, ..] = v
        {
            if n.starts_with(crate::kw::choice_grammar::DECIDING_OPPONENT)
                && self.introduced.get(*i as usize) == Some(&false)
            {
                let t = self.target_phrase(*i);
                self.introduced[*i as usize] = true;
                self.plural_vars.push(*var);
                let d = self.effect(d);
                return Some((effects::join_words(&[d, format!("choose {t}")]), 2));
            }
        }
        // "Pay any amount of life. The amount you pay can't be more than [V]."
        if let [Effect::StoreValue { var, value }, pay @ Effect::Custom(n), ..] = v {
            if *var == crate::kw::pay_any_amount_of_life::CAP
                && n.as_str() == crate::kw::pay_any_amount_of_life::PAY_ANY_LIFE
            {
                let p = self.effect(pay);
                let cap = self.value(value);
                return Some((
                    format!("{p}. The amount you pay can't be more than {cap}"),
                    2,
                ));
            }
        }
        // "create twice that many tokens": X doubled after "pay any amount".
        if let [Effect::SetX { value }, rest @ ..] = v {
            let doubled = match value {
                Value::Sum(p) => matches!(p.as_slice(), [Value::X, Value::X]),
                Value::Mul(a, b) => {
                    matches!(
                        (a.as_ref(), b.as_ref()),
                        (Value::Const(2), Value::X) | (Value::X, Value::Const(2))
                    )
                }
                _ => false,
            };
            if doubled && self.extra.x_paid && !rest.is_empty() && self.extra.x_words.is_none() {
                self.extra.x_words = Some("twice that many");
                let s = self.seq(rest);
                self.extra.x_words = None;
                return Some((s, v.len()));
            }
        }
        let [Effect::May {
            who: PlayerRef::You,
            effect,
        }, Effect::If {
            cond,
            then,
            otherwise,
        }, ..] = v
        else {
            return None;
        };
        let cost = paid_any_number_of_times(effect)?;
        if !matches!(cond, Condition::Compare(Value::X, Cmp::Ge, Value::Const(1)))
            || !matches!(otherwise.as_ref(), Effect::Noop)
        {
            return None;
        }
        let Effect::Reflexive { body } = then.as_ref() else {
            return None;
        };
        let c = self.cost(cost);
        let saved = std::mem::replace(&mut self.extra.x_paid, true);
        let b = self.in_event_scope(|r| r.body(body));
        self.extra.x_paid = saved;
        Some((
            format!(
                "you may pay {c} any number of times. When you pay this cost one or more times, {}",
                effects::lower_first(&b)
            ),
            2,
        ))
    }
}

/// The cost of "you may pay [cost] any number of times" with the times counted as X:
/// a payment repeated until the player declines, counted, then X set to the count.
fn paid_any_number_of_times(e: &Effect) -> Option<&Cost> {
    let Effect::Seq(v) = e else {
        return None;
    };
    let [Effect::StoreValue {
        var: count,
        value: Value::Const(0),
    }, Effect::StoreValue {
        var: stop,
        value: Value::Const(0),
    }, Effect::Repeat { effect, .. }, Effect::SetX {
        value: Value::Var(x),
    }] = v.as_slice()
    else {
        return None;
    };
    if x != count {
        return None;
    }
    let Effect::If {
        cond: Condition::Compare(Value::Var(s), Cmp::Eq, Value::Const(0)),
        then,
        otherwise,
    } = effect.as_ref()
    else {
        return None;
    };
    if s != stop || !matches!(otherwise.as_ref(), Effect::Noop) {
        return None;
    }
    let Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then,
        otherwise,
    } = then.as_ref()
    else {
        return None;
    };
    let counts = matches!(then.as_ref(), Effect::StoreValue { var, value: Value::Sum(p) }
        if var == count && matches!(p.as_slice(), [Value::Var(a), Value::Const(1)] if a == count));
    let stops = matches!(otherwise.as_ref(), Effect::StoreValue { var, value: Value::Const(1) } if var == stop);
    (counts && stops).then_some(cost)
}

impl Renderer<'_> {
    /// A static permission to play only one of the cards it's for (`PlayTerms::limit` of
    /// one): "you may play a card exiled with ~", "you may cast a creature spell from your
    /// graveyard", "without paying its mana cost".
    pub(crate) fn one_play_permission(&mut self, pp: &PlayPermission) -> Option<String> {
        let t = &pp.terms;
        if t.limit != Some(1) || t.what.is_some() || t.until_another || t.later_turn {
            return None;
        }
        let mut q = pp.clone();
        q.terms.limit = None;
        let s = self.play_permission(&q);
        let s = s.replace(
            " without paying their mana costs",
            " without paying its mana cost",
        );
        let (head, rest) = ["may play ", "may cast "]
            .iter()
            .find_map(|v| s.find(v).map(|i| (i + v.len(), ())))
            .map(|(i, _)| (s[..i].to_string(), s[i..].to_string()))?;
        // "{alt:spells exiled with ~|spells from among cards exiled with ~}".
        if let Some(r) = rest.strip_prefix("{alt:spells exiled with ") {
            let r = r.replacen("|spells from among", "|a spell from among", 1);
            return Some(format!("{head}{{alt:a spell exiled with {r}"));
        }
        let end = [
            " from ",
            " exiled with ",
            " without ",
            " {",
            " until ",
            " this turn",
        ]
        .iter()
        .filter_map(|m| rest.find(m))
        .min()
        .unwrap_or(rest.len());
        let (noun, tail) = rest.split_at(end);
        let one = with_article(&singular(noun));
        // "a creature spell exiled with ~" is one "from among cards exiled with ~".
        if let Some(w) = tail.strip_prefix(" exiled with ") {
            if one.ends_with(" spell") {
                let (who, after) = w.split_once(' ').unwrap_or((w, ""));
                let after = if after.is_empty() {
                    String::new()
                } else {
                    format!(" {after}")
                };
                return Some(format!(
                    "{head}{{alt:{one} exiled with {who}|{one} from among cards exiled with {who}}}{after}"
                ));
            }
        }
        Some(format!("{head}{one}{tail}"))
    }
}

impl Renderer<'_> {
    /// Remembering a group made of several parts (`Effect::Store` of a `Sel::Union`):
    /// a choice added to what the variable already holds ("You may choose a creature card
    /// in your graveyard"), two choices ("You choose from it a nonland card with mana value
    /// 3 or less and a card with mana value 4 or greater"), or a group named where it's
    /// used ("destroy it and all creatures it blocked this turn").
    pub(crate) fn remember_union(&mut self, var: Var, parts: &[Sel]) -> Option<String> {
        // The variable's own earlier contents plus new ones: the new ones are chosen.
        let new: Vec<&Sel> = parts
            .iter()
            .filter(|p| !matches!(p, Sel::Var(v) if *v == var))
            .collect();
        if new.len() < parts.len() {
            if let [one @ Sel::Choose { .. }] = new.as_slice() {
                return Some(self.effect(&Effect::Store {
                    var,
                    sel: (*one).clone(),
                }));
            }
        }
        if new.len() == parts.len()
            && parts.len() == 2
            && parts.iter().all(|p| matches!(p, Sel::Choose { .. }))
        {
            let a = self.effect(&Effect::Store {
                var,
                sel: parts[0].clone(),
            });
            let b = self.effect(&Effect::Store {
                var,
                sel: parts[1].clone(),
            });
            self.plural_vars.push(var);
            let (Some(a), Some(b)) = (a.strip_prefix("choose "), b.strip_prefix("choose ")) else {
                return Some(format!("{a}, then {b}"));
            };
            let mut forms = vec![
                format!("choose {a} and {b}"),
                format!("choose {a}, then choose {b}"),
            ];
            if let (Some(x), Some(y)) = (a.strip_suffix(" from it"), b.strip_suffix(" from it")) {
                forms.push(format!("choose from it {x} and {y}"));
            }
            return Some(format!("{{alt:{}}}", forms.join("|")));
        }
        // Named where it's used.
        if parts.iter().all(|p| {
            matches!(
                p,
                Sel::This | Sel::All(_) | Sel::Target(_) | Sel::TriggerObject
            )
        }) {
            self.var_defs.push((var, Sel::Union(parts.to_vec()), false));
            return Some(String::new());
        }
        None
    }
}

impl Renderer<'_> {
    /// "You may reveal that card. If an instant or sorcery card is revealed this way,
    /// [effect]." (Delver of Secrets): the looked-at card optionally revealed, remembered
    /// if it was, then a check of what was revealed.
    fn revealed_this_way(&mut self, v: &[Effect]) -> Option<String> {
        let [Effect::May {
            who: PlayerRef::You,
            effect,
        }, Effect::If {
            cond: Condition::PrevHappened,
            then: kept,
            otherwise: none,
        }, Effect::Store {
            sel: Sel::All(_), ..
        }, Effect::If {
            cond,
            then,
            otherwise,
        }, ..] = v
        else {
            return None;
        };
        let Effect::DigStep(step) = effect.as_ref() else {
            return None;
        };
        let DigStep::Take {
            count: Some(Value::Const(1)),
            reveal: true,
            up_to: false,
            random: false,
            to,
            ..
        } = step.as_ref()
        else {
            return None;
        };
        if to.zone != ZoneKind::Library || !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let (
            Effect::Store {
                var,
                sel: Sel::Var(_),
            },
            Effect::Store {
                var: v2,
                sel: Sel::None,
            },
        ) = (kept.as_ref(), none.as_ref())
        else {
            return None;
        };
        if var != v2 {
            return None;
        }
        // "the revealed card exists and matches F".
        let Condition::And(cs) = cond else {
            return None;
        };
        let [Condition::SelNonEmpty(Sel::Var(a)), Condition::Not(n)] = cs.as_slice() else {
            return None;
        };
        let Condition::SelMatches(Sel::Var(b), Filter::Not(f)) = n.as_ref() else {
            return None;
        };
        if a != var || b != var {
            return None;
        }
        let what = self.noun_det(f, super::nouns::Det::A);
        let e = self.effect(then);
        Some(format!(
            "you may reveal {{alt:that card|it}}. If {what} is revealed this way, {e}"
        ))
    }
}

impl Renderer<'_> {
    /// "Counter target activated ability. If a permanent's ability is countered this way,
    /// destroy that permanent." / "That permanent's activated abilities can't be
    /// activated this turn.": the source of the targeted ability remembered before it's
    /// countered, then "that permanent".
    fn countered_source(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::Store {
            var,
            sel: Sel::All(f),
        }, counter @ Effect::CounterSpell {
            what: Sel::Target(0),
        }, rest @ ..] = v
        else {
            return None;
        };
        let parts: Vec<&Filter> = match f {
            Filter::And(p) => p.iter().collect(),
            other => vec![other],
        };
        let is_source = |x: &Filter| {
            matches!(x, Filter::Custom(n)
                if n.as_str() == "basic_effects:source of the ability in target slot 0")
        };
        if !parts.iter().any(|x| is_source(x))
            || !parts
                .iter()
                .all(|x| is_source(x) || matches!(x, Filter::Permanent))
        {
            return None;
        }
        let destroys = matches!(
            rest.first(),
            Some(Effect::If {
                cond: Condition::PrevHappened,
                ..
            })
        );
        let that = if destroys {
            "{alt:that permanent|that artifact}"
        } else {
            "that permanent"
        };
        self.target_vars.push((*var, that.into(), false));
        let c = self.effect(counter);
        match rest.first() {
            Some(Effect::If {
                cond: Condition::PrevHappened,
                then,
                otherwise,
            }) if matches!(otherwise.as_ref(), Effect::Noop) => {
                let e = self.effect(then);
                Some((
                    format!(
                        "{{alt:{c}. If a permanent's ability is countered this way, {e}|{c} and {e} if it's on the battlefield|if it's on the battlefield, {c} and {e}}}"
                    ),
                    3,
                ))
            }
            Some(next) => {
                let e = self.effect(next);
                Some((format!("{c}. {e}"), 3))
            }
            None => None,
        }
    }
}

/// Whether an instruction refers to what variable `var` holds without naming it: "look
/// at" names "it" ([`vars::IT`]).
pub(crate) fn names_it(e: &Effect, var: Var) -> bool {
    matches!(e, Effect::Custom(n) if n.as_str() == crate::zones::MAY_LOOK_AT_EXILED)
        && var == vars::IT
}

impl Renderer<'_> {
    /// Alternative qualities of one noun, said as one quality after it: "with mana value 2
    /// or 3", "your team controls" (you or a teammate, CR 808.2), "from graveyards or exile"
    /// (a spell cast from there, or a card there).
    pub(crate) fn or_quality(&mut self, v: &[Filter]) -> Option<String> {
        if let Some(q) = self.or_predicates(v) {
            return Some(q);
        }
        let mvs: Option<Vec<String>> = v
            .iter()
            .map(|x| match x {
                Filter::ManaValue(Cmp::Eq, n) => match n.as_ref() {
                    Value::Const(n) => Some(n.to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        if let Some(m) = mvs {
            if m.len() >= 2 {
                return Some(format!("with mana value {}", join_list(&m, "or")));
            }
        }
        if let [Filter::ControlledBy(PlayerRel::You), Filter::ControlledBy(PlayerRel::Teammate)] = v
        {
            return Some("your team controls".into());
        }
        // (Nested lists may have been flattened: "in a graveyard or cast from one, or in
        // exile or cast from there".)
        let flat: Vec<&Filter> = v
            .iter()
            .flat_map(|x| match x {
                Filter::Or(p) => p.iter().collect::<Vec<_>>(),
                other => vec![other],
            })
            .collect();
        let zones: Option<Vec<&str>> = flat
            .chunks(2)
            .map(|pair| match pair {
                [Filter::InZone(a), Filter::CastFrom(b)] if a == b => match a {
                    ZoneKind::Graveyard => Some("graveyards"),
                    ZoneKind::Exile => Some("exile"),
                    ZoneKind::Library => Some("libraries"),
                    ZoneKind::Hand => Some("hands"),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let zones = zones.filter(|z| z.len() >= 2)?;
        let z: Vec<String> = zones.iter().map(|s| s.to_string()).collect();
        Some(format!("from {}", join_list(&z, "or")))
    }
}

impl Renderer<'_> {
    /// Selections put into words here: "one of them" (chosen among a group named
    /// before), "the rest" / "the other" (the group without what was chosen from it).
    pub(crate) fn sel_extra(&mut self, s: &Sel, case: Case) -> Option<String> {
        match s {
            // "Exile it. If you do, exile the top card of your library. ... one of those
            // cards": the cards both instructions moved, worded as "those cards" (see
            // `permission_grammar::keep_earlier_moved`).
            Sel::Union(parts)
                if matches!(parts.as_slice(), [Sel::All(Filter::And(f)), Sel::Var(v)]
                    if *v == vars::IT
                        && matches!(f.first(), Some(Filter::In(s)) if matches!(s.as_ref(),
                            Sel::Var(e) if *e == crate::oracle::patterns::permission_grammar::EARLIER_MOVED))) =>
            {
                Some(self.sel(&Sel::Var(vars::IT), case))
            }
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::In(g),
                count: Value::Const(n),
                up_to: false,
                ..
            } => {
                let Sel::Var(g) = g.as_ref() else {
                    return None;
                };
                let fresh =
                    |v: &[(Var, String, bool)]| v.iter().any(|(x, _, used)| x == g && !used);
                if *g == vars::IT
                    || fresh(&self.target_vars)
                    || self.var_defs.iter().any(|(x, _, used)| x == g && !used)
                {
                    return None;
                }
                Some(super::players::decline(
                    format!("{} of them", number_word(*n)),
                    case,
                ))
            }
            // "Search your library and graveyard for five cards and exile the rest": each
            // zone searched, without what was found.
            Sel::Union(parts) if parts.len() >= 2 => {
                let without: Option<Vec<Var>> = parts
                    .iter()
                    .map(|p| match p {
                        Sel::All(Filter::And(v))
                            if v.iter().all(|x| {
                                matches!(x, Filter::InZone(_) | Filter::OwnedBy(_) | Filter::Not(_))
                            }) =>
                        {
                            v.iter().find_map(|x| match x {
                                Filter::Not(n) => match n.as_ref() {
                                    Filter::In(s) => match s.as_ref() {
                                        Sel::Var(v) => Some(*v),
                                        _ => None,
                                    },
                                    _ => None,
                                },
                                _ => None,
                            })
                        }
                        _ => None,
                    })
                    .collect();
                let w = without?;
                if w.iter().any(|x| *x != w[0]) {
                    return None;
                }
                Some(super::players::decline("the rest".into(), case))
            }
            Sel::All(f) => {
                let rest = |f: &Filter| {
                    match f {
                    Filter::And(p) => p.iter().any(|x| {
                        matches!(x, Filter::Not(n) if matches!(n.as_ref(), Filter::In(s) if matches!(s.as_ref(), Sel::Var(_))))
                    }) && p.iter().all(|x| {
                        matches!(x, Filter::Not(_) | Filter::In(_))
                    }),
                    _ => false,
                }
                };
                if !rest(f) {
                    return None;
                }
                let Filter::And(p) = f else {
                    return None;
                };
                // The group: what the variable holds, without the chosen ones.
                let [Filter::In(a), Filter::Not(b)] = p.as_slice() else {
                    return None;
                };
                let (Sel::Var(_), Filter::In(c)) = (a.as_ref(), b.as_ref()) else {
                    return None;
                };
                let Sel::Var(chosen) = c.as_ref() else {
                    return None;
                };
                let _ = chosen;
                Some(super::players::decline(
                    "{alt:the rest|the other|the others}".into(),
                    case,
                ))
            }
            _ => None,
        }
    }
}

impl Renderer<'_> {
    /// Numbers put into words here: "the number of creatures sacrificed this way" (the
    /// objects a repeatable additional cost of the card acted on).
    pub(crate) fn value_extra(&mut self, v: &Value) -> Option<String> {
        let Value::CountSel(s) = v else {
            return None;
        };
        let Sel::Matching(inner, f) = s.as_ref() else {
            return None;
        };
        let Sel::Var(var) = inner.as_ref() else {
            return None;
        };
        let done = match *var {
            vars::SACRIFICED => "sacrificed",
            vars::TAPPED => "tapped",
            _ => return None,
        };
        if !self.extra.paid_costs.iter().any(|(_, _, d)| *d == done) {
            return None;
        }
        let n = self.noun(f, Num::Many);
        Some(format!("the number of {n} {done} this way"))
    }
}
