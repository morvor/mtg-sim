//! Rulings that predate battles (March of the Machine, 2023-04-21) and leave them out:
//! the user's policy of Nov 19 (register `pre-battle-rulings-give-way-to-the-cr`,
//! docs/interpretations/sources.md). The CR decides: battle is a permanent type (CR 110.4,
//! 110.4a, 110.4b), a battle's protector is the defending player of a creature attacking it
//! (CR 310.9d, 508.5), a creature may attack a battle (CR 508.1b, 508.1d, 508.4, 702.22d,
//! 702.49c) and a battle's protector may block creatures attacking it (CR 310.9c, 509.1a).
//! Each test cites the ruling for what it says of the other permanent types, players and
//! planeswalkers, and shows the battle case the ruling leaves out (its battle part is
//! exempt in docs/rulings-exemptions/pre-battle-rulings.tsv and logged as outdated in
//! docs/RULING_DISCREPANCIES.md).

use crate::r_p016_common::lots;
use crate::r_p132_common::where_is;
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::attach_new;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s12_common::attack_target;
use crate::r_s21_common::{castable, legal_blocks};
use crate::r_s35_common::pick_options;
use mtg_engine::battle;
use mtg_engine::combat::player_has_attacked;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const POLICY: &str = "pre-battle-rulings-give-way-to-the-cr";

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `controller`'s Invasion of Azgol (a Siege) on the battlefield, protected by `protector`.
fn siege(t: &mut TestGame, controller: PlayerId, protector: PlayerId) -> ObjectId {
    let id = t.battlefield(controller, "Invasion of Azgol");
    t.g.objects[id.0 as usize].choices.player = Some(protector);
    t.g.dirty = true;
    assert_eq!(battle::protector(&t.g, id), Some(protector));
    id
}

// ---------------------------------------------------------------------------------
// Permanent types (CR 110.4: "There are six permanent types: artifact, battle, creature,
// enchantment, land, and planeswalker.")
// ---------------------------------------------------------------------------------

#[test]
fn creeping_renaissance_can_return_battle_cards() {
    cr!("110.4", "205.2a");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Creeping Renaissance",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker."
    );
    supported("Creeping Renaissance");
    // "Choose a permanent type. Return all cards of the chosen type from your graveyard to
    // your hand." The choices are the six permanent types, battle among them, and choosing
    // it returns the battle card.
    let mut t = TestGame::new(2);
    let invasion = t.graveyard(P0, "Invasion of Azgol");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    pick_options(&mut t, P0, &["battle"]);
    crate::r_p079_common::cast_card(&mut t, P0, "Creeping Renaissance", &[]).unwrap();
    t.resolve_all();
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    let mut opts = options[0].clone();
    opts.sort();
    assert_eq!(
        opts,
        [
            "artifact",
            "battle",
            "creature",
            "enchantment",
            "land",
            "planeswalker"
        ],
        "{POLICY}"
    );
    assert_eq!(where_is(&t, invasion), "hand");
    assert_eq!(where_is(&t, bears), "graveyard");
}

#[test]
fn role_reversal_exchanges_two_battles() {
    cr!("110.4", "701.12a", "310.11");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Role Reversal",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker. Supertypes, like legendary, aren’t permanent types."
    );
    supported("Role Reversal");
    // "Exchange control of two target permanents that share a permanent type." Two
    // battles share one; a battle and a creature share none.
    // (Mana in the pool, not lands: two lands would share a permanent type.)
    let mut t = TestGame::new(2);
    let mine = siege(&mut t, P0, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.set_step(P0, Step::PrecombatMain);
    let c = t.hand(P0, "Role Reversal");
    assert!(t
        .cast(P0, c)
        .targets(&[obj(mine), obj(bears)])
        .try_go()
        .is_err());
    let mut t = TestGame::new(2);
    let mine = siege(&mut t, P0, P1);
    let theirs = siege(&mut t, P1, P0);
    t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.set_step(P0, Step::PrecombatMain);
    let c = t.hand(P0, "Role Reversal");
    t.cast(P0, c).targets(&[obj(mine), obj(theirs)]).go();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.obj_now(theirs).controller, P0);
    // Each Siege's protector is now its controller, who can't protect it: its controller
    // chooses an opponent instead (CR 310.11, 310.12a).
    assert_eq!(battle::protector(&t.g, mine), Some(P0));
    assert_eq!(battle::protector(&t.g, theirs), Some(P1));
}

