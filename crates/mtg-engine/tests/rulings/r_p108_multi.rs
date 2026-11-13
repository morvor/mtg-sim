//! Rulings batch P108 — multi-character cards and "multi-land ramp": Trostani Discordant
//! (an anthem and "each player gains control of all creatures they own"), a triggered
//! mana ability (CR 605.1b), creatures put onto the battlefield attacking (CR 508.4),
//! several permanents entering at once (CR 603.6a), extra land plays (CR 305.2), and
//! lands returned "tapped" (CR 614.1c).

use crate::r_p108_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Trostani Discordant -------------------------------------------------------------------

#[test]
fn trostani_leaving_can_make_damage_lethal() {
    cr!("120.6", "704.5g", "613.4c");
    ruling!(
        "Trostani Discordant",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to other creatures you control may become lethal if Trostani leaves the battlefield during that turn."
    );
    supported("Trostani Discordant");
    let mut t = TestGame::new(2);
    let tr = t.battlefield(P0, "Trostani Discordant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    cast_new(&mut t, P1, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    destroy(&mut t, tr);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn trostani_returns_a_token_to_the_player_who_created_it() {
    cr!("111.2", "613.1b");
    ruling!("Trostani Discordant", "The owner of a token is the player who created it.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trostani Discordant");
    let tok = create_token(&mut t, P1, "Soldier");
    let mine = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, tok, P0);
    give_control(&mut t, mine, P1);
    assert_eq!(t.obj_now(tok).controller, P0);
    end_step(&mut t, P0);
    assert_eq!(t.obj_now(tok).controller, P1);
    assert_eq!(t.obj_now(mine).controller, P0);
}

#[test]
fn trostani_regained_creatures_end_step_abilities() {
    cr!("603.3a", "603.2");
    ruling!(
        "Trostani Discordant",
        "If a creature has an ability that triggers at the beginning of each end step and Trostani's ability causes you to gain control of it, the ability of that creature is still controlled by the creature's former controller. If the creature has an ability that triggers at the beginning of your end step, that ability doesn't trigger when you gain control of it."
    );
    supported("The Gaffer");
    supported("Twinblade Assassins");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trostani Discordant");
    // "At the beginning of each end step, if you gained 3 or more life this turn, draw a
    // card." P1 controls P0's Gaffer and gained life.
    let gaffer = t.battlefield(P0, "The Gaffer");
    give_control(&mut t, gaffer, P1);
    t.g.gain_life(P1, 3);
    t.g.flush_events();
    // "At the beginning of your end step, if a creature died this turn, draw a card."
    let twinblade = t.battlefield(P0, "Twinblade Assassins");
    give_control(&mut t, twinblade, P1);
    a_creature_dies(&mut t, P1);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.advance_to(P0, Step::End);
    t.settle();
    let gaffer_trigger = crate::r_s25_common::abilities_from(&t, gaffer);
    assert_eq!(gaffer_trigger.len(), 1);
    assert_eq!(t.g.obj(gaffer_trigger[0]).controller, P1);
    assert!(crate::r_s25_common::abilities_from(&t, twinblade).is_empty());
    t.resolve_all();
    assert_eq!(t.obj_now(gaffer).controller, P0);
    assert_eq!(t.obj_now(twinblade).controller, P0);
    // P1 drew for the Gaffer; P0 drew for nothing (the Twinblade didn't trigger).
    assert_eq!(t.hand_size(P1), h1 + 1);
    assert_eq!(t.hand_size(P0), h0);
    assert!(crate::r_s25_common::abilities_from(&t, twinblade).is_empty());
}

// --- Groundchuck & Dirtbag ------------------------------------------------------------------

#[test]
fn groundchuck_and_dirtbags_ability_is_a_mana_ability() {
    cr!("605.1b", "605.4a");
    ruling!(
        "Groundchuck & Dirtbag",
        "last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Groundchuck & Dirtbag");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Groundchuck & Dirtbag");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).expect("tap for mana");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
}

// --- Raph & Mikey ---------------------------------------------------------------------------

