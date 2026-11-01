//! Rulings batch P004 — attack-matters abilities: what they count and when (CR 603.4,
//! 608.2h), creatures put onto the battlefield attacking (CR 508.4), Underworld Sentinel,
//! Crumbling Colossus and Augusta.

use crate::r_p004_common::*;
use crate::r_p079_common::{concede, enter_attacking};
use crate::r_s01_common::{attack_with, block_and_finish};
use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
use mtg_engine::testing::*;
use mtg_engine::*;

fn p(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `who` gains control of `id` (permanently).
fn give_control(t: &mut TestGame, id: ObjectId, who: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, who);
    ctx.targets = vec![vec![obj(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn military_intelligence_counts_attackers_attacking_different_players() {
    cr!("508.1", "603.2");
    ruling!(
        "Military Intelligence",
        "The creatures don’t have to be attacking the same player or planeswalker."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Military Intelligence");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, p(P1)), (b, p(P2))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn attack_triggers_count_as_they_resolve() {
    cr!("608.2h", "508.4", "603.3b");
    ruling!(
        "Stormbeacon Blade",
        "The second ability checks the number of attacking creatures you control only as the ability resolves. If another attack trigger (such as from the mobilize ability) puts creatures onto the battlefield attacking, you can choose to order the triggered abilities so that those creatures enter before Stormbeacon Blade’s ability resolves."
    );
    ruling!(
        "Marshal of the Lost",
        "The value of X is calculated only once, as Marshal of the Lost’s last ability resolves."
    );
    ruling!(
        "Terra Ravager",
        "Count the number of lands the defending player controls when the ability resolves to determine the value of X."
    );
    // Stormbeacon Blade: two attackers, then a third enters attacking before the trigger
    // resolves: draw.
    for extra in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let blade = t.battlefield(P0, "Stormbeacon Blade");
        assert!(t.g.attach(blade, obj(a)));
        let hand = t.hand_size(P0);
        attack_with(&mut t, &[(a, p(P1)), (b, p(P1))]);
        assert_eq!(t.stack_len(), 1);
        if extra {
            enter_attacking(&mut t, P0, "Raging Goblin", p(P1));
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + extra as usize, "extra={extra}");
    }
    // Marshal of the Lost: X is the number of attackers as it resolves, fixed afterwards.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Marshal of the Lost");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(m)]);
    attack_with(&mut t, &[(m, p(P1)), (b, p(P1))]);
    let goblin = enter_attacking(&mut t, P0, "Raging Goblin", p(P1));
    t.resolve_all();
    assert_eq!(t.pt(m), (6, 6), "X = 3 attackers");
    destroy_now(&mut t, goblin);
    destroy_now(&mut t, b);
    t.settle();
    assert_eq!(t.pt(m), (6, 6), "X was locked in");
    // Terra Ravager: the defending player's lands as it resolves.
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    let r = t.battlefield(P0, "Terra Ravager");
    attack_with(&mut t, &[(r, p(P1))]);
    t.battlefield(P1, "Forest");
    t.resolve_all();
    assert_eq!(t.pt(r), (3, 4));
}

#[test]
fn brazen_blademaster_checks_its_artifacts_only_as_it_attacks() {
    cr!("603.2", "508.1m");
    ruling!(
        "Brazen Blademaster",
        "If you controlled at least two artifacts when you declared Brazen Blademaster as an attacker, it doesn't matter how many you control as its ability resolves. Brazen Blademaster will still get +2/+1 until end of turn."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brazen Blademaster");
    let a1 = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Mind Stone");
    attack_with(&mut t, &[(b, p(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy_now(&mut t, a1);
    t.resolve_all();
    assert_eq!(t.pt(b), (4, 4));
    // With one artifact it doesn't trigger.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brazen Blademaster");
    t.battlefield(P0, "Mind Stone");
    attack_with(&mut t, &[(b, p(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(b), (2, 3));
}

#[test]
fn haazda_marshal_doesnt_recount_attackers() {
    cr!("603.2", "603.4");
    ruling!(
        "Haazda Marshal",
        "Once Haazda Marshal’s ability has triggered, it doesn’t matter how many creatures are still attacking when that ability resolves."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Haazda Marshal");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(m, p(P1)), (a, p(P1)), (b, p(P1))]);
    destroy_now(&mut t, a);
    destroy_now(&mut t, b);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 1);
}

#[test]
fn hero_of_bladehold_tokens_attack_but_were_never_declared() {
    cr!("508.4", "508.3a", "508.4a");
    ruling!(
        "Hero of Bladehold",
        "Although the tokens are attacking, they never were declared as attacking creatures (for purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    ruling!(
        "Hero of Bladehold",
        "You choose which opponent or planeswalker an opponent controls that each token is attacking when it is put onto the battlefield."
    );
    // Brutal Hordechief triggers for the Hero only, not for the tokens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brutal Hordechief");
    let h = t.battlefield(P0, "Hero of Bladehold");
    attack_with(&mut t, &[(h, p(P1))]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 2);
    assert_eq!(t.life(P1), 19, "Hordechief triggered for the tokens");
    // In multiplayer, P0 chooses what each token attacks.
    let mut t = TestGame::new(3);
    let h = t.battlefield(P0, "Hero of Bladehold");
    t.answer_choose(P0, &[p(P2)]);
    t.answer_choose(P0, &[p(P1)]);
    attack_with(&mut t, &[(h, p(P1))]);
    t.resolve_all();
    let attacks = crate::r_s09_common::attacks_now(&t);
    let tokens: Vec<Entity> = t
        .named_on_battlefield("Soldier Token")
        .into_iter()
        .filter_map(|s| attacks.iter().find(|(a, _)| *a == s).map(|(_, x)| *x))
        .collect();
    assert_eq!(tokens.len(), 2);
    assert!(tokens.contains(&p(P2)), "{tokens:?}");
    assert!(tokens.contains(&p(P1)), "{tokens:?}");
}

#[test]
fn crumbling_colossus_cant_be_sacrificed_by_its_former_controller() {
    cr!("701.21a", "603.7");
    ruling!(
        "Crumbling Colossus",
        "If another player controls Crumbling Colossus when its ability tries to resolve, you’ll be unable to sacrifice it. It will simply remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crumbling Colossus");
    attack_with(&mut t, &[(c, p(P1))]);
    t.resolve_all();
    give_control(&mut t, c, P1);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(c), "sacrificed by a player who doesn't control it");
    assert_eq!(t.obj_now(c).controller, P1);
    // Control: P0 still controls it: it's sacrificed.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crumbling Colossus");
    attack_with(&mut t, &[(c, p(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(c));
}

#[test]
fn underworld_sentinel_exiled_cards() {
    cr!("607.2a", "400.7", "800.4a", "508.1");
    ruling!(
        "Underworld Sentinel",
        "If Underworld Sentinel leaves the battlefield without dying, the exiled creature cards are exiled for the rest of the game. If Underworld Sentinel returns to the battlefield later, it’ll be a new object with no memory of the creatures it exiled previously."
    );
    ruling!(
        "Underworld Sentinel",
        "If you gain control of another player’s Underworld Sentinel and it then dies, you’ll put the exiled cards onto the battlefield under your control."
    );
    ruling!(
        "Underworld Sentinel",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any permanents you control from Underworld Sentinel but don’t own are exiled."
    );
    ruling!(
        "Underworld Sentinel",
        "Underworld Sentinel can attack even if there’s no creature card in your graveyard to exile."
    );
    // It can attack with an empty graveyard.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Underworld Sentinel");
    attack_with(&mut t, &[(s, p(P1))]);
    t.resolve_all();
    assert_eq!(crate::r_s09_common::attacks_now(&t).len(), 1);
    // It leaves without dying: the cards stay exiled, and a new Sentinel doesn't remember
    // them.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let gi = t.g.player(P0).graveyard[0];
    let s = t.battlefield(P0, "Underworld Sentinel");
    t.answer_targets(P0, &[obj(gi)]);
    attack_with(&mut t, &[(s, p(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    let ex = t.g.move_object(
        t.g.current(s),
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    t.settle();
    assert!(t.in_exile("Hill Giant"));
    let s2 = t
        .g
        .move_object(
            ex.unwrap(),
            mtg_engine::object::Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            None,
        )
        .unwrap();
    t.g.flush_events();
    destroy_now(&mut t, s2);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.in_exile("Hill Giant"));
    // P1 gains control of P0's Sentinel and it dies: P1 gets the cards; when P1 leaves
    // the game, they (owned by P0) are exiled.
    let mut t = TestGame::new(3);
    t.graveyard(P0, "Hill Giant");
    let gi = t.g.player(P0).graveyard[0];
    let s = t.battlefield(P0, "Underworld Sentinel");
    t.answer_targets(P0, &[obj(gi)]);
    attack_with(&mut t, &[(s, p(P1))]);
    t.resolve_all();
    give_control(&mut t, s, P1);
    destroy_now(&mut t, s);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P1);
    assert_eq!(t.obj_now(g[0]).owner, P0);
    concede(&mut t, P1);
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn augusta_triggers_when_you_attack_without_her() {
    cr!("508.1", "603.2");
    ruling!(
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
        "Augusta’s last ability triggers whenever you attack with any creatures, even if you didn’t attack with Augusta."
    );
    let mut t = TestGame::new(2);
    let aug = crate::r_p079_common::back_face(
        &mut t,
        P0,
        "Plargg, Dean of Chaos // Augusta, Dean of Order",
    );
    assert_eq!(t.obj_now(aug).chars.name, "Augusta, Dean of Order");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(b, p(P1))]);
    assert_eq!(t.stack_len(), 1, "Augusta didn't trigger");
    t.resolve_all();
    // Untapped (and not tapped again: no creature chosen).
    assert!(!t.obj_now(b).tapped);
}
