//! Rulings batch P211 — cards the batch's compiler additions newly compile: "transform it"
//! naming the source (CR 701.27a), "you may play up to N additional lands this turn"
//! (CR 305.2) and "~ can be attached only to a [filter]" (CR 301.5c, 701.3b).

use crate::r_s01_common::*;
use crate::r_s02_common::can_play_land;
use crate::r_s06_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// From `p`'s postcombat main phase, advances to `p`'s end step and resolves what
/// triggered.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::PostcombatMain);
    t.advance_to(p, Step::End);
    t.settle();
    t.resolve_all();
}

#[test]
fn avacynian_missionaries_transforms_at_end_step_only_if_equipped() {
    cr!("701.27a", "603.4");
    supported("Avacynian Missionaries // Lunarch Inquisitors");
    // "At the beginning of your end step, if this creature is equipped, transform it."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Avacynian Missionaries // Lunarch Inquisitors");
    to_end_step(&mut t, P0);
    assert_eq!(name(&t, m), "Avacynian Missionaries");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Avacynian Missionaries // Lunarch Inquisitors");
    attach_new(&mut t, P0, "Bonesplitter", m);
    to_end_step(&mut t, P0);
    assert_eq!(name(&t, m), "Lunarch Inquisitors");
    assert_eq!(t.pt(m), (6, 4));
}

#[test]
fn uninvited_geist_transforms_after_dealing_combat_damage_to_a_player() {
    cr!("701.27a", "510.2");
    supported("Uninvited Geist // Unimpeded Trespasser");
    // "When this creature deals combat damage to a player, transform it."
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Uninvited Geist // Unimpeded Trespasser");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(g, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(name(&t, g), "Unimpeded Trespasser");
    assert_eq!(t.pt(g), (3, 3));
    assert_eq!(name(&t, other), "Grizzly Bears");
    // Blocked: no damage to a player, no transformation.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Uninvited Geist // Unimpeded Trespasser");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.attack(&[(g, Entity::Player(P1))], &[(wall, g)]);
    t.resolve_all();
    assert_eq!(name(&t, g), "Uninvited Geist");
}

#[test]
fn insidious_mist_may_pay_to_transform_when_unblocked() {
    cr!("701.27a", "118.12", "509.1h");
    supported("Elusive Tormentor // Insidious Mist");
    // "Whenever this creature attacks and isn't blocked, you may pay {2}{B}. If you do,
    // transform it."
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Elusive Tormentor // Insidious Mist");
        assert!(mtg_engine::dfc::transform(&mut t.g, e));
        t.g.recompute();
        assert_eq!(name(&t, e), "Insidious Mist");
        t.lands(P0, "Swamp", 3);
        t.answer_yes(P0, pay);
        t.attack(&[(e, Entity::Player(P1))], &[]);
        t.resolve_all();
        let expected = if pay {
            "Elusive Tormentor"
        } else {
            "Insidious Mist"
        };
        assert_eq!(name(&t, e), expected, "pay: {pay}");
        assert_eq!(tapped_lands(&t, P0), if pay { 3 } else { 0 }, "pay: {pay}");
    }
}

#[test]
fn soul_seizer_transforms_and_attaches_to_that_players_creature() {
    cr!("701.27a", "701.3a", "118.12");
    supported("Soul Seizer // Ghastly Haunting");
    // "When this creature deals combat damage to a player, you may transform it. If you
    // do, attach it to target creature that player controls." Ghastly Haunting: "Enchant
    // creature. You control enchanted creature."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Soul Seizer // Ghastly Haunting");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(name(&t, s), "Ghastly Haunting");
    assert_eq!(attached_to(&t, s), Some(Entity::Object(bears)));
    assert_eq!(t.obj_now(bears).controller, P0);
    // Declined: it stays a creature, nothing is attached.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Soul Seizer // Ghastly Haunting");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(name(&t, s), "Soul Seizer");
    assert_eq!(attached_to(&t, s), None);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn foreboding_statue_untaps_and_transforms_with_three_omen_counters() {
    cr!("701.27a", "603.4", "122.1");
    supported("Foreboding Statue // Forsaken Thresher");
    // "At the beginning of your end step, if there are three or more omen counters on
    // this creature, untap it, then transform it."
    for (n, transforms) in [(2, false), (3, true)] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Foreboding Statue // Forsaken Thresher");
        t.g.add_counters(Entity::Object(s), "omen", n, None);
        t.g.tap(s);
        to_end_step(&mut t, P0);
        if transforms {
            assert_eq!(name(&t, s), "Forsaken Thresher");
            assert!(!t.obj_now(s).tapped);
            assert_eq!(t.pt(s), (5, 5));
        } else {
            assert_eq!(name(&t, s), "Foreboding Statue");
            assert!(t.obj_now(s).tapped);
        }
    }
}

