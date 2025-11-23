# Menu Items API Documentation

## Overview

The Menu Items API provides endpoints to manage menu items within a chef's menus. Menu items represent individual dishes that can be offered on a specific menu, including details like name, description, course type, and quantity available.

## Authentication

All endpoints require bearer token authentication. Include the JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

## Authorization

Menu item operations require the authenticated user to be the owner of the menu (the chef who created it) or have admin privileges.

## Endpoints

### 1. Create Menu Item

Creates a new menu item within a specific menu.

**Endpoint:** `POST /api/menus/{menu_id}/items`

**Required Headers:**
- `Authorization: Bearer <token>`
- `Content-Type: application/json`

**Path Parameters:**
- `menu_id` (UUID): The ID of the menu to add the item to

**Request Body:**

```json
{
  "name": "string (required)",
  "description": "string (optional)",
  "course_type": "string (optional, e.g., 'appetizer', 'main', 'dessert')",
  "image_url": "string (optional)",
  "is_featured": "boolean (optional, default: false)",
  "display_order": "integer (optional, default: 0)",
  "quantity": "integer (optional)"
}
```

**Example Request:**

```bash
curl -X POST "http://localhost:8080/api/menus/550e8400-e29b-41d4-a716-446655440000/items" \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pan-Seared Duck Breast",
    "description": "Succulent duck breast served with cherry gastrique and seasonal vegetables",
    "course_type": "main",
    "image_url": "https://example.com/duck.jpg",
    "is_featured": true,
    "display_order": 2,
    "quantity": 6
  }'
```

**Success Response (201 Created):**

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "menu_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Pan-Seared Duck Breast",
  "description": "Succulent duck breast served with cherry gastrique and seasonal vegetables",
  "course_type": "main",
  "image_url": "https://example.com/duck.jpg",
  "is_featured": true,
  "display_order": 2,
  "quantity": 6,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

**Error Responses:**

- `401 Unauthorized`: User does not own the menu
- `400 Bad Request`: Invalid request body
- `500 Internal Server Error`: Database error

---

### 2. Get Menu Items

Retrieves all menu items for a specific menu, ordered by display_order.

**Endpoint:** `GET /api/menus/{menu_id}/items`

**Required Headers:**
- `Authorization: Bearer <token>`

**Path Parameters:**
- `menu_id` (UUID): The ID of the menu

**Example Request:**

```bash
curl -X GET "http://localhost:8080/api/menus/550e8400-e29b-41d4-a716-446655440000/items" \
  -H "Authorization: Bearer eyJhbGc..."
```

**Success Response (200 OK):**

```json
[
  {
    "id": "660e8400-e29b-41d4-a716-446655440000",
    "menu_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Pan-Seared Duck Breast",
    "description": "Succulent duck breast served with cherry gastrique and seasonal vegetables",
    "course_type": "main",
    "image_url": "https://example.com/duck.jpg",
    "is_featured": true,
    "display_order": 2,
    "quantity": 6,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  },
  {
    "id": "770e8400-e29b-41d4-a716-446655440000",
    "menu_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Chocolate Soufflé",
    "description": "Light and airy chocolate soufflé with vanilla bean ice cream",
    "course_type": "dessert",
    "image_url": "https://example.com/souffle.jpg",
    "is_featured": false,
    "display_order": 1,
    "quantity": 8,
    "created_at": "2024-01-15T10:35:00Z",
    "updated_at": "2024-01-15T10:35:00Z"
  }
]
```

---

### 3. Update Menu Item

Updates specific fields of an existing menu item. Only provided fields are updated.

**Endpoint:** `PUT /api/menus/{menu_id}/items/{item_id}`

**Required Headers:**
- `Authorization: Bearer <token>`
- `Content-Type: application/json`

**Path Parameters:**
- `menu_id` (UUID): The ID of the menu
- `item_id` (UUID): The ID of the menu item to update

**Request Body (all fields optional):**

```json
{
  "name": "string",
  "description": "string",
  "course_type": "string",
  "image_url": "string",
  "is_featured": "boolean",
  "display_order": "integer",
  "quantity": "integer"
}
```

**Example Request:**

```bash
curl -X PUT "http://localhost:8080/api/menus/550e8400-e29b-41d4-a716-446655440000/items/660e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer eyJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "quantity": 8,
    "is_featured": false
  }'
```

