//! Rulings batch P114 — Backgrounds (CR 702.124k): a commander with "Choose a Background"
//! and a Background commander; "commander creatures you own have ..." abilities granted
//! by Backgrounds (Battle for Baldur's Gate).

use crate::r_p114_common::*;
use crate::r_s01_common::triggers_on_stack;
use crate::r_s02_common::{can_cast, create_token, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to, give_control};
use crate::r_s25_common::abilities_from;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::DeckProblem;
use mtg_engine::events::MoveCause;
use mtg_engine::kw::partner::check_commander_deck;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;
use std::sync::Arc;

const GUT: &str = "Gut, True Soul Zealot";
const GANAX: &str = "Ganax, Astral Hunter";
const GIANTS: &str = "Raised by Giants";

fn cards(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

/// A 100-card deck: the commanders, the other cards, and basic lands `filler` to fill it.
fn deck(commanders: &[&str], others: &[&str], filler: &str) -> Vec<Arc<CardDef>> {
    let mut d = cards(commanders);
    d.extend(cards(others));
    while d.len() < 100 {
        d.push(card(filler));
    }
    d
}

/// Puts the real cards onto the battlefield under P0's control at the same time, the
/// first one as P0's commander.
fn enter_together_first_commander(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    let objs: Vec<ObjectId> = names
        .iter()
        .map(|n| t.g.create_card_object(card(n), P0, Zone::Nowhere))
        .collect();
    t.g.objects[objs[0].0 as usize].is_commander = true;
    t.g.players[0].commander_names.push(names[0].into());
    let moves = objs
        .iter()
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids: Vec<ObjectId> = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

// --- Two commanders ---------------------------------------------------------------------------

#[test]
fn a_background_counts_as_a_commander_creature_only_while_its_a_creature() {
    cr!("702.124k", "903.3", "613.1d");
    ruling!(
        "Raised by Giants",
        "If a card refers to a commander creature you own, a Background won't usually be counted or included for that effect. If another spell or ability causes your Background to become a creature, however, it will be included. Any effect that refers to your commander or a commander you own or control without specifying creature will apply to a Background that is your commander, as appropriate."
    );
    let mut t = commander_game_n(2);
    let gut = commander_on_battlefield(&mut t, P0, GUT);
    let giants = commander_on_battlefield(&mut t, P0, GIANTS);
    assert_eq!(t.pt(gut), (10, 10));
    assert!(!t.obj(giants).is_creature());
    assert!(
        !t.obj(giants).chars.has_subtype("Giant"),
        "the noncreature Background isn't a commander creature"
    );
    // Another effect makes the Background a creature: Raised by Giants now applies to it.
    add_types(&mut t, giants, vec![CardType::Creature]);
    assert_eq!(t.pt(giants), (10, 10));
    assert!(t.obj(giants).chars.has_subtype("Giant"));

    // Thunderfoot Baloth (5/5): "Lieutenant — As long as you control your commander,
    // Thunderfoot Baloth gets +2/+2 ..." — a Background commander is your commander.
    supported("Thunderfoot Baloth");
    let mut t = commander_game_n(2);
    let baloth = t.battlefield(P0, "Thunderfoot Baloth");
    commander_on_battlefield(&mut t, P0, GIANTS);
    t.g.recompute();
    assert_eq!(t.pt(baloth), (7, 7));
}

#[test]
fn a_commander_that_loses_choose_a_background_is_still_a_commander() {
    cr!("903.3", "903.10a", "613.1f");
    ruling!(
        "Gut, True Soul Zealot",
        "If your commander loses the choose a Background ability or stops being a Background during the game, as appropriate, it is still your commander."
    );
    let mut t = commander_game_n(2);
    let gut = commander_on_battlefield(&mut t, P0, GUT);
    commander_on_battlefield(&mut t, P0, GIANTS);
    // Humility: Gut loses all abilities (Choose a Background among them) and is a 1/1.
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(t.obj(gut).chars.abilities.is_empty());
    assert!(t.obj(gut).is_commander);
    attack_with(&mut t, &[(gut, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.player(P1).commander_damage.get(GUT), Some(&1));
}

#[test]
fn two_commanders_of_the_same_color() {
    cr!("702.124k", "702.124c", "903.4");
    ruling!(
        "Gut, True Soul Zealot",
        "You can choose two commanders that are the same color or colors."
    );
    // Gut and Street Urchin are both red.
    let pair = cards(&[GUT, "Street Urchin"]);
    let d = deck(&[GUT, "Street Urchin"], &["Lightning Bolt"], "Mountain");
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
}

#[test]
fn two_commanders_combined_color_identity() {
    cr!("702.124c", "903.5c");
    ruling!(
        "Gut, True Soul Zealot",
        "If your Commander deck has two commanders, you can include only cards whose own color identities are also found in your commanders’ combined color identities."
    );
    ruling!(
        "Francisco, Fowl Marauder",
        "If your Commander deck has two commanders, you can only include cards whose own color identities are also found in your commanders' combined color identities."
    );
    // Gut (red) and Raised by Giants (green).
    let pair = cards(&[GUT, GIANTS]);
    let ok = deck(
        &[GUT, GIANTS],
        &["Lightning Bolt", "Giant Growth"],
        "Forest",
    );
    assert!(check_commander_deck(&ok, &pair, &[], false).is_empty());
    let bad = deck(&[GUT, GIANTS], &["Counterspell"], "Mountain");
    assert!(check_commander_deck(&bad, &pair, &[], false).contains(
        &DeckProblem::OutsideColorIdentity {
            name: "Counterspell".into()
        }
    ));
    // Francisco (black) and Kraum (blue-red).
    let pair = cards(&["Francisco, Fowl Marauder", "Kraum, Ludevic's Opus"]);
    let names = ["Francisco, Fowl Marauder", "Kraum, Ludevic's Opus"];
    let ok = deck(&names, &["Counterspell", "Lightning Bolt"], "Swamp");
    assert!(check_commander_deck(&ok, &pair, &[], false).is_empty());
    let bad = deck(&names, &["Giant Growth"], "Island");
    assert!(check_commander_deck(&bad, &pair, &[], false).contains(
        &DeckProblem::OutsideColorIdentity {
            name: "Giant Growth".into()
        }
    ));
}

#[test]
fn a_commander_and_its_background_are_tracked_separately() {
    cr!("702.124d", "903.8", "903.10a", "704.6c");
    ruling!(
        "Gut, True Soul Zealot",
        "Once the game begins, your two commanders are tracked separately. If you cast one, you won’t have to pay an additional {2} the first time you cast the other. A player loses the game after having been dealt 21 combat damage from any one of them, not from both of them combined (although your Background won’t usually be a creature anyway)."
    );
    // Commander tax: Gut ({2}{R}) is cast, dies and returns to the command zone; Raised by
    // Giants ({5}{G}) costs no more for it; Gut costs {2} more.
    let mut t = commander_game_n(2);
    let gut = {
        let id = t.command(P0, GUT);
        make_commander(&mut t, id);
        id
    };
    let giants = {
        let id = t.command(P0, GIANTS);
        make_commander(&mut t, id);
        id
    };
    t.lands(P0, "Mountain", 3);
    t.cast(P0, gut).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    destroy(&mut t, gut);
    assert_eq!(t.zone(gut), Zone::Command);
    t.lands(P0, "Forest", 6);
    t.cast(P0, giants).go();
    t.resolve_all();
    assert!(t.on_battlefield(giants));
    let gut = t.g.current(gut);
    t.lands(P0, "Mountain", 3);
    assert!(!can_cast(&mut t, P0, gut, CastMethod::Normal));
    t.lands(P0, "Wastes", 2);
    assert!(can_cast(&mut t, P0, gut, CastMethod::Normal));
    t.cast(P0, gut).go();
    t.resolve_all();

    // Combat damage: 15 from the Background before, 10 from Gut now (a 10/10): 25 from
    // P0's commanders combined, but no more than 21 from either.
    let gut = t.g.current(gut);
    t.g.players[1].life = 40;
    t.g.players[1].commander_damage.insert(GIANTS.into(), 15);
    t.g.objects[gut.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(gut, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    assert_eq!(t.player(P1).commander_damage.get(GUT), Some(&10));
    assert!(!t.has_lost(P1));
    // Another 11 from Gut alone is 21.
    t.g.players[1].commander_damage.insert(GUT.into(), 21);
    t.g.dirty = true;
    t.settle();
    assert!(t.has_lost(P1));
}

// --- Granted abilities ----------------------------------------------------------------------

#[test]
fn street_urchin_commander_can_sacrifice_itself_if_its_an_artifact() {
    cr!("602.2", "701.21a", "113.7a");
    ruling!(
        "Street Urchin",
        "If your commander is an artifact creature, you may sacrifice it to pay the cost of this ability."
    );
    // Silas Renn is a legendary artifact creature.
    let mut t = commander_game_n(2);
    let silas = commander_on_battlefield(&mut t, P0, "Silas Renn, Seeker Adept");
    commander_on_battlefield(&mut t, P0, "Street Urchin");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[ent(silas)]);
    t.answer_yes(P0, false);
    t.activate(P0, silas, 0, &[Entity::Player(P1)])
        .expect("sacrificing itself");
    assert!(!t.on_battlefield(silas));
    t.resolve_all();
    assert_eq!(t.life(P1), 39);
}

#[test]
fn street_urchin_commander_deals_damage_even_if_no_longer_a_creature() {
    cr!("608.2h", "613.1d");
    ruling!(
        "Street Urchin",
        "If your commander that had the ability Street Urchin gave it somehow isn't a creature at the time the ability resolves, it will still deal 1 damage."
    );
    let mut t = commander_game_n(2);
    let gut = commander_on_battlefield(&mut t, P0, GUT);
    commander_on_battlefield(&mut t, P0, "Street Urchin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[ent(bears)]);
    t.activate(P0, gut, 0, &[Entity::Player(P1)]).expect("Gut");
    // In response, Gut stops being a creature.
    run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &[ent(gut)],
    );
    assert!(!t.obj(gut).is_creature());
    t.resolve_all();
    assert_eq!(t.life(P1), 39);
}

#[test]
fn master_chef_entering_with_others_only_the_commander_gets_its_counter() {
    cr!("614.12", "614.1c");
    ruling!(
        "Master Chef",
        "If your commander is entering the battlefield at the same time as other creatures, it will enter the battlefield with an additional +1/+1 counter, but the other creatures will not. This is because a replacement effect that is created by an object entering the battlefield can only apply to itself."
    );
    supported("Master Chef");
    let mut t = commander_game_n(2);
    t.battlefield(P0, "Master Chef");
    let ids = enter_together_first_commander(&mut t, &[GANAX, "Grizzly Bears"]);
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1, "the commander");
    assert_eq!(t.counters(ids[1], counters::PLUS1), 0, "entering with it");
    // A creature entering later gets one.
    let later = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(later, counters::PLUS1), 1);
}

#[test]
fn criminal_past_counts_the_controllers_graveyard() {
    cr!("109.5", "613.4c");
    ruling!(
        "Criminal Past",
        "If an opponent controls a commander creature you own while you control Criminal Past, X will be the number of creatures cards in that player's graveyard, not yours."
    );
    let mut t = commander_game_n(2);
    let gut = commander_on_battlefield(&mut t, P0, GUT);
    commander_on_battlefield(&mut t, P0, "Criminal Past");
    t.graveyard(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.graveyard(P1, "Hill Giant");
    }
    t.g.recompute();
    assert_eq!(t.pt(gut), (3, 2), "P0's graveyard");
    give_control(&mut t, gut, P1);
    assert_eq!(t.pt(gut), (5, 2), "P1's graveyard");
}

#[test]
fn cloakwood_hermit_looks_at_the_whole_turn() {
    cr!("603.4");
    ruling!(
        "Cloakwood Hermit",
        "The ability Cloakwood Hermit grants triggers if you control a creature with that ability as the end step begins and a creature card was put into your graveyard from anywhere this turn. This is true even if no creatures had this ability at the time the creature card went to the graveyard or the creature card is no longer in your graveyard as your end step begins."
    );
    supported("Cloakwood Hermit");
    // A creature card goes to the graveyard and is exiled from there before the commander
    // with the ability is on the battlefield.
    let mut t = commander_game_n(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    move_to(&mut t, card, Zone::Exile);
    commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, "Cloakwood Hermit");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Squirrel").len(), 2);
    // Without any creature card put into the graveyard: none.
    let mut t = commander_game_n(2);
    commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, "Cloakwood Hermit");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens_named(&t, P0, "Squirrel").is_empty());
}

#[test]
fn cloakwood_hermit_counts_a_commander_that_died_and_was_recast() {
    cr!("903.9a", "603.4");
    ruling!(
        "Cloakwood Hermit",
        "Notably, this means that if your commander creature dies and you choose to return it to the command zone from the graveyard, then cast it again the same turn, the ability granted to it by Cloakwood Hermit will trigger at the beginning of that turn's end step."
    );
    let mut t = commander_game_n(2);
    let ganax = commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, "Cloakwood Hermit");
    t.answer_yes(P0, true);
    destroy(&mut t, ganax);
    let ganax = t.g.current(ganax);
    assert_eq!(t.zone(ganax), Zone::Command);
    t.lands(P0, "Mountain", 5);
    t.cast(P0, ganax).go();
    t.resolve_all();
    assert!(t.on_battlefield(ganax));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Squirrel").len(), 2);
}

