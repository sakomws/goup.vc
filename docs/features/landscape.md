# Landscape

**Active contributors:** Sergio Castaño Arteaga, Cintia Sánchez García, Sako Mammadov

## Purpose

The landscape feature provides a directory of startups, open source projects, partner communities, and podcast leads associated with an alliance. Entries are browsable on the public site and searchable via the platform search and MCP tools. New entries can be submitted through the site or created directly via the MCP server.

## Directory layout

```
ocg-server/src/
├── handlers/site/landscape.rs      # public landscape directory handler
├── handlers/dashboard/             # dashboard handlers for managing landscape entries (admin)
├── db/landscape.rs                 # DB queries: get_landscape_entries, add_entry, update_entry
├── templates/site/                 # MiniJinja template structs for landscape pages
└── types/landscape.rs              # LandscapeEntry, LandscapeKind, and related types
```

## Key abstractions

| Abstraction | File | Description |
|-------------|------|-------------|
| `DBLandscape` | `ocg-server/src/db/landscape.rs` | Trait: `get_landscape_entries`, `add_landscape_entry`, `update_landscape_entry` |
| `LandscapeKind` | `ocg-server/src/types/landscape.rs` | Enum: `startup`, `github_project`, `partner_community`, `podcast_lead` |

## How it works

The landscape directory is a straightforward CRUD feature:

1. **Public browsing** — `GET /:alliance_name/landscape` is handled by `ocg-server/src/handlers/site/landscape.rs`, which queries `DBLandscape::get_landscape_entries` filtered by alliance and renders the MiniJinja template.
2. **Submission** — Users can submit new entries through a form. Submissions may be moderated before appearing publicly.
3. **Admin management** — Alliance admins can approve, edit, and delete entries through the dashboard.
4. **MCP creation** — The MCP server exposes `goup_create_startup` and `goup_create_github_project` tools that insert entries directly via the database.

## Landscape entry kinds

| Kind | Description |
|------|-------------|
| `startup` | Early-stage companies associated with the alliance |
| `github_project` | Open source projects hosted on GitHub |
| `partner_community` | Partner organizations or communities |
| `podcast_lead` | Podcasts relevant to the community |

## Integration points

- [Groups and alliances](groups-and-alliances.md) — landscape entries are scoped to an alliance.
- [MCP server](../services/mcp-server.md) — `goup_search_landscape`, `goup_create_startup`, `goup_create_github_project` tools.
- Site search (`handlers/site/search.rs`) includes landscape entries in global search results.

## Entry points for modification

- Add a new landscape kind: add a variant to `LandscapeKind` in `ocg-server/src/types/landscape.rs`, update the DB query in `ocg-server/src/db/landscape.rs`, and add a migration if needed.
- Add a new landscape field: extend the entry struct in `ocg-server/src/types/landscape.rs` and update the DB layer.
- Change the public view: edit `ocg-server/src/handlers/site/landscape.rs` and the corresponding MiniJinja template.