#[test]
fn wedding_announcement_transforms_on_its_third_counter() {
    cr!("701.27a", "122.1", "608.2c");
    supported("Wedding Announcement // Wedding Festivity");
    // "At the beginning of your end step, put an invitation counter on this enchantment.
    // If you attacked with two or more creatures this turn, draw a card. Otherwise,
    // create a 1/1 white Human creature token. Then if this enchantment has three or more
    // invitation counters on it, transform it."
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wedding Announcement // Wedding Festivity");
    t.g.add_counters(Entity::Object(w), "invitation", 1, None);
    to_end_step(&mut t, P0);
    assert_eq!(name(&t, w), "Wedding Announcement");
    assert_eq!(t.counters(w, "invitation"), 2);
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wedding Announcement // Wedding Festivity");
    t.g.add_counters(Entity::Object(w), "invitation", 2, None);
    to_end_step(&mut t, P0);
    assert_eq!(name(&t, w), "Wedding Festivity");
    // The Human was created first, and gets +1/+1 from Wedding Festivity.
    let humans = with_subtype(&t, P0, "Human");
    assert_eq!(humans.len(), 1);
    assert_eq!(t.pt(humans[0]), (2, 2));
}

#[test]
fn cecil_untaps_and_transforms_at_half_starting_life_or_less() {
    cr!("701.27a", "119.1");
    supported("Cecil, Dark Knight // Cecil, Redeemed Paladin");
    // "Darkness — Whenever Cecil deals damage, you lose that much life. Then if your life
    // total is less than or equal to half your starting life total, untap Cecil and
    // transform it." (Cecil is 2/3.)
    for (life, transforms) in [(13, false), (12, true)] {
        let mut t = TestGame::new(2);
        t.g.players[0].life = life;
        let c = t.battlefield(P0, "Cecil, Dark Knight // Cecil, Redeemed Paladin");
        t.attack(&[(c, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.life(P0), life - 2);
        if transforms {
            assert_eq!(name(&t, c), "Cecil, Redeemed Paladin");
            assert!(!t.obj_now(c).tapped);
        } else {
            assert_eq!(name(&t, c), "Cecil, Dark Knight");
            assert!(t.obj_now(c).tapped);
        }
    }
}

#[test]
fn vincent_valentine_may_transform_as_it_attacks() {
    cr!("701.27a", "508.1m");
    supported("Vincent Valentine // Galian Beast");
    // "Whenever Vincent Valentine attacks, you may transform it." Galian Beast: 3/2
    // trample, lifelink.
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let v = t.battlefield(P0, "Vincent Valentine // Galian Beast");
        t.answer_yes(P0, yes);
        t.attack(&[(v, Entity::Player(P1))], &[]);
        t.resolve_all();
        if yes {
            assert_eq!(name(&t, v), "Galian Beast");
            assert_eq!(t.life(P1), 17);
            assert_eq!(t.life(P0), 23);
        } else {
            assert_eq!(name(&t, v), "Vincent Valentine");
            assert_eq!(t.life(P1), 18);
            assert_eq!(t.life(P0), 20);
        }
    }
}

#[test]
fn summer_bloom_allows_three_additional_land_plays() {
    cr!("305.2", "305.2a");
    supported("Summer Bloom");
    // "You may play up to three additional lands this turn."
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Summer Bloom");
    t.lands(P0, "Forest", 2);
    t.cast(P0, c).go();
    t.resolve_all();
    for _ in 0..4 {
        let land = t.hand(P0, "Forest");
        assert!(can_play_land(&mut t, P0, land));
        t.play_land(P0, land).unwrap();
    }
    let land = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, land));
}

#[test]
fn o_naginata_attaches_only_to_power_3_or_greater() {
    cr!("301.5c", "701.3b");
    supported("O-Naginata");
    // "This Equipment can be attached only to a creature with power 3 or greater.
    // Equipped creature gets +3/+0 and has trample."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let o = t.battlefield(P0, "O-Naginata");
    assert!(!t.g.attach(o, Entity::Object(bears)));
    assert!(t.g.attach(o, Entity::Object(giant)));
    t.g.recompute();
    assert_eq!(t.pt(giant), (6, 3));
    // Its power includes O-Naginata's own bonus: with a -1/-1 counter it's 5/2 and
    // stays equipped.
    t.g.add_counters(Entity::Object(giant), counters::MINUS1, 1, None);
    t.settle();
    assert_eq!(attached_to(&t, o), Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (5, 2));
    assert_eq!(t.zone(o), Zone::Battlefield);
}
