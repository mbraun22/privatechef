use askama::Template;
use crate::models::{UserResponse, Menu, MenuItem};

// Home page template
#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub user: Option<UserResponse>,
}

// Login page template
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub user: Option<UserResponse>,
    pub is_register: bool,
    pub error: Option<String>,
    pub loading: bool,
}

// Dashboard template
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub user: Option<UserResponse>,
}

// Helper struct for menu with items (for template rendering)
#[derive(Clone)]
pub struct MenuWithItems {
    pub menu: Menu,
    pub items: Vec<MenuItem>,
}

// Chef Dashboard template
#[derive(Template)]
#[template(path = "chef_dashboard.html")]
pub struct ChefDashboardTemplate {
    pub user: Option<UserResponse>,
    pub chef: Option<crate::models::Chef>,
    pub menus_with_items: Vec<MenuWithItems>,
    pub error: Option<String>,
    pub success: Option<String>,
}
