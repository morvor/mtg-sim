//! State-based actions (CR 704) and the priority-time "settle" loop (CR 117.5, 704.3).

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::*;
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

impl Game {
    /// Performs state-based actions and puts triggered abilities on the stack until
    /// neither happens (CR 704.3, 117.5). Returns true if anything happened.
    pub fn settle(&mut self) -> bool {
        let mut did = false;
        for _ in 0..1000 {
            self.flush_events();
            if self.result.is_some() {
                return did;
            }
            if self.check_sbas() {
                did = true;
                continue;
            }
            self.check_state_triggers();
            if !self.pending_triggers.is_empty() {
                self.put_triggers_on_stack();
                did = true;
                continue;
            }
            break;
        }
        did
    }

    /// Checks and performs all applicable state-based actions simultaneously (CR 704.3).
    /// Returns true if any were performed.
    pub fn check_sbas(&mut self) -> bool {
        self.sba_checks += 1;
        if self.dirty {
            self.recompute();
        }
        let mut performed = false;

        // --- Player losses (704.5a–c, 704.6a–c) -----------------------------
        let two_headed = self.config.variant == Variant::TwoHeadedGiant;
        let mut losers: BTreeSet<PlayerId> = BTreeSet::new();
        for p in self.players_in_game() {
            let pl = self.player(p);
            if !two_headed && pl.life <= 0 {
                losers.insert(p); // 704.5a
            }
            if pl.drew_from_empty_library {
                losers.insert(p); // 704.5b
            }
            if !two_headed && pl.poison() >= 10 {
                losers.insert(p); // 704.5c
            }
            if self.config.variant == Variant::Commander
                && pl
                    .commander_damage
                    .values()
                    .any(|d| *d >= self.config.commander_damage_limit)
            {
                losers.insert(p); // 704.6c
            }
        }
        if two_headed {
            // 704.6a/b: team life and poison.
            let teams: BTreeSet<u8> = self
                .players_in_game()
                .iter()
                .map(|p| self.player(*p).team)
                .collect();
            for t in teams {
                let members: Vec<PlayerId> = self
                    .players_in_game()
                    .into_iter()
                    .filter(|p| self.player(*p).team == t)
                    .collect();
                let life = members.first().map(|p| self.player(*p).life).unwrap_or(1);
                let poison = members
                    .first()
                    .map(|p| self.player(*p).poison())
                    .unwrap_or(0);
                if life <= 0 || poison >= 15 {
                    losers.extend(members);
                }
            }
        }
        for p in self.players.iter_mut() {
            p.drew_from_empty_library = false;
        }

        // --- Objects ------------------------------------------------------------
        let mut to_graveyard: Vec<ObjectId> = Vec::new(); // put into graveyard (not destroyed)
        let mut to_destroy: Vec<ObjectId> = Vec::new(); // destroyed (regeneration applies)
        let mut cease: Vec<ObjectId> = Vec::new();
        let mut unattach: Vec<ObjectId> = Vec::new();
        let mut counter_removals: Vec<(ObjectId, CounterKind, u32)> = Vec::new();
        let mut sacrifice: Vec<ObjectId> = Vec::new();

        // 704.5d: tokens outside the battlefield cease to exist.
        // 704.5e: copies of spells off the stack / copies of cards off stack & battlefield.
        for (i, o) in self.objects.iter().enumerate() {
            if o.next.is_some() || o.zone == Zone::Nowhere {
                continue;
            }
            let id = ObjectId(i as u32);
            match o.kind {
                ObjKind::Token if o.zone != Zone::Battlefield => cease.push(id),
                ObjKind::SpellCopy if o.zone != Zone::Stack => cease.push(id),
                ObjKind::CardCopy if !matches!(o.zone, Zone::Stack | Zone::Battlefield) => {
                    cease.push(id)
                }
                _ => {}
            }
        }

        let perms: Vec<ObjectId> = self.permanent_ids();
        for &id in &perms {
            let o = self.obj(id);
            if o.is_creature() {
                let t = o.toughness();
                if t <= 0 {
                    to_graveyard.push(id); // 704.5f
                } else if o.damage > 0 && o.damage as i32 >= t {
                    to_destroy.push(id); // 704.5g
                } else if o.deathtouch_damage {
                    to_destroy.push(id); // 704.5h
                }
            }
            if o.is(CardType::Planeswalker) && o.loyalty() <= 0 {
                to_graveyard.push(id); // 704.5i
            }
        }

        // "since the last time state-based actions were checked" (704.5h)
        for &id in &perms {
            self.objects[id.0 as usize].deathtouch_damage = false;
        }
        // Indestructible permanents can't be destroyed (702.12b); lethal damage stays marked.
        to_destroy.retain(|id| !self.obj(*id).has_keyword(KeywordKind::Indestructible));

        // 704.5j: legend rule.
        let mut legends: BTreeMap<(PlayerId, String), Vec<ObjectId>> = BTreeMap::new();
        for &id in &perms {
            let o = self.obj(id);
            if o.chars.is_legendary() && !o.chars.name.is_empty() {
                legends
                    .entry((o.controller, o.chars.name.to_string()))
                    .or_default()
                    .push(id);
            }
        }
        for ((p, name), ids) in legends {
            if ids.len() > 1 {
                let keep = self.ask_objects(
                    p,
                    None,
                    &format!("Legend rule: choose the {name} to keep"),
                    ids.clone(),
                    1,
                    1,
                );
                let keep = keep.first().copied().unwrap_or(ids[0]);
                for id in ids {
                    if id != keep {
                        to_graveyard.push(id);
                    }
                }
            }
        }

        // 704.5k: world rule.
        let worlds: Vec<ObjectId> = perms
            .iter()
            .copied()
            .filter(|id| self.obj(*id).chars.has_supertype(Supertype::World))
            .collect();
        if worlds.len() > 1 {
            let newest = worlds
                .iter()
                .map(|id| self.obj(*id).world_since.unwrap_or(0))
                .max()
                .unwrap_or(0);
            let newest_ids: Vec<ObjectId> = worlds
                .iter()
                .copied()
                .filter(|id| self.obj(*id).world_since.unwrap_or(0) == newest)
                .collect();
            for id in &worlds {
                if newest_ids.len() > 1 || !newest_ids.contains(id) {
                    to_graveyard.push(*id);
                }
            }
        }

        // 704.5m/n/p: attachments.
        for &id in &perms {
            let o = self.obj(id);
            let is_aura = o.chars.has_subtype("Aura");
            let is_equipment = o.chars.has_subtype("Equipment");
            let is_fortification = o.chars.has_subtype("Fortification");
            if is_aura && o.chars.is(CardType::Enchantment) && !o.is_creature() {
                match o.attached_to {
                    None => to_graveyard.push(id),
                    Some(t) => {
                        if !crate::attach::legal_attachment(self, id, t) {
                            to_graveyard.push(id);
                        }
                    }
                }
            } else if (is_equipment || is_fortification)
                && !o.is_creature()
                && !o.is(CardType::Battle)
            {
                if let Some(t) = o.attached_to {
                    if !crate::attach::legal_attachment(self, id, t) {
                        unattach.push(id);
                    }
                }
            } else if o.attached_to.is_some() {
                // 704.5p: creatures/battles, and other permanents that aren't
                // Auras/Equipment/Fortifications, become unattached.
                if o.is_creature()
                    || o.is(CardType::Battle)
                    || !(is_aura || is_equipment || is_fortification)
                {
                    unattach.push(id);
                }
            }
        }

        // 704.5q: +1/+1 and -1/-1 counters annihilate.
        for &id in &perms {
            let o = self.obj(id);
            let plus = o.counter(counters::PLUS1);
            let minus = o.counter(counters::MINUS1);
            let n = plus.min(minus);
            if n > 0 {
                counter_removals.push((id, counters::PLUS1.into(), n));
                counter_removals.push((id, counters::MINUS1.into(), n));
            }
        }

        // 704.5r: "can't have more than N counters".
        for &id in &perms {
            for (kind, max) in crate::custom::counter_limits(self, id) {
                let have = self.obj(id).counter(&kind);
                if have > max {
                    counter_removals.push((id, kind, have - max));
                }
            }
        }

        // 704.5s: Sagas.
        for &id in &perms {
            let o = self.obj(id);
            if o.chars.has_subtype("Saga") {
                if let Some(final_ch) = crate::saga::final_chapter(o) {
                    if o.counter(counters::LORE) >= final_ch && !self.is_source_of_stack_trigger(id)
                    {
                        sacrifice.push(id);
                    }
                }
            }
        }

        // 704.5t: dungeons.
        let dungeon_done = crate::variants::dungeon_sba(self);
        performed |= dungeon_done;

        // 704.5v/w: battles with defense 0.
        for &id in &perms {
            let o = self.obj(id);
            if o.is(CardType::Battle) && o.defense() <= 0 {
                if o.chars.has_subtype("Siege") {
                    if !self.is_source_of_stack_trigger(id) {
                        to_graveyard.push(id);
                    }
                } else {
                    to_graveyard.push(id);
                }
            }
        }
        // 704.5x/y: battle protectors.
        performed |= crate::battle::protector_sba(self, &mut to_graveyard);

        // 704.5z: roles.
        let mut roles: BTreeMap<(ObjectId, PlayerId), Vec<ObjectId>> = BTreeMap::new();
        for &id in &perms {
            let o = self.obj(id);
            if o.chars.has_subtype("Role") {
                if let Some(Entity::Object(t)) = o.attached_to {
                    roles.entry((t, o.controller)).or_default().push(id);
                }
            }
        }
        for (_, mut ids) in roles {
            if ids.len() > 1 {
                ids.sort_by_key(|id| self.obj(*id).timestamp);
                ids.pop();
                to_graveyard.extend(ids);
            }
        }

        // 704.5aa: start your engines!
        for &id in &perms {
            let o = self.obj(id);
            if o.has_keyword(KeywordKind::StartYourEngines) {
                let p = o.controller;
                if self.player(p).speed.is_none() {
                    self.players[p.idx()].speed = Some(1);
                    performed = true;
                }
            }
        }

        // 704.6d: commanders in graveyard/exile may go to the command zone.
        let mut commander_moves: Vec<ObjectId> = Vec::new();
        if self.config.variant == Variant::Commander {
            for (i, o) in self.objects.iter().enumerate() {
                if o.is_commander
                    && o.next.is_none()
                    && matches!(o.zone, Zone::Graveyard(_) | Zone::Exile)
                    && self
                        .commander_moved_since_last_sba
                        .contains(&ObjectId(i as u32))
                {
                    commander_moves.push(ObjectId(i as u32));
                }
            }
        }
        self.commander_moved_since_last_sba.clear();

        // 704.6e/f: archenemy and planechase.
        performed |= crate::variants::variant_sbas(self);

        // --- Perform everything simultaneously ---------------------------------
        to_graveyard.sort();
        to_graveyard.dedup();
        to_destroy.retain(|id| !to_graveyard.contains(id));
        to_destroy.sort();
        to_destroy.dedup();

        let any = !losers.is_empty()
            || !to_graveyard.is_empty()
            || !to_destroy.is_empty()
            || !cease.is_empty()
            || !unattach.is_empty()
            || !counter_removals.is_empty()
            || !sacrifice.is_empty()
            || !commander_moves.is_empty();
        if !any {
            return performed;
        }

        for id in cease {
            let zone = self.obj(id).zone;
            if let Some(list) = self.zone_list_mut(zone) {
                list.retain(|x| *x != id);
            }
            self.objects[id.0 as usize].zone = Zone::Nowhere;
            self.dirty = true;
        }
        for id in unattach {
            self.unattach(id);
        }
        for (id, k, n) in counter_removals {
            self.remove_counters(Entity::Object(id), &k, n);
        }

        let mut moves: Vec<MoveEv> = Vec::new();
        for id in &to_graveyard {
            let owner = self.obj(*id).owner;
            moves.push(MoveEv {
                obj: *id,
                to: Zone::Graveyard(owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::StateBased,
                by: None,
                etb: EtbInfo::default(),
                source: None,
            });
        }
        for id in &to_destroy {
            for e in self.replace(ReplEvent::Destroy {
                obj: *id,
                source: None,
            }) {
                match e {
                    ReplEvent::Destroy { obj, .. } => {
                        let owner = self.obj(obj).owner;
                        moves.push(MoveEv {
                            obj,
                            to: Zone::Graveyard(owner),
                            pos: LibraryPosition::Top,
                            cause: MoveCause::Destroy,
                            by: None,
                            etb: EtbInfo::default(),
                            source: None,
                        });
                    }
                    other => self.execute_repl_event(other),
                }
            }
        }
        for id in &sacrifice {
            let owner = self.obj(*id).owner;
            let ctl = self.obj(*id).controller;
            moves.push(MoveEv {
                obj: *id,
                to: Zone::Graveyard(owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Sacrifice,
                by: Some(ctl),
                etb: EtbInfo::default(),
                source: None,
            });
        }
        for id in commander_moves {
            let owner = self.obj(id).owner;
            if self.ask_yes_no(
                owner,
                Some(id),
                "Move your commander to the command zone?",
                true,
            ) {
                moves.push(MoveEv {
                    obj: id,
                    to: Zone::Command,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Commander,
                    by: Some(owner),
                    etb: EtbInfo::default(),
                    source: None,
                });
            }
        }
        if !moves.is_empty() {
            let sac_ids: Vec<(ObjectId, PlayerId)> = sacrifice
                .iter()
                .map(|id| (*id, self.obj(*id).controller))
                .collect();
            let destroyed: Vec<ObjectId> = to_destroy.clone();
            let res = self.move_objects(moves);
            let _ = res;
            for (id, p) in sac_ids {
                self.history.sacrificed.push((p, id));
                self.emit(crate::events::Event::Sacrificed { obj: id, player: p });
            }
            for id in destroyed {
                if !self.is_live(id) {
                    self.emit(crate::events::Event::Destroyed { obj: id });
                }
            }
        }

        for p in losers {
            let ctx = Ctx::new(None, p);
            let cant_lose = self
                .player_restricted(p, |r| matches!(r, Restriction::CantLoseGame(_)))
                || self
                    .player(p)
                    .has_mod(|m| matches!(m, PlayerModification::CantLoseGame));
            let _ = ctx;
            if cant_lose {
                continue;
            }
            for e in self.replace(ReplEvent::LoseGame { player: p }) {
                self.execute_repl_event(e);
            }
        }
        self.recompute();
        true
    }

    /// Whether the object is the source of a triggered ability that has triggered but not
    /// yet left the stack (CR 704.5s, 704.5v).
    pub fn is_source_of_stack_trigger(&self, id: ObjectId) -> bool {
        self.pending_triggers.iter().any(|t| t.source == id)
            || self.stack.iter().any(|s| {
                matches!(
                    self.obj(*s).stack.as_deref().map(|x| &x.kind),
                    Some(StackKind::Triggered { source, .. }) if *source == id
                )
            })
    }
}