/// P0 ventures into Lost Mine of Phandelver, with the cards `extra` on P0's battlefield
/// and Ganax as P0's commander there; returns how many Cave Entrance ("Scry 1") abilities
/// triggered, and the room P0's venture marker is in after they resolve.
fn delver_venture(extra: &[&str]) -> (usize, Option<usize>) {
    let mut t = commander_game_n(2);
    commander_on_battlefield(&mut t, P0, GANAX);
    for n in extra {
        t.battlefield(P0, n);
    }
    t.custom(
        P0,
        (*card("Lost Mine of Phandelver")).clone(),
        Zone::Outside(P0),
    );
    t.g.recompute();
    run_from(
        &mut t,
        P0,
        None,
        Effect::KeywordAction {
            action: KeywordAction::Venture,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &[],
    );
    let n = triggers_on_stack(&t, "Scry");
    t.resolve_all();
    (n, mtg_engine::dungeons::marker(&t.g, P0).map(|(_, r)| r))
}

#[test]
fn dungeon_delver_room_triggers_additional_times_in_the_same_room() {
    cr!("309.4c", "603.2d");
    ruling!(
        "Dungeon Delver",
        "If you control more than one permanent with this ability, or a permanent with multiple instances of this ability, the room triggers that many additional times. For example, if you control Hama Pashar, Ruin Seeker; Dungeon Delver; and a commander creature you own, room abilities of dungeons you own trigger two additional times."
    );
    ruling!(
        "Dungeon Delver",
        "The ability Dungeon Delver grants causes the same room you ventured into to trigger an additional time. It does not cause you to move to the next room."
    );
    supported("Dungeon Delver");
    assert_eq!(delver_venture(&[]), (1, Some(0)));
    assert_eq!(delver_venture(&["Dungeon Delver"]), (2, Some(0)));
    assert_eq!(
        delver_venture(&["Dungeon Delver", "Hama Pashar, Ruin Seeker"]),
        (3, Some(0))
    );
    // Two Dungeon Delvers: the commander has two instances of the ability.
    assert_eq!(
        delver_venture(&["Dungeon Delver", "Dungeon Delver"]),
        (3, Some(0))
    );
}

/// In a three-player game where P0 has 30 life, P1 20 and P2 `p2_life`, Ganax (P0's
/// commander, with the ability Background `bg` grants) attacks P1. Returns how many of the
/// granted "if no opponent has more life than that player" abilities triggered, and the
/// game.
fn attack_with_life_check(bg: &str, p2_life: i32) -> (usize, TestGame) {
    supported(bg);
    let mut t = commander_game_n(3);
    let ganax = commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, bg);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 30;
    t.g.players[1].life = 20;
    t.g.players[2].life = p2_life;
    t.answer_targets(P0, &[ent(bears)]);
    attack_with(&mut t, &[(ganax, Entity::Player(P1))]);
    (triggers_on_stack(&t, "no opponent has more life"), t)
}