#[test]
fn liliana_dreadhorde_general_keeps_one_battle() {
    cr!("110.4");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Liliana, Dreadhorde General",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker. Supertypes, like legendary, aren't permanent types."
    );
    supported("Liliana, Dreadhorde General");
    // −9: "Each opponent chooses a permanent they control of each permanent type and
    // sacrifices the rest." P1 keeps the second of its two battles (and its creature, its
    // only one), and sacrifices the first.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lili = t.battlefield(P0, "Liliana, Dreadhorde General");
    t.g.objects[lili.0 as usize]
        .counters
        .insert("loyalty".into(), 9);
    let other = siege(&mut t, P1, P0);
    let kept = siege(&mut t, P1, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(kept)]);
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(kept) && t.on_battlefield(bears));
    assert!(!t.on_battlefield(other));
    assert_eq!(where_is(&t, other), "graveyard");
}

#[test]
fn eureka_puts_a_battle_card_onto_the_battlefield() {
    cr!("110.4a", "310.9a");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Eureka",
        "A “permanent card” is a card that would be a permanent once it’s on the battlefield. Specifically, it’s an artifact, creature, enchantment, land, or planeswalker card."
    );
    supported("Eureka");
    // "Starting with you, each player may put a permanent card from their hand onto the
    // battlefield. Repeat this process until no one puts a card onto the battlefield." P0
    // puts a battle card, P1 a creature card; P0's instant can't be put.
    let mut t = TestGame::new(2);
    let invasion = t.hand(P0, "Invasion of Azgol");
    let shock = t.hand(P0, "Shock");
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(invasion)]);
    t.answer_choose(P0, &[Entity::Player(P1)]); // the Siege's protector
    t.answer_choose(P1, &[obj(bears)]);
    let from = t.asked().len();
    crate::r_p079_common::cast_card(&mut t, P0, "Eureka", &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(invasion), "{}", t.dump_log());
    assert_eq!(battle::protector(&t.g, t.g.current(invasion)), Some(P1));
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P0, "Shock"));
    // The Shock was never offered.
    let shock_offered = t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => candidates.contains(&obj(shock)),
        _ => false,
    });
    assert!(!shock_offered);
}

#[test]
fn in_search_of_greatness_casts_a_battle_spell() {
    cr!("110.4b", "608.2g");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "In Search of Greatness",
        "A permanent spell is an artifact spell, a creature spell, an enchantment spell, or a planeswalker spell."
    );
    supported("In Search of Greatness");
    // "At the beginning of your upkeep, you may cast a permanent spell from your hand with
    // mana value equal to 1 plus the greatest mana value among other permanents you control
    // without paying its mana cost." P0 controls a Grizzly Bears (mana value 2): the battle
    // spell Invasion of Segovia and the creature spell Gray Ogre (mana value 3) are offered,
    // and the battle is cast; the sorcery Divination (mana value 3) isn't offered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "In Search of Greatness");
    t.battlefield(P0, "Grizzly Bears");
    let invasion = t.hand(P0, "Invasion of Segovia");
    let ogre = t.hand(P0, "Gray Ogre");
    let sorcery = t.hand(P0, "Divination");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(invasion)]);
    t.answer_choose(P0, &[Entity::Player(P1)]); // the Siege's protector
    let from = t.asked().len();
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(invasion), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Gray Ogre"));
    let offered = |id: ObjectId| {
        t.asked()[from..].iter().any(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => candidates.contains(&obj(id)),
            _ => false,
        })
    };
    assert!(offered(invasion) && offered(ogre));
    assert!(!offered(sorcery));
}

