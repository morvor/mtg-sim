//! Requests (a decision with its complete list of legal options) and answers, and their
//! conversion to and from the engine's [`Decision`] and [`Answer`].

use crate::describe::{
    action_text, cast_method_code, entity_name, object_name, player_name, visible_id,
};
use crate::events::EventView;
use crate::legal::{priority_options, PriorityOptions};
use crate::view::{observe, step_name, EntityRef, Observation};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::eval::Ctx;
use mtg_engine::{Entity, Game, ObjectId, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The protocol version; see `docs/AGENT_PROTOCOL.md`.
pub const PROTOCOL_VERSION: u32 = 1;

/// The shape of the answer a request expects.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerSpec {
    /// Pick one option: `{"index": i}`.
    #[default]
    ChooseOne,
    /// Pick between `min` and `max` options: `{"indices": [i, j, ...]}`. Unless
    /// `distinct` is false, each option at most once. Options with a `group` may be
    /// chosen at most `group_max` times per group. With `budget`, the options' `cost`s
    /// may total at most that.
    ChooseMany {
        min: u32,
        max: u32,
        distinct: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        budget: Option<u32>,
    },
    /// Order all the options: `{"indices": [...]}`, a permutation of the option indices,
    /// first = first in the order described by the prompt.
    Order,
    /// A number between `min` and `max` (no upper bound when `max` is absent):
    /// `{"number": n}`.
    Number {
        min: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// Divide `total` among the options, at least `min_each` to each:
    /// `{"numbers": [n0, n1, ...]}`, one per option, summing to `total`.
    Divide { total: u32, min_each: u32 },
    /// Assign `total` combat damage among the options: `{"numbers": [...]}`, one per
    /// option, summing to `total`. With `trample`, damage may be assigned to the options
    /// after the blocking creatures only once each blocking creature is assigned at least
    /// its `lethal` damage, and to a planeswalker's controller only once the planeswalker
    /// is assigned its `lethal` damage (CR 510.1c, 702.19b–c).
    AssignDamage { total: u32, trample: bool },
    /// Yes or no: `{"bool": true}` (or `{"index": 1}` for the "yes" option).
    YesNo,
    /// Split all the options into two ordered lists: `{"split": [[...], [...]]}`, every
    /// option index in exactly one of them. The first list is `first` (e.g. "top", in
    /// order from the top), the second `second` ("bottom" or "graveyard").
    Split { first: String, second: String },
    /// Free text: `{"text": "..."}`. `what` is "card_name" for naming a card (CR 201.3).
    Text { what: String },
}

/// A priority action, structured.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionView {
    /// "pass", "play_land", "cast", "activate", "mana_ability", "special", "concede".
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>,
    /// The ability's id (for "activate", "mana_ability" and some special actions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability: Option<u64>,
    /// How the spell is cast: "normal", "free", "flashback", "face_down:morph", ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<String>,
    /// For special actions: which one ("turn_face_up", "suspend", "foretell", ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special: Option<String>,
}

/// One legal option.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OptionView {
    pub index: usize,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionView>,
    /// The player or object this option is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityRef>,
    /// Attackers and blockers: the creature this option is about (options of the same
    /// creature form a group).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<u32>,
    /// The most options of this option's group that may be chosen (absent: no limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_max: Option<u32>,
    /// Attackers: what the creature would attack; blockers: the attacker it would block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<EntityRef>,
    /// Combat damage: the damage that's lethal for this recipient.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lethal: Option<u32>,
    /// Modes with pawprints: this mode's pawprints (CR 700.2i).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<u32>,
    /// Yes/no questions: the answer this option stands for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<bool>,
}

/// A decision for an agent: everything it needs to answer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Always "decision".
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: u32,
    /// Identifies the decision; an answer may echo it.
    pub id: u64,
    /// The seat (player) whose agent is asked.
    pub seat: u8,
    /// The player the decision is for (another player than `seat` while `seat` controls
    /// that player, CR 723).
    pub player: u8,
    /// The kind of decision: "priority", "mulligan", "put_on_bottom", "choose_modes",
    /// "choose_x", "casting_method", "optional_cost", "targets", "divide", "yes_no",
    /// "choose", "order", "choose_option", "choose_number", "name_card",
    /// "declare_attackers", "declare_blockers", "assign_combat_damage", "scry",
    /// "surveil", "replacement".
    pub kind: String,
    pub prompt: String,
    /// The spell, ability or permanent the decision is about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    pub answer: AnswerSpec,
    pub options: Vec<OptionView>,
    /// What happened since this seat's previous decision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventView>,
    /// What this seat can see of the game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<Observation>,
    /// Why the previous answer to this decision was rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 0 for the first time this decision is asked, then 1, 2, ... after rejected answers.
    #[serde(default)]
    pub attempt: u32,
}

