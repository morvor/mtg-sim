//! Trigger events qualified by the spell or ability they're about, or by a condition on
//! the event that the event's own wording carries: "Whenever you activate a loyalty
//! ability of a Chandra planeswalker" (CR 606), "an artifact's ability without {T} in its
//! activation cost" (CR 602.1a), "Whenever you or a permanent you control becomes the
//! target of a spell or ability an opponent controls" (CR 115.10), "a creature entering
//! causes a triggered ability of that creature to trigger" (CR 603.2), "attacks the
//! player with the most life or tied for most life" (dethrone's condition, CR 702.105a),
//! "At the end of the first combat phase on your turn" (CR 506.1, 511).

use super::nouns::Det;
use super::*;
use crate::kw::activated_ability_kind as aak;

/// What kind of activated ability a `Condition` on an "activates an ability" event says
/// it is, as a noun phrase without article and whether "an" goes before it.
fn ability_kind(cond: &Condition) -> Option<&'static str> {
    Some(match cond {
        Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(n))
            if n == crate::stack_ability_filters::LOYALTY_ABILITY =>
        {
            "a loyalty ability"
        }
        Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(n))
            if n == crate::stack_ability_filters::X_IN_ACTIVATION_COST =>
        {
            "an ability with {X} in its activation cost"
        }
        Condition::Custom(n) if n == aak::NINJUTSU => "a ninjutsu ability",
        Condition::Custom(n) if n == aak::POWER_UP => "a power-up ability",
        Condition::Custom(n) if n == "outlast:the activated ability is an outlast ability" => {
            "an outlast ability"
        }
        Condition::Custom(n)
            if n == "embalm:the activated ability is an embalm or eternalize ability" =>
        {
            "{alt:an eternalize or embalm ability|an embalm or eternalize ability}"
        }
        _ => return None,
    })
}

/// "from anywhere other than your hand" (CR 305.1: a land played is played from its
/// owner's hand unless an effect allows otherwise).
const FROM_ELSEWHERE: &str = "from anywhere other than {alt:your|their} hand";

/// The filter without "came from a hand" negated, if it has that.
fn not_from_hand(f: &Filter) -> Option<Filter> {
    let is_it = |x: &Filter| matches!(x, Filter::Not(c) if matches!(c.as_ref(), Filter::Custom(n) if n == "came from:Hand"));
    let Filter::And(v) = f else {
        return None;
    };
    if !v.iter().any(is_it) {
        return None;
    }
    Some(Filter::and(
        v.iter().filter(|x| !is_it(x)).cloned().collect(),
    ))
}