// ---------------------------------------------------------------------------------
// The defending player (CR 508.5, 310.9d): attacking a battle, its protector.
// ---------------------------------------------------------------------------------

/// Three players: P2 controls a Siege that P1 protects.
fn three_with_siege() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(3);
    let b = siege(&mut t, P2, P1);
    (t, b)
}

#[test]
fn torrent_elemental_attacking_a_battle_taps_its_protectors_creatures() {
    cr!("508.5", "310.9d");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Torrent Elemental",
        "\"Defending player\" refers to the player Torrent Elemental is attacking or the controller of the planeswalker Torrent Elemental is attacking."
    );
    supported("Torrent Elemental");
    // "Whenever this creature attacks, tap all creatures defending player controls."
    // Attacking P2's planeswalker: P2's creatures. Attacking the Siege P2 controls and P1
    // protects: P1's.
    let (mut t, _) = three_with_siege();
    let jace = t.battlefield(P2, "Jace Beleren");
    let torrent = t.battlefield(P0, "Torrent Elemental");
    let p1s = t.battlefield(P1, "Grizzly Bears");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(torrent, obj(jace))]);
    t.resolve_all();
    assert!(!t.obj_now(p1s).tapped && t.obj_now(p2s).tapped);
    let (mut t, b) = three_with_siege();
    let torrent = t.battlefield(P0, "Torrent Elemental");
    let p1s = t.battlefield(P1, "Grizzly Bears");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(torrent, obj(b))]);
    t.resolve_all();
    assert!(t.obj_now(p1s).tapped, "the protector's creature ({POLICY})");
    assert!(!t.obj_now(p2s).tapped, "not the battle's controller's");
}

#[test]
fn wardscale_dragon_attacking_a_battle_stops_its_protector_casting_spells() {
    cr!("508.5", "310.9d");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Wardscale Dragon",
        "“Defending player” refers to the player Wardscale Dragon is attacking or the controller of the planeswalker Wardscale Dragon is attacking."
    );
    supported("Wardscale Dragon");
    // "As long as this creature is attacking, defending player can't cast spells."
    // Attacking P2's planeswalker, P2 can't cast; attacking the Siege P2 controls and P1
    // protects, P1 can't and P2 can.
    for battle_attacked in [false, true] {
        let (mut t, b) = three_with_siege();
        let jace = t.battlefield(P2, "Jace Beleren");
        let dragon = t.battlefield(P0, "Wardscale Dragon");
        let s1 = t.hand(P1, "Shock");
        let s2 = t.hand(P2, "Shock");
        t.lands(P1, "Mountain", 1);
        t.lands(P2, "Mountain", 1);
        to_combat(&mut t, P0);
        let target = if battle_attacked { obj(b) } else { obj(jace) };
        attack_with(&mut t, &[(dragon, target)]);
        assert_eq!(castable(&mut t, P1, s1), !battle_attacked, "P1 ({POLICY})");
        assert_eq!(castable(&mut t, P2, s2), battle_attacked, "P2");
    }
}

// ---------------------------------------------------------------------------------
// What a creature attacks (CR 508.1b, 508.1d, 508.4, 702.22d, 702.49c) and blocks
// (CR 509.1a, 310.9c). Two players: P0 controls a Siege that P1 protects (a Siege can be
// attacked by its own controller, CR 310.9b).
// ---------------------------------------------------------------------------------

fn two_with_siege() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let b = siege(&mut t, P0, P1);
    (t, b)
}