/// An answer, as sent by an agent. Exactly one of the answer fields must be present.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonAnswer {
    /// Optional; when present it must match the request's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// Optional message type ("answer").
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub msg_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<Vec<usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbers: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bool: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split: Option<(Vec<usize>, Vec<usize>)>,
    /// `{"default": true}`: let the engine choose its default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

impl JsonAnswer {
    pub fn index(i: usize) -> Self {
        JsonAnswer {
            index: Some(i),
            ..Default::default()
        }
    }
    pub fn indices(v: Vec<usize>) -> Self {
        JsonAnswer {
            indices: Some(v),
            ..Default::default()
        }
    }
    pub fn number(n: i64) -> Self {
        JsonAnswer {
            number: Some(n),
            ..Default::default()
        }
    }
    pub fn numbers(v: Vec<i64>) -> Self {
        JsonAnswer {
            numbers: Some(v),
            ..Default::default()
        }
    }
    pub fn yes_no(b: bool) -> Self {
        JsonAnswer {
            bool: Some(b),
            ..Default::default()
        }
    }
    pub fn text(t: impl Into<String>) -> Self {
        JsonAnswer {
            text: Some(t.into()),
            ..Default::default()
        }
    }
    pub fn split(a: Vec<usize>, b: Vec<usize>) -> Self {
        JsonAnswer {
            split: Some((a, b)),
            ..Default::default()
        }
    }
    pub fn default_choice() -> Self {
        JsonAnswer {
            default: Some(true),
            ..Default::default()
        }
    }

    /// Parses one line of JSON.
    pub fn parse(line: &str) -> Result<JsonAnswer, String> {
        serde_json::from_str(line.trim()).map_err(|e| format!("invalid answer JSON: {e}"))
    }

    fn field_count(&self) -> usize {
        [
            self.index.is_some(),
            self.indices.is_some(),
            self.number.is_some(),
            self.numbers.is_some(),
            self.bool.is_some(),
            self.text.is_some(),
            self.split.is_some(),
            self.default == Some(true),
        ]
        .iter()
        .filter(|b| **b)
        .count()
    }
}

/// A decision prepared for an agent: the request, and what's needed to turn an answer
/// into the engine's [`Answer`].
#[derive(Clone, Debug)]
pub struct Prepared {
    pub request: Request,
    pub decision: Decision,
    /// For each option: the priority action, or the (creature, attack target / attacker)
    /// pair, or the engine index (modes), or the entity.
    payload: Vec<Payload>,
}

#[derive(Clone, Debug)]
enum Payload {
    None,
    Action(Action),
    Entity(Entity),
    Index(usize),
    Pair(ObjectId, Entity),
}

/// Settings for how decisions are presented.
#[derive(Clone, Debug)]
pub struct PresentOptions {
    pub priority: PriorityOptions,
    /// Include the observation in each request.
    pub observation: bool,
}

impl Default for PresentOptions {
    fn default() -> Self {
        PresentOptions {
            priority: PriorityOptions::default(),
            observation: true,
        }
    }
}

