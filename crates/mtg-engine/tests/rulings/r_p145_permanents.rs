//! Rulings batch P145 — combat and static abilities of the looting creatures: evasion
//! gained after blocks, tokens entering attacking, Felothar's damage assignment, threshold,
//! Duelist of the Mind's power in every zone, targets, Sanctum of Calm Waters without
//! Shrines, and Mirror-Mad Phantasm copies.

use crate::r_p145_common::*;
use crate::r_s02_common::target_candidates;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with `attacker`, P1 blocks it with a Grizzly Bears; the game stops in
/// the declare blockers step with P0 having priority. Returns the blocker.
fn blocked(t: &mut TestGame, attacker: ObjectId) -> ObjectId {
    let blocker = t.battlefield(P1, "Grizzly Bears");
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(attacker));
    blocker
}

#[test]
fn evasion_gained_after_blocks_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Daring Saboteur",
        "Activating the first ability of Daring Saboteur after it has become blocked won't cause it to become unblocked."
    );
    ruling!(
        "Furtive Courier",
        "Sacrificing an artifact after Furtive Courier has been blocked won't cause it to become unblocked."
    );
    supported("Daring Saboteur");
    supported("Furtive Courier");
    let mut t = TestGame::new(2);
    let saboteur = t.battlefield(P0, "Daring Saboteur");
    let blocker = blocked(&mut t, saboteur);
    t.lands(P0, "Island", 3);
    activate_containing(&mut t, P0, saboteur, "can't be blocked").expect("activate");
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = blocker;
    // Furtive Courier: a Treasure sacrificed (for mana) after blocks.
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Furtive Courier");
    let treasure = create_token(&mut t, P0, "Treasure");
    t.hand(P0, "Island");
    blocked(&mut t, courier);
    t.activate(P0, treasure, 0, &[]).expect("Treasure");
    assert!(!t.on_battlefield(treasure));
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn leonin_warleader_tokens_attack_what_their_controller_chooses() {
    cr!("508.4", "506.3");
    ruling!(
        "Leonin Warleader",
        "You choose which players or planeswalkers the two tokens are attacking. They don’t have to be attacking the same player or planeswalker that Leonin Warleader is attacking"
    );
    supported("Leonin Warleader");
    let mut t = TestGame::new(3);
    let warleader = t.battlefield(P0, "Leonin Warleader");
    let walker = t.battlefield(P2, "Jace Reawakened");
    attack_with(&mut t, &[(warleader, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[obj(walker)]);
    t.resolve_all();
    let combat = t.g.combat.as_ref().unwrap();
    let mut targets: Vec<Entity> = tokens(&t, P0)
        .into_iter()
        .map(|c| combat.attack_target(c).expect("attacking"))
        .collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![Entity::Player(P2), obj(walker)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, want);
    assert_eq!(combat.attack_target(warleader), Some(Entity::Player(P1)));
}

#[test]
fn felothar_changes_damage_assigned_not_power() {
    cr!("510.1a", "510.1", "208.1");
    ruling!(
        "Felothar the Steadfast",
        "Felothar’s first ability doesn’t actually change any creature’s power. It changes only the amount of combat damage those creatures assign."
    );
    supported("Felothar the Steadfast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felothar the Steadfast");
    let wall = t.battlefield(P0, "Wall of Omens");
    attack_with(&mut t, &[(wall, Entity::Player(P1))]);
    assert_eq!(t.pt(wall), (0, 4));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(wall), (0, 4));
}

#[test]
fn dreadwing_scavenger_marked_damage_becomes_lethal() {
    cr!("704.5g", "120.6");
    ruling!(
        "Dreadwing Scavenger",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Dreadwing Scavenger may become lethal if the number of cards in your graveyard falls below seven."
    );
    supported("Dreadwing Scavenger");
    let mut t = TestGame::new(2);
    let scavenger = t.battlefield(P0, "Dreadwing Scavenger");
    fill_graveyard(&mut t, P0, 7);
    t.g.recompute();
    assert_eq!(t.pt(scavenger), (3, 3));
    cast_resolve(&mut t, P1, "Shock", &[obj(scavenger)]);
    assert!(t.on_battlefield(scavenger));
    let one = t.g.player(P0).graveyard[0];
    move_to(&mut t, one, Zone::Exile);
    assert!(t.in_graveyard(P0, "Dreadwing Scavenger"));
}

#[test]
fn duelist_of_the_mind_power_works_in_every_zone() {
    cr!("604.3", "208.3");
    ruling!(
        "Duelist of the Mind",
        "The ability that defines Duelist of the Mind’s power works in all zones, not just the battlefield."
    );
    supported("Duelist of the Mind");
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Duelist of the Mind");
    let gy = t.graveyard(P0, "Duelist of the Mind");
    crate::r_p143_common::draw(&mut t, P0, 2);
    t.g.recompute();
    assert_eq!(t.pt(hand), (2, 3));
    assert_eq!(t.pt(gy), (2, 3));
}

#[test]
fn targets_harrier_strix_and_obsessive_stitcher() {
    cr!("115.1", "601.2c", "602.2b");
    ruling!(
        "Harrier Strix",
        "You may target a permanent that is already tapped with Harrier Strix’s second ability."
    );
    ruling!(
        "Obsessive Stitcher",
        "Obsessive Stitcher can't be the target of its last ability."
    );
    supported("Harrier Strix");
    supported("Obsessive Stitcher");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.g.objects[land.0 as usize].tapped = true;
    let from = n_asked(&t);
    t.answer_targets(P0, &[obj(land)]);
    enter(&mut t, P0, "Harrier Strix");
    assert!(target_candidates(&t, P0, from)[0].contains(&obj(land)));
    assert_eq!(targets_of(&t, t.g.stack[0]), vec![obj(land)]);
    // Stitcher: it's on the battlefield as its target is chosen (the sacrifice comes
    // later), so it isn't a creature card in the graveyard.
    let mut t = TestGame::new(2);
    let stitcher = t.battlefield(P0, "Obsessive Stitcher");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let from = n_asked(&t);
    activate_containing(&mut t, P0, stitcher, "Return target").expect("activate");
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(bears)]]);
    assert!(t.in_graveyard(P0, "Obsessive Stitcher"));
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert!(t.in_graveyard(P0, "Obsessive Stitcher"));
}

