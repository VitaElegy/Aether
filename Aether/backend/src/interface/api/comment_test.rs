
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt; // for oneshot
    use crate::domain::{
        ports::{AuthService, AuthError, CommentRepository, RepositoryError},
        models::{AuthClaims, Comment, CommentId, CommentableId},
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // --- Mocks ---

    struct MockAuthService;
    #[async_trait]
    impl AuthService for MockAuthService {
        async fn authenticate(&self, _u: &str, _p: &str) -> Result<AuthClaims, AuthError> { panic!("Unused") }
        fn verify_token(&self, token: &str) -> Result<AuthClaims, AuthError> {
            if token == "valid_token" {
                Ok(AuthClaims {
                    sub: Uuid::new_v4().to_string(),
                    exp: 9999999999,
                    perms: 0,
                })
            } else {
                Err(AuthError::InvalidToken)
            }
        }
        fn generate_token(&self, _c: &AuthClaims) -> Result<String, AuthError> { panic!("Unused") }
    }

    struct MockCommentRepo;
    #[async_trait]
    impl CommentRepository for MockCommentRepo {
        async fn add_comment(&self, c: Comment) -> Result<CommentId, RepositoryError> {
            Ok(c.id)
        }
        async fn get_comments(&self, _t: &CommentableId) -> Result<Vec<Comment>, RepositoryError> { Ok(vec![]) }
        async fn get_comments_batch(&self, _t: &[CommentableId]) -> Result<Vec<Comment>, RepositoryError> { Ok(vec![]) }
    }

    // --- Test ---

    #[tokio::test]
    async fn test_create_comment_success() {
        let auth_service = Arc::new(MockAuthService);
        let repo = Arc::new(MockCommentRepo);
        
        // We need AppState structure but unfortunately AppState struct has fields we can't easily mock validation on 
        // unless we construct it fully.
        // However, the handler only extracts `State(repo)` and `AuthenticatedUser`.
        // `AuthenticatedUser` extracts `Arc<dyn AuthService>` from state.
        // So we can define a smaller state or just use the exact AppState if we mock everything.
        // OR we can rely on `axum::extract::FromRef` impls.
        
        // Let's create a minimal test AppState that implements FromRef for the needed parts.
        #[derive(Clone)]
        struct TestState {
            auth: Arc<dyn AuthService>,
            repo: Arc<dyn CommentRepository>,
        }
        
        impl axum::extract::FromRef<TestState> for Arc<dyn AuthService> {
            fn from_ref(state: &TestState) -> Self { state.auth.clone() }
        }
        
        impl axum::extract::FromRef<TestState> for Arc<dyn CommentRepository> {
            fn from_ref(state: &TestState) -> Self { state.repo.clone() }
        }

        let state = TestState {
            auth: auth_service,
            repo: repo,
        };

        let app = Router::new()
            .route("/api/comments/:type/:id", post(create_comment_handler))
            .with_state(state);

        // 1. Valid Request
        let response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/comments/Content/{}", Uuid::new_v4()))
                .header("Authorization", "Bearer valid_token")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "Hello"}"#))
                .unwrap()
        ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // 2. Invalid Token
        let response = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/comments/Content/{}", Uuid::new_v4()))
                .header("Authorization", "Bearer invalid")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "Hello"}"#))
                .unwrap()
        ).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

         // 3. Missing Header
        let response = app.oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/comments/Content/{}", Uuid::new_v4()))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "Hello"}"#))
                .unwrap()
        ).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