**Success Response (200 OK):**

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "menu_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Pan-Seared Duck Breast",
  "description": "Succulent duck breast served with cherry gastrique and seasonal vegetables",
  "course_type": "main",
  "image_url": "https://example.com/duck.jpg",
  "is_featured": false,
  "display_order": 2,
  "quantity": 8,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:45:00Z"
}
```

**Error Responses:**

- `401 Unauthorized`: User does not own the menu
- `404 Not Found`: Menu item does not exist
- `400 Bad Request`: Invalid request body
- `500 Internal Server Error`: Database error

---

### 4. Delete Menu Item

Deletes a menu item from the menu.

**Endpoint:** `DELETE /api/menus/{menu_id}/items/{item_id}`

**Required Headers:**
- `Authorization: Bearer <token>`

**Path Parameters:**
- `menu_id` (UUID): The ID of the menu
- `item_id` (UUID): The ID of the menu item to delete

**Example Request:**

```bash
curl -X DELETE "http://localhost:8080/api/menus/550e8400-e29b-41d4-a716-446655440000/items/660e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer eyJhbGc..."
```

**Success Response (204 No Content):**

```
(empty response body)
```

**Error Responses:**

- `401 Unauthorized`: User does not own the menu
- `500 Internal Server Error`: Database error

---

## Data Models

### MenuItem

Complete menu item object returned from the API.

```typescript
interface MenuItem {
  id: UUID;                    // Unique identifier
  menu_id: UUID;              // Reference to parent menu
  name: string;               // Item name
  description: string | null; // Detailed description
  course_type: string | null; // e.g., "appetizer", "main", "dessert"
  image_url: string | null;   // URL to item image
  is_featured: boolean;       // Whether item is featured
  display_order: number;      // Display order for sorting
  quantity: number | null;    // Number of plates/servings available
  created_at: DateTime;       // Creation timestamp
  updated_at: DateTime;       // Last update timestamp
}
```

### CreateMenuItem

Request object for creating a new menu item.

```typescript
interface CreateMenuItem {
  name: string;               // Required
  description?: string;       // Optional
  course_type?: string;       // Optional
  image_url?: string;         // Optional
  is_featured?: boolean;      // Optional, default: false
  display_order?: number;     // Optional, default: 0
  quantity?: number;          // Optional
}
```

### UpdateMenuItem

Request object for updating a menu item. All fields are optional.

```typescript
interface UpdateMenuItem {
  name?: string;
  description?: string;
  course_type?: string;
  image_url?: string;
  is_featured?: boolean;
  display_order?: number;
  quantity?: number;
}
```

---

## Common Error Responses

### 401 Unauthorized

Returned when:
- No bearer token is provided
- Token is invalid or expired
- User does not own the menu

```json
{
  "error": "Not authorized"
}
```

### 404 Not Found

Returned when menu item does not exist.

```json
{
  "error": "Menu item not found"
}
```

### 400 Bad Request

Returned when request body is invalid or missing required fields.

```json
{
  "error": "Invalid request body"
}
```

---

## Usage Examples

### Complete Workflow

1. **Create a menu item:**

```bash
# Create menu item
RESPONSE=$(curl -X POST "http://localhost:8080/api/menus/$MENU_ID/items" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Beef Wellington",
    "description": "Tender beef loin wrapped in mushroom duxelles and pastry",
    "course_type": "main",
    "quantity": 4,
    "display_order": 1,
    "is_featured": true
  }')

ITEM_ID=$(echo $RESPONSE | jq -r '.id')
```

2. **Retrieve all items for the menu:**

```bash
curl -X GET "http://localhost:8080/api/menus/$MENU_ID/items" \
  -H "Authorization: Bearer $TOKEN" \
  | jq '.'
```

3. **Update the item:**

```bash
curl -X PUT "http://localhost:8080/api/menus/$MENU_ID/items/$ITEM_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "quantity": 6,
    "display_order": 2
  }'
```

4. **Delete the item:**

```bash
curl -X DELETE "http://localhost:8080/api/menus/$MENU_ID/items/$ITEM_ID" \
  -H "Authorization: Bearer $TOKEN"
```

---

## Implementation Notes

- **Ordering**: Menu items are returned ordered by `display_order` ascending, then by `created_at` ascending
- **Ownership Verification**: All mutations (create, update, delete) verify the user owns the menu
- **Timestamps**: `created_at` is set at creation and never changes; `updated_at` is set at creation and updated on each modification
- **Soft Deletes**: Menu items are hard deleted (permanently removed from database)
- **Cascading Deletes**: When a menu is deleted, all its menu items are automatically deleted
- **Logging**: All operations include debug and info level logging for monitoring and debugging