fn action_view(g: &Game, a: &Action) -> ActionView {
    let mut v = ActionView::default();
    match a {
        Action::Pass => v.kind = "pass".into(),
        Action::Concede => v.kind = "concede".into(),
        Action::PlayLand { card } => {
            v.kind = "play_land".into();
            v.card = Some(card.0);
        }
        Action::Cast { card, method } => {
            v.kind = "cast".into();
            v.card = Some(card.0);
            v.method = Some(cast_method_code(method));
            v.mana_cost = g.obj(*card).chars.mana_cost.as_ref().map(|m| m.to_string());
        }
        Action::Activate { source, ability } => {
            let mana = g
                .obj(*source)
                .chars
                .abilities
                .iter()
                .any(|x| x.uid == *ability && x.is_mana_ability());
            v.kind = if mana { "mana_ability" } else { "activate" }.into();
            v.source = Some(source.0);
            v.ability = Some(*ability);
        }
        Action::Special(s) => {
            v.kind = "special".into();
            let (name, card) = match s {
                SpecialAction::TurnFaceUp { obj } => ("turn_face_up", Some(*obj)),
                SpecialAction::Suspend { card } => ("suspend", Some(*card)),
                SpecialAction::Foretell { card } => ("foretell", Some(*card)),
                SpecialAction::Plot { card } => ("plot", Some(*card)),
                SpecialAction::CompanionToHand { card } => ("companion_to_hand", Some(*card)),
                SpecialAction::Static { source, ability } => {
                    v.ability = Some(*ability);
                    v.source = Some(source.0);
                    ("static", None)
                }
                SpecialAction::Offer { .. } => ("offer", None),
                SpecialAction::RollPlanarDie => ("roll_planar_die", None),
                SpecialAction::Other { obj, .. } => ("other", *obj),
            };
            v.special = Some(name.into());
            v.card = card.map(|c| c.0);
        }
    }
    v
}

fn pawprints(mode_text: &str) -> u32 {
    let head = mode_text.split('—').next().unwrap_or("");
    head.matches("{P}").count() as u32
}

fn decision_source(d: &Decision) -> Option<ObjectId> {
    match d {
        Decision::ChooseModes { source, .. }
        | Decision::ChooseX { source, .. }
        | Decision::OptionalCost { source, .. }
        | Decision::ChooseTargets { source, .. }
        | Decision::Divide { source, .. } => Some(*source),
        Decision::ChooseCastingMethod { card, .. } => Some(*card),
        Decision::YesNo { source, .. }
        | Decision::ChooseEntities { source, .. }
        | Decision::ChooseOption { source, .. }
        | Decision::ChooseNumber { source, .. }
        | Decision::NameCard { source, .. } => *source,
        Decision::AssignCombatDamage { creature, .. } => Some(*creature),
        _ => None,
    }
}

/// The `kind` string of a decision.
pub fn decision_kind(d: &Decision) -> &'static str {
    match d {
        Decision::Priority { .. } => "priority",
        Decision::Mulligan { .. } => "mulligan",
        Decision::PutOnBottom { .. } => "put_on_bottom",
        Decision::ChooseModes { .. } => "choose_modes",
        Decision::ChooseX { .. } => "choose_x",
        Decision::ChooseCastingMethod { .. } => "casting_method",
        Decision::OptionalCost { .. } => "optional_cost",
        Decision::ChooseTargets { .. } => "targets",
        Decision::Divide { .. } => "divide",
        Decision::YesNo { .. } => "yes_no",
        Decision::ChooseEntities { .. } => "choose",
        Decision::Order { .. } => "order",
        Decision::ChooseOption { .. } => "choose_option",
        Decision::ChooseNumber { .. } => "choose_number",
        Decision::NameCard { .. } => "name_card",
        Decision::DeclareAttackers { .. } => "declare_attackers",
        Decision::DeclareBlockers { .. } => "declare_blockers",
        Decision::AssignCombatDamage { .. } => "assign_combat_damage",
        Decision::Scry { .. } => "scry",
        Decision::Surveil { .. } => "surveil",
        Decision::ChooseReplacement { .. } => "replacement",
    }
}

fn yes_no_options() -> (Vec<OptionView>, Vec<Payload>) {
    (
        vec![
            OptionView {
                index: 0,
                text: "No".into(),
                value: Some(false),
                ..Default::default()
            },
            OptionView {
                index: 1,
                text: "Yes".into(),
                value: Some(true),
                ..Default::default()
            },
        ],
        vec![Payload::None, Payload::None],
    )
}

