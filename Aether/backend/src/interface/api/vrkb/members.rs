use crate::domain::models::VrkbMember;
use crate::domain::ports::VrkbRepository;
use crate::interface::api::auth::AuthenticatedUser;
use crate::interface::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- VRKB-08: Role definitions ---
const VALID_ROLES: &[&str] = &["owner", "lead", "researcher", "observer"];

// --- VRKB-08: Permission definitions ---
#[derive(Debug, Serialize, Clone)]
struct Permission {
    action: String,
    allowed_roles: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PermissionMatrix {
    roles: Vec<String>,
    permissions: Vec<Permission>,
}

#[derive(Debug, Serialize)]
struct MemberPermissions {
    user_id: Uuid,
    role: String,
    permissions: Vec<String>,
}

fn get_permission_matrix() -> PermissionMatrix {
    PermissionMatrix {
        roles: VALID_ROLES.iter().map(|s| s.to_string()).collect(),
        permissions: vec![
            Permission {
                action: "create_finding".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string()],
            },
            Permission {
                action: "update_finding".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string()],
            },
            Permission {
                action: "change_severity".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string()],
            },
            Permission {
                action: "manage_members".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string()],
            },
            Permission {
                action: "export_report".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string()],
            },
            Permission {
                action: "delete_project".to_string(),
                allowed_roles: vec!["owner".to_string()],
            },
            Permission {
                action: "manage_docs".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string()],
            },
            Permission {
                action: "view_audit_log".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string()],
            },
            Permission {
                action: "manage_assets".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string()],
            },
            Permission {
                action: "view_project".to_string(),
                allowed_roles: vec!["owner".to_string(), "lead".to_string(), "researcher".to_string(), "observer".to_string()],
            },
        ],
    }
}

fn get_permissions_for_role(role: &str) -> Vec<String> {
    let matrix = get_permission_matrix();
    matrix
        .permissions
        .iter()
        .filter(|p| p.allowed_roles.iter().any(|r| r == role))
        .map(|p| p.action.clone())
        .collect()
}

/// RBAC permission check helper — looks up current user's role in the project
/// and verifies they have the required action permission.
async fn check_permission(
    state: &AppState,
    project_id: &Uuid,
    user: &AuthenticatedUser,
    action: &str,
) -> Result<(), (StatusCode, String)> {
    let members = state
        .repo
        .list_members(project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let member = members
        .iter()
        .find(|m| m.user_id == user.id)
        .ok_or((
            StatusCode::FORBIDDEN,
            "You are not a member of this project".to_string(),
        ))?;

    let permissions = get_permissions_for_role(&member.role);
    if !permissions.contains(&action.to_string()) {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Role '{}' does not have '{}' permission", member.role, action),
        ));
    }

    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/vrkb/projects/:id/members",
            get(list_members).post(add_member),
        )
        .route(
            "/api/vrkb/projects/:id/members/:uid",
            delete(remove_member).put(update_member),
        )
        // --- VRKB-08: Permission endpoints ---
        .route(
            "/api/vrkb/projects/:id/members/:uid/permissions",
            get(get_member_permissions),
        )
        .route(
            "/api/vrkb/projects/:id/permissions",
            get(get_project_permission_matrix),
        )
}

#[derive(Deserialize)]
struct AddMemberRequest {
    user_id: Uuid,
    role: String,
}

#[derive(Deserialize)]
struct UpdateMemberRequest {
    role: String,
}

async fn list_members(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<VrkbMember>>, (StatusCode, String)> {
    let members = state
        .repo
        .list_members(&project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(members))
}

async fn add_member(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // VRKB-08: Check manage_members permission via RBAC
    check_permission(&state, &project_id, &user, "manage_members").await?;

    // VRKB-08: Validate role
    let role = payload.role.to_lowercase();
    if !VALID_ROLES.contains(&role.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid role '{}'. Valid roles: {:?}", role, VALID_ROLES),
        ));
    }

    let member = VrkbMember {
        project_id,
        user_id: payload.user_id,
        role,
        joined_at: Utc::now(),
        user: None,
    };
    state
        .repo
        .add_member(member)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::CREATED)
}

async fn remove_member(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // VRKB-08: Check manage_members permission
    check_permission(&state, &project_id, &user, "manage_members").await?;

    // Prevent removing the sole owner
    let members = state.repo.list_members(&project_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let target_member = members.iter().find(|m| m.user_id == user_id);
    if let Some(m) = target_member {
        if m.role == "owner" {
            let owner_count = members.iter().filter(|m| m.role == "owner").count();
            if owner_count <= 1 {
                return Err((StatusCode::BAD_REQUEST, "Cannot remove the sole owner".to_string()));
            }
        }
    }

    state
        .repo
        .remove_member(&project_id, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

async fn update_member(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemberRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // VRKB-08: Check manage_members permission
    check_permission(&state, &project_id, &user, "manage_members").await?;

    // VRKB-08: Validate role
    let role = payload.role.to_lowercase();
    if !VALID_ROLES.contains(&role.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid role '{}'. Valid roles: {:?}", role, VALID_ROLES),
        ));
    }

    state
        .repo
        .update_member_role(&project_id, &user_id, role)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

// --- VRKB-08: Get permissions for a specific member ---
async fn get_member_permissions(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MemberPermissions>, (StatusCode, String)> {
    let members = state
        .repo
        .list_members(&project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let member = members
        .iter()
        .find(|m| m.user_id == user_id)
        .ok_or((StatusCode::NOT_FOUND, "Member not found".to_string()))?;

    let permissions = get_permissions_for_role(&member.role);

    Ok(Json(MemberPermissions {
        user_id: member.user_id,
        role: member.role.clone(),
        permissions,
    }))
}

// --- VRKB-08: Get full permission matrix for a project ---
async fn get_project_permission_matrix(
    _user: AuthenticatedUser,
) -> Json<PermissionMatrix> {
    Json(get_permission_matrix())
}
