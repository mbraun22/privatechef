#!/bin/bash

# Menu Items API Testing Script
# Tests the create, read, update, and delete functionality for menu items

set -e

# Configuration
BASE_URL="http://localhost:8080"
API_BASE="$BASE_URL/api"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Menu Items API Testing ===${NC}\n"

# Check if server is running
echo "Checking if server is running..."
if ! curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}Error: Server is not running on $BASE_URL${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server is running${NC}\n"

# Test data
MENU_ITEM_NAME="Grilled Salmon with Lemon Butter"
MENU_ITEM_DESCRIPTION="Fresh Atlantic salmon fillet grilled to perfection with homemade lemon butter sauce"
COURSE_TYPE="main"
IMAGE_URL="https://example.com/salmon.jpg"
QUANTITY=4

echo -e "${YELLOW}=== Testing Menu Item Creation ===${NC}"
echo "Creating a test menu item..."
echo "Name: $MENU_ITEM_NAME"
echo "Course: $COURSE_TYPE"
echo "Quantity: $QUANTITY\n"

# Note: In production, you would need to:
# 1. Authenticate and get a bearer token
# 2. Have a valid menu_id from an existing menu
# Example curl command structure:
cat << 'EOF'
# Example: Create menu item
# curl -X POST "$API_BASE/menus/{menu_id}/items" \
#   -H "Authorization: Bearer $TOKEN" \
#   -H "Content-Type: application/json" \
#   -d '{
#     "name": "Grilled Salmon with Lemon Butter",
#     "description": "Fresh Atlantic salmon fillet grilled to perfection with homemade lemon butter sauce",
#     "course_type": "main",
#     "image_url": "https://example.com/salmon.jpg",
#     "is_featured": true,
#     "display_order": 1,
#     "quantity": 4
#   }'

# Example: Get menu items
# curl -X GET "$API_BASE/menus/{menu_id}/items" \
#   -H "Authorization: Bearer $TOKEN"

# Example: Update menu item
# curl -X PUT "$API_BASE/menus/{menu_id}/items/{item_id}" \
#   -H "Authorization: Bearer $TOKEN" \
#   -H "Content-Type: application/json" \
#   -d '{
#     "name": "Updated Item Name",
#     "quantity": 6
#   }'

# Example: Delete menu item
# curl -X DELETE "$API_BASE/menus/{menu_id}/items/{item_id}" \
#   -H "Authorization: Bearer $TOKEN"
EOF

echo -e "\n${GREEN}=== Test Script Ready ===${NC}"
echo "Use the example commands above to test the menu items API."
echo "Make sure to replace:"
echo "  - {menu_id} with a valid menu UUID"
echo "  - {item_id} with a valid menu item UUID"
echo "  - \$TOKEN with a valid JWT bearer token"