#[test]
fn attacks_if_able_attacks_a_battle_when_attacking_the_player_costs() {
    cr!("508.1d", "310.9b", "508.1b");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Archon of Absolution",
        "Your opponents can choose not to pay to attack with a creature that attacks “if able.” If there's no other player or planeswalker to attack, that creature simply doesn't attack."
    );
    ruling!(
        "Forbidding Spirit",
        "While Forbidding Spirit’s effect is in effect, your opponents can choose not to pay to attack with a creature that attacks “if able.” If there’s no other player or planeswalker to attack, that creature simply doesn’t attack."
    );
    ruling!(
        "Sivitri, Dragon Master",
        "A player who controls a creature that “attacks if able” can decline to pay additional costs to have that creature attack. If they do and there’s no other player or planeswalker to attack, that creature simply doesn’t attack."
    );
    ruling!(
        "Nils, Discipline Enforcer",
        "Your opponents can choose not to pay to attack with a creature with one or more counters on it that attacks \"if able.\" If there's no other player or planeswalker to attack, the creature simply doesn't attack."
    );
    // Each of P1's cards: "creatures can't attack you or planeswalkers you control unless
    // their controller pays ...". P0's Juggernaut ("attacks each combat if able") needn't
    // pay to attack P1, but nothing taxes attacking the battle P1 protects: it must attack
    // the battle. With no battle, it needn't attack.
    for name in [
        "Archon of Absolution",
        "Forbidding Spirit",
        "Sivitri, Dragon Master",
        "Nils, Discipline Enforcer",
    ] {
        supported(name);
        for with_battle in [false, true] {
            let mut t = TestGame::new(2);
            let b = with_battle.then(|| siege(&mut t, P0, P1));
            let jugg = t.battlefield(P0, "Juggernaut");
            match name {
                "Forbidding Spirit" => {
                    t.enter(P1, name);
                    t.resolve_all();
                }
                "Sivitri, Dragon Master" => {
                    t.set_step(P1, Step::PrecombatMain);
                    let s = t.battlefield(P1, name);
                    t.activate(P1, s, 0, &[]).unwrap();
                    t.resolve_all();
                }
                "Nils, Discipline Enforcer" => {
                    t.battlefield(P1, name);
                    t.g.add_counters(obj(jugg), "+1/+1", 1, None);
                }
                _ => {
                    t.battlefield(P1, name);
                }
            }
            t.lands(P0, "Wastes", 4);
            to_combat(&mut t, P0);
            assert!(
                legal_attack(&mut t, &[(jugg, Entity::Player(P1))]),
                "{name}"
            );
            assert_eq!(
                legal_attack(&mut t, &[]),
                !with_battle,
                "{name}, battle {with_battle} ({POLICY})"
            );
            if let Some(b) = b {
                assert!(legal_attack(&mut t, &[(jugg, obj(b))]), "{name}");
            }
        }
    }
}

#[test]
fn territorial_hellkite_may_attack_a_battle_when_free_not_to_attack_the_chosen_player() {
    cr!("508.1d", "508.1b", "508.1c");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Territorial Hellkite",
        "If Territorial Hellkite can't attack the chosen player, it's free to attack any other player or planeswalker or to not attack. If there's a cost associated with attacking the chosen player, it's free to attack any other player or planeswalker or to not attack. But if there's a cost associated with Territorial Hellkite attacking at all, Territorial Hellkite has to attack the chosen player or not attack."
    );
    supported("Territorial Hellkite");
    supported("Blazing Archon");
    supported("Brainwash");
    // "At the beginning of combat on your turn, choose an opponent at random that this
    // creature didn't attack during your last combat. This creature attacks that player this
    // combat if able." The chosen player is P1, who protects P0's Siege. Free to attack P1,
    // the Hellkite must attack P1. If it can't attack P1 (P1's Blazing Archon: "Creatures
    // can't attack you.") or attacking P1 costs (P1's Archon of Absolution), it may attack the
    // battle or not attack. If attacking at all costs (Brainwash: "Enchanted creature can't
    // attack unless its controller pays {3}."), it attacks P1 or doesn't attack.
    for case in ["free", "can't attack P1", "attacking P1 costs", "attacking costs"] {
        let (mut t, b) = two_with_siege();
        let hk = t.battlefield(P0, "Territorial Hellkite");
        match case {
            "can't attack P1" => {
                t.battlefield(P1, "Blazing Archon");
            }
            "attacking P1 costs" => {
                t.battlefield(P1, "Archon of Absolution");
            }
            "attacking costs" => {
                attach_new(&mut t, P1, "Brainwash", hk);
            }
            _ => {}
        }
        t.lands(P0, "Wastes", 3);
        t.set_step(P0, Step::PrecombatMain);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        // (no attack, the battle, P1)
        let legal = match case {
            "free" => (false, false, true),
            "can't attack P1" => (true, true, false),
            "attacking P1 costs" => (true, true, true),
            _ => (true, false, true),
        };
        assert_eq!(legal_attack(&mut t, &[]), legal.0, "{case}: no attack");
        assert_eq!(
            legal_attack(&mut t, &[(hk, obj(b))]),
            legal.1,
            "{case}: the battle ({POLICY})"
        );
        assert_eq!(
            legal_attack(&mut t, &[(hk, pl(P1))]),
            legal.2,
            "{case}: P1"
        );
    }
}

