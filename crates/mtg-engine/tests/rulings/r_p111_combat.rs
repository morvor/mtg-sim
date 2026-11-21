//! Rulings batch P111 — activated abilities of attackers and blockers (CR 509.1, 509.1h,
//! 506.4): evasion gained after blockers are declared doesn't change blocks; repeated
//! menace is redundant (CR 702.111b) while extra blocks are cumulative (CR 509.1b);
//! "must be blocked if able" is a blocking requirement (CR 509.1c); "defending player"
//! (CR 506.2, 508.5); creatures put onto the battlefield attacking were never declared as
//! attackers (CR 506.3a, 508.4, 508.4a).

use crate::r_p076_common::unblockable_after_blocked;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::activate_containing;
use crate::r_s09_common::to_combat;
use crate::r_s10_common::{attacking, blocking};
use crate::r_s21_common::{blocks_now, legal_blocks};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, cost: &str) {
    for c in cost.chars() {
        let ty = match c {
            'W' => ManaType::W,
            'U' => ManaType::U,
            'B' => ManaType::B,
            'R' => ManaType::R,
            'G' => ManaType::G,
            d => {
                let n = d.to_digit(10).expect("a mana symbol");
                t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
                continue;
            }
        };
        t.g.players[p.idx()].mana_pool.add_type(ty, 1);
    }
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn evasion_gained_after_blocks_doesnt_change_them() {
    cr!("509.1h", "506.4", "702.9b", "702.111b");
    ruling!(
        "Gruul Nodorog",
        "Activating Gruul Nodorog's ability after blockers have been declared has no effect on the creatures already blocking it."
    );
    ruling!(
        "Harbor Bandit",
        "Activating Harbor Bandit's ability after it's been blocked won't cause it to become unblocked."
    );
    ruling!(
        "Leaping Master",
        "Activating Leaping Master's ability after it's been blocked won't change or undo the block."
    );
    ruling!(
        "Agent of Horizons",
        "Activating the ability of Agent of Horizons after it's been blocked won't change or undo the block."
    );
    ruling!(
        "Dragonsoul Knight",
        "If Dragonsoul Knight gains flying after blockers have been declared, it won’t cause those blocks to change."
    );
    ruling!(
        "Paragon of the Amesha",
        "If Paragon of the Amesha gains flying after blockers have been declared, it won’t cause those blocks to change."
    );
    // (attacker, cost, the activated ability's text, a keyword it gains or None)
    for (name, cost, needle) in [
        ("Gruul Nodorog", "R", "menace"),
        ("Harbor Bandit", "1U", "can't be blocked"),
        ("Leaping Master", "2W", "flying"),
        ("Agent of Horizons", "2U", "can't be blocked"),
        ("Dragonsoul Knight", "WUBRG", "Dragon"),
        ("Paragon of the Amesha", "WUBRG", "Angel"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        // A blocker that survives Dragonsoul Knight's trample.
        let blocker = t.battlefield(P1, "Wall of Stone");
        unblockable_after_blocked(&mut t, attacker, blocker, |t| {
            mana(t, P0, cost);
            activate_containing(t, P0, attacker, needle).expect("activation");
            t.resolve_all();
            // The effect did apply: this block couldn't be declared now.
            assert!(!legal_blocks(t, P1, &[(blocker, attacker)]), "{name}");
        });
    }
}

#[test]
fn either_ability_activated_after_blocks_doesnt_change_them() {
    cr!("509.1h", "506.4", "509.1c");
    ruling!(
        "Nacatl Hunt-Pride",
        "If either ability is activated after blockers have been declared, it won’t cause those blocks to change."
    );
    supported("Nacatl Hunt-Pride");
    // "{R}, {T}: Target creature can't block this turn." on the blocker, and "{G}, {T}:
    // Target creature blocks this turn if able." on a creature that didn't block.
    let mut t = TestGame::new(2);
    let nacatl = t.battlefield(P0, "Nacatl Hunt-Pride");
    let other = t.battlefield(P0, "Nacatl Hunt-Pride");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(nacatl, Entity::Player(P1))], &[(blocker, nacatl)]);
    mana(&mut t, P0, "RG");
    t.answer_targets(P0, &[obj(blocker)]);
    activate_containing(&mut t, P0, nacatl, "can't block").expect("{R}");
    t.answer_targets(P0, &[obj(idle)]);
    activate_containing(&mut t, P0, other, "blocks this turn").expect("{G}");
    t.resolve_all();
    assert_eq!(blocks_now(&t), vec![(blocker, nacatl)]);
    assert!(!blocking(&t, idle));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the blocker was dealt damage");
}