/// Prepares `decision`, asked of `seat`'s agent for `player`, as a request with id `id`.
pub fn prepare(
    g: &Game,
    seat: PlayerId,
    player: PlayerId,
    decision: &Decision,
    id: u64,
    opts: &PresentOptions,
) -> Prepared {
    let viewer = Some(seat);
    let ename = |e: Entity| entity_name(g, viewer, e);
    let entity_option = |i: usize, e: Entity| OptionView {
        index: i,
        text: ename(e),
        entity: match e {
            Entity::Object(o) => visible_id(g, viewer, o).map(EntityRef::Object),
            Entity::Player(p) => Some(EntityRef::Player(p.0)),
        },
        ..Default::default()
    };
    let entity_options = |es: &[Entity]| -> (Vec<OptionView>, Vec<Payload>) {
        (
            es.iter()
                .enumerate()
                .map(|(i, e)| entity_option(i, *e))
                .collect(),
            es.iter().map(|e| Payload::Entity(*e)).collect(),
        )
    };
    let text_options = |ts: &[String]| -> (Vec<OptionView>, Vec<Payload>) {
        (
            ts.iter()
                .enumerate()
                .map(|(i, t)| OptionView {
                    index: i,
                    text: t.clone(),
                    ..Default::default()
                })
                .collect(),
            ts.iter()
                .enumerate()
                .map(|(i, _)| Payload::Index(i))
                .collect(),
        )
    };
    let src_name = |s: Option<ObjectId>| s.map(|s| object_name(g, viewer, s));
    let (prompt, spec, (options, payload)): (String, AnswerSpec, (Vec<OptionView>, Vec<Payload>)) =
        match decision {
            Decision::Priority { actions } => {
                let acts = priority_options(g, player, actions, &opts.priority);
                let views: Vec<OptionView> = acts
                    .iter()
                    .enumerate()
                    .map(|(i, a)| OptionView {
                        index: i,
                        text: action_text(g, viewer, a),
                        action: Some(action_view(g, a)),
                        ..Default::default()
                    })
                    .collect();
                let stack = if g.stack.is_empty() {
                    "the stack is empty".to_string()
                } else {
                    format!(
                        "top of the stack: {}",
                        object_name(g, viewer, *g.stack.last().expect("stack"))
                    )
                };
                (
                    format!(
                        "You have priority: turn {}, {}'s {} step; {}.",
                        g.turn.number,
                        player_name(g.turn.active),
                        step_name(g.turn.step).replace('_', " "),
                        stack
                    ),
                    AnswerSpec::ChooseOne,
                    (views, acts.into_iter().map(Payload::Action).collect()),
                )
            }
            Decision::Mulligan { mulligans_taken } => (
                format!(
                    "Take a mulligan? (yes = mulligan, no = keep this hand; mulligans taken so far: {mulligans_taken})"
                ),
                AnswerSpec::YesNo,
                yes_no_options(),
            ),
            Decision::PutOnBottom { cards, n } => (
                format!("Choose {n} card(s) to put on the bottom of your library."),
                AnswerSpec::ChooseMany {
                    min: *n,
                    max: *n,
                    distinct: true,
                    budget: None,
                },
                entity_options(&cards.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>()),
            ),
            Decision::ChooseModes {
                modes,
                min,
                max,
                allow_repeat,
                available,
                pawprint_budget,
                ..
            } => {
                let avail: Vec<usize> = if available.is_empty() {
                    (0..modes.len()).collect()
                } else {
                    available.clone()
                };
                let views = avail
                    .iter()
                    .enumerate()
                    .map(|(i, m)| OptionView {
                        index: i,
                        text: modes[*m].clone(),
                        cost: pawprint_budget.map(|_| pawprints(&modes[*m])),
                        ..Default::default()
                    })
                    .collect();
                let max = if *allow_repeat {
                    *max
                } else {
                    (*max).min(avail.len() as u32)
                };
                (
                    format!(
                        "Choose {} mode(s){}.",
                        if min == &max {
                            min.to_string()
                        } else {
                            format!("{min} to {max}")
                        },
                        if *allow_repeat {
                            " (the same mode may be chosen more than once)"
                        } else {
                            ""
                        }
                    ),
                    AnswerSpec::ChooseMany {
                        min: *min,
                        max,
                        distinct: !allow_repeat,
                            budget: *pawprint_budget,
                    },
                    (views, avail.into_iter().map(Payload::Index).collect()),
                )
            }
            Decision::ChooseX { max, .. } => (
                format!("Choose the value of X (0 to {max})."),
                AnswerSpec::Number {
                    min: 0,
                    max: Some(*max),
                },
                (vec![], vec![]),
            ),
            Decision::ChooseCastingMethod { options, .. } => (
                "Choose how to cast this spell.".into(),
                AnswerSpec::ChooseOne,
                text_options(options),
            ),
            Decision::OptionalCost {
                name, repeatable, ..
            } => {
                if *repeatable {
                    (
                        format!("How many times do you pay the {name} cost?"),
                        AnswerSpec::Number { min: 0, max: None },
                        (vec![], vec![]),
                    )
                } else {
                    (
                        format!("Pay the optional {name} cost?"),
                        AnswerSpec::YesNo,
                        yes_no_options(),
                    )
                }
            }
            Decision::ChooseTargets {
                text,
                candidates,
                min,
                max,
                ..
            } => (
                format!("Choose {}: {text}", count_text(*min, *max, "target")),
                AnswerSpec::ChooseMany {
                    min: *min,
                    max: *max,
                    distinct: true,
                    budget: None,
                },
                entity_options(candidates),
            ),
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => (
                format!("Divide {total} among these, at least {min_each} each."),
                AnswerSpec::Divide {
                    total: *total,
                    min_each: *min_each,
                },
                entity_options(recipients),
            ),
            Decision::YesNo { prompt, .. } => {
                (prompt.clone(), AnswerSpec::YesNo, yes_no_options())
            }
            Decision::ChooseEntities {
                prompt,
                candidates,
                min,
                max,
                ..
            } => (
                format!("{prompt} ({})", count_text(*min, *max, "choice")),
                AnswerSpec::ChooseMany {
                    min: *min,
                    max: *max,
                    distinct: true,
                    budget: None,
                },
                entity_options(candidates),
            ),
            Decision::Order { prompt, items } => {
                (prompt.clone(), AnswerSpec::Order, text_options(items))
            }
            Decision::ChooseOption {
                prompt, options, ..
            } => (prompt.clone(), AnswerSpec::ChooseOne, text_options(options)),
            Decision::ChooseNumber {
                prompt, min, max, ..
            } => (
                format!("{prompt} ({min} to {max})"),
                AnswerSpec::Number {
                    min: *min,
                    max: Some(*max),
                },
                (vec![], vec![]),
            ),
            Decision::NameCard { prompt, .. } => (
                format!("{prompt} (the exact English name of a card)"),
                AnswerSpec::Text {
                    what: "card_name".into(),
                },
                (vec![], vec![]),
            ),
            Decision::DeclareAttackers { options } => {
                let mut views = Vec::new();
                let mut pay = Vec::new();
                for (a, ts) in options {
                    for t in ts {
                        views.push(OptionView {
                            index: views.len(),
                            text: format!("{} attacks {}", object_name(g, viewer, *a), ename(*t)),
                            group: Some(a.0),
                            group_max: Some(1),
                            target: Some(EntityRef::from(*t)),
                            ..Default::default()
                        });
                        pay.push(Payload::Pair(*a, *t));
                    }
                }
                (
                    "Declare attackers: choose the attacking creatures and what each attacks (none to not attack). The declaration must obey attack requirements and restrictions (CR 508.1c-d).".into(),
                    AnswerSpec::ChooseMany {
                        min: 0,
                        max: options.len() as u32,
                        distinct: true,
                        budget: None,
                    },
                    (views, pay),
                )
            }
            Decision::DeclareBlockers { options } => {
                let mut views = Vec::new();
                let mut pay = Vec::new();
                for (b, atts) in options {
                    for a in atts {
                        views.push(OptionView {
                            index: views.len(),
                            text: format!(
                                "{} blocks {}",
                                object_name(g, viewer, *b),
                                object_name(g, viewer, *a)
                            ),
                            group: Some(b.0),
                            group_max: g.max_blocks(*b),
                            target: Some(EntityRef::Object(a.0)),
                            ..Default::default()
                        });
                        pay.push(Payload::Pair(*b, Entity::Object(*a)));
                    }
                }
                let n = views.len() as u32;
                (
                    "Declare blockers: choose blocking creatures and the attacker each blocks (none to not block). The declaration must obey blocking requirements and restrictions (CR 509.1b-c); a creature may block several attackers only if an effect allows it.".into(),
                    AnswerSpec::ChooseMany {
                        min: 0,
                        max: n,
                        distinct: true,
                            budget: None,
                    },
                    (views, pay),
                )
            }
            Decision::AssignCombatDamage {
                creature,
                amount,
                recipients,
                lethal,
                trample,
            } => {
                let (mut views, pay) = entity_options(recipients);
                for (v, l) in views.iter_mut().zip(lethal) {
                    v.lethal = Some(*l);
                }
                (
                    format!(
                        "Assign {amount} combat damage dealt by {}.",
                        object_name(g, viewer, *creature)
                    ),
                    AnswerSpec::AssignDamage {
                        total: *amount,
                        trample: *trample,
                    },
                    (views, pay),
                )
            }
            Decision::Scry { cards } => (
                "Scry: choose the cards to keep on top (in order, first = top) and the cards to put on the bottom.".into(),
                AnswerSpec::Split {
                    first: "top".into(),
                    second: "bottom".into(),
                },
                entity_options(&cards.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>()),
            ),
            Decision::Surveil { cards } => (
                "Surveil: choose the cards to keep on top (in order, first = top) and the cards to put into your graveyard.".into(),
                AnswerSpec::Split {
                    first: "top".into(),
                    second: "graveyard".into(),
                },
                entity_options(&cards.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>()),
            ),
            Decision::ChooseReplacement { options } => (
                "Choose which replacement or prevention effect to apply first (CR 616.1).".into(),
                AnswerSpec::ChooseOne,
                text_options(options),
            ),
        };
    let source = decision_source(decision);
    let request = Request {
        msg_type: "decision".into(),
        version: PROTOCOL_VERSION,
        id,
        seat: seat.0,
        player: player.0,
        kind: decision_kind(decision).into(),
        prompt,
        source: source.and_then(|s| visible_id(g, viewer, s)),
        source_name: src_name(source),
        answer: spec,
        options,
        events: vec![],
        observation: opts.observation.then(|| observe(g, viewer)),
        error: None,
        attempt: 0,
    };
    Prepared {
        request,
        decision: decision.clone(),
        payload,
    }
}