#[test]
fn boros_battleshaper_its_forced_attacker_may_attack_a_battle_and_its_forced_blocker_blocks_for_one(
) {
    cr!("508.1b", "508.1d", "509.1a", "509.1b", "509.1c", "310.9c");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Boros Battleshaper",
        "The controller of each creature chooses which player or planeswalker it attacks or which attacking creature it blocks, as appropriate."
    );
    ruling!(
        "Boros Battleshaper",
        "A creature forced to block does so only if it’s able to do so as blockers are declared. If, at that time, the creature is tapped, is affected by a spell or ability that says it can’t block, or no creatures are attacking its controller or a planeswalker controlled by that player, then it doesn’t block. If there’s a cost associated with having the creature block, its controller isn’t forced to pay that cost, so it doesn’t block in that case either."
    );
    supported("Boros Battleshaper");
    // "At the beginning of each combat, up to one target creature attacks or blocks this
    // combat if able and up to one target creature can't attack or block this combat."
    // P0's Bears, forced to attack: attacking P1 or the battle P1 protects obeys it; not
    // attacking doesn't.
    let (mut t, b) = two_with_siege();
    t.battlefield(P0, "Boros Battleshaper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(bears, obj(b))]), "{POLICY}");
    // P1's Savannah Lions, forced to block, in a three-player game where P1 protects P0's
    // Siege. P0's Bears attack P1 (and with a Hill Giant attacking P1 too, P1 chooses which
    // of them the Lions block), the battle P1 protects, or P2. The Lions must block unless,
    // as blockers are declared, they're tapped or can't block (Pacifism), no creature is
    // attacking P1, a planeswalker P1 controls or a battle P1 protects, or blocking costs
    // (War Cadence for 1; P1 has the mana).
    for case in [
        "P1",
        "two attackers",
        "the battle",
        "tapped",
        "can't block",
        "P2",
        "cost",
    ] {
        let mut t = TestGame::new(3);
        let b = siege(&mut t, P0, P1);
        t.battlefield(P0, "Boros Battleshaper");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let lions = t.battlefield(P1, "Savannah Lions");
        t.lands(P1, "Plains", 1);
        t.answer_targets(P0, &[obj(lions)]);
        t.answer_targets(P0, &[]);
        t.set_step(P0, Step::PrecombatMain);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        match case {
            "tapped" => {
                t.g.tap(lions);
            }
            "can't block" => {
                attach_new(&mut t, P2, "Pacifism", lions);
            }
            "cost" => {
                t.lands(P0, "Mountain", 2);
                let cadence = t.battlefield(P0, "War Cadence");
                t.answer(P0, DecisionKind::X, Answer::Number(1));
                t.activate(P0, cadence, 0, &[]).unwrap();
                t.resolve_all();
            }
            _ => {}
        }
        let attacks = match case {
            "two attackers" => vec![(bears, pl(P1)), (giant, pl(P1))],
            "the battle" => vec![(bears, obj(b))],
            "P2" => vec![(bears, pl(P2))],
            _ => vec![(bears, pl(P1))],
        };
        attack_with(&mut t, &attacks);
        let forced = matches!(case, "P1" | "two attackers" | "the battle");
        assert_eq!(
            legal_blocks(&mut t, P1, &[]),
            !forced,
            "{case}: no block ({POLICY})"
        );
        let can_block = forced || case == "cost";
        assert_eq!(
            legal_blocks(&mut t, P1, &[(lions, bears)]),
            can_block,
            "{case}: the Lions block the Bears"
        );
        if case == "two attackers" {
            assert!(legal_blocks(&mut t, P1, &[(lions, giant)]), "{case}");
        }
    }
}