#[test]
fn a_vigilant_attacker_can_activate_its_tap_ability_before_blockers() {
    cr!("508.8", "702.20b", "509.1a");
    ruling!(
        "Nacatl Hunt-Pride",
        "If you attack with Nacatl Hunt-Pride, you get the chance to activate one of its abilities before blockers are declared."
    );
    let mut t = TestGame::new(2);
    let nacatl = t.battlefield(P0, "Nacatl Hunt-Pride");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(nacatl, Entity::Player(P1))]);
    assert!(attacking(&t, nacatl) && !t.obj_now(nacatl).tapped);
    assert!(legal_blocks(&mut t, P1, &[(bears, nacatl)]), "control");
    mana(&mut t, P0, "R");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, nacatl, "can't block").expect("{R}, {T}");
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, nacatl)]));
}

#[test]
fn a_second_menace_has_no_further_effect() {
    cr!("702.111b", "509.1b");
    ruling!(
        "Gruul Nodorog",
        "Activating Gruul Nodorog's ability a second time in the same turn has no further effect."
    );
    let mut t = TestGame::new(2);
    let gruul = t.battlefield(P0, "Gruul Nodorog");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..2 {
        mana(&mut t, P0, "R");
        activate_containing(&mut t, P0, gruul, "menace").unwrap();
        t.resolve_all();
    }
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(gruul, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(b1, gruul)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, gruul), (b2, gruul)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, gruul), (b2, gruul), (b3, gruul)]));
}

#[test]
fn extra_blocks_are_cumulative() {
    cr!("509.1b", "509.1a");
    ruling!(
        "Coastline Chimera",
        "Coastline Chimera's activated ability is cumulative. Activating it a second time will allow it to block three creatures, and so on."
    );
    supported("Coastline Chimera");
    let mut t = TestGame::new(2);
    let attackers = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Grizzly Bears"),
    ];
    let chimera = t.battlefield(P1, "Coastline Chimera");
    to_combat(&mut t, P0);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(&mut t, &decl);
    let blocks: Vec<_> = attackers.iter().map(|a| (chimera, *a)).collect();
    assert!(!legal_blocks(&mut t, P1, &blocks[..2]), "control");
    for n in [2, 3] {
        mana(&mut t, P1, "1W");
        activate_containing(&mut t, P1, chimera, "additional").unwrap();
        t.resolve_all();
        assert!(legal_blocks(&mut t, P1, &blocks[..n]), "{n} blocks");
        if n == 2 {
            assert!(!legal_blocks(&mut t, P1, &blocks[..3]));
        }
    }
}

