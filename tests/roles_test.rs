// Integration tests for role-based access control

use privatechefspace_backend::models::{Role, User, CreateUser, UserResponse};
use std::str::FromStr;

#[test]
fn test_role_display() {
    assert_eq!(Role::Admin.to_string(), "admin");
    assert_eq!(Role::Mod.to_string(), "mod");
    assert_eq!(Role::Chef.to_string(), "chef");
    assert_eq!(Role::Diner.to_string(), "diner");
}

#[test]
fn test_role_from_str() {
    assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
    assert_eq!(Role::from_str("ADMIN").unwrap(), Role::Admin);
    assert_eq!(Role::from_str("mod").unwrap(), Role::Mod);
    assert_eq!(Role::from_str("chef").unwrap(), Role::Chef);
    assert_eq!(Role::from_str("diner").unwrap(), Role::Diner);
    assert!(Role::from_str("invalid").is_err());
}

#[test]
fn test_role_default() {
    assert_eq!(Role::default(), Role::Diner);
}

#[test]
fn test_role_serialization() {
    use serde_json;
    
    // Test serialization
    let role = Role::Admin;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"admin\"");
    
    // Test deserialization
    let role: Role = serde_json::from_str("\"admin\"").unwrap();
    assert_eq!(role, Role::Admin);
    
    let role: Role = serde_json::from_str("\"chef\"").unwrap();
    assert_eq!(role, Role::Chef);
}

#[test]
fn test_user_response_includes_role() {
    use chrono::Utc;
    use uuid::Uuid;
    
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hash".to_string(),
        role: Role::Chef,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let response: UserResponse = user.into();
    assert_eq!(response.role, Role::Chef);
}

#[test]
fn test_create_user_defaults_to_diner() {
    let create_user = CreateUser {
        email: "test@example.com".to_string(),
        password: "password".to_string(),
        role: None,
    };
    
    assert_eq!(create_user.role, None);
    
    // In actual registration, we default to Diner if None
    let role = create_user.role.unwrap_or(Role::Diner);
    assert_eq!(role, Role::Diner);
}

#[test]
fn test_role_permissions() {
    use privatechefspace_backend::middleware::roles::has_permission;
    
    // Admin can do everything
    assert!(has_permission(Role::Admin, "manage_content"));
    assert!(has_permission(Role::Admin, "manage_users"));
    assert!(has_permission(Role::Admin, "view_reports"));
    assert!(has_permission(Role::Admin, "manage_own_chef_profile"));
    
    // Mod can manage content and users
    assert!(has_permission(Role::Mod, "manage_content"));
    assert!(has_permission(Role::Mod, "manage_users"));
    assert!(has_permission(Role::Mod, "view_reports"));
    assert!(!has_permission(Role::Mod, "manage_own_chef_profile"));
    
    // Chef can manage own content
    assert!(has_permission(Role::Chef, "manage_own_chef_profile"));
    assert!(has_permission(Role::Chef, "manage_own_menus"));
    assert!(has_permission(Role::Chef, "manage_own_bookings"));
    assert!(!has_permission(Role::Chef, "manage_content"));
    
    // Diner can view and book
    assert!(has_permission(Role::Diner, "view_chefs"));
    assert!(has_permission(Role::Diner, "create_booking"));
    assert!(has_permission(Role::Diner, "view_own_bookings"));
    assert!(!has_permission(Role::Diner, "manage_content"));
}

