# Battlefield — Kafka Chaos Mini-Game

A medieval siege mini-game where you aim and fire chaos actions at Kafka topics rendered as buildings on a battlefield canvas.

## Core Concept

Topics are medieval structures (castles, towers, keeps) arranged on the right side of a canvas scene. The player's siege camp with weapons sits on the left. You select a weapon via WoW-style hotbar buttons (keybinds `1` and `2`), aim by clicking and dragging to set a trajectory arc, and release to fire. Projectiles animate along the arc. Hits execute real chaos actions against the Kafka cluster. Misses waste the weapon's cooldown.

## Weapons (MVP: 2)

| Slot | Keybind | Weapon    | Chaos Action  | Cooldown | Projectile Visual         |
|------|---------|-----------|---------------|----------|---------------------------|
| 1    | `1`     | Crossbow  | Poison Pills  | ~3s      | Bolt with green poison trail |
| 2    | `2`     | Trebuchet | Delete Topic  | ~6s      | Flaming boulder            |

## Architecture

### Rendering: Canvas + DOM Overlay

- **Canvas** handles the game scene: terrain, buildings, projectiles, impact effects, trajectory preview
- **DOM overlay** handles the UI chrome: action bar, cooldown animations, HUD, toasts
- All game state lives in Dioxus signals so both layers stay in sync

### File Structure

```
console/src/app/features/battlefield/
├── mod.rs              — module exports
├── page.rs             — BattlefieldPage component (route: /battlefield)
├── canvas/
│   ├── mod.rs          — canvas game loop + render orchestration
│   ├── scene.rs        — terrain, buildings, background rendering
│   ├── projectile.rs   — projectile arc calculation, flight animation
│   └── aiming.rs       — drag-to-aim input handling, trajectory preview
├── action_bar.rs       — WoW-style hotbar (DOM overlay)
├── cooldown.rs         — radial cooldown sweep animation
└── state.rs            — battlefield game state
```

### Data Flow

1. `BattlefieldPage` reads topics from `TopicsState` (populated by existing SSE subscription)
2. Topics are mapped to `BuildingTarget` structs with position, visual state, and hitbox
3. Canvas game loop reads `BattlefieldState` signal each frame via `requestAnimationFrame`
4. Mouse/keyboard events update `BattlefieldState` (weapon selection, aim vector, fire trigger)
5. On hit, the component calls existing `SiegeClient` chaos endpoints
6. SSE events (`ChaosPoisonPillsSent`, `TopicDeleted`, etc.) update `TopicsState`, which flows back into building visuals

## Scene

### Terrain

- Static medieval landscape on canvas
- Green ground with slight elevation variation, distant hill line, sky gradient
- Left side: player's siege camp (weapon placements)
- Right side: topic buildings (enemy fortification)

### Topic Buildings

- Each topic rendered as a medieval structure at a deterministic position based on list order
- Topic name displayed above each building
- Three visual states:
  - **Healthy** — intact structure, neutral colors
  - **Damaged** — visual indicator of applied chaos (e.g., green poison cloud for poison pills)
  - **Destroyed** — rubble pile (topic was deleted)
- Layout adapts when topics are added/removed via SSE
- If many topics, arrange in rows with depth (further rows appear smaller/higher)

### Hitboxes

- Rectangular AABB per building
- Slightly generous sizing (forgiving aim) but small enough that you can miss between buildings
- Collision checked once at projectile arc endpoint, not per-frame

## Aiming & Firing

### Selection

- Press `1` or `2` (or click the hotbar button) to select a weapon
- Selected weapon gets a golden border glow
- Cursor changes to crosshair over the canvas
- Cannot select a weapon that is on cooldown

### Aiming

- Click and drag on the canvas to aim
- Mouse position determines the landing target; the trajectory arc connects the weapon to the cursor
- Trajectory preview: dotted parabolic curve rendered as a quadratic bezier (weapon → arc peak → cursor)
- Arc peak height is proportional to the horizontal distance between weapon and cursor
- Arc color matches the weapon (red-orange for trebuchet, green for crossbow)
- Press `Escape` or right-click to cancel without firing

### Firing

- Release mouse to fire
- Projectile animates along the bezier arc (~0.5-1 second flight time)
- On hit: chaos API call, impact visual effect (flash + particles), building transitions state
- On miss: projectile hits ground, dust puff, cooldown consumed

### Cooldowns

- Radial sweep animation on the hotbar button (WoW-style clock fill from 12 o'clock)
- Button appears grayed/dimmed during cooldown
- Hotkey press ignored during cooldown
- Cooldown is cosmetic pacing — does not wait for the Kafka API response
- Cooldown consumed on miss (you fired, it's gone)

## Action Bar & HUD (DOM Overlay)

### Action Bar (bottom center)

- Two square buttons, dark background, ornate medieval border
- Each button displays: weapon icon, keybind number in corner, chaos action name
- Radial cooldown overlay when on cooldown
- Selected weapon: golden border glow

### Top Bar (minimal)

- "The Battlefield" title
- Target count: "6 targets standing" — decrements on delete
- Back navigation to topics page

### Hit Feedback

- Successful hit: toast notification via existing Toaster — e.g., "Poison pills launched at 'orders' — 10 messages sent"
- Miss: toast — "The bolt lands in the dirt. Wasted."

## Integration

### Routing

- New route `/battlefield` added to `Route` enum in `routes.rs`
- Navigation link added to sidebar alongside Wheel of Chaos

### API Reuse

- No new backend endpoints
- Crossbow hit calls `topic.poison_pills(10)` via `SiegeClient`
- Trebuchet hit calls `topic.delete()` via `SiegeClient`
- SSE events update `TopicsState` which updates building visuals

### Canvas Technical Details

- `web-sys` for `CanvasRenderingContext2d`
- `requestAnimationFrame` loop via `web-sys` or `gloo`
- Loop idles (skips redraws) when no animation is active
- Aiming preview: ~20-30 dots along the bezier curve

### Error Handling

- API failure: building stays in current state, toast shows error
- Cooldown consumed regardless of API success

## Scope Boundaries (Not in MVP)

- No sound effects
- No additional weapons beyond the initial two
- No multi-projectile or area-of-effect
- No mobile/touch support
- No persistence of battlefield state across page navigation
- No score system or health bars