#[test]
fn granted_attack_abilities_compare_only_the_attacking_players_opponents() {
    cr!("603.4", "508.1");
    ruling!(
        "Guild Artisan",
        "The ability granted by Guild Artisan is looking to see if any of the attacking player's opponents have a higher life total than that defending player. The attacking player's life total is not included in this comparison."
    );
    ruling!(
        "Hardy Outlander",
        "The ability granted by Hardy Outlander is looking to see if any of the attacking player's opponents have a higher life total than that defending player. The attacking player's life total is not included in this comparison."
    );
    ruling!(
        "Sword Coast Sailor",
        "The ability granted by Sword Coast Sailor is looking to see if any of the attacking player's opponents have a higher life total than that defending player. The attacking player's life total is not included in this comparison."
    );
    ruling!(
        "Veteran Soldier",
        "The ability granted by Veteran Soldier is looking to see if any of the attacking player's opponents have a higher life total than that defending player. The attacking player's life total is not included in this comparison."
    );
    for bg in [
        "Guild Artisan",
        "Hardy Outlander",
        "Sword Coast Sailor",
        "Veteran Soldier",
    ] {
        // P0 has more life than P1, and P2 as much: it triggers.
        let (n, mut t) = attack_with_life_check(bg, 20);
        assert_eq!(n, 1, "{bg}");
        t.resolve_all();
        if bg == "Guild Artisan" {
            assert_eq!(tokens_named(&t, P0, "Treasure").len(), 2);
        }
        // P2 has more life than P1: it doesn't.
        let (n, _) = attack_with_life_check(bg, 21);
        assert_eq!(n, 0, "{bg}");
    }
}