/// The creatures attacking now that aren't among `declared`.
fn new_attackers(t: &TestGame, declared: &[ObjectId]) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .map(|a| a.id)
                .filter(|id| !declared.iter().any(|d| t.g.current(*d) == *id))
                .collect()
        })
        .unwrap_or_default()
}

/// The "what the creature is attacking" choices asked since `from`.
fn attack_choices(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if prompt.contains("is attacking") => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_creature_put_onto_the_battlefield_attacking_may_attack_a_battle() {
    cr!("508.4");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Militia's Pride",
        "You choose which player or planeswalker the token is attacking. It doesn't have to attack the same player or planeswalker as the creature that caused the ability to trigger."
    );
    ruling!(
        "Minion of the Mighty",
        "You choose which player or planeswalker the Dragon is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Minion of the Mighty is attacking."
    );
    ruling!(
        "Najeela, the Blade-Blossom",
        "The controller of each Warrior token created by Najeela chooses which player or planeswalker it's attacking. The tokens don't have to be attacking the same player or planeswalker as the creature that caused the ability to trigger. They can even be attacking players or planeswalkers that weren't being attacked."
    );
    ruling!(
        "Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier",
        "Ultra Magnus's controller chooses which player or planeswalker the artifact creature is attacking. It doesn't have to be the same player or planeswalker that Ultra Magnus is attacking."
    );
    // P0 attacks P1's planeswalker; each new creature is offered P1, the planeswalker and
    // the battle P1 protects, and attacks the battle.
    for name in [
        "Militia's Pride",
        "Minion of the Mighty",
        "Najeela, the Blade-Blossom",
        "Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier",
    ] {
        supported(name);
        let (mut t, b) = two_with_siege();
        let jace = t.battlefield(P1, "Jace Beleren");
        let mut attackers = Vec::new();
        match name {
            "Militia's Pride" => {
                t.battlefield(P0, name);
                t.lands(P0, "Plains", 1);
                t.answer_yes(P0, true);
                attackers.push(t.battlefield(P0, "Grizzly Bears"));
            }
            "Minion of the Mighty" => {
                // Pack tactics: total power 6 or greater (0 + 6).
                attackers.push(t.battlefield(P0, name));
                attackers.push(t.battlefield(P0, "Craw Wurm"));
                let dragon = t.hand(P0, "Shivan Dragon");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(dragon)]);
            }
            "Najeela, the Blade-Blossom" => {
                attackers.push(t.battlefield(P0, name));
                t.answer_yes(P0, true);
            }
            _ => {
                attackers.push(t.battlefield(P0, name));
                let thopter = t.hand(P0, "Ornithopter");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[obj(thopter)]);
            }
        }
        t.answer_choose(P0, &[obj(b)]);
        let from = t.asked().len();
        to_combat(&mut t, P0);
        let decl: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, obj(jace))).collect();
        attack_with(&mut t, &decl);
        t.resolve_all();
        let new = new_attackers(&t, &attackers);
        assert_eq!(new.len(), 1, "{name}: {}", t.dump_log());
        assert_eq!(attack_target(&t, new[0]), Some(obj(b)), "{name} ({POLICY})");
        let offered = attack_choices(&t, from);
        assert_eq!(offered.len(), 1, "{name}");
        for e in [Entity::Player(P1), obj(jace), obj(b)] {
            assert!(offered[0].contains(&e), "{name}: {e:?} in {:?}", offered[0]);
        }
    }
}

