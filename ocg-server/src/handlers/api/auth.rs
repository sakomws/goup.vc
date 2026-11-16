//! API bearer-token authentication helpers.

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

pub(crate) use ocg_core::shared::{ApiScope, ApiToken, hash_token};

use crate::{auth::User, handlers::api::error::ApiError, router};

/// Parsed and authenticated API caller.
#[derive(Debug, Clone)]
pub(crate) struct ApiUser {
    /// Authenticated user.
    pub user: User,
    /// Token scopes granted to the caller.
    pub scopes: Vec<ApiScope>,
}

impl ApiUser {
    /// Require a specific API scope.
    pub(crate) fn require_scope(&self, scope: ApiScope) -> Result<(), ApiError> {
        if self.scopes.contains(&scope) || self.scopes.contains(&ApiScope::AdminPlatform) {
            return Ok(());
        }
        Err(ApiError::forbidden())
    }

    /// Current user id.
    pub(crate) fn user_id(&self) -> Uuid {
        self.user.user_id
    }
}

impl From<ocg_core::shared::ApiUser> for ApiUser {
    fn from(value: ocg_core::shared::ApiUser) -> Self {
        Self {
            user: value.user,
            scopes: value.scopes,
        }
    }
}

impl FromRequestParts<router::State> for ApiUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &router::State,
    ) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get(AUTHORIZATION) else {
            return Err(ApiError::unauthenticated());
        };
        let header = header.to_str().map_err(|_| ApiError::unauthenticated())?;
        let Some(token) = header.strip_prefix("Bearer ") else {
            return Err(ApiError::unauthenticated());
        };
        let token_hash = hash_token(token);

        state
            .db
            .get_api_token_auth(&token_hash)
            .await?
            .map(Self::from)
            .ok_or_else(ApiError::unauthenticated)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ApiScope, hash_token};

    #[test]
    fn hash_token_is_stable_and_not_plaintext() {
        let hash = hash_token("goup_test_token");

        assert_eq!(
            hash,
            "8a097b2bf9772e4086101d9ebb6791be85bb23d639bdc9915174f5bdd97f920c"
        );
        assert_ne!(hash, "goup_test_token");
    }

    #[test]
    fn api_scope_parses_persisted_scope_strings() {
        assert_eq!(
            ApiScope::from_str("admin:alliance").expect("valid scope"),
            ApiScope::AdminAlliance
        );
        assert!(ApiScope::from_str("admin:unknown").is_err());
    }
}