impl Renderer<'_> {
    /// "you play a land or cast a spell from anywhere other than your hand": both events
    /// qualified the same way, said once.
    fn land_or_spell_from_elsewhere(&mut self, v: &[TriggerCond]) -> Option<(String, String)> {
        let [TriggerCond::LandPlayed { who, filter }, TriggerCond::CastSpell {
            who: w2,
            filter: sf,
        }] = v
        else {
            return None;
        };
        if who != w2 {
            return None;
        }
        let land = not_from_hand(filter)?;
        let Filter::And(sv) = sf else {
            return None;
        };
        let not_cast_from_hand = |x: &Filter| matches!(x, Filter::Not(c) if matches!(c.as_ref(), Filter::CastFrom(ZoneKind::Hand)));
        if !sv.iter().any(not_cast_from_hand) {
            return None;
        }
        let spell = Filter::and(
            sv.iter()
                .filter(|x| !not_cast_from_hand(x))
                .cloned()
                .collect(),
        );
        let w = self.rel_subject(*who);
        let l = self.noun_det(&land, Det::A);
        let s = self.spell_noun(&spell, Det::A);
        Some((w, format!("play {l} or cast {s} {FROM_ELSEWHERE}")))
    }

    /// The (subject, verb phrase) of a trigger event this module words, if `t` is one.
    pub(crate) fn qualified_trigger_event(
        &mut self,
        t: &TriggerCond,
        det: &Det,
    ) -> Option<(String, String)> {
        match t {
            TriggerCond::Where { trigger, cond } => self.qualified_where(trigger, cond, det),
            TriggerCond::Custom(n) => {
                let (who, by) = aak::player_targeted_parts(n)?;
                let w = self.rel_subject(who);
                let by = self.targeting_controller(by);
                Some((w, format!("become the target of a spell or ability{by}")))
            }
            // "Whenever an opponent attacks with creatures": a player attacks with
            // creatures (CR 508.1a).
            TriggerCond::PlayerAttacks(r) => {
                let w = self.rel_subject(*r);
                Some((w, "attack {opt:with creatures}".into()))
            }
            TriggerCond::AnyOf(v) => self
                .player_or_permanent_targeted(v, false)
                .or_else(|| self.land_or_spell_from_elsewhere(v)),
            // "Whenever a land you control enters from anywhere other than your hand".
            TriggerCond::EntersBattlefield(f) => {
                let rest = not_from_hand(f)?;
                let o = self.noun_det(&rest, det.clone());
                Some((o, format!("enters {FROM_ELSEWHERE}")))
            }
            TriggerCond::LandPlayed { who, filter } => {
                let rest = not_from_hand(filter)?;
                let w = self.rel_subject(*who);
                let o = self.noun_det(&rest, Det::A);
                Some((w, format!("play {o} {FROM_ELSEWHERE}")))
            }
            TriggerCond::Batched {
                trigger,
                per: BatchPer::Batch,
            } => match trigger.as_ref() {
                TriggerCond::AnyOf(v) => self.player_or_permanent_targeted(v, true),
                _ => None,
            },
            _ => None,
        }
    }

    /// " an opponent controls", " you control", or nothing.
    fn targeting_controller(&mut self, by: PlayerRel) -> String {
        match by {
            PlayerRel::Any => String::new(),
            PlayerRel::Opponent => " an opponent controls".into(),
            PlayerRel::You => " you control".into(),
            r => format!(" {}", self.controls_phrase(r, Num::One)),
        }
    }

    /// "you or a permanent you control becomes the target of a spell or ability an
    /// opponent controls" (`batch`: "you and/or at least one permanent you control", once
    /// for all of them, CR 603.2c).
    fn player_or_permanent_targeted(
        &mut self,
        v: &[TriggerCond],
        batch: bool,
    ) -> Option<(String, String)> {
        let [TriggerCond::Custom(n), TriggerCond::BecomesTarget { filter, by }] = v else {
            return None;
        };
        let (who, by2) = aak::player_targeted_parts(n)?;
        if by2 != *by {
            return None;
        }
        let w = self.rel_subject(who);
        let by = self.targeting_controller(*by);
        let o = if batch {
            let n = self.noun(filter, Num::One);
            format!("{{alt:at least one|one or more}} {n}")
        } else {
            self.noun_det(filter, Det::A)
        };
        let (join, o) = match o.strip_prefix("a ").or_else(|| o.strip_prefix("an ")) {
            // "a player or permanent becomes the target".
            Some(rest) if w == "a player" && !batch => ("or", rest.to_string()),
            _ if batch => ("and/or", o),
            _ => ("or", o),
        };
        Some((
            format!("{w} {join} {o}"),
            format!("becomes the target of a spell or ability{by}"),
        ))
    }

    fn qualified_where(
        &mut self,
        trigger: &TriggerCond,
        cond: &Condition,
        det: &Det,
    ) -> Option<(String, String)> {
        // "Whenever you become the target of a spell": only spells, or only abilities.
        if let Condition::SelMatches(Sel::TriggerSpell, f @ (Filter::Spell | Filter::Not(_))) = cond
        {
            let what = match f {
                Filter::Spell => "a spell",
                Filter::Not(x) if matches!(x.as_ref(), Filter::Spell) => "an ability",
                _ => return None,
            };
            let (s, vp) = self.qualified_trigger_event(trigger, det)?;
            if !vp.contains("a spell or ability") {
                return None;
            }
            return Some((s, vp.replacen("a spell or ability", what, 1)));
        }
        match (trigger, cond) {
            (
                TriggerCond::AbilityActivated {
                    who,
                    source,
                    include_mana,
                },
                cond,
            ) => {
                let w = self.rel_subject(*who);
                // "a player activates an artifact's ability without {T} in its activation
                // cost".
                if let Condition::Not(c) = cond {
                    // Mana abilities count too ("an artifact's ability", CR 605).
                    if matches!(c.as_ref(), Condition::Custom(n) if n == aak::TAP_IN_COST)
                        && *include_mana
                    {
                        let of = self.ability_owner(source);
                        return Some((
                            w,
                            format!("activate {of} without {{T}} in its activation cost"),
                        ));
                    }
                    return None;
                }
                // "a player activates an ability of enchanted creature with {T} in its
                // activation cost that isn't a mana ability" (CR 605.1a).
                if matches!(cond, Condition::Custom(n) if n == aak::TAP_IN_COST) && !*include_mana {
                    let o = self.noun_det(source, Det::A);
                    return Some((
                        w,
                        format!("activate an ability of {o} with {{T}} in its activation cost that isn't a mana ability"),
                    ));
                }
                let kind = ability_kind(cond)?;
                let vp = match source {
                    Filter::Any | Filter::Permanent => format!("activate {kind}"),
                    Filter::Source => {
                        let k = kind
                            .strip_prefix("a ")
                            .or_else(|| kind.strip_prefix("an "))
                            .unwrap_or(kind);
                        format!("activate {{alt:~'s {k}|{kind} of ~}}")
                    }
                    other => {
                        let o = self.noun_det(other, Det::A);
                        format!("activate {kind} of {o}")
                    }
                };
                Some((w, vp))
            }
            // "Whenever a creature you control becomes the target of a backup ability".
            (
                TriggerCond::BecomesTarget { filter, by },
                Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(n)),
            ) if n == aak::BACKUP_ABILITY => {
                let o = self.noun_det(filter, det.clone());
                let by = self.targeting_controller(*by);
                Some((o, format!("becomes the target of a backup ability{by}")))
            }
            // "Whenever a creature entering under an opponent's control causes a triggered
            // ability of that creature to trigger".
            (
                TriggerCond::AbilityTriggered { cause, source },
                Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(n)),
            ) if n == crate::stack_ability_filters::OF_THE_TRIGGERING_OBJECT
                && matches!(source, Filter::Any | Filter::Permanent) =>
            {
                let (event, kind) = self.event_gerund(cause)?;
                Some((
                    format!("{event} causes a triggered ability of that {kind}"),
                    "to trigger".into(),
                ))
            }
            // "Whenever one or more creatures attack one of your opponents", "Whenever that
            // creature attacks the monarch": attacking a player (not a planeswalker or
            // battle, CR 508.1b) who is such a player.
            (TriggerCond::Attacks(f), Condition::And(cs))
                if matches!(cs.as_slice(), [
                    Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Opponent | PlayerFilter::Monarch),
                    Condition::Not(n),
                ] if matches!(n.as_ref(), Condition::SelNonEmpty(Sel::TriggerOtherObject))) =>
            {
                let who = match &cs[0] {
                    Condition::PlayerMatches(_, PlayerFilter::Monarch) => "the monarch",
                    _ => "{alt:one of your opponents|an opponent}",
                };
                let o = self.noun_det(f, det.clone());
                Some((o, format!("attacks {who}")))
            }
            // "Whenever ~ attacks the player with the most life or tied for most life".
            (
                TriggerCond::AttacksRecipient {
                    attacker,
                    recipient: DamageRecipient::Player(PlayerRel::Any),
                },
                Condition::Custom(n),
            ) if n == "dethrone:that player has the most life" => {
                let a = match attacker {
                    Filter::Source => self.me(),
                    f => self.noun_det(f, det.clone()),
                };
                Some((
                    a,
                    "attacks the player with the most life or tied for most life".into(),
                ))
            }
            // "Whenever you attack the player who has the initiative".
            (
                TriggerCond::PlayerAttacked {
                    attacker,
                    defender: PlayerFilter::Any,
                    with,
                    min: 0 | 1,
                },
                Condition::Custom(n),
            ) if n == aak::EVENT_PLAYER_HAS_INITIATIVE
                && matches!(with, Filter::Any | Filter::Type(CardType::Creature)) =>
            {
                let w = self.rel_subject(*attacker);
                Some((
                    w,
                    "attack the player who has the initiative {opt:with one or more creatures}"
                        .into(),
                ))
            }
            // "At the end of the first combat phase on your turn": the end of combat step
            // of the turn's first combat phase (CR 511.1).
            (
                TriggerCond::BeginningOf {
                    step: TriggerStep::EndOfCombat,
                    whose,
                },
                Condition::Custom(n),
            ) if n == aak::FIRST_COMBAT_PHASE => {
                let on = match whose {
                    PlayerRel::You => " on your turn".to_string(),
                    PlayerRel::Any => String::new(),
                    r => {
                        let p = self.rel_possessive(*r, Num::One);
                        format!(" on {p} turn")
                    }
                };
                Some((
                    String::new(),
                    format!("the end of the first combat phase{on}"),
                ))
            }
            // "When a Faerie is championed with ~" (CR 702.72a).
            (
                TriggerCond::Custom(n),
                Condition::SelMatches(Sel::TriggerLki | Sel::TriggerObject, f),
            ) if n == "champion:championed with this" => {
                let o = self.noun_det(f, Det::A);
                let m = self.me();
                Some((o, format!("is championed with {m}")))
            }
            _ => None,
        }
    }

    /// "an artifact's ability", "an ability of enchanted artifact".
    fn ability_owner(&mut self, source: &Filter) -> String {
        let f = match source {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|x| !matches!(x, Filter::Permanent))
                    .cloned()
                    .collect(),
            ),
            other => other.clone(),
        };
        match &f {
            Filter::Any | Filter::Permanent => "an ability".into(),
            Filter::Source => "{alt:~'s ability|an ability of ~}".into(),
            other => {
                let o = self.noun_det(other, Det::A);
                if let Some(rest) = o.strip_prefix("a ").or_else(|| o.strip_prefix("an ")) {
                    let poss = super::nouns::possessive(&o);
                    format!(
                        "{{alt:{poss} ability|an ability of {}}}",
                        with_article(rest)
                    )
                } else {
                    format!("an ability of {o}")
                }
            }
        }
    }

    /// An event as a gerund phrase, with the kind of object it's about ("a creature you
    /// control attacking", "a creature entering under an opponent's control").
    fn event_gerund(&mut self, t: &TriggerCond) -> Option<(String, String)> {
        let (f, verb) = match t {
            TriggerCond::EntersBattlefield(f) => (f, "entering"),
            TriggerCond::Attacks(f) => (f, "attacking"),
            TriggerCond::Dies(f) => (f, "dying"),
            _ => return None,
        };
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let kind = atoms.iter().find_map(|a| match a {
            Filter::Type(t) => Some(t.word().to_lowercase()),
            _ => None,
        })?;
        // "entering under an opponent's control": who controls it as it enters.
        if verb == "entering" {
            let ctl = atoms.iter().find_map(|a| match a {
                Filter::ControlledBy(r) => Some(*r),
                _ => None,
            });
            if let Some(r) = ctl {
                let rest: Vec<Filter> = atoms
                    .iter()
                    .filter(|a| !matches!(a, Filter::ControlledBy(_)))
                    .map(|a| (*a).clone())
                    .collect();
                let o = self.noun_det(&Filter::and(rest), Det::A);
                let p = self.rel_possessive(r, Num::One);
                let plain = self.noun_det(f, Det::A);
                return Some((
                    format!("{{alt:{o} entering under {p} control|{plain} entering}}"),
                    kind,
                ));
            }
        }
        let o = self.noun_det(f, Det::A);
        Some((format!("{o} {verb}"), kind))
    }
}