#[test]
fn raph_and_mikey_choose_what_the_new_attacker_attacks() {
    cr!("508.4");
    ruling!(
        "Raph & Mikey, Troublemakers",
        "You choose the player, planeswalker, or battle the creature you put onto the battlefield is attacking. It doesn't have to be the same player, planeswalker, or battle that Raph & Mikey or any other attacking creatures are attacking."
    );
    supported("Raph & Mikey, Troublemakers");
    let mut t = TestGame::new(3);
    let rm = t.battlefield(P0, "Raph & Mikey, Troublemakers");
    t.g.players[0].library.clear();
    let giant = t.library_top(P0, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(rm, Entity::Player(P1))]),
    );
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P2)]));
    t.advance_to(P0, Step::DeclareAttackers);
    let from = t.asked().len();
    t.resolve_all();
    let combat = t.g.combat.as_ref().expect("combat");
    let g = t.g.current(giant);
    let target = combat
        .attackers
        .iter()
        .find(|a| a.id == g)
        .and_then(|a| a.target);
    assert_eq!(target, Some(Entity::Player(P2)), "{:?}", &t.asked()[from..]);
}

// --- Genesis Wave, Splendid Reclamation, Azusa ------------------------------------------------

#[test]
fn genesis_wave_permanents_enter_together_and_see_each_other() {
    cr!("603.6a");
    ruling!(
        "Genesis Wave",
        "All of the permanents put onto the battlefield this way enter at the same time. If any have triggered abilities that trigger on something else entering, they'll see each other."
    );
    supported("Genesis Wave");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    crate::r_s01_common::stack_library(&mut t, P0, &["Soul Warden", "Grizzly Bears", "Forest"]);
    t.answer_choose(
        P0,
        &t.g.player(P0).library.iter().map(|c| obj(*c)).collect::<Vec<_>>(),
    );
    lands_for_cost(&mut t, P0, "Genesis Wave");
    t.lands(P0, "Wastes", 3);
    let wave = t.hand(P0, "Genesis Wave");
    t.cast(P0, wave).x(3).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 1);
    // Soul Warden saw the Bears enter with it.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn genesis_wave_can_always_put_lands() {
    cr!("202.3a");
    ruling!(
        "Genesis Wave",
        "If a permanent card in your library has no mana symbols in its upper right corner (because it's a land card, for example), its mana value is 0. Such cards can always be put onto the battlefield with Genesis Wave."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    let lib = crate::r_s01_common::stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    lands_for_cost(&mut t, P0, "Genesis Wave");
    t.lands(P0, "Wastes", 2);
    let wave = t.hand(P0, "Genesis Wave");
    let before = t.named_on_battlefield("Forest").len();
    t.cast(P0, wave).x(2).go();
    t.resolve_all();
    // The Forest (mana value 0) is put onto the battlefield; the Hill Giant (4) can't be.
    assert_eq!(t.named_on_battlefield("Forest").len(), before + 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn splendid_reclamation_returns_check_lands_tapped() {
    cr!("614.1c", "614.12");
    ruling!(
        "Splendid Reclamation",
        "If an effect states that a land enters the battlefield tapped unless a condition is met, Splendid Reclamation's effect puts that land onto the battlefield tapped even if that condition is true."
    );
    supported("Splendid Reclamation");
    supported("Glacial Fortress");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    t.graveyard(P0, "Glacial Fortress");
    cast_new(&mut t, P0, "Splendid Reclamation", &[]);
    t.resolve_all();
    let f = t.named_on_battlefield("Glacial Fortress");
    assert_eq!(f.len(), 1);
    assert!(t.obj_now(f[0]).tapped);
}

#[test]
fn azusa_is_cumulative_with_other_extra_land_plays() {
    cr!("305.2", "305.2a");
    ruling!(
        "Azusa, Lost but Seeking",
        "Azusa's ability is cumulative with other effects that allow you to play additional lands"
    );
    supported("Azusa, Lost but Seeking");
    supported("Exploration");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Azusa, Lost but Seeking");
    t.battlefield(P0, "Exploration");
    let mut played = 0;
    for _ in 0..6 {
        let land = t.hand(P0, "Forest");
        if t.play_land(P0, land).is_ok() {
            played += 1;
        }
    }
    assert_eq!(played, 4);
}