fn count_text(min: u32, max: u32, what: &str) -> String {
    if min == max {
        format!("{min} {what}(s)")
    } else {
        format!("{min} to {max} {what}(s)")
    }
}

impl Prepared {
    fn n(&self) -> usize {
        self.request.options.len()
    }

    fn check_index(&self, i: usize) -> Result<(), String> {
        if i < self.n() {
            Ok(())
        } else {
            Err(format!(
                "option index {i} out of range (there are {} options, 0 to {})",
                self.n(),
                self.n().saturating_sub(1)
            ))
        }
    }

    fn want(&self, what: &str) -> String {
        format!("this {} decision expects {what}", self.request.kind)
    }

    /// Validates `ans` for this decision and converts it to the engine's answer.
    pub fn convert(&self, g: &Game, ans: &JsonAnswer) -> Result<Answer, String> {
        if let Some(id) = ans.id {
            if id != self.request.id {
                return Err(format!(
                    "answer id {id} doesn't match the request id {}",
                    self.request.id
                ));
            }
        }
        match ans.field_count() {
            0 => {
                return Err(
                    "the answer has no answer field (index, indices, number, numbers, bool, text, split or default)"
                        .into(),
                )
            }
            1 => {}
            _ => return Err("the answer has more than one answer field".into()),
        }
        if ans.default == Some(true) {
            return Ok(Answer::Default);
        }
        match &self.request.answer {
            AnswerSpec::ChooseOne => {
                let i = ans.index.ok_or_else(|| self.want("{\"index\": i}"))?;
                self.check_index(i)?;
                Ok(match (&self.decision, &self.payload[i]) {
                    (_, Payload::Action(a)) => Answer::Action(a.clone()),
                    _ => Answer::Index(i),
                })
            }
            AnswerSpec::YesNo => {
                let b = match (ans.bool, ans.index) {
                    (Some(b), _) => b,
                    (None, Some(0)) => false,
                    (None, Some(1)) => true,
                    (None, Some(i)) => {
                        return Err(format!("option index {i} out of range (0 = no, 1 = yes)"))
                    }
                    _ => return Err(self.want("{\"bool\": true|false}")),
                };
                Ok(Answer::Bool(b))
            }
            AnswerSpec::Number { min, max } => {
                let n = ans.number.ok_or_else(|| self.want("{\"number\": n}"))?;
                if n < *min || max.is_some_and(|m| n > m) {
                    return Err(match max {
                        Some(m) => format!("number {n} out of range ({min} to {m})"),
                        None => format!("number {n} is less than {min}"),
                    });
                }
                Ok(Answer::Number(n))
            }
            AnswerSpec::Text { .. } => {
                let t = ans
                    .text
                    .as_ref()
                    .ok_or_else(|| self.want("{\"text\": \"...\"}"))?;
                let t = t.trim();
                if !mtg_engine::choices::valid_card_name(t, None) {
                    return Err(format!("\"{t}\" isn't the name of a card"));
                }
                Ok(Answer::Text(t.to_string()))
            }
            AnswerSpec::Order => {
                let v = ans
                    .indices
                    .as_ref()
                    .ok_or_else(|| self.want("{\"indices\": [...]}"))?;
                let mut s = v.clone();
                s.sort_unstable();
                if s != (0..self.n()).collect::<Vec<_>>() {
                    return Err(format!(
                        "the indices must be a permutation of 0..{} (each option exactly once)",
                        self.n()
                    ));
                }
                Ok(Answer::Indices(v.clone()))
            }
            AnswerSpec::Split { .. } => {
                let (a, b) = ans
                    .split
                    .as_ref()
                    .ok_or_else(|| self.want("{\"split\": [[...], [...]]}"))?;
                let mut s: Vec<usize> = a.iter().chain(b).copied().collect();
                s.sort_unstable();
                if s != (0..self.n()).collect::<Vec<_>>() {
                    return Err(format!(
                        "the two lists must together contain each option index 0..{} exactly once",
                        self.n()
                    ));
                }
                let obj = |i: &usize| match &self.payload[*i] {
                    Payload::Entity(Entity::Object(o)) => *o,
                    _ => ObjectId(0),
                };
                Ok(Answer::Split(
                    a.iter().map(obj).collect(),
                    b.iter().map(obj).collect(),
                ))
            }
            AnswerSpec::Divide { total, min_each } => {
                let v = ans
                    .numbers
                    .as_ref()
                    .ok_or_else(|| self.want("{\"numbers\": [...]}"))?;
                if v.len() != self.n() {
                    return Err(format!("expected {} numbers, one per option", self.n()));
                }
                if v.iter().any(|x| *x < *min_each as i64) {
                    return Err(format!("each number must be at least {min_each}"));
                }
                if v.iter().sum::<i64>() != *total as i64 {
                    return Err(format!("the numbers must add up to {total}"));
                }
                Ok(Answer::Numbers(v.clone()))
            }
            AnswerSpec::AssignDamage { total, trample } => {
                let v = ans
                    .numbers
                    .as_ref()
                    .ok_or_else(|| self.want("{\"numbers\": [...]}"))?;
                self.check_damage(g, v, *total, *trample)?;
                Ok(Answer::Numbers(v.clone()))
            }
            AnswerSpec::ChooseMany {
                min,
                max,
                distinct,
                budget,
            } => {
                let v = ans
                    .indices
                    .as_ref()
                    .ok_or_else(|| self.want("{\"indices\": [...]}"))?;
                for i in v {
                    self.check_index(*i)?;
                }
                if (v.len() as u32) < *min || (v.len() as u32) > *max {
                    return Err(format!(
                        "choose between {min} and {max} options (got {})",
                        v.len()
                    ));
                }
                if *distinct && v.iter().collect::<BTreeSet<_>>().len() != v.len() {
                    return Err("each option may be chosen only once".into());
                }
                let mut per_group: std::collections::BTreeMap<u32, u32> = Default::default();
                for i in v {
                    let o = &self.request.options[*i];
                    if let Some(gr) = o.group {
                        let n = per_group.entry(gr).or_default();
                        *n += 1;
                        if o.group_max.is_some_and(|m| *n > m) {
                            return Err(format!(
                                "at most {} option(s) of group #{gr} may be chosen",
                                o.group_max.unwrap_or(0)
                            ));
                        }
                    }
                }
                if let Some(b) = budget {
                    let total: u32 = v
                        .iter()
                        .map(|i| self.request.options[*i].cost.unwrap_or(0))
                        .sum();
                    if total > *b {
                        return Err(format!("the chosen options cost {total}, more than {b}"));
                    }
                }
                self.convert_many(g, v)
            }
        }
    }

