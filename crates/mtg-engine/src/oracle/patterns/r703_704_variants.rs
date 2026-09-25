//! Oracle patterns for nontraditional cards and their keyword actions:
//!
//! - "When you encounter [this phenomenon]" (CR 312.5).
//! - "When you set this scheme in motion" (CR 701.32, 904.9) and "abandon this scheme"
//!   (CR 701.33).
//! - Dungeon rooms: "[Room name] — [effect] (Leads to: [rooms])" (CR 309.4c) and "venture
//!   into the dungeon" (CR 701.49).
//! - Visit abilities of Attractions: "Visit — [effect]" (CR 702.159, 717.5) and "roll to
//!   visit your Attractions" (CR 701.52).

use super::{AbilityPattern, BlockGroupPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;
use crate::variants::{ENCOUNTER, SET_THIS_IN_MOTION, VISIT};
use smol_str::SmolStr;

fn variant_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let name = match end(r) {
        "you encounter ~" | "you encounter this phenomenon" | "you encounter this" => ENCOUNTER,
        "you set ~ in motion" => SET_THIS_IN_MOTION,
        _ => return None,
    };
    Some((
        TriggerCond::Custom(SmolStr::new(name)),
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "r704 encounter / set in motion", priority: 100, parse: variant_triggers } }

fn keyword_action(action: KeywordAction, what: Sel) -> Effect {
    Effect::KeywordAction {
        action,
        who: PlayerRef::You,
        what,
        n: Value::c(1),
    }
}

fn variant_actions(l: &str, _b: &mut Builder) -> Option<Effect> {
    Some(match end(l) {
        "venture into the dungeon" | "you venture into the dungeon" => {
            keyword_action(KeywordAction::Venture, Sel::None)
        }
        "abandon ~" => keyword_action(KeywordAction::Abandon, Sel::This),
        "set the top scheme of your scheme deck in motion" => {
            keyword_action(KeywordAction::SetInMotion, Sel::None)
        }
        "roll to visit your attractions" => {
            keyword_action(KeywordAction::RollAttractions, Sel::None)
        }
        _ => return None,
    })
}

inventory::submit! { EffectPattern { name: "r704 venture / abandon / set in motion / roll to visit", priority: 0, parse: variant_actions } }

/// Normalizes typography the way the compiler's normalizer does.
fn typo(s: &str) -> String {
    s.replace('\u{2019}', "'").replace('\u{2014}', "—")
}

/// The rooms printed on a dungeon card: (name, names of the rooms its arrows lead to).
fn printed_rooms(raw: &str) -> Vec<(String, Vec<String>)> {
    typo(raw)
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(" — ")?;
            let leads = rest
                .split_once("(Leads to: ")
                .and_then(|(_, r)| r.split_once(')'))
                .map(|(l, _)| l.split(", ").map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            Some((name.trim().to_string(), leads))
        })
        .collect()
}

/// Marks an Attraction's visit ability during grouping.
const VISIT_MARK: &str = "{VISIT} ";

/// Room names and "Visit" look like ability words ("Cave Entrance — Scry 1."): mark them
/// so they reach the patterns below intact. A room line is marked with its room ability's
/// trigger name ("{room:0>1,2} Cave Entrance — Scry 1."), which records the dungeon's map
/// (from the "(Leads to: ...)" text) even if the room's effect isn't supported.
fn mark_rooms_and_visits(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    let dungeon = ctx.type_line.card_types.contains(CardType::Dungeon);
    let attraction = ctx
        .type_line
        .subtypes
        .iter()
        .any(|s| s.as_str() == "Attraction");
    let rooms = if dungeon {
        printed_rooms(&crate::oracle::raw_text())
    } else {
        vec![]
    };
    let index_of = |name: &str| rooms.iter().position(|(n, _)| n == name.trim());
    blocks
        .into_iter()
        .map(|b| {
            let room = b
                .split_once(" — ")
                .filter(|_| dungeon && !b.contains('\n'))
                .and_then(|(name, _)| {
                    let i = index_of(name)?;
                    let leads = rooms[i]
                        .1
                        .iter()
                        .map(|l| index_of(l))
                        .collect::<Option<Vec<usize>>>()?;
                    Some(crate::dungeons::room_trigger_name(i, &leads))
                });
            if let Some(room) = room {
                format!("{{{room}}} {b}")
            } else if attraction && b.starts_with("Visit — ") {
                format!("{VISIT_MARK}{b}")
            } else {
                b
            }
        })
        .collect()
}

inventory::submit! { BlockGroupPattern { name: "r704 dungeon rooms and visit abilities", priority: 0, group: mark_rooms_and_visits } }

/// "[Room name] — [effect]" on a dungeon card: a room ability, "When you move your venture
/// marker into this room, [effect]" (CR 309.4c).
fn dungeon_room(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (room, text) = crate::dungeons::split_room_mark(block)?;
    let (_, eff) = text.split_once(" — ")?;
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let tr = TriggeredAbility::new(TriggerCond::Custom(room.into()), body);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), text)])
}

inventory::submit! { AbilityPattern { name: "r704 dungeon room", priority: 70, parse: dungeon_room } }

/// "Visit — [effect]": "Whenever you roll to visit your Attractions, if the result is
/// equal to a number that is lit up on this Attraction, [effect]" (CR 702.159a).
fn visit(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let block = block.strip_prefix(VISIT_MARK)?;
    let eff = block.strip_prefix("Visit — ")?;
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let tr = TriggeredAbility::new(TriggerCond::Custom(SmolStr::new(VISIT)), body);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r704 attraction visit", priority: 70, parse: visit } }