#[test]
fn sanctum_of_calm_waters_without_shrines() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Sanctum of Calm Waters",
        "If Sanctum of Calm Waters leaves the battlefield while its ability is on the stack, you may control no Shrines as the ability resolves. If so, you may choose to draw zero cards and discard one"
    );
    supported("Sanctum of Calm Waters");
    for draw in [true, false] {
        let mut t = TestGame::new(2);
        let sanctum = t.battlefield(P0, "Sanctum of Calm Waters");
        t.hand(P0, "Island");
        t.set_step(P0, Step::Draw);
        assert!(t.g.run_until(1000, |g| !g.stack.is_empty()));
        assert_eq!(t.g.turn.step, Step::PrecombatMain);
        assert_eq!(stacked_triggers(&t), 1);
        destroy(&mut t, sanctum);
        t.answer_yes(P0, draw);
        t.resolve();
        assert_eq!(t.library_size(P0), 30, "draws zero");
        assert_eq!(t.hand_size(P0), usize::from(!draw), "draw: {draw}");
    }
}

#[test]
fn mirror_mad_phantasm_token_mills_the_whole_library() {
    cr!("111.7", "111.6", "701.20a");
    ruling!(
        "Mirror-Mad Phantasm",
        "If no card named Mirror-Mad Phantasm is revealed (possibly because it was a card copying Mirror-Mad Phantasm or it was a token), all cards from that library will be put into their owner's graveyard."
    );
    let mut t = TestGame::new(2);
    let phantasm = t.battlefield(P0, "Mirror-Mad Phantasm");
    let copy = crate::r_s17_common::token_copy(&mut t, P0, phantasm)[0];
    let lib = t.library_size(P0);
    t.lands(P0, "Island", 2);
    t.activate(P0, copy, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), lib);
    assert_eq!(
        t.named_on_battlefield("Mirror-Mad Phantasm"),
        vec![phantasm]
    );
}