    fn convert_many(&self, g: &Game, v: &[usize]) -> Result<Answer, String> {
        match &self.decision {
            Decision::ChooseModes { .. } => Ok(Answer::Indices(
                v.iter()
                    .map(|i| match self.payload[*i] {
                        Payload::Index(m) => m,
                        _ => *i,
                    })
                    .collect(),
            )),
            Decision::DeclareAttackers { options } => {
                let decl: Vec<(ObjectId, Entity)> = v
                    .iter()
                    .filter_map(|i| match self.payload[*i] {
                        Payload::Pair(a, t) => Some((a, t)),
                        _ => None,
                    })
                    .collect();
                if !mtg_engine::combat::attack_declaration_legal(g, options, &decl) {
                    return Err("illegal attack: it breaks an attack restriction or doesn't obey as many attack requirements as possible (CR 508.1c-d)".into());
                }
                let ap = g.turn.active;
                let cost = mtg_engine::combat::total_attack_cost(g, &decl);
                if !cost.is_free() && !g.can_pay_cost(ap, &cost, None, &Ctx::new(None, ap)) {
                    return Err("you can't pay the costs to attack with those creatures".into());
                }
                Ok(Answer::Attackers(decl))
            }
            Decision::DeclareBlockers { options } => {
                let decl: Vec<(ObjectId, ObjectId)> = v
                    .iter()
                    .filter_map(|i| match self.payload[*i] {
                        Payload::Pair(b, Entity::Object(a)) => Some((b, a)),
                        _ => None,
                    })
                    .collect();
                for (_, a) in &decl {
                    let n = decl.iter().filter(|(_, x)| x == a).count() as u32;
                    if n < g.min_blockers(*a) {
                        return Err(format!(
                            "{} can't be blocked except by {} or more creatures",
                            object_name(g, Some(PlayerId(self.request.seat)), *a),
                            g.min_blockers(*a)
                        ));
                    }
                    if g.max_blocked_by(*a).is_some_and(|m| n > m) {
                        return Err(format!(
                            "{} can't be blocked by more than {} creature(s)",
                            object_name(g, Some(PlayerId(self.request.seat)), *a),
                            g.max_blocked_by(*a).unwrap_or(0)
                        ));
                    }
                }
                if !mtg_engine::combat::block_declaration_legal(g, options, &decl) {
                    return Err("illegal block: it breaks a blocking restriction or doesn't obey as many blocking requirements as possible (CR 509.1b-c)".into());
                }
                let payable = mtg_engine::combat::block_costs_by_player(g, &decl)
                    .iter()
                    .all(|(p, c)| g.can_pay_cost(*p, c, None, &Ctx::new(None, *p)));
                if !payable {
                    return Err("the costs to block that way can't be paid".into());
                }
                Ok(Answer::Blockers(decl))
            }
            _ => Ok(Answer::Entities(
                v.iter()
                    .filter_map(|i| match self.payload[*i] {
                        Payload::Entity(e) => Some(e),
                        _ => None,
                    })
                    .collect(),
            )),
        }
    }

