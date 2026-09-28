//! Rulings batch S12 — myriad (CR 702.116): "Whenever this creature attacks, for each
//! opponent other than defending player, you may create a token that's a copy of this
//! creature that's tapped and attacking that player or a planeswalker they control. If one
//! or more tokens are created this way, exile the tokens at end of combat."

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s05_common::colors;
use crate::r_s06_common::attach_new;
use crate::r_s12_common::*;
use mtg_engine::battle;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Tokens `p` controls that are copies named `name`.
fn copies(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.obj(*id).chars.name.as_str() == name)
        .collect()
}

/// The players (or planeswalkers) the tokens `ids` are attacking, sorted.
fn targets_of(t: &TestGame, ids: &[ObjectId]) -> Vec<Entity> {
    let mut v: Vec<Entity> = ids.iter().filter_map(|id| attack_target(t, *id)).collect();
    v.sort();
    v
}

/// `p`'s Invasion of Segovia, protected by `protector`.
fn siege(t: &mut TestGame, p: PlayerId, protector: PlayerId) -> ObjectId {
    t.answer_choose(p, &[Entity::Player(protector)]);
    let id = t.enter(p, "Invasion of Segovia");
    assert_eq!(battle::protector(&t.g, id), Some(protector));
    id
}

#[test]
fn myriad_tokens_all_enter_at_the_same_time() {
    cr!("702.116a", "603.6a", "603.2c");
    ruling!(
        "Blade of Selves",
        "The token creatures all enter the battlefield at the same time."
    );
    ruling!(
        "Scion of Calamity",
        "The tokens all enter the battlefield at the same time."
    );
    supported("Blade of Selves");
    supported("Soul Warden");
    supported("Cloakwood Swarmkeeper");
    // Blade of Selves on Soul Warden ("Whenever another creature enters, you gain 1
    // life."), four players: two token copies enter at once, and each copy sees the other
    // enter (CR 603.6a) as well as the original seeing both.
    let mut t = TestGame::new(4);
    let warden = t.battlefield(P0, "Soul Warden");
    attach_new(&mut t, P0, "Blade of Selves", warden);
    attack_with(&mut t, &[(warden, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(copies(&t, P0, "Soul Warden").len(), 2);
    assert_eq!(t.life(P0), 24);
    // Cloakwood Swarmkeeper ("Whenever one or more tokens you control enter, put a +1/+1
    // counter on this creature.") triggers once for Scion of Calamity's two tokens.
    let mut t = TestGame::new(4);
    let keeper = t.battlefield(P0, "Cloakwood Swarmkeeper");
    let scion = t.battlefield(P0, "Scion of Calamity");
    attack_with(&mut t, &[(scion, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(copies(&t, P0, "Scion of Calamity").len(), 2);
    assert_eq!(t.counters(keeper, counters::PLUS1), 1);
}

#[test]
fn each_token_copies_only_what_is_printed_on_the_original() {
    cr!("702.116a", "707.2");
    ruling!(
        "Blade of Selves",
        "Each token copies exactly what was printed on the original creature and nothing else. It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Serra Angel");
    // Blade of Selves on Serra Angel (4/4 flying, vigilance) with two +1/+1 counters and
    // Rancor, made blue by Cerulean Wisps.
    let mut t = TestGame::new(3);
    let angel = t.battlefield(P0, "Serra Angel");
    let blade = attach_new(&mut t, P0, "Blade of Selves", angel);
    let rancor = attach_new(&mut t, P0, "Rancor", angel);
    t.g.add_counters(Entity::Object(angel), counters::PLUS1, 2, None);
    let wisps = in_hand_with_mana(&mut t, P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(angel).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (8, 6));
    assert_eq!(colors(&t, angel), ColorSet::single(Color::Blue));
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    t.resolve_all();
    // Vigilance: the Angel attacks untapped; the token is tapped (as myriad says).
    assert!(!t.obj(angel).tapped);
    let tok = copies(&t, P0, "Serra Angel");
    assert_eq!(tok.len(), 1);
    let tok = tok[0];
    assert!(t.obj(tok).tapped);
    assert_eq!(t.pt(tok), (4, 4));
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(colors(&t, tok), ColorSet::single(Color::White));
    // Nothing is attached to it: Rancor and the Blade stay on the Angel, so the token
    // has no trample and no myriad.
    assert_eq!(t.obj(rancor).attached_to, Some(Entity::Object(angel)));
    assert_eq!(t.obj(blade).attached_to, Some(Entity::Object(angel)));
    assert!(!t.obj(tok).has_keyword(KeywordKind::Trample));
    assert!(!t.obj(tok).has_keyword(KeywordKind::Myriad));
    assert!(t.obj(tok).has_keyword(KeywordKind::Flying));
}

#[test]
fn doubled_myriad_tokens_each_choose_what_they_attack() {
    cr!("702.116a", "508.4", "614.1a");
    ruling!(
        "Warchief Giant",
        "If myriad creates more than one token for any given player (due to an effect such as the one Doubling Season creates), you may choose separately for each token whether it’s attacking the player or a planeswalker they control."
    );
    ruling!(
        "Blade of Selves",
        "If myriad creates more than one token for any given player (due to an effect such as the one Doubling Season creates), you may choose separately for each token whether it's attacking the player or a planeswalker they control."
    );
    supported("Parallel Lives");
    // Parallel Lives: "If an effect would create one or more tokens under your control,
    // it creates twice that many of those tokens instead."
    for (name, blade) in [("Warchief Giant", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Parallel Lives");
        let jace = t.battlefield(P2, "Jace Beleren");
        let attacker = t.battlefield(P0, name);
        if blade {
            attach_new(&mut t, P0, "Blade of Selves", attacker);
        }
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        let from = t.asked().len();
        t.answer_choose(P0, &[Entity::Object(jace)]);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.resolve_all();
        let tok = copies(&t, P0, name);
        assert_eq!(tok.len(), 2, "{name}");
        let mut want = vec![Entity::Player(P2), Entity::Object(jace)];
        want.sort();
        assert_eq!(targets_of(&t, &tok), want, "{name}");
        // Asked once for each token.
        let asked = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| {
                matches!(d, mtg_engine::decision::Decision::ChooseEntities { prompt, .. }
                    if prompt.contains("token is attacking"))
            })
            .count();
        assert_eq!(asked, 2, "{name}");
    }
}

#[test]
fn myriad_tokens_werent_declared_as_attackers_and_pay_no_attack_costs() {
    cr!("702.116a", "508.4", "508.4c", "508.1g");
    ruling!(
        "Wizards of Thay",
        "Although the tokens enter the battlefield attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won’t trigger, including the myriad ability of the tokens. If there are any costs to have a creature attack, those costs won’t apply to the tokens."
    );
    ruling!(
        "Scion of Calamity",
        "Although the tokens enter the battlefield attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won't trigger, including the myriad ability of the tokens. If there are any costs to have a creature attack, those costs won't apply to the tokens."
    );
    supported("Wizards of Thay");
    supported("Gleam of Battle");
    supported("Ghostly Prison");
    for name in ["Wizards of Thay", "Scion of Calamity"] {
        // Gleam of Battle: "Whenever a creature you control attacks, put a +1/+1 counter
        // on it." P2's Ghostly Prison: "Creatures can't attack you unless their controller
        // pays {2} for each creature they control that's attacking you." P0 has no mana.
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Gleam of Battle");
        t.battlefield(P2, "Ghostly Prison");
        let orig = t.battlefield(P0, name);
        attack_with(&mut t, &[(orig, Entity::Player(P1))]);
        t.resolve_all();
        let tok = copies(&t, P0, name);
        assert_eq!(tok.len(), 1, "{name}");
        // The token attacks P2 without paying for Ghostly Prison.
        assert_eq!(attack_target(&t, tok[0]), Some(Entity::Player(P2)));
        // Gleam of Battle triggered only for the declared attacker; the token's own
        // myriad didn't trigger.
        assert_eq!(t.counters(orig, counters::PLUS1), 1, "{name}");
        assert_eq!(t.counters(tok[0], counters::PLUS1), 0, "{name}");
        assert_eq!(t.stack_len(), 0);
        assert_eq!(tokens(&t, P0).len(), 1, "{name}");
    }
}

#[test]
fn enters_abilities_of_the_copied_creature_work_for_the_tokens() {
    cr!("702.116a", "707.2", "614.1c", "603.6a");
    ruling!(
        "Duke Ulder Ravengard",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the tokens enter the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied creature will also work."
    );
    ruling!(
        "Tiamat's Fanatics",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the tokens enter the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the copied creature will also work."
    );
    supported("Duke Ulder Ravengard");
    supported("Elvish Visionary");
    supported("Servant of the Scale");
    // Four players. Duke Ulder Ravengard gives Elvish Visionary ("When this creature
    // enters, draw a card.") haste and myriad at the beginning of combat; Blade of Selves
    // gives Servant of the Scale ("This creature enters with a +1/+1 counter on it.")
    // myriad.
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Duke Ulder Ravengard");
    let elf = t.battlefield_sick(P0, "Elvish Visionary");
    // A 0/0 that entered with its +1/+1 counter.
    let servant = t.battlefield(P0, "Servant of the Scale");
    t.g.add_counters(Entity::Object(servant), counters::PLUS1, 1, None);
    attach_new(&mut t, P0, "Blade of Selves", servant);
    t.answer_targets(P0, &[Entity::Object(elf)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj(elf).has_keyword(KeywordKind::Myriad));
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &[(elf, Entity::Player(P1)), (servant, Entity::Player(P1))],
    );
    t.resolve_all();
    let elves = copies(&t, P0, "Elvish Visionary");
    let servants = copies(&t, P0, "Servant of the Scale");
    assert_eq!((elves.len(), servants.len()), (2, 2));
    // Each Visionary token's enters trigger drew a card.
    assert_eq!(t.hand_size(P0), hand + 2);
    // Each Servant token entered with a +1/+1 counter.
    for s in servants {
        assert_eq!(t.counters(s, counters::PLUS1), 1);
    }
}

#[test]
fn you_choose_the_player_or_which_planeswalker_each_token_attacks() {
    cr!("702.116a", "508.4");
    ruling!(
        "Wizards of Thay",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it’s attacking a planeswalker, you choose which one."
    );
    ruling!(
        "Wyrm's Crossing Patrol",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it's attacking a planeswalker, you choose which one."
    );
    supported("Wyrm's Crossing Patrol");
    supported("Liliana of the Veil");
    for name in ["Wizards of Thay", "Wyrm's Crossing Patrol"] {
        // Four players: P2 controls two planeswalkers, P3 one.
        let mut t = TestGame::new(4);
        t.battlefield(P2, "Jace Beleren");
        let lili = t.battlefield(P2, "Liliana of the Veil");
        let garruk = t.battlefield(P3, "Garruk Wildspeaker");
        let orig = t.battlefield(P0, name);
        attack_with(&mut t, &[(orig, Entity::Player(P1))]);
        let from = t.asked().len();
        // For P2: Liliana (not Jace or P2). For P3: P3 itself.
        t.answer_choose(P0, &[Entity::Object(lili)]);
        t.answer_choose(P0, &[Entity::Player(P3)]);
        t.resolve_all();
        let tok = copies(&t, P0, name);
        let mut want = vec![Entity::Player(P3), Entity::Object(lili)];
        want.sort();
        assert_eq!(targets_of(&t, &tok), want, "{name}");
        // The choice for P3's token was between P3 and Garruk only.
        let offered = crate::r_s03_common::choice_candidates(&t, from, "token is attacking");
        assert_eq!(offered.len(), 2, "{name}");
        assert_eq!(offered[1], vec![Entity::Player(P3), Entity::Object(garruk)]);
        assert_eq!(offered[0].len(), 3, "{name}");
    }
}

#[test]
fn the_defending_player_is_the_one_it_attacks_or_last_attacked() {
    cr!("702.116a", "508.5");
    ruling!(
        "Wizards of Thay",
        "The term “defending player” in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking or the controller of the planeswalker it was attacking at the time the ability resolves. If that creature is no longer attacking, it refers to the player it was last attacking or the controller of the planeswalker it was last attacking."
    );
    ruling!(
        "Tiamat's Fanatics",
        "The term \"defending player\" in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking or the controller of the planeswalker it was attacking at the time the ability resolves. If that creature is no longer attacking, it refers to the player it was last attacking or the controller of the planeswalker it was last attacking."
    );
    supported("Tiamat's Fanatics");
    supported("Reconnaissance");
    for name in ["Wizards of Thay", "Tiamat's Fanatics"] {
        // Attacking P2's planeswalker: P2 is the defending player.
        let mut t = TestGame::new(4);
        let jace = t.battlefield(P2, "Jace Beleren");
        let orig = t.battlefield(P0, name);
        attack_with(&mut t, &[(orig, Entity::Object(jace))]);
        t.resolve_all();
        let tok = copies(&t, P0, name);
        assert_eq!(
            targets_of(&t, &tok),
            vec![Entity::Player(P1), Entity::Player(P3)],
            "{name}"
        );
        // Removed from combat (Reconnaissance) before the ability resolves: the player it
        // was last attacking, P1, is still the defending player.
        let mut t = TestGame::new(4);
        let recon = t.battlefield(P0, "Reconnaissance");
        let orig = t.battlefield(P0, name);
        attack_with(&mut t, &[(orig, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        t.activate(P0, recon, 0, &[Entity::Object(orig)]).unwrap();
        t.resolve();
        assert!(!t.g.is_attacking(orig));
        t.resolve_all();
        let tok = copies(&t, P0, name);
        assert_eq!(
            targets_of(&t, &tok),
            vec![Entity::Player(P2), Entity::Player(P3)],
            "{name}"
        );
    }
}

#[test]
fn the_defending_player_is_set_as_it_became_an_attacking_creature() {
    cr!("702.116a", "508.5", "310.5");
    ruling!(
        "Herald of the Host",
        "The term “defending player” in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking at the time it became an attacking creature this combat, or the controller of the planeswalker the creature was attacking at the time it became an attacking creature this combat."
    );
    ruling!(
        "Chittering Dispatcher",
        "or the protector of the battle this creature was attacking at the time it became an attacking creature this combat."
    );
    ruling!(
        "Dalek Squadron",
        "or the controller of the planeswalker or the protector of the battle the creature was attacking at the time it became an attacking creature this combat."
    );
    supported("Herald of the Host");
    supported("Chittering Dispatcher");
    supported("Dalek Squadron");
    // Herald of the Host attacks P2's planeswalker: tokens for P1 and P3.
    let mut t = TestGame::new(4);
    let jace = t.battlefield(P2, "Jace Beleren");
    let herald = t.battlefield(P0, "Herald of the Host");
    attack_with(&mut t, &[(herald, Entity::Object(jace))]);
    t.resolve_all();
    let tok = copies(&t, P0, "Herald of the Host");
    assert_eq!(
        targets_of(&t, &tok),
        vec![Entity::Player(P1), Entity::Player(P3)]
    );
    // Chittering Dispatcher and Dalek Squadron attack P1's Invasion of Segovia, which P2
    // protects: P2 is their defending player, so their tokens attack P1 and P3.
    for name in ["Chittering Dispatcher", "Dalek Squadron"] {
        let mut t = TestGame::new(4);
        let b = siege(&mut t, P1, P2);
        let orig = t.battlefield(P0, name);
        attack_with(&mut t, &[(orig, Entity::Object(b))]);
        t.resolve_all();
        let tok = copies(&t, P0, name);
        assert_eq!(
            targets_of(&t, &tok),
            vec![Entity::Player(P1), Entity::Player(P3)],
            "{name}"
        );
    }
}