#[test]
fn veteran_soldier_creates_a_token_attacking_each_opponent() {
    cr!("508.4", "506.2");
    ruling!(
        "Veteran Soldier",
        "Veteran Soldier has received an update to its Oracle text to clarify its function. Specifically, a Soldier creature token is created for each opponent of the controller of the attacking commander."
    );
    let mut t = commander_game_n(4);
    let ganax = commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, "Veteran Soldier");
    attack_with(&mut t, &[(ganax, Entity::Player(P2))]);
    t.resolve_all();
    let soldiers = tokens_named(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 3, "one for each opponent");
    let combat = t.g.combat.as_ref().expect("combat");
    let mut attacked: Vec<Entity> = soldiers
        .iter()
        .map(|s| {
            assert!(t.obj(*s).tapped);
            combat
                .attackers
                .iter()
                .find(|a| a.id == *s)
                .and_then(|a| a.target)
                .expect("attacking")
        })
        .collect();
    attacked.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(
        attacked,
        vec![Entity::Player(P1), Entity::Player(P2), Entity::Player(P3)]
    );
}

/// Ganax (P0's commander with Hardy Outlander's ability) attacks P1; the granted trigger
/// targets P0's Grizzly Bears. `respond` runs while the trigger is on the stack. Returns
/// the game, Ganax and the Bears after the trigger resolved.
fn hardy_attack(respond: impl FnOnce(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    supported("Hardy Outlander");
    let mut t = commander_game_n(2);
    let ganax = commander_on_battlefield(&mut t, P0, GANAX);
    t.battlefield(P0, "Hardy Outlander");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[ent(bears)]);
    attack_with(&mut t, &[(ganax, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, ganax).len(), 1);
    respond(&mut t, ganax);
    t.resolve_all();
    (t, ganax, bears)
}

fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::Const(p), Value::Const(0))],
    );
}