#[test]
fn must_be_blocked_if_able_requires_a_blocker() {
    cr!("509.1c", "509.1a");
    ruling!(
        "Loathsome Catoblepas",
        "If Loathsome Catoblepas is attacking and its activated ability has resolved, the defending player must assign at least one blocker to it during the declare blockers step if that player controls any creatures that could block it."
    );
    supported("Loathsome Catoblepas");
    let mut t = TestGame::new(2);
    let cato = t.battlefield(P0, "Loathsome Catoblepas");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bird = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(cato, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]), "control");
    mana(&mut t, P0, "2G");
    activate_containing(&mut t, P0, cato, "must be blocked").unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, cato)]));
    assert!(legal_blocks(&mut t, P1, &[(bears, cato), (bird, cato)]));
    // With no creature that could block it, no block is required.
    let mut t = TestGame::new(2);
    let cato = t.battlefield(P0, "Loathsome Catoblepas");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(cato, Entity::Player(P1))]);
    mana(&mut t, P0, "2G");
    activate_containing(&mut t, P0, cato, "must be blocked").unwrap();
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn defending_player_is_only_the_player_this_creature_attacks() {
    cr!("508.5", "508.5a", "506.2");
    ruling!(
        "Torrent Elemental",
        "In multiplayer formats that allow attacking multiple players, creatures controlled by other players won't be tapped, even if you control other creatures attacking those players or planeswalkers those players control."
    );
    supported("Torrent Elemental");
    // "Whenever this creature attacks, tap all creatures defending player controls."
    let mut t = TestGame::new(3);
    let torrent = t.battlefield(P0, "Torrent Elemental");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let p1s = t.battlefield(P1, "Grizzly Bears");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(torrent, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    t.resolve_all();
    assert!(t.obj_now(p1s).tapped);
    assert!(!t.obj_now(p2s).tapped);
}

/// P0's Alesha attacks P1 (with `with` also attacking); its trigger, paid, targets `card`
/// in P0's graveyard.
fn alesha_attacks(t: &mut TestGame, card: ObjectId, with: &[ObjectId]) -> ObjectId {
    supported("Alesha, Who Smiles at Death");
    let alesha = t.battlefield(P0, "Alesha, Who Smiles at Death");
    to_combat(t, P0);
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.lands(P0, "Plains", 2);
    let mut decl = vec![(alesha, Entity::Player(P1))];
    decl.extend(with.iter().map(|c| (*c, Entity::Player(P1))));
    attack_with(t, &decl);
    t.resolve_all();
    alesha
}

#[test]
fn alesha_checks_the_power_only_in_the_graveyard() {
    cr!("115.1", "608.2b", "506.3a");
    ruling!(
        "Alesha, Who Smiles at Death",
        "Alesha's triggered ability checks the card's power only in your graveyard. The creature will still be tapped and attacking if its power is greater than 2 after it's on the battlefield."
    );
    // Glorious Anthem: "Creatures you control get +1/+1." Grizzly Bears is a 2/2 card in
    // the graveyard and a 3/3 attacker on the battlefield.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let bears = t.graveyard(P0, "Grizzly Bears");
    alesha_attacks(&mut t, bears, &[]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(attacking(&t, t.g.current(bears)) && t.obj_now(bears).tapped);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 4 - 3);
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_werent_declared_as_attackers() {
    cr!("506.3a", "508.4", "508.4a", "508.1a");
    ruling!(
        "Alesha, Who Smiles at Death",
        "Although the returned creature is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    ruling!(
        "Alesha, Who Smiles at Death",
        "Any effects that say that the returned creature can't attack (such as that of Propaganda, or if the creature has defender) affect only the declaration of attackers. They won't stop the returned creature from entering the battlefield attacking."
    );
    ruling!(
        "Najeela, the Blade-Blossom",
        "Although the tokens created by Najeela's first ability are attacking, they were never declared as attacking creatures (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    supported("Hellrider");
    supported("Propaganda");
    // Hellrider: "Whenever a creature you control attacks, Hellrider deals 1 damage to
    // the player or planeswalker it's attacking." Alesha attacking: 1 damage; the
    // returned Wall of Wood (0/3 defender), despite Propaganda, adds none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    t.battlefield(P1, "Propaganda");
    let wall = t.graveyard(P0, "Wall of Wood");
    // Propaganda's attack cost for Alesha.
    t.lands(P0, "Wastes", 2);
    alesha_attacks(&mut t, wall, &[]);
    assert!(t.on_battlefield(wall));
    assert!(attacking(&t, t.g.current(wall)));
    assert_eq!(t.life(P1), 19);
    // Najeela: "Whenever a Warrior attacks, you may have its controller create a 1/1
    // white Warrior creature token that's tapped and attacking." The token, a Warrior,
    // doesn't trigger Najeela or Hellrider.
    supported("Najeela, the Blade-Blossom");
    let mut t = TestGame::new(2);
    let najeela = t.battlefield(P0, "Najeela, the Blade-Blossom");
    t.battlefield(P0, "Hellrider");
    to_combat(&mut t, P0);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    attack_with(&mut t, &[(najeela, Entity::Player(P1))]);
    t.resolve_all();
    let warriors: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Warrior"))
        .map(|o| o.id)
        .collect();
    assert_eq!(warriors.len(), 1);
    assert!(attacking(&t, warriors[0]));
    assert_eq!(t.life(P1), 19, "Hellrider triggered for Najeela only");
}
