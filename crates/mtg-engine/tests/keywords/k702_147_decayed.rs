//! CR 702.147 Decayed.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::k702_001_010_common::{grant, remove_kw};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Rotten Reunion ("Exile up to one target card from a graveyard. Create a 2/2 black
/// Zombie creature token with decayed.") for P0 and returns the Zombie token.
fn zombie_with_decayed(t: &mut TestGame) -> ObjectId {
    let spell = t.hand(P0, "Rotten Reunion");
    add_mana(t, P0, ManaType::B, 1);
    add_mana(t, P0, ManaType::C, 1);
    t.cast(P0, spell).targets(&[]).go();
    t.resolve_all();
    let tokens = creature_tokens(t, P0);
    assert_eq!(tokens.len(), 1);
    tokens[0]
}

#[test]
fn a_creature_with_decayed_cant_block() {
    cr!("702.147", "702.147a");
    assert_supported("Rotten Reunion");
    let mut t = TestGame::new(2);
    let zombie = zombie_with_decayed(&mut t);
    assert!(has_kw(&t, zombie, KeywordKind::Decayed));
    let bear = t.battlefield(P0, "Grizzly Bears");
    // It's P1's turn: P1's creature attacks; the Zombie can't block it, the Bears can.
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(attacker, Entity::Player(P0))]);
    assert!(!can_block_now(&mut t, zombie, attacker));
    assert!(can_block_now(&mut t, bear, attacker));
}

#[test]
fn when_it_attacks_its_sacrificed_at_end_of_combat() {
    cr!("702.147a");
    ruling!(
        "Curse of the Restless Dead",
        "Decayed does not grant haste. Creatures with decayed that enter the battlefield during your turn may not attack until your next turn."
    );
    let mut t = TestGame::new(2);
    let zombie = zombie_with_decayed(&mut t);
    // No haste: it can't attack the turn it's created.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack_now(&mut t, zombie));
    t.g.objects[zombie.0 as usize].summoning_sick = false;
    assert!(can_attack_now(&mut t, zombie));
    t.attack(&[(zombie, Entity::Player(P1))], &[]);
    // It dealt its combat damage, then it's sacrificed at end of combat.
    assert_eq!(t.life(P1), 18);
    t.resolve_all();
    assert!(!t.g.is_live(zombie));
    assert!(creature_tokens(&t, P0).is_empty());
}

#[test]
fn it_doesnt_have_to_attack_and_isnt_sacrificed_if_it_doesnt() {
    cr!("702.147a");
    ruling!(
        "Curse of the Restless Dead",
        "Decayed does not create any attacking requirements. You may choose not to attack with a creature that has decayed."
    );
    let mut t = TestGame::new(2);
    let rakshasa = t.battlefield(P0, "Rot-Curse Rakshasa");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[], &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(rakshasa));
}

#[test]
fn it_is_sacrificed_even_if_it_lost_decayed() {
    cr!("702.147a");
    ruling!(
        "Curse of the Restless Dead",
        "Once a creature with decayed attacks, it will be sacrificed at end of combat, even if it no longer has decayed at that time."
    );
    let mut t = TestGame::new(2);
    // Rot-Curse Rakshasa: 5/5 trample, decayed.
    let rakshasa = t.battlefield(P0, "Rot-Curse Rakshasa");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(rakshasa, Entity::Player(P1))]);
    t.resolve_all();
    remove_kw(&mut t, rakshasa, KeywordKind::Decayed);
    assert!(!has_kw(&t, rakshasa, KeywordKind::Decayed));
    to_step(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P0, "Rot-Curse Rakshasa"));
}

#[test]
fn a_creature_that_gains_decayed_cant_block_and_is_sacrificed_after_attacking() {
    cr!("702.147a");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    grant(&mut t, bear, Keyword::new(KeywordKind::Decayed));
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(attacker, Entity::Player(P0))]);
    assert!(!can_block_now(&mut t, bear, attacker));
    t.advance_to(P0, Step::BeginningOfCombat);
    grant(&mut t, bear, Keyword::new(KeywordKind::Decayed));
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn it_isnt_sacrificed_if_another_player_controls_it_at_end_of_combat() {
    cr!("702.147a", "701.21a");
    let mut t = TestGame::new(2);
    let rakshasa = t.battlefield(P0, "Rot-Curse Rakshasa");
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[(rakshasa, Entity::Player(P1))]);
    t.resolve_all();
    // P1 gains control of it before end of combat: P0 can't sacrifice it.
    let ctl = mtg_engine::ability::Effect::GainControl {
        what: mtg_engine::ability::Sel::Target(0),
        who: mtg_engine::ability::PlayerRef::You,
        duration: mtg_engine::ability::Duration::Permanent,
    };
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(rakshasa)]];
    t.g.exec(&ctl, &mut ctx);
    t.g.recompute();
    to_step(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(rakshasa));
    assert_eq!(t.obj_now(rakshasa).controller, P1);
}
