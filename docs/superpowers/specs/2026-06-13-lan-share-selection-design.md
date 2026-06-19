# Android LAN Share Selection Design

## Status

Approved direction for a fast Android-first implementation on 2026-06-13.

## Goal

Let the photographer share a filtered project photo set from the Android app to
the local network. The app shows one LAN URL containing one token. One guest
operator opens that URL in a browser, reviews the shared photos, and records a
guest-only mark per photo.

The result is visible back in the photographer's project photo view as an extra
badge/state. Guest marks do not change photographer favorites, photographer
marks, model recommendations, model scores, or files on disk.

## Scope

First version:

- Android app acts as the LAN host.
- One active share URL has one token and one guest operator.
- Share source is the current project plus the photographer's selected
  collection, format filter, and sort order.
- Guest can set one mark per asset group:
  - `favorite`
  - `marked`
  - `reject`
- Guest can clear the current mark, returning it to no mark.
- No guest result is shown when no guest mark exists.
- Android project photo data exposes `guest_mark` alongside photographer
  `user_marks`.

Out of scope:

- Multiple guest identities.
- Merging competing guest results.
- Internet access, relay, HTTPS, or cloud sync.
- Real deletion of project files from guest actions.
- Custom photographer tag taxonomy.
- Sharing history comparison across many links.

## Terms

### LAN Share Session

A token-scoped local sharing session for one project and one saved asset query.
The token is required in the URL and API calls.

### Guest Mark

A guest-only selection result attached to one asset group within one share
session. `reject` means the guest suggests eliminating the photo; it must never
delete files.

## Core Responsibilities

Core should own share semantics and storage. This layer must stay platform
neutral so the same LAN share model can later be used by CLI, desktop, NAS, or
another mobile shell:

- Create and persist LAN share sessions.
- Generate unguessable share tokens.
- Store the project id, query, sort, creation time, and active state.
- Query share assets with existing `AssetGroupQuery` semantics.
- Store and clear `guest_mark` values per `share_id` and `asset_group_id`.
- Expose `guest_mark` in the project asset read model when present.

Core should not own Android-specific networking or storage stream behavior.

## Android Responsibilities

Android should own LAN hosting and browser delivery. The Android-specific
customization should be limited to the adapter edges: socket listening,
foreground/lifecycle behavior, Android storage streams, and Compose entry
points:

- Start and stop a local HTTP service bound to a LAN-reachable host and port.
- Keep the share service alive while enabled.
- Show the user the LAN URL, for example `http://<lan-ip>:<port>/s/<token>`.
- Serve the guest web UI.
- Serve preview images by reusing Android preview loading and thumbnail cache
  behavior.
- Route guest mark API calls into core.
- Surface active share state and guest results in the project photo screen.

## Layering Rule

Keep the interface and lower layers as reusable as possible:

- Core defines the durable model, validation, token lookup, query behavior, and
  guest mark semantics.
- FFI/mobile gateway exposes platform-neutral JSON operations over that core
  model.
- The guest web API shape should not depend on Compose or Android UI state.
- Android is the first host implementation, but its custom layer should mostly
  be the local HTTP listener plus Android preview/file streaming.
- If a future desktop or CLI host is added, it should reuse the same core and
  gateway operations and only replace the HTTP/file-stream adapter.

## Data Model

Suggested core tables:

```text
lan_share_sessions
  share_id TEXT PRIMARY KEY
  project_id TEXT NOT NULL
  token TEXT NOT NULL UNIQUE
  query_json TEXT NOT NULL
  title TEXT
  active INTEGER NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL
  stopped_at_ms INTEGER

lan_share_guest_marks
  share_id TEXT NOT NULL
  project_id TEXT NOT NULL
  asset_group_id TEXT NOT NULL
  guest_mark TEXT NOT NULL
  updated_at_ms INTEGER NOT NULL
  PRIMARY KEY (share_id, asset_group_id)
```

Valid `guest_mark` values are `favorite`, `marked`, and `reject`.

The read model should expose:

```json
{
  "user_marks": { "favorite": true, "marked": false },
  "guest_mark": "reject"
}
```

`guest_mark` should be omitted or `null` when no guest result exists.

## API Shape

Android HTTP endpoints can be adapter-owned, but should map to core operations:

```text
GET  /s/{token}
GET  /api/s/{token}/assets
GET  /api/s/{token}/assets/{asset_group_id}
GET  /api/s/{token}/preview/{asset_group_id}
PUT  /api/s/{token}/assets/{asset_group_id}/guest-mark
```

The update body:

```json
{ "guest_mark": "favorite" }
```

Clearing:

```json
{ "guest_mark": null }
```

Invalid tokens, stopped sessions, missing assets, and invalid marks should
return explicit errors and never modify photographer marks.

## Guest UI

The guest page is a simple browser grid:

- Photo preview.
- Filename or group title.
- Optional model score or recommendation badge when available.
- Existing photographer favorite/marked indicators may be shown as context.
- Guest action controls: favorite, marked, reject.
- Clicking the currently selected guest action clears it.

Guest marks are visually distinct from photographer marks.

## Photographer UI

The Android project photo screen should add a share action near the photo list
controls or selection action bar. The user flow:

1. Photographer chooses collection/filter/sort.
2. Photographer starts LAN share.
3. App shows URL and active share status.
4. Guest marks photos in browser.
5. Photographer sees guest marks appear as badges on project photo tiles and
   detail view.
6. Photographer stops the share when done.

## Error Handling

- Port binding failure shows a clear Android error and does not create a
  misleading active URL.
- Token mismatch or inactive session returns unauthorized/not found.
- Preview decode failure returns a placeholder state for that asset.
- Guest mark write failure keeps the browser state unchanged and shows an
  inline failure.
- Guest `reject` never calls project delete APIs.

## Testing

Core tests:

- Create share session stores project/query/token.
- Share asset query reuses collection, format, and sort semantics.
- Set `guest_mark` to each valid value.
- Clear `guest_mark`.
- Reject invalid `guest_mark`.
- Guest marks do not mutate `AssetUserMarks`.

Android unit tests:

- Guest mark JSON maps to and from core gateway models.
- Project asset mapping exposes `guestMark`.
- UI model shows no guest badge for null and shows each valid mark.
- Guest reject action does not call delete APIs.

Manual validation:

- Start share on Android.
- Open URL from another device on the same LAN.
- Mark and clear a photo.
- Confirm Android project view updates without changing photographer
  favorite/marked state.
