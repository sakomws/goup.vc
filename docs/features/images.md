# Images

The images feature handles upload, validation, storage, and serving of user-uploaded images. It is built around a storage abstraction that allows the backend to switch between database storage and S3-compatible object storage without changing handler logic.

## Storage abstraction

`ocg-server/src/services/images.rs` defines the `ImageStorage` trait:

```rust
pub(crate) trait ImageStorage {
    async fn get(&self, file_name: &str) -> Result<Option<Image>>;
    async fn save(&self, image: &NewImage<'_>) -> Result<()>;
}
```

The shared trait object type is:

```rust
pub(crate) type DynImageStorage = Arc<dyn ImageStorage + Send + Sync>;
```

`mockall::automock` is derived on the trait in test builds, enabling unit tests for all handlers that depend on `DynImageStorage`.

### DbImageStorage

`ocg-server/src/services/images/db.rs` — stores image bytes and content-type directly in PostgreSQL via the existing `DynDB` abstraction (`db.get_image` / `db.save_image`). Used as the default storage backend when no S3 configuration is provided.

### S3ImageStorage

`ocg-server/src/services/images/s3.rs` — stores images in any S3-compatible bucket using the `aws-sdk-s3` crate. Configured via `ImageStorageConfigS3` (access key, secret, region, optional custom endpoint, optional path-style flag). Falls back to `mime_guess` for content-type when the provider does not return one.

## Image targets

The upload handler recognises a `target` form field that controls image handling:

| Target | Dimensions (px) |
|---|---|
| `banner` | 2428 × 192 |
| `banner_mobile` | 1220 × 192 |
| `logo` | Any source size; raster uploads are normalized into a 360 × 360 PNG |
| `open_graph` | 1200 × 630 |

Banner and Open Graph dimensions are validated after decoding using the `image` crate. Raster logos are proportionally resized, centered on a transparent 360 × 360 canvas, and stored as PNG without cropping.

## Supported formats

GIF, JPEG, PNG, SVG, TIFF, WebP. SVG files bypass raster-dimension validation because they are vector documents. Only JPEG, PNG, and WebP are accepted for Open Graph targets.

## Upload flow

1. The client sends a `multipart/form-data` POST to `/images` with fields `target` and `file`.
2. `ocg-server/src/handlers/images.rs` — `upload` handler — extracts and validates the fields. Non-logo uploads are limited to **1 MiB**; logo sources are limited to **10 MiB** and a decoded-pixel guard before normalization.
3. The referer header is checked against the configured hostname via `request_matches_site`; mismatches return `403 Forbidden`.
4. The handler computes a content hash of the bytes (via `util::compute_hash`) to produce a deterministic file name, avoiding duplicate storage.
5. The handler verifies banner/Open Graph dimensions and normalizes raster logos before storage.
6. The `NewImage` struct is handed to `DynImageStorage::save`.
7. A JSON body containing the resulting file name and public URL is returned to the client.

## Serving images

- `GET /images/:file_name` — `serve` handler. Validates the referer header then delegates to `DynImageStorage::get`. On success, returns the bytes with `Content-Type` and `Cache-Control: immutable` headers so browsers cache images indefinitely.
- `GET /og/:file_name` — `serve_open_graph` handler. Skips the referer check but first queries `db.is_open_graph_image` to confirm the image is currently designated as a public Open Graph preview. Returns `404` otherwise.

## Active contributors

Sako Mammadov, Sergio Castaño Arteaga