#[test]
fn hardy_outlander_x_is_the_commanders_power_as_the_ability_resolves() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Hardy Outlander",
        "If the commander creature leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    ruling!(
        "Hardy Outlander",
        "If the commander creature's power is negative as its triggered ability resolves, X is considered to be 0."
    );
    ruling!(
        "Hardy Outlander",
        "The value of X is determined only as the granted triggered ability resolves. Once that happens, the value of X won't change later in the turn even if the commander creature's power changes."
    );
    // Ganax (3/4) as it resolves: +3/+3.
    let (t, _, bears) = hardy_attack(|_, _| {});
    assert_eq!(t.pt(bears), (5, 5));
    // Pumped to 5 power before it resolves, then more after: +5/+5 only.
    let (mut t, ganax, bears) = hardy_attack(|t, g| pump(t, g, 2));
    assert_eq!(t.pt(bears), (7, 7));
    pump(&mut t, ganax, 4);
    assert_eq!(t.pt(bears), (7, 7));
    // Pumped to 4 power, then gone: its last known power.
    let (t, _, bears) = hardy_attack(|t, g| {
        pump(t, g, 1);
        t.answer_yes(P0, false);
        destroy(t, g);
    });
    assert_eq!(t.pt(bears), (6, 6));
    // Power -2: X is 0.
    let (t, _, bears) = hardy_attack(|t, g| pump(t, g, -5));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn dragon_cultist_counts_all_damage_a_single_source_dealt() {
    cr!("603.4", "120.1");
    ruling!(
        "Dragon Cultist",
        "The triggered ability granted by Dragon Cultist looks for the total amount of damage dealt by a single source, even if that damage wasn't dealt all at once or wasn't dealt to the same player or permanent."
    );
    supported("Dragon Cultist");
    for same_source in [true, false] {
        let mut t = commander_game_n(2);
        commander_on_battlefield(&mut t, P0, GUT);
        t.battlefield(P0, "Dragon Cultist");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let target = t.battlefield(P1, "Craw Wurm");
        deal(&mut t, giant, 3, Entity::Player(P1));
        let second = if same_source { giant } else { bears };
        deal(&mut t, second, 2, ent(target));
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let dragons = tokens_named(&t, P0, "Dragon").len();
        assert_eq!(
            dragons,
            usize::from(same_source),
            "same source {same_source}"
        );
    }
}