#[test]
fn a_ninja_enters_attacking_the_battle_the_returned_creature_was_attacking() {
    cr!("702.49c");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Ninja of the Deep Hours",
        "The creature put onto the battlefield with ninjutsu enters the battlefield attacking the same player or planeswalker that the returned creature was attacking."
    );
    supported("Ninja of the Deep Hours");
    let (mut t, b) = two_with_siege();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    to_blockers(&mut t, &[(bears, obj(b))], &[]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, ninja, 0, &[]).expect("ninjutsu");
    t.resolve_all();
    assert_eq!(attack_target(&t, ninja), Some(obj(b)), "{POLICY}");
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn a_band_attacks_one_battle() {
    cr!("702.22d");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Benalish Hero",
        "Creatures in the same band must all attack the same player or planeswalker."
    );
    supported("Benalish Hero");
    let band_of = |t: &TestGame, a: ObjectId| {
        t.g.combat
            .as_ref()
            .and_then(|c| c.attacker(a))
            .and_then(|x| x.band)
    };
    // Both attacking the battle: a band.
    let (mut t, b) = two_with_siege();
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(hero, obj(b)), (bears, obj(b))]);
    assert!(band_of(&t, hero).is_some(), "{POLICY}");
    assert_eq!(band_of(&t, hero), band_of(&t, bears));
    // One attacking the battle, one its protector: no band.
    let (mut t, b) = two_with_siege();
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(hero, obj(b)), (bears, Entity::Player(P1))]);
    assert_eq!(band_of(&t, hero), None);
    assert_eq!(band_of(&t, bears), None);
}

#[test]
fn a_reselected_attacker_may_attack_a_battle() {
    cr!("508.7", "508.7a", "508.7c");
    interp!("pre-battle-rulings-give-way-to-the-cr");
    ruling!(
        "Portal Mage",
        "If you reselect which player or planeswalker an attacking creature is attacking, that creature is still considered to have attacked the player or planeswalker as declared, but it is now attacking the new player or planeswalker."
    );
    supported("Portal Mage");
    // "When this creature enters during the declare attackers step, you may reselect which
    // player or permanent target attacking creature is attacking." Three players: P1
    // controls a Siege that P2 protects. P0's Bears attack P1; P1 flashes in Portal Mage and
    // sends them at the Siege. They still attacked P1, as declared, and now attack the
    // battle, which takes their damage.
    let mut t = TestGame::new(3);
    let b = siege(&mut t, P1, P2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, pl(P1))]);
    let options = mtg_engine::kw::reselect_attack::reselect_options(&t.g, bears);
    assert!(options.contains(&obj(b)), "{options:?} ({POLICY})");
    give_mana_for(&mut t, P1, "Portal Mage");
    let mage = t.hand(P1, "Portal Mage");
    t.answer_targets(P1, &[obj(bears)]);
    t.answer_choose(P1, &[obj(b)]);
    t.cast(P1, mage).go();
    t.resolve();
    t.resolve();
    assert_eq!(attack_target(&t, bears), Some(obj(b)), "{POLICY}");
    assert!(player_has_attacked(&t.g, P0, P1));
    let defense = t.counters(b, mtg_engine::types::counters::DEFENSE);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(
        t.counters(b, mtg_engine::types::counters::DEFENSE),
        defense - 2
    );
    assert_eq!((t.life(P1), t.life(P2)), (20, 20));
}