    fn check_damage(&self, g: &Game, v: &[i64], total: u32, trample: bool) -> Result<(), String> {
        let Decision::AssignCombatDamage {
            creature,
            recipients,
            lethal,
            ..
        } = &self.decision
        else {
            return Err("not a damage assignment".into());
        };
        if v.len() != recipients.len() {
            return Err(format!(
                "expected {} numbers, one per option",
                recipients.len()
            ));
        }
        if v.iter().any(|x| *x < 0) {
            return Err("damage amounts can't be negative".into());
        }
        if v.iter().sum::<i64>() != total as i64 {
            return Err(format!("the amounts must add up to {total}"));
        }
        if !trample {
            return Ok(());
        }
        // The attacked planeswalker's controller follows the planeswalker (CR 702.19c).
        let target = g.combat.as_ref().and_then(|c| c.attack_target(*creature));
        let spill = matches!(target, Some(Entity::Object(_)))
            && matches!(recipients.last(), Some(Entity::Player(_)))
            && recipients.len() >= 2
            && Some(recipients[recipients.len() - 2]) == target;
        let nb = recipients.len() - 1 - usize::from(spill);
        let beyond: i64 = v[nb..].iter().sum();
        if beyond > 0 && (0..nb).any(|i| (v[i] as u32) < lethal.get(i).copied().unwrap_or(0)) {
            return Err("with trample, each blocking creature must be assigned lethal damage before any is assigned beyond them (CR 702.19b)".into());
        }
        if spill && v[nb + 1] > 0 && (v[nb] as u32) < lethal.get(nb).copied().unwrap_or(0) {
            return Err("the planeswalker must be assigned lethal damage before its controller is assigned any (CR 702.19c)".into());
        }
        Ok(())
    }
}