#[test]
fn far_traveler_and_displacer_kitten_return_a_new_object() {
    cr!("400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Far Traveler",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled. Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    ruling!(
        "Displacer Kitten",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled. Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    supported("Far Traveler");
    supported("Displacer Kitten");
    for kitten in [false, true] {
        let mut t = commander_game_n(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let pacifism = attach_new(&mut t, P0, "Pacifism", ent(bears));
        let splitter = attach_new(&mut t, P0, "Bonesplitter", ent(bears));
        t.g.add_counters(ent(bears), counters::PLUS1, 2, None);
        t.answer_targets(P0, &[ent(bears)]);
        if kitten {
            t.battlefield(P0, "Displacer Kitten");
            let spell = t.hand(P0, "Divination");
            t.lands(P0, "Island", 3);
            t.cast_with(P0, spell, &[]).unwrap();
            t.resolve_all();
        } else {
            commander_on_battlefield(&mut t, P0, GUT);
            t.battlefield(P0, "Far Traveler");
            t.g.objects[bears.0 as usize].tapped = true;
            t.advance_to(P0, Step::End);
            t.resolve_all();
        }
        let now = t.g.current(bears);
        assert_ne!(now, bears, "a new object");
        assert!(t.on_battlefield(now));
        assert!(t.in_graveyard(P0, "Pacifism"), "kitten {kitten}");
        assert!(!t.on_battlefield(t.g.current(pacifism)));
        assert!(t.on_battlefield(splitter));
        assert_eq!(attached_to(&t, splitter), None);
        assert_eq!(t.counters(now, counters::PLUS1), 0);
        assert_eq!(t.pt(now), (2, 2));
    }
}

#[test]
fn jaheira_gives_the_mana_ability_to_noncreature_tokens() {
    cr!("111.1", "111.10f", "605.1a");
    ruling!(
        "Jaheira, Friend of the Forest",
        "Jaheira's ability gives the mana ability to all tokens you control, not just creature tokens."
    );
    supported("Jaheira, Friend of the Forest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jaheira, Friend of the Forest");
    let clue = create_token(&mut t, P0, "Clue");
    t.g.recompute();
    let i = t
        .obj(clue)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Add {G}"))
        .expect("the Clue has \"{T}: Add {G}.\"");
    t.activate(P0, clue, i, &[]).expect("tap for G");
    assert_eq!(
        t.player(P0).mana_pool.count(mtg_engine::mana::ManaType::G),
        1
    );
}

#[test]
fn gut_can_sacrifice_itself_if_its_an_artifact() {
    cr!("701.21a", "603.5");
    ruling!(
        "Gut, True Soul Zealot",
        "If Gut somehow becomes an artifact, you may sacrifice it to its own ability."
    );
    for artifact in [false, true] {
        let mut t = TestGame::new(2);
        let gut = t.battlefield(P0, GUT);
        if artifact {
            add_types(&mut t, gut, vec![CardType::Artifact]);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[ent(gut)]);
        attack_with(&mut t, &[(gut, Entity::Player(P1))]);
        t.resolve_all();
        let skeletons = tokens_named(&t, P0, "Skeleton");
        if artifact {
            assert!(t.in_graveyard(P0, GUT));
            assert_eq!(skeletons.len(), 1);
            assert!(t.obj(skeletons[0]).tapped);
        } else {
            // Not an artifact, Gut isn't "another creature or an artifact".
            assert!(t.on_battlefield(gut));
            assert!(skeletons.is_empty());
        }
    }
}
